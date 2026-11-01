# Bazel 9 native design: Rust, Zig, Kotlin, and Android

## Status and method

- Status:
   design research only;
   nothing was built,
   installed,
   or run with Bazel,
   Gradle,
   or Cargo.
- Date:
   2026-09-16.
- Repository state read:
   `main` at `52fff57ff`,
   working tree files under `/var/home/user/Monochromatic`.
- Ruleset sources read
   (shallow clones under `~/temp/agent/<name>-2026-09-16`):
  - `bazel` at `8e90a0d`
  - `rules_rust` at `dd2b107`
     (latest release `0.74.0`, 2026-08-28)
  - `rules_kotlin` at `02f96a1`
     (latest release `v2.4.10`, 2026-08-20)
  - `rules_android` at `7bd7cbb`
     (latest release `v0.7.3`, 2026-06-08)
  - `rules_android_ndk` at HEAD
     (latest release `v0.1.5`, 2026-03-19)
  - `hermetic_android_toolchains` (keith) at `941e3b5`
  - `rules_jvm_external` at `4f38388`
     (latest release `7.1`, 2026-07-23)
  - `rules_zig` at `b6e0d3a`
     (latest release `v0.16.0`, 2026-05-27)
  - `rules_fuzzing` at `ef7f8ad`
  - `rules_lint` (existing clone)
- Claim labels:
  - `[V]` verified against the cited source line or command output
  - `[U]` unverified:
     recalled or documented but not traced to source here
  - `[I]` inference from verified facts,
     not demonstrated by a build

## Shared Bazel layer

This `MODULE.bazel` sketch is the skeleton every section below plugs into.
It is a design artifact and has never been evaluated.

```starlark
# MODULE.bazel (design sketch, never evaluated)
bazel_dep(name = "rules_rust", version = "0.74.0")
bazel_dep(name = "rules_kotlin", version = "2.4.10")
bazel_dep(name = "rules_android", version = "0.7.3")
bazel_dep(name = "rules_android_ndk", version = "0.1.5")
bazel_dep(name = "hermetic_android_toolchains", version = "<current BCR>")
bazel_dep(name = "rules_jvm_external", version = "7.1")
bazel_dep(name = "contrib_rules_jvm", version = "<current BCR>")
bazel_dep(name = "rules_zig", version = "0.16.0")
bazel_dep(name = "aspect_rules_lint", version = "<current BCR>")
bazel_dep(name = "opus", version = "1.6.0")

rust = use_extension("@rules_rust//rust:extensions.bzl", "rust")
rust.toolchain(
    versions = ["nightly/2026-09-15"],
    extra_target_triples = ["aarch64-linux-android", "x86_64-linux-android"],
)
use_repo(rust, "rust_toolchains")
register_toolchains("@rust_toolchains//:all")
```

## 1. Rust

### Today: crate inventory

`rg --files --glob '**/Cargo.toml'` lists 19 manifests,
and every one carries its own `[workspace]` table
and its own `Cargo.lock` `[V]`.
The no-workspace architecture is deliberate and enforced by file-enforcer:
`doc/planning/cargo-toml-file-enforcer.md:17-31`
says collapsing the islands into one workspace is something "the architecture forbids" `[V]`.
All 19 carry the identical `[lints.clippy]` table
(`disallowed_methods = "deny"`, `implicit_return = "deny"`, `needless_return = "allow"`) `[V]`.

- `package/forbidden-strings/cli` (`forbidden-strings` 0.4.0, published):
   lib plus bin;
   `build.rs` precompiles the rule baseline into a serialized `RegexSet`
   through the `forbidden-regex` build-dependency,
   `#[path]`-including `src/rule/frx/*.rs`;
   feature `fuzzing`;
   release profile `lto`, `codegen-units = 1`, `panic = "unwind"`, `overflow-checks = true`, `strip`;
   `build-override opt-level = 3` for dev and release;
   `tests/integration.rs` uses `env!("CARGO_BIN_EXE_forbidden-strings")`.
  The release binary is consumed by path:
   `cli-git.config.ts:48` and `file-enforcer.config.ts:183-184`
   point at `package/forbidden-strings/cli/target/release/forbidden-strings` `[V]`.
- `package/nested-wayland-session/cli` (published 0.1.1):
   lib plus bin;
   `smithay = "=0.7.0"` with winit, DRM, GLES features;
   CI installs `libwayland-dev libxkbcommon-dev libdrm-dev libegl-dev clang libclang-dev`
   (`.github/workflows/cargo-publish.yml`, nws build job) `[V]`.
- `package/wg/cli-quicker-exempt`:
   bin only,
   `libc` raw syscalls;
   its `target/release` path is referenced by `package/wg/cli-quicker` tests `[V]`.
- `package/file-manager/app`:
   lib plus bin;
   `gtk4 0.11` over system GTK;
   `cfg(windows)` `windows 0.58`,
   `cfg(target_os = "macos")` `objc2` crates `[V]`.
- `package/file-manager/variant-gtk-sticky`:
   lib plus bin;
   path dependency on `file-manager`;
   `tests/wayland_boundary.rs` runs
   `../../nested-wayland-session/cli/target/release/monochromatic-nested-wayland-session`
   and its own `target/release` binary by Cargo path `[V]`.
- `package/file-manager/variant-qt`:
   bin;
   `cxx 1.0.176`, `cxx-qt 0.9.1`, `cxx-qt-lib` feature `full`;
   `build.rs` runs `cxx_qt_build::CxxQtBuilder` with a QML module and `src/qt_log.cpp`;
   `mise.toml` sets `QMAKE = "/usr/bin/qmake6"` against system Qt 6.11.1 `[V]`.
- `package/terminal/app`:
   lib plus bin;
   Slint 1.17 with `slint-build`;
   `libghostty-vt 0.1.1` whose `-sys` crate builds Ghostty with Zig (section 3);
   `portable-pty` from a `wezterm` git revision;
   `build.rs` emits `cargo:rustc-link-arg-bin=monochromatic-terminal=-Wl,-rpath,$ORIGIN/../lib/monochromatic-terminal`;
   host builds need `LIBCLANG_PATH`, `BINDGEN_EXTRA_CLANG_ARGS`, `ZIG_GLOBAL_CACHE_DIR`,
   with a podman container fallback (`package/terminal/app/mise.toml`) `[V]`.
- `package/linter/rust` (`monochromatic-rust-linter`):
   lib plus bin `rust-linter`;
   path dependencies on `rust-linter-core`, `rust-linter-plugin/builtin`, `rust-linter-pattern`;
   `ra_ap_syntax = "=0.0.335"` pinned identically in all four crates
   because `SyntaxNode` values cross crate boundaries (`package/linter/rust/Cargo.toml`) `[V]`.
- `package/rust-linter-plugin/builtin`, `package/rust-module/rust-linter-core`,
   `package/rust-module/rust-linter-pattern`:
   libraries of the linter family (section 2) `[V]`.
- `package/forbidden-strings/core` (published 0.1.0):
   lib;
   release `panic = "unwind"`, `overflow-checks = true` `[V]`.
- `package/forbidden-strings/core.bench`:
   bins `forbidden-regex-bench`, `seedless`, `dialectport`;
   path dependencies on `forbidden-regex` and `forbidden-strings`;
   release profile deliberately leaves overflow checks off "to measure the algorithm" `[V]`.
- `package/forbidden-strings/core.fuzz` and `package/forbidden-strings/cli.fuzz`:
   `cargo-fuzz` crates with 5 and 4 `fuzz_targets` bins respectively `[V]`.
- `package/music-player/truepeak-core`:
   lib;
   optional feature `service` pulls `turso 0.6` and `tokio` `[V]`.
- `package/music-player/truepeak-core.bench`:
   bins `truepeak-core-bench` and `truepeak-core-collect` `[V]`.
- `package/music-player/desktop-app` (`music-player`):
   lib plus bin;
   Slint 1.17 with `slint-build`;
   `opus` from a git revision whose `opusic-sys` bundles libopus and builds it with CMake;
   `gxhash 3` needing `-C target-feature=+aes` from `.cargo/config.toml` per triple;
   Linux `pipewire 0.10` (bindgen) and `zbus`;
   non-Linux `cpal 0.18`;
   Windows `windows 0.62` and a `winresource` build-dependency;
   `truepeak-core` with `service`;
   `i-slint-backend-testing = "=1.17.0"` dev-dependency;
   built natively on Linux, macOS, and Windows (`package/music-player/desktop-app/mise.toml`) `[V]`.
- `package/music-player/android-app/rust` (`musicplayer-native`):
   `cdylib` loaded by `System.loadLibrary("musicplayer_native")`
   (`app/src/main/kotlin/dev/monochromatic/musicplayer/NativeBridge.kt:69`);
   `jni`, `gxhash` (`+aes,+neon` from `rust/.cargo/config.toml`), git `opus`, `symphonia`,
   `ndk 0.9` feature `audio`, `truepeak-core` with `service`, `tokio`, `paranoid-android`;
   `build.rs` emits `cargo:rustc-link-lib=dylib=log` `[V]`.

Root `.cargo/config.toml` sets `ZIG_GLOBAL_CACHE_DIR` and `SLINT_ENABLE_EXPERIMENTAL_FEATURES`;
root `mise.toml` `[env]` sets `SLINT_EMIT_DEBUG_INFO = "1"`,
which changes compiled Slint UI output that the ElementHandle tests rely on `[V]`.

### Today: toolchain, lint, test, publish

- Toolchain:
   `rust = { version = "nightly", components = "clippy,rust-src,llvm-tools-preview" }`
   (`mise.toml:94`),
   floating nightly chosen for cargo-fuzz sanitizer flags;
   no `rust-toolchain.toml` exists `[V]`.
- No source file uses `#![feature(...)]`
   (`rg '#!\[feature\('` over `package/` returned nothing) `[V]`.
- Clippy:
   root `clippy.toml` bans `Result::unwrap` via `disallowed-methods`;
   each package's `lint:clippy` runs `cargo clippy --release --all-targets -- -D warnings`;
   `format:clippy` runs `cargo clippy --fix` `[V]`.
- Tests:
   every package's `test` runs `cargo nextest run --release`;
   no `.config/nextest.toml` exists `[V]`.
- Fuzzing:
   `package/forbidden-strings/cli.fuzz/mise.toml` runs
   `cargo fuzz build --fuzz-dir . --sanitizer address --target x86_64-unknown-linux-gnu`
   and `cargo fuzz run` with dual corpus directories and a dictionary `[V]`.
- Android cross-compile:
   `package/music-player/android-app/mise.toml` `build:native` runs
   `cargo ndk -t arm64-v8a -t x86_64 --platform 26 -o ../app/src/main/jniLibs build --release`,
   exporting `ANDROID_NDK_HOME` for the `opusic-sys` CMake build
   and purging poisoned CMake caches first `[V]`.
  The untracked `app/src/main/jniLibs/*/` also holds `libturso_sdk_kit*.so` copies
   that cargo-ndk swept up from `turso_sdk_kit`'s `cdylib` crate type;
   only `libmusicplayer_native.so` is loaded `[V]`.
- Publishing:
   `.github/workflows/cargo-publish.yml` runs `cargo package`,
   attests the `.crate`,
   then `cargo publish --no-verify` with crates.io Trusted Publishing
   for `forbidden-strings`, `monochromatic-nested-wayland-session`, and `forbidden-regex`,
   plus a binary matrix
   (`x86_64`/`aarch64` gnu and musl, two macOS, `x86_64-pc-windows-msvc`)
   built with plain `cargo build --release --target` `[V]`.
- Lockfile census
   (`scratchpad/lock-census.ts` over the 19 `Cargo.lock` files):
   3,616 external package entries in total,
   1,061 unique `name@version`,
   841 unique crate names,
   171 names resolved at more than one version,
   and three git sources
   (`opus` from `SpaceManiac/opus-rs`, `portable-pty` and `filedescriptor` from `wezterm/wezterm`) `[V]`.

### Bazel: toolchain

- Floating nightly is not expressible.
  `check_version_valid` fails with "iso_date must be specified if version is 'beta' or 'nightly'"
   (`rules_rust/rust/private/repository_utils.bzl:842-843`) `[V]`,
   so the design pins `nightly/<date>` in `rust.toolchain(versions = ...)`
   (`rust/extensions.bzl:236-241` documents the `nightly/2022-11-02` form) `[V]`.
- Clippy ships inside the rules_rust toolchain repository
   (`load_clippy`, `repository_utils.bzl:593-610`),
   and llvm-tools can be included for coverage (`repository_utils.bzl:412`, `438-442`) `[V]`.
  `rust-src` for IDEs maps to the rust-analyzer toolchain and `gen_rust_project`
   (`tools/rust_analyzer`), not to RustRover `[V]` for the tool, `[U]` for RustRover consumption.
- Android triples are supported std targets:
   `aarch64-linux-android` and `x86_64-linux-android`
   (`rust/platform/triple_mappings.bzl:48`, `:80`) `[V]`.
  Linking needs an NDK C toolchain from `rules_android_ndk` or `hermetic_android_toolchains` (section 6).
  rules_rust has no maintained Android example today
   (`examples/` lists no android directory;
   only `examples/cross_compile_nix` configures `aarch64-linux-android`) `[V]`,
   issue bazelbuild/rules_rust#1276 (undefined symbols linking on `aarch64-linux-android`) is open since 2022,
   and #3565 was closed by requiring
   `--@rules_rust//rust/settings:experimental_use_allocator_libraries_with_mangled_symbols=True` `[V]`.

### Bazel: third-party crates

`crate.from_cargo` keeps `Cargo.toml` and `Cargo.lock` as the source,
one `cargo_lockfile` per generated hub (`crate_universe/extensions.bzl:1227-1289`) `[V]`.
The repository's 19 independent workspaces collide with the splicer:

- Passing several manifests to one hub fails when they belong to more than one workspace root:
   `SplicerKind::new` computes each manifest's nearest ancestor with `[workspace]`
   and bails with "manifests are not allowed to from from different workspaces"
   (`crate_universe/src/splicing/splicer.rs:44-66`) `[V]`.
  Every manifest here is its own root,
   so a single hub over all 19 is refused `[V]` for the code path, `[I]` for this repository.
- Hub-per-island is allowed,
   but crate repositories are named per hub:
   `"{repo_name}__{name}-{version}"` (`crate_universe/extensions.bzl:742`) `[V]`.
  Nineteen hubs therefore materialize up to the 3,616 per-lockfile entries as separate repositories
   and compile shared crates such as `serde` once per hub `[I]`.
- Cross-island path dependencies become generated local crates inside the consuming hub
   (`SourceAnnotation::Path`, `crate_universe/src/context/crate_context.rs:822-824`) `[V]`.
  A hand-written first-party `rust_library` for `rust-linter-core` would take `ra_ap_syntax`
   from its own hub,
   while `rust-linter` would take it from the linter hub,
   producing two distinct `ra_ap_syntax` crates in one link and a type mismatch
   on `SyntaxNode` `[I]`.
  The same shape applies to `truepeak-core` shared by desktop, Android, and bench.
- Generated external crates are skipped by the Clippy aspect
   ("Ignore external targets", `rust/private/clippy.bzl:87-88`) `[V]`,
   so path crates rendered inside a hub lose Clippy coverage unless also defined first-party `[I]`.

Three designs survive,
 each with a structural cost:

- Option A, one `from_cargo` hub per island:
   preserves the no-workspace architecture and every `Cargo.lock`.
  Cost:
   duplicated repositories and compiles,
   and first-party cross-island libraries must be instantiated once per consuming hub
   (a macro stamping `rust_library` per hub) to keep type identity.
- Option B, one Bazel-only aggregator manifest
   (for example `third_party/rust/Cargo.toml` with path dependencies on the 19 crates,
   which Cargo allows because path dependencies need no workspace membership):
   one hub,
   one resolution,
   hand-written first-party `BUILD` files referencing `@crates//:<name>`.
  Cost:
   Bazel builds resolve versions and unify features across all islands
   (for example `image` with `default-features = false` in `nested-wayland-session`
   versus defaults in `file-manager`),
   diverging from the 19 `Cargo.lock` files that `cargo publish`, fuzzing, and IDEs keep using `[I]`.
- Option C, `crate.spec` declarations in `MODULE.bazel`:
   no Cargo input at all.
  Cost:
   a second dependency list beside `Cargo.toml`,
   which publishing still requires.

Recommendation within this design:
 Option B,
 because type identity and single compilation are correctness and cache properties,
 while lockfile divergence can be checked by a repository test comparing resolved versions `[I]`.

### Bazel: Cargo features that need explicit translation

- Profiles are not read.
  Map them to settings:
   `@rules_rust//rust/settings:lto` (`rust/settings/settings.bzl:62-67`),
   `codegen_units` (`settings.bzl:556-562`),
   `extra_rustc_flags` and `per_crate_rustc_flag` (`settings.bzl:463-535`),
   and target `rustc_flags` for `-Coverflow-checks=on` and `-Cpanic=unwind` `[V]`.
  Cargo applies `overflow-checks` to the whole graph of one workspace root,
   and `forbidden-regex` is compiled with checks on for the scanner
   but deliberately off for `forbidden-regex.bench`;
   in Bazel one target has one configuration,
   so per-consumer profiles need a transition or duplicated targets `[I]`.
- `build-override opt-level = 3`:
   Bazel's `--host_compilation_mode` defaults to `opt`
   (`bazel/src/main/java/com/google/devtools/build/lib/analysis/config/CoreOptions.java:385-387`)
   and rules_rust maps `opt` to `opt-level=3` (`rust/private/toolchain.bzl:813-820`) `[V]`,
   so build scripts and their build-dependencies compile optimized by default `[I]`.
- Build scripts:
   `cargo_build_script` runs with Bazel's default shell environment unless disabled
   (`cargo/private/cargo_build_script.bzl:449-457`,
   default `True` at `cargo/settings/settings.bzl:38-44`) `[V]`,
   so `pkg-config`, `qmake6`, `cmake`, and `libclang` on the host remain reachable,
   and system headers and libraries are not declared inputs `[I]`.
  `rustc-env` values containing `OUT_DIR` are rewritten to `${out_dir}`
   (`cargo/private/cargo_build_script_runner/lib.rs:246-255`),
   which covers `SLINT_INCLUDE_GENERATED` from `slint-build 1.17.0` `lib.rs:541` `[V]`.
  `cargo::metadata=` becomes `DEP_<links>_<KEY>` (`lib.rs:87-99`) `[V]`,
   the mechanism `cxx-qt-build` uses for `DEP_*_CXX_QT_MANIFEST_PATH`
   (`cxx-qt-build-0.9.1/src/dependencies.rs:42-47`) `[V]`.
- Unsupported directive:
   `rustc-link-arg-bin`, `rustc-link-arg-bins`, and `rustc-cdylib-link-arg`
   print "build script returned unsupported directive" and are dropped
   (`cargo_build_script_runner/lib.rs:101-110`) `[V]`.
  `package/terminal/app/build.rs` depends on `rustc-link-arg-bin` for its rpath,
   so the flag moves to the `rust_binary` target `[I]`.
- Per-crate knobs come from `crate.annotation`:
   `crate_features`, `rustc_flags`, `gen_build_script`, `override_target_build_script`,
   `override_target_lib`, `build_script_env`, `build_script_toolchains`, `patches`, and more
   (attribute list in `crate_universe/extensions.bzl`) `[V]`.
  Plan per native crate:
  - `gxhash`:
     `rustc_flags = ["-Ctarget-feature=+aes"]` `[I]`;
     whether crate-local `+aes` suffices where Cargo applied it graph-wide is `[U]`.
  - `opusic-sys`:
     `override_target_build_script` or `gen_build_script = False`
     plus a dependency on BCR `opus` 1.6.0 (repository bundles libopus 1.6.1),
     replacing the CMake plus `ANDROID_NDK_HOME` path `[V]` for BCR presence, `[I]` for the approach.
  - `gtk4-sys` family, `pipewire-sys`, `libspa-sys`, `gbm-sys`, `drm-sys`, `input-sys`,
     `libudev-sys`, `yeslogic-fontconfig-sys`, `alsa-sys`, `wayland-sys`:
     keep host `pkg-config` or bindgen through the default shell environment `[I]`.
  - `cxx-qt-build` and `cxx-build`:
     `build_script_env = {"QMAKE": "/usr/bin/qmake6"}`;
     no public Bazel precedent found
     (GitHub code search for `cxx-qt` in `MODULE.bazel` and `cxx_qt` in `.bazel` files returned nothing) `[V]`.
  - `libghostty-vt-sys`:
     section 3.
- Command-line-only dependency features such as `--features slint/mcp` in `terminal:mcp`
   need a separate hub or annotation,
   because annotations have no `select` over features `[I]`.
- Test environment:
   rules_rust sets `CARGO_BIN_EXE_<name>` from binary `data` dependencies to a runfiles short path
   (`rust/private/rustc.bzl:1448-1450`) `[V]`,
   which suits `forbidden-strings/tests/integration.rs:29` and `linter/rust/tests/integration.rs:46`.
  Compile-time `CARGO_MANIFEST_DIR` is `${pwd}/<package>` (`rustc.bzl:1135`) `[V]`,
   a sandbox path at compile time,
   so `env!("CARGO_MANIFEST_DIR")` fixture lookups in
   `linter/rust/tests/integration.rs:57`, `:206`,
   `file-manager-gtk-sticky/tests/wayland_boundary.rs:21`,
   and `music-player/desktop-app/src/engine_tests.rs:98`
   need runfiles lookups `[I]`.

### Bazel: Clippy

- `rust_clippy_aspect` reads a single `@rules_rust//rust/settings:clippy.toml` label flag
   (`settings.bzl:302-310`) and global `clippy_flags` (`settings.bzl:429-439`) `[V]`;
   the root `clippy.toml` fits.
- `[lints.clippy]` is read with `extract_cargo_lints`
   (`cargo/private/cargo_lints.bzl`, exported at `cargo/defs.bzl:34`)
   wired through `lint_config` (`rust/private/rust.bzl:814`, consumed at `rust/private/clippy.bzl:143-149`) `[V]`.
  Because all 19 tables are identical,
   one global `clippy_flag` set is equivalent `[I]`.
- `--all-targets` maps to running the aspect over `rust_test` targets as well `[I]`.
- `format:clippy` maps to `rules_lint`'s Clippy aspect with its patch output
   (`rules_lint/lint/rust/clippy.bzl`, `lint/rust/patcher_run.bzl`) `[V]` for presence.
- The Android crate's Clippy runs under an Android platform,
   so every analysis of it needs the NDK toolchain;
   mark it `target_compatible_with = ["@platforms//os:android"]`
   because `ndk-sys` emits `compile_error!` off Android
   (`android-app/rust/.cargo/config.toml` header comment) `[V]` for the error, `[I]` for the constraint.

### Bazel: tests and cargo-nextest

- rules_rust has no nextest integration and no sharding hooks
   (`rg 'TEST_SHARD_INDEX|nextest'` over the clone returned nothing) `[V]`.
  `rust_test` runs the libtest harness in one process with parallel threads `[U]`.
- Process-global state in tests that nextest isolates today:
  - `rust-module/rust-linter-core/src/config/load_tests.rs:320` calls `std::env::set_current_dir` `[V]`;
     concurrent tests resolving relative paths can observe the changed directory `[I]`.
  - `forbidden-strings/cli/src/frx_scan_tests.rs:92` replaces the process panic hook `[V]`.
  These need `RUST_TEST_THREADS=1` on those targets or restructuring `[I]`.

### Bazel: fuzzing

- rules_rust has no fuzzing rule (`rg --ignore-case fuzz` over `*.bzl` and docs found only a lint group name) `[V]`.
- rules_fuzzing: "The rule library currently provides support for C++ and Java fuzz tests"
   (`rules_fuzzing/README.md:7`) `[V]`.
- Design:
   keep `cargo fuzz` outside Bazel,
   or write a repository rule set:
   a transition adding cargo-fuzz's sanitizer-coverage `rustc` flags and `-Zsanitizer=address`,
   plus `libfuzzer-sys`'s C++ build through the C toolchain `[I]`.

### Bazel: publishing and release binaries

- rules_rust has no crates.io packaging or publishing rule
   (`rg 'cargo publish|cargo package'` over `*.bzl` and `*.md` returned nothing) `[V]`.
  `cargo package` verifies by compiling with Cargo,
   so the three published crates keep a working Cargo build permanently `[I]`.
- The release matrix maps to `bazel build --platforms=...` per runner,
   with musl through `examples/cross_compile_musl`-style toolchains `[V]` for the example's presence.
- The commit gate and file-enforcer call the release binary by Cargo path.
  Under Bazel the path becomes a `bazel run` or a copied output.
  One Bazel server "can handle at most one invocation at a time;
   further concurrent invocations will either block or fail-fast"
   (`bazel/docs/run/client-server.mdx:12-14`) `[V]`,
   so a commit hook calling `bazel run` waits behind any running watch build `[I]`.

## 2. Repository-owned Rust lint

### Today

- Rules:
   `max-lines` (300 code lines, lexer-based) and `require-rustdoc`,
   both non-suppressible,
   with defaults compiled from `package/rust-module/rust-linter-core/default.toml`
   that exempt `tests/`, `*_tests.rs`, `fuzz/`, `build.rs`, and fixture directories
   (`package/linter/rust/README.md`, sections "Rules" and "Exemptions") `[V]`.
- Configuration:
   `rust-linter.toml` discovered by walking up from the working directory;
   no `rust-linter.toml` exists in the repository today `[V]`.
- Invocation:
   each package's `lint:rust` runs
   `cargo run --quiet --manifest-path ../../linter/rust/Cargo.toml -- .`,
   walking the package directory with the gitignore-aware `ignore` crate;
   the root `lint:rust` fans out per package (`mise.toml:900-908`) `[V]`.
  Every fanout child builds the linter through the same `package/linter/rust/target` `[I]`.

### Bazel design

- Build `//package/linter/rust:rust-linter` as a `rust_binary` from the four linter crates.
  Under Option A this is the first place the per-hub type-identity problem appears
   (section 1, `ra_ap_syntax`) `[I]`.
- Run it two ways:
  - A `rules_lint`-style aspect over `rust_library`, `rust_binary`, `rust_test`,
     and `cargo_build_script` targets,
     passing `ctx.rule.files.srcs` explicitly,
     producing a report output group and a failing `lint_test` `[I]`.
  - A per-package `rust_linter_test(srcs = glob(["**/*.rs"]))` macro,
     because an aspect never sees `.rs` files outside any target,
     while today's directory walk does `[I]`.
- Declared inputs:
   explicit file arguments replace the directory walk,
   and any future `rust-linter.toml` must be listed as data,
   since walk-up discovery from the sandbox execroot finds no undeclared config `[I]`.
- Relative exemption globs (`**/*_tests.rs`) still match execroot-relative paths such as
   `package/forbidden-strings/cli/src/frx_scan_tests.rs` `[I]`.
- Autofix mode:
   the aspect emits a patch applied by a `bazel run` wrapper,
   the `rules_lint` pattern `[I]`.
- The linter's language server (`src/lsp.rs`) is unaffected `[I]`.

## 3. Zig

### Today

- No `.zig`, `build.zig`, or `build.zig.zon` file is tracked
   (`rg --files --hidden --no-ignore-vcs --glob '**/build.zig*' --glob '**/*.zig'` returned nothing),
   and no `zig cc` use exists `[V]`.
- The only use:
   `libghostty-vt-sys 0.1.1`, a dependency of `package/terminal/app`.
  Its `build.rs`:
  - clones `https://github.com/ghostty-org/ghostty.git`
     at commit `bebca84668947bfc92b9a30ed58712e1c34eee1d` into `OUT_DIR`
     unless `GHOSTTY_SOURCE_DIR` is set;
  - runs `zig build -Demit-lib-vt --prefix <OUT_DIR>/ghostty-install` with the Ghostty checkout as working directory;
  - emits `rustc-link-search=native=<OUT_DIR>/ghostty-install/lib` and `rustc-link-lib=dylib=ghostty-vt`;
  - maps only Linux and macOS Rust triples to Zig targets
   (`~/.cargo/registry/src/index.crates.io-*/libghostty-vt-sys-0.1.1/build.rs`) `[V]`.
- Ghostty's `build.zig.zon` in the fetched checkout lists 15 `url =` entries,
   and `package/terminal/app/target/zig-global-cache/p` holds 30 fetched packages `[V]`,
   so the build fetches Zig packages from the network at build time `[I]`.
- Zig is pinned to 0.15.2 (`mise.toml:132-138`);
   Ghostty rejects 0.16.0 (`doc/troubleshooting/desktop-app-host-build.md:310-324`) `[V]`.
- Known failure class:
   zig 0.15.2 spawns a build-step binary through a broken relative path
   when the global cache and project live in different symlink path spaces
   (`doc/troubleshooting/desktop-app-host-build.md:141-215`) `[V]`.
- The resulting `libghostty-vt.so.0` is copied beside the binary by `stage:runtime-libs:*` tasks `[V]`.

### Bazel design

- rules_zig compiles Zig sources directly:
   `zig_binary` "corresponding to `zig build-exe`" (`rules_zig/docs/rules.md:23-24`) `[V]`;
   no rule drives a foreign `build.zig` project
   (`rg 'build\.zig|zig build'` over `*.bzl` and `*.md` found only ZLS runner code) `[V]`.
- rules_zig does ship Zig 0.15.2 (`zig/private/versions.json:36`)
   and exposes `ZigToolchainInfo.zig_exe` (`zig/private/zig_toolchain.bzl:56`) `[V]`,
   so it can supply the `zig` binary to a build script `[I]`.
- hermetic_cc_toolchain provides `zig cc` as a C/C++ toolchain;
   nothing in the repository uses `zig cc`,
   so it has no role here `[I]`.
- Design:
  1. Fetch Ghostty at the pinned commit with `git_repository` or `http_archive`
     and pass it as `GHOSTTY_SOURCE_DIR` through `build_script_env` `[I]`.
  2. Pre-fetch Ghostty's Zig packages in a repository rule
     so the build action needs no network `[I]`;
     the exact Zig 0.15.2 fetch-only invocation is `[U]`.
  3. Patch `libghostty-vt-sys`'s `build.rs` through `crate.annotation(patches = ...)`
     to pass writable `--cache-dir` and `--global-cache-dir` under `OUT_DIR`,
     because the Ghostty source is a read-only input
     and `zig build` writes `.zig-cache` into its working directory `[I]`.
  4. Put the Zig toolchain on `PATH` for the build script,
     since `build.rs` calls `Command::new("zig")` and `tools` are not added to `PATH` by name
     (`cargo_build_script.bzl:846-850` documents only "Tools required by the build script")
     `[V]` for the attribute, `[I]` for `PATH`.
  5. Wrap the produced shared library in `cc_import` and give `monochromatic-terminal`
     the rpath link option dropped from `build.rs` `[I]`.
- Alternative:
   `override_target_lib` replaces the `-sys` crate with a hand-written `rust_library`
   over its checked-in `src/bindings.rs` plus a Bazel action running `zig build` `[I]`.
- Unquantified risks:
   the symlink path-space bug may recur between Bazel's execroot symlink forest and the output base `[I]`;
   rules_zig warns its cache needs `--sandbox_add_mount_pair=/tmp` on Bazel 7+
   (`rules_zig/README.md:101-115`) `[V]`;
   no public Bazel build of `libghostty-vt` was found
   (GitHub code search for `libghostty-vt` in `.bazel` files returned nothing) `[V]`.

## 4. Android

### Today: build features

From `package/music-player/android-app/settings.gradle.kts`, `build.gradle.kts`,
`app/build.gradle.kts`, `gradle.properties`, `gradle/wrapper/gradle-wrapper.properties`, and `mise.toml` `[V]`:

- Gradle 9.5.1 wrapper;
   AGP `com.android.application` 9.2.1 with its built-in Kotlin (KGP 2.2.10);
   `org.jetbrains.kotlin.plugin.compose` 2.2.10.
- No version catalog, no KSP, no kapt, no Room, no Hilt, no product flavors,
   no baseline profiles, no AAB task, `isMinifyEnabled = false` (no R8).
- `compileSdk = 37`, `targetSdk = 36`, `minSdk = 26`, `versionCode = 1`, `versionName = "0.1.0"`,
   `namespace`/`applicationId = "dev.monochromatic.musicplayer"`.
- Java and Kotlin JVM target 17.
- Dependencies:
   Compose BOM `2026.05.01`,
   `material3 1.5.0-alpha27`,
   `activity-compose 1.13.0`,
   `core-ktx 1.19.0`,
   `media3-session 1.10.1`,
   `work-runtime 2.11.2`,
   `debugImplementation` `ui-tooling`.
- No `res/` directory;
   the manifest uses `@android:style/Theme.Material.NoActionBar`,
   declares two foreground services and media permissions.
- Signing:
   release uses `signingConfigs.getByName("debug")`, the Gradle debug key.
- Native code:
   prebuilt `.so` files dropped into `app/src/main/jniLibs/<abi>/` by `build:native`.
- Host tests:
   `testDebugUnitTest`, JUnit 4, 15 `*Test.kt` files plus `FakeAudioEngine.kt`;
   `OklchColorTest.kt` imports `androidx.compose.ui.graphics.Color` from an AAR
   (note 2026-10-13: that file is deleted; other host tests such as `DeckColorTest.kt` still import Compose types).
- Device tests:
   `connectedDebugAndroidTest` and a non-destructive `test:instrumented:device`
   using `adb install -r` plus `am instrument` to keep the SAF grant and warm peak cache;
   runner `androidx.test:runner 1.7.0`, `androidx.test.ext:junit 1.2.1`, `work-testing 2.11.2`;
   `NativeBridgeTest` verifies the native fingerprint on the physical device.
- Android Lint:
   `lint` runs `./gradlew :app:lintDebug`.
- KDoc lint:
   `lint:detekt` runs the Kotlin linter's Gradle `detektCheck` over `app/src/main/kotlin`.
- IDE:
   IntelliJ IDEA Ultimate 2026.2 EAP with the Android plugin's Gradle project system
   (`.idea/AndroidProjectSystem.xml` sets `com.android.tools.idea.GradleProjectSystem`;
   `doc/troubleshooting/intellij-idea-device-manager-avd-sdk-entry.md:1-8`) `[V]`.

### Bazel mapping

- Ruleset status:
   rules_android says "This branch is a development preview of the Starlark implementation of Android rules for Bazel.
   This code is incomplete and may not function as-is."
   (`rules_android/README.md:5-6`) `[V]`.
  Bazel 9 removed the native `android_binary`, `android_library`, and `android_sdk_repository`
   (`bazel/CHANGELOG.md:2720-2721`, `:2818-2823`) `[V]`.
- Public rules:
   `aar_import`, `android_application`, `android_binary`, `android_library`, `android_local_test`,
   `android_sdk`, `android_sdk_repository`, `android_tools_defaults_jar`, `instrumented_app_info_aspect`
   (`rules_android/android/rules.bzl:65-75`) `[V]`.
- AGP 9 and Gradle:
   no equivalent;
   the build becomes `kt_android_library` plus `android_binary` `[I]`.
- Kotlin:
   rules_kotlin's current compiler is 2.4.20
   (`src/main/starlark/core/repositories/versions.bzl:29`),
   with capability templates only for 2.3 and 2.4
   (`src/main/starlark/core/repositories/kotlin/capabilities_2.3.bzl`, `capabilities_2.4.bzl`) `[V]`,
   so the app moves from Kotlin 2.2.10 to 2.4.x `[I]`.
- Compose compiler plugin:
   supported through `kt_compiler_plugin` with `target_embedded_compiler = True`
   and `kotlin-compose-compiler-plugin-embeddable`
   (`rules_kotlin/examples/jetpack_compose/BUILD:19-33`, `MODULE.bazel:50`) `[V]`.
- KSP:
   not used;
   rules_kotlin pins KSP 2.3.12 and has `examples/ksp` `[V]`.
- Maven and AAR dependencies:
   `rules_jvm_external` `maven.install` with `boms`
   (`private/extensions/maven.bzl:55`),
   `use_starlark_android_rules` and `aar_import_bzl_label` (`maven.bzl:77-78`),
   and an optional `resolver = "gradle"` (`maven.bzl:63`) for Gradle module metadata `[V]`.
  Resolution of Compose BOM `2026.05.01` with `material3 1.5.0-alpha27` is `[U]`.
- Resource processing and manifest merging:
   `android_binary` has `manifest_merger` and `manifest_values`
   (`rules/android_binary/attrs.bzl:71-72`) `[V]`;
   `minSdk`, `targetSdk`, `versionCode`, `versionName` move to `manifest_values` `[I]`.
  Merging of AAR manifests that register `androidx.startup` and WorkManager components is `[U]`;
   issues #101 ("Update manifest merger") and #445 are open `[V]`.
- Debug versus release:
   the debug flag passed to resource processing is `compilation_mode != OPT`
   (`rules/resources.bzl:787`) `[V]`,
   so debug and release become `-c fastbuild` and `-c opt` `[I]`;
   `debugImplementation(ui-tooling)` becomes a `select` on compilation mode `[I]`.
- R8:
   not needed today;
   rules_android documents R8 (`docs/r8-optimization.md`) with open correctness issues #536 and #534 `[V]`.
- AAB:
   not needed today;
   `android_application` and `rules/bundletool.bzl` exist `[V]`.
- Baseline profiles:
   not needed today;
   `rules/baseline_profiles.bzl` exists `[V]`.
- Signing:
   `debug_key` defaults to `//tools/android:debug_keystore`,
   rules_android's own `tools/android/bazel_debug.keystore`
   (`rules/android_binary/attrs.bzl:145-147`, `tools/android/BUILD:427`) `[V]`.
  An APK signed with a different key than the Gradle-installed app cannot update it in place,
   so the first Bazel install forces an uninstall that deletes the SAF grant and warm peak cache
   unless `debug_key` points at a copy of the existing Gradle debug keystore `[I]`.
- Native library packaging:
   `native_deps.process` links static `CcInfo` inputs into `lib<target>.so`
   and packages dynamic libraries under their own basenames
   (`rules/native_deps.bzl:93-160`, `_collect_unique_shared_libs` at `:192-245`) `[V]`.
  The Rust library therefore goes in as a `cc_import(shared_library = ...)` of a
   `rust_shared_library(name = "musicplayer_native")` built under the split Android platform,
   preserving `libmusicplayer_native.so` for `System.loadLibrary` `[I]`.
  APKs are zipaligned with `-P 16` for 16 KB pages (`rules/apk_packaging.bzl:330`) `[V]`.
- Rust cross-compile replacing cargo-ndk:
   `rust_shared_library` plus rules_rust Android triples plus the NDK C toolchain;
   `opusic-sys` switches to BCR `opus` (section 1);
   `build.rs`'s `-llog` becomes `linkopts` or stays in the build script `[I]`;
   rules_rust issues #1276 and #3565 apply `[V]`.
- Host unit tests:
   pure-logic tests fit `kt_jvm_test` (default runner `BazelTestRunner`,
   `rules_kotlin/kotlin/internal/jvm/jvm.bzl:415`) `[V]`;
   `OklchColorTest` (deleted 2026-10-13; `DeckColorTest` and `PlayerThemeTest` need the same)
   needs Compose classes from an AAR,
   so it goes to `android_local_test` with Robolectric or a JVM test with `android.jar` on the classpath `[I]`.
  rules_android issues #169 ("android_local_test tests are very fragile") and #560 are open `[V]`.
- Instrumentation tests:
   no `android_instrumentation_test` exists in Bazel 9 sources
   (`rg --files-with-matches instrumentation_test` over `bazel/src/main` returned nothing)
   or in rules_android's public rules `[V]`.
  Bazel's 9.1.0 documentation still describes it
   (`bazel/docs/versions/9.1.0/docs/android-instrumentation-test.mdx:12`, `:29`) `[V]`.
  rules_android #102 ("Open source android_instrumentation_test in Starlark") is open;
   #296 ("Can instrumented tests be run on a connected device?") has no answer;
   `bazelbuild/rules_utp`, cited there as a working example, is archived `[V]`.
  Design:
   build the test APK with `android_binary(instruments = ...)` and run
   `adb install -r` plus `am instrument` from a `bazel run` script,
   the same shape as today's `test:instrumented:device`,
   without Bazel test result caching `[I]`.
- Android Lint:
   no rules_android integration;
   issue #321 ("Feature Request: Android Lint Tool Integration") is open,
   and its latest comment reports a lint CLI wrapper with project XML descriptors
   averaging "7-10mins" `[V]`.
- Emulator:
   `hermetic_android_toolchains` can download emulator and system images
   (`README.md`, "Downloading emulators") `[V]`;
   whether it offers the `google_apis_playstore_ps16k` Android 37.0 image the current AVD uses is `[U]`.
- IDE:
   the JetBrains Bazel plugin's Marketplace description says "Supports Java and Kotlin"
   (`https://plugins.jetbrains.com/api/plugins/22977`) `[V]`.
  The Android plugin inside IntelliJ IDEA has no Bazel project system `[U]`.
  "Bazel for Android Studio" (plugin 9185) is released from AOSP for Android Studio only,
   each build pinned to one Android Studio build
   (latest `261.26222.65.2614.16204760`, 2026-09-08, `since` equals `until`) `[V]`;
   its Compose preview and Device Manager support is `[U]`.

## 5. Kotlin linter module

### Today

`package/linter/kotlin` (`build.gradle.kts`, `gradle.properties`, `mise.toml`) `[V]`:

- Kotlin JVM plugin 2.4.0, JVM target 17,
   `compileOnly("dev.detekt:detekt-api:2.0.0-alpha.5")`,
   a `detektCli` configuration for `detekt-cli`,
   tests on `detekt-test` (excluding `detekt-api`), `junit-jupiter 5.10.2`, `useJUnitPlatform()`.
- `detektCheck` is a `JavaExec` of `dev.detekt.cli.Main` with the built rule jar as `--plugins`.
- Publishing:
   `maven-publish` to a `file://` staging repository `build/central-staging`,
   `signing` with in-memory PGP keys,
   `withSourcesJar()`, `withJavadocJar()` filled from `README.md`,
   then `centralBundle` zips the staging repository excluding `maven-metadata.xml` and signature checksums.
- `.github/workflows/kotlin-linter-publish.yml` runs `verify:publication`,
   uploads the bundle to `https://central.sonatype.com/api/v1/publisher/upload`,
   and polls `/api/v1/publisher/status` `[V]`.

### Bazel design

- Compile:
   `kt_jvm_library` for the rules,
   with `detekt-api` through a `neverlink` wrapper so it stays out of the plugin jar `[I]`.
- Tests:
   `kt_jvm_test` defaults to the JUnit 4 `BazelTestRunner` (`jvm.bzl:415`) `[V]`;
   JUnit 5 uses `contrib_rules_jvm`'s `java_junit5_test`
   ("a drop-in replacement for `java_test` that can run JUnit5 tests",
   `bazel-contrib/rules_jvm` README) over a test-only `kt_jvm_library` `[V]` for the rule, `[I]` for the combination.
- detekt runner:
   `java_binary(main_class = "dev.detekt.cli.Main")` over `@maven//:dev_detekt_detekt_cli`,
   invoked by a test macro or aspect for `lint:detekt` consumers `[I]`.
- Publishing:
   `kt_jvm_export` "is the Kotlin JVM version of `java_export`" and generates `.publish`
   (`rules_jvm_external/private/rules/kt_jvm_export.bzl:7-37`) `[V]`.
  The publisher reads `MAVEN_REPO`, `GPG_SIGN`, `USE_IN_MEMORY_PGP_KEYS`, `PGP_SIGNING_KEY`
   (`private/tools/java/.../maven/MavenPublisher.java:111-128`),
   uploads `.md5`, `.sha1`, `.sha256`, `.sha512` sidecars (`:369-384`),
   and maintains `maven-metadata.xml` (`:260-302`) `[V]`.
  It does not speak the Central Portal upload API
   (no `central`, `portal`, or `sonatype` match in `MavenPublisher.java`) `[V]`,
   so the design publishes to a `file://` staging repository with `bazel run :detekt_rules.publish`
   and keeps the workflow's zip, metadata exclusion, upload, and polling steps `[I]`.
  The README-as-javadoc jar needs `classifier_artifacts` or the `no-javadocs` tag
   (`kt_jvm_export.bzl:50-54`) `[V]` for the tag, `[U]` for `classifier_artifacts` on `kt_jvm_export`.
- `verify:publication` and `verify:central` stay as scripts driving throwaway consumer builds `[I]`.

## 6. JDK and Android SDK provisioning

### Today

- `java = "temurin-21"` (`mise.toml:186`) `[V]`.
- `"android-sdk" = "latest"` installs only cmdline-tools and exports `ANDROID_HOME` (`mise.toml:188-197`) `[V]`.
- `prepare:android` installs `platforms;android-37.0`, `build-tools;37.0.0`, `ndk;29.0.13846066`,
   and `platform-tools` with `sdkmanager`,
   accepts licenses by piping `y`,
   and adds the two Android `rustup` targets (`mise.toml:1040-1085`) `[V]`.
- `"cargo:cargo-ndk" = "latest"` (`mise.toml:208`) `[V]`.

### Bazel design

- JDK:
   Bazel's `--java_runtime_version` defaults to `local_jdk`
   and `--tool_java_runtime_version` to `remotejdk_11`
   (`bazel/src/main/java/com/google/devtools/build/lib/rules/java/JavaOptions.java:377-378`, `:385-386`) `[V]`.
  `.bazelrc` sets both to `remotejdk_21`,
   which rules_java defines (`java/repositories.bzl:277`, `remotejdk21_linux`) `[V]`,
   plus `--java_language_version=17` `[I]`.
- Android SDK, option 1:
   rules_android's `android_sdk_repository` takes `path` or reads `ANDROID_HOME`
   (`rules/android_sdk_repository/rule.bzl:114-117`, `environ = ["ANDROID_HOME"]` at `:199`),
   creates an empty repository when neither is set,
   and never downloads anything `[V]`.
  mise `android-sdk` and `prepare:android` would stay.
- Android SDK, option 2 (preferred):
   `hermetic_android_toolchains` downloads SDK, build-tools, platform-tools, NDK, and emulator images,
   with licenses accepted through `--repo_env=ACCEPTED_ANDROID_SDK_LICENSE_VERSION` and
   `ACCEPTED_ANDROID_NDK_LICENSE_VERSION` (`README.md:13-45`) `[V]`.
  Its metadata includes SDK platform `37.0` (`sdk/versions.json:432`),
   build-tools `37.0.0` (`sdk/versions.json:64-65`),
   and NDK `29.0.13846066`, which it labels `r29-beta3` (`ndk/versions.json:30`) `[V]`.
  rules_kotlin's Compose example already overrides `@androidsdk` with it
   (`examples/jetpack_compose/MODULE.bazel:21-41`) `[V]`.
  This removes `android-sdk`, `prepare:android`, `cargo-ndk`, and the `rustup target add` step `[I]`.
- Observation:
   the repository pins an NDK build that this metadata names a beta,
   while stable `r29` is `29.0.14206865` (`ndk/versions.json:32`) `[V]`.
- Rust targets:
   `rust.toolchain(extra_target_triples = [...])` replaces `rustup target add` `[I]`.

## Yikes candidates

Ordered by how directly each one could disqualify the Bazel route or force a redesign of existing architecture.

1. Rust dependency model conflicts with the enforced no-workspace architecture.
   - Evidence:
      all 19 manifests are workspace roots,
      enforced per `doc/planning/cargo-toml-file-enforcer.md:17-31` `[V]`;
      crate_universe refuses several workspace roots in one hub
      (`crate_universe/src/splicing/splicer.rs:44-66`) `[V]`;
      hubs name crate repositories per hub (`crate_universe/extensions.bzl:742`) `[V]`;
      the four linter crates must share one `ra_ap_syntax` crate instance (`package/linter/rust/Cargo.toml`) `[V]`.
   - Why:
      Bazel either collapses the islands into one resolution
      that differs from the 19 `Cargo.lock` files Cargo keeps using,
      or keeps 19 hubs and stamps shared first-party crates once per hub to avoid split type identity `[I]`.
      Both change a decision the repository records as forbidden.
2. Android instrumentation tests have no Bazel test rule.
   - Evidence:
      no `android_instrumentation_test` in Bazel 9 `src/main` or rules_android's public rules
      (`rules_android/android/rules.bzl:65-75`) `[V]`;
      rules_android #102 open,
      #296 unanswered,
      `rules_utp` archived `[V]`;
      Bazel 9.1.0 documentation still presents the removed rule `[V]`.
   - Why:
      the owner's device tests (`NativeBridgeTest`, `RustEngineTest`, `PeakSweepWorkerTest`)
      become `bazel run` scripts outside `bazel test`,
      and the documentation contradiction is itself the kind of confusion trigger that excluded Bazel in the vet `[I]`.
3. Android Lint has no Bazel integration.
   - Evidence:
      rules_android #321 open,
      latest comment reports a CLI wrapper with project XML averaging 7 to 10 minutes `[V]`;
      no lint attribute or rule in rules_android `[V]`.
   - Why:
      `lint` (`./gradlew :app:lintDebug`) either stays on Gradle,
      keeping two Android build definitions,
      or becomes a repository-owned wrapper generating lint project descriptors `[I]`.
4. The Android IDE workflow loses its project model.
   - Evidence:
      the user edits Android in IntelliJ IDEA with `GradleProjectSystem` (`.idea/AndroidProjectSystem.xml`) `[V]`;
      the JetBrains Bazel plugin states Java and Kotlin support only `[V]`;
      "Bazel for Android Studio" targets Android Studio builds only,
      pinned per build `[V]`.
   - Why:
      Compose previews, Device Manager deploys, and resource-aware editing in IDEA depend on a Gradle model
      that a Bazel-only Android build does not provide `[U]` for feature-level behavior.
5. rules_android is a self-declared incomplete preview on the path that matters.
   - Evidence:
      `rules_android/README.md:5-6` `[V]`;
      open issues on manifest merging (#101, #445), `android_local_test` fragility (#169),
      and `kt_android_library` aspect traversal (#506) `[V]`;
      no maintained rules_rust Android example and open linking issue #1276 `[V]`.
   - Why:
      the Kotlin, Compose, AAR, manifest-merge, and Rust-JNI combination this app needs
      has no single upstream example that exercises it together `[I]`,
      so a failure surfaces only after the port.
6. Ghostty's Zig build is a networked foreign build system inside a Cargo build script.
   - Evidence:
      `libghostty-vt-sys` clones Ghostty and runs `zig build` (`build.rs`) `[V]`;
      15 direct and 30 cached Zig packages `[V]`;
      rules_zig has no `build.zig` rule (`docs/rules.md:23-24`) `[V]`;
      a recorded zig 0.15.2 path-space spawn bug (`doc/troubleshooting/desktop-app-host-build.md:141-215`) `[V]`;
      no public Bazel precedent found `[V]`.
   - Why:
      a sandboxed, cacheable build requires patching the crate,
      pre-fetching Zig packages in a repository rule,
      and relocating Zig caches,
      all repository-owned with no upstream reference `[I]`.
7. Cargo cannot be removed for Rust.
   - Evidence:
      no crates.io publish rule in rules_rust `[V]`;
      no Rust support in rules_fuzzing (`README.md:7`) `[V]`;
      `cargo package` verification compiles with Cargo (`.github/workflows/cargo-publish.yml`) `[V]`.
   - Why:
      the three published crates and both fuzz crates keep Cargo builds permanently,
      so every Rust change must stay green under two build definitions `[I]`.
8. System-library crates make cache hits depend on undeclared host state.
   - Evidence:
      `cargo_build_script` inherits the default shell environment by default
      (`cargo_build_script.bzl:449-457`, `cargo/settings/settings.bzl:38-44`) `[V]`;
      GTK, PipeWire, Wayland, DRM, Qt (`cxx-qt-build` via `qmake6`), fontconfig, and libclang
      are host-provided today `[V]`.
   - Why:
      after an `rpm-ostree` upgrade of those libraries,
      Bazel's action keys are unchanged and cached `-sys` outputs are reused `[I]`,
      which removes the sandbox-enforced-input advantage the route comparison credited to Bazel for these packages.
9. cargo-nextest's process isolation is load-bearing for some tests.
   - Evidence:
      `rust-linter-core/src/config/load_tests.rs:320` changes the process working directory;
      `forbidden-strings/src/frx_scan_tests.rs:92` swaps the process panic hook `[V]`;
      rules_rust has no nextest integration `[V]`.
   - Why:
      under a single-process libtest harness these tests can race with concurrent tests `[I]`;
      fixable per target,
      but a silent flake source during migration.
10. Silent loss of the terminal's rpath.
    - Evidence:
       `rustc-link-arg-bin` is dropped with only a warning
       (`cargo_build_script_runner/lib.rs:101-110`) `[V]`;
       `package/terminal/app/build.rs` depends on it `[V]`.
    - Why:
       the build succeeds and the binary fails at runtime to find `libghostty-vt.so.0` `[I]`.
11. Signing-key change wipes on-device state.
    - Evidence:
       rules_android's default `debug_key` is its own keystore (`rules/android_binary/attrs.bzl:145-147`) `[V]`;
       the repository's non-destructive device workflow exists to keep the SAF grant and warm cache
       (`package/music-player/android-app/mise.toml`, `test:instrumented:device`) `[V]`.
    - Why:
       the first Bazel-built install over the Gradle-built app needs an uninstall `[I]`;
       avoidable by pointing `debug_key` at the existing Gradle debug keystore.
12. Floating nightly and Kotlin 2.2 are not expressible.
    - Evidence:
       dated nightly required (`repository_utils.bzl:842-843`) `[V]`;
       rules_kotlin capability templates cover 2.3 and 2.4 only `[V]`.
    - Why:
       version policy changes (pinned nightly dates, Kotlin 2.4 for the app);
       not disqualifying on its own `[I]`.
