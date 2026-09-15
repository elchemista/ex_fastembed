# Changelog

## 0.1.0

First Hex release, with local text embeddings and document reranking.

- Upgrade FastEmbed to 6.1.0 and ONNX Runtime bindings to 2.0.0-rc.13.
- Expose repository names and explicit variants for 46 embedding models and four rerankers, preserving legacy aliases.
- Validate UTF-8 strings and improper lists without raising; preserve empty-input behavior.
- Download precompiled NIFs with RustlerPrecompiled and verify all six archives with SHA-256 checksums.
- Add `mix fastembed.models` and `mix fastembed.download`, with cache-aware variant metadata and reuse of existing model files.
- Honor `FASTEMBED_CACHE_DIR` changes made from Elixir before model discovery or loading.
- Resume incomplete caches at the same repository revision to keep weights and tokenizer files consistent.
- Document model sharing, cache configuration, replacement, and reranking results.
- Generate the model catalog from the bundled dependency and verify API docs/specs.
- Add coverage thresholds, inference tests, package validation, and current CI targets.
