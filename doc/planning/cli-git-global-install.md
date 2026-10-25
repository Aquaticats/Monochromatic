# Global install of the cli-git wrapper (issue 1796)

## Status

Accepted by the user on 2026-10-25, then amended the same day: review found that a link to one version directory goes
stale when an upgrade runs inside the repository, and the user approved linking through the `latest` symlink instead.
Every experiment ran either read-only on the host or inside podman with a throwaway `HOME`.
The amended block passed the podman check, and it was applied to the user's global mise config on 2026-10-25.
The user-boundary check passed: a clean login shell resolves `git` to `~/.local/bin/git`, and a `claude -p` commit in
a clone outside this repository reached its bare remote with the user's real settings and no hook.
The runbook for another machine is `doc/runbook/cli-git-global-install.md`.

## Problem

The native Git policy wrapper (`cli-git-native`, installed as `git`) reaches `PATH` only through this repository's
`mise.toml` `[env] _.path`, which names the gitignored per-worktree directory `.cache/cli-git-bin`.
A command that works in another repository leaves that scope and resolves `git` to `/usr/bin/git`.
Commits made there carry no policy checks, no auto-push and no error.
Issue 1796 recorded a static-site clone whose `main` stopped being pushed after a Claude Code restart,
and the gap went unnoticed until GitHub Pages was checked.

## User requirements

- Auto-push should enforce everything on machines the user controls.
- At least this repository should be enforced on machines the user does not control.
- A warning when `git` is not the wrapper is rejected, because it would not work for every consumer.
- The wrapper is installed at two scopes: locally per worktree (exists) and globally per user (this proposal).

## Evidence

### The login PATH of the user

From a clean login shell started in the home directory (`env -i HOME=... bash -lic`), the `PATH` order is:
the mise tool directories, then `~/.local/share/mise/shims`, then `~/.local/bin`, `~/bin`, `/usr/local/bin`,
`/usr/bin`.
No `git` exists in any directory before `/usr/bin`; `type -a git` prints only `/usr/bin/git`.
`claude` itself lives in `~/.local/bin`.
A `git` link in `~/.local/bin` therefore wins in terminals and in any Claude Code session that inherits the shell
`PATH`.

### Routes that reach Claude Code's Bash tool (host, `claude -p`, base `PATH` without the checkout)

- Baseline: `command -v git` prints `/usr/bin/git`, so the issue reproduces.
- `--settings` with `env.PATH`: works, but `env` values are not shell-expanded, so it needs a literal full `PATH`.
- Project `.claude/settings.json` with `env.PATH`: works under `-p`, which does not exercise the folder-trust step.
- A `SessionStart` hook appending an `export PATH=...` line to `$CLAUDE_ENV_FILE`: works.
- mise global `_.path`: not tested validly, because the probe stripped the mise activation hook and the shims directory.

### The proposed route (podman, Debian 13 image, throwaway `HOME`, host binaries and credentials mounted read-only)

- A user-level link `~/.local/bin/git` to the wrapper, with no hook: `claude -p` resolved to the link, and a commit with
  a pathspec reached a bare remote (local and remote `247ab88`).
- An entry for the crate in a global mise config, with a `postinstall` that creates the link: `mise install` ran the
  `postinstall` from outside any project, created `~/.local/bin/git`, and `git --version` worked through it.
  mise created a shim only for `cli-git-native`, none for `git`.
- In a repository with no `cli-git.config.jsonc`, the wrapper still applies its built-in policies
  (`add .` is rejected with a hint; a commit with a pathspec passes).

### The mise shim alternative (podman)

- With a `git` link inside the tool's own `bin/`, `mise reshim` creates `shims/git`.
  The wrapper is engaged through it (`add .` exits 1 with `add-explicit/bulk-add-rejected`), and there is no loop.
  An earlier claim in this work that the shim would loop was wrong and was retracted after this run.
- Cost per 50 sequential `git --version` calls, three interleaved rounds: plain 32, 29 and 30 ms; a second plain arm as
  the same-binary control 29, 29 and 31 ms; direct link 64, 64 and 63 ms; shim 295, 272 and 282 ms.
  That is about 0.6 ms per call plain, 1.3 ms through the link and 5.6 ms through the shim.
- Limits: `git --version` is the lightest path, three rounds, container rather than host.

### Upgrades from a project context (podman)

- With the global entry and a repository-style entry for the same tool key, installing the newest version from the
  project directory ran only the project entry's `postinstall`.
- A link to one version directory (the first form of this proposal) still resolved to 0.2.7 afterwards.
- A link through the `latest` symlink resolved to 0.2.8 afterwards, because mise moves `latest` on every install.
- The atomic form from a fresh home: installing 0.2.7 pointed the link at 0.2.7, a plain `mise install` installed 0.2.8
  and repointed it, and with the link deleted `mise install` reported "already installed" and still recreated it.
  The directory listing showed no leftover temporary file.
- A first run of the same check failed with `cargo-binstall` finding no prebuilt binary and `cargo install` not finding
  the crate; a standalone binstall dry run and a verbose `mise install` succeeded right after, so the cause is unproven
  and looks like a transient network failure.

### How the wrapper finds real Git

`resolve_real_git` (`package/git-policy/cli/src/native/real_git.rs`) walks `PATH` and skips a candidate that is the
same file as the running wrapper, an identical copy, or a TypeScript-wrapper launcher.
A link to the wrapper is skipped by file identity, so the wrapper never selects itself.

## Proposal

Keep the repository's per-worktree link unchanged.
Add the wrapper to the user's own global mise config, with a `postinstall` that links `~/.local/bin/git`:

```toml
# ~/.config/mise/config.toml (user-level, outside the repository)
[tools]
"cargo:monochromatic-git-policy-cli" = { version = "latest", os = ["linux"], postinstall = { run = "mkdir --parents \"$HOME/.local/bin\" && ln --symbolic \"$(dirname \"$MISE_TOOL_INSTALL_PATH\")/latest/bin/cli-git-native\" \"$HOME/.local/bin/.git.$$\" && mv --no-target-directory \"$HOME/.local/bin/.git.$$\" \"$HOME/.local/bin/git\"", when = "always" } }
```

The temporary-name-then-move form copies the atomic relink used by the repository's own entry.
The link targets mise's `latest` symlink in the tool's install directory, not one version directory.
Inside this repository the repository's own entry for the same tool wins over the global entry,
so an upgrade run there runs only the repository's `postinstall`;
a link to one version directory would then keep pointing at the old version, while the `latest` symlink moves with
every install (see `Upgrades from a project context`).

No Claude Code hook is needed, because `~/.local/bin` is already on the `PATH` that Claude Code sessions inherit.

## Alternatives rejected

- A `SessionStart` hook alone: reaches only Claude Code, not terminals, and needs a link anyway.
- The repository's own `postinstall` also writing `~/.local/bin/git`: a repository install step would silently take over
  `git` for the whole user account on machines the user does not control, and would collide with any existing file.
- A mise `git` shim: works, but costs about 4 ms more per call than the link, and every `git` call then depends on
  `mise` starting correctly.
- Warn when `git` is not the wrapper: rejected by the user.

## Not covered

- Machines the user does not control.
  No client-side change can force the wrapper there, and auto-push cannot run server-side.
  Server-side rulesets or CI that run the same policies are the only enforcement that holds, and they need a separate
  decision.
- Operating systems other than Linux; the tool entry is `os = ["linux"]` because only Linux archives are released.
- The wording of `APG` and `APQ` in `AGENTS.md`, which follows once the guarantee is settled.

## Risks and open questions

- Every `git` call by every tool goes through the wrapper once the link exists, including IDEs, scripts and `gh`,
  which calls Git to infer the repository (`doc/troubleshooting/gh-implicit-repository-git-wrapper.md`).
  Policies such as the bulk-add rejection then apply to those callers.
  Overrides exist (`--no-enforce-*`), but how a given IDE reacts has not been measured.
- The latency numbers are from the lightest command in a container; real commands add the wrapper's own work, which the
  `native:perf` harness bounds.
- A running shell may have hashed the old `git` path; `hash -r` or a new shell picks up the link.
- The link follows `latest`, so no `postinstall` run is needed after an upgrade; `mise upgrade` itself was not tested,
  only installs of a newer version.

## Steps after acceptance

1. In podman, verify the atomic link form and that installing a newer version repoints the link.
2. Add the entry to the host's `~/.config/mise/config.toml` and run `mise install`.
3. Verify at the user boundary: `type -a git` in a clean login shell, and a `claude -p` commit in a scratch clone with
   a bare remote reaches the remote.
4. Write a short runbook for setting up another machine the user controls, and settle the `AGENTS.md` wording.
5. Update or close issue 1796 with the evidence.

## Rollback

Remove the entry from the global mise config and delete `~/.local/bin/git`.
The repository's per-worktree link is untouched, so this repository keeps working.
