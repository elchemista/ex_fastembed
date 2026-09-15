# Releasing to Hex

Package: `ex_fastembed`. Maintainer: **Yuriy Zhar**. Repository:
[elchemista/ex_fastembed](https://github.com/elchemista/ex_fastembed).
The intended Hex owner is `elchemista`; Hex assigns ownership to the account
that actually publishes the package.

## 1. Verify the source

Keep `mix.exs`, `native/ex_fastembed/Cargo.toml`, the native lockfile,
`CHANGELOG.md`, and the README installation version in sync.

```bash
task check
task coverage
```

All public functions must have `@doc` and `@spec`; public types must have
`@typedoc`. CI checks these contracts, the generated model guide, formatting,
static analysis, and a 90% Elixir line coverage floor. Native coverage has an
80% floor and includes real inference through the BEAM.

## 2. Build the precompiled NIFs

Run **Actions → Build precompiled NIFs → Run workflow** on the release branch.
Changes to native code, the loader, or the build workflow also start it on branch pushes. Manual and branch
runs upload artifacts without publishing a GitHub release.

The matrix contains four targets (Linux x86_64/aarch64, macOS Apple Silicon,
and Windows x86_64 MSVC) and NIF ABI versions 2.15 and 2.16. Its final
job validates that every archive is present and generates
`checksum-Elixir.ExFastembed.Native.exs` using SHA-256 hashes of the actual archives.
The `nif-release` artifact contains both the archives and the checksum map.

Download and extract that artifact to `_build/release-artifacts`. If extraction
creates a nested `release-artifacts` directory, move its archives to the top level.
Regenerate and validate the local checksum file from those exact archives:

```bash
elixir scripts/checksums.exs _build/release-artifacts
elixir scripts/checksums.exs --check
bash scripts/package_smoke.sh
```

The smoke test consumes the Hex archive and cached precompiled NIFs with Rust
compiler commands blocked. It does not depend on an unpublished release URL.

## 3. Publish the native release

Commit and push the release source, then merge it into `master`. On GitHub,
create and publish the release with tag `v0.1.0` targeting that updated commit.
If the tag already exists, verify that it points to the intended release commit.
ExDoc source links use this tag, which must match the Mix and Cargo versions.

Publishing the release starts the full NIF matrix. The final job generates the
checksum map and attaches it and all eight archives to that same GitHub release.
Saving a draft or pushing a tag alone does not start the release build.

A rebuild can change archive hashes. After the release workflow succeeds,
download all published NIFs and regenerate the checksum map from those exact
archives:

```bash
EX_FASTEMBED_BUILD=1 mix rustler_precompiled.download ExFastembed.Native --all
elixir scripts/checksums.exs --check
```

Alternatively, download the **release run's** `nif-release` artifact and regenerate
the map as in step 2. Do not reuse checksums from an earlier build. Commit the
generated map, then verify the real release download in a fresh consumer without
`EX_FASTEMBED_BUILD` or a seeded NIF cache.

## 4. Rehearse and publish Hex

Build checks require no Hex account: CI runs `mix hex.build` and
`mix docs --warnings-as-errors`. Hex's publish command requires authentication
even for a dry run. Check the active account with `mix hex.user whoami` and
authenticate as `elchemista` with `mix hex.user auth` if needed.

```bash
elixir scripts/checksums.exs --check
EX_FASTEMBED_BUILD=1 mix hex.publish --dry-run --yes
```

The dry run builds the package and HexDocs without uploading. Inspect
`ex_fastembed-0.1.0.tar` and `doc/index.html`. Confirm that the package includes
all eight checksum entries, the guides, changelog, and native source files.

Then publish both the package and docs:

```bash
mix hex.publish
```

Check [Hex](https://hex.pm/packages/ex_fastembed) and
[HexDocs](https://hexdocs.pm/ex_fastembed) after publication. Use
`mix hex.publish docs` to update documentation for an already published version.
