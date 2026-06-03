# ExFastembed

ExFastembed is an Elixir wrapper around the [fastembed-rs](https://github.com/Anush008/fastembed-rs) crate. It provides a simple interface to load text embedding models and reranker models, generate embeddings for a list of texts, and rerank documents based on a query.

## Installation

Add `ex_fastembed` to your list of dependencies in `mix.exs`:

```elixir
def deps do
  [
    {:ex_fastembed, github: "elchemista/ex_fastembed", branch: "master"}
  ]
end
```

How to compile:

* Install clang (e.g., `sudo apt install clang`)
* Install Rust (e.g., `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)

```bash
export CC=/usr/bin/clang CXX=/usr/bin/clang++ RUSTLER_PRECOMPILATION_EXAMPLE_BUILD=1 && mix compile
```

## Usage

### Loading an Embedding Model

Before generating embeddings, you must load one of the supported text embedding models. For example, to load the `BAAI/bge-small-en-v1.5` model:

```elixir
iex> ExFastembed.load("BAAI/bge-small-en-v1.5")
{:ok, 384}
```

### Generating Embeddings

After loading the embedding model, you can generate embeddings for a list of texts:

```elixir
iex> ExFastembed.embed_text(["Hello, world!", "Elixir is awesome"])
{:ok, [[...], [...]]}
```

### Loading a Reranker Model

To load a reranker model, use the `load_reranker/1` function. For example, to load `jinaai/jina-reranker-v1-turbo-en`:

```elixir
iex> ExFastembed.load_reranker("jinaai/jina-reranker-v1-turbo-en")
{:ok, true}
```

### Reranking Documents

Once the reranker model is loaded, you can rerank documents based on a query:

```elixir
iex> ExFastembed.rerank("search query", ["Document 1", "Document 2"], true)
{:ok, [{0, 0.95, "Document 1"}, {1, 0.90, "Document 2"}]}
```

## Supported Models

The canonical source is `ExFastembed.embed_models/0` and `ExFastembed.reranker_models/0`.
The lists include legacy aliases from earlier ExFastembed versions plus upstream `fastembed-rs` v5 model names. Quantized variants whose upstream repository name is ambiguous are exposed by their enum-style variant names.

### Embedding Models

- `"BAAI/bge-small-en-v1.5"`
- `"Xenova/bge-small-en-v1.5"`
- `"Qdrant/bge-small-en-v1.5-onnx-Q"`
- `"BGESmallENV15Q"`
- `"BAAI/bge-base-en-v1.5"`
- `"Xenova/bge-base-en-v1.5"`
- `"Qdrant/bge-base-en-v1.5-onnx-Q"`
- `"BGEBaseENV15Q"`
- `"BAAI/bge-large-en-v1.5"`
- `"Xenova/bge-large-en-v1.5"`
- `"Qdrant/bge-large-en-v1.5-onnx-Q"`
- `"BGELargeENV15Q"`
- `"BAAI/bge-small-zh-v1.5"`
- `"Xenova/bge-small-zh-v1.5"`
- `"BAAI/bge-large-zh-v1.5"`
- `"Xenova/bge-large-zh-v1.5"`
- `"BAAI/bge-m3"`
- `"sentence-transformers/all-MiniLM-L6-v2"`
- `"Qdrant/all-MiniLM-L6-v2-onnx"`
- `"Xenova/all-MiniLM-L6-v2"`
- `"AllMiniLML6V2Q"`
- `"sentence-transformers/all-MiniLM-L12-v2"`
- `"Xenova/all-MiniLM-L12-v2"`
- `"AllMiniLML12V2Q"`
- `"sentence-transformers/all-mpnet-base-v2"`
- `"Xenova/all-mpnet-base-v2"`
- `"sentence-transformers/paraphrase-MiniLM-L12-v2"`
- `"sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2"`
- `"Xenova/paraphrase-multilingual-MiniLM-L12-v2"`
- `"Qdrant/paraphrase-multilingual-MiniLM-L12-v2-onnx-Q"`
- `"ParaphraseMLMiniLML12V2Q"`
- `"sentence-transformers/paraphrase-multilingual-mpnet-base-v2"`
- `"Xenova/paraphrase-multilingual-mpnet-base-v2"`
- `"lightonai/ModernBERT-embed-large"`
- `"lightonai/modernbert-embed-large"`
- `"nomic-ai/nomic-embed-text-v1"`
- `"nomic-ai/nomic-embed-text-v1.5"`
- `"NomicEmbedTextV15Q"`
- `"intfloat/multilingual-e5-small"`
- `"intfloat/multilingual-e5-base"`
- `"intfloat/multilingual-e5-large"`
- `"Qdrant/multilingual-e5-large-onnx"`
- `"mixedbread-ai/mxbai-embed-large-v1"`
- `"MxbaiEmbedLargeV1Q"`
- `"Alibaba-NLP/gte-base-en-v1.5"`
- `"GTEBaseENV15Q"`
- `"Alibaba-NLP/gte-large-en-v1.5"`
- `"GTELargeENV15Q"`
- `"Qdrant/clip-ViT-B-32-text"`
- `"jinaai/jina-embeddings-v2-base-code"`
- `"jinaai/jina-embeddings-v2-base-en"`
- `"onnx-community/embeddinggemma-300m-ONNX"`
- `"snowflake/snowflake-arctic-embed-xs"`
- `"SnowflakeArcticEmbedXSQ"`
- `"snowflake/snowflake-arctic-embed-s"`
- `"SnowflakeArcticEmbedSQ"`
- `"Snowflake/snowflake-arctic-embed-m"`
- `"snowflake/snowflake-arctic-embed-m"`
- `"SnowflakeArcticEmbedMQ"`
- `"snowflake/snowflake-arctic-embed-m-long"`
- `"SnowflakeArcticEmbedMLongQ"`
- `"snowflake/snowflake-arctic-embed-l"`
- `"SnowflakeArcticEmbedLQ"`

### Reranker Models

- `"BAAI/bge-reranker-base"`
- `"BAAI/bge-reranker-v2-m3"`
- `"rozgo/bge-reranker-v2-m3"`
- `"jinaai/jina-reranker-v1-turbo-en"`
- `"jinaai/jina-reranker-v2-base-multiligual"`
- `"jinaai/jina-reranker-v2-base-multilingual"`

## License

ExFastembed is released under the Apache License 2.0. See [LICENSE](LICENSE) for details.


## Acknowledgments

ExFastembed is a wrapper around [fastembed-rs](https://github.com/Anush008/fastembed-rs), a fast Rust-based text embedding library.
