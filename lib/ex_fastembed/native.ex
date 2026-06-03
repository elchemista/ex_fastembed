defmodule ExFastembed.Native do
  @moduledoc false

  use Rustler,
    otp_app: :ex_fastembed,
    crate: "ex_fastembed"

  @doc false
  @spec embed_models() :: [String.t()]
  def embed_models, do: :erlang.nif_error("NIF embed_models/0 not loaded")

  @doc false
  @spec reranker_models() :: [String.t()]
  def reranker_models, do: :erlang.nif_error("NIF reranker_models/0 not loaded")

  @doc false
  @spec load(String.t()) :: {:ok, pos_integer()} | {:error, String.t()}
  def load(_model_name), do: :erlang.nif_error("NIF load/1 not loaded")

  @doc false
  @spec embed_text([String.t()]) :: {:ok, [[float()]]} | {:error, String.t()}
  def embed_text(_texts), do: :erlang.nif_error("NIF embed_text/1 not loaded")

  @doc false
  @spec load_reranker(String.t()) :: {:ok, true} | {:error, String.t()}
  def load_reranker(_model_name), do: :erlang.nif_error("NIF load_reranker/1 not loaded")

  @doc false
  @spec rerank(String.t(), [String.t()], boolean()) ::
          {:ok, [{non_neg_integer(), float(), String.t() | nil}]} | {:error, String.t()}
  def rerank(_query, _documents, _return_docs), do: :erlang.nif_error("NIF rerank/3 not loaded")
end
