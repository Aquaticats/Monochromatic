# Preventing mistakes: fake clocks and native timers

Rules for tests that replace the clock:
which timers the fake clock owns,
and what happens to a timer it did not make.

Part of [Preventing this package's mistakes](mistake-prevention.md),
which lists every file of this record.
The real-clock side of the same tests is in
[Preventing mistakes: the real world, clocks and arrival order][real-clocks].

## Fake clocks that leave native timers

What happened:
the seat-reading fixture of the run package faked `setTimeout` and `clearTimeout` after the module logger had begun
its start check,
a native timer of five seconds.
The fake `clearTimeout` refuses a timer it did not make:
it writes `FakeTimers: clearTimeout was invoked to clear a native timer` to stderr and leaves the timer running.
Two suites of about 100 ms each then held their process for about 5.3 s,
and nothing the suite printed said so (ledger B837).
The change had been gated on the suites' times (677 ms to 158 ms) and on the claim that no case sleeps,
and the whole-package `test:unit` time moved by under a second because the files run in parallel.
Three more installs of a fake clock in three packages carried the same omission,
and the payment-hold suite had the option and a comment saying why.

The rule:
a fake clock that fakes `clearTimeout` is installed with `shouldClearNativeTimers: true`,
so a clear of a timer made before the clock was installed reaches the real `clearTimeout`.
A change that moves a suite to a fake clock is gated on the process,
not only on the suite:
the whole command of that file is timed three times,
and its stderr is read for a `FakeTimers` line.

What to do when a scan fails:
add the option to the install the finding names.
A case that needs a native clear to fail does not exist,
since the real `clearTimeout` of a timer already gone does nothing.

What enforces it:
`fake-timer-native-clear.unit.test.ts`,
among the source scans,
fails any `useFakeTimers` call of any family package,
tests and fixtures included,
that fakes `clearTimeout`
(no `toFake` list,
an empty one,
or a list that names it)
without the literal `true` for the option,
naming the file and the line.
Options the scan cannot read are findings
(no options,
a variable,
a spread,
a `toFake` that is not a written-out list).
An empty `toFake` list is read as naming every timer:
the `install` function of `@sinonjs/fake-timers` (version 15.4.0 installed here)
takes the keys of every timer when the list it is handed has no member
(`fake-timers-src.js`,
the `hasToFake` branch),
so `toFake: []` fakes `clearTimeout` and `toFake: ['Date']` does not.
A clock whose non-empty list leaves `clearTimeout` out,
the date alone for one,
is left alone by rule.
Out of its reach:
a clock installed through the fake-timers library itself,
and a `toNotFake` list that keeps `clearTimeout` real.
The process time stays a habit of the gate (ledger M288).

## Real waits that a fake-clock walk leaves running

What happened:
the two bench suites that walk a call on a fake clock
(`model-health-probe.unit.test.ts` and `translate-probe-run.unit.test.ts`)
held their process for about 61 s after the last case,
with `shouldClearNativeTimers` already set on their fake clock.
The walk races its calls against real waits of the hang stop,
one minute each,
and never ended them.
Each is a real timer,
and a real timer nobody ends keeps its process alive until it fires.
A trace of every timer of a second or more,
taken with an async-hook preload on the model-health-probe suite,
named those waits in `fake-clock-walk.test-fixture.ts`
(five timers of 60,000 ms)
and nothing else (ledger B837).
The whole `mise run` command of each file took 60.96 s and 61.05 s,
and 61.78 s and 61.28 s,
two runs each;
it takes 0.98 s to 1.26 s and 1.30 s to 1.56 s,
three runs each,
with the waits ended.

The rule:
a test fixture that starts a real wait it will not wait for
hands it an abort signal and aborts it when the fixture returns,
however it returns.
The check that no real timer is left is a count of the process's live timers
(`process.getActiveResourcesInfo()`) before and after,
which names the cause in a case,
where a bound on the process wall would only show it as slow.

What to do when a suite holds its process after its last case:
preload a hook on `node:async_hooks` that writes the stack of every `Timeout` of a second or more,
run the file with `node --import <preload> <file>`,
and read the stacks;
the caller that made the long timer is the one to fix.
The `FakeTimers` line on stderr is only one cause of a held process.

What enforces it:
the case `LEAVES NO REAL TIMER LIVE WHEN THE WALK HAS ENDED` of `fake-clock-walk.unit.test.ts`.
No scan reads fixtures for real waits that are never ended;
a scan would have to follow where each wait goes,
and the count in the case reads the result.

[real-clocks]:
  mistake-prevention-tests-clocks-and-arrival-order.md#tests-on-the-real-clock
