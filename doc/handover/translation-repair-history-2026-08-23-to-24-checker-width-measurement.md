# Translation repair history 2026-08-23 to 2026-08-24: checker width measured and the wide arm deleted

The switch that relaxed checker independence and the floor of three checkers,
the instrument that reads both arms off one run,
the four wide runs that answered that width changes nothing,
the computable bound on the measurement's power,
the interim readings and one statistic that moved,
and the corrections to a rebuild-safety reason and a suite-counting rule.

Part of [Translation repair history: 2026-08-23 to 2026-08-24](translation-repair-history-2026-08-23-to-24.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## Checker width: measured over 231 rounds, and the wide arm is deleted (`#188`)

The owner revised the "all producing roles to 4" ruling on 2026-08-23.
Producing roles STAY AT
THREE,
because `#186` measured that exact comparison and came back null and because four writers
leave two checkers,
which cannot decide anything.
Checker width was ruled separately and NOT by
opinion:
relax the independence assertion behind a switch,
run the arms,
compare per-issue
resolution,
ship the winner and delete the loser.

### What landed

`assertCheckerIndependence` takes `selfCertificationPermitted`.
Set,
the whole roster may check and
`tallyResolutionChecks` halves a checker per issue whose shipped text it wrote.
Unset,
which is the
default,
nothing changed.
Two refusals ignore the switch entirely:
a repeated checker id,
because it
makes `gatherStageVoices` count toward quorum what `runCheckerStage` collapses to one ballot,
and a
roster below three,
which is the new `assertCheckerQuorumReachable`.

The floor is `MINIMUM_CHECKER_COUNT = 3`,
derived rather than chosen:
resolution needs
`fixed > (notFixed + worse)`,
so at two checkers a disagreement returns nothing and only a unanimous
pair decides.
It is scale-free under the discount,
since halving both sides of that comparison
leaves it unchanged.

The floor found two test fixtures modelling rosters production would refuse:
`repair-translation`
ran two checkers and `repair-window-threading` ran one.
Both widened to three,
every checker still
clear of that fixture's editors.

### The instrument: one run, both arms

`runCheckerStage` summed three verdicts into a tally and discarded the verdicts.
It now publishes
one line per ballot naming checker,
issue,
verdict,
and whether that checker wrote the text under
review.
Ids and verdicts only,
pinned by a test,
because it prints on every run over an archive
nobody licensed us to copy.

THAT LINE IS WHAT MAKES THE COMPARISON PAIRED.
`buildResolutionMessages` does not read the roster,
so the sheet a narrow run sends is the sheet a wide run sends,
and every ballot a narrow arm would
have bought is present in a wide arm's log.
Both tallies are therefore read off ONE run:

-   narrow,
    over the three disjoint checkers at full weight
-   wide,
    over all six with the author marker choosing each weight

The arms differ by exactly the three ballots under test,
so run-to-run variation,
which `#186` found
large enough to swamp its signal,
cannot reach this comparison at all.

Rounds are recovered per issue rather than by log contiguity:
the k-th ballot a given model casts on
a given issue belongs to the k-th time that issue was checked.
Issue ids are `adjudicated/<hex>` and
unique per document,
measured 25 distinct over 25 rows on `lintong`,
and the refinement recheck asks
about the same issue a second time,
which is why the index is needed.

The analyser is `~/temp/agent/checker-width-tally.mjs`,
positive-controlled before use on a
synthetic log where one round must flip (narrow 2 fixed against 1 not-fixed resolves;
three authoring
writers at half weight take it to 2 against 2.5,
undecided) and one must not.

CORRECTED THE SAME DAY,
AND THE CORRECTION IS THE LESSON.
Reading ballots out of a log was a
workaround for the artifact not carrying them,
and the right fix was to make the artifact carry them.
`RepairIssueRecord.checkerReading` now holds every ballot,
the author flag that sets each weight,
the
seated roster and the tally,
so `~/temp/agent/checker-width-from-artifacts.mjs` reads settled runs
directly and re-tallies at either width with no log parsing and no re-running of the stage.
That
analyser is positive-controlled the same way,
and additionally recomputes each stored tally from its
own ballots so a weighting that changed under a settled artifact becomes visible rather than silent.

The log lines stay,
for two reasons that are not the artifact's job:
an operator watching a live pass
has the log and not the artifact,
and a run that aborts before settling leaves nothing else.
They are
derived from the readings,
so the two cannot drift.

### ANSWERED 2026-08-24: width changes nothing, and the wide arm is deleted

Four wide runs answered it.
Read off their `checker-ballot` lines by `~/temp/agent/checker-width-tally.mjs`,
one arm counting only the disjoint three and the other counting all six with a writer at half weight:

```text
run                                     rounds  ballots  narrow split  six disagreed  flips
checker-width-wide-batch-2026-08-24         89      515             1              9      0
checker-width-wide-batch2-2026-08-24        92      545             3              5      0
checker-width-wide-2026-08-24               21      126             0              0      0
checker-reading-vub-2026-08-24              29      174             0              0      0
TOTAL                                      231     1360             4             14      0
```

THE NULL IS ABOUT WIDTH RATHER THAN ABOUT SILENCE.
The six disagreed on 14 rounds,
and on 10 of those a writer answered something no narrow checker said,
so the three extra ballots carried real information that the arithmetic then absorbed.
A checker judging text it helped write counts half,
so three writers bring 1.5 against a unanimous 3.0 and cannot move it.
They can only reach a split three,
which happened on 4 rounds,
and on none of those did all three writers dissent together.

POSITIVE-CONTROLLED BEFORE THE NULL WAS BELIEVED,
per QPC.
`~/temp/agent/control-width-log/cat-control.log` is a synthetic ballot log of two rounds:
one where the narrow three split 2-1 and three writers dissent,
which must flip resolved to undecided at 2 against 2.5,
and one where the narrow three are unanimous and the same three writers dissent,
which must not flip at 3.0 against 1.5.
The analyser reports `FLIPS: 1 of 2` and names the right one.

CONSEQUENCE IN THE CODE:
`TRANSLATION_REPAIR_WIDE_CHECKERS` is gone,
and `RUN_MODELS.checkerModelIds` is the disjoint three unconditionally.
`checkerSelfCertificationPermitted` stays on `RepairModels` as the guard that refuses a writer`s
self-certification,
and production leaves it unset,
so the half-weight discount has nothing to apply to.

ONE CAVEAT FOR A READER RE-DOING THIS FROM ARTIFACTS RATHER THAN LOGS.
Three of those four runs were launched from a build predating `RepairIssueRecord.checkerReading`,
so `~/temp/agent/checker-width-from-artifacts.mjs` sees 0 rounds in them and 29 in the fourth.
The log analyser is what carried this measurement.
A rebuild does not change a running pass,
so a pass launched before a field exists never writes it,
however long it runs.

### The power of this measurement is bounded, and the bound is computable

Measured over 12 settled entries carrying issues,
514 accepted issues in all:

```text
shipped      / resolved=true    303
shipped      / resolved=false     7
no-region    / resolved=false   203
not-selected / resolved=false     1
```

The headline "58.9 percent resolved" is a dilution,
not a finding:
`no-region` issues never reach a
checker,
because no repair region served them.
Among the 310 issues the checkers actually ruled on,
303 resolved,
which is 97.7 percent and matches the 98.1 percent `checker-sensitivity.ts` recorded.

WHICH ROUNDS CAN FLIP IS DECIDABLE IN ADVANCE,
and the answer is narrow:
only rounds where the three
disjoint checkers SPLIT.
At three to nil the wide arm adds at most 1.5 of opposing weight against
3.0,
so the verdict holds.
At two to one the narrow arm resolves on 2 against 1,
and three authoring
writers answering the other way take it to 2 against 2.5,
which does not resolve.
So the count of
narrow-split rounds is a ceiling on the flip count,
and the analyser prints it first for that reason.

A NULL FLIP COUNT IS THEREFORE ONLY MEANINGFUL BESIDE THAT CEILING.
If the narrow three split on
almost nothing,
zero flips says the checkers agree,
not that width is irrelevant,
and the honest
report is that the experiment could not have answered the question at this sample size.
The 7 of 310
unresolved rate suggests the ceiling will be low,
so this is the likely outcome and is written down
BEFORE the data arrives rather than after.

### Two working hazards checked while the wide runs were in flight

REBUILDING `dist/` DURING A LIVE PASS IS SAFE,
and it was worth checking rather than assuming:
this
session rebuilt the bundle roughly a dozen times while two multi-hour verification passes were
executing out of it.
Every `import(` in `src/` is a TSDoc `{@link import('...')}` reference,
so the
package has no runtime dynamic import and a running pass holds every module it will ever need from
startup.
A rebuild that renames hashed chunks therefore cannot reach it.

`mise run //package/module/translation-repair:test:unit` (moved on 2026-10-18 to
`//package/translation-repair/<package>:test:unit`) DOES NOT BUILD.
The package declares two
tasks with that name,
and the bare one runs the suite against whatever `dist/` already holds.
A
source change followed by `test:unit` alone reports on the previous build,
which looks exactly like a
change that had no effect.
Run `build` first,
always.
This session read "the new log line never
appears" off a stale bundle before a positive control on a neighbouring line from the same function
exposed it.

### Interim reading, and one interim statistic that already moved

Across three wide passes,
234 ballots over 39 checker rounds,
every round hearing all six:

```text
rounds carrying at least one self-vote                     36 of 39
rounds where the narrow three split                         2 of 39   <- ceiling on flips
rounds where any of the six dissented                       4 of 39
rounds where a writer said something no narrow checker did  2 of 39
FLIPS                                                       0 of 39
```

CORRECTING AN EARLIER READING OF THIS SAME RUN.
At 24 rounds the last line read 0 of 24,
and that
was written up as the finding that carried the weight:
the added voices were not contributing a
distinct opinion at all.
Fifteen rounds later it reads 2,
so that claim was an artefact of sample
size and is withdrawn.
This is recorded rather than quietly overwritten because it is the exact
failure the ceiling discipline exists to catch,
and it caught it here on the strength of the numbers
rather than on anybody noticing.

WHAT THE NUMBERS SUPPORT SO FAR,
stated no more strongly than they earn.
The three added voices do
occasionally say something the disjoint three did not,
about one round in twenty.
In none of those
rounds did the verdict move,
and the arithmetic says why:
those rounds had a UNANIMOUS narrow panel,
so the wide arm reads 3.0 against at most 1.5 and cannot resolve differently.
The two rounds that
could have flipped,
the ones where the narrow three split,
are not the two where a writer dissented
uniquely.

So the honest position at 39 rounds is that no flip has been observed and only two rounds have ever
been able to produce one.
That is not yet an answer about width;
it is a measurement of how rarely
this checker panel splits at all,
which is 2 of 39.

The discount is genuinely engaged rather than idle:
36 of 39 rounds carry at least one self-vote,
so
this measures self-certification and not merely panel size.
That was the failure mode written down
in advance,
and it did not happen.

### Correcting the rebuild-safety reason, and a suite-counting rule that was wrong

THE REBUILD CONCLUSION HOLDS,
THE REASON GIVEN FOR IT DOES NOT.
The claim above says the package has
no runtime dynamic import because every `import(` in `src/` is a TSDoc reference.
The built bundle
was then searched directly,
which is the artifact a running pass actually executes,
and it carries
two:
`await import('node:fs/promises')` and `await import('node:path')`,
both inside
`run-config-*.mjs`.
Both resolve to Node builtins rather than to hashed chunks,
so a rebuild still
cannot reach a running pass,
but the safety rests on what those two imports name and not on their
absence.
The check to run is:

```sh
# from package/module/translation-repair
grep -oE '(^|[^.[:alnum:]_$])import\(' dist/final/node/*.mjs | grep -v 'import.meta' | wc -l
```

`package/module/translation-repair` was split on 2026-10-18 into the packages under `package/translation-repair`.

Anything that is not a `node:` specifier would break the guarantee.

`] PASS ` LINE COUNTS DO NOT DETECT FAILURE,
which contradicts how this document has been counting
suites.
The runner emits one `] PASS ` line per describe listing the cases that passed,
and separate
`] FAIL ` lines for the ones that did not,
so a describe with one failing case out of three emits a
PASS line AND a FAIL line.
A guard-removal run in this session held at 568 PASS while two cases were
failing,
and only the exit code and the FAIL count showed it.
Count all three:

```sh
grep -c '] PASS ' suite.log; grep -c '] FAIL ' suite.log; echo "exit=$?"
```

Exit code decides.
The clean baseline for this package is 568 PASS,
0 FAIL,
exit 0.
