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
- The starts of a run (`src/supervise-pi-run.ts`):
  one start of Pi,
  then the pause,
  then the next start.
  Whether another start is made turns on how the last one ended,
  which is known only after it has ended,
  and two Pis started at once would both take the same prompt.
  The loop is bounded by the number of starts,
  by the time one start-up may take,
  and by the run's own wall clock.
- The wait for runs that should have come back (`src/watch-for-overdue-runs.ts`):
  a look,
  then one wait,
  then the next look.
  One command has to stay up while the runs work,
  and each look reads the whole finished corpus,
  so a promise per look would read that corpus many times at once and separate the waits from the deadline that bounds them.
- Claiming a run folder (`src/claim-a-run-folder.ts`):
  one `mkdir`,
  then the number after it,
  then the next `mkdir`.
  Which number is free is what that claim's own answer says,
  and two claims taken at once would be two owners of one folder.
  Each turn asks for a number one higher than the last,
  and the loop ends with an error once the number reaches the largest one a run can be counted to,
  so it reads nothing it has not just tried and nothing keeps it asking forever.
- Making a worktree (`src/worktree-making.ts`):
  one `worktree add`,
  then the clearing of what this start own making left,
  then the add after it.
  Whether the second add is needed turns on how the first ended,
  which is known only after it has ended.
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
