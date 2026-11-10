# niri 26.04 nested testing: mise cargo installs a stub or source build, rpm-ostree installs runnable niri

Tool under test:
 niri 26.04,
 mise 2026.6.13,
 and Bazzite 44's rpm-ostree host package layer.
Surface trigger:
 installing niri for nested Slint GPU testing from a repo task session.
Failure mode:
 `mise install cargo:niri@latest` installs a binary that only prints an error,
while a mise cargo git install builds source and fails on missing native development packages.

## Symptom

The crates.
io path succeeds as an install but not as a runnable compositor:

```sh
mise install cargo:niri@latest
mise exec cargo:niri@latest -- niri --version
```

Observed output:

```text
niri cannot be properly installed with `cargo install`.
Please install niri from your distribution packages:
https://niri-wm.github.io/niri/Getting-Started.html
```

The git source path without a crate selector fails before build planning finishes:

```sh
mise install 'cargo:https://github.com/niri-wm/niri@tag:v26.04'
```

Observed output:

```text
error: multiple packages with binaries found: niri, niri-visual-tests. When installing a git repository, cargo will always search the entire repo for any Cargo.toml.
Please specify a package, e.g. `cargo install --git https://github.com/niri-wm/niri niri`.
```

The git source path with `crate=niri` gets into the real build,
 then fails when the host lacks development packages:

```sh
mise install 'cargo:https://github.com/niri-wm/niri[crate=niri]@tag:v26.04'
```

Observed output:

```text
error: failed to run custom build command for `libudev-sys v0.1.4`
Package 'libudev' not found
The system library `libudev` required by crate `libudev-sys` was not found.
The file `libudev.pc` needs to be installed and the PKG_CONFIG_PATH environment variable must contain its parent directory.
```

On this Bazzite host,
 direct `dnf install niri` is also the wrong installer:

```text
ERROR: Fedora Atomic images utilize rpm-ostree instead (and is discouraged to use).
```

## Root cause

There are several different install surfaces that look like they should all produce `niri`,
but only the system-package surface matches upstream's packaging model.

The crates.
io `niri` package is intentionally a sentinel.
 Its manifest publishes version `0.0.0`,
and its binary only prints the distribution-package instruction before exiting nonzero.

```toml
# niri-0.0.0/Cargo.toml.orig:1
[package]
name = "niri"
version = "0.0.0"
edition = "2021"
description = "A scrollable-tiling Wayland compositor"
authors = ["Ivan Molodetskikh <yalterz@gmail.com>"]
license = "GPL-3.0-or-later"
repository = "https://github.com/niri-wm/niri"
```

```rust
// niri-0.0.0/src/main.rs:1
fn main() {
    eprintln!(
        "niri cannot be properly installed with `cargo install`.\n\
         Please install niri from your distribution packages:\n\
         https://niri-wm.github.io/niri/Getting-Started.html"
    );
    std::process::exit(1);
}
```

The real source repository has more than one workspace binary package.
 Cargo refuses an unqualified git install,
because the workspace contains both `niri` and `niri-visual-tests`.

```toml
# Cargo.toml:1
[workspace]
members = [
    "niri-config",
    "niri-ipc",
    "niri-visual-tests",
]
```

```toml
# niri-visual-tests/Cargo.toml:1
[package]
name = "niri-visual-tests"
version.workspace = true
```

After selecting the `niri` crate,
 the build still needs the same native packages upstream expects distro packagers
to provide.
 Fedora packaging records these as RPM build requirements,
 including `pkgconfig(udev)`,
`pkgconfig(libinput)`,
 `pkgconfig(libseat)`,
 `pkgconfig(libdisplay-info)`,
 PipeWire,
 Pango,
 Cairo,
 and Clang.

```spec
# niri.spec.rpkg:65
BuildRequires:  cargo-rpm-macros >= 26
BuildRequires:  pkgconfig(udev)
BuildRequires:  pkgconfig(gbm)
BuildRequires:  pkgconfig(xkbcommon)
BuildRequires:  wayland-devel
BuildRequires:  pkgconfig(libinput)
BuildRequires:  pkgconfig(dbus-1)
BuildRequires:  pkgconfig(systemd)
BuildRequires:  pkgconfig(libseat)
BuildRequires:  pkgconfig(libdisplay-info)
BuildRequires:  pipewire-devel
BuildRequires:  pango-devel
BuildRequires:  cairo-gobject-devel
BuildRequires:  clang
BuildRequires:  mesa-libEGL
```

The upstream build documentation names the same Fedora development packages for manual source builds.

```markdown
<!-- doc/wiki/Getting-Started.md:177 -->
- Fedora:

    ```sh
    sudo dnf install gcc libudev-devel libgbm-devel libxkbcommon-devel wayland-devel libinput-devel dbus-devel systemd-devel libseat-devel pipewire-devel pango-devel cairo-gobject-devel clang libdisplay-info-devel
    ```
```

Upstream's packaging page also says the recommended packaging shape is a standalone desktop session,
and specifically excludes `niri-visual-tests` from packaged outputs.

```markdown
<!-- doc/wiki/Packaging-niri.md:13 -->
The `niri-visual-tests` sub-crate/binary is development-only and should not be packaged.

The recommended way to package niri is so that it runs as a standalone desktop session.
```

The niri 26.04 source carries RPM packaging metadata,
 another signal that the distro package path is the intended
runtime installation path.

```toml
# Cargo.toml:167
[package.metadata.generate-rpm]
version = "26.04"
assets = [
    { source = "target/release/niri", dest = "/usr/bin/", mode = "755" },
    { source = "resources/niri-session", dest = "/usr/bin/", mode = "755" },
    { source = "resources/niri.desktop", dest = "/usr/share/wayland-sessions/", mode = "644" },
    { source = "resources/niri-portals.conf", dest = "/usr/share/xdg-desktop-portal/", mode = "644" },
    { source = "resources/niri.service", dest = "/usr/lib/systemd/user/", mode = "644" },
    { source = "resources/niri-shutdown.target", dest = "/usr/lib/systemd/user/", mode = "644" },
]
```

On Bazzite and other Fedora Atomic variants,
 the package manager at that boundary is rpm-ostree,
not direct `dnf`.
 Therefore the correct local workaround is a live rpm-ostree overlay of the Fedora `niri` package,
not a cargo install and not a source build inside the repo session.

## Verification

Version and source trace:

- `niri-wm/niri` tag `v26.04` checked out at commit `8ed0da44d974c32c6877d2f4630c314da0717ecb`.
- crates.
  io `niri` version `0.0.0` from the local cargo registry source.
- mise version `2026.6.13 linux-x64`,
   recorded in mise install error output.
- Host distro:
   Bazzite `44.20260629.0 (Kinoite)` from `/etc/os-release`.

Failure catalog:

- `mise install cargo:niri@latest` installs `cargo:niri@0.0.0`,
   but `niri --version` exits with the sentinel message.
- `mise install 'cargo:https://github.com/niri-wm/niri@tag:v26.04'` fails with Cargo's multiple-binary-package error.
- `mise install 'cargo:https://github.com/niri-wm/niri[crate=niri]@tag:v26.04'` reaches compilation and fails at
  `libudev-sys` because `libudev.pc` is absent.
- `dnf --assumeno install niri` aborts on Bazzite because Fedora Atomic images use rpm-ostree.
- `mise bootstrap packages use --dry-run brew:niri` fails because Homebrew has no `niri` formula.

Working catalog:

- `mise bootstrap packages use --dry-run dnf:niri` shows the system-package intent by printing
  `sudo dnf install -y niri`,
   but that exact command is not suitable on Bazzite.
- `rpm-ostree install --apply-live --assumeyes --idempotent niri` is the Bazzite-compatible system-package path.

## Verified workarounds

### Install the Fedora package through rpm-ostree on Bazzite

Use this when a session is allowed to mutate the host package layer:

```sh
rpm-ostree install --apply-live --assumeyes --idempotent niri
niri --version
```

Tradeoff:
 this is a system-wide overlay,
 not a project-local tool.
 It changes the current live deployment and the next
booted deployment.
 That is appropriate when the goal is to exercise a real Wayland compositor path for GUI testing,
but it is not an isolated per-repo tool install.

### Build from source only after installing development packages

Use this only when the goal is to hack on niri itself or test an unreleased niri revision:

```sh
rpm-ostree install --apply-live --assumeyes --idempotent \
  gcc clang systemd-devel mesa-libgbm-devel libxkbcommon-devel wayland-devel libinput-devel \
  dbus-devel libseat-devel pipewire-devel pango-devel cairo-gobject-devel libdisplay-info-devel
mise install 'cargo:https://github.com/niri-wm/niri[crate=niri]@tag:v26.04'
```

Tradeoff:
 this overlays many development packages onto the host and compiles niri.
 It is slower and broader than
installing the distro `niri` package,
 so it is the wrong path for merely running nested niri during Slint app tests.

### Use mise only as the system-package declarator on non-Atomic Fedora

On a mutable Fedora host,
 this is the mise-shaped path:

```sh
mise bootstrap packages use dnf:niri
```

Tradeoff:
 mise's dnf bootstrap package manager invokes `dnf`,
 so it is not the right executor on Bazzite.
On Fedora Atomic,
 use rpm-ostree directly for this package.

## What does not work

- `cargo:niri@latest` through mise does not install runnable niri.
   It installs the intentional crates.
  io sentinel.
- A git cargo install without `[crate=niri]` does not work because the source workspace has multiple binary packages.
- A git cargo install with `[crate=niri]` is a source build,
   so it still needs system development packages.
- Direct `dnf install niri` is rejected on Bazzite by the host's package-management guard.
- Homebrew cannot bridge this on Linux because `brew:niri` is not a published formula.

## Upstream filing decision

`.out-of-scope/` was checked for `niri`,
 `mise`,
 `rpm-ostree`,
 `cargo install`,
 and Slint.
 No exemption matched.
Duplicate search was run with:

```sh
gh search issues --repo niri-wm/niri 'crates.io niri 0.0.0 cargo install' --state open
gh search issues --repo niri-wm/niri 'crates.io niri 0.0.0 cargo install' --state closed
gh search issues --repo niri-wm/niri 'install niri from cargo' --state open
gh search issues --repo niri-wm/niri 'install niri from cargo' --state closed
gh search prs --repo niri-wm/niri 'cargo install niri' --state open
gh search prs --repo niri-wm/niri 'cargo install niri' --state closed
```

No matching issues or pull requests were returned.

1.  Is it really upstream's fault?
     No. Upstream intentionally publishes the crates.
    io sentinel and explicitly directs
    users to distro packages.
2.  Can upstream fix it?
     Not applicable after constraint 1 fails.
     They could publish a real cargo-installable binary,
    but that would contradict the documented packaging model and native dependency set.
3.  Are they supporting this use case?
     No. The supported user-facing install path is distribution packaging;
    source builds are documented for builders with development packages.
4.  Would the repo welcome our contribution?
     The repo accepts issues and contributions,
     and `CONTRIBUTING.md` says LLM
    assistance must be checked and cleaned up.
     This does not overcome constraint 1.
5.  Will they likely fix it?
     No signal points that way.
     The docs and sentinel both point away from cargo install.
6.  Have we prototyped a minimal fix compatible with their architecture?
     No,
     because constraints 1 to 3 fail.
    The auto-prototype trigger does not fire.

Do not file as-is:

~~~md
Title: Make niri installable through cargo install

Do not file. This request contradicts the upstream sentinel crate and packaging documentation.
The correct fix for Bazzite nested-compositor testing is to install the Fedora niri package through rpm-ostree,
not to ask upstream to make cargo install a supported runtime installation path.
~~~
