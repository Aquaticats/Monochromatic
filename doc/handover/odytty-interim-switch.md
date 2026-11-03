# Interim switch to OdyTTY

## Authorized scope

On 2026-11-02 the user requested:
"Ghostty is currently bad. For the meantime we want to use Odytty. Do all the changes needed. Ask me whenever unsure."
The user subsequently chose to retain existing OdyTTY settings and also change the development VM image.

This is implementation of the user's terminal choice,
not a new comparative technology recommendation.
The previous [terminal comparison](../planning/ghostty-interim-terminal-options.md)
and [daily-use proposal](../planning/monochromatic-terminal-daily-use.md) remain historical evidence.
No new ranking or complete technology audit is claimed.

## Work queue

- [x] Switch KDE's terminal defaults and pinned taskbar launcher to the installed OdyTTY desktop entry.
  Verify the live Plasma configuration and actual command launch.
- [ ] Switch the explicit terminal launcher preference and exercise command arguments,
  working directory,
  title,
  and application ID through `terminal-exec` in a disposable environment.
- [ ] Replace the VM image's Ghostty installation and dotfile seed with OdyTTY and explicit terminal defaults.
  Validate the release package and image integration without replacing an existing VM.
- [ ] Check current agent input compatibility,
  especially the historical Claude Code Shift+Enter limitation.
- [ ] Commit scoped changes and record verification results and remaining limits.

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
