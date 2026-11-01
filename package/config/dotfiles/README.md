# config-dotfiles

Dotfile configs baked into the dev VM image at build time.
The Containerfile COPYs these into the user's `~/.config/` directory.

## Files

- `ghostty/config`:
   Ghostty terminal configuration

Tool versions are not seeded here.
`mise run toolchain:install` provisions them per worktree
(`toolchain.lock.json` pins the versions),
and the image's mise serves only `mise run` task running.

## Adding a new dotfile

1. Create a subdirectory matching the `~/.config/` target (e.g. `foo/` for `~/.config/foo/`)
2. Add the config file(s) inside it
3. Add a corresponding `COPY` line in the vm-builder Containerfile
