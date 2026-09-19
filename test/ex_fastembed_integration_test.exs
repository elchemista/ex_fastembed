defmodule ExFastembedIntegrationTest do
  # Native models are global to the VM; inference tests must run serially.
  use ExUnit.Case, async: false

  alias Mix.Tasks.Fastembed.Download

  @moduletag :integration
  @moduletag timeout: 300_000

  setup do
    on_exit(fn ->
      ExFastembed.unload()
      ExFastembed.unload_reranker()
    end)

    :ok
  end

  test "unload releases sessions independently, keeps files, and supports reloading" do
    assert {:ok, 384} = ExFastembed.load("BGESmallENV15")
    assert {:ok, true} = ExFastembed.load_reranker("JINARerankerV1TurboEn")
    assert {:ok, models} = ExFastembed.loaded_models()
    assert Enum.map(models, & &1.kind) == [:embedding, :reranker]
    assert Enum.all?(models, & &1.loaded)
    assert Enum.all?(models, &(&1.variant_bytes > 0 and &1.disk_bytes > 0))

    assert {:ok, true} = ExFastembed.unload()
    assert {:error, _} = ExFastembed.embed_text(["unloaded"])

    assert {:ok, %{loaded: false, cached: true}} =
             ExFastembed.model_info("BGESmallENV15", :embedding)

    assert {:ok, [_]} = ExFastembed.rerank("query", ["document"], false)
    assert {:ok, [%{kind: :reranker}]} = ExFastembed.loaded_models()

    assert {:ok, 384} = ExFastembed.load("BGESmallENV15")
    assert {:ok, true} = ExFastembed.unload_reranker()
    assert {:error, _} = ExFastembed.rerank("query", ["document"], false)
    assert {:ok, vectors} = ExFastembed.embed_text(["reloaded"])
    assert_embeddings(vectors, 1, 384)
    assert {:ok, [%{kind: :embedding}]} = ExFastembed.loaded_models()
  end

  test "deletion unloads matching sessions and tracks changes of cache root" do
    assert {:ok, 384} = ExFastembed.load("BGESmallENV15")
    assert {:ok, true} = ExFastembed.load_reranker("JINARerankerV1TurboEn")
    assert {:ok, original} = ExFastembed.model_info("BGESmallENV15", :embedding)
    original_cache = System.get_env("FASTEMBED_CACHE_DIR")
    cache = Path.join(System.tmp_dir!(), "fastembed-delete-#{System.unique_integer([:positive])}")
    File.mkdir_p!(cache)

    try do
      System.put_env("FASTEMBED_CACHE_DIR", cache)

      assert {:ok, %{loaded: false, cached: false}} =
               ExFastembed.model_info("BGESmallENV15", :embedding)

      assert {:ok, loaded} = ExFastembed.loaded_models()
      assert Enum.find(loaded, &(&1.kind == :embedding)).path == original.path
      assert {:ok, true} = ExFastembed.delete_model("BGESmallENV15", :embedding)
      assert {:ok, [_]} = ExFastembed.embed_text(["original cache still loaded"])

      destination = Path.join(cache, Path.basename(original.path))
      File.cp_r!(original.path, destination)
      assert {:ok, 384} = ExFastembed.load("BGESmallENV15")

      assert {:ok, %{loaded: true, path: ^destination}} =
               ExFastembed.model_info("BGESmallENV15", :embedding)

      assert {:ok, true} = ExFastembed.delete_model("BGESmallENV15", :embedding)
      assert {:error, _} = ExFastembed.embed_text(["deleted"])
      assert {:ok, [_]} = ExFastembed.rerank("query", ["document"], false)
      assert {:ok, [%{kind: :reranker}]} = ExFastembed.loaded_models()
      refute File.exists?(destination)
      assert File.dir?(original.path)
    after
      if original_cache,
        do: System.put_env("FASTEMBED_CACHE_DIR", original_cache),
        else: System.delete_env("FASTEMBED_CACHE_DIR")

      ExFastembed.unload()
      ExFastembed.unload_reranker()
      File.rm_rf!(cache)
    end
  end

  test "download task populates an empty cache and produces a usable embedding model" do
    original_cache = System.get_env("FASTEMBED_CACHE_DIR")

    cache =
      Path.join(System.tmp_dir!(), "fastembed-download-#{System.unique_integer([:positive])}")

    try do
      System.put_env("FASTEMBED_CACHE_DIR", cache)
      assert {:ok, %{cached: false}} = ExFastembed.model_info("AllMiniLML6V2Q", :embedding)
      assert :ok = Download.run(["AllMiniLML6V2Q"])
      assert {:ok, %{cached: true}} = ExFastembed.model_info("AllMiniLML6V2Q", :embedding)
      assert {:ok, embeddings} = ExFastembed.embed_text(["Downloaded model"])
      assert_embeddings(embeddings, 1, 384)
    after
      if original_cache,
        do: System.put_env("FASTEMBED_CACHE_DIR", original_cache),
        else: System.delete_env("FASTEMBED_CACHE_DIR")

      File.rm_rf!(cache)
    end
  end

  test "embedding inference, replacement, and failed loads preserve the active model" do
    texts = ["Hello, world!", "Elixir rende semplice la concorrenza."]

    assert {:ok, 384} = ExFastembed.load("BAAI/bge-small-en-v1.5")
    assert_partial_download("BGESmallENV15", :embedding)
    assert {:ok, original_embeddings} = ExFastembed.embed_text(texts)
    assert_embeddings(original_embeddings, 2, 384)

    assert {:ok, 384} = ExFastembed.load("allminilml6v2q")
    assert {:ok, replacement_embeddings} = ExFastembed.embed_text(texts)
    assert_embeddings(replacement_embeddings, 2, 384)
    refute original_embeddings == replacement_embeddings

    assert {:error, _message} = ExFastembed.load("invalid-model")

    texts
    |> List.duplicate(4)
    |> Task.async_stream(&ExFastembed.embed_text/1, timeout: 60_000)
    |> Enum.each(fn {:ok, {:ok, embeddings}} ->
      assert_embeddings(embeddings, 2, 384)

      for {actual, expected} <-
            Enum.zip(List.flatten(embeddings), List.flatten(replacement_embeddings)) do
        assert_in_delta actual, expected, 1.0e-5
      end
    end)
  end

  test "reranker returns sorted scores, document indices, and optional documents" do
    query = "What is the capital of Italy?"
    documents = ["Rome is the capital of Italy.", "Bananas are yellow fruit."]

    assert {:ok, true} = ExFastembed.load_reranker("JINARerankerV1TurboEn")
    assert_partial_download("JINARerankerV1TurboEn", :reranker)
    assert {:ok, with_docs} = ExFastembed.rerank(query, documents, true)
    assert Enum.sort(Enum.map(with_docs, &elem(&1, 0))) == [0, 1]
    assert with_docs == Enum.sort_by(with_docs, &elem(&1, 1), :desc)
    assert [{0, _score, _document} | _rest] = with_docs

    for {index, score, document} <- with_docs do
      assert is_float(score)
      assert document == Enum.at(documents, index)
    end

    assert {:error, _message} = ExFastembed.load_reranker("invalid-reranker")
    assert {:ok, without_docs} = ExFastembed.rerank(query, documents, false)
    assert length(without_docs) == length(documents)

    for {{index, score, document}, {expected_index, expected_score, _document}} <-
          Enum.zip(without_docs, with_docs) do
      assert index == expected_index
      assert document == nil
      assert_in_delta score, expected_score, 1.0e-5
    end
  end

  defp assert_partial_download(name, kind) do
    {:ok, %{repository: repository, cached: true}} = ExFastembed.model_info(name, kind)
    original_cache = System.get_env("FASTEMBED_CACHE_DIR")
    cache = original_cache || ".fastembed_cache"
    folder = "models--" <> String.replace(repository, "/", "--")

    partial_cache =
      Path.join(System.tmp_dir!(), "fastembed-partial-#{System.unique_integer([:positive])}")

    destination = Path.join(partial_cache, folder)
    File.mkdir_p!(partial_cache)
    File.cp_r!(Path.join(cache, folder), destination)
    revision = File.read!(Path.join(destination, "refs/main"))
    File.rm!(Path.join([destination, "snapshots", revision, "config.json"]))

    try do
      System.put_env("FASTEMBED_CACHE_DIR", partial_cache)
      assert {:ok, %{cached: false}} = ExFastembed.model_info(name, kind)
      args = if kind == :reranker, do: [name, "--reranker"], else: [name]
      assert :ok = Download.run(args)
      assert {:ok, %{cached: true}} = ExFastembed.model_info(name, kind)
      assert :ok = Download.run(args)
      assert File.read!(Path.join(destination, "refs/main")) == revision

      # Initialization errors remain actionable and preserve the active model.
      File.write!(
        Path.join([destination, "snapshots", revision, "onnx/model.onnx"]),
        "invalid ONNX"
      )

      File.rm!(Path.join([destination, "snapshots", revision, "config.json"]))
      assert_raise Mix.Error, ~r/Could not download\/load/, fn -> Download.run(args) end
    after
      if original_cache,
        do: System.put_env("FASTEMBED_CACHE_DIR", original_cache),
        else: System.delete_env("FASTEMBED_CACHE_DIR")

      File.rm_rf!(partial_cache)
    end
  end

  defp assert_embeddings(embeddings, count, dimension) do
    assert length(embeddings) == count

    for embedding <- embeddings do
      assert length(embedding) == dimension
      assert Enum.all?(embedding, &is_float/1)

      assert_in_delta Enum.reduce(embedding, 0.0, fn value, norm -> norm + value * value end),
                      1.0,
                      1.0e-4
    end
  end
end
