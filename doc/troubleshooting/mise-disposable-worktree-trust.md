# Mise 2026.7.0 rejects tasks in a new disposable worktree until its config path is trusted

## Symptom

A package-scoped command in a new Git worktree stopped before the requested command ran:

```text
mise ERROR error parsing config file: /var/home/user/temp/agent/mono-cac-probe.kbXvUyzU/mise.toml
mise ERROR Config files in /var/home/user/temp/agent/mono-cac-probe.kbXvUyzU/mise.toml are not trusted.
```

The emitting tool was mise 2026.7.0.
Pnpm did not start,
so this was not a package-manager or CAC failure.

The main worktree's trusted `mise.toml` did not cover a new absolute path in a separate worktree.

## Root cause

Mise requires trust before loading configuration that can execute code or affect the environment.
The [`mise trust` reference][trust] says an untrusted config can be skipped,
prompted for,
or rejected when mise cannot prompt.

Trust is keyed by config path and stored as machine-local state.
The [mise directory reference][directories] identifies the default as
`${XDG_STATE_HOME:-$HOME/.local/state}/mise` and documents `MISE_STATE_DIR` as its override.
It explicitly names trusted config files as content stored there.

A Git worktree has a distinct absolute `mise.toml` path.
Trusting the main checkout therefore does not authorize the new worktree's config.
This is expected security behavior,
not evidence that its TOML is malformed.

## Verification

The following disposable harness reproduces the failure without using repository state:

```sh
scratch="$(mktemp --directory)"
mkdir --parents "${scratch}/state"
printf '%s\n' \
  '[env]' \
  'MISE_TRUST_PROBE = "yes"' \
  '' \
  '[tasks.show]' \
  'run = "printf probe"' \
  > "${scratch}/mise.toml"

MISE_STATE_DIR="${scratch}/state" \
  mise --cd "${scratch}" tasks ls
```

Observed pre-fix result:

```text
exit=1
mise ERROR Config files in .../mise.toml are not trusted.
```

Trust the exact config into disposable state and repeat with the same override:

```sh
MISE_STATE_DIR="${scratch}/state" \
  mise trust --yes "${scratch}/mise.toml"
MISE_STATE_DIR="${scratch}/state" \
  mise --cd "${scratch}" tasks ls
```

Observed post-fix result:

```text
exit=0
show
```

The verified scratch fixture was
`~/temp/agent/mise-trust-probe.7BxIOz7S`.
It failed before trust and passed after trust on 2026-08-14.

### Working catalog

- Main worktree command with its already trusted path.
- Disposable worktree command after trusting its exact config path.
- Isolated command when trust and execution share one `MISE_STATE_DIR`.

### Failing catalog

- New worktree path with no trust record.
- Config trusted in one state directory but executed with another.
- Retrying the original command without changing trust state.

## Verified workaround

Use a private state directory for the complete disposable-worktree lifecycle:

```sh
state="${HOME}/temp/agent/mise-state-cli-git-cac"
mkdir --parents "${state}"
chmod 700 "${state}"
MISE_STATE_DIR="${state}" \
  mise trust --yes \
  "${HOME}/temp/agent/mono-cac-probe.kbXvUyzU/mise.toml"
MISE_STATE_DIR="${state}" \
  mise --cd "${HOME}/temp/agent/mono-cac-probe.kbXvUyzU" \
  run //package/git-policy/cli:lint:types
```

This preserves mise's trust check while keeping verification state disposable.
Deleting the private state directory removes only the temporary trust record.
The user's ordinary mise trust registry is not touched.

Tradeoff:
every command in that worktree must receive the same `MISE_STATE_DIR`.
A missing override returns to ordinary state and can reproduce the error.

## What does not work

- Repeating `mise run` does not create trust.
- Trusting only the main checkout does not trust another worktree path.
- Treating the message as a TOML parse failure sends investigation to the wrong input.
  The second diagnostic identifies trust as the failed check.
- Trusting into ordinary mise state works,
  but it leaves persistent machine state for a disposable experiment.
  It is unnecessary when `MISE_STATE_DIR` is available.
- Changing only `HOME` also moves unrelated config,
  cache,
  data,
  and tool lookup boundaries.
  `MISE_STATE_DIR` isolates the trust record directly.

## Upstream filing artifact

### Upstream filing decision

1.  **Is it really upstream's fault?**
    No.
    Mise rejected an untrusted executable config path exactly as its trust documentation describes.
2.  **Can upstream fix it?**
    No fix is needed.
    `MISE_STATE_DIR` provides the required disposable-state boundary.
3.  **Are they supporting this use case?**
    Yes.
    The directory reference explicitly documents state-directory override and trust storage.
4.  **Would the repository welcome our contribution?**
    Not applicable because there is no defect or documentation gap established by this probe.
5.  **Will they likely fix it?**
    Not applicable because the behavior is intentional.
6.  **Have we prototyped a minimal compatible fix?**
    Yes for local operation:
    isolated trust plus the same isolated state on execution passes the positive control.

Do not file an upstream issue.
The durable artifact is this local troubleshooting record and the isolated-state command pattern.

[directories]: https://mise.en.dev/directories.html
[trust]: https://mise.en.dev/cli/trust.html
