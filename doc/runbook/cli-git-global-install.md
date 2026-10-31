# Install the cli-git wrapper globally on a machine

## Purpose

Make `git` resolve to the Git policy wrapper in every repository on a machine you control,
so commits outside this checkout get the policy checks and auto-push.
The reasoning and the evidence are in `doc/planning/cli-git-global-install.md` (issue 1796).
Inside this checkout `mise run toolchain:install` installs the wrapper and links `git` into
`.cache/cli-git-bin`,
and the generated `mise.toml` `[env]` puts the tool exposure directory
(`.cache/toolchain-bin`) first on `PATH`.
The steps that follow configure the user's global mise;
the toolchain migration accepted in `doc/decision/mpm-toolchain-adoption.md`
on 2026-10-26
removes tool management from the repository's `mise.toml`
but leaves this user-global setup unchanged.

## Before you start

- The machine runs Linux, because only Linux archives of the wrapper are released.
- `mise` is installed and `~/.local/bin` comes before `/usr/bin` on your login `PATH`.
  Check with `env -i HOME="$HOME" bash -lic 'echo "$PATH"'`.
- `cargo-binstall` is available to mise (the user's global config lists it as a tool),
  otherwise mise falls back to compiling the crate, which takes far longer.
- No file named `git` exists yet in `~/.local/bin`; the install replaces it.

## Steps

1. Add the entry to the `[tools]` table of `~/.config/mise/config.toml`.
   Copy the single `"cargo:monochromatic-git-policy-cli" = { ... }` line from the fenced block in
   `doc/planning/cli-git-global-install.md`, section `Proposal`.
   Keep `version = "latest"`: the wrapper is our own package and is never pinned.
2. Install only this tool, from the home directory, so the global entry applies:

   ```shell
   # run from a terminal, outside any repository
   cd ~ && mise install cargo:monochromatic-git-policy-cli
   ```

   A bare `mise install` would install every missing global tool.
   If the install reports a timeout while checking a download, run the same command again.
3. Check the result in a clean login shell:

   ```shell
   # expected: the first line names ~/.local/bin/git, the second /usr/bin/git
   env -i HOME="$HOME" bash -lic 'cd ~; type -a git'
   ```

## Verify that auto-push works

In a throwaway repository with a bare remote, commit with an explicit pathspec
(`git commit --message test file.txt`) and compare `git rev-parse HEAD` with the remote branch.
A pathspec-less commit and `git add .` are rejected by the wrapper's policies on purpose.

## Roll back

Remove the entry from `~/.config/mise/config.toml` and delete `~/.local/bin/git`.
The repository's own per-worktree link is separate and stays in place.
