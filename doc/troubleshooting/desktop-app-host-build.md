# Host-native builds of desktop app crates on immutable Fedora (Bazzite 44)

Two host-only build failures hit `package/music-player/desktop-app` and `package/terminal/app` when
building natively instead of in the podman container (see `doc/decision/desktop-app-podman-build.md`).
Both are specific to the immutable-Fedora host layout (Bazzite 44,
 ostree) and do not occur in the container,
 which
has a conventional `/usr/lib` layout and a physical `HOME`.

1. bindgen cannot find `stdbool.h`:
    libclang lives in `/usr/lib64` but Fedora's clang resource headers are under
   `/usr/lib`,
    so clang's derived resource dir is wrong.
2. zig 0.15.2 fails to spawn a generated build-step binary (`uucode_build_tables`) through a broken relative path,
   because the global cache resolves through the `/home -> var/home` symlink while the build root is physical.

Both are fixed at our boundary by host-only env in each package `mise.toml` (`host_env` var).
 The details,
 the
reproductions,
 and the upstream-filing decisions follow.

## Issue 1: bindgen cannot find stdbool.h

### Symptom

Any bindgen-using `-sys` crate in these packages fails its build script.
 The bindgen consumers are `libspa-sys`,
`pipewire-sys`,
 and `audiopus_sys` (music-player) and `libghostty-vt-sys` (terminal).
 Verbatim:

```text
/usr/include/pipewire-0.3/pipewire/version.h:14:10: fatal error: 'stdbool.h' file not found

thread 'main' (35372) panicked at /home/user/.cargo/registry/src/index.crates.io-.../libspa-sys-0.8.0/build.rs:46:39:
Unable to generate bindings: ClangDiagnostic("/usr/include/pipewire-0.3/pipewire/version.h:14:10: fatal error: 'stdbool.h' file not found\n")
```

`stdbool.h` is a compiler builtin header (it ships with clang,
 not glibc),
 so the failure is about clang's resource
directory,
 not about pipewire-devel or glibc-headers.

### Root cause

clang derives its resource directory (where builtin headers like `stdbool.h`,
 `stddef.h`,
 and the intrinsics live)
from the location of the libclang it is loaded from.
 bindgen passes no `-resource-dir` by default,
 so clang uses
that derived default.

On this host the two locations disagree:

```bash
# libclang lives in lib64 (from clang-libs-22.1.5-1.fc44):
ls /usr/lib64/libclang.so*        # -> /usr/lib64/libclang.so.22.1, libclang.so.22.1.5
# but the resource headers live in lib (NOT lib64):
ls /usr/lib/clang/22/include/stdbool.h   # -> present
ls -d /usr/lib64/clang 2>/dev/null        # -> absent
```

clang,
 loaded from `/usr/lib64/libclang.so.22.1`,
 looks beside itself for `/usr/lib64/clang/22/include` and finds
nothing,
 so `stdbool.h` is unresolved.
 Fedora deliberately places the clang resource tree under `/usr/lib`,
 so the
fix is to point clang at it explicitly.

`LIBCLANG_PATH=/usr/lib64` (set in each package `mise.toml`) only tells clang-sys where to load libclang from;
 it
does not change where clang looks for resource headers.

### Verification

Versions under test:
 clang-libs 22.1.5-1.
fc44,
 bindgen 0.72.1,
 libspa-sys 0.10.0,
 host PipeWire 1.6.5.

```bash
# FAIL: no resource-dir -> stdbool.h not found (above)
mise run //package/music-player/desktop-app:lint

# PASS: point clang at the lib-side resource dir
BINDGEN_EXTRA_CLANG_ARGS="-resource-dir=/usr/lib/clang/22" mise run //package/music-player/desktop-app:lint
# -> Checking pipewire v0.10.0 ... compiles (libspa-sys bindgen succeeds)
```

Both were run in the session that produced this doc.

### Verified workaround

Set `BINDGEN_EXTRA_CLANG_ARGS` on the host branch only,
 computing the resource dir so an LLVM bump needs no edit.
This is the `hostEnv()` helper in each package's `cargo_dispatch` var:

```js
const clangDirs = globSync('/usr/lib/clang/*').sort()
const resourceDir = clangDirs[clangDirs.length - 1]
// returns { ...process.env, BINDGEN_EXTRA_CLANG_ARGS: `-resource-dir=${resourceDir}`, ... }
```

Tradeoff:
 the glob picks the highest installed clang major and assumes it is ABI-compatible with the libclang
clang-sys loads.
 That holds while a single LLVM is layered (the normal case);
 if several clang majors are layered,
pin the dir to the one matching `LIBCLANG_PATH`.
 No change to the generated bindings.

The container is unaffected:
 it installs `clang-devel`,
 which provides a conventional resource dir,
 and podman does
not forward host env into it,
 so `BINDGEN_EXTRA_CLANG_ARGS` stays host-side.

### What does not work

- Setting only `LIBCLANG_PATH=/usr/lib64`:
   locates libclang but not the resource headers.
- Relying on clang-sys auto-detection:
   it derives the resource dir from the lib64 location and misses
  `/usr/lib/clang`.
- `-isystem /usr/lib/clang/22/include` instead of `-resource-dir`:
   resolved `stdbool.h` in testing but only adds an
  include path,
   leaving clang's builtin and target search keyed to the wrong resource dir.
   `-resource-dir` sets the
  whole tree consistently,
   so it is the one used.

## Issue 2: zig spawns a build-step binary via a broken relative path

### Symptom

The terminal build fails inside `libghostty-vt-sys`'s vendored Ghostty `zig build`,
 while running a generated
codegen step.
 Verbatim:

```text
error: failed to spawn and capture stdio from ./../../../../../../var/home/user/Monochromatic/package/terminal/app/target/release/build/libghostty-vt-sys-c49e05344848fff5/out/ghostty-src/.zig-cache/o/c2021477bddae9a9776f08e7b1136e99/uucode_build_tables: FileNotFound

run exe uucode_build_tables (tables.zig) failure
...
thread 'main' (46337) panicked at /home/user/.cargo/registry/src/index.crates.io-.../libghostty-vt-sys-0.1.1/build.rs:132:5:
zig build failed with status exit status: 1
```

The spawn path is relative (`./../../../../../../var/home/...`):
 a relative prefix glued onto what should be an
absolute path.
 Only the host hits this;
 the container,
 with clean physical paths,
 does not.

### Root cause

`libghostty-vt-sys` 0.1.1 invokes `zig build` with no `--global-cache-dir`,
 so zig falls back to
`$ZIG_GLOBAL_CACHE_DIR` or `$HOME/.cache/zig`:

```rust
// libghostty-vt-sys-0.1.1/build.rs:44-49
let mut build = Command::new("zig");
build
    .arg("build")
    .arg("-Demit-lib-vt")
    .arg("--prefix")
    .arg(&install_prefix);
// build.rs:129-132 -- run() asserts success, panicking with the message above:
let status = command.status().unwrap_or_else(|error| panic!("failed to execute {context}: {error}"));
assert!(status.success(), "{context} failed with status {status}");
```

On Bazzite (ostree) the home path is a symlink while the build tree is physical:

```bash
ls -ld /home                 # -> /home -> var/home
echo $HOME                   # -> /home/user        (the symlink path)
node -e 'console.log(process.cwd())'  # -> /var/home/user/... (physical; process.cwd resolves it)
```

So zig's default global cache is `$HOME/.cache/zig` = `/home/user/.cache/zig` (symlink-rooted),
 while the in-tree
`.zig-cache` and the spawned exe sit under the physical `/var/home/...`.
 zig 0.15's build runner executes each
build-step exe with `cd <cache>/o/<hash> && <relative-path-to-exe>` (it prefixes relative path arguments).
 When the
two roots are in different path-spaces (symlink vs physical),
 the computed relative path escapes to the wrong base
and the exe is not found.

This is the residual of ziglang/zig#24216 ("running a binary relative to a setCwd broken on latest master",
 labeled
bug,
 closed 2025-06-18).
 Its main case was fixed by PR #24218 ("std.
Build.
Step.
Run:
 prefix relative path arguments
with '.
/'").
 That fix is already in our zig:

```bash
gh api repos/ziglang/zig/compare/0.15.2...36499c251c592d10a8258b1562bee22e5fb7899a --jq '.status'
# -> behind   (the #24218 merge commit is an ancestor of tag 0.15.2)
```

Yet 0.15.2 still fails here,
 matching the maintainer's note in #24216 that a related relative-`--cache-dir` bug
remained ("That looks like a separate (but definitely related) bug;
 I'll look at it later").

### Verification

Versions under test:
 zig 0.15.2 (mise),
 libghostty-vt-sys 0.1.1,
 Ghostty pin `bebca84`.

```bash
cd package/terminal/app
# FAIL: fresh default cache under the symlinked HOME, no override (proves it is not a one-time poisoned cache)
rm -rf /home/user/.cache/zig
rm -rf target/release/build/libghostty-vt-sys-*
env -u ZIG_GLOBAL_CACHE_DIR SLINT_ENABLE_EXPERIMENTAL_FEATURES=1 \
    BINDGEN_EXTRA_CLANG_ARGS="-resource-dir=/usr/lib/clang/22" \
    cargo build --release
# -> exit 101, "run exe uucode_build_tables (tables.zig) failure"

# PASS: same build with the global cache pinned to a physical path
ZIG_GLOBAL_CACHE_DIR="$PWD/target/zig-global-cache" \
    env SLINT_ENABLE_EXPERIMENTAL_FEATURES=1 \
    BINDGEN_EXTRA_CLANG_ARGS="-resource-dir=/usr/lib/clang/22" \
    cargo build --release
# -> Finished `release`; libghostty-vt.so staged; host clippy + 16 tests pass
```

The isolation (clean default cache,
 no override) reproducing the failure is what disproves the "poisoned cache"
hypothesis:
 the physical cache path is the fix,
 not the clear.

### Verified workaround

Pin `ZIG_GLOBAL_CACHE_DIR` to a physical path under the package target on the host branch (`process.cwd()`
resolves physically).
 This is part of the terminal `hostEnv()` helper in `mise.toml`:

```js
// ZIG_GLOBAL_CACHE_DIR: `${process.cwd()}/target/zig-global-cache`
```

The repo also sets the same cache for Cargo invocations that do not go through mise,
including JetBrains IDEA Rust run configurations:

```toml
# .cargo/config.toml
[env]
ZIG_GLOBAL_CACHE_DIR = { value = "package/terminal/app/target/zig-global-cache", relative = true }
SLINT_ENABLE_EXPERIMENTAL_FEATURES = "1"
```

`relative = true` is important:
 Cargo resolves the value through the config file's real path,
so opening the repo through `/home/user/Monochromatic` still yields a physical `/var/home/user/Monochromatic/...`
cache path.
 Verified with `cargo check --manifest-path package/terminal/app/Cargo.toml` from the repo root;
the direct Cargo path completed successfully after this config was added.

Tradeoff:
 a per-package zig cache,
 not shared with other zig builds,
 so the first build refetches and recompiles
Ghostty.
 Negligible,
 since terminal is the only crate using zig.
 No change to build output.
 The container is
unaffected (its cache lives in a named volume with physical container paths;
 host env is not forwarded).

### What does not work

- Clearing `~/.cache/zig` alone:
   a fresh default cache under the symlinked HOME reproduces the failure (verified
  above).
   The clear is not the fix.
- Leaving the cache at `$HOME/.cache/zig`:
   `HOME` is the symlink path,
   so the mismatch persists.
- Upgrading within 0.15.
  x:
   0.15.2 already contains #24218;
   the residual relative-path case is not covered.
- Upgrading to zig 0.16.
  x:
   not an option here.
   The vendored Ghostty (libghostty-vt-sys 0.1.1,
   pin `bebca84`)
  hard-requires zig 0.15.2 and `@compileError`s on 0.16.0,
   verified this session:

  ```text
  src/build/Config.zig:69:17: error: root source file struct 'process' has no member named 'EnvMap'
  src/build/zig.zig:13:9: error: Your Zig version v0.16.0 does not meet the required build version of v0.15.2
  ```

  So zig stays pinned at 0.15.2 (root `mise.toml`) and the `ZIG_GLOBAL_CACHE_DIR` workaround stands until a
  libghostty-vt-sys release whose vendored Ghostty supports a zig version carrying the relative-cache-dir fix.

## Upstream filing

### Issue 1: do not file

1. Really upstream's fault?
    No,
    not cleanly.
    This is the well-known Fedora `/usr/lib` vs `/usr/lib64` split
   interacting with clang's resource-dir derivation.
    The standard,
    documented remedy across the ecosystem is
   `BINDGEN_EXTRA_CLANG_ARGS` or installing the clang driver;
    it is environment configuration,
    not a bindgen defect.
2. to 5.
    Moot given (1).

Decision:
 do not file.
 The fix belongs at our boundary (`mise.toml` `host_env`),
 where it is.
 Searched
`rust-lang/rust-bindgen` for "stdbool.
h not found" and "resource-dir libclang":
 no matching actionable defect;
 the
pattern is configuration,
 consistent with constraint 1.

### Issue 2: existing issue, do not refile

1. Really upstream's fault?
    Yes.
    zig's build runner emits a wrong relative spawn path;
    #24216 is labeled bug and a
   related residual was acknowledged by a maintainer in-thread.
2. Can upstream fix it?
    Yes;
    they fixed the main case in #24218.
3. Supporting this use case?
    Yes;
    building in any directory with the default cache is supported.
4. Will they fix it?
    Plausible.
    The main bug was fixed within a day;
    the related residual was acknowledged.
5. Prototyped a minimal fix?
    No,
    and intentionally so:
    a matching upstream issue already exists (#24216,
    closed via
   #24218) and the residual relative-`--cache-dir` case was explicitly acknowledged in that thread.
    Per the
   duplicate rule,
    we neither open a competing issue nor prototype a competing patch.
    The consumer-side workaround
   (`ZIG_GLOBAL_CACHE_DIR` on a physical path) solves the user-facing problem at our boundary regardless of upstream
   movement.

Existing issue:
 [ziglang/zig#24216](https://github.com/ziglang/zig/issues/24216) (closed,
 main case fixed by
PR #24218).
 Our reproduction on 0.15.2 with an ostree symlinked-HOME global cache is a concrete instance of the
acknowledged residual.
 A search for an open follow-up ("relative cache-dir") found none,
 so there is no live thread
to comment on;
 this doc records the reproduction.
 If an open follow-up appears later,
 the draft below becomes the
additive contribution.

Draft (do not file as-is;
 kept for a future session if an open follow-up appears):

~~~md
Title: 0.15.2 build runner spawns a step exe via a broken relative path when the global cache is a symlinked path (residual of #24216)

zig 0.15.2 (which already contains #24218). On an ostree host where `/home -> var/home` and `HOME=/home/user`
(the symlink), building from the physical `/var/home/...` with the default `$HOME/.cache/zig` global cache, a
`zig build` step that runs a generated exe fails:

    error: failed to spawn and capture stdio from ./../../../../../../var/home/user/.../uucode_build_tables: FileNotFound

The spawn path is a relative prefix glued onto an absolute path, because the global cache resolves through the
`/home` symlink while the build/local-cache root is physical. #24218 ("prefix relative path arguments with './'")
is present in 0.15.2 but does not cover this case. Workaround: set `ZIG_GLOBAL_CACHE_DIR` to a physical path so
both roots share one path-space.

Repro: ostree host (or any host where `$HOME` is a symlink whose physical target differs from the build cwd root).
~~~
