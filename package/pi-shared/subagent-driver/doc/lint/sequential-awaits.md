# `no-await-in-loop` suppressions in the subagent driver

## Rule and configuration

`package/config/oxlint/src/rule/correctness.ts` enables `eslint/no-await-in-loop` as a warning,
and this package's lint exits 1 on any warning.
The Oxlint rule has no options and no allow-list,
so turning it off for this package would loosen a lint rule,
which needs prior approval.
Each loop that must wait for one step before the next carries one disable comment,
and the tests use one per wait inside a polling loop.

## Where the order is the design

- The watch (`src/watch-pi-run.ts`):
  every tick looks at the limits,
  reads the next message,
  and waits for Pi's answer before the next tick.
  Delivering two messages at once would reorder the inbox,
  and looking at the limits while a delivery is outstanding would report the end of the run before the message's own feed
  line.
- The wait for runs that should have come back (`src/watch-for-overdue-runs.ts`):
  a look,
  then one wait,
  then the next look.
  One command has to stay up while the runs work,
  and each look reads the whole finished corpus,
  so a promise per look would read that corpus many times at once and separate the waits from the deadline that bounds them.
- Emptying a run's scratch folder (`src/deleting-what-a-run-left-in-its-scratch-folder.ts`):
  one entry at a time,
  then one folder at a time, deepest first.
  A deletion of tens of GiB runs at the idle disk class,
  so it must neither starve another process's reads nor open a descriptor for every file at once,
  and the folders come out after everything inside them is gone so a folder that still holds an entry is named rather than
  its parent being reported for it.
  The count of a kept folder takes the same shape for the same reason.
- The same deletion in a test (`src/emptying-a-scratch-folder.unit.test.ts`):
  one run name at a time,
  each folder made, filled, emptied and read before the next one,
  so a failure names the name it belongs to.
- The test polling helpers (`src/drive-pi-subagent.unit.test.ts`,
  `src/connection-to-pi.unit.test.ts`,
  `src/record-of-pi-run.unit.test.ts`):
  a test waits for a feed line,
  a log line or a reported problem until a deadline,
  reading the file again after each look.
  Reading once and giving up would make every such test a sleep in disguise.

## Why the other shapes do not work

- Recursion keeps a frame per look for a run that lasts hours.
- A promise chain is forbidden by the repository rule that allows `async` and `await` only.
- `Promise.all` over the inbox starts every delivery at once,
  which the message order forbids.
- A file watcher reports changes that may not be visible yet,
  so a look at the file after each event is still needed,
  and the watch must also see the stop file and the wall clock whether or not any file changes.
- Timers alone cannot replace the wait between looks of the overdue watch:
  the deadline has to shorten the last wait,
  which is one calculation the loop does with what it has left.
