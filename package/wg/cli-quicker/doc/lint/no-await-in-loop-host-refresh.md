# `no-await-in-loop` suppression in the update-host refresh loop

`src/bypass-host-refresh.ts` `runHostRefreshLoop` runs one refresh after another until the watcher is asked to stop.
Its loop body is a single `await refreshThenWait(...)`,
and that one line carries the only suppression.

## Why the shape exists

Each refresh lists the rules the previous refresh left in the kernel,
decides what to add and remove,
and then waits for the smallest DNS TTL.
Starting the next refresh before the previous one finished would plan against rules that are still being changed.
The loop therefore must wait for each round,
one after another,
for as long as the watcher lives.

The wait is not a second suppression.
It is `sleepUnlessAborted`,
which ends at once when the stop signal aborts,
and it lives inside `refreshThenWait`,
so the loop body holds one `await`.

## Why the other shapes do not work

- Recursion (`await runHostRefreshLoop()` from itself) keeps every earlier call's frame alive for the life of a watcher that runs for days.
- A promise chain is forbidden by the repository rule that allows `async` and `await` only.
- A timer that schedules the next refresh from the end of the previous one is the same loop spelled as callbacks,
  and it loses the single place where the stop signal is checked.
- The bounded `ip` command runner (`src/map-in-chunks.ts`) needs no suppression because it recurses over a list whose depth is the item count divided by the chunk size.

## Why config cannot resolve it

The rule has no options.
The oxlint rule page (<https://oxc.rs/docs/guide/usage/linter/rules/eslint/no-await-in-loop.html>,
read on 2026-10-31) says:
"This rule disallows the use of `await` within loop bodies."
It lists the covered loops as "(for,
for-in,
for-of,
while,
do-while)",
so a `while` loop is covered,
and its only documented switch is turning the rule on or off.
Turning the rule off for the package would loosen a lint rule,
which needs prior approval,
so the tightest suppression is the one `oxlint-disable-next-line` comment in `runHostRefreshLoop`.

## Sources

- Rule behavior:
  the oxlint page above.
- The suppressed line:
  `src/bypass-host-refresh.ts`,
  the `oxlint-disable-next-line eslint/no-await-in-loop` comment in `runHostRefreshLoop`.
