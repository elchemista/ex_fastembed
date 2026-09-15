#!/usr/bin/env bash
set -euo pipefail

project_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
artifacts_dir=$(realpath "${1:-$project_root/_build/release-artifacts}")
smoke_root=$(mktemp -d)
trap 'rm -rf "$smoke_root"' EXIT

cd "$project_root"
elixir scripts/checksums.exs --check
EX_FASTEMBED_BUILD=1 mix hex.build
project_version=$(awk -F'"' '/^  @version / {print $2; exit}' mix.exs)
mkdir -p "$smoke_root/archive" "$smoke_root/package" "$smoke_root/consumer" "$smoke_root/nif-cache" "$smoke_root/deny-rust"
tar -xf "ex_fastembed-$project_version.tar" -C "$smoke_root/archive"
tar -xzf "$smoke_root/archive/contents.tar.gz" -C "$smoke_root/package"
test -f "$smoke_root/package/checksum-Elixir.ExFastembed.Native.exs"
test -f "$smoke_root/package/native/ex_fastembed/Cargo.lock"
test -f "$smoke_root/package/guides/models.md"
if tar -tzf "$smoke_root/archive/contents.tar.gz" | rg '(^|/)(target|_build|deps|priv)(/|$)|\.(so|dll|dylib)$'; then
  echo 'Unexpected build artifacts in Hex package' >&2
  exit 1
fi
cp "$artifacts_dir"/*.tar.gz "$smoke_root/nif-cache/"

# A consumer of the Hex package must load its NIF without a Rust toolchain.
cat > "$smoke_root/deny-rust/cargo" <<'SH'
#!/bin/sh
echo 'Unexpected Rust compilation in precompiled package smoke test' >&2
exit 99
SH
cp "$smoke_root/deny-rust/cargo" "$smoke_root/deny-rust/rustc"
chmod +x "$smoke_root/deny-rust/"*
cat > "$smoke_root/consumer/mix.exs" <<'ELIXIR'
defmodule PackageSmoke.MixProject do
  use Mix.Project
  def project do
    [app: :package_smoke, version: "0.1.0", deps: [{:ex_fastembed, path: "../package"}]]
  end
end
ELIXIR
cat > "$smoke_root/consumer/smoke.exs" <<'ELIXIR'
false = Code.ensure_loaded?(Rustler)
50 = length(ExFastembed.models())
{:ok, 384} = ExFastembed.load("BGESmallENV15")
{:ok, [embedding]} = ExFastembed.embed_text(["Hex package smoke test"])
384 = length(embedding)
true = Enum.all?(embedding, &is_float/1)
{:ok, %{cached: true}} = ExFastembed.model_info("BGESmallENV15", :embedding)
Mix.Task.run("fastembed.models", ["--cached"])
Mix.Task.run("fastembed.download", ["BGESmallENV15"])
IO.puts("Precompiled Hex package: inference and model tasks passed without Rust")
ELIXIR

unset EX_FASTEMBED_BUILD RUSTLER_PRECOMPILED_FORCE_BUILD_ALL MIX_DEPS_PATH MIX_BUILD_PATH
export MIX_ENV=prod
export PATH="$smoke_root/deny-rust:$PATH"
export RUSTLER_PRECOMPILED_GLOBAL_CACHE_PATH="$smoke_root/nif-cache"
export FASTEMBED_CACHE_DIR="${FASTEMBED_CACHE_DIR:-$project_root/.fastembed_cache}"
cd "$smoke_root/consumer"
mix deps.get
mix run smoke.exs

# Cached archives must still be verified, including when an extracted NIF exists.
for archive in "$smoke_root/nif-cache"/*.tar.gz; do
  printf 'tampered' >> "$archive"
done
if mix deps.compile ex_fastembed --force > "$smoke_root/tampered.log" 2>&1; then
  echo 'Tampered NIF archive was incorrectly accepted' >&2
  exit 1
fi
rg -q 'checksum of files does not match' "$smoke_root/tampered.log"
cp "$artifacts_dir"/*.tar.gz "$smoke_root/nif-cache/"
mix deps.compile ex_fastembed --force
echo 'Tampered NIF archives were rejected; original archives load successfully'
