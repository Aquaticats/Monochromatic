# Rust hash crates: build, dispatch, and test quirks found while vetting meow's cache key hash

Found on 2026-09-17 while measuring twenty-three 128-bit hash functions for meow
 (`doc/audit/tech-meow-cache-key-hash-vet-2026-09-17-600031ed.md`).
Each item cost a failed run or a misleading result,
 and each is a property of an external crate or tool rather than of this repository.
Uncommitted by instruction of the vet brief;
 keep or delete as the vet's conclusions land.

## `blake3` 1.8.7 cannot build for aarch64 without a C cross compiler

`blake3`'s AVX-512 and NEON kernels are C and assembly compiled by `cc` in `build.rs`
 (`c/blake3_avx512_x86-64_unix.S`,
`c/blake3_neon.c`).
A cross build for `aarch64-unknown-linux-gnu` or `aarch64-unknown-linux-musl` in a container
 without `aarch64-linux-gnu-gcc` fails in the build script:

```text
warning: blake3@1.8.7: Compiler family detection failed due to error:
  ToolNotFound: failed to find tool "aarch64-linux-gnu-gcc": No such file or directory (os error 2)
error: failed to run custom build command for `blake3 v1.8.7`
```

Workaround:
 the crate's `pure` feature drops the C and assembly kernels,
 and the build then succeeds with rustc alone.
Cost:
 a `pure` build has SSE2,
SSE4.1,
and AVX2 in Rust and no aarch64 vector path at all,
 and it measured 18% to 22% slower than the C build on `x86-64-v4`.

## `keccak` 0.2.2 has no x86 SIMD backend unless a cfg flag is set

`keccak` auto-detects only the aarch64 SHA-3 extension.
Its x86 lanes are nightly `portable_simd` code behind
 `--cfg keccak_backend="simd128"`,
`"simd256"`,
or `"simd512"`
 (`src/lib.rs:1-12`,
`:57-64`).
Without one of those flags a `sha3`,
`k12`,
`turboshake`,
or `cshake` build on x86_64 is scalar,
 which is easy to mistake for "the SIMD path is slow".

## A pinned `rust-toolchain.toml` breaks offline container runs through a rustup shim

Container images that ship rustup (including `rust:slim` and images built from it)
 expose `cargo` as a rustup shim.
Inside a repository whose `rust-toolchain.toml` pins a toolchain the image does not have,
 the shim tries to download it and fails with no network:

```text
info: syncing channel updates for 1.98.0-x86_64-unknown-linux-gnu
error: could not download file from 'https://static.rust-lang.org/dist/channel-rust-1.98.0.toml.sha256'
  ... dns error: failed to lookup address information: Temporary failure in name resolution
```

Fix:
 mount a real toolchain directory and put its `bin` first on `PATH`.
The binaries in `~/.rustup/toolchains/<name>/bin` are not shims and ignore `rust-toolchain.toml`.
Setting `RUSTUP_TOOLCHAIN` also works when the image has that toolchain installed.

## Feature-gated crates make `cargo test` look green while testing nothing

`rscrypto` 0.9.0 compiles each algorithm behind a feature.
`cargo test --release --lib` with default features passed 89 tests and covered no hash module;
 with `--features std,xxh3,blake3,sha2` the same command passed 147.
Worse,
`cargo miri test --lib hashes::` exited 0 with

```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 71 filtered out
```

because no test in the default build matched the filter.
Always read the passed and filtered counts,
 not just the exit status,
 and name the features the consumer will enable.

## `xoodyak` 0.8.4 `absorb` is per message, not per chunk

`XoodyakCommon::absorb` absorbs one complete message with domain separation
 (`src/xoodyak/mod.rs:119-123`).
Calling it once per chunk of a larger input produces a different digest for every chunking.
`absorb_more` continues an absorb,
 but it splits internally at the absorb rate,
 so chunk boundaries must be multiples of that rate to match a single absorb.
A streaming wrapper therefore needs its own rate-aligned buffer.

## `cryptoxide` 0.6.5 selects SHA-2 kernels at compile time and has no x86 SHA-NI path

`src/hashing/sha2/impl256/mod.rs` chooses its kernel from `target_feature`,
 and the `x86_64` + `sha` arm is commented out at line 26.
So on any x86-64 build it uses AVX or SSE4.1 message-schedule code,
 which measured 0.47 GiB/s against 2.26 for `sha2`'s SHA-NI path on the same host,
 and its aarch64 kernel needs a build raised with `+sha2`.

## `x86-64-v4` does not enable the SHA or AES instruction features

```bash
rustc --print cfg -Ctarget-cpu=x86-64-v4 --target x86_64-unknown-linux-gnu | rg 'sha|aes|avx512f'
# target_feature="avx512f"
```

Crates that dispatch at run time (`sha2`,
`sha1`,
`aes`,
`bitcoin_hashes`,
`scytale`,
`graviola`,
`rscrypto`)
 still reach SHA-NI and AES-NI in such a build;
 crates that select at compile time do not.
Adding `-Ctarget-feature=+sha,+aes` reaches them,
 at the cost of a binary that needs a startup capability check.

## Compile-time target features matter beyond the annotated kernel

`hashcrew` 0.3.0 reports `selected_backend=Avx2` in every build,
 baseline included,
 yet it measured 0.40 of the fastest finalist on a baseline build and 0.98 on an `x86-64-v3` build.
Only the `#[target_feature(enable = "avx2")]` function is compiled with AVX2 in a baseline build;
 the code around it is not.
A backend probe alone does not explain a throughput difference between builds.

## Cross-linking aarch64 gnu with the host `rust-lld` rejects an aarch64-only flag

Linking an `aarch64-unknown-linux-gnu` binary with the x86-64 host's `rust-lld` fails with

```text
rust-lld: error: --fix-cortex-a53-843419 is only supported on AArch64 targets
```

although compilation succeeds.
`cargo build --lib` for that target avoids the link step and still proves the crate compiles;
 `aarch64-unknown-linux-musl` with `rust-lld` links without the flag.
