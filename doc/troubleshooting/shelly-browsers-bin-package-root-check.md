# Shelly 3.2.1 and 3.3.0 fail the `browsers-bin` package root check

Diagnosis recorded on `2026-11-17T15:30:08-05:00`. The build and installation checks ran in disposable roots.
The desktop session and its default browser were outside those checks. This record is unapproved.

## Symptom

On CachyOS, `shelly install aur browsers-bin` reached `package()` for `browsers-bin` `0.8.6-1` and failed.
The per-build log and CLI report the same phase and diagnostic.

```text
# Shelly per-build log
[phase] package
[stdout] You must run this installer with sudo when using --system flag
[stdout] Please run again as:
[stdout] sudo ./install.sh --system
[error] browsers-bin: package: StepFailed
```

The native build backend reports exit code `1`. The outer AUR operation reports `BuildFailed`.

## Root cause

The AUR recipe at `663d3039b29958918ff2fe2718dec7237dded4e5` first redirects the installer's `/usr` paths
into `$pkgdir`. `PKGBUILD:26` then invokes the external installer in system mode. [AUR recipe][aur-recipe]

```bash
# PKGBUILD:26
XDG_DATA_DIRS="$pkgdir/usr/share" "$srcdir/install.sh" --system --skip-desktop-database
```

Browsers `0.8.6` checks `id -u` before it uses any of those paths.
The installer emits the quoted diagnostic and exits `1` if the check fails.
[Browsers installer][browsers-installer]

```bash
# extra/linux/dist/install.sh:20
is_user_root_or_sudoer() {
  [ "$(id -u)" -eq 0 ]
}
```

Shelly's native builder starts the package phase through `/bin/bash` and inserts wrappers into `PATH`
for selected package metadata operations. `createPackageWrappers` enumerates the wrapped commands.
The installer calls `id` outside that set and sees the unprivileged UID. [Shelly launch][shelly-launch]
[Shelly wrappers][shelly-wrappers]

```zig
// Shelly.PackageManager/src/aur/builder/steps.zig:343
for ([_][]const u8{ "chown", "chgrp", "install", "mknod" }) |name| {
    try directory.writeFile(self.io, .{ .sub_path = name, .data = launcher });
    try directory.setFilePermissions(self.io, name, .fromMode(0o700));
}
```

The installed `makepkg` enters `fakeroot` before packaging. The child `id` reports UID `0` under
`fakeroot`, so the same installer proceeds with its redirected staging paths.

```bash
# /usr/bin/makepkg:188 (CachyOS pacman 7.1.0.r9.g54d9411-6)
enter_fakeroot() {
	msg "$(gettext "Entering %s environment...")" "fakeroot"
	fakeroot -- bash -$- "${BASH_SOURCE[0]}" -F "${ARGLIST[@]}" || exit $?
}
```

## Verification

The cloned recipe matched the cached `PKGBUILD` byte for byte. The release archive
`browsers_linux-0.8.6.tar.xz` matched the recipe and GitHub release SHA-256,
`197fe816f24644b03e2871931db5e39c4e33f77dbf1e88620f33cbca9b5e5df0`.
The Shelly builder implementation in `steps.zig` was byte-identical across the tested releases.
[Browsers release][browsers-release]

In disposable copies of the pinned AUR recipe, with the archive beside `PKGBUILD`, the following
commands exercise the failing and working builders respectively. Run both as a regular user.

```bash
# In a disposable browsers-bin AUR checkout with the recipe and release archive inspected
shelly build --reviewed --no-confirm PKGBUILD
makepkg
```

- **Fails.** CachyOS Shelly `3.2.1-1` and the official Shelly `v3.3.0` CLI both returned the same
  `StepFailed` from `package`, with native exit code `1` and the same installer message.
- **Works.** `makepkg` produced `browsers-bin-0.8.6-1-x86_64.pkg.tar.zst`. `pacman --upgrade`
  installed it into a disposable root with `LocalFileSigLevel = Optional`, matching the host's policy
  for local archives. The package query returned `browsers-bin 0.8.6-1`. The installed desktop entry
  passed `desktop-file-validate`, and `browsers --no-gui https://example.org/` listed Firefox Developer Edition.

## Verified workaround

Run `makepkg` as the regular user from a reviewed checkout of the AUR recipe. Then install the archive
with pacman. Elevation is needed for the package transaction.

```bash
# In the directory containing the reviewed PKGBUILD
makepkg
sudo pacman --upgrade ./browsers-bin-0.8.6-1-x86_64.pkg.tar.zst
```

The commands assume the recipe's `desktop-file-utils` and `xdg-utils` build dependencies are present.
They were installed on the tested host. Future upgrades through this workaround require a fresh
build with `makepkg`. Pacman tracks the resulting files and owns package removal.

## What does not work

- Retrying with the official Shelly `v3.3.0` CLI reproduced the same failure.
- `sudo ./install.sh --system`, as suggested by the diagnostic, belongs to the upstream system
  installer. Running it directly would leave pacman's packaging path. The AUR recipe runs it with
  rewritten paths and packaging-specific arguments.

## Upstream filing decision

- **Attribution.** The failure requires the recipe's external UID gate and Shelly's selected metadata
  wrappers. The existing Shelly issue about `chown` and related ownership operations reaches a different
  error path, `PrivilegedPackageOperationUnsupported`. [Shelly issue 2025][ownership-issue]
- **Fixability.** An explicit staging interface in the Browsers installer could make the AUR recipe
  independent of simulated identity. Full process-level fakeroot compatibility is a separate Shelly design choice.
- **Supported use.** The builder documents simulation of common ownership operations. The inspected code
  supplies wrappers for those operations and leaves identity checks to the underlying process.
  [Builder design notes][builder-design]
- **Intake.** Shelly's contribution guide and issue forms were inspected. No AI-specific intake policy
  was found there on `2026-11-17`.
- **Maintenance.** Tracker searches for `fakeroot`, `browsers-bin`, `fakechroot`, and `EUID` found no exact
  duplicate. The latest released CLI still reproduces the failure. A maintainer position on this case is unknown.
- **Prototype.** The `makepkg` workaround was exercised end to end in a disposable root. An upstream
  compatibility patch was outside this installation diagnosis and was not built or tested.

The filing decision is to keep the diagnosis and verified workaround in this record.

[aur-recipe]: https://aur.archlinux.org/cgit/aur.git/tree/PKGBUILD?h=browsers-bin&id=663d3039b29958918ff2fe2718dec7237dded4e5#n26
[browsers-installer]: https://github.com/Browsers-software/browsers/blob/be2ad4ca938efd1e59739bce2dd644669589b8f8/extra/linux/dist/install.sh#L20-L34
[browsers-release]: https://github.com/Browsers-software/browsers/releases/tag/0.8.6
[shelly-launch]: https://github.com/Seafoam-Labs/Shelly-ALPM/blob/a1648fd10b37e3f051aa08dbbc410680ce772abe/Shelly.PackageManager/src/aur/builder/steps.zig#L240-L254
[shelly-wrappers]: https://github.com/Seafoam-Labs/Shelly-ALPM/blob/a1648fd10b37e3f051aa08dbbc410680ce772abe/Shelly.PackageManager/src/aur/builder/steps.zig#L335-L347
[ownership-issue]: https://github.com/Seafoam-Labs/Shelly-ALPM/issues/2025
[builder-design]: https://github.com/Seafoam-Labs/Shelly-ALPM/blob/a1648fd10b37e3f051aa08dbbc410680ce772abe/Shelly.PackageManager/src/aur/builder/README.md#L185-L190
