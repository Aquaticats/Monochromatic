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

## Every place a value could be written or sent, and the point before it that masks

A list of Pi's hooks is what this package started with, and it was short by one twice: a tool result's `details` and every
message Pi keeps. So the write-down starts from the places a value could be written or sent, not from the hooks.

| Place | Where it happens, in Pi 1.5.2 as installed | The one point before it that masks |
| --- | --- | --- |
| A tool result message: `content`, `details`, `structuredContent` | `dist/core/agent-session.js:403` (`_afterToolCall`), then `finalizeExecutedToolCall` and `createToolResultMessage` in `dist/bundle/chunks/chunk-VEJDBY3O.js` | the `tool_result` handler, which returns every field masked (`secret-value-masking-extension.ts`) |
| Any message Pi keeps, of every role and every field: the first prompt, a steer, an inbox prompt, an assistant message with its tool-call arguments, a tool result, a custom message | `dist/core/agent-session.js:994` (`emitMessageEnd`), the replacement applied at `:1000-1013`, then the append at `:814` and the write at `dist/core/session-manager.js:813` | the `message_end` handler, which returns the masked message before Pi keeps it |
| An ordinary model request, with the whole transcript | `dist/core/extensions/runner.js:1011` (`emitContext`), wired at `dist/core/sdk.js:277` | the `context_with_system` handler |
| Pi's compaction request | `dist/core/agent-session.js:2338` (`getBranch`), `:2302` (`_runDefaultCompaction`), `dist/core/compaction/compaction.js:494` and `:524` | no hook runs before it; it is built from the session's own messages, which the `message_end` handler has already masked, so it can hold no value |
| Pi's branch-summary request | `dist/core/compaction/branch-summarization.js:208` | the same: it reads the session, which is masked at `message_end` |
| The file Pi keeps a truncated tool's whole output in | `dist/utils/output-files.js`, `dist/core/tools/output-accumulator.js:81` | the `tool_result` handler, which masks it and puts the masked text in place through a file of its own renamed over it |
| The driver's `feed.jsonl`, every record: tool arguments, tool results, an inbox preview, Pi's event detail, a problem | `src/record-of-pi-run.ts` (`note`) | one boundary there: every entry is masked whole before the line is written |
| The driver's `summary.json`: the last reply, a tool error, what Pi printed when a start failed | `src/write-summary.ts` | the same masking, applied to the whole summary before it is written |
| The driver's `stderr.log`, which holds Pi's own diagnostics and an extension's | `src/keep-standard-error.ts` | the masking on the way into the file, holding back the longest form's length between two writes so a value split across them is still matched |
| The session file at rest: compaction and branch-summary entries, context edits, and a custom entry an extension wrote | `dist/core/session-manager.js:816` (`_appendEntry`) for every kind | no hook runs before `appendCompaction`, `appendContextEdit` or `appendCustomEntry`, and Pi's interface gives an extension no way to intercept them; the file is masked whole once the run is over (`src/session-file-at-rest.ts`), which is before any later request and before any reader of the run's folder |

Two of those are not "before the sink" and are named as such: the compaction and branch-summary requests, which are covered
by the session being masked rather than by a hook of their own, and the session file at rest, which is masked after the run
because Pi offers no point before those three entry kinds.

What the run does about a place it cannot cover: it does not start. A store, Pi's credential file, a shell start-up file or
the age identity that is there and cannot be read stops the run before Pi is started, and a Pi that does not answer a command
list saying the masking is armed is sent no prompt. An entry an extension writes during the run is the one place no hook
reaches: it never reaches a model request (compaction reads messages, not custom entries), and it does not survive the run,
because the file is masked at rest.

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
