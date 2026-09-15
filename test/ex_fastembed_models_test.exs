defmodule ExFastembedModelsTest do
  use ExUnit.Case, async: false

  alias Mix.Tasks.Fastembed.{Download, Models}

  setup do
    directory =
      Path.join(System.tmp_dir!(), "fastembed-models-#{System.unique_integer([:positive])}")

    previous_cache = System.get_env("FASTEMBED_CACHE_DIR")
    previous_shell = Mix.shell()
    System.put_env("FASTEMBED_CACHE_DIR", directory)
    Mix.shell(Mix.Shell.Process)

    on_exit(fn ->
      if previous_cache,
        do: System.put_env("FASTEMBED_CACHE_DIR", previous_cache),
        else: System.delete_env("FASTEMBED_CACHE_DIR")

      Mix.shell(previous_shell)
      File.rm_rf!(directory)
    end)

    %{cache: directory}
  end

  test "catalog lists distinct variants with dimensions and local cache status", %{cache: cache} do
    models = ExFastembed.models()
    assert length(models) == 50
    assert length(Enum.uniq_by(models, &{&1.kind, &1.name})) == 50
    assert models == Enum.sort_by(models, &{&1.kind, &1.name})
    refute Enum.any?(models, & &1.cached)
    assert Enum.count(models, &(&1.kind == :reranker and is_nil(&1.dimension))) == 4

    snapshot = cache_fixture(cache, "Xenova/all-MiniLM-L12-v2", "onnx/model.onnx")

    assert {:ok, %{name: "AllMiniLML12V2", dimension: 384, cached: true}} =
             ExFastembed.model_info("xenova/all-minilm-l12-v2", :embedding)

    assert {:ok, %{cached: false}} = ExFastembed.model_info("AllMiniLML12V2Q", :embedding)
    File.rm!(Path.join(snapshot, "config.json"))
    assert {:ok, %{cached: false}} = ExFastembed.model_info("AllMiniLML12V2", :embedding)
  end

  test "metadata resolves reranker aliases and rejects invalid input" do
    assert {:ok, %{name: "JINARerankerV2BaseMultiligual", kind: :reranker}} =
             ExFastembed.model_info("jinaai/jina-reranker-v2-base-multiligual", :reranker)

    for {name, kind} <- [
          {"unknown", :embedding},
          {"unknown", :reranker},
          {<<255>>, :embedding},
          {nil, :embedding},
          {"BGESmallENV15", :image}
        ] do
      assert {:error, reason} = ExFastembed.model_info(name, kind)
      assert is_binary(reason)
    end
  end

  test "list task shows cached variants and filters by family", %{cache: cache} do
    cache_fixture(cache, "Xenova/bge-small-en-v1.5", "onnx/model.onnx")
    cache_fixture(cache, "jinaai/jina-reranker-v1-turbo-en", "onnx/model.onnx")

    assert :ok = Models.run([])
    output = messages()
    assert output =~ "BGESmallENV15"
    assert output =~ "download"
    assert output =~ "50 models shown; 2 cached."

    assert :ok = Models.run(["--cached", "--type", "embedding"])
    output = messages()
    assert output =~ "BGESmallENV15"
    refute output =~ "JINARerankerV1TurboEn"
    assert output =~ "1 models shown; 1 cached."

    assert :ok = Models.run(["--type", "reranker"])
    assert messages() =~ "4 models shown; 1 cached."
  end

  test "list task explains an empty cache and rejects malformed arguments" do
    assert :ok = Models.run(["--cached"])
    assert messages() =~ "0 models shown; 0 cached."

    for args <- [["unexpected"], ["--type", "image"]] do
      assert_raise Mix.Error, ~r/Usage:/, fn -> Models.run(args) end
    end

    assert_raise OptionParser.ParseError, fn -> Models.run(["--invalid"]) end
  end

  test "download task reuses complete cached embedding and reranker files", %{cache: cache} do
    cache_fixture(cache, "Xenova/bge-small-en-v1.5", "onnx/model.onnx")
    cache_fixture(cache, "jinaai/jina-reranker-v1-turbo-en", "onnx/model.onnx")
    assert :ok = Download.run(["BAAI/bge-small-en-v1.5"])
    assert messages() =~ "BGESmallENV15 is already cached; no download needed."
    assert :ok = Download.run(["JINARerankerV1TurboEn", "--reranker"])
    assert messages() =~ "JINARerankerV1TurboEn is already cached"
  end

  test "download task reports model and argument errors" do
    for args <- [[], ["one", "two"]] do
      assert_raise Mix.Error, ~r/Usage:/, fn -> Download.run(args) end
    end

    assert_raise OptionParser.ParseError, fn -> Download.run(["--invalid"]) end
    assert_raise Mix.Error, ~r/Model not recognized/, fn -> Download.run(["invalid"]) end

    assert_raise Mix.Error, ~r/Reranker model not recognized/, fn ->
      Download.run(["invalid", "--reranker"])
    end
  end

  defp cache_fixture(cache, repository, model_file) do
    root = Path.join(cache, "models--" <> String.replace(repository, "/", "--"))
    snapshot = Path.join(root, "snapshots/fixture")
    File.mkdir_p!(Path.join(root, "refs"))
    File.write!(Path.join(root, "refs/main"), "fixture")

    for file <- [
          model_file,
          "tokenizer.json",
          "config.json",
          "special_tokens_map.json",
          "tokenizer_config.json"
        ] do
      path = Path.join(snapshot, file)
      File.mkdir_p!(Path.dirname(path))
      File.write!(path, "fixture")
    end

    snapshot
  end

  defp messages do
    receive do
      {:mix_shell, :info, [message]} -> IO.iodata_to_binary(message) <> "\n" <> messages()
    after
      0 -> ""
    end
  end
end
