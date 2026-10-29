# A binary built in the CachyOS container needs GLIBC_2.44 and does not start on a Fedora 44 host with glibc 2.43

Investigated on 2026-10-29 while moving the music-player build image from Fedora 41 to
`docker.io/cachyos/cachyos` (see [container-base-distro](../decision/container-base-distro.md)).

Short answer: a rolling base image ships a glibc newer than the host's, and a Rust binary that calls `sinh` or `cosh`
(through libm) is linked against the new symbol versions glibc 2.44 introduced for them.
Such a binary runs only where glibc is at least 2.44.
The `run` task of `package/music-player/desktop-app` builds in the container and starts the binary on the host,
so it broke on this host.

## Symptom

The `run` task builds in the container when the host lacks the development libraries,
then starts `target/release/music-player` on the host (glibc 2.43, Fedora 44):

```text
./target/release/music-player: /usr/lib64/libm.so.6: version `GLIBC_2.44' not found (required by ./target/release/music-player)
```

The task's comment before 2026-10-29 recorded that a binary built on the Fedora 41 image (glibc 2.40) ran on the newer host glibc;
that was not re-measured here.

## Root cause

The release binary built in the CachyOS image (glibc 2.44) lists two undefined symbols with the version `GLIBC_2.44`:

```text
$ objdump -T target/release/music-player | grep 'GLIBC_2.44'
0000000000000000      DF *UND*	0000000000000000 (GLIBC_2.44) sinh
0000000000000000      DF *UND*	0000000000000000 (GLIBC_2.44) cosh
```

glibc 2.44 gave `cosh` and `sinh` a new symbol version in libm.
`math/Versions` on the `release/2.44/master` branch
(<https://sourceware.org/git/?p=glibc.git;a=blob_plain;f=math/Versions;hb=refs/heads/release/2.44/master>, lines 721 to 725):

```text
  GLIBC_2.44 {
    # No SVID compatible error handling.
    cosh;
    sinh;
  }
```

The release notes (`NEWS` on the same branch) say why:
"The SVID error handling for cosh and sinh was moved to compatibility symbols, allowing improvements in performance."
A dynamic linker refuses a binary that needs a version its libm does not define, so a host with glibc 2.43 cannot start it.
Only these two symbols exceed 2.43 in this binary, so any program that links the default `sinh` or `cosh` from a glibc 2.44 build is affected,
and any later glibc release may add more versions the same way.

## Verification

- Host: glibc 2.43 (`ldd --version`: `ldd (GNU libc) 2.43`).
- Container: `docker.io/cachyos/cachyos-v4:latest` pulled 2026-10-29, `ldd (GNU libc) 2.44`.
- Failing build: `cargo build --release` inside the image, then running the binary on the host prints the error in the symptom.
- Working build: `cargo zigbuild --release --target x86_64-unknown-linux-gnu.2.39` inside the same image
   (Zig 0.15.2, cargo-zigbuild 0.24.0), then `objdump -T` on the result:

```text
$ objdump -T target/x86_64-unknown-linux-gnu/release/music-player | grep --only-matching 'GLIBC_[0-9][0-9.]*' | sort --unique --version-sort | tail -n 2
GLIBC_2.35
GLIBC_2.39
$ objdump -T target/x86_64-unknown-linux-gnu/release/music-player | grep -c 'GLIBC_2.4[0-9]'
0
$ env --ignore-environment target/x86_64-unknown-linux-gnu/release/music-player --version
music-player 0.1.13
```

The `run` task itself ran this way with the host build path disabled, a scratch `HOME`, and `-- --version`
(its install step copied the binary into the scratch `HOME`, and the real `~/.local/bin/music-player` kept its earlier modification time).

## Verified workarounds

- Build with `cargo zigbuild --target x86_64-unknown-linux-gnu.2.39`.
  The `.2.39` suffix makes Zig link against its bundled glibc 2.39 stubs, so the binary needs no symbol newer than 2.39.
  Tradeoffs: the image needs Zig and cargo-zigbuild; the output goes to `target/<triple>/release/` instead of `target/release/`;
  C dependencies (libopus through CMake, bindgen for PipeWire) build with `zig cc`, and the build took 8 minutes 56 seconds here
  on a loaded host.
  The shared libraries the program loads at run time (`libfontconfig`, `libpipewire-0.3`) still come from the host.
  Commit `4723d28bc9` applies this to the `run` task only; `run-quick` and `run-debug` still build with plain cargo in the container.
  The owner was reconsidering this approach when the doc was written.
- Build on an image whose glibc is not newer than the host's (a pinned older base, or the Ubuntu 24.04 release image).
  Tradeoff: it contradicts the owner's rule that base images track upstream.
- Run the binary inside the container instead of on the host.
  Tradeoff: the task was written to avoid the Wayland, PipeWire and D-Bus socket passthrough.

## What does not work

- A plain `cargo build --release` in the CachyOS image, for the reason above.
- Hiding `cargo` from `PATH` to force the container branch of the `run` task:
  mise puts the repository's tool directories on `PATH` before it runs a task, so the host branch stayed selected
  (the log showed the host path `.../truepeak-core`, not the container's `/truepeak-core`).
  Forcing the branch needs an edit of `hostOk` in a throwaway checkout.
- An empty `HOME` to keep the install step away from the real `~/.local/bin`:
  mise then created its state directory inside the package directory (`.local/state/mise`), and cargo warned about file system loops.
  A scratch `HOME` works, but podman derives its storage from `HOME`, so it started an empty store and rebuilt the image;
  the fix was to also export `XDG_DATA_HOME` and `XDG_CONFIG_HOME` of the real home for podman.
  The files that rootless podman wrote into the scratch store are owned by mapped user ids, so only `podman unshare rm --recursive --force` removes them.

## Upstream filing decision

- Is it upstream's fault? No. glibc adds symbol versions on purpose, and a rolling distribution ships the newest glibc;
  the behavior is documented in the glibc release notes.
- The remaining constraints are moot once the first fails, so nothing is drafted.
- `.out-of-scope/` was not consulted for an upstream draft because no draft exists.
