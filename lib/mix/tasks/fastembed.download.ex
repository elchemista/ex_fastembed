defmodule Mix.Tasks.Fastembed.Download do
  use Mix.Task

  @shortdoc "Downloads a FastEmbed model unless its files are already cached"
  @moduledoc """
  Downloads a model into `FASTEMBED_CACHE_DIR` (default `.fastembed_cache`).
  An `HF_HOME` exported before starting the VM takes precedence.

      mix fastembed.download BGESmallENV15
      mix fastembed.download AllMiniLML6V2Q
      mix fastembed.download JINARerankerV1TurboEn --reranker

  Accepts the same aliases as `ExFastembed.load/1` and `ExFastembed.load_reranker/1`.
  Use `mix fastembed.models` to list variants and their cache status.

  Complete cached models are skipped. Missing files are downloaded without
  initializing an ONNX session or changing loaded models. Existing non-empty files
  are reused. Empty snapshot files and dangling snapshot links are replaced;
  their link targets are kept. A directory occupying a required file path returns
  an error. A download failure exits with a Mix error.
  Call `ExFastembed.load/1` or `ExFastembed.load_reranker/1` before inference to
  initialize the model and validate its contents.
  """

  @doc false
  @impl Mix.Task
  @spec run([String.t()]) :: :ok
  def run(args) do
    {opts, names} = OptionParser.parse!(args, strict: [reranker: :boolean])

    case names do
      [name] ->
        Mix.Task.run("app.start")
        kind = if opts[:reranker], do: :reranker, else: :embedding
        download(name, kind)

      _ ->
        Mix.raise("Usage: mix fastembed.download MODEL [--reranker]")
    end
  end

  @spec download(String.t(), ExFastembed.model_kind()) :: :ok
  defp download(name, kind) do
    case ExFastembed.model_info(name, kind) do
      {:ok, %{cached: true} = model} ->
        Mix.shell().info("#{model.name} is already cached; no download needed.")

      {:ok, model} ->
        Mix.shell().info("Downloading #{model.name} from #{model.repository}...")

        case ExFastembed.download(model.name, kind) do
          {:ok, true} -> Mix.shell().info("#{model.name} downloaded and ready to load.")
          {:error, reason} -> Mix.raise("Could not download #{model.name}: #{reason}")
        end

      {:error, reason} ->
        Mix.raise(reason)
    end
  end
end
