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

## Claude Code sessions are not covered

At 2026-11-08T17:53:25-05:00 a commit through `/usr/bin/git` in a throwaway repository succeeded from the coordinator's Claude Code session.
Fed the same PreToolUse payload directly,
`package/claude-code-plugin/guardrail/bundle/node/index.mjs` denied that command from both the session folder and the repository folder,
and passed a wrapper command through,
so the rule itself works.
`~/.claude/settings.json` holds no entry naming the guardrail,
and the repository's own `.claude/settings.json` is empty,
which fits that session not running the bundle.
Other ways a Claude Code session could load it were not examined.
Whether to register it is an owner question in `doc/handover/questions-for-the-owner.md`.

## Open

The first check straight after an install exited with `Unknown provider "neuralwatt"`.
Later checks through the same path passed.
The cause was not measured.

The guard does not detect a Git path it cannot read from the command text:
`bash -c "$CMD commit"`,
`G=/usr/bin/git; $G commit`,
and `PATH=/usr/bin:$PATH git commit`.
