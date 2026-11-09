# Interim switch to OdyTTY

> References below this note that no longer resolve in the tree.
> Each entry gives the value that identifier has now:
> `package/config/dotfiles/ghostty/config` is deleted (2026-11-02 `39113fdeba`) and no path
> of that name exists today.

## Authorized scope

On 2026-11-02 the user requested:
"Ghostty is currently bad.
For the meantime we want to use Odytty.
Do all the changes needed.
Ask me whenever unsure."
The user subsequently chose to retain existing OdyTTY settings and also change the development VM image.

This is implementation of the user's terminal choice,
not a new comparative technology recommendation.
The previous [terminal comparison](../planning/ghostty-interim-terminal-options.md)
and [daily-use proposal](../planning/monochromatic-terminal-daily-use.md) remain historical evidence.
No new ranking or complete technology audit is claimed.

## Work queue

- [x] Switch KDE's terminal defaults and pinned taskbar launcher to the installed OdyTTY desktop entry.
  Verify the live Plasma configuration and actual command launch.
- [x] Switch the explicit terminal launcher preference and exercise command arguments,
  working directory,
  title,
  and application ID through `terminal-exec` in a disposable environment.
- [x] Replace the VM image's Ghostty installation and dotfile seed with OdyTTY and explicit terminal defaults.
  Validate the release package and image integration without replacing an existing VM.
- [x] Check current agent input compatibility,
  especially the historical Claude Code Shift+Enter limitation.
- [x] Carry the existing terminal I/O priority to OdyTTY desktop services and AppImage scopes.
- [x] Commit scoped changes and record verification results and remaining limits.

## Completed verification

The VM image build completed with exit status `0`:
`localhost/monochromatic-dev:latest`,
image `545a8c81c868ed6d5495357552bb618b3d0d21dd53004af288d292016ac3b326`.
`mise run //package/vm-builder/cli:image:check` passed in disposable containers with networking disabled.
As the image's regular `user`,
OdyTTY reports `0.17.5`,
loads the seeded JetBrains Mono font preference at size `20`,
and resolves the custom theme.
The KDE and XDG defaults both point to `io.unfinished_works.odytty.desktop`.
The installed RPM also passed a graphical launch from that exact image in a disposable container:
its window rendered the test command output,
showed exit status `0`,
and closed successfully after Enter.
This passed both with host render devices exposed and with no GPU devices passed to the container.
The screenshots were inspected.
The fixture mounted only a private runtime directory and the nested compositor's Wayland socket,
used no network,
and ran as the image's regular `user`.
The final graphical fixtures were `image-gui-fno70w` and `image-gui-iXs75q` under the migration scratch directory.
The first harness attempt used an unsupported Podman tmpfs ownership option and exited `125` before OdyTTY started;
the successful fixtures used a private directory owned by the test user instead.
The same rejected option is documented in
[the Podman ownership notes](../troubleshooting/depot-rootless-podman-group-identity.md).
No full VM boot,
disk conversion,
or replacement of an existing libvirt VM was performed.

The active host OdyTTY service initially had no `IOWeight` setting;
its kernel `io.weight` and `io.bfq.weight` were both `default 100`.
The existing Ghostty policy used `IOWeight=10`.
The new host drop-ins are
`~/.config/systemd/user/app-odytty@.service.d/10-io-weight.conf`
and `~/.config/systemd/user/app-odytty-.scope.d/10-io-weight.conf`.
Disposable service and scope fixtures inherited `IOWeight=10` and were stopped.
The running OdyTTY service was updated without restarting it;
both kernel weight files now read `default 10`.
This is a configuration check,
not a new performance measurement.

Claude Code `2.1.317` passed the live OdyTTY input check without patching:
Shift+Enter inserted a newline,
Ctrl+C cleared input,
and Enter submitted.
The [keyboard verification](../troubleshooting/claude-code-odytty-keyboard.md) records isolation and limits.

`probe-launch.ts launcher-verified` in the private migration scratch directory passed through the built `terminal-exec`.
The isolated OdyTTY window received the requested title,
application ID,
working directory,
and a literal argument containing spaces,
a semicolon,
and command-substitution syntax.
Its child reported `TERM_PROGRAM=odytty` and `TERM=xterm-256color`.
Legacy input and Kitty disambiguation mode were exercised with Enter,
Shift+Enter,
Ctrl+C,
and Escape.
The hold option retained the window after child exit;
the next key closed it with exit status `0`.
The same checks passed through the installed system `xdg-terminal-exec` in `xdg-verified`.
Its read-only `--print-id --print-cmd` probe against the real host configuration selects `odytty.desktop`.

The scoped Markdown lint task passed for every changed Markdown file.
The installed `marked` renderer produced the expected headings,
code spans,
links,
and lists for those files;
rendered output was inspected.

The VM image definition and dotfiles were committed in `39113fdeba`.
The build-context allowlist referenced obsolete `packages/config/dotfiles/`;
it now names `package/config/dotfiles/` so the new seeds reach the image.
An `image:build` task builds only the OS image,
without converting disks or replacing a libvirt VM.
The first build verified the RPM checksum,
then was stopped during Fedora metadata retrieval.
Its selected mirror connection received `36,790,730` bytes after `550.930` seconds since its request.
A separate fetch of Fedora's `15,678,154`-byte primary metadata file from the direct Fedora endpoint
completed in `3.460687` seconds.
These are different requests,
not a controlled bandwidth benchmark or a build completion forecast.
The successful build used the optional `FEDORA_DIRECT_MIRROR=true` argument,
preserving package signature verification.
The stopped build log is `${HOME}/temp/agent/odytty-switch-20261102/image-build.log`.
The successful build and inspection logs are `image-build-direct.log` and `image-check.log` in the same directory.

## Initial inventory

The installed AppImage reports OdyTTY `0.17.5`.
Its desktop entry is `odytty.desktop`,
with command,
directory,
title,
application-ID,
and hold arguments already declared.
The host's `kdeglobals`,
`xdg-terminals.list`,
and taskbar launcher still select Ghostty.
The Ghostty global launch shortcut is `none`;
there is no active shortcut to transfer.

The user's existing `odytty.conf` is retained unchanged.
Host backups and disposable probes belong in `${HOME}/temp/agent/odytty-switch-20261102`.

`package/vm-builder/cli/Containerfile` installs Ghostty through a COPR repository.
`package/config/dotfiles/ghostty/config` contains comments only.
OdyTTY's upstream RPM installs desktop entry `io.unfinished_works.odytty.desktop`,
which differs from the AppImage integration's host entry.
The VM must use the RPM entry's identifier.

The repository's `libghostty-vt` terminal engine,
compatibility warnings,
title limits,
and support for launching explicitly selected Ghostty windows are separate responsibilities.
They do not choose the user's default desktop terminal.

## Release evidence

Upstream source checkout:
`${HOME}/temp/agent/odytty-0175-20261101`,
revision `a12b66dfdc97e2789677c973ce19e1289af6c696`.
Read `PACKAGING.md`,
the launch parser in `src/cli.rs`,
and `.github/workflows/release.yml`.
The GitHub release API for `v0.17.5` reports the x86-64 RPM SHA-256:
`66694895f83eee90b2b75250c4f8bc0acce84052b200344295b6df2c9472ad55`.
The upstream RPM is cross-built and described as best-effort;
installation and runtime checks remain necessary.

## Recovery

Host configuration was switched with `kwriteconfig6 --notify` and Plasma's live scripting API.
Taskmanager widget `30` now pins `applications:odytty.desktop` in Ghostty's former position.
`kreadconfig6` passed a disposable fixture before the real write.
The live API returned the changed launcher list,
and the persisted files match.
The before/after `odytty.conf` comparison was byte-identical.
Backups are under `${HOME}/temp/agent/odytty-switch-20261102/host-before`.

Retain the host Ghostty installation and configuration during this interim switch.
Restoring the saved KDE terminal keys,
explicit terminal list,
and taskbar launcher reverses the desktop switch.
Do not restore whole shared KDE files over concurrent desktop changes.
The VM change is reversible through a scoped follow-up commit.

The OdyTTY I/O priority addition is confined to the host's named service and scope drop-ins.
The Ghostty policy remains present.
All disposable terminal windows,
containers,
and I/O-priority fixture units were closed or removed after their checks.
The temporary read-only KWin identity script was unloaded;
its log output was unavailable and is not used as verification evidence.
