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

  test "cache paths and byte sizes describe complete and partial variants", %{cache: cache} do
    assert ExFastembed.cache_directory() == Path.expand(cache)
    assert {:ok, missing} = ExFastembed.model_info("AllMiniLML12V2", :embedding)
    assert missing.cache_dir == Path.expand(cache)
    assert missing.disk_bytes == 0
    assert missing.variant_bytes == nil
    assert missing.revision == nil
    refute missing.loaded
    assert Enum.all?(missing.file_details, &is_nil(&1.path))

    snapshot = cache_fixture(cache, "Xenova/all-MiniLM-L12-v2", "onnx/model.onnx")
    assert {:ok, info} = ExFastembed.model_info("AllMiniLML12V2", :embedding)
    assert info.cached
    assert info.revision == "fixture"
    assert info.variant_bytes == 35
    assert info.disk_bytes == 42
    assert info.path == Path.dirname(Path.dirname(snapshot))
    assert info.files == Enum.sort(Enum.uniq(info.files))
    assert Enum.all?(info.file_details, &(&1.size_bytes == 7))
    assert Enum.all?(info.file_details, &(&1.path == Path.join(snapshot, &1.name)))

    File.write!(Path.join(snapshot, "tokenizer.json"), "")
    assert {:ok, partial} = ExFastembed.model_info("AllMiniLML12V2", :embedding)
    refute partial.cached
    assert partial.variant_bytes == nil
    assert partial.disk_bytes == 35
  end

  test "deletion removes every variant in a repository but keeps unrelated models", %{
    cache: cache
  } do
    cache_fixture(cache, "Xenova/all-MiniLM-L12-v2", "onnx/model.onnx")
    cache_fixture(cache, "Xenova/all-MiniLM-L12-v2", "onnx/model_quantized.onnx")
    cache_fixture(cache, "Xenova/bge-small-en-v1.5", "onnx/model.onnx")
    assert {:ok, info} = ExFastembed.model_info("AllMiniLML12V2", :embedding)
    assert File.dir?(info.path)
    assert Enum.all?(info.file_details, &File.regular?(&1.path))

    assert {:ok, true} = ExFastembed.delete_model("AllMiniLML12V2", :embedding)
    refute File.exists?(info.path)
    assert Enum.all?(info.file_details, &(not File.exists?(&1.path)))
    assert {:ok, true} = ExFastembed.delete_model("AllMiniLML12V2", :embedding)

    for model <- ["AllMiniLML12V2", "AllMiniLML12V2Q"] do
      assert {:ok, %{cached: false, disk_bytes: 0}} = ExFastembed.model_info(model, :embedding)
    end

    assert {:ok, %{cached: true}} = ExFastembed.model_info("BGESmallENV15", :embedding)
    cache_fixture(cache, "jinaai/jina-reranker-v1-turbo-en", "onnx/model.onnx")
    assert {:ok, true} = ExFastembed.delete_model("JINARerankerV1TurboEn", :reranker)
    assert {:ok, %{cached: false}} = ExFastembed.model_info("JINARerankerV1TurboEn", :reranker)
  end

  test "deletion rejects invalid input and non-directory cache entries", %{cache: cache} do
    for {name, kind} <- [
          {"unknown", :embedding},
          {"unknown", :reranker},
          {nil, :embedding},
          {<<255>>, :embedding},
          {"BGESmallENV15", :unknown}
        ] do
      assert {:error, reason} = ExFastembed.delete_model(name, kind)
      assert is_binary(reason)
    end

    File.mkdir_p!(cache)
    path = Path.join(cache, "models--Xenova--bge-small-en-v1.5")
    File.write!(path, "keep")
    assert {:error, _} = ExFastembed.delete_model("BGESmallENV15", :embedding)
    assert File.read!(path) == "keep"
  end

  @tag skip: match?({:win32, _}, :os.type())
  test "repository symlinks are rejected without deleting the target", %{cache: cache} do
    outside = cache <> "-outside"
    on_exit(fn -> File.rm_rf!(outside) end)
    snapshot = cache_fixture(outside, "Xenova/bge-small-en-v1.5", "onnx/model.onnx")
    target = Path.dirname(Path.dirname(snapshot))
    File.mkdir_p!(cache)
    link = Path.join(cache, Path.basename(target))
    File.ln_s!(target, link)

    assert {:error, message} = ExFastembed.delete_model("BGESmallENV15", :embedding)
    assert message =~ "not a regular directory"
    assert File.read!(Path.join(snapshot, "onnx/model.onnx")) == "fixture"
    assert {:ok, %{type: :symlink}} = File.lstat(link)
  end

  @tag skip: match?({:win32, _}, :os.type())
  test "disk bytes include partial files and old revisions without double-counting blobs", %{
    cache: cache
  } do
    snapshot = cache_fixture(cache, "Xenova/bge-small-en-v1.5", "onnx/model.onnx")
    root = Path.dirname(Path.dirname(snapshot))
    blob = Path.join(root, "blobs/weights")
    File.mkdir_p!(Path.dirname(blob))
    File.rename!(Path.join(snapshot, "onnx/model.onnx"), blob)
    File.ln_s!(blob, Path.join(snapshot, "onnx/model.onnx"))
    File.write!(Path.join(root, "blobs/download.part"), "partial")
    File.mkdir_p!(Path.join(root, "snapshots/older"))
    File.write!(Path.join(root, "snapshots/older/model.onnx"), "old")

    assert {:ok, %{cached: true, variant_bytes: 35, disk_bytes: 52}} =
             ExFastembed.model_info("BGESmallENV15", :embedding)

    assert {:ok, true} = ExFastembed.delete_model("BGESmallENV15", :embedding)
    refute File.exists?(root)
    refute File.exists?(blob)
  end

  test "deletion removes an interrupted download without a complete snapshot", %{cache: cache} do
    root = Path.join(cache, "models--Xenova--bge-small-en-v1.5")
    File.mkdir_p!(Path.join(root, "blobs"))
    File.write!(Path.join(root, "blobs/weights.part"), "partial")

    assert {:ok, %{cached: false, variant_bytes: nil, disk_bytes: 7}} =
             ExFastembed.model_info("BGESmallENV15", :embedding)

    assert {:ok, true} = ExFastembed.delete_model("BGESmallENV15", :embedding)
    refute File.exists?(root)
    assert File.dir?(cache)
  end

  test "unload is idempotent and the loaded filter is empty without sessions" do
    assert {:ok, true} = ExFastembed.unload()
    assert {:ok, true} = ExFastembed.unload()
    assert {:ok, true} = ExFastembed.unload_reranker()
    assert {:ok, true} = ExFastembed.unload_reranker()
    assert {:ok, []} = ExFastembed.loaded_models()
    assert {:error, _} = ExFastembed.embed_text(["after unload"])
    assert {:error, _} = ExFastembed.rerank("query", ["after unload"], false)
    assert :ok = Models.run(["--loaded"])
    assert messages() =~ "0 models shown; 0 cached."
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
