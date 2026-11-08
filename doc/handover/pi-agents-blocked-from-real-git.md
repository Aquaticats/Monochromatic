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
and `PATH=/usr/bin:$PATH git commit`.
