#!/usr/bin/env bash
set -euo pipefail

project_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
package_mode=precompiled
if [ "${1:-}" = --source ]; then
  package_mode=source
  artifacts_dir=
else
  artifacts_dir=$(realpath "${1:-$project_root/_build/release-artifacts}")
fi
smoke_root=$(mktemp -d)
trap 'rm -rf "$smoke_root"' EXIT

cd "$project_root"
if [ "$package_mode" = precompiled ]; then
  elixir scripts/checksums.exs --check
fi
EX_FASTEMBED_BUILD=1 mix hex.build
project_version=$(awk -F'"' '/^  @version / {print $2; exit}' mix.exs)
mkdir -p "$smoke_root/archive" "$smoke_root/package" "$smoke_root/consumer" "$smoke_root/nif-cache" "$smoke_root/deny-rust"
tar -xf "ex_fastembed-$project_version.tar" -C "$smoke_root/archive"
tar -xzf "$smoke_root/archive/contents.tar.gz" -C "$smoke_root/package"
if [ "$package_mode" = precompiled ]; then
  test -f "$smoke_root/package/checksum-Elixir.ExFastembed.Native.exs"
fi
test -f "$smoke_root/package/native/ex_fastembed/Cargo.lock"
test -f "$smoke_root/package/guides/models.md"
test -f "$smoke_root/package/guides/model_lifecycle.md"
test -f "$smoke_root/package/native/ex_fastembed/src/runtime.rs"
if tar -tzf "$smoke_root/archive/contents.tar.gz" | rg '(^|/)(target|_build|deps|priv)(/|$)|\.(so|dll|dylib)$'; then
  echo 'Unexpected build artifacts in Hex package' >&2
  exit 1
fi
if [ "$package_mode" = precompiled ]; then
  cp "$artifacts_dir"/*.tar.gz "$smoke_root/nif-cache/"
fi

# Precompiled consumers must load their NIF without a Rust toolchain.
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
    rustler =
      if System.get_env("EX_FASTEMBED_BUILD") == "1",
        do: [{:rustler, "~> 0.38.0", runtime: false}],
        else: []

    [app: :package_smoke, version: "0.1.0", deps: [{:ex_fastembed, path: "../package"} | rustler]]
  end
end
ELIXIR
cat > "$smoke_root/consumer/smoke.exs" <<'ELIXIR'
if System.get_env("EX_FASTEMBED_BUILD") != "1" do
  false = Code.ensure_loaded?(Rustler)
end
50 = length(ExFastembed.models())
{:ok, 384} = ExFastembed.load("BGESmallENV15")
{:ok, [embedding]} = ExFastembed.embed_text(["Hex package smoke test"])
384 = length(embedding)
true = Enum.all?(embedding, &is_float/1)
{:ok, %{cached: true}} = ExFastembed.model_info("BGESmallENV15", :embedding)
Mix.Task.run("fastembed.models", ["--cached"])
Mix.Task.run("fastembed.download", ["BGESmallENV15"])
{:ok, [%{loaded: true}]} = ExFastembed.loaded_models()
{:ok, true} = ExFastembed.unload()
{:ok, []} = ExFastembed.loaded_models()
{:error, _} = ExFastembed.embed_text(["Unloaded"])
IO.puts("Hex package: inference, model tasks, and unload passed")
ELIXIR

unset EX_FASTEMBED_BUILD RUSTLER_PRECOMPILED_FORCE_BUILD_ALL MIX_DEPS_PATH MIX_BUILD_PATH
export MIX_ENV=prod
if [ "$package_mode" = precompiled ]; then
  export PATH="$smoke_root/deny-rust:$PATH"
else
  export EX_FASTEMBED_BUILD=1
  export CARGO_TARGET_DIR="$project_root/native/ex_fastembed/target"
fi
export RUSTLER_PRECOMPILED_GLOBAL_CACHE_PATH="$smoke_root/nif-cache"
export FASTEMBED_CACHE_DIR="${FASTEMBED_CACHE_DIR:-$project_root/.fastembed_cache}"
cd "$smoke_root/consumer"
mix deps.get
mix run smoke.exs

if [ "$package_mode" = source ]; then
  echo 'Source Hex package: isolated production consumer passed'
  exit 0
fi

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
