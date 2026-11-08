# Pi agents blocked from real Git

## State at 2026-11-08T17:45:18-05:00

The guardrail in `package/pi-plugin/guardrail` denies a mutating Git run through a real-Git path (merge `f0f033a6be`).
The package is listed in the `packages` array of `~/.pi/agent/settings.json`,
so every Pi process started from now on loads it.
A real agent in a throwaway repository,
started with those real settings,
was told to run `/usr/bin/git commit`.
The guardrail refused the call and the repository holds no commit.

Agents already running loaded their extensions when they started,
so they are not covered until they restart.

## Guarded spellings at 2026-11-08T23:15:00-05:00

The rule behind both guardrails lives in `package/agent-harness-shared/shell-command-analyzer`.
Beside an absolute path,
it now denies a command that takes the wrapper off the search path:

- `PATH=/usr/bin:$PATH git commit`, `env PATH=/usr/bin git commit`,
  a standalone `PATH=/usr/bin; git commit`, and `export PATH=/usr/bin; git commit`,
  including the same assignment in front of a nested shell (`PATH=/usr/bin bash -c 'git commit'`).
- `command -p git commit`, `env -i git commit`, and `env -u PATH git commit`.
- The attached `env -S'...'` payload, and the arguments GNU `env` appends to that payload.

The analyzer README lists every covered and every undetected spelling.
Each guarded spelling has a unit test that fails when its check is removed.

## What can stop every Pi from starting

The built plugin imports `@monochromatic-dev/git-executable-core/ts` at run time.
Listing the plugin before that package was linked in the main checkout made Pi exit at start with
`Failed to load extension`.
The entry was removed within minutes and no agent hit the failure (no `stderr.log` under the run folders held that message).
`mise exec -- pnpm install --offline --frozen-lockfile` in the main checkout linked it,
and the entry was added back only after a run against a throwaway agent directory passed.
If that message appears again,
run that install or remove the entry.

## Open

The first check straight after an install exited with `Unknown provider "neuralwatt"`.
Later checks through the same path passed.
The cause was not measured.

The guard does not detect a Git path it cannot read from the command text:
`bash -c "$CMD commit"`,
`G=/usr/bin/git; $G commit`,
`eval "$G commit"`,
and `$(command -v git) commit`.
It also does not follow a program that runs another program but carries no wrapper entry,
such as `find -exec`, `flock`, `busybox`, `systemd-run`, `watch`, or `ssh`.
Closing either gap needs a shell evaluator or a larger wrapper table, not a text rule.
