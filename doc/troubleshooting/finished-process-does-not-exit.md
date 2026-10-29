# A finished process that will not exit

FOUND 2026-08-17 in `@monochromatic-dev/module-translation-repair`,
FIXED the same day.

## Symptom

A corpus audit wrote its results,
printed `kept at <path>`,
and then did not exit for about another three minutes.
It held ten socket and pipe descriptors the whole time.

Nothing was lost:
the run file is already on disk when that line prints.
But anything waiting on the process waits for the tail,
a supervisor with a completion deadline can kill a run that finished correctly,
and to a reader it looks exactly like a hang.

## The wrong cause, which the evidence supported

The log was full of these:

```text
[warn] runRenderingAudit hf:nvidia/...: abandoned 180000ms after quorum (AbortError)
```

Ten open descriptors plus a log full of abandoned requests
reads as leaked sockets,
and that is what the first version of this document said.

It was wrong.
`runStageRound` DOES abort its stragglers,
through an `AbortController` every call in the round honors.
Reading the source settled in a minute
what the symptom had already mis-explained.

## The real cause

```ts
await Promise.race([
  Promise.allSettled(asks,),
  wait(graceMs,),
],);
```

`Promise.race` settles on the first result and does NOTHING to the loser.
When the roster answers before the grace expires,
which is the ordinary case,
`wait(graceMs)` is still pending.

And `wait` is a bare `setTimeout`
that returns no handle,
so nothing can clear it,
and never calls `unref`,
so it holds the event loop:

```ts
const { promise, resolve, } = Promise.withResolvers<undefined>();
setTimeout(function resolveAfterDelay(): void { resolve(undefined,); }, ms,);
return promise;
```

Every round therefore left one live timer.
The last round's is the one that matters,
and `graceMs` was 180000.

Not specific to the audit:
every stage round did it,
so any CLI that ran one could outlive its own work by up to the grace window.

## The fix

`@monochromatic-dev/async-time-core` gained `settleWithin`,
the same race with its loser cleared through a `using` disposer,
returning `'settled'` or `'expired'` as a VALUE:

```ts
await settleWithin({
  promise: Promise.allSettled(asks,),
  ms: graceMs,
},);
```

`withTimeout` was already in that module and already cleared its timer,
so the pattern was proven.
What it does not do is treat expiry as ordinary:
it rejects, which is right when the caller needed the VALUE
and wrong when the caller needed the WAIT.
Using it for a grace window means catching an error on the expected path
and then proving the error was the timeout's.

`unref` would also stop a timer holding the loop,
and is the weaker choice:
it hides the leak rather than removing it,
and a timer that no longer keeps the process alive still fires
on a process kept alive by something else.

Verified at the user boundary rather than by a return value:
a `--cap 1` audit now exits in 28 seconds for about 26 seconds of work,
with no tail.

## Testing a leak like this, which took two attempts

The first test counted timers with `process.getActiveResourcesInfo()`.
That DOES detect the leak,
proven with a positive control that leaks one deliberately
and watches the count move from 0 to 1,
against `settleWithin` holding at 1
and the naive race going from 1 to 2.

It is still the wrong test,
because a unit suite shares one process
with other cases that leak timers of their own,
so the baseline moves underneath the assertion
and the result depends on scheduling.

The test that landed spawns a CHILD PROCESS and measures whether it EXITS,
which is the property a reader recognises
and the one the defect is actually about.
Its positive control is the old pattern written out and asserted to hang,
because a test that only checked the return value
would pass on the broken version:
`Promise.race([work, wait(ms,),],)` returns exactly the right answer
and still holds the process.

## Telling a lingering process from a working one

If the last log line is `kept at <path>` and the file exists,
the work is DONE and the process is merely lingering.
Waiting it out is safe and so is killing it.
If that line is absent, it is still working.

That distinction is why a completion line is worth printing with its path in it
rather than relying on process exit,
and it is still worth having now that this particular tail is gone.

## The related trap, which is not this one

A wait loop that never fires can also be the WATCHER's fault rather than the
watched process's:
see [A `pgrep --full` wait loop matches its own shell and never exits](pgrep-wait-loop-matches-itself.md).
Both were hit within hours of each other,
each looking exactly like the other,
which is the argument for checking the log's completion line
before concluding anything about either.
