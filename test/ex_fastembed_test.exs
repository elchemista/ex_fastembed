defmodule ExFastembedTest do
  use ExUnit.Case, async: true

  doctest ExFastembed

  describe "embedding model functions" do
    test "embed_models/0 includes legacy names and explicit upstream variants" do
      models = ExFastembed.embed_models()

      assert models == Enum.sort_by(models, &{String.downcase(&1), &1})
      assert models == Enum.uniq(models)
      assert models == documented_models("Embedding Models", "### Reranker Models")
      assert "BAAI/bge-small-en-v1.5" in models
      assert "BGESmallENV15Q" in models
      assert "BGESmallENV15" in models
      assert "EmbeddingGemma300M" in models
      assert "BAAI/bge-m3" in models
      assert "jinaai/jina-embeddings-v2-base-code" in models
      assert "onnx-community/embeddinggemma-300m-ONNX" in models
      assert "EmbeddingGemma300MQ4" in models
      assert "snowflake/snowflake-arctic-embed-l" in models
      assert "SnowflakeArcticEmbedLQ" in models
    end

    test "load/1 with an invalid embedding model returns an error tuple" do
      assert {:error, "Model not recognized or not implemented: invalid-model"} ==
               ExFastembed.load("invalid-model")
    end

    test "load/1 with invalid input returns an error tuple" do
      assert {:error, "Model not recognized or not implemented: 123"} == ExFastembed.load(123)

      assert {:error, "Model not recognized or not implemented: <<255>>"} ==
               ExFastembed.load(<<255>>)
    end

    test "embed_text/1 validates input before calling native code" do
      assert {:error, "Invalid input: texts must be a list of strings"} ==
               ExFastembed.embed_text("Not a list")

      assert {:error, "Invalid input: texts must be a list of strings"} ==
               ExFastembed.embed_text(["doc", 123])

      assert {:error, "Invalid input: texts must be a list of strings"} ==
               ExFastembed.embed_text([<<255>>])

      assert {:ok, []} == ExFastembed.embed_text([])
    end

    test "embed_text/1 rejects improper lists without raising" do
      for tail <- [:invalid, "tail", 123] do
        assert {:error, "Invalid input: texts must be a list of strings"} ==
                 ExFastembed.embed_text(["doc" | tail])
      end
    end

    test "embed_text/1 reports when no model has been loaded" do
      assert {:error, "No model loaded. Call load/1 first."} ==
               ExFastembed.embed_text(["document"])
    end
  end

  describe "reranker model functions" do
    test "reranker_models/0 includes legacy names and corrected upstream names" do
      models = ExFastembed.reranker_models()

      assert models == Enum.sort_by(models, &{String.downcase(&1), &1})
      assert models == Enum.uniq(models)
      assert models == documented_models("Reranker Models", "## Updating the catalog")
      assert "BAAI/bge-reranker-base" in models
      assert "BGERerankerBase" in models
      assert "BAAI/bge-reranker-v2-m3" in models
      assert "rozgo/bge-reranker-v2-m3" in models
      assert "jinaai/jina-reranker-v2-base-multiligual" in models
      assert "jinaai/jina-reranker-v2-base-multilingual" in models
      assert "JINARerankerV2BaseMultilingual" in models
      assert "JINARerankerV2BaseMultiligual" in models
    end

    test "load_reranker/1 with an invalid reranker model returns an error tuple" do
      assert {:error, "Reranker model not recognized: invalid-reranker"} ==
               ExFastembed.load_reranker("invalid-reranker")
    end

    test "load_reranker/1 with invalid input returns an error tuple" do
      assert {:error, "Reranker model not recognized: 123"} == ExFastembed.load_reranker(123)

      assert {:error, "Reranker model not recognized: <<255>>"} ==
               ExFastembed.load_reranker(<<255>>)
    end

    test "rerank/3 validates input before calling native code" do
      assert {:error, "Invalid input: documents must be a list of strings"} ==
               ExFastembed.rerank("search query", ["doc1", 123], true)

      assert {:error, "Invalid input: expected a string, a list of strings, and a boolean"} ==
               ExFastembed.rerank(123, ["doc1", "doc2"], true)

      assert {:error, "Invalid input: expected a string, a list of strings, and a boolean"} ==
               ExFastembed.rerank("search query", ["doc1", "doc2"], "true")

      assert {:error, "Invalid input: query must be a valid UTF-8 string"} ==
               ExFastembed.rerank(<<255>>, ["doc1"], true)

      assert {:error, "Invalid input: documents must be a list of strings"} ==
               ExFastembed.rerank("search query", [<<255>>], true)

      assert {:ok, []} == ExFastembed.rerank("search query", [], true)
    end

    test "rerank/3 rejects improper lists without raising" do
      for tail <- [:invalid, "tail", 123] do
        assert {:error, "Invalid input: documents must be a list of strings"} ==
                 ExFastembed.rerank("query", ["doc" | tail], true)
      end
    end

    test "rerank/3 reports when no reranker has been loaded" do
      assert {:error, "No reranker loaded. Call load_reranker/1 first."} ==
               ExFastembed.rerank("query", ["document"], false)
    end
  end

  test "batch options are validated for embedding, retrieval and reranking, even with empty input" do
    for opts <- [
          nil,
          %{},
          [unknown: 1],
          [batch_size: 0],
          [batch_size: -1],
          [batch_size: 1.5],
          [batch_size: 18_446_744_073_709_551_616],
          [batch_size: "2"],
          [batch_size: 1, batch_size: 2],
          [{:batch_size, 1} | :invalid]
        ] do
      assert {:error, _} = ExFastembed.embed_text([], opts)
      assert {:error, _} = ExFastembed.embed_query("query", opts)
      assert {:error, _} = ExFastembed.embed_documents([], opts)
      assert {:error, _} = ExFastembed.rerank("query", [], false, opts)
    end

    for opts <- [[], [batch_size: nil], [batch_size: 1]] do
      assert {:ok, []} = ExFastembed.embed_text([], opts)
      assert {:ok, []} = ExFastembed.embed_documents([], opts)
      assert {:ok, []} = ExFastembed.rerank("query", [], false, opts)
    end
  end

  test "retrieval helpers validate inputs and require a session for nonempty work" do
    for query <- [nil, 1, [], <<255>>] do
      assert {:error, _} = ExFastembed.embed_query(query)
    end

    for documents <- [nil, "doc", [<<255>>], ["doc", 1], ["doc" | :invalid]] do
      assert {:error, _} = ExFastembed.embed_documents(documents)
    end

    assert {:ok, []} = ExFastembed.embed_documents([])
    assert {:error, "No model loaded. Call load/1 first."} = ExFastembed.embed_query("query")
    assert {:error, "No model loaded. Call load/1 first."} = ExFastembed.embed_documents(["doc"])
  end

  defp documented_models(heading, next_heading) do
    guide = File.read!(Path.expand("../guides/models.md", __DIR__))
    [_before, section] = String.split(guide, "### #{heading}", parts: 2)
    [section, _after] = String.split(section, next_heading, parts: 2)

    ~r/^- `"([^"]+)"`$/m
    |> Regex.scan(section, capture: :all_but_first)
    |> List.flatten()
  end
end
