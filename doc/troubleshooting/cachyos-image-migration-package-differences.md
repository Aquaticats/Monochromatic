# Moving Fedora build images to CachyOS: package and tool differences that broke a build or a test

Observed on 2026-10-29 while moving the music-player, terminal and IDE images from Fedora to `docker.io/cachyos/cachyos`
(see [container-base-distro](../decision/container-base-distro.md)).
Each item names the exact failure, the cause that was read from the image, and the change that fixed it.
The pacman hook failure in rootless builds has its own record:
[pacman-hook-sandbox-rootless-podman-build](pacman-hook-sandbox-rootless-podman-build.md).

## `zlib` conflicts with the preinstalled `zlib-ng-compat`

`pacman --disable-sandbox --sync --refresh --sysupgrade --noconfirm --needed ... zlib` in the signing build stage stopped with:

```text
:: zlib-1:1.3.2-4 and zlib-ng-compat-2.3.3-2 are in conflict
error: unresolvable package conflicts detected
```

CachyOS ships `zlib-ng-compat` in place of `zlib`; it provides the same library, and `--noconfirm` answers the removal question with no.
Fix: leave `zlib` out of the package list (commit `8ff6727144`, `Containerfile.sign`).

## `osslsigncode` is in no repository

`pacman -Si osslsigncode` inside the image, after `pacman -Sy`, reports `error: package 'osslsigncode' was not found`;
the Arch package search (`https://archlinux.org/packages/search/json/?name=osslsigncode`) also returns no result.
`Containerfile.sign` therefore compiles the latest upstream release with CMake in a build stage.
Upstream publishes no checksum or signature for the source: the release has only a macOS zip and a Windows zip,
and the tag object is unsigned (`gh api repos/mtrojnar/osslsigncode/git/tags/<sha>` returns `"verified":false,"reason":"unsigned"`).
Version 2.14 changed the PKCS#11 options: `-pkcs11engine` is now `-engine`, `-provider` exists, and `-pkcs11module`,
`-pkcs11cert` and `-key` remain.

## Arch's `wine` installs `wine`, not `wine64`

The Windows GNU image set the cargo runner to `wine64`; `nextest` failed with `failed to exec "wine64" ... No such file or directory`.
`ls /usr/bin/wine*` in the image lists `wine` and no `wine64` (wine 11.20 runs 64-bit programs through its WoW64 mode).
Fix: `CARGO_TARGET_X86_64_PC_WINDOWS_GNU_RUNNER=wine`.

## Arch's `wine` has no Arial or Times New Roman, and Slint needs a system font

Three `ui_focus_ring_pages_tests` panicked at `i-slint-renderer-software-1.18.1/fonts/systemfonts.rs:71:70`
(`called Option::unwrap() on a None value`) because no font was found under Wine.
`/usr/share/wine/fonts` in the image holds `tahoma.ttf` and symbol fonts but no Arial or Times New Roman,
which the Fedora `wine-arial-winefonts` and `wine-times-new-roman-winefonts` packages had added.
Installing `ttf-liberation` did not help; installing `noto-fonts` made all eight `ui_focus_ring_pages_tests` pass.
Fix: add `noto-fonts` to the image.

## The cross-built Windows executable imports UCRT, not msvcrt

Arch's mingw-w64 packages default to the UCRT C runtime: `x86_64-w64-mingw32-objdump -p` on the cross-built test executable lists
`api-ms-win-crt-*` DLLs, where the Fedora image's executable imported `msvcrt.dll`.
The Wine proof now uses the same C runtime as the Windows release asset.

## Containers have no `XDG_RUNTIME_DIR`

Three `unplayable_notification` tests panicked with `XDG_RUNTIME_DIR holds the short socket directory`
after commit `438bc434e4` moved the private D-Bus socket folder into the per-user runtime directory.
A podman container sets none. Fix: a private tmpfs at `/run/user/0` and `--env XDG_RUNTIME_DIR=/run/user/0`
(`--tmpfs /run/user/0:mode=0700`) in the container cargo runs, as `nested-wayland-session` does.

## A test that fails on both images

`ui_lists_render_tests::a_search_result_title_wider_than_the_page_ends_in_an_ellipsis` fails with
`the long title ends in an ellipsis` (`src/ui_lists_render_tests.rs:227`) on the CachyOS image and on a Fedora 41 image built from the former
`Containerfile`, so the base image is not its cause.
Installing `noto-fonts` or `ttf-dejavu` in the CachyOS image did not change the result.
The cause was not investigated.
