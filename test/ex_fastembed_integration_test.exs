defmodule ExFastembedIntegrationTest do
  # Native models are global to the VM; inference tests must run serially.
  use ExUnit.Case, async: false

  @moduletag :integration
  @moduletag timeout: 300_000

  test "embedding inference, replacement, and failed loads preserve the active model" do
    texts = ["Hello, world!", "Elixir rende semplice la concorrenza."]

    assert {:ok, 384} = ExFastembed.load("BAAI/bge-small-en-v1.5")
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
