defmodule ExFastembed.Native do
  @moduledoc false

  version = Mix.Project.config()[:version]

  use RustlerPrecompiled,
    otp_app: :ex_fastembed,
    crate: "ex_fastembed",
    base_url: "https://github.com/elchemista/ex_fastembed/releases/download/v#{version}",
    version: version,
    force_build: System.get_env("EX_FASTEMBED_BUILD") in ["1", "true"],
    targets: [
      "aarch64-apple-darwin",
      "aarch64-unknown-linux-gnu",
      "x86_64-pc-windows-msvc",
      "x86_64-unknown-linux-gnu"
    ],
    nif_versions: ["2.15", "2.16"]

  @doc false
  @spec models(String.t()) :: [ExFastembed.model_info()]
  def models(_cache_dir), do: :erlang.nif_error("NIF models/1 not loaded")

  @doc false
  @spec model_info(String.t(), ExFastembed.model_kind(), String.t()) ::
          {:ok, ExFastembed.model_info()} | ExFastembed.error()
  def model_info(_name, _kind, _cache_dir), do: :erlang.nif_error("NIF model_info/3 not loaded")

  @doc false
  @spec embed_models() :: [String.t()]
  def embed_models, do: :erlang.nif_error("NIF embed_models/0 not loaded")

  @doc false
  @spec reranker_models() :: [String.t()]
  def reranker_models, do: :erlang.nif_error("NIF reranker_models/0 not loaded")

  @doc false
  @spec load(String.t(), String.t()) :: {:ok, pos_integer()} | {:error, String.t()}
  def load(_model_name, _cache_dir), do: :erlang.nif_error("NIF load/2 not loaded")

  @doc false
  @spec embed_text([String.t()]) :: {:ok, [[float()]]} | {:error, String.t()}
  def embed_text(_texts), do: :erlang.nif_error("NIF embed_text/1 not loaded")

  @doc false
  @spec load_reranker(String.t(), String.t()) :: {:ok, true} | {:error, String.t()}
  def load_reranker(_model_name, _cache_dir),
    do: :erlang.nif_error("NIF load_reranker/2 not loaded")

  @doc false
  @spec rerank(String.t(), [String.t()], boolean()) ::
          {:ok, [{non_neg_integer(), float(), String.t() | nil}]} | {:error, String.t()}
  def rerank(_query, _documents, _return_docs), do: :erlang.nif_error("NIF rerank/3 not loaded")
end
