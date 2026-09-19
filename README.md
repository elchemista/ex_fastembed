# ExFastembed

Local text embeddings and document reranking for Elixir, powered by
[FastEmbed](https://github.com/Anush008/fastembed-rs) and ONNX Runtime.

For lifecycle examples, file-size definitions, and safe removal of downloads,
see [model memory, files, and lifecycle](guides/model_lifecycle.md).

## Installation

Add the dependency to `mix.exs`:

```elixir
{:ex_fastembed, "~> 0.1.1"}
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
Git checkouts without matching release NIFs and checksums also require a source
build. See the [release guide](guides/releasing.md) before publishing a package.

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

## Unloading from RAM and deleting model files

```elixir
ExFastembed.cache_directory()                   # Absolute effective cache root
{:ok, info} = ExFastembed.model_info("BGESmallENV15", :embedding)
{info.loaded, info.path, info.variant_bytes, info.disk_bytes}
{:ok, loaded} = ExFastembed.loaded_models()       # Includes original cache locations

{:ok, true} = ExFastembed.unload()                # Embedding session only; keeps files
{:ok, true} = ExFastembed.unload_reranker()       # Reranker session only; keeps files
# Unloads the model and physically deletes its downloaded files from disk.
{:ok, true} = ExFastembed.delete_model("BGESmallENV15", :embedding)
```

`models/0` and `model_info/2` include the required `files`, per-file snapshot paths
and byte sizes in `file_details`, and the cached revision. `variant_bytes` is the
sum of all required files, or `nil` for an incomplete download. `disk_bytes` is the
repository's regular file bytes across all variants and revisions, including
partial downloads; snapshot symlinks are not counted twice. Deduplicate by `path`
when summing repositories. Sizes describe local storage, not RAM consumption or
remote download sizes. `mix fastembed.models --loaded` filters loaded variants in
the current cache and shows paths and byte sizes.

Unloading waits for active native work and drops the ONNX session and its owned
buffers. The allocator may retain freed pages, so RSS may not fall immediately.
Previously returned vectors remain owned by their BEAM processes. **Applications
own request queues**: stop submitting work and drain your queue before unloading
or deleting. The library serializes native operations and does not cancel jobs;
concurrent operations have no guaranteed ordering.

`unload/0` and `unload_reranker/0` release native sessions from RAM and keep all
model files on disk. `delete_model/2` unloads matching sessions and physically
deletes the repository directory, including ONNX weights, tokenizer/config files,
blobs, partial downloads, and **all of its variants and revisions**. The cache
is the directory containing the actual downloaded model files. Other
repositories and models loaded from different cache roots are preserved. Loads
and deletion are coordinated within this VM; coordinate other cache users in the
application. Repository symlinks are rejected. A filesystem error can leave a
partially removed repository; fix the error and retry.

## Development

```bash
EX_FASTEMBED_BUILD=1 mix test --cover
EX_FASTEMBED_BUILD=1 mix test --include integration --cover
EX_FASTEMBED_BUILD=1 mix docs --warnings-as-errors
```

Both Elixir and Rust line coverage must meet a **90% minimum**. Run
`bash scripts/coverage.sh` for the combined native and real-inference report.

The default suite runs without downloading models. Integration tests exercise real
embedding and reranking. See [development and coverage](guides/development.md) and
the [release guide](guides/releasing.md) for the full checks.

## License

[Apache-2.0](LICENSE). Maintained by Yuriy Zhar ([elchemista](https://github.com/elchemista)).
