# Desktop app builds run host-first, container fallback

## Decision

Cargo work for `package/terminal/app` and `package/music-player/desktop-app` runs on the host when the
native development libraries are present,
 and falls back to a CachyOS podman container (see each package's
`Containerfile`) otherwise.
This covers build,
 check,
 clippy,
 test,
 and the `format:clippy` autofix task.

Each mise task evaluates a `host_ok` predicate before dispatching:

- music-player:
   `cargo` on PATH and `pkg-config --exists libpipewire-0.3 opus fontconfig freetype2`.
- terminal:
   `cargo` and `zig` on PATH and `pkg-config --exists fontconfig freetype2`.

When the predicate holds the task runs `cargo ...` directly;
when any piece is missing it runs the identical `cargo ...` invocation inside podman,
 building the image first if
it is absent.
The toolchain pieces come from mise (the repo-wide `rust` and `zig` tools,
 plus `cargo-nextest`);
the C development libraries come from the host package manager.

On an immutable-style Fedora (the original motivation for the container) the libraries are layered once with
rpm-ostree:

```bash
rpm-ostree install pipewire-devel opus-devel fontconfig-devel freetype-devel
```

That set is the full host requirement,
 verified against both Containerfiles and both `Cargo.lock` files:

- `pipewire-devel` (music-player):
   `libspa-sys`/`pipewire-sys` run bindgen against the SPA C headers and link via
  `libpipewire-0.3.pc` / `libspa-0.2.pc`.
- `opus-devel` (music-player):
   `audiopus_sys` resolves system opus through `opus.pc`.
- `fontconfig-devel` (both):
   `yeslogic-fontconfig-sys` links system fontconfig.
- `freetype-devel` (both):
   pulled transitively because `fontconfig.pc` declares `Requires: freetype2`.

The runtime GUI/audio libraries (mesa,
 wayland,
 libxkbcommon,
 fontconfig,
 freetype) and libclang for bindgen are
already present on a KDE desktop,
 so they are not part of the rpm-ostree set.
The winit Wayland,
 xkbcommon,
 and GL bindings dlopen their libraries at runtime,
 so no `wayland-devel`,
`libxkbcommon-devel`,
 or `mesa-*-devel` is needed for the build.

Zig is not layered with rpm-ostree because Fedora packages an older 0.14;
the terminal needs Zig 0.15.
x for `libghostty-vt-sys`,
 supplied by the root mise `zig` tool pinned to 0.15.2 to
match the terminal Containerfile.

## Container stays asserted

The container path is not just a fallback;
it remains the reference build that must keep working even on a fully provisioned host.
Each package exposes `*:container` tasks (`build:container`,
 `lint:clippy:container`,
 `test:container`) that force
the podman path,
 and a `verify:container` umbrella that builds the image,
 then builds,
 clippys,
 and tests inside
the container,
 aborting on the first failure.
Run both at once from the repository root with the monorepo glob:

```bash
mise run '//package/desktop-app/...:verify:container'
```

The host path uses the repo-wide mise toolchain (currently nightly Rust),
 while the container pins rustup stable,
so `clippy -- -D warnings` can diverge between them.
`verify:container` is what guards the stable reference;
the host path is the fast development loop.

## Rejected alternatives

Manage the native dependencies with mise (including its conda backend) and never touch the host package manager.
Rejected.

mise versions tool and runtime binaries,
 not system C development libraries with headers and pkg-config `.pc`
files.
It can supply the toolchain pieces (`zig` 0.15.2,
 `rust`,
 `cargo-nextest`),
 but not `pipewire-devel`,
`opus-devel`,
 or the `*-devel` header packages.

The conda backend was evaluated separately because it self-bootstraps:
it downloads packages directly from anaconda.
org and needs no conda,
 mamba,
 or micromamba installed.
It can install some pieces (`conda:clang`,
 and `conda:libopus` at 1.6.1).
It is still rejected for two reasons:

- `pipewire` is not packaged on conda-forge at all.
  `mise ls-remote conda:pipewire` returns nothing,
   and `api.anaconda.org/package/conda-forge/pipewire` returns
  404,
   so music-player's `pipewire` and `libspa-sys` dependencies cannot be satisfied this way.
- Even for the pieces conda can supply,
   wiring `-sys` crates to conda libraries means putting `PKG_CONFIG_PATH`
  and `LIBCLANG_PATH` into mise `[env]`.
  That is unofficial,
   unsupported usage and dirty,
   and conda-forge libraries (built against conda's own sysroot)
  risk a glibc and ABI mismatch against a rustup host toolchain.

Build only in the container,
 never on the host (the previous decision).
Rejected once the host gained the `-devel` libraries:
a native build skips the podman process,
 the bind-mount,
 and the SELinux relabel,
 so the inner development loop is
faster,
 while `verify:container` still guarantees the container path.

## Consequence

Cargo tasks dispatch through the `host_ok` predicate in each package's `mise.toml`.
The host branch runs `cargo` directly with `SLINT_ENABLE_EXPERIMENTAL_FEATURES=1` and `LIBCLANG_PATH=/usr/lib64`
set in the package `[env]` (podman does not forward host env into the container,
 which carries its own copies).
The container branch shells out to `podman run ... cargo ...` against the package's image and a named cargo-cache
volume,
 building the image when missing.
The `format:clippy` task passes `--allow-no-vcs` in both branches:
it is required in the container (only the package dir is bind-mounted,
 so the repo-root `.git` is not visible) and
harmless on the host.
The binaries run on the host Wayland and PipeWire session regardless of where they were built;
the `run` task installs the host-facing binary and desktop metadata.
