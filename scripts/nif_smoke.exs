# Load the exact matrix artifact without compiling Rust or fetching a release.
# Every NIF export needs a stub before the native library can replace it.
defmodule ExFastembed.Native do
  @moduledoc false

  for {name, arity} <- [
        models: 1,
        model_info: 3,
        embed_models: 0,
        reranker_models: 0,
        load: 2,
        embed_text: 1,
        load_reranker: 2,
        rerank: 3
      ] do
    def unquote(name)(unquote_splicing(Macro.generate_arguments(arity, __MODULE__))),
      do: :erlang.nif_error(:nif_not_loaded)
  end

  def load_library(path), do: :erlang.load_nif(String.to_charlist(path), 0)
end

[library] = System.argv()
:ok = ExFastembed.Native.load_library(library |> Path.expand() |> Path.rootname())
Code.require_file("../lib/ex_fastembed.ex", __DIR__)

50 = length(ExFastembed.models())
{:ok, 384} = ExFastembed.load("BGESmallENV15")
{:ok, [embedding]} = ExFastembed.embed_text(["Precompiled NIF runtime check"])
384 = length(embedding)
true = Enum.all?(embedding, &is_float/1)
{:ok, %{cached: true}} = ExFastembed.model_info("BGESmallENV15", :embedding)

IO.puts("Precompiled NIF: model discovery, loading, cache, and embedding inference passed")
