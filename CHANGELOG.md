# Changelog

## 0.1.2

- Add upstream descriptions to model metadata, `download/2` without session initialization, batch-size options for embeddings and reranking, and E5/BGE/Nomic retrieval helpers.
- Make the download task populate files without loading a model; keep existing embedding and reranker sessions intact.
- Recover malformed cache references from remote main, handle trailing newlines consistently, snapshot loaded identities once per catalog listing, and limit cache-deletion advice to HTTP 404 errors.
- Let documentation-only publishing and reverts proceed without checksum validation; keep release scripts out of the Hex package.

- Recover from Rust initialization, inference, and cleanup panics without permanently poisoning the runtime; preserve previous sessions on initialization failures and discard sessions whose inference panics.
- Guard `mix hex.publish` with mandatory checksum validation, including offline tests for missing, stale, and malformed maps before the Hex task runs.
- Repair orphaned snapshots when `refs/main` is absent, omit invalid revisions from metadata, and explain recovery through `delete_model/2` for unusable cached revisions.
- Clarify that loading, inference, and state inspection remain serialized in this release.
- Upgrade fastembed-rs to 7.1.1, the native Rustler crate to 0.39.0, and RustlerPrecompiled to 0.10.0; refresh the compatible Rust and Hex dependency lockfiles. RustlerPrecompiled includes the fix for replacing a NIF library while another process is using it.
- Retain the Rust 1.91 minimum, the Rustler 0.38 Mix compiler, and ONNX Runtime bindings 2.0.0-rc.13. Keep hf-hub 0.5 compatible with FastEmbed's cache API.
- Adopt the corrected `JINARerankerV2BaseMultilingual` name while preserving the old `JINARerankerV2BaseMultiligual` variant and repository aliases. Model metadata now returns the corrected name.
- Repair empty snapshot files and dangling snapshot symlinks before resuming a model download, without deleting their targets or unrelated directories.
- Reject malformed cached revisions before repairing files or creating pinned references.
- Regenerate the model catalog with dimensions and repositories for all 46 embedding variants and four rerankers.
- Add regression checks for alias compatibility, catalog metadata, version consistency, and incomplete-cache recovery, including real embedding and reranking inference.
- Force the source-package smoke test to rebuild the extracted crate so normalized archive timestamps cannot reuse a stale local NIF.
- Remove the previous release's NIF checksums; generate the 0.1.2 checksums from its eight release archives before publishing to Hex.

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
