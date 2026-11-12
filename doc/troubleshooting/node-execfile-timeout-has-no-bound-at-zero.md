# Node's `execFile` timeout has no bound at zero and one millisecond above a 32-bit integer

## Symptom

`native:install --deadline-seconds 0` and `native:ts-checks --deadline-seconds 0`
printed nothing and were still running when a fixture stopped them after three seconds,
although both commands promise that nothing waits forever.
The opposite input was silent in a worse way:
`--deadline-seconds 2147484` stopped its first step after a few milliseconds.

A person reaches this by typing a deadline the option accepts.
`numberOption` reads a whole number from zero,
so `0` became `deadlineMilliseconds = 0`,
which the install and the check runner pass to `node:child_process.execFile` as `timeout`.
Node's `timeout` is `0` by default,
and `0` means no timeout at all,
so the bound that the option exists to give was absent.
The clear refusal the commands print now is at the end of this document.

## Source trace

Read on 2026-11-11 in this checkout:

- Node `v26.13.0` (`node --version`).
- `node_modules/@types/node/child_process.d.ts:537-545` declares `CommonOptions.timeout`
  with `@default 0`,
  and `:515-529` declares `MessagingOptions.timeout`
  as "In milliseconds the maximum amount of time the process is allowed to run".

A `timeout` of `0` arms no timer at all,
so the command runs to its own end or never.
Node's timers take a delay as a 32-bit signed integer,
and a larger delay warns and uses `1` instead:

```text
(node:194389) TimeoutOverflowWarning: 2147483648 does not fit into a 32-bit signed integer.
Timeout duration was set to 1.
```

`execFile` passes `timeout` to that timer,
so a deadline past `2_147_483_647` milliseconds stops the command at once instead of bounding it.

## Measurement

Both holes were measured on this host on 2026-11-11,
before any guard was written,
with a command that outlives its bound.
Zero,
which let a two-second command run to its own end:

```sh
# Two seconds of `sleep`, given no bound at all.
node --input-type=module -e '
import { execFile } from "node:child_process";
const probe = (timeout) => new Promise((resolve) => {
  const started = Date.now();
  execFile("sleep", ["2"], { timeout, killSignal: "SIGKILL" }, (error) => {
    resolve({ timeout, milliseconds: Date.now() - started, killed: error?.killed ?? false });
  });
});
console.log(JSON.stringify(await probe(0)));
'
```

```text
{"timeout":0,"milliseconds":2006,"killed":false}
```

The largest honoured delay,
and one millisecond more:

```sh
# The same command, bounded first by the largest 32-bit delay and then by one more millisecond.
node --input-type=module -e '
import { execFile } from "node:child_process";
const probe = (timeout) => new Promise((resolve) => {
  const started = Date.now();
  execFile("sleep", ["2"], { timeout, killSignal: "SIGKILL" }, (error) => {
    resolve({ timeout, milliseconds: Date.now() - started, killed: error?.killed ?? false });
  });
});
process.on("warning", (warning) => console.log("warning:", warning.name, warning.message));
console.log(JSON.stringify(await probe(2_147_483_647)));
console.log(JSON.stringify(await probe(2_147_483_648)));
'
```

```text
{"timeout":2147483647,"milliseconds":2007,"killed":false}
(node:194389) TimeoutOverflowWarning: 2147483648 does not fit into a 32-bit signed integer.
Timeout duration was set to 1.
{"timeout":2147483648,"milliseconds":4,"killed":true}
```

The zero case was measured through the commands themselves as well,
on a throwaway checkout whose stand-in `mise` never exits:
both `native:install --deadline-seconds 0` and `native:ts-checks --deadline-seconds 0`
were still running after three seconds and had to be killed,
while their one-second controls ended by themselves,
the install with status 1 and the checks with status 2 after ending the scope.

## What this repository does now

`bin/native-gate-arguments.ts` reads every step deadline through `deadlineMillisecondsOption`,
which refuses `0` and anything longer than `2_147_483` seconds,
naming the option and the shortest and longest deadline it takes.
`native:install` and `native:ts-checks` both use it,
so neither can hand Node a `timeout` that is not a bound.

Any other caller of `runGateCommand` that computes a bound from a command line
must do the same,
because `runGateCommand` passes the value through unchanged.
