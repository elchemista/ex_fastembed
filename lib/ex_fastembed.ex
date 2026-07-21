defmodule ExFastembed do
  @moduledoc """
  Loads FastEmbed text embedding and reranker models through `fastembed-rs`.

  The public API validates Elixir input before delegating to the native Rustler
  module. Model names are resolved by the native layer so the supported model
  lists stay aligned with the bundled `fastembed` crate.
  """

  alias ExFastembed.Native

  @type embedding :: [float()]
  @type rerank_result :: {non_neg_integer(), float(), String.t() | nil}
  @type error :: {:error, String.t()}

  @doc """
  Returns text embedding model names accepted by `load/1`.
  """
  @spec embed_models() :: [String.t()]
  def embed_models, do: Native.embed_models()

  @doc """
  Returns reranker model names accepted by `load_reranker/1`.
  """
  @spec reranker_models() :: [String.t()]
  def reranker_models, do: Native.reranker_models()

  @doc """
  Loads a text embedding model and returns its embedding dimension.

  A subsequent call replaces the active embedding model.

  ## Examples

      iex> ExFastembed.load("BAAI/bge-small-en-v1.5")
      {:ok, 384}

      iex> ExFastembed.load("invalid-model")
      {:error, "Model not recognized or not implemented: invalid-model"}
  """
  @spec load(String.t()) :: {:ok, pos_integer()} | error()
  def load(model_name) when is_binary(model_name) do
    if String.valid?(model_name),
      do: Native.load(model_name),
      else: invalid_embedding_model(model_name)
  end

  def load(model_name), do: invalid_embedding_model(model_name)

  @doc """
  Embeds a list of strings with the loaded text embedding model.

  Call `load/1` before calling this function. An empty list returns `{:ok, []}`
  without running inference.
  """
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

  A subsequent call replaces the active reranker model.

  ## Examples

      iex> ExFastembed.load_reranker("BAAI/bge-reranker-base")
      {:ok, true}

      iex> ExFastembed.load_reranker("invalid-reranker")
      {:error, "Reranker model not recognized: invalid-reranker"}
  """
  @spec load_reranker(String.t()) :: {:ok, true} | error()
  def load_reranker(model_name) when is_binary(model_name) do
    if String.valid?(model_name),
      do: Native.load_reranker(model_name),
      else: invalid_reranker(model_name)
  end

  def load_reranker(model_name), do: invalid_reranker(model_name)

  @doc """
  Reranks documents for a query using the loaded reranker model.

  Call `load_reranker/1` before calling this function. An empty document list
  returns `{:ok, []}` without running inference.
  """
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

  @spec validate_string_list([term()], String.t()) :: :ok | error()
  defp validate_string_list(values, message) do
    if Enum.all?(values, &(is_binary(&1) and String.valid?(&1))) do
      :ok
    else
      {:error, "Invalid input: #{message}"}
    end
  end

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
