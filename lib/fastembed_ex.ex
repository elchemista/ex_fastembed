defmodule FastembedEx do
  use Rustler,
    otp_app: :fastembed_ex,
    crate: "fastembed_nif"

  @doc """
  Loads the chosen text embedding model (e.g. "BAAI/bge-small-en-v1.5").
  """
  def load(_model_name), do: :erlang.nif_error("NIF load/1 not loaded")

  @doc """
  Returns a list of embeddings for each text in the given list.
  """
  def embed_text(_texts), do: :erlang.nif_error("NIF embed_text/1 not loaded")

  def load_reranker(_model_name), do: :erlang.nif_error("NIF load_reranker/1 not loaded")

  def rerank(_query, _documents, _return_docs), do: :erlang.nif_error("NIF rerank/3 not loaded")
end
