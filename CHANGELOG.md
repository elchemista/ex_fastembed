# Changelog

## 0.1.1

- Document model memory, on-disk storage, shared repository deletion, and application-owned queues in a dedicated lifecycle guide.
- Enforce 90% Elixir and Rust line coverage, including real inference in CI, and smoke-test every precompiled target.

- Upgrade fastembed-rs from 6.1.0 to 7.0.1, retaining ONNX Runtime bindings 2.0.0-rc.13.
- Add idempotent `unload/0` and `unload_reranker/0`, releasing native sessions and buffers after active operations finish.
- Serialize loading, inference, and unloading per model family; failed loads preserve the existing session.
- Add `cache_directory/0` and `loaded_models/0`, tracking each loaded model's original cache root.
- Extend catalog metadata with loaded state, cache and repository paths, revision, required files, per-file sizes, variant bytes, and repository disk bytes.
- Add `delete_model/2` to unload matching sessions and remove all cached variants of a repository without following symlinks.
- Show paths and sizes in `mix fastembed.models`, with a new `--loaded` filter.
- Keep request queues, cancellation, and admission control in the application.

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
