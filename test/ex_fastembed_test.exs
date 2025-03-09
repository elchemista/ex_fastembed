defmodule ExFastembedTest do
  use ExUnit.Case

  # Helper to load the embedding model if not already loaded.
  defp load_embedding_model_if_needed() do
    case ExFastembed.load("BAAI/bge-small-en-v1.5") do
      {:ok, dim} ->
        assert is_integer(dim)
        :ok

      {:error, "Model already loaded!"} ->
        :ok

      other ->
        flunk("Unexpected result when loading embedding model: #{inspect(other)}")
    end
  end

  describe "Embedding model functions" do
    test "load/1 with valid embedding model 'BAAI/bge-small-en-v1.5'" do
      result = ExFastembed.load("BAAI/bge-small-en-v1.5")

      case result do
        {:ok, dim} ->
          assert is_integer(dim)

        {:error, "Model already loaded!"} ->
          assert true

        other ->
          flunk("Unexpected result: #{inspect(other)}")
      end
    end

    test "load/1 with an invalid embedding model returns an error tuple" do
      assert {:error, "Model not recognized or not implemented: invalid-model"} ==
               ExFastembed.load("invalid-model")
    end

    test "embed_text/1 with valid input (list of strings)" do
      load_embedding_model_if_needed()
      # Valid call should return a tuple with a list of embeddings.
      case ExFastembed.embed_text(["Hello", "World"]) do
        {:ok, embeddings} ->
          assert is_list(embeddings)

        {:error, reason} ->
          flunk("Unexpected error when embedding text: #{reason}")
      end
    end

    test "embed_text/1 with invalid input (not a list)" do
      assert_raise ArgumentError, fn ->
        ExFastembed.embed_text("Not a list")
      end
    end
  end

  describe "Reranker model functions" do
    test "load_reranker/1 with valid reranker model 'jinaai/jina-reranker-v1-turbo-en'" do
      result = ExFastembed.load_reranker("jinaai/jina-reranker-v1-turbo-en")

      case result do
        {:ok, true} ->
          assert true

        {:error, "Reranker already loaded!"} ->
          assert true

        other ->
          flunk("Unexpected result: #{inspect(other)}")
      end
    end

    test "load_reranker/1 with an invalid reranker model returns an error tuple" do
      assert {:error, "Reranker model not recognized: invalid-reranker"} ==
               ExFastembed.load_reranker("invalid-reranker")
    end

    test "rerank/3 with valid inputs but no reranker loaded" do
      load_embedding_model_if_needed()

      case ExFastembed.rerank("search query", ["doc1", "doc2"], true) do
        {:ok, [{0, _, "doc1"}, {1, _, "doc2"}]} ->
          assert true

        {:error, "No reranker loaded. Call load_reranker/1 first."} ->
          assert true

        other ->
          flunk("Unexpected result: #{inspect(other)}")
      end
    end

    test "rerank/3 with invalid documents (non-string in list)" do
      assert_raise ArgumentError, fn ->
        ExFastembed.rerank("search query", ["doc1", 123], true)
      end
    end

    test "rerank/3 with invalid input types" do
      # Wrong type for the query argument.
      assert_raise ArgumentError, fn ->
        ExFastembed.rerank(123, ["doc1", "doc2"], true)
      end

      # Wrong type for the return_docs argument.
      assert_raise ArgumentError, fn ->
        ExFastembed.rerank("search query", ["doc1", "doc2"], "true")
      end
    end
  end
end
