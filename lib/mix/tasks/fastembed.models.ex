defmodule Mix.Tasks.Fastembed.Models do
  use Mix.Task

  @shortdoc "Lists FastEmbed model variants and their local cache status"
  @moduledoc """
  Lists supported models, marking complete local downloads as `cached`.

      mix fastembed.models
      mix fastembed.models --cached
      mix fastembed.models --type reranker

  Options:

    * `--cached` — only show models with all required files present.
    * `--type embedding|reranker` — filter by model family.

  Reads `FASTEMBED_CACHE_DIR` (default `.fastembed_cache`) without accessing the
  network. Cache status checks file presence, not contents or runtime compatibility.
  An `HF_HOME` exported before starting the VM overrides `FASTEMBED_CACHE_DIR`.
  The `MODEL` column contains explicit variant names accepted by
  `mix fastembed.download`, `ExFastembed.load/1`, and `ExFastembed.load_reranker/1`.
  """

  @doc false
  @impl Mix.Task
  @spec run([String.t()]) :: :ok
  def run(args) do
    {opts, rest} = OptionParser.parse!(args, strict: [cached: :boolean, type: :string])

    if rest != [] or opts[:type] not in [nil, "embedding", "reranker"] do
      Mix.raise("Usage: mix fastembed.models [--cached] [--type embedding|reranker]")
    end

    Mix.Task.run("app.start")

    models =
      Enum.filter(ExFastembed.models(), fn model ->
        (opts[:cached] != true or model.cached) and
          (is_nil(opts[:type]) or Atom.to_string(model.kind) == opts[:type])
      end)

    cache_dir =
      System.get_env("HF_HOME") || System.get_env("FASTEMBED_CACHE_DIR", ".fastembed_cache")

    Mix.shell().info("Cache: #{Path.expand(cache_dir)}")

    rows =
      Enum.map(models, fn model ->
        [
          if(model.cached, do: "cached", else: "download"),
          Atom.to_string(model.kind),
          to_string(model.dimension || "-"),
          model.name,
          model.repository
        ]
      end)

    print_table([~w(STATUS TYPE DIM MODEL REPOSITORY) | rows])
    Mix.shell().info("#{length(models)} models shown; #{Enum.count(models, & &1.cached)} cached.")
    :ok
  end

  @spec print_table([[String.t()]]) :: :ok
  defp print_table(rows) do
    widths =
      rows
      |> Enum.zip()
      |> Enum.map(fn column ->
        column |> Tuple.to_list() |> Enum.map(&String.length/1) |> Enum.max()
      end)

    Enum.each(rows, fn row ->
      row
      |> Enum.zip_with(widths, &String.pad_trailing/2)
      |> Enum.join("  ")
      |> String.trim_trailing()
      |> Mix.shell().info()
    end)
  end
end
