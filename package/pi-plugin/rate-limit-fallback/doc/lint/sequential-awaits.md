# `no-await-in-loop` suppressions in the rate-limit-fallback package

## Rule and configuration

`package/config/oxlint/src/rule/correctness.ts` enables `eslint/no-await-in-loop` as a warning, and
this package's lint exits 1 on any warning. The rule has no options and no allow-list, so a loop that
must wait for one step before the next carries one statement-scoped
`oxlint-disable-next-line` comment on the `await` line. Turning the rule off for this package would
loosen a lint rule, which needs prior approval.

## Where the order is the design

`src/verify-overflow-session.ts` `overflowSession` sends one prompt per Pi process and each prompt
resumes the transcript the previous one wrote (`--session-dir` with one `--session-id`). The third
turn's overflow and compaction are what the check measures, so it exists only after the first two
turns have settled; running the prompts together would give three unrelated sessions, none of them
over a padded transcript.

`src/verify-hybrid-sessions.ts` runs one disposable home for every arm and lists that home's
package directory and stand-in extension in one `settings.json`. Each arm is a real `pi` process that
writes into the same home, so the next arm starts after the previous one has exited; concurrent arms
would contend for the same settings file and session directory.

## What the wait is for

Both waits are for another process (`pi`) to finish writing state the next step reads, not for a
condition this process can poll. Neither loop can be expressed as a `Promise.all`: the second
element's input is the first element's output.

The ordinary alternative, a chained `reduce` or recursion over the prompts, would replace the loop
with the same ordering expressed less directly, and `AGENTS.md` rule `ITR` keeps linear input on
iteration rather than recursion.
