# Secret values reaching a model provider from a Pi run

## What happened

On 2026-11-14 at 09:11:15 a Pi run this package started
was working out why `mise` could not decrypt `.env.local.json` under a disposable `HOME`.
It ran `mise env 2>&1 | head -3`.
The tool result held the real value of one credential that file declares,
and a tool result is sent to the model whole.

The rules file every brief names already forbids printing the environment
(`doc/rules-every-pi-brief-carries.md`).
That was not enough twice over:
the agent did not read `mise env` as printing the environment,
and nothing stood between the command's output and the model.

A coordinating session afterwards scanned 1,837 Pi sessions under `${HOME}/temp/agent/pi-runs`
for the real values of the 35 names `.env.local.json` declares,
by exact match.
It found real values of 30 names, in six sessions, each in a tool result.
The commands before those results:

- `mise env --json 2>/dev/null | head -40` (29 names in one result) and
  `mise env 2>&1 | head -3`.
- `tr '\0' '\n' < /proc/$pid/environ | grep -i "CARGO_TARGET\|MISE\|RUSTUP"`
  and `env | rg -i "CARGO_TARGET|MISE"`:
  a filter for one harmless word that also matched a token's name.
- `sed -n '40,70p' ~/.bashrc`:
  a file outside the repository that holds a token in clear.
- `cat ~/.pi/agent/models.json`:
  Pi's own provider file, which holds keys in clear.

## Every path text takes to the model

From Pi 1.5.2 as installed, with what this package's masking does about each.

- **A tool result**, whatever tool made it:
  bash, read, grep, ls, find, edit, write, and any extension tool.
  `tool_result` fires before Pi ends the call and keeps the message
  (`dist/core/agent-session.js:403`, and `finalizeExecutedToolCall` then `createToolResultMessage` in
  `dist/bundle/chunks/chunk-VEJDBY3O.js`), so masking there covers the model, the session file and the
  run's feed at once. Covered.
- **A tool that threw**: its error text becomes the tool result before the hook runs, so it is covered by the same hook.
- **The whole transcript before a request**, which is where a resumed session's earlier tool results,
  the first prompt, a steer or a note from the inbox, the system prompt (AGENTS.md, the skills),
  a compaction summary and every earlier assistant message travel:
  `context_with_system` sees all of it (`dist/core/extensions/runner.js:1011`,
  wired at `dist/core/sdk.js:277`). Masked on the way out. Covered.
- **Pi's compaction and branch summarization requests**:
  these build their own conversation text from the session
  (`dist/core/compaction/compaction.js:494` and `:524`, `dist/core/compaction/branch-summarization.js:208`)
  and do not run `context_with_system`.
  A value that a *new* run's session never holds is out of reach of them;
  a value an old session already holds could travel inside one. Not covered.
- **Pi's provider payload** (`before_provider_request`) can hold any of the above.
  Not used: masking the transcript is provider-neutral, and a payload rewrite would have to know each
  provider's shape.
- **A file the agent reads with the read tool** is a tool result, so it is covered the same way.

## What the masking does not cover

- A container the value was encoded inside: base64 of a whole file, hex, gzip, base64 wrapped at column 76,
  a substitution cipher. A decoder in front of every matcher would be needed, and this repository's
  leaks were the value itself.
- A value shorter than eight characters, because such a string appears in ordinary text by chance.
  Every value the store declares today is at least ten characters
  (`keep/measure-store-value-lengths.ts` in the report's scratch folder).
- A steer message's preview on the run's feed, which is a driver-written line
  (`src/watch-pi-run.ts`), is not masked: the message itself reaches the model masked.
- Pi's output files for a *greater* output than it shows are rewritten by the extension only when the
  details name a file Pi named `pi-output-*`.

## Fix

`README.md` under "What is masked before it reaches the model". The values are read once in the driver,
travel on a descriptor the driver opens for Pi, and are replaced in each tool result and again in the whole
transcript by a marker that names where the value came from.
