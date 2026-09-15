# ExFastembed

Local text embeddings and document reranking for Elixir, powered by
[FastEmbed](https://github.com/Anush008/fastembed-rs) and ONNX Runtime.

## Installation

Add the dependency to `mix.exs`:

```elixir
{:ex_fastembed, "~> 0.1.0"}
```

Requires **Elixir 1.18+**. Precompiled NIFs support Linux x86_64/aarch64
(glibc 2.38+, OpenSSL 3, such as Ubuntu 24.04), macOS Apple Silicon, and
Windows x86_64 (MSVC). See [platform requirements](guides/development.md#platforms).
Installation verifies their SHA-256 checksums; Rust is not needed on these targets.

```bash
mix deps.get
mix compile
```

To build from source, add `{:rustler, "~> 0.38.0", runtime: false}` to your
application's dependencies and set `EX_FASTEMBED_BUILD=1`. This requires Rust
1.91+, a C/C++ compiler, and the [platform dependencies](guides/development.md#platforms).

## Quick start

```elixir
# Turn text into vectors. Each vector has 384 dimensions with this model.
{:ok, 384} = ExFastembed.load("BAAI/bge-small-en-v1.5")
{:ok, vectors} = ExFastembed.embed_text(["Hello, world!", "Elixir is awesome"])

# Rank documents by relevance to a query.
{:ok, true} = ExFastembed.load_reranker("jinaai/jina-reranker-v1-turbo-en")
{:ok, results} = ExFastembed.rerank(
  "What is the capital of Italy?",
  ["Rome is the capital of Italy.", "Bananas are yellow fruit."],
  true
)
```

Reranking returns `{original_index, score, document}` tuples, ordered by descending
score. Pass `false` as the last argument to return `nil` instead of document text.

## Models and runtime behavior

```bash
mix fastembed.models                        # All variants, with cache status
mix fastembed.models --cached               # Only complete local downloads
mix fastembed.models --type reranker        # Only rerankers
mix fastembed.download BGESmallENV15         # Skips models already cached
mix fastembed.download JINARerankerV1TurboEn --reranker
```

Use `ExFastembed.models/0` or `ExFastembed.model_info/2` for the same metadata in
Elixir. Cache status checks all required files locally without loading a model.

- List accepted names with `ExFastembed.embed_models/0` and `ExFastembed.reranker_models/0`.
- Names are case-insensitive. Explicit variants such as `EmbeddingGemma300MQ4` select quantization.
- Each VM shares one embedding model and one reranker. Loading another replaces it for all processes; failed loads preserve the previous model.
- Model files download on first use and are cached in `.fastembed_cache`. Set `FASTEMBED_CACHE_DIR` to change the location; an exported `HF_HOME` takes precedence.
- Empty input lists return `{:ok, []}`. Invalid input and inference failures return `{:error, reason}`.

See the [complete model catalog](guides/models.md) and the
[API documentation](https://hexdocs.pm/ex_fastembed/ExFastembed.html) for details.

## Development

```bash
EX_FASTEMBED_BUILD=1 mix test --cover
EX_FASTEMBED_BUILD=1 mix test --include integration --cover
EX_FASTEMBED_BUILD=1 mix docs --warnings-as-errors
```

The default suite runs without downloading models. Integration tests exercise real
embedding and reranking. See [development and coverage](guides/development.md) and
the [release guide](guides/releasing.md) for the full checks.

## License

[Apache-2.0](LICENSE). Maintained by Yuriy Zhar ([elchemista](https://github.com/elchemista)).
