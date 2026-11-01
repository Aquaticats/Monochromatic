# Package audit: findings of the calibrate and probes slices

The findings of the calibrate slice and of the probes slice,
the instruments that measure the pipeline.

Part of [Translation-repair package audit](translation-repair-package-audit.md),
which lists every file of this record.

## calibrate-1, MAJOR, verified: the "nothing above warn" premise of `#235`'s silence half is wrong

`src/corpus-run/editor-calibrate.ts:359-367` ends the calibration with `coverageGapLines`,
and `src/producer-silence.ts:222` renders
`WROTE NOTHING AT ALL: <models>. No candidate of theirs reached any slate, so the table covers N of M seats`.
The arm A log carries that line twice (EDITOR and REFINER),
each ending `covers 5 of 10 seats`.
So the handover's "said nothing at any level above warn" and the troubleshooting document's
"nothing aggregates it" were wrong,
and both are corrected in this commit.
What is actually missing is narrower and still real:
the line cannot tell a budget refusal from a timeout from a seat failing every call with HTTP 400
(its own wording says so),
it carries no per-seat call or failure count,
it is stdout prose under exit 0,
and it exists only in the two calibration CLIs;
the pass's per-entry `TALLY` line (`src/corpus-run/settled-tally.ts`) has no seat field,
and the other 36 CLIs have nothing of the kind.
Consequence for the `#235` design:
the process-scoped seat tally reported by `reportingRefusals`
stays,
because it is the only universal boundary,
and it must print asked,
usable,
and failure
counts per dark seat with the failure class;
`coverageGapLines` stays as the standing-coverage
measure it already is,
and its sentence gains a pointer at the seat report.

FIXED 2026-08-26:
the seat report landed with A-1 (`8b289c3ab`:
`SEAT <model> asked= usable= unusable= threw=`
per seat and a `SEATS DARK:` line at the end of every command),
and the pointer landed in `8bffaba9b`:
the
`WROTE NOTHING AT ALL` sentence now says the `SEAT` lines at the end of the command carry how often each seat was
asked and how many answers were usable,
and that the run log names the failure.
Guard:
the pointer removed fails
`POINTS AT THE SEAT LINES from the silent-seat sentence`;
restored,
passes.
Found unmarked when the closing tally
was measured;
the pointer was the only half of the consequence still owed.

SUPERSEDED IN PART 2026-08-26 (`#263`,
landed in source `d8a040edf` and `04f1ebafd`,
unbuilt while the arms hold
`dist`):
the sentence itself was wrong for a seat that answered.
Arm A's REFINER table called `minimax-m3` silent
beside `SEAT minimax-m3 asked=31 usable=31`,
and arm A2 did it for two seats,
because the refiner's `produced` list
was built from slates and a rewriter that leaves a paragraph as it stands never reaches one.
The refine stage now
carries out who it heard,
the coverage split takes `answered` as a fourth input,
and the line reads `ANSWERED AND
WAS NEVER SLATED` for that seat;
a seat that does not record who answered (editor,
translate;
`#266`) says so in
its silent line instead of calling the unknown silent.
The pointer at the `SEAT` lines stays on every wording.

## calibrate-2, MAJOR, verified: the pass prints any error's message into its stdout summary

`src/corpus-run/pass-entry.ts:522-529` and `:641-648`:
`Error.isError(error) ? error.message.slice(0, ERROR_MESSAGE_CAP) : String(error)` lands in
`TALLY <id> status=ERROR ... error=<message>` and `CLEANUP <id> ... error=<message>`.
That bypasses the `#227` rule that only `messageNamesOnly` classes may repeat their message:
`SyntheticHttpError` (renamed `ProviderHttpError` on 2026-10-19)
(`src/completion-shape.ts`) embeds an excerpt of the provider's raw body,
and any unmarked parser class may quote a passage.
The `TALLY` line is the one operators grep and paste.
Fix:
`error=${refusalText({ error })}` at both sites,
with the full text kept in the run log
through the tagged logger where diagnosis needs it.
Tracked as `#237`.

FIXED 2026-08-26 in `c19d203c6` (lint tidy in `82db8fb4e`):
the TALLY and CLEANUP lines,
the sentinel probe's
PROBE line,
and the rendering audit's REFUSED line print `refusalText`,
a marked class in its own words and
anything else by name;
the CLEANUP guard fails with the printer restored (2 failing lines),
passes restored.

## calibrate-3, MINOR, verified: an operator refusal thrown as a bare `Error`

`src/corpus-run/editor-width-control.ts:186-190` throws `new Error('editor width control refused: ...')`,
so the boundary prints `refused by Error` plus a fault stack and exits 5;
the sibling site `editor-width-probe.ts:139` already uses `StatedRefusalError`.

FIXED 2026-08-26 in `acfc7ad22`:
the refusal is a `StatedRefusalError`,
so the boundary prints it in one line
and exits 6,
and the usable suite asserts the class.
Guard:
with the site throwing a bare `Error` again,
the case
`REFUSES a draw whose every slice holds one sentence` fails;
restored,
passes.

## calibrate-4, MINOR, verified: `MislabelledArtifactError` writes remedies nobody sees

`src/corpus-run/pass-schema-guard.ts:229-277` builds an operator remedy ("Ways forward")
into a class that is correctly unmarked,
because `reason` carries a parser's text
(`caughtValueText(error)` at `:378`),
so the boundary prints only `refused by MislabelledArtifactError`.
Fix:
a marked class naming entry,
generation,
and the ways forward;
the reader's reason goes
to the log through `refusalText`.

FIXED 2026-08-26 in `acfc7ad22`:
`MislabelledArtifactError` carries `messageNamesOnly` and names the entry,
the generation and the ways forward;
the reader's own objection is logged at warn through `refusalText` at the
throw site,
and the inventory in `message-names-only.unit.test.ts` lists the class as marked.
Guard:
with the marker removed,
the inventory cases `KEEPS exactly the classes the inventory records` and
`KEEPS a reason for every class that writes its sentence and stays unmarked` fail;
restored,
passes.

## calibrate-5, MINOR, verified: `editor-calibrate` builds a client per slice

`src/corpus-run/editor-calibrate.ts:211`:
`createRunClient()` inside `runOne`,
so budget cooldowns,
meter caches,
and per-model limiters restart on every slice,
and a provider held out on slice N is re-asked immediately on slice N+1.
`editor-width-probe.ts:94` and `window-trial-probe.ts:326` build once in `main`.
This is also why the arm A log carries four `is not set` warnings.

FIXED 2026-08-26 in `acfc7ad22`:
`main` builds the client once,
before the slice loop,
and hands it to
`runOne`;
the only `createRunClient()` call left in the file is that one.
No test observes client construction,
so this one is verified by reading the diff rather than by a guard.

## calibrate-6, MINOR, verified: the refusal streak resets on slices the ledger already held

`src/corpus-run/window-trial-probe.ts:410-416`:
`bought.refusedInARow = 0` runs before
`if (rows.length === 0) continue`,
and a slice already complete in the ledger returns empty rows,
so on a resumed run a fault refusing every NEW slice never reaches `REFUSALS_BEFORE_STOPPING`,
and each refusal still buys a producer slate.
Fix:
reset only when `rows.length > 0`.

FIXED 2026-08-26 in `acfc7ad22`:
the streak is a function,
`streakAfter` in `window-trial-protocol.ts`,
and a slice the ledger already held leaves it where it was;
the loop reads it on both paths.
Guard:
with the held slice resetting the streak again,
the cases
`REFUSES to reset the streak on a slice the ledger already held` and
`reaches the stop across a resumed run whose held slices interleave with refusals of every new one` fail;
restored,
passes.

## calibrate-7, MINOR, verified: the control's sentence cut needs a trailing space

`src/corpus-run/editor-width-control.ts:38-43`:
`TERMINATORS = ['。', '. ', '! ', '? ']`,
so a passage with one sentence per line reads as one sentence and leaves the positive control.

FIXED 2026-08-26 in `acfc7ad22`:
`TERMINATORS` gains the three stops that end their line.
Guard:
with them removed,
the case `REMOVES THE FIRST SENTENCE at a stop that ends its line` fails;
restored,
passes.

## calibrate-8, MINOR, verified: `verify-published` overclaims on unweighable entries

`src/corpus-run/verify-published.ts:260` returns `missing.length === 0` for an entry whose
artifact predates the stored archive text,
and the closing line at `:378-381` says every page
carries its wordings "at the length it implies" although no length was checked for them.
The per-entry line does say `UNWEIGHED`;
the summary does not.

FIXED 2026-08-26 in `acfc7ad22`:
`reportEntry` answers `agreed-weighed`,
`agreed-unweighed` or `disagreed`,
and the closing line reads
`N of M pages carry every wording their artifact promised; K of those at the length it implies, U UNWEIGHED because
the artifact predates the stored archive text`.
Verified at the boundary:
over the 2026-08-17 runs directory,
which has no published tree,
the line reads
`0 of 0 ... 0 of those ... 0 UNWEIGHED` and the command exits 1;
over an empty directory the command takes its `NOTHING VERIFIED` exit 2 before the line.
No unit guard:
`verify-published.ts` has no suite and the counting sits inside `main`;
recorded as a gap,
not claimed.

## calibrate-9, MINOR, verified: `PublishedPageDisagreesError` is unmarked

`src/corpus-run/published-page-check.ts:499-507`:
the class documents that it names slices
and counts and quotes nothing,
and carries no `messageNamesOnly`,
so at a boundary it is muted.

FIXED 2026-08-26 in `acfc7ad22`:
the class moved to `published-page-disagreement.ts`,
takes a typed
`PageDisagreement` (wordings missing,
or weight off) and writes its own sentence from indices and counts,
which is
what lets it carry the marker:
the inventory refuses the marker to a class that forwards a caller's sentence,
so the
old shape could never have been marked.
Guard:
with the marker removed the same two inventory cases as calibrate-4 fail;
restored,
passes.

## calibrate-10, MINOR, verified: four modules in the slice are referenced by no test

`pass-settled.ts` (whose module note records a past silent defect:
a directory named `<id>.json`
counted as settled),
`editor-width-arm.ts`,
`pass-schema-census.ts`,
`window-trial-probe.ts`:
`rg --files-with-matches` over `*.test.ts` finds 0 references for each.

FIXED 2026-08-26 in `acfc7ad22`:
suites for `pass-settled.ts` (a directory or a symlink named like an
artifact is not counted,
and the count agrees with the id set),
`pass-schema-census.ts` (every classification,
in name
order,
skipping the directory and the suffix-less file),
`editor-width-arm.ts` (a blank arm with no producer,
and a
composite arm naming both editors as producers) and `window-trial-protocol.ts` (the digest and the streak,
lifted out
of the command so they reach the bundle),
plus a boundary case in `window-trial-probe.unit.test.ts` that runs the built
command with every provider key withheld and expects the stated refusal and exit 6.
The census suite found a real defect on the way:
a file holding a JSON array passed `isJsonRecord`,
carried no version
field,
and was filed as an unversioned generation,
a sound result to keep;
it is `malformed` now.
Guards:
the listing filter answering true for every entry fails three pass-settled cases;
the array check removed
fails `classifies JSON that is not a record as malformed too`;
the blank rule removed fails
`reads an arm whose editors proposed nothing as BLANK with no producer`;
each restored,
passes.
The scheduler's listing is renamed `artifactBackedIds`:
the bundle already exported a `settledEntryIds` that lists
the published tree,
and the barrel refused the second.

## calibrate-11, note: the window-trial ledger stores model-produced English

`src/corpus-run/window-trial-ledger.ts:83` writes `winnerText` to `<runs>/window-trial/arms.jsonl`.
Outside the repository by design;
listed so the sanitization step knows the file exists.

## probes-1, BLOCKER, verified: the sensitivity instrument labels an arm `prior=shown` that shows nothing

`src/corpus-run/probe-sensitivity.ts:92-103` calls `runIntroducedDefectProbe` with `issues` and no
`disclosure`;
the probe's default is `'withheld'` (`src/introduced-defect-probe.ts:155`),
and the sheet
renders the "PRE-EXISTING DEFECTS THIS EDIT TARGETED" block only under `disclosure === 'rendered'`
(`src/introduced-defect-wire.ts:438`),
whose own default is `'rendered'` (`:365`).
So the `condition: 'shown'` arm (`:172`) and the `'absent'` arm (`:168`) send byte-identical prompts
and differ only in what the deterministic screen dismisses,
while the closing NOTE (`:243-247`)
tells the reader a stage that "goes quiet with prior=shown is one the production prompt silences".
Production withholds explicitly (`src/repair-chunk.ts:389`),
so the counts describe production;
the labels,
the comments,
and the NOTE do not,
and the instrument cannot detect a regression in the
rendered prompt at all.
No recorded decision rests on a post-flip run;
any rerun would mislead.
Tracked as `#247`.

FIXED 2026-08-26 in `bf7d6afbd`.
Production's disclosure is one exported constant (`PRODUCTION_PRIOR_ISSUE_DISCLOSURE`) read by both stages
and both instruments;
the sensitivity arms are a data table (`probe-sensitivity-arms.ts`) whose unit test
holds every arm's printed list to the disclosure it sends;
each accuracy region runs under `none`,
`withheld`,
and `rendered`,
so none-against-withheld isolates the screen and withheld-against-rendered the prompt.
Guard shown to fail with the rendered arm sending `withheld` (2 failing lines),
restored,
passed.
Live run of the instrument,
2026-08-26,
exit 0:
eighteen `SENSITIVITY` lines each carrying `list=` and `issue=`,
three checker seats asked eighteen times and usable eighteen of eighteen,
no dark seat.
It measured what the old arms could not:
on `deletion/mislabelled`,
`list=withheld` raises `removal=3`
while `list=rendered` raises `removal=1` with two probers filing the real deletion as `preExisting`,
so the rendered prompt lets a false accepted issue talk two of three probers out of seeing real damage;
`list=none` against `list=withheld` moves nothing on any region,
so the screen dismisses no real claim here.

## probes-2, MAJOR, verified: the relabel instrument's "shown" arm shows nothing either

`src/corpus-run/probe-relabel.ts:78-88` passes no `disclosure`,
and `:150` labels the arm
"Production condition:
the issue list is shown".
Same mechanism;
folded into `#247`.

FIXED 2026-08-26 in `bf7d6afbd`:
three arms per region (the production disclosure,
the other prompt,
no list),
each labelled from the constant it sends.
Its live run waits on `#257`,
since it reads the round-three draw.

## probes-3, MAJOR, verified: the damage sheet tells the grader every item was flagged

`src/corpus-run/probe-verify-sheet.ts:303-304` prints,
unconditionally,
that "an automated reviewer
claims each one introduced a defect",
while `src/corpus-run/damage-sample.ts:333-357` builds the
sheet from `probe-flagged` AND `probe-silent` items with every claim stripped.
`score-verify.ts` reads a Y on a silent item as damage the probe missed,
so the preamble primes the
grader toward Y on exactly the partition scored as probe misses.
The 20-item damage sheet of 2026-08-17 is still ungraded,
so this can still be fixed before it costs a grade.
Tracked as `#248`.

FIXED 2026-08-26 in `b88d6a947`:
`formatVerifySheet` takes a framing;
the verify sheet keeps its wording and the
damage sample writes the blind one,
which says the reviewer flagged some items and stayed silent on others without
saying which.
Guard:
the blind sheet never says "claims each one";
with the framings collapsed the case fails.
The two ungraded damage sheets on disk (`damage-sheet.md`,
`damage-sheet-agent-read.md` in this worktree's runs
directory) had their preamble replaced in place with the blind wording,
items untouched,
so grading them now is safe.

## probes-4, MAJOR, verified: two probes carve with the deterministic pairer the pass no longer uses

`src/corpus-run/judge-fidelity-probe.ts:384` and `src/corpus-run/displacement-probe.ts:198` call the
bare `prepareDocumentPair` under a comment reading "Slices exactly as the lanes would see them",
while the pass carves through `prepareDocumentPairWithRoster` (`src/corpus-run/pass-entry.ts:267`),
whose LLM-assisted pairing took one entry from zero slices to nine (`#189`) and whose deterministic
fallback slid whole documents (`#71`).
Fidelity trials therefore run on pairs the lanes never see,
and the displacement census counts the
deterministic aligner's slides as translation displacement.
Tracked as `#249`.

FIXED 2026-08-26 in `26f346de8`,
`d2c580995` and `459482191`.
The artifact now records the other half of its pairing recipe (`preparation.sectionPairing`,
always written as
`deterministic` or `supplied` with the pairs,
read as `unrecorded` on older files),
`rebuildPreparation` and
`recipeOf` turn both halves back into the inputs `prepareDocumentPair` consumed,
and `settled-carve.ts` lists a
runs directory's settled entries and carves each over the pair at the artifact's own commit.
Both probes walk that population and log `carved from its settled artifact (complete recipe)` or the halves the
deterministic default stood in for;
an entry with no two-lane artifact is skipped by name of its kind.
Guards:
the rebuild reproduces a crossed section pairing and a declined block,
proved by the identity hash with the
bare carve as positive control;
removing either recipe half from the carve or the reader's generation check fails
2 to 3 cases each.
Measured at the boundary:
this worktree's runs directory holds 56 legacy artifacts and both probes carve none;
`TRANSLATION_REPAIR_RUNS_DIR` pointed at the 2026-08-17 runs directory carves 6 of 6 two-lane artifacts,
every one
with both halves defaulted since they predate the recorded pairing,
and one capped fidelity trial ran over them
with all ten seats usable.

## probes-5, MAJOR, verified: the recall scorecard is overwritten in place, non-atomically

`src/corpus-run/recall-benchmark.ts:332-335`:
plain `writeFile` to the fixed name `recall-scorecard.json`.
`src/corpus-run/probe-store.ts` states the repo's own rule for repeated measurements (stamped names,
atomic writes);
a twelve-hour run replaces the previous one with no trace.
Tracked as `#250`.

FIXED 2026-08-26 in `2c9886a55`:
`persistRecallScorecard` (`src/corpus-run/recall-scorecard-store.ts`) writes
`recall-scorecard/<start stamp>-<tip>.json` through `writeFileAtomic`,
the benchmark stamps its start and records
`startedAt` and `finishedAt`,
and the `SCORECARD kept at <path>` line names the file.
Guards:
two runs keep both files with their own rows,
two builds started in the same instant keep two files,
the
name carries no colon;
collapsing the name to the old fixed one or dropping the tip fails all 4 cases.
Plan mode (`-- --plan`) still constructs the client and writes nothing.
The old `recall-scorecard.json` files on disk are left where they are;
nothing reads them programmatically.

## probes-6 to probes-13, MINOR, verified where cited

The derivability sheet's fixed `=====` fence (`src/derivability-wire.ts:22`;
zero collisions in the
pinned corpus,
measured by the reviewer with `git grep --count`);
stale six-model roster prose in
`src/corpus-run/judge-independence.ts:7-11` and the NOTE pasted into verdicts by
`src/corpus-run/score-crosscheck.ts:363-366`;
per-region tallies counting checks rather than probers
(`src/introduced-defect-screen.ts:504-513`);
damage sheet and manifest written with plain
`writeFile` to fixed names (`damage-sample.ts:347-365`,
`probe-verify.ts:210-219`);
the relabel
control stamping `baseIndex: index` (`probe-relabel-control.ts:234`);
`coverage-probe` and
`judge-fidelity-probe` printing rows that carry corpus-derived text to stdout by design
(`coverage-probe.ts:463-467`,
`judge-fidelity-probe.ts:501`);
caught error text persisted into bench
rows (`bench-record.ts:289-290`,
`benchmark.ts:415-424`);
`writeBenchReport` named by no test.
Full text in `~/temp/agent/audit-probes.md`.

FIXED 2026-08-26 in `ebc53f24f` (probes-6 landed with repair-5 in `9135037e8`).
probes-7:
the judge-independence note and the crosscheck's printed NOTE describe the whole-roster panel,
six from one
provider when written and ten across two since 2026-08-24,
with the shape (no seat outside the panel to bar) as the
point;
prose only.
probes-8:
the screen counts one check per prober per region,
the first with a known verdict,
and counts a prober that
cast nothing on the region as uncertain;
a prober whose only check carried an unknown verdict stays dropped,
which the
existing schema-noise case pins.
Guard:
the collapse undone fails `COUNTS a prober once on a region however many checks it cast there`;
restored,
passes.
probes-9:
`sheet-write.ts` lands a sheet and its manifest atomically and refuses,
as a stated refusal naming the path,
to write into a directory already holding either;
`damage-sample` and `probe-verify` write through it.
Guard:
the refusal removed fails `REFUSES to write into a directory already holding the sheet`;
restored,
passes.
probes-10:
the relabel control's slices are stamped from the finished order through `reindexSlicePair` and checked by
`assertSliceIndexing`,
the path `probe-relabel-case.ts` already took.
No guard:
`gatherControlCases` walks the corpus;
the stamping rule's own suite covers the helper.
probes-11:
the README names `coverage-probe` and `judge-fidelity-probe` as the two outputs that quote corpus text,
and
the fidelity probe keeps its rows through `persistProbeRun` before printing them.
No unit guard:
command-level.
probes-12:
bench rows carry `refusalText` instead of `String(error)`,
in both recorders,
and the per-call deadline's
reason is `CallTimeoutError`,
marked,
so a hung call still reads as a timeout in a row.
Guard:
the marker removed fails the two inventory cases;
restored,
passes.
The benchmark suite's hung-call case pins
the timeout wording through the marked message.
probes-13:
`writeBenchReport` has a suite on a disposable runs directory.
Guard:
the file name changed fails `LANDS roster-bench/rows.json under the runs directory`;
restored,
passes.
The fidelity probe's argument reader and its constants moved to `judge-fidelity-args.ts`,
since the probe crossed the
file-length cap with the persistence added.
