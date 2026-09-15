#!/usr/bin/env bash
set -euo pipefail

project_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
coverage_root=$(mktemp -d)
trap 'rm -rf "$coverage_root"' EXIT

export EX_FASTEMBED_BUILD=1
export CARGO_TARGET_DIR="$project_root/_build/rust_coverage"
export CARGO_PROFILE_RELEASE_OPT_LEVEL=0
export CARGO_PROFILE_RELEASE_LTO=false
export CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16
export CARGO_PROFILE_RELEASE_DEBUG=2
export CARGO_PROFILE_RELEASE_STRIP=none
export FASTEMBED_CACHE_DIR="${FASTEMBED_CACHE_DIR:-$project_root/.fastembed_cache}"

cd "$project_root/native/ex_fastembed"
coverage_env=$(cargo llvm-cov show-env --sh)
eval "$coverage_env"
cargo llvm-cov clean --workspace
cargo test --release --locked

# Rustler writes its NIF into priv/. Keep that write in a temporary checkout.
cp -R "$project_root/lib" "$project_root/test" "$project_root/guides" "$coverage_root/"
cp "$project_root/mix.exs" "$project_root/mix.lock" "$project_root/README.md" "$coverage_root/"
ln -s "$project_root/native" "$coverage_root/native"
export MIX_DEPS_PATH="$project_root/deps"
export MIX_BUILD_PATH="$project_root/_build/elixir_coverage"
cd "$coverage_root"
mix test --include integration --cover
mkdir -p "$project_root/cover"
cp -R cover/. "$project_root/cover/"

cd "$project_root/native/ex_fastembed"
cargo llvm-cov report --release --fail-under-lines 80 --show-missing-lines
cargo llvm-cov report --release --html --output-dir "$project_root/cover/rust"
