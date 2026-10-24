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
or a list that names it)
without the literal `true` for the option,
naming the file and the line.
Options the scan cannot read are findings
(no options,
a variable,
a spread,
a `toFake` that is not a written-out list).
A clock that fakes the date alone is left alone by rule.
Out of its reach:
a clock installed through the fake-timers library itself,
and a `toNotFake` list that keeps `clearTimeout` real.
The process time stays a habit of the gate (ledger M288).

[real-clocks]:
  mistake-prevention-tests-clocks-and-arrival-order.md#tests-on-the-real-clock
