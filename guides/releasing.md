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

## 2. Publish the native release

Commit and push the release source, then merge it into `master`. On GitHub,
create and publish the release with tag `v0.1.0` targeting that updated commit.
If the tag already exists, verify that it points to the intended release commit.
ExDoc source links use this tag, which must match the Mix and Cargo versions.

The **Build precompiled NIFs** workflow runs only when a GitHub release is
published. Branch pushes and pull requests run the separate **Checks** workflow.
Saving a draft or pushing a tag alone does not start a NIF release build.

The release workflow first checks that the tag is exactly `v0.1.0` and matches
the Mix and Cargo versions. A tag named `0.1.0` is invalid because the NIF download
URLs include the `v` prefix. This check runs before any matrix build starts.

The matrix contains four targets (Linux x86_64/aarch64, macOS Apple Silicon,
and Windows x86_64 MSVC) and NIF ABI versions 2.15 and 2.16. Its final job validates
all eight archives, generates `checksum-Elixir.ExFastembed.Native.exs` using their
SHA-256 hashes, and attaches everything to the GitHub release. The `nif-release`
artifact contains the same archives and checksum map.

## 3. Update and verify checksums

A rebuild can change archive hashes. After the release workflow succeeds,
download all published NIFs and regenerate the checksum map from those exact
archives:

```bash
EX_FASTEMBED_BUILD=1 mix rustler_precompiled.download ExFastembed.Native --all
elixir scripts/checksums.exs --check
```

For a package smoke test, extract the release run's `nif-release` artifact to
`_build/release-artifacts`. If extraction creates a nested `release-artifacts`
directory, move its archives to the top level. Then run:

```bash
elixir scripts/checksums.exs _build/release-artifacts
elixir scripts/checksums.exs --check
bash scripts/package_smoke.sh
```

The smoke test loads the packaged NIF with Rust compiler commands blocked.
Do not reuse checksums from an earlier build. Commit the generated map, then
verify the real release download in a fresh consumer without
`EX_FASTEMBED_BUILD` or a seeded NIF cache.

Push the checksum commit to `master` and publish the Hex package from that
commit. Keep the `v0.1.0` tag on the source commit that produced the release
archives. Updating checksums does not require moving the tag, republishing the
GitHub release, or rebuilding the NIFs.

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
