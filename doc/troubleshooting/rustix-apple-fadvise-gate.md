# rustix 1.1.4 on `aarch64-apple-darwin`: `fs::fadvise` is configured out on Apple targets, so a Linux page-cache harness fails to build with `E0433`/`E0425`

## Symptom

A crate that calls `rustix::fs::fadvise` to drop a file's page cache builds on
`x86_64-unknown-linux-gnu` and fails on `aarch64-apple-darwin`.
The call site is unchanged and the `fs` feature is enabled in both builds.

The failing source line:

```rust
// lab/src/bin/bigfile.rs:23 in the meow cache key hash vet's scratch lab crate
rustix::fs::fadvise(&file, 0, None, rustix::fs::Advice::DontNeed).expect("fadvise");
```

`cargo build --release` on macOS reports two errors against that one line,
 and
rustc points at the `cfg` that removed the item rather than at a missing feature:

```text
error: could not compile `hashvet2` (bin "bigfile") due to 2 previous errors
note: found an item that was configured out
  --> .../registry/src/index.crates.io-1949cf8c6b5b557f/rustix-1.1.4/src/fs/mod.rs:86:18
   |
75 |   #[cfg(not(any(
   |  __________-
76 | |     apple,
...  |
85 | | )))]
   | |__- the item is gated here
86 |   pub use fadvise::fadvise;
   |                    ^^^^^^^
Some errors have detailed explanations: E0425, E0433.
```

`E0433` is for the `Advice` path and `E0425` for the `fadvise` function.
Neither message names Apple in its own text;
 only the trailing
`found an item that was configured out` note does,
 which is easy to lose when
build output is piped through `tail`.

## Root cause

`posix_fadvise` is a POSIX advisory-information interface that Darwin does not
implement.
 Apple's equivalent controls are `fcntl` commands (`F_NOCACHE`,
`F_RDADVISE`,
`F_GLOBAL_NOCACHE`),
 and the symbol is simply absent:

```bash
# On macOS 27.0, MacBookAir10,1.
nm -gU /usr/lib/system/libsystem_kernel.dylib | wc -l          # 1614
nm -gU /usr/lib/system/libsystem_kernel.dylib | grep fadvise   # no output
# The same probe finds the neighbouring interfaces, so it is looking:
nm -gU /usr/lib/system/libsystem_kernel.dylib | grep -E '_fcntl$|_madvise$|_posix_madvise$'
# 00000000000015e8 T _fcntl
# 0000000000003544 T _madvise
# 0000000000003544 T _posix_madvise
```

So rustix cannot provide `fadvise` on Apple targets and gates both the module and
its re-export out.

The module gate,
 `rustix-1.1.4/src/fs/mod.rs:10-21`:

```rust
#[cfg(not(any(
    apple,
    netbsdlike,
    target_os = "dragonfly",
    target_os = "espidf",
    target_os = "haiku",
    target_os = "horizon",
    target_os = "redox",
    target_os = "solaris",
    target_os = "vita",
)))]
mod fadvise;
```

The re-export gate,
 `rustix-1.1.4/src/fs/mod.rs:75-86`:

```rust
#[cfg(not(any(
    apple,
    netbsdlike,
    target_os = "dragonfly",
    target_os = "espidf",
    target_os = "haiku",
    target_os = "horizon",
    target_os = "redox",
    target_os = "solaris",
    target_os = "vita",
)))]
pub use fadvise::fadvise;
```

The Apple replacements are present in the same module,
 under the opposite gate
(`rustix-1.1.4/src/fs/mod.rs:88-89`):

```rust
#[cfg(apple)]
pub use fcntl_apple::*;
```

and are declared in `rustix-1.1.4/src/fs/fcntl_apple.rs`:

```text
12: pub fn fcntl_rdadvise<Fd: AsFd>(fd: Fd, offset: u64, len: u64) -> io::Result<()>
24: pub fn fcntl_fullfsync<Fd: AsFd>(fd: Fd) -> io::Result<()>
44: pub fn fcntl_nocache<Fd: AsFd>(fd: Fd, value: bool) -> io::Result<()>
64: pub fn fcntl_global_nocache<Fd: AsFd>(fd: Fd, value: bool) -> io::Result<()>
```

So this is a deliberate platform gate,
 not a missing feature flag and not a
regression:
 no rustix feature turns `fadvise` on for Apple,
 because the syscall it
wraps does not exist there.

An earlier reading that was wrong:
 the first guess was that the `fs` feature or
some `linux_kernel` feature had not been enabled,
 because the visible errors are
`cannot find function` and `failed to resolve`.
Enabling more features changes nothing;
 the `cfg` above is keyed on the target OS
alone.

## Verification

Versions under test:

- `rustix` 1.1.4 from `index.crates.io`,
   fetched with `cargo fetch --locked`.
- `rustc 1.100.0-nightly (0fc141305 2026-09-11)`,
   toolchain `nightly-2026-09-12-aarch64-apple-darwin`.
- macOS 27.0 (build `26A428`),
   Darwin 27.0.0,
   `MacBookAir10,1` (Apple M1).
- Comparison:
   the crate that produced the symptom,
   the meow cache key hash vet's scratch lab,
   builds the same `bigfile` binary without complaint on
   `x86_64-unknown-linux-gnu` with the same lockfile and the same nightly.

Minimal harness,
 run on the Apple Silicon machine
 (`scripts/m1-quirks.ts` in that vet's scratch;
 logs in its `m1/quirks/`):

```toml
# Cargo.toml
[package]
name = "fadvise-probe"
version = "0.0.0"
edition = "2024"
publish = false

[workspace]

[dependencies]
rustix = { version = "=1.1.4", features = ["fs"] }
```

```bash
# Each probe body is written to src/main.rs, then:
cargo +nightly-2026-09-12 check --offline --message-format short
cargo +nightly-2026-09-12 check --offline --quiet && echo COMPILES || echo FAILS
```

Probe bodies that fail,
 with the exact diagnostic each produces:

```text
rustix::fs::fadvise(&file, 0, None, rustix::fs::Advice::DontNeed)
  src/main.rs:3:53: error[E0433]: cannot find `Advice` in `fs`: could not find `Advice` in `fs`
  src/main.rs:3:17: error[E0425]: cannot find function `fadvise` in module `rustix::fs`: not found in `rustix::fs`

let advice: rustix::fs::Advice = rustix::fs::Advice::DontNeed;
  src/main.rs:2:50: error[E0433]: cannot find `Advice` in `fs`: could not find `Advice` in `fs`
  src/main.rs:2:29: error[E0425]: cannot find type `Advice` in module `rustix::fs`: not found in `rustix::fs`

rustix::fs::copy_file_range(&a, None, &b, None, 1)
  src/main.rs:4:17: error[E0425]: cannot find function `copy_file_range` in module `rustix::fs`: not found in `rustix::fs`
```

The two error codes are not interchangeable:
 `E0433` is the path resolution of `Advice`,
 `E0425` is the value or type name itself.
`copy_file_range` produces only `E0425`,
 because it takes no gated type;
 it is gated on `linux_kernel` at
 `rustix-1.1.4/src/fs/mod.rs:60-62` rather than on `apple`,
 so it is a different gate with the same consequence off Linux.

Probe bodies that compile cleanly on the same target and feature set:

```rust
// COMPILES: all four Apple fcntl helpers in one program.
let file = std::fs::File::open("/etc/hosts").expect("open");
rustix::fs::fcntl_nocache(&file, true).expect("nocache");
rustix::fs::fcntl_global_nocache(&file, true).expect("global");
rustix::fs::fcntl_rdadvise(&file, 0, 0).expect("rdadvise");
rustix::fs::fcntl_fullfsync(&file).expect("fullfsync");
```

```rust
// COMPILES: ordinary rustix::fs items are unaffected.
let file = std::fs::File::open("/etc").expect("open");
let dir = rustix::fs::Dir::read_from(&file).expect("dir");
println!("{}", dir.count());
```

## Verified workarounds

- Drop the Linux-only binary from the macOS build by naming the binaries that do
   build,
   instead of `--bins`:

  ```bash
  cargo build --release --bin bench --bin onebyte --bin streamcheck --bin refcheck --bin backend
  ```

   Tradeoff:
   the excluded binary is silently absent on macOS,
   so any driver that
  assumes it exists must be told.
   This is the route the meow cache key hash vet
  took,
   because the excluded binary measures page-cache-cold throughput,
   which
  the vet does not rate on that machine anyway.

- Put the call behind a target gate and use the Apple `fcntl` equivalent:

  ```rust
  /// Drops this file's cached pages before a cold read.
  fn drop_cache(file: &std::fs::File) -> std::io::Result<()> {
      #[cfg(not(target_vendor = "apple"))]
      {
          rustix::fs::fadvise(file, 0, None, rustix::fs::Advice::DontNeed)?;
      }
      #[cfg(target_vendor = "apple")]
      {
          rustix::fs::fcntl_nocache(file, true)?;
      }
      Ok(())
  }
  ```

   Tradeoff:
   the two calls are not equivalent.
   `posix_fadvise(DONTNEED)` asks the
  kernel to evict pages that are already cached.
   `F_NOCACHE`,
   which is what
  `fcntl_nocache` sets,
   is documented on macOS only as
  "Turns data caching off/on.
   A non-zero value in arg turns data caching off"
  (`man 2 fcntl`),
   with no statement about pages already resident.
   A
  cold-cache benchmark written this way measures a different thing on macOS than
  on Linux,
   so the two platforms' numbers must not be compared.
   `purge(8)` evicts
  system-wide but needs root and affects the whole machine,
   which is why the vet
  did not use it.

- Gate on the capability rather than the vendor when more platforms matter:
   `#[cfg(any(target_os = "linux", target_os = "android", target_os = "freebsd"))]`
  matches the set rustix compiles `fadvise` for more closely than
  `not(target_vendor = "apple")` does.
   Tradeoff:
   the list has to be maintained
  against rustix's own gate,
   which is longer
  (`rustix-1.1.4/src/fs/mod.rs:10-21`).

## What does not work

- Adding rustix features (`fs`,
  `std`,
  `all-apis`,
   `use-libc`):
   the gate is on
  `target_os`,
   not on a feature,
   so no feature combination exposes `fadvise` on
  Apple.
   Checked by reading `rustix-1.1.4/src/fs/mod.rs:10-21` and
  `Cargo.toml`;
   no feature name appears in that `cfg`.
- Piping the build through `tail` to shorten output:
   `cargo build ... 2>&1 | tail -30`
  exits with `tail`'s status,
   so the failed build reports exit 0 and the driver
  continues as if the binaries existed.
   This cost one wasted build cycle in the
  vet,
   because the error text above scrolled past while the exit status said success.
   Reproduced on the same machine:

  ```bash
  # src/main.rs contains `fn main() { nope(); }`
  cargo check --offline 2>&1 | tail -5 > /dev/null && echo "PIPELINE REPORTS SUCCESS"
  # prints: PIPELINE REPORTS SUCCESS

  cargo check --offline --quiet > /dev/null 2>&1 && echo SUCCESS
  # prints nothing; exit status 101
  ```

   Either drop the pipe,
   set `set -o pipefail`,
   or assert the artifact after
  the build
  (`... && test -x "${CARGO_TARGET_DIR}/release/bench"`),
   which is what
  the vet's build driver now does.
- Expecting `rustix::fs::Advice` to exist so the call site can stay and only the
   call be gated:
   `Advice` is inside the gated module,
   so the type disappears with
  the function and a `use` of it fails first.

## Upstream filing decision

Nothing to file,
 and nothing was filed or commented.
The 6-constraint check stops at the first constraint:

1.  **Is it really upstream's fault?
    ** No.
     Darwin has no `posix_fadvise`;
     rustix
    gates the wrapper out on exactly the targets that lack the syscall and ships
    the Apple `fcntl` equivalents under `#[cfg(apple)]` in the same module.
     The
    surprise is entirely on the consumer side:
     a Linux-only call was written
    without a target gate.
     Constraints 2 through 6 are not reached.

The one thing that could be improved upstream is the diagnostic,
 and rustc owns
that,
 not rustix:
 the `found an item that was configured out` note already names
the gate,
 so there is no missing information,
 only output that is easy to truncate
away.
