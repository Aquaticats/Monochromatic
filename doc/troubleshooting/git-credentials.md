# `gh auth setup-git` writes a version-pinned absolute path into `~/.gitconfig`, breaking `git push` on every `mise upgrade gh`

## Symptom

`git push` works.
 `mise upgrade gh` runs successfully (e.g. moves
gh from 2.89.0 to 2.90.0).
 `git push` now fails with an
authentication error.
 Running `gh auth setup-git` again restores
credentials,
 but the fix lasts only until the next upgrade.

```sh
git push                 # works
mise upgrade gh          # upgrades 2.89.0 -> 2.90.0
git push                 # auth failure
gh auth setup-git        # rewrites gitconfig with new versioned path
git push                 # works again, until next upgrade
```

## Root cause

`gh auth setup-git` resolves its own binary path via
`os.Executable()` and writes that absolute version-pinned path
into `~/.gitconfig`:

```ini
[credential "https://github.com"]
    helper =
    helper = !/var/home/user/.local/share/mise/installs/gh/2.89.0/gh_2.89.0_linux_amd64/bin/gh auth git-credential
```

Source citations from [cli/cli](https://github.com/cli/cli):

- `pkg/cmd/auth/setupgit/setupgit.go:56`:
   calls `f.Executable()`
  to get the binary path:

```go
// Approximate excerpt from the referenced file
path, err := f.Executable()
```

- `pkg/cmdutil/factory.go:42-51`:
   checks `GH_PATH` env var
  first;
   if unset,
   calls `executable()`:

```go
if path := os.Getenv("GH_PATH"); path != "" {
    return path, nil
}
return executable()
```

- `pkg/cmdutil/factory.go:69-105`:
   `executable()` calls
  `os.Executable()` to get the running binary,
   then scans
  `$PATH` for any symlink whose `filepath.EvalSymlinks` target
  equals the same real binary.
   The intent is to prefer a stable
  Homebrew-style symlink (e.g. `/usr/local/bin/gh`) over the
  versioned cellar path.

The symlink walk is designed for Homebrew:

- `/usr/local/bin/gh` -> `/usr/local/Cellar/gh/<version>/bin/gh`

With mise it fails:

- Mise shim:
   `~/.local/share/mise/shims/gh` ->
  `~/.local/bin/mise` (the mise binary itself,
   not the gh
  binary).
- `filepath.EvalSymlinks` on the shim resolves to the **mise
  binary**,
   not the gh binary.
- The comparison at `factory.go:98` (`realP == realExe`) fails
  because `mise != gh`.
- The fallback returns the versioned install path from
  `os.Executable()`.

When mise installs a new gh version,
 the old versioned directory
(`mise/installs/gh/2.89.0/`) is removed,
 and the gitconfig entry
points to a nonexistent binary.
 The credential helper invocation
fails;
 git falls back to interactive auth or errors out.

## Verification

Versions under test:

- mise 2026.3.8 linux-x64
- gh 2.89.0 (before upgrade),
   2.90.0 (after)
- git 2.45+

Reproduce:
 install gh via mise;
 run `gh auth setup-git`;
 inspect
`~/.gitconfig` to see the versioned path;
 run `mise upgrade gh`;
attempt `git push`;
 observe the auth failure.

## Verified workaround: rewrite to the stable mise shim path + set `GH_PATH` env

Two changes,
 both required.

### Fix 1: Replace versioned paths in `~/.gitconfig` with the stable mise shim

```ini
[credential "https://github.com"]
    helper =
    helper = !/var/home/user/.local/share/mise/shims/gh auth git-credential
[credential "https://gist.github.com"]
    helper =
    helper = !/var/home/user/.local/share/mise/shims/gh auth git-credential
```

The shim is a symlink to the mise binary,
 which dynamically
resolves the correct gh version at runtime.
 The shim path never
changes across upgrades.

Tradeoff:
 the gitconfig is hand-edited;
 future `gh auth
setup-git` invocations will overwrite it back to the versioned
path unless Fix 2 is also applied.

### Fix 2: Set `GH_PATH` so future `gh auth setup-git` writes the shim path

In `~/.config/mise/config.toml`:

```toml
[env]
# Prevents `gh auth setup-git` from writing a version-pinned absolute
# path into ~/.gitconfig. Without this, mise upgrades break the
# credential helper. See:
# https://github.com/cli/cli/blob/trunk/pkg/cmdutil/factory.go
GH_PATH = "/var/home/user/.local/share/mise/shims/gh"
```

`factory.go:43-46` checks `GH_PATH` before calling
`os.Executable()`.
 With this env var set,
 `gh auth setup-git`
writes the shim path instead of the versioned path.

Tradeoff:
 `GH_PATH` overrides gh's own resolution everywhere,
 not
just for `setup-git`.
 Acceptable because the shim is the correct
binary in all gh invocations on this machine.

## What does not work

- Running `gh auth setup-git` after each upgrade:
   rewrites the
  versioned path every time;
   fix is temporary by design.
- Setting the credential helper to bare `gh` (no absolute path):
  Homebrew's `brew update` runs git commands without
  `/usr/local/bin` in PATH,
   so bare `gh` fails in that context.
  The gh authors intentionally use absolute paths to work around
  this (see comment at `factory.go:57-68`).
   Not a problem for
  mise users since the shim path is absolute.
- Expecting `gh auth setup-git` to detect mise shims:
   the
  symlink resolution logic cannot follow the mise shim
  indirection (shim -> mise binary,
   not shim -> gh binary).

## Native push wrapper loses the inherited environment

### Observed failure and contrast

During auto-mode migration work,
commit `59d5d36943` was saved locally,
but its auto-push reported
`fatal: could not read Username for 'https://github.com': No such device or address`.
A separate `git push origin main` in `proc_67eb` failed in `git-policy-cli` with
`code: content-unavailable` and
`Git could not determine push updates` wrapping the same Git credential diagnostic.
Neither invocation establishes a successful push.

In the same restricted launcher,
`gh auth status` reported an active keyring account,
and `git config --show-origin --get-regexp credential` showed the stable mise shim helper.
`git ls-remote origin refs/heads/main` then succeeded.
These probes distinguish this push-path failure from a universally unavailable credential or remote.
They do not prove token expiry or a broken version-pinned helper.

### Source evidence

The native manual-push path in
`package/git-policy/cli/src/native/wrapped_command.rs:535`
supplies only the child overlay to `PushGit`.
This is an annotated source listing,
not a standalone Rust program:

```text
# package/git-policy/cli/src/native/wrapped_command.rs
535: let git = PushGit {
536:     real_git: context.real_git,
537:     cwd,
538:     environment: context.overlay,
539: };
```

The post-commit path does the same at
`package/git-policy/cli/src/native/commit_command.rs:223`.
`package/git-policy/cli/src/native/auto_push_single_flight.rs:202`
clears the environment before applying that field:

```text
# package/git-policy/cli/src/native/auto_push_single_flight.rs
204: command
205:     .current_dir(&git.cwd)
206:     .env_clear()
207:     .envs(git.environment.iter().map(|pair| {
208:         return (&pair.0, &pair.1);
209:     }))
210:     .args(arguments);
```

`package/git-policy/cli/src/native/child_environment.rs:307`
constructs the overlay from lock-PID configuration additions and the forward-target marker,
not the full caller environment.
Its stated contract explicitly requires the remaining caller environment to be inherited.
The push builder therefore conflicts with that overlay contract.
This source finding is consistent with the credential failure;
no patched-binary contrast has yet established recovery for this incident.

### Disposition

No credential settings,
tokens,
Git policies,
or wrapper source were changed during this diagnosis.
Repeating the ordinary push did not recover it.
The migration continues with local commits;
remote publication remains unverified.
This is a repository-owned wrapper finding,
not evidence of an upstream Git or GitHub CLI defect.
No upstream report or speculative configuration workaround is warranted by these observations.

## Research-branch recovery with a scoped helper environment

On 2026-10-18,
the ordinary HTTPS push of research commit `3829831039cc43aa717c6c927f835b867e9313b6`
reported a remote rejection with `(failed)`.
That diagnostic differs from the missing-username incident;
the observations do not establish an identical cause.
A read-only SSH bridge using the existing agent and strict known-host checking separately returned
`Permission denied (publickey)`.

A credential-fill control returned a password with the inherited environment,
while the cleared-environment control exited 128 with
`fatal: unable to get password from user`.
The executable resolved through `PATH` was not independently fixed in that contrast,
so this is supporting evidence rather than an isolated environment-variable experiment.
Credential values remained in mode-0600 private captures and were not published.

The successful recovery kept the normal Git wrapper and its policy checks.
For that command only,
it cleared the generic and GitHub-specific helper lists,
then selected the existing absolute-path `gh auth git-credential` helper prefixed with `/usr/bin/env`.
The prefix restored caller-derived `HOME`,
`PATH`,
`DBUS_SESSION_BUS_ADDRESS` and `XDG_RUNTIME_DIR`,
with each value encoded as a single shell argument.
The existing `push-mvm-with-scoped-helper-environment` research task executed this construction
and `git push origin research/depot-native-orchestration-20261018`.
It exited zero and published `3829831039cc43aa717c6c927f835b867e9313b6`;
the subsequent dispatch independently required the remote ref to equal that commit and passed.

The relevant command-scoped configuration arguments were:

```text
# Arguments passed by the executed Node research task, with the private helper value described in prose
-c credential.helper=
-c credential.https://github.com.helper=
-c credential.https://github.com.helper=<existing-helper-with-scoped-env-prefix>
push origin research/depot-native-orchestration-20261018
```

No persistent credential configuration,
token,
Git policy or wrapper binary changed.
Restoring these variables together proves this recovery,
not which individual variable mattered or the cause of every remote rejection.
This is a completed bridge for the research branch,
not a verified wrapper-source repair or recovery of the separate migration incident.

## Token expiry note

`gh auth status` may show a `gho_*` token (OAuth) rather than a
`ghp_*` token (PAT).
 OAuth tokens can expire independently of
the credential-helper path issue.
 If credentials fail even with
a correct shim path,
 check `gh auth status` for token expiry and
re-authenticate via `gh auth login`.

## Why we do not file this upstream

The cited factory source explains the historical path-selection boundary,
and the recorded shim-path/`GH_PATH` workaround addresses that incident.
This investigation has not prototyped an upstream patch,
established maintainer intent or verified contribution acceptance.
The separate native-wrapper environment finding belongs to this repository;
the scoped helper recovery does not establish an upstream GitHub CLI defect.
No upstream report is ready from these observations.
