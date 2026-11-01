# Audit ledger: holds a test registers and the seat reading sleeps

A hold a test registers for a provider is a wait the seat reading sleeps for real,
and the gates that paid it without a case failing.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B817: a unit test slept the router's five-minute hold for real, and every gate paid it from g138 on

Status:
fixed on 2026-10-22 (UTC),
guard `de2ce3786f`,
fix `52a625199e`,
cache versions none.
A member of the family of B318 and B813,
found by an implementer who asked why the corpus-pass package's `test:unit` had stopped taking about a minute.

What the gates showed:
in every gate up to g137 (at `7c9d5fa35f`) that package's `test:unit` took 43 to 85 s.
From g138 (`3637592bd4`,
which merged the w253 and w254 work) it took 305 to 313 s in every gate,
about 30 gates in all.
Each gate was green,
and nobody compared the time with the gates before it (M288).

The cause:
`4508efa798` (2026-10-21 (UTC),
the red commit of B710) added the case "MARKS IN THE CHUNK LINE THE CHECKERS THAT ARE PADDING"
to `package/translation-repair/corpus-pass/src/corpus-run/pass-reseat.unit.test.ts`.
Its rig seats only NeuralWatt and Bedrock wet,
so a bench of the lanes phase is short of quorum,
and it registered `DRY_HOLD_MS` = 300000 ms for Synthetic,
the hold the router registers on a refusal from a meter that reads dry.
When a bench is short and a hold runs,
`readDrynessPastShortBench` (`run/src/corpus-run/run-seats-read.ts`) waits out the shortest hold once through `waitOutHold`
(`provider/src/budget-hold-wait.ts`),
which sleeps the whole hold in steps of `pollMs`;
`pollMs` only sets how often an abort is noticed.
The case slept 300 s of real time and then passed.
`965a5c9c67` made the case's assertions pass,
and changed nothing about the wait:
the red commit slept the same five minutes before it failed.
The log of `test:unit` on `855963d613` shows the describe block holding the case at 300.3 s;
the other ten cases of the file registered the same figure on views that leave every bench full,
so they waited for nothing.
The case was never a candidate for the real-clock scan:
it holds no timer call,
and a hold reaches the sleep through a client's reader of holds.

The fix:
the file registers one hold,
`SHORT_HOLD_MS` = 40 ms,
in every case,
and none asserts the figure.
The padded case now also reads `waiting 40ms for the shortest hold to end` in its log line,
and the served case reads that its reading waits for nothing,
so the wait the short hold stands for is visible in the case.
After the fix the package's `test:unit` takes 45 s,
and its slowest block is the CPU-bound `pass-consistency-shipped-marks.unit.test.ts`
(36.6 s alone,
41 s of user and system time).

The guard:
the real-clock scan (`source-scan/src/real-clock-bounds.unit.test.ts`) reads,
in tests and fixtures,
a hold of 1000 ms or more that is written onto a rig's `holds`,
or given to a provider in a record that a holds reader returns or a constant named for a hold holds,
and keys it `path#site: holds.<provider> = value` or `path#site: hold <provider>: value`.
Run on `855963d613` it found eleven `holds.synthetic = DRY_HOLD_MS` in `pass-reseat.unit.test.ts`.
Three holds that sleep nothing are listed with why:
`stop-landing-hook.test-fixture.ts` (the stop lands in the dryness read,
before the wait),
`budget-hold-wait.unit.test.ts` (a record that `shortestHold` reads)
and `run-seats-read.unit.test.ts` (a view where every provider is wet).
The scan cannot tell a hold that is waited out from one that is not:
that depends on whether a bench is short,
which no source read decides,
so it flags every hold of a second or more and a listed hold carries the reason it sleeps nothing.
Its sites key `<module>` where a case is an anonymous function,
so it counts holds in a file and does not locate one.
A hold written through a name it does not read,
or a number a parameter hands on,
is out of its reach.

Measured after the fix:
every other package of the family ran its `test:unit` once on this tree,
and none holds a describe block past 10 s but the CPU-bound corpus-pass block;
no case waits out a hold or a retry delay of a second or more but the archive fixture's `HELD_REPLY_MS` = 1000 ms,
which B813 sized on purpose and the real-clock scan lists as `reached`.

Cache versions:
none moves.
The change is in a test file and a test of the scans.

Recurrence:
`mistake-prevention-tests-clocks-and-arrival-order.md`,
"Tests on the real clock":
a hold a test registers for a provider is the short one,
and the seat reading sleeps whatever hold it is told of.
