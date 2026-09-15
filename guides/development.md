# Development and coverage

## Checks

```bash
export EX_FASTEMBED_BUILD=1
mix deps.get --check-locked
mix format --check-formatted
mix compile --warnings-as-errors
mix test --cover
mix credo --strict
mix dialyzer
mix docs --warnings-as-errors
cargo fmt --check --manifest-path native/ex_fastembed/Cargo.toml
cargo test --locked --manifest-path native/ex_fastembed/Cargo.toml
cargo clippy --locked --all-targets --all-features --manifest-path native/ex_fastembed/Cargo.toml -- -D warnings
```

`task check` runs the same checks. Tests also verify that every public Elixir
function has documentation and a type specification, and that the model guide
matches the native catalog.

CI runs these checks on pull requests and pushes to `master`. A newer commit
cancels an older check run for the same pull request or branch. Rust dependencies,
ONNX Runtime, Mix dependencies, and Dialyzer PLTs are cached. Cache keys separate
toolchain versions; the library's own code is rebuilt and tested on every run.
The first run after a toolchain or dependency change may need to rebuild caches.
The eight precompiled release archives are built only when a GitHub release is
published.

## Coverage

`mix test --cover` enforces **90% Elixir line coverage** and writes HTML reports to
`cover/`. The internal native module is excluded because its Elixir bodies are fallback
stubs replaced by the compiled NIF; they do not measure the Rust implementation.

Measure the Rust implementation, including calls made through Elixir, with:

```bash
rustup component add llvm-tools-preview
cargo install cargo-llvm-cov --locked
bash scripts/coverage.sh
```

The script combines Rust unit tests with the complete ExUnit suite and real
inference tests. It enforces **80% Rust line coverage** and writes the native HTML
report to `cover/rust/html/index.html`. Test helper code and third-party crates
are excluded from the native report. An isolated Elixir build keeps the regular
NIF build unchanged.

The first integration run downloads two embedding models and one reranker:

```bash
EX_FASTEMBED_BUILD=1 mix test --include integration --cover
```

The tests check vector dimensions and normalization, model replacement, concurrent
calls, errors, ranking order, original document indices, and optional document text.

## Model catalog

After updating `native/ex_fastembed/Cargo.lock`, regenerate the guide:

```bash
EX_FASTEMBED_BUILD=1 mix run scripts/update_models.exs
```

The main API deliberately covers FastEmbed's dense text embedding and reranking
models. Optional sparse, image, and Candle model APIs are not exposed.

## Platforms

Precompiled NIFs target Linux x86_64/aarch64 with glibc, macOS Apple Silicon,
and Windows x86_64 with MSVC,
for NIF ABI versions 2.15 and 2.16. RustlerPrecompiled selects the compatible ABI
and verifies the downloaded archive against `checksum-Elixir.ExFastembed.Native.exs`.

The ONNX Runtime x86_64 binaries require an x86-64-v3 CPU (AVX2), such as Intel
Haswell or AMD Ryzen and newer. See the [upstream release notes](https://github.com/pykeio/ort/releases/tag/v2.0.0-rc.12).

Linux archives are built on Ubuntu 24.04 and require glibc 2.38+, OpenSSL 3
(`libssl.so.3` and `libcrypto.so.3`), and `libstdc++` with `GLIBCXX_3.4.31` or newer. Older distributions
need a source build with a compatible ONNX Runtime configuration.

Windows archives contain `.dll` files and are built and tested on Windows Server
2022. They require a current Windows 10/11 or Windows Server installation with
DirectML and the Microsoft Visual C++ 2015–2022 x64 runtime. The CI loads each
Windows NIF ABI and runs an actual embedding inference before uploading it.

Set `EX_FASTEMBED_BUILD=1` to compile with Rustler. Source builds require Rust
1.91+ and a C/C++ compiler. On Debian/Ubuntu, install `clang`, `libssl-dev`, and
`pkg-config`; on macOS, use `xcode-select --install`; on Windows, use Visual Studio
Build Tools with the C++ workload and the MSVC Rust toolchain. Source builds download
ONNX Runtime during compilation.

Windows GNU, macOS Intel, musl, ARMv7, and RISC-V require a compatible ONNX Runtime installation
supplied separately. Follow the [ort linking documentation](https://ort.pyke.io/setup/linking)
when configuring a custom runtime. GPU providers are not enabled by default.

## Package smoke test

```bash
bash scripts/package_smoke.sh
```

This builds the Hex archive, extracts its actual contents into a temporary
consumer, uses the matching NIF archives from `_build/release-artifacts`, and runs an
embedding inference in the production environment with Rust compiler commands
blocked. It checks that the checksum file is included and local build artifacts
are absent. Pass another artifact directory as its first argument if needed.
