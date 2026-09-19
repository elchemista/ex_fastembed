defmodule ExFastembed do
  @moduledoc """
  Loads FastEmbed text embedding and reranker models through `fastembed-rs`.

  Use `load/1` followed by `embed_text/1` to generate vectors, or
  `load_reranker/1` followed by `rerank/3` to score documents for a query.

  Each BEAM VM shares one embedding model and one independent reranker. Loading
  a replacement changes the model used by all processes; a failed load preserves
  the previous model. Loading, inference, and unloading within each family run
  serially. Applications own their request queues and must stop submitting work
  before unloading or deleting a model.

  Model files are downloaded on first use and cached in `.fastembed_cache`.
  Set `FASTEMBED_CACHE_DIR` before loading a model to use another directory.
  If `HF_HOME` is exported when the VM starts, FastEmbed uses that directory
  instead. Partial downloads reuse the cached revision for consistent model files.
  Loading and inference block the caller while the native work runs on a dirty
  scheduler, allowing other BEAM processes to continue running.

  All public functions return error tuples for invalid input. Strings must be
  valid UTF-8. Model names are matched case-insensitively against the bundled
  FastEmbed catalog, including the legacy aliases.

  See the [model lifecycle guide](model_lifecycle.html) for memory, disk usage,
  cache configuration, and deletion examples.
  """

  alias ExFastembed.Native

  @typedoc "A dense vector with the dimension reported by `load/1`."
  @type embedding :: [float()]

  @typedoc "A zero-based document index, relevance score, and optional document text."
  @type rerank_result :: {non_neg_integer(), float(), String.t() | nil}

  @typedoc "An input, model-loading, or inference failure with a human-readable reason."
  @type error :: {:error, String.t()}

  @typedoc "The two supported model families."
  @type model_kind :: :embedding | :reranker

  @typedoc "A model variant with runtime state, local file paths, and disk byte sizes."
  @type model_info :: %{
          name: String.t(),
          kind: model_kind(),
          repository: String.t(),
          dimension: pos_integer() | nil,
          cached: boolean(),
          loaded: boolean(),
          cache_dir: String.t(),
          path: String.t(),
          revision: String.t() | nil,
          files: [String.t()],
          file_details: [model_file()],
          variant_bytes: non_neg_integer() | nil,
          disk_bytes: non_neg_integer() | nil
        }

  @typedoc "A required file, its snapshot path when present, and its non-empty size in bytes."
  @type model_file :: %{
          name: String.t(),
          path: String.t() | nil,
          size_bytes: pos_integer() | nil
        }

  @doc """
  Returns the absolute effective cache root without creating it.

  Respects `HF_HOME` from the native process environment at VM startup, otherwise
  `FASTEMBED_CACHE_DIR`, with `.fastembed_cache` as the default. Existing roots
  are canonicalized, resolving symlinks.
  """
  @doc group: :discovery
  @spec cache_directory() :: String.t()
  def cache_directory, do: Native.cache_directory(cache_dir())

  @doc """
  Lists the models currently held by the native runtime, with their original cache paths.

  Returns `{:ok, []}` when nothing is loaded. Unlike `models/0`, this also finds
  models loaded from a different `FASTEMBED_CACHE_DIR` before it was changed.
  Metadata is a snapshot and can change immediately under concurrent calls.
  """
  @doc group: :discovery
  @spec loaded_models() :: {:ok, [model_info()]} | error()
  def loaded_models, do: Native.loaded_models()

  @doc """
  Unloads the shared embedding model while keeping downloaded files.

  Waits for an active load or inference, then drops the ONNX session, weights,
  and its owned native buffers. Returns `{:ok, true}` even if already unloaded.
  The reranker is independent and remains available.

  Stop submitting model work and drain application queues before calling this
  function. The library does not manage or cancel jobs; concurrent calls are
  serialized without an ordering guarantee. A subsequent load can recreate the
  session. Allocators may retain freed memory, so process RSS need not immediately
  decrease by the size of the model. Already returned embeddings remain owned by
  the calling BEAM processes.
  """
  @doc group: :lifecycle
  @spec unload() :: {:ok, true} | error()
  def unload, do: Native.unload()

  @doc """
  Unloads the shared reranker while keeping downloaded files and the embedding model.

  Returns `{:ok, true}` even if already unloaded. The synchronization, application
  queue ownership, and memory reclamation behavior described in `unload/0` also apply.
  """
  @doc group: :lifecycle
  @spec unload_reranker() :: {:ok, true} | error()
  def unload_reranker, do: Native.unload_reranker()

  @doc """
  Deletes the downloaded model files from disk, unloading matching native sessions first.

  The cache directory is the model's on-disk storage: deletion physically removes
  its repository directory, including ONNX weights, tokenizer/config files,
  blobs, and partial downloads. To release only RAM and keep these files, use
  `unload/0` or `unload_reranker/0`.

  Accepts the same names and kinds as `model_info/2`. **All cached variants and
  revisions in the selected repository are removed**, including shared tokenizer
  files and blobs. Both model families are checked before removal. Unrelated
  repositories and models loaded from a different cache root remain untouched.

  Returns `{:ok, true}` if removed or already absent. Repository symlinks and
  non-directory entries are rejected. Internal symlinks are removed without
  following their targets. If file removal fails after unloading, deletion can
  be partial and the model stays unloaded; correct the error and retry.

  Stop submitting work and drain application queues first, as for `unload/0`.
  Deletion is serialized with library loads in this VM. Other VMs or external
  processes sharing the cache must be coordinated by the application.
  """
  @doc group: :lifecycle
  @spec delete_model(String.t(), model_kind()) :: {:ok, true} | error()
  def delete_model(name, kind) when is_binary(name) and kind in [:embedding, :reranker] do
    with :ok <- validate_string(name, "model name must be a valid UTF-8 string") do
      Native.delete_model(name, kind, cache_dir())
    end
  end

  def delete_model(_name, _kind),
    do: {:error, "Invalid input: expected a model name and :embedding or :reranker"}

  @doc """
  Lists distinct model variants with their repository, dimension, and cache status.

  Checks the cache locally without network access or loading a model. `cached`
  means that the selected ONNX weights, additional data, and four tokenizer/config
  files are present and non-empty. It does not validate their contents or mean that
  the model is loaded in the VM. Call `load/1` or `load_reranker/1` before inference.

  Uses `FASTEMBED_CACHE_DIR`, defaulting to `.fastembed_cache`. Embedding variants
  appear first, then rerankers, sorted by name within each family. Rerankers have
  a `nil` dimension.

  Includes `loaded`, the absolute `cache_dir` and repository `path`, cached
  `revision`, relative required `files`, and per-file `file_details` with snapshot
  paths and byte sizes. `variant_bytes` is the sum of all required non-empty files
  (`nil` if incomplete). `disk_bytes` counts regular file bytes across the entire
  repository, including other variants, revisions and partial downloads, without
  following symlinks or counting snapshot links twice (`nil` on read errors).
  These are local file sizes, not RAM usage or remote download estimates. Shared
  repositories repeat `disk_bytes`; deduplicate by `path` before summing.

  `loaded` matches the variant and current cache root. Use `loaded_models/0` to
  find sessions loaded from other cache roots. Discovery may wait for active
  operations while reading runtime state.

  ## Examples

      iex> Enum.any?(ExFastembed.models(), &(&1.name == "BGESmallENV15"))
      true
  """
  @doc group: :discovery
  @spec models() :: [model_info()]
  def models, do: Native.models(cache_dir())

  @doc """
  Resolves an accepted model name to its variant, metadata, and local cache status.

  `kind` must be `:embedding` or `:reranker`. Resolution follows the same aliases
  and case-insensitive matching as `load/1` and `load_reranker/1`. See `models/0`
  for the meaning of `cached`.

  ## Examples

      iex> {:ok, info} = ExFastembed.model_info("BAAI/bge-small-en-v1.5", :embedding)
      iex> {info.name, info.dimension}
      {"BGESmallENV15", 384}
  """
  @doc group: :discovery
  @spec model_info(String.t(), model_kind()) :: {:ok, model_info()} | error()
  def model_info(name, kind) when is_binary(name) and kind in [:embedding, :reranker] do
    with :ok <- validate_string(name, "model name must be a valid UTF-8 string") do
      Native.model_info(name, kind, cache_dir())
    end
  end

  def model_info(_name, _kind),
    do: {:error, "Invalid input: expected a model name and :embedding or :reranker"}

  @doc """
  Returns text embedding model names accepted by `load/1`.

  Includes repository names, explicit FastEmbed variant names, and legacy aliases.
  Names are matched case-insensitively.

  The list is sorted and contains both full-precision and quantized variants.
  A repository shared by several variants selects its non-quantized model when
  available; use an explicit name such as `EmbeddingGemma300MQ4` for a specific variant.

  ## Examples

      iex> "BGESmallENV15" in ExFastembed.embed_models()
      true
  """
  @doc group: :discovery
  @spec embed_models() :: [String.t()]
  def embed_models, do: Native.embed_models()

  @doc """
  Returns reranker model names accepted by `load_reranker/1`.

  Includes repository names, explicit FastEmbed variant names, and legacy aliases.
  Names are matched case-insensitively.

  ## Examples

      iex> "BAAI/bge-reranker-base" in ExFastembed.reranker_models()
      true
  """
  @doc group: :discovery
  @spec reranker_models() :: [String.t()]
  def reranker_models, do: Native.reranker_models()

  @doc """
  Loads a text embedding model and returns its embedding dimension.

  A successful call replaces the embedding model shared by all processes.
  A failed download or initialization leaves the previous model available.
  See `embed_models/0` for accepted names. The returned dimension is the length
  of each vector generated by `embed_text/1`.

  ## Examples

  ```elixir
  {:ok, 384} = ExFastembed.load("BAAI/bge-small-en-v1.5")
  ```

      iex> ExFastembed.load("invalid-model")
      {:error, "Model not recognized or not implemented: invalid-model"}
  """
  @doc group: :embeddings
  @spec load(String.t()) :: {:ok, pos_integer()} | error()
  def load(model_name) when is_binary(model_name) do
    if String.valid?(model_name),
      do: Native.load(model_name, cache_dir()),
      else: invalid_embedding_model(model_name)
  end

  def load(model_name), do: invalid_embedding_model(model_name)

  @doc """
  Embeds a list of strings with the loaded text embedding model.

  Call `load/1` before calling this function. An empty list returns `{:ok, []}`
  without running inference.

  Results preserve the input order, with one vector per text. A non-empty input
  returns an error if no embedding model is loaded. Tokenization and truncation
  follow the selected model's FastEmbed defaults.

  ## Examples

      iex> ExFastembed.embed_text([])
      {:ok, []}

      iex> ExFastembed.embed_text(["document", 123])
      {:error, "Invalid input: texts must be a list of strings"}
  """
  @doc group: :embeddings
  @spec embed_text([String.t()]) :: {:ok, [embedding()]} | error()
  def embed_text([]), do: {:ok, []}

  def embed_text(texts) when is_list(texts) do
    with :ok <- validate_string_list(texts, "texts must be a list of strings") do
      Native.embed_text(texts)
    end
  end

  def embed_text(_texts), do: {:error, "Invalid input: texts must be a list of strings"}

  @doc """
  Loads a reranker model.

  A successful call replaces the reranker shared by all processes, independently
  of the embedding model. A failed load preserves the previous reranker.
  See `reranker_models/0` for accepted names.

  ## Examples

  ```elixir
  {:ok, true} = ExFastembed.load_reranker("BAAI/bge-reranker-base")
  ```

      iex> ExFastembed.load_reranker("invalid-reranker")
      {:error, "Reranker model not recognized: invalid-reranker"}
  """
  @doc group: :reranking
  @spec load_reranker(String.t()) :: {:ok, true} | error()
  def load_reranker(model_name) when is_binary(model_name) do
    if String.valid?(model_name),
      do: Native.load_reranker(model_name, cache_dir()),
      else: invalid_reranker(model_name)
  end

  def load_reranker(model_name), do: invalid_reranker(model_name)

  @doc """
  Reranks documents for a query using the loaded reranker model.

  Call `load_reranker/1` before calling this function. An empty document list
  returns `{:ok, []}` without running inference.

  Results are sorted by descending relevance score. Each result contains the
  document's zero-based index in the original list, its score, and its text when
  `return_docs` is `true` (`nil` otherwise). Scores are model-specific and are
  not necessarily probabilities. A non-empty input requires a loaded reranker.

  ## Examples

      iex> ExFastembed.rerank("query", [], false)
      {:ok, []}
  """
  @doc group: :reranking
  @spec rerank(String.t(), [String.t()], boolean()) :: {:ok, [rerank_result()]} | error()
  def rerank(query, documents, return_docs)
      when is_binary(query) and is_list(documents) and is_boolean(return_docs) do
    with :ok <- validate_string(query, "query must be a valid UTF-8 string"),
         :ok <- validate_string_list(documents, "documents must be a list of strings") do
      Native.rerank(query, documents, return_docs)
    end
  end

  def rerank(_query, _documents, _return_docs),
    do: {:error, "Invalid input: expected a string, a list of strings, and a boolean"}

  @spec cache_dir() :: String.t()
  defp cache_dir, do: System.get_env("FASTEMBED_CACHE_DIR", ".fastembed_cache")

  @spec validate_string_list(term(), String.t()) :: :ok | error()
  defp validate_string_list(values, message) do
    if valid_string_list?(values) do
      :ok
    else
      {:error, "Invalid input: #{message}"}
    end
  end

  @spec valid_string_list?(term()) :: boolean()
  defp valid_string_list?([]), do: true

  defp valid_string_list?([value | rest]) when is_binary(value),
    do: String.valid?(value) and valid_string_list?(rest)

  defp valid_string_list?(_values), do: false

  @spec validate_string(binary(), String.t()) :: :ok | error()
  defp validate_string(value, message) do
    if String.valid?(value), do: :ok, else: {:error, "Invalid input: #{message}"}
  end

  @spec invalid_embedding_model(term()) :: error()
  defp invalid_embedding_model(model_name) do
    {:error, "Model not recognized or not implemented: #{inspect(model_name)}"}
  end

  @spec invalid_reranker(term()) :: error()
  defp invalid_reranker(model_name) do
    {:error, "Reranker model not recognized: #{inspect(model_name)}"}
  end
end
