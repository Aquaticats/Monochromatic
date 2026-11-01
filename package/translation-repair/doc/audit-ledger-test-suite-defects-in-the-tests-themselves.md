# Audit ledger: defects in the tests themselves

The test-suite findings T1 to T7 and T9,
covering an assertion that never runs,
vacuous checks,
tests pinning wrong or retired behaviour,
corpus text and real personal data in fixtures,
flaky timing,
names claiming more than they check,
magic numbers,
and the log every test run writes into `node_modules/.monochromatic/`.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## T1: an assertion that never runs

Status:
fixed in `9603ff7a8`;
shown to fail with chunk inheritance disabled (3 of 4 slices governed).
`document-preparation.unit.test.ts` "inherits the line-structure verdict from the enclosing CHUNK":
its loop runs zero times,
because the verse fixture is too short to subdivide.

## T2: vacuous checks and catch-only asserts

Status:
fixed in `75900e601` (oxlint does not flag an unused `using` binding),
`5955c9d8e` and `38f59b737`.
32 `expect(<using binding>).not.toBe(undefined)` on disposables that are always objects
(`writer-grace-override`,
`corpus-run/slice-overlap`,
`grace-override`,
`corpus-run/pass-entry`,
`corpus-run/artifact-pool-names`);
`assembly-content-survival.unit.test.ts` has no boundary case for its six-letter floor or its two-use cap;
`prompt-uniqueness-client.unit.test.ts` asserts only inside `catch`.

## T3: tests pinning wrong or retired behaviour

Status:
fixed in `36c3576e0` and `07b88949e`,
with S6 (`e7e3f9c17`) for the silent-original test;
checked on 2026-09-27:
no test named here still claims a stopped entry or the owner's whole handle rule,
and the behaviour gap the restore test pinned is A17,
fixed at the floors in `7ec9669bd`.
The two standing tests are renamed to what they assert;
the restore test no longer claims the owner's whole rule,
and the behaviour gap it pinned is A17;
`dropped-covers-the-page.unit.test.ts` moves with S6.
`dropped-covers-the-page.unit.test.ts` pins the silent-original wording of S6;
`corpus-run/contributor-name-restore.unit.test.ts` pins handle restorations
that drop the literal gloss the house rule requires,
under a name citing the owner's "with the literal translation in parentheses";
`consolidate-gate-wire.unit.test.ts`'s class fifty-six name still says a kept standing would stop the entry;
`consolidate-standing-verdict.unit.test.ts` names "still stops the entry" and never asserts it.

## T4: corpus text and real personal data in about 45 test files

Status:
fixtures fixed in `98054d72b` (58 test files).
A rescan on 2026-09-27 (`~/temp/agent/audit-tests/scan-corpus.mjs`,
previous hits kept as `*-before.json`)
left shared markup,
public facts,
glossary entries that are dictionary terms by design,
and corpus quotes in comments and test names that cite an incident
(for example `name-gloss-restore.unit.test.ts:3`,
`owner-cull.unit.test.ts:4`);
those wait for the owner's sanitization pass after the project.
A real birth date and hometown,
a suicide-site sentence,
method sentences,
self-harm scars,
real names,
handles and entry ids,
and verbatim or near-verbatim corpus lines,
many under a header claiming "no corpus content appears here".
`package.json` `files` includes `src`,
so the tests would ship with the package.
The owner said sanitization of the repository comes after the project;
the fixture rule (cat-themed invention) stands,
so these are fixed as fixtures.

## T5: flaky timing

Status:
fixed;
the finding text below the measurements is as first recorded.
The contest case's delayed seats never answer,
on an abortable timer:
5 of 5 pass at 0.2 CPU
(`~/temp/agent/audit-tests/t5-fix-summary.log`).
The naturalness-review sibling measured 5 of 5 at 0.2 CPU before any change and is left as it is:
its fake answers in microtasks,
so the race read into it does not occur.
Real sleeps removed:
the `stage-quorum` stall (30.49 s to 0.51 s),
the contest driver's production backoff (27.4 s to 0.55 s),
and the client's malformed-body case (11.98 s to 0.76 s).
`consolidate-driver` ran 10.24 s,
0.77 s after the fix:
its two positive controls ("reaches the roster" cases) passed only because
five production backoffs outlasted `driveWith`'s 5 s abort,
so they asserted a timeout,
not a roster call;
they now count transport calls under a zero-retry client and assert the observed exits.
A sweep of all 729 test files for timers of 500 ms or more
(`~/temp/agent/audit-repair/long-timer-sweep.json`) found three more real waits:
the `benchmark` fake's two 2 s waits,
now a fake clock the benchmark reads (`544169f98`,
2.36 s to 0.59 s),
the `stage-round-refill` slow seat,
now abortable (`9441af9de`,
1.94 s to 0.46 s),
and `transient-retry`'s backoff,
which is the behaviour under test and stays.
Long `settleWithin` and `armCallDeadline` timers are armed and cleared and cost no wall time.
The shapes read into the rest were measured,
8 runs each at 0.2 CPU
(`~/temp/agent/audit-tests/t5-rest-summary.log`,
`t5-peak-summary.log`):
`synthetic-client`,
`hyper-client`,
`provider-router`,
`transient-retry`,
`budget-hold-wait`,
`refine-phase` and `lane-contest-driver` passed 8 of 8 and are left as they are
(a floor on a wait only grows under load;
the `transient-retry` ceiling is a daily refusal's wait).
`stage-round` failed 1 of 8:
time to the first answer (398 ms) passed the 250 ms grace it was compared with;
it is now anchored on when the first voice really answered (`c10d62663`),
16 of 16 after.
`consolidate-driver` failed 1 of 8 on the T6 order check this audit added,
ordered by 20 ms against 5 ms sleeps;
both overlap cases now order by a gate (`d76d2424a`,
M11),
16 of 16 each at 0.2 CPU after (`t5-gate-summary.log`).
`lane-contest-stage.unit.test.ts` "RECORDS RAW HALF-QUORUM BALLOTS" fails 2 of 5 at 0.2 CPU
(positive control:
the `podman run` in `~/temp/agent/audit-tests/run-container.mjs`);
its sibling case,
the naturalness-review grace case,
settle-by-timer checks in `synthetic-client`,
`hyper-client` and `provider-router`,
and the driver trio's `peak` checks are the same shape by reading;
wall-clock floors in `transient-retry`,
`budget-hold-wait`,
`stage-round` and the benchmark have no slack.
Real sleeps:
`stage-quorum` waits 30 s ignoring the abort signal;
`synthetic-client` and `lane-contest-driver` run production backoff.

Recurred in `request-pace.unit.test.ts`,
found 2026-10-01 (UTC) from the coverage census:
the default sleeper's resumption (`request-pace.ts:128`)
ran in `census-zkJwfY` and not in `census-dnr81C`,
the source and its test unchanged between them.
The case "SLEEPS ON THE REAL CLOCK WITH THE DEFAULT SLEEPER" took two places on a 20 ms window on the real clock.
A take works out its wait before it first awaits,
so a run stalled past the window between the two takes found room,
never slept,
and passed its one-sided check all the same:
its name claimed a sleep it did not make sure of (the T6 shape),
and what it covered rode on load.
The pacer's clock is now held still until the second take has started its sleep on the default sleeper,
then released,
so the sleep is the whole window on every run;
the default clock stays covered by the B78 case on a stepped wall clock.

Recurred in `absolute-naturalness-review-stage.unit.test.ts`,
found 2026-10-03 (UTC) in the trial's suite start under the whole suite's load:
the "STARTS GRACE AT HALF instead of requiring delayed final seat" case
ordered its delayed seat's rejection 30 ms against its peers' microtask answers,
and load starved the peers' quorum to the seat's arrival,
so the reply raced the 0 ms grace close
and the case failed 2 of 5 full-suite runs
(a correction naming itself:
the Status's claim that the naturalness-review sibling's "race read into it does not occur" rested on 5 of 5 at 0.2 CPU,
which never probes the whole suite's load).
The seat's answer is now held on a gate the case releases after the review settled
(the M11 shape),
so the straggler arrives after the close on every run
and its words still flow past the no-leak check,
which a seat that never answers would have left vacuous;
"KEEPS REJECTION that arrives inside bounded post-quorum grace" keeps its 30 ms sleeper against a 100 ms grace,
safe in both orderings,
as the control.

## T6: names claiming more than they check

Status:
fixed in `292939dab`,
`6baf52dbe`,
`bc80c8c14` and `6d7d83c1b`,
whose order checks were timing-ordered and are made deterministic in `d76d2424a` (M11);
the "trio" is two files (refine phase,
consolidation driver) whose names claim the second call answers first.
`bedrock-catalog` "EVERY ROUTE" checks one route;
`synthetic-catalog` pins a literal price;
`block-pairing-protocol` compares a wrapper to its own builder;
the driver trio never asserts the second call answers first.

## T7: magic numbers

Status:
fixed in `c0ce1a2c5`,
`7a4d521ed`,
`962770574`,
`04a9f499a` and `e2c887ee7`;
`roster-reach` under E9.
`roster-reach`,
`request-pace`,
`synthetic-catalog`,
`deepseek-v41-admission`,
`synthetic-client`,
`repair-slice-key`,
`anthropic-request` (a cap that should be computed from the exported caps),
and vote weights not derived from exported constants in `candidate-select` and `candidate-select-decision`.
The rule applied:
a measured provider fact or owner limit is pinned once,
where it is owned
(`hyper-catalog`,
the GLM wire facts,
the account limit in `request-pace`),
with its source named;
every other test derives from the export.
Kept as pins,
deliberately:
`repair-slice-key`'s key literals,
which exist to fail when a key changes,
and `request-pace`'s account limit.
`completion-cap` gained the only check that a pooled card name resolves to one shared figure;
before,
the DeepSeek test's 13,082 literal was its only cover.

## T9: every test run writes a log into `node_modules/.monochromatic/`

Status:
resolved for issue #576 in `module-logger` `0.4.5`
 (decision record `doc/decision/module-logger-log-accumulation.md`):
 `MONOCHROMATIC_LOG_FILE=off` disables the file sink entirely,
 and redirecting `MONOCHROMATIC_LOG_DIRECTORY` to a per-run scratch directory
 keeps test runs hermetic even without the switch.
The issue 573 verification-timeout line recorded in this entry stays open.
Owned by `module-logger` (issue #576,
measurements added 2026-09-27).
1,220,455 files there on the measurement date;
the tests were not hermetic then.
Measured on 2026-09-27:
most of that count is the deliberate `translation-repair-runs*` run directories;
the logs themselves are 34,919 top-level `*.log.jsonl` files (846 MB with the runs),
about 22 added per unit suite run,
and the logger's file sink (`package/logger/core/src/sink/file.ts`) reads no switch that could turn it off in tests.

Added on 2026-10-10 (UTC):
the logger's sink verification,
bounded at 5,000 ms,
writes a line of its own into a process's stderr when it times out,
`logger internal error: sink verification failed for entry 3: Timed out after 5000ms: sink 3 verify`,
and in the first run of the whole-suite census at `116bf6f4bba63f7375181dfddfef4e251eb10d34`
that line failed nine as-built cases,
each pinning its child's stderr whole
(the T8 passage "The runner and git cluster of the final census").
It is the logger's defect,
open as GitHub issue 573 of `module-logger`,
on which the lead put the evidence.
The package does not filter the line in tests:
a case it reaches fails,
is reported,
and is run again (M145).
