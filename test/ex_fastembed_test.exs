defmodule ExFastembedTest do
  use ExUnit.Case, async: true

  describe "embedding model functions" do
    test "embed_models/0 includes legacy names and upstream v5 additions" do
      models = ExFastembed.embed_models()

      assert "BAAI/bge-small-en-v1.5" in models
      assert "BAAI/bge-m3" in models
      assert "jinaai/jina-embeddings-v2-base-code" in models
      assert "onnx-community/embeddinggemma-300m-ONNX" in models
      assert "snowflake/snowflake-arctic-embed-l" in models
      assert "SnowflakeArcticEmbedLQ" in models
    end

    test "load/1 with an invalid embedding model returns an error tuple" do
      assert {:error, "Model not recognized or not implemented: invalid-model"} ==
               ExFastembed.load("invalid-model")
    end

    test "load/1 with invalid input returns an error tuple" do
      assert {:error, "Model not recognized or not implemented: 123"} == ExFastembed.load(123)
    end

    test "embed_text/1 validates input before calling native code" do
      assert {:error, "Invalid input: texts must be a list of strings"} ==
               ExFastembed.embed_text("Not a list")

      assert {:error, "Invalid input: texts must be a list of strings"} ==
               ExFastembed.embed_text(["doc", 123])
    end
  end

  describe "reranker model functions" do
    test "reranker_models/0 includes legacy names and corrected upstream names" do
      models = ExFastembed.reranker_models()

      assert "BAAI/bge-reranker-base" in models
      assert "BAAI/bge-reranker-v2-m3" in models
      assert "rozgo/bge-reranker-v2-m3" in models
      assert "jinaai/jina-reranker-v2-base-multiligual" in models
      assert "jinaai/jina-reranker-v2-base-multilingual" in models
    end

    test "load_reranker/1 with an invalid reranker model returns an error tuple" do
      assert {:error, "Reranker model not recognized: invalid-reranker"} ==
               ExFastembed.load_reranker("invalid-reranker")
    end

    test "load_reranker/1 with invalid input returns an error tuple" do
      assert {:error, "Reranker model not recognized: 123"} == ExFastembed.load_reranker(123)
    end

    test "rerank/3 validates input before calling native code" do
      assert {:error, "Invalid input: documents must be a list of strings"} ==
               ExFastembed.rerank("search query", ["doc1", 123], true)

      assert {:error, "Invalid input: expected a string, a list of strings, and a boolean"} ==
               ExFastembed.rerank(123, ["doc1", "doc2"], true)

      assert {:error, "Invalid input: expected a string, a list of strings, and a boolean"} ==
               ExFastembed.rerank("search query", ["doc1", "doc2"], "true")
    end
  end
end
