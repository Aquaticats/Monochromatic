# Cargo's loop warnings before the first error

Status as of 2026-11-11T18:30:00-05:00:
measured,
the check runner reads a check's whole output,
and the fix-forward report shows the error a long noisy start used to hide.

## Symptom

A fix-forward issue from the daily update
(`doc/planning/toolchain-always-latest.md`,
section "Checking every package after the update")
says a check failed "without printing a line that starts with error",
although the check did print one.
The reproduction command in the issue,
run by hand,
fails with an ordinary compiler error.

That is what issue 6284 reported for `package/git-policy/cli` `lint:clippy`
after the daily update of 2026-11-11,
for a `clippy::manual_map` finding in `worktree_copy_prune.rs`.

## What was measured

`mise run //package/git-policy/cli:lint:clippy`
at the update commit `637a834afd`
in a worktree of that commit,
run the way the daily update runs a check
(`NO_COLOR=1`,
`CARGO_TERM_COLOR=never`,
`CARGO_BUILD_JOBS=2`):

- The whole output was 184,067 characters over 551 lines.
- Its first 257 lines,
  173,538 characters,
  were `warning: File system loop found: ...` and nothing else.
- The compiler's first error stood at character 183,066,
  line 532 of 551.

So the first error was in the last 0.5 percent of the output,
behind warnings that cargo printed before it started compiling anything.

## Cause

Two facts,
both needed for the report to lose the error.

Cargo itself prints one `warning: File system loop found: ...` line
for every symlink loop its walk meets,
naming a path and the ancestor it points back to.
The pnpm link farm under
`package/forbidden-strings/cli/node_modules/@monochromatic-dev/`
holds such loops:
a workspace package's `node_modules` links back to packages whose own
`node_modules` link forward again.
`strace` on a build shows the writing process is the real cargo,
not mise and not cargo-shadow,
so no repository-owned tool emits these lines and none can silence them.

The check runner kept the first 20,000 characters of a check's output
and found the first error in that sample alone.
The sample of this run held no error line,
so `firstErrorOf` answered its "no error in the output" sentinel
and the issue carried
"the check failed without printing a line that starts with error".

## Fix

`package/monorepo-task/core-exec/src/daily-update-first-error.ts`
reads the output as it streams past:
`firstErrorScan` keeps the first error it meets and its excerpt lines,
wherever in the output that error stands,
and `firstErrorOf` is now that same read applied to one text,
so the two cannot disagree.

`package/monorepo-task/core-exec/src/daily-update-check-command.ts`
feeds it (`keepFirstError`),
splitting each stream into lines and decoding each stream as UTF-8 itself,
so a character split across two chunks arrives whole.
A line is kept only up to 8,192 characters,
and a line that never ends stops growing there,
so a check holds a bounded amount whatever its command prints.
The longest line of the recorded output was 902 characters,
nine times less than that bound.

`CheckOutcome` carries the error instead of a sample of the output,
so nothing downstream truncates it again.

## How to recognize it

The issue's excerpt names the sentinel,
and the check's own output,
run by hand,
begins with hundreds of `warning: File system loop found` lines.
Read past them:
the compiler's first error is there.

A check that genuinely prints no line that starts an error
is still reported that way,
for example a container build that ends on a failing step
with no compiler diagnostic of its own.

## Limits

- The warnings stay in a build's output;
  only the report no longer loses the error behind them.
  Cargo prints them for the pnpm link farm this repository's workspace
  layout creates,
  and silencing them would mean changing that layout or
  cargo's own walk.
- The scan recognizes the error lines this repository's tools print:
  a Rust error with or without a code,
  and a TypeScript error,
  with or without an ANSI color before it
  and with or without the `mise` task prefix.
  A tool whose error line starts some other way is reported as printing
  no error,
  and the issue's excerpt carries the reproduce command.
