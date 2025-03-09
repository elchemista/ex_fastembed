defmodule ExFastembed do
  @moduledoc """
  ExFastembed provides functions to load text embedding and reranker models,
  generate embeddings for lists of texts, and rerank documents given a query.

  ## Example Usage

      # Load an embedding model
      iex> ExFastembed.load("BAAI/bge-small-en-v1.5")
      {:ok, 768}  # assuming the model returns embeddings of dimension 768

      # Embed a list of texts
      iex> ExFastembed.embed_text(["Hello", "World"])
      {:ok, [[0.1, 0.2, ...], [0.3, 0.4, ...]]}

      # Load a reranker model
      iex> ExFastembed.load_reranker("BAAI/bge-reranker-base")
      {:ok, true}

      # Rerank documents for a given query
      iex> ExFastembed.rerank("search query", ["doc1", "doc2"], true)
      {:ok, [{0, 0.95, "doc1"}, {1, 0.90, "doc2"}]}
  """
  version = Mix.Project.config()[:version]

  use RustlerPrecompiled,
    otp_app: :ex_fastembed,
    crate: "ex_fastembed",
    base_url: "https://github.com/elchemista/ex_fastembed/releases/download/v#{version}/",
    force_build: System.get_env("RUSTLER_PRECOMPILATION_EXAMPLE_BUILD") in ["1", "true"],
    version: version

  @valid_models [
    "BAAI/bge-small-en-v1.5",
    "sentence-transformers/all-MiniLM-L6-v2",
    "sentence-transformers/all-MiniLM-L12-v2",
    "mixedbread-ai/mxbai-embed-large-v1",
    "Qdrant/clip-ViT-B-32-text",
    "BAAI/bge-large-en-v1.5",
    "BAAI/bge-small-zh-v1.5",
    "BAAI/bge-base-en-v1.5",
    "sentence-transformers/paraphrase-MiniLM-L12-v2",
    "sentence-transformers/paraphrase-multilingual-mpnet-base-v2",
    "lightonai/ModernBERT-embed-large",
    "nomic-ai/nomic-embed-text-v1",
    "nomic-ai/nomic-embed-text-v1.5",
    "intfloat/multilingual-e5-small",
    "intfloat/multilingual-e5-base",
    "intfloat/multilingual-e5-large",
    "Alibaba-NLP/gte-base-en-v1.5",
    "Alibaba-NLP/gte-large-en-v1.5"
  ]

  @reranker_models [
    "BAAI/bge-reranker-base",
    "BAAI/bge-reranker-v2-m3",
    "jinaai/jina-reranker-v1-turbo-en",
    "jinaai/jina-reranker-v2-base-multiligual"
  ]

  @spec embed_models() :: [<<_::64, _::_*8>>, ...]
  @doc """
  Returns the list of valid text embedding models.
  """
  def embed_models, do: @valid_models

  @spec reranker_models() :: [<<_::64, _::_*8>>, ...]
  @doc """
  Returns the list of valid reranker models.
  """
  def reranker_models, do: @reranker_models

  @doc """
  Loads the chosen text embedding model.

  ## Examples

      iex> ExFastembed.load("BAAI/bge-small-en-v1.5")
      {:ok, 768}

      iex> ExFastembed.load("invalid-model")
      {:error, "Model not recognized or not implemented: invalid-model"}

  """
  @spec load(String.t()) :: {:ok, integer()} | {:error, String.t()}
  def load(model_name) when is_bitstring(model_name) and model_name in @valid_models do
    :erlang.nif_error("NIF load/1 not loaded")
  end

  def load(model_name),
    do: {:error, "Model not recognized or not implemented: #{model_name}"}

  @doc """
  Returns a list of embeddings for each text in the given list.

  ## Examples

      iex> ExFastembed.embed_text(["Hello", "World"])
      {:ok, [[0.1, 0.2, ...], [0.3, 0.4, ...]]}

      iex> ExFastembed.embed_text("Hello")
      {:error, "Invalid input, expected a list of strings"}

  """
  @spec embed_text([String.t()]) :: {:ok, [[float()]]} | {:error, String.t()}
  def embed_text(texts) when is_list(texts) do
    :erlang.nif_error("NIF embed_text/1 not loaded")
  end

  def embed_text(_texts),
    do: {:error, "Invalid input, expected a list of strings"}

  @doc """
  Loads the chosen reranker model.

  ## Examples

      iex> ExFastembed.load_reranker("BAAI/bge-reranker-base")
      {:ok, true}

      iex> ExFastembed.load_reranker("invalid-reranker")
      {:error, "Model not recognized or not implemented: invalid-reranker"}

  """
  @spec load_reranker(String.t()) :: {:ok, boolean()} | {:error, String.t()}
  def load_reranker(model_name)
      when is_bitstring(model_name) and model_name in @reranker_models do
    :erlang.nif_error("NIF load_reranker/1 not loaded")
  end

  def load_reranker(model_name),
    do: {:error, "Model not recognized or not implemented: #{model_name}"}

  @doc """
  Reranks documents based on a query using the loaded reranker model.

  This function validates that:
    - `query` is a string,
    - `documents` is a list of strings,
    - `return_docs` is a boolean.

  If validation passes, it returns a list of tuples. Each tuple consists of:
    - the document index (non-negative integer),
    - the score (float),
    - optionally the document string (or `nil` if `return_docs` is false).

  ## Examples

      iex> ExFastembed.rerank("search query", ["doc1", "doc2"], true)
      {:ok, [{0, 0.95, "doc1"}, {1, 0.90, "doc2"}]}

      iex> ExFastembed.rerank("search query", ["doc1", 123], true)
      {:error, "Invalid input: documents must be a list of strings"}

      iex> ExFastembed.rerank(123, ["doc1", "doc2"], true)
      {:error, "Invalid input: Expected (String, list of strings, boolean)"}

      iex> ExFastembed.rerank("search query", ["doc1", "doc2"], "true")
      {:error, "Invalid input: Expected (String, list of strings, boolean)"}
  """
  @spec rerank(String.t(), [String.t()], boolean()) ::
          {:ok, [{non_neg_integer(), float(), String.t() | nil}]} | {:error, String.t()}

  def rerank(query, documents, return_docs)
      when is_bitstring(query) and is_list(documents) and is_boolean(return_docs) do
    cond do
      not Enum.all?(documents, &is_bitstring/1) ->
        {:error, "Invalid input: documents must be a list of strings"}

      true ->
        # All validations passed; call the native implementation.
        :erlang.nif_error("NIF rerank/3 not loaded")
    end
  end

  def rerank(_query, _documents, _return_docs),
    do: {:error, "Invalid input: Expected (String, list of strings, boolean)"}
end
