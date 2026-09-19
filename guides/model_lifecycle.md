# Model memory, files, and lifecycle

ExFastembed holds one embedding session and one independent reranker session per
BEAM VM. Every process uses those same sessions. Downloaded models are stored on
disk in the Hugging Face cache layout; loading a session and storing its files
are separate operations.

## Choose an operation

| Operation | Native memory | Files on disk | Return value |
| --- | --- | --- | --- |
| `ExFastembed.load(name)` | Creates or replaces the embedding session | Downloads missing required files | `{:ok, dimension}` |
| `ExFastembed.load_reranker(name)` | Creates or replaces the reranker session | Downloads missing required files | `{:ok, true}` |
| `ExFastembed.unload()` | Drops the embedding session and its owned buffers | Keeps all files | `{:ok, true}` |
| `ExFastembed.unload_reranker()` | Drops the reranker session and its owned buffers | Keeps all files | `{:ok, true}` |
| `ExFastembed.delete_model(name, kind)` | Drops sessions using that repository in the current cache root | Physically removes the whole repository directory | `{:ok, true}` |

These operations return `{:error, reason}` on failure. Repeated unloading and
repeated deletion of an absent repository succeed. A failed model initialization
preserves the previously loaded session. Because replacement initializes the new
model before dropping the old one, allow enough RAM for both during replacement;
unload first if you prefer to free memory before loading another model.

A non-empty inference call after unloading returns an error until you load a
model again. Empty inputs continue to return `{:ok, []}`.

## Load, use, and release RAM

```elixir
{:ok, 384} = ExFastembed.load("BGESmallENV15")
{:ok, vectors} = ExFastembed.embed_text(["A document to index"])
{:ok, [%{name: "BGESmallENV15", loaded: true}]} = ExFastembed.loaded_models()

{:ok, true} = ExFastembed.unload()
{:ok, %{cached: true, loaded: false}} =
  ExFastembed.model_info("BGESmallENV15", :embedding)

# Uses the existing complete download.
{:ok, 384} = ExFastembed.load("BGESmallENV15")
```

The example assumes no reranker is loaded. `loaded_models/0` returns both
families when both are loaded, embedding first.

Unloading waits for active native work in that family and destroys its ONNX
session and owned buffers before returning. A global runtime or the allocator
can retain memory for reuse, so process RSS need not fall immediately. Embeddings
already returned to Elixir remain owned by the BEAM processes holding them.
`variant_bytes` and `disk_bytes` do not measure RAM.

## Locate the model files

```elixir
root = ExFastembed.cache_directory()
{:ok, info} = ExFastembed.model_info("BGESmallENV15", :embedding)
IO.inspect(info.path, label: "Repository directory")
IO.inspect(info.file_details, label: "Required files")
```

The effective root is chosen in this order:

1. `HF_HOME`, exported before starting the VM, overrides other settings. This
   library uses its value directly as the model cache root.
2. `FASTEMBED_CACHE_DIR`, read from Elixir on each discovery or load call.
3. `.fastembed_cache` relative to the current working directory.

`cache_directory/0` returns an absolute path and resolves existing symlinks. It
does not create directories. Changing `FASTEMBED_CACHE_DIR` affects subsequent
operations; it does not move files or unload sessions. `loaded_models/0` retains
each session's original root. `models/0` and `model_info/2` describe the current
root, so their `loaded` field is false for a session loaded from another root.

## Read metadata and sizes

`models/0` lists every supported variant without downloading files or initializing
sessions. `model_info/2` resolves one variant using the same case-insensitive
names and aliases as the loading functions. Metadata can change under concurrent
operations and reading runtime state can wait for active native work.

| Field | Meaning |
| --- | --- |
| `name`, `kind`, `repository` | Explicit variant name, `:embedding` or `:reranker`, and Hugging Face repository |
| `dimension` | Embedding dimension; `nil` for rerankers |
| `cached` | Every required weight, tokenizer/config, and additional data file is present and non-empty |
| `loaded` | This variant is loaded from the inspected cache root |
| `cache_dir` | Absolute effective cache root |
| `path` | Absolute repository directory, including when not downloaded yet |
| `revision` | Cached `refs/main` revision, or `nil` when unavailable |
| `files` | Sorted, distinct relative names of required files |
| `file_details` | Required files as `%{name: relative_name, path: snapshot_path_or_nil, size_bytes: bytes_or_nil}` |
| `variant_bytes` | Sum of all required non-empty file sizes; `nil` if any is missing, empty, unreadable, or not a regular file |
| `disk_bytes` | Sum of regular file sizes throughout the repository; `0` when absent, `nil` on filesystem read errors |

`cached: true` verifies presence, not ONNX validity or compatibility. A corrupt
non-empty file can still be marked cached and fail during loading. For an empty
file, `file_details` can contain a path with `size_bytes: nil`.

`disk_bytes` includes other variants, older revisions, reference metadata, and
partial downloads. It does not follow symlinks: snapshot links do not count the
same stored blob twice. Sizes are logical file bytes, not filesystem block
allocation, remote download estimates, or native memory usage. Unknown sizes are
`nil`, not an estimate of zero.

Variants sharing a repository repeat its `disk_bytes`. Deduplicate by `path`:

```elixir
repositories = ExFastembed.models() |> Enum.uniq_by(& &1.path)

# Retain nil values when reporting errors instead of presenting an incomplete total.
if Enum.all?(repositories, &is_integer(&1.disk_bytes)) do
  Enum.sum(Enum.map(repositories, & &1.disk_bytes))
else
  {:error, :some_repository_sizes_unavailable}
end
```

## Delete downloaded files

Stop submitting work for the affected models, then:

```elixir
{:ok, info} = ExFastembed.model_info("BGESmallENV15", :embedding)
{:ok, true} = ExFastembed.delete_model("BGESmallENV15", :embedding)
false = File.exists?(info.path)
{:ok, %{cached: false, loaded: false, disk_bytes: 0}} =
  ExFastembed.model_info("BGESmallENV15", :embedding)
```

Deletion removes the directory containing the actual model files: ONNX weights,
external weight data, tokenizer/config files, blobs, partial downloads, and
**all variants and revisions sharing the repository**. There is no separate copy
of the downloaded model retained by the library. The cache root itself and
unrelated repositories remain in place. A subsequent load downloads files again.

The selected repository is resolved from the catalog, not from an arbitrary path.
A symlink or non-directory at the repository root is rejected before unloading.
Symlinks inside a regular repository are removed without deleting their external
targets. Sessions using another cache root remain untouched.

Removal is not a filesystem transaction. If removal fails after unloading, the
session remains unloaded and some files may already be gone. Fix the reported
filesystem error and retry; an absent repository is treated as success.

## Coordinate requests in the application

The library has no request queue, job cancellation, idle timer, or worker pool.
Your application decides when to load, unload, and delete models. Before a
maintenance operation, stop admitting work and finish or cancel queued jobs at
the application level. Resume requests only after the operation returns.

Loading, inference, and unloading are serialized within each family. Concurrent
calls have no FIFO guarantee; a concurrent load can recreate a session after an
unload. Native inference already running is allowed to finish. Deletion excludes
library loads across both families while it unloads matching sessions and removes
the files. These protections are local to one VM; coordinate other VMs or tools
using the same directory separately.

## Command-line discovery

```bash
mix fastembed.models
mix fastembed.models --cached --type embedding
mix fastembed.models --type reranker
mix fastembed.download BGESmallENV15
mix fastembed.download JINARerankerV1TurboEn --reranker
```

`mix fastembed.models --loaded` shows sessions in that Mix VM only; it cannot
inspect a separately running application. The listing includes repository paths,
variant bytes, and repository bytes. A `-` in a size column means unknown.

`mix fastembed.download` skips complete downloads. Missing files are downloaded
and a session is initialized to verify loading; this can require substantial
RAM. The session belongs to that Mix VM. Applications still call `load/1` or
`load_reranker/1` before inference.
