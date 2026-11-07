# Moving Fedora build images to CachyOS: package and tool differences that broke a build or a test

Observed on 2026-10-29 while moving the music-player,
terminal and IDE images from Fedora to `docker.io/cachyos/cachyos`
(see [container-base-distro](../decision/container-base-distro.md)).
Each item names the exact failure,
the cause that was read from the image,
and the change that fixed it.
The pacman hook failure in rootless builds has its own record:
[pacman-hook-sandbox-rootless-podman-build](pacman-hook-sandbox-rootless-podman-build.md).

## `zlib` conflicts with the preinstalled `zlib-ng-compat`

`pacman --disable-sandbox --sync --refresh --sysupgrade --noconfirm --needed ... zlib` in the signing build stage stopped with:

```text
:: zlib-1:1.3.2-4 and zlib-ng-compat-2.3.3-2 are in conflict
error: unresolvable package conflicts detected
```

CachyOS ships `zlib-ng-compat` in place of `zlib`;
it provides the same library,
and `--noconfirm` answers the removal question with no.
Fix:
leave `zlib` out of the package list (commit `8ff6727144`,
`Containerfile.sign`).

## `osslsigncode` is in no repository

`pacman -Si osslsigncode` inside the image,
after `pacman -Sy`,
reports `error: package 'osslsigncode' was not found`;
the Arch package search (`https://archlinux.org/packages/search/json/?name=osslsigncode`) also returns no result.
`Containerfile.sign` therefore compiles the latest upstream release with CMake in a build stage.
Upstream publishes no checksum or signature for the source:
the release has only a macOS zip and a Windows zip,
and the tag object is unsigned (`gh api repos/mtrojnar/osslsigncode/git/tags/<sha>` returns `"verified":false,"reason":"unsigned"`).
Version 2.14 changed the PKCS#11 options:
`-pkcs11engine` is now `-engine`,
`-provider` exists,
and `-pkcs11module`,
`-pkcs11cert` and `-key` remain.

## Arch's `wine` installs `wine`, not `wine64`

The Windows GNU image set the cargo runner to `wine64`;
`nextest` failed with `failed to exec "wine64" ... No such file or directory`.
`ls /usr/bin/wine*` in the image lists `wine` and no `wine64` (wine 11.20 runs 64-bit programs through its WoW64 mode).
Fix:
`CARGO_TARGET_X86_64_PC_WINDOWS_GNU_RUNNER=wine`.

## Arch's `wine` has no Arial or Times New Roman, and Slint needs a system font

Three `ui_focus_ring_pages_tests` panicked at `i-slint-renderer-software-1.18.1/fonts/systemfonts.rs:71:70`
(`called Option::unwrap() on a None value`) because no font was found under Wine.
`/usr/share/wine/fonts` in the image holds `tahoma.ttf` and symbol fonts but no Arial or Times New Roman,
which the Fedora `wine-arial-winefonts` and `wine-times-new-roman-winefonts` packages had added.
Installing `ttf-liberation` did not help;
installing `noto-fonts` made all eight `ui_focus_ring_pages_tests` pass.
Fix:
add `noto-fonts` to the image.

## The cross-built Windows executable imports UCRT, not msvcrt

Arch's mingw-w64 packages default to the UCRT C runtime:
`x86_64-w64-mingw32-objdump -p` on the cross-built test executable lists
`api-ms-win-crt-*` DLLs,
where the Fedora image's executable imported `msvcrt.dll`.
The Wine proof now uses the same C runtime as the Windows release asset.

## Containers have no `XDG_RUNTIME_DIR`

Three `unplayable_notification` tests panicked with `XDG_RUNTIME_DIR holds the short socket directory`
after commit `438bc434e4` moved the private D-Bus socket folder into the per-user runtime directory.
A podman container sets none.
Fix:
a private tmpfs at `/run/user/0` and `--env XDG_RUNTIME_DIR=/run/user/0`
(`--tmpfs /run/user/0:mode=0700`) in the container cargo runs,
as `nested-wayland-session` does.

## A test that fails on both images

`ui_lists_render_tests::a_search_result_title_wider_than_the_page_ends_in_an_ellipsis` fails with
`the long title ends in an ellipsis` (`src/ui_lists_render_tests.rs:227`) on the CachyOS image and on a Fedora 41 image built from the former
`Containerfile`,
so the base image is not its cause.
Installing `noto-fonts` or `ttf-dejavu` in the CachyOS image did not change the result.
The cause was not investigated then; on 2026-11-07 it was measured: the test drew the Search page's fade-in against the wall
clock, so a release build drew it at partial ink and a debug build did not
(`slint-render-test-fade-in-reads-the-wall-clock.md`).

## The Git wrapper's gate and end-to-end images (observed 2026-11-04)

Moving `package/git-policy/cli` from Debian images to `docker.io/cachyos/cachyos-v4` (see
[git-policy-cli-cachyos-test-containers](../planning/git-policy-cli-cachyos-test-containers.md)) showed three differences.
The first two were fixes for building Git and `strace` from source;
the owner then moved every tool of these images to
pacman packages (2026-11-04:
"We don't need to pin git because my current system is already using git latest.",
"We
don't need to pin Node too.",
"Pacman for everything it has"),
which removed both source builds.
The fixes are kept below as history,
with what replaced each.

### The CachyOS image already carries a Git

`podman run docker.io/cachyos/cachyos-v4:latest pacman --query git` printed `git 2.56.0-1.1`,
and `/usr/bin/git` exists.
The source-built images removed it with `pacman --remove --nodeps --nodeps --noconfirm git` (`--nodeps` given twice
skips every dependency check) before building Git 2.56.0 and 2.55.0 from tarballs.
Now:
both images keep the packaged Git,
and the gate's tests name that `/usr/bin/git`.

### strace 6.13 does not build on the newest compiler and kernel headers

The static `strace` build in the gate image stopped twice:

```text
macros.h:141:9: error: static assertion failed: "Unexpected size of arg.resv (sizeof(uint64_t) * 3 expected).  --enabled-bundled=yes configure option may be used to work around that."
ioctl.c:52:13: error: assignment discards 'const' qualifier from pointer target type [-Werror=discarded-qualifiers]
```

The first comes from the newest `linux-api-headers`;
the second from the newest compiler and C library under strace's
default `-Werror`.
The source build's fix was `./configure --enable-bundled=yes --disable-gcc-Werror`.
Now:
the gate image installs pacman's `strace` (7.3-1 in `extra` on 2026-11-04),
so no build runs.
The Arch Linux Archive lists `strace-6.13-1-x86_64.pkg.tar.zst` for a pacman-native way back to 6.13.

### A test read Git's documentation from the Debian image's source tree

`invocation::hooks_off::tests::every_documented_hook_event_is_listed` panicked with
`no copy of the githooks documentation is installed (looked for githooks.adoc under /usr/share/doc/git and a rendered page
through man githooks)`.
The Debian gate image kept the Git source tree at `/work/git-source`,
which the test reads first.
Now:
the packaged `git` ships `/usr/share/man/man5/githooks.5.gz` and no `githooks.adoc`
(`pacman --query --list git | grep githooks` listed only the manual page),
and the CachyOS image has no `man`
(`which man` printed `no man in (...)`),
so the base image installs `man-db` and the test reads the rendered page.

### Package versions the images got

`mise run //package/git-policy/cli:test:git-image` on 2026-11-04 reported `git 2.56.0-1.1`,
`nodejs 26.12.1-2`,
`pnpm 12.17.0-1.1`,
`strace 7.3-1`,
`cargo-nextest 0.9.150-1.1` and `rust 1:1.99.1-1`.
CachyOS's repositories come first in the image's `/etc/pacman.conf` (`cachyos-v4`,
`cachyos-core-v4`,
`cachyos-extra-v4`,
`cachyos`,
then Arch's `core`,
`extra`,
`multilib`),
and pacman takes the first repository that has a package,
not the
newest version:
the image got `nodejs 26.12.1-2` from `cachyos-v4` while `extra` held `26.13.0-1`.
