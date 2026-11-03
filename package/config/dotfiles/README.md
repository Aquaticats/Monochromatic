# config-dotfiles

Dotfile configs baked into the dev VM image at build time.
The Containerfile COPYs these into the user's `~/.config/` directory.

## Files

- `odytty/odytty.conf`:
  OdyTTY preferences copied from the user's existing configuration on 2026-11-02.
- `odytty/themes/custom-custom.theme`:
  Custom theme selected by those preferences.
- `kdeglobals`:
  KDE terminal command and desktop-entry defaults only.
  Machine-specific display and window state is not seeded.
- `xdg-terminals.list`:
  Default terminal selection for terminal launchers.

The VM uses the upstream RPM desktop entry `io.unfinished_works.odytty.desktop`.
The host's existing AppImage integration uses `odytty.desktop`;
its settings and themes remain in the host's configuration directory.

Tool versions are not seeded here.
`mise run toolchain:install` provisions them per worktree
(`toolchain.lock.json` pins the versions),
and the image's mise serves only `mise run` task running.

## Adding a new dotfile

1. Create a subdirectory matching the `~/.config/` target (e.g. `foo/` for `~/.config/foo/`)
2. Add the config file(s) inside it
3. Add a corresponding `COPY` line in the vm-builder Containerfile
