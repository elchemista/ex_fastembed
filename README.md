# ExFastembed

ExFastembed is an Elixir wrapper around the [fastembed-rs](https://github.com/Anush008/fastembed-rs) crate. It provides a simple interface to load various text embedding models and reranker models, generate embeddings for a list of texts, and rerank documents based on a query.

## Installation

Add `ex_fastembed` to your list of dependencies in `mix.exs`:

```elixir
def deps do
  [
    {:ex_fastembed, "~> 0.1.1"}
  ]
end
```

Then run:

```bash
mix deps.get
```

## Usage

### Loading an Embedding Model

Before generating embeddings, you must load one of the supported text embedding models. For example, to load the `BAAI/bge-small-en-v1.5` model:

```elixir
iex> ExFastembed.load("BAAI/bge-small-en-v1.5")
#It returns either:
# {:ok, dimension } when the model is successfully loaded, or
# {:error, "Model already loaded!" } if it has been loaded previously.
```

### Generating Embeddings

After loading the embedding model, you can generate embeddings for a list of texts:

```elixir
iex> ExFastembed.embed_text(["Hello, world!", "Elixir is awesome"])
#Expected output:
# {:ok, [[...], [...]] }
```

### Loading a Reranker Model

To load a reranker model, use the `load_reranker/1` function. For example, to load `jinaai/jina-reranker-v1-turbo-en`:

```elixir
iex> ExFastembed.load_reranker("jinaai/jina-reranker-v1-turbo-en")
#It returns either:
# {:ok, true } when the model is successfully loaded, or
# {:error, "Reranker already loaded!" } if it has been loaded previously.
```

### Reranking Documents

Once the reranker model is loaded, you can rerank documents based on a query:

```elixir
iex> ExFastembed.rerank("search query", ["Document 1", "Document 2"], true)
#Expected output:
# {:ok, [{0, 0.95, "Document 1" }, {1, 0.90, "Document 2" }] }
```

## Supported Models

### Embedding Models

- `"BAAI/bge-small-en-v1.5"`
- `"sentence-transformers/all-MiniLM-L6-v2"`
- `"sentence-transformers/all-MiniLM-L12-v2"`
- `"mixedbread-ai/mxbai-embed-large-v1"`
- `"Qdrant/clip-ViT-B-32-text"`
- `"BAAI/bge-large-en-v1.5"`
- `"BAAI/bge-small-zh-v1.5"`
- `"BAAI/bge-base-en-v1.5"`
- `"sentence-transformers/paraphrase-MiniLM-L12-v2"`
- `"sentence-transformers/paraphrase-multilingual-mpnet-base-v2"`
- `"lightonai/ModernBERT-embed-large"`
- `"nomic-ai/nomic-embed-text-v1"`
- `"nomic-ai/nomic-embed-text-v1.5"`
- `"intfloat/multilingual-e5-small"`
- `"intfloat/multilingual-e5-base"`
- `"intfloat/multilingual-e5-large"`
- `"Alibaba-NLP/gte-base-en-v1.5"`
- `"Alibaba-NLP/gte-large-en-v1.5"`

### Reranker Models

- `"BAAI/bge-reranker-base"`
- `"BAAI/bge-reranker-v2-m3"`
- `"jinaai/jina-reranker-v1-turbo-en"`
- `"jinaai/jina-reranker-v2-base-multiligual"`

## License

ExFastembed is released under the Apache License 2.0. See [LICENSE](LICENSE) for details.


## Acknowledgments

ExFastembed is a wrapper around [fastembed-rs](https://github.com/Anush008/fastembed-rs), a fast Rust-based text embedding library.