# Audit ledger: docs and comments that disagreed with the code

Statements in the docs,
comments and examples that the code or the record contradicted.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## D1: the README says the editor roster check still requires disinterested judges

Status:
fixed in `4397d7d2a`.
`README.md:301-303`;
`repair-contract.ts:224-229` says the 2026-08-15 ruling removed that requirement,
and the check refuses only repeats,
no editor,
or judge capacity short of the minimum weight.

## D2: the schema generation the pass writes is misstated in three places

Status:
Markdown fixed in `4397d7d2a` (generations 1 and 2 use the chunk spelling,
3 is mixed);
the two `.ts` comments fixed in `a25f09b2e` on 2026-09-28,
found open while writing the status page's open list:
V14 now says it is what the pass writes,
and the older history says where versions 10 to 14 are recorded.
`doc/configuration.md:326` (moved on 2026-10-18 to `package/translation-repair/doc/configuration.md`),
`:334`,
`:352` say generation 4 and three generations;
`artifact-schema-version.ts:34` says V7;
`corpus-run/artifact-two-lane-contract.ts:36` says V12.
The pass writes V14 (`corpus-run/artifact-two-lane-build.ts:260`,
`corpus-run/pass-schema-guard.ts:426`)
and reads generations 1 to 14.
`artifact-schema-version.ts:42-90` stops its history at version 9.

## D3: pull-request runs are documented through a variable nothing reads

Status:
fixed in `4397d7d2a` (two probe variables were also missing);
two repo docs outside the package still name the variable (D18).
`doc/configuration.md:293-295` (moved on 2026-10-18 to `package/translation-repair/doc/configuration.md`) names
`TRANSLATION_REPAIR_CORPUS_DIR` and an uncommitted fork;
production reads `TRANSLATION_REPAIR_CORPUS_CLONE_DIR` and `TRANSLATION_REPAIR_CORPUS_COMMIT`
(`corpus-run/corpus-pin-override.ts`),
documented nowhere in the package.
`doc/configuration.md:5` (moved on 2026-10-18 to `package/translation-repair/doc/configuration.md`) claims every knob is
listed and omits six.

## D4: the OpenRouter checker substitute is misnamed

Status:
fixed in `4397d7d2a`,
then superseded by `f10de5198`,
which replaced the substitute with the measured order.
`doc/design-commitments.md:177` (moved on 2026-10-18 to `package/translation-repair/doc/design-commitments.md`) says
gemma-4-26b-a4b-it;
`corpus-run/run-seats.ts:113` has `google.gemma-4-e2b`.

## D5: the removed preparation layer is described in the present tense

Status:
fixed in `4397d7d2a`;
the `.ts` comments still describing it,
which this entry counted as two,
ran through eight source files
and were rewritten under D16 (`5a5079549`),
and the two fields kept only for it were removed under D20 (`89a15887a`).
`doc/seats-and-calibration.md:50-239` (moved on 2026-10-18 to `package/translation-repair/doc/seats-and-calibration.md`)
names about fifteen identifiers,
four artefacts and two mise tasks removed in `cbedea357`;
`:124` claims a bootstrap build dependency `mise.toml` no longer has.
`blockPairingQuestion`,
`blockPairingProtocol` and `prepareBlockPairing` survive.

## D6: stale constants and counts in the docs

Status:
Markdown fixed in `4397d7d2a`;
`corpus-run/run-config.ts:785` fixed in `79b749388`
(the comment was right when written on 2026-07-26,
at a 90 minute cap,
and went stale at each raise).
`doc/pictures.md:89` (moved on 2026-10-18 to `package/translation-repair/slice-reading/doc/pictures.md`) says 8 MiB (7
MiB since 2026-08-22,
`image-reading-stage.ts:120`);
`doc/configuration.md:18` (moved on 2026-10-18 to `package/translation-repair/doc/configuration.md`) says a run without
the Synthetic key throws (every key is optional,
`corpus-run/run-providers.ts:94-130`);
`doc/configuration.md:201` (moved on 2026-10-18 to `package/translation-repair/doc/configuration.md`) says a stalled
entry drops after its second try (its first,
`corpus-run/entry-reattempt.ts:214-223`);
the picture reader count is four or five in `README.md:353`,
`doc/pictures.md:54` (moved on 2026-10-18 to `package/translation-repair/slice-reading/doc/pictures.md`),
`doc/slice-context.md:316` (moved on 2026-10-18 to `package/translation-repair/doc/slice-context.md`) and
`image-reading-stage.ts:17,21` (six);
`README.md:109` says 20 Synthetic slots across four models (two models,
10);
`README.md:287` misdescribes stage quorum retries (`stage-quorum.ts:337-376`);
`doc/seats-and-calibration.md:458` (moved on 2026-10-18 to `package/translation-repair/doc/seats-and-calibration.md`)
says ten editors (nine);
`corpus-run/run-config.ts:785` says a 90 minute entry ceiling (420);
`doc/provider-availability.md:67,114,121` (moved on 2026-10-18 to
`package/translation-repair/doc/provider-availability.md`) frames two providers (four meters).

## D7: the OpenRouter routing description is stale

Status:
fixed in `4397d7d2a`.
`doc/configuration.md:73,81-85` (moved on 2026-10-18 to `package/translation-repair/doc/configuration.md`) and
`doc/roster-changes.md:25-31` (moved on 2026-10-18 to `package/translation-repair/doc/roster-changes.md`)
against `model-cards.ts:209,320-328` and `openrouter-catalog.ts:98-100,263`.

## D8: comments name functions that never existed

Status:
fixed in `8ce84fb73`,
which also fixed D16's `SLICE_SPELLED_KEYS` part.
`image-reading-pair.ts:393` names `runStageRound` (`runGatherRound`);
`declined-target-runs.ts:44` names `pairBlocksAcrossRoster` (`pairBlocksWithRoster`);
`artifact-key-vocabulary.ts:19,47,104` miscounts its keys and misdates a table.

## D9: examples call the wrong function or pass keys it does not take

Status:
fixed in `d43933e6e` (red guard `d181e7c9b`,
`tsdoc-example-scan.unit.test.ts`),
which scans every source file's `@example` against the function it documents.
Mutation check:
dropping the callee report,
the key check,
a required key from a source example,
or counting defaulted keys as required was each caught;
the control survived.
The scan flags an example only when it never names its function,
since an example may pass a predicate by name rather than call it.
Wrong callee or keys:
`contributor-translation-guard.ts:22`,
`corpus-run/displacement-probe.ts:110`,
`delivery-coherence.ts:174,245`,
`editor-ensemble.ts:314`,
`consolidate-slice-buy.ts` (`buyConsolidationAttempt`),
`corpus-run/artifact-two-lane-read-naturalness-seat.ts:98`,
`corpus-run/pass-archive.ts:28`,
`restore-typography.ts:94-105` (no `@param` for `mask`).
These omit a required key:
`apply-patch.ts:194` and `editor-ensemble.ts:472` (`preservation`),
`corpus-run/artifact-two-lane-consolidate.ts:446` (`sliceIndex`),
`corpus-run/artifact-two-lane-read-contest.ts:68` (`keys`,
`generation`,
`comparison`),
`corpus-run/artifact-two-lane-read-rows.ts:52,140,278` (`keys`),
`corpus-run/band-order.ts:140` (`settledPerBand`),
`corpus-run/bench-sample.ts:81` (`pin`),
`corpus-run/pass-entry-artifact.ts:56` (`pageAssembly`),
`corpus-run/pass-entry.ts:586` (`publishDir`,
`declinedDir`),
`corpus-run/rendering-audit-settled-input.ts:282` (`runSetDir`),
`corpus-run/title-reference-locate.ts:411` and `corpus-run/title-reference-marks.ts:193` (`rendering`),
`document-readings.ts:74` (`readOcr`),
`front-matter.ts:320` (`openLength`),
`lane-slice-text.ts:196` (`notApplicableHere`),
`pair-agreement.ts:175` (`pairings`,
`needed`,
`pairingShape`),
`refine-eligibility.ts:224` (`minimumChars`),
`refine-slice-settle.ts:460` (`refineContributors`),
`repair-chunk-verdict.ts:108` and `repair-refine-step.ts:95` (`declaredNames`),
`repair-contract.ts:254` (`role`),
`sample-manifest.ts:166` (`generation`),
`slice-cost-log.ts:189` (`signal`),
`stream-drain.ts:191` (`label`).
The probe checked `function` declarations only,
not arrow functions,
methods or types.

## D10: owner quotes not in the record

Status:
fixed in `7b2d382bc`;
the first cites the paraphrase's decision record,
the second quotes in full.
`provider-name.ts:11` quotes the owner in the first person where the records hold a paraphrase;
`translate-runoff-tie.ts:25` truncates "prefer the best valid proposal,
else fail the slice at once".

## D11: status and hygiene

Status:
Markdown fixed in `4397d7d2a` except the ALL-CAPS paragraphs it did not touch (D19);
`seat-tally.ts:326` fixed in `59fb991c7`,
which gives the decision-seat tally its real reason
(the `SEAT` lines and the dark-seat report).
`doc/status.md` (moved on 2026-10-18 to `package/translation-repair/doc/status.md`) puts its history under the current
heading,
names a consolidation cache generation 14 that collides with artifact schema generation 14,
says `assertFinalSelectionSettled` remains (removed in `1ba8f713a`),
and calls a 2026-08-26 audit current evidence.
ALL-CAPS emphasis across the docs,
lines over 120 characters in `doc/slice-context.md` (moved on 2026-10-18 to
`package/translation-repair/doc/slice-context.md`) and `doc/status.md` (moved on 2026-10-18 to
`package/translation-repair/doc/status.md`),
positional references (`doc/configuration.md:328` (moved on 2026-10-18 to
`package/translation-repair/doc/configuration.md`),
`doc/status.md:105,123` (moved on 2026-10-18 to `package/translation-repair/doc/status.md`)),
an italic (`doc/status.md:428` (moved on 2026-10-18 to `package/translation-repair/doc/status.md`)),
unbackticked model ids,
double blank lines,
mixed list markers in the README.
`seat-tally.ts:326` cites an untracked script.

## D13: a setting documented as read by launch logs that nothing reads

Status:
fixed in `0df81f0c1`:
`corpus-pass` prints `CORPUS PIN OVERRIDDEN`
naming the clone,
the commit and each one's source whenever either comes from the environment;
`doc/configuration.md` (moved on 2026-10-18 to `package/translation-repair/doc/configuration.md`) said no launch line
did until `f921e8f0e`.
`corpus-run/run-config.ts:875-879`:
the `RUN_CORPUS_PIN_SETTING` TSDoc says "for launch logs",
but nothing reads it,
so no launch line names where the corpus pin came from.

## D14: an invalid Hyper request rate is not refused

Status:
fixed in `f69912818` (red guard `2fc22b098`):
a set value that is not a positive number is a stated refusal.
`request-pace.ts:305` falls back to the default rate for an invalid `TRANSLATION_REPAIR_HYPER_REQUESTS_PER_HOUR`,
where every other dial refuses an invalid value.

## D15: probe variables exported empty are used as given

Status:
fixed in `ea0493bd5` (red guard `89caeda65`):
both read through `textSettingOf`,
which folds an exported-empty variable into its fallback as `artifact-pool.ts` does.
`corpus-run/damage-sample.ts:267` and `corpus-run/score-verify.ts:199` read their variables with `??`,
so an exported-empty `DAMAGE_SAMPLE_SEED` or `VERIFY_SHEET_BASENAME` is used rather than defaulted or refused.

## D16: stale comments on providers and removed features

Status:
fixed in `5a5079549`,
the fence test in `9f20342b1`;
the `SLICE_SPELLED_KEYS` part in `8ce84fb73` (D8).
`corpus-run/budget-sample.ts:14-27,54-55` says "both providers" (four);
`block-pairing-question.ts:41` and `block-pairing-protocol.ts:63` mention replay and receipt planning
that `cbedea357` removed;
the `SLICE_SPELLED_KEYS` TSDoc in `artifact-key-vocabulary.ts` says generation 3 writes the slice spelling
and its header compares generations 1 and 2 with 4.

The two block-pairing sites were a family,
not a pair (M35).
A search for the removed layer's vocabulary (receipt,
replay,
recipe,
retally,
qualification,
calibration pool)
found the same framing in `block-pairing-question-key.ts`,
`pair-blocks-read-outcomes.ts`,
`pair-blocks-stage.ts`,
`prepare-block-pairing.ts`,
`queried-block-pairing-details.ts` and `prepare-with-pairing.ts`,
and in two test names;
all were rewritten to say what the code does now.
`budget-sample.ts` now names the four meters (three endpoints and the Bedrock spend ledger)
and its `@throws` names `StatedRefusalError`.

Found while fixing:
`pair-blocks-wire.ts` and `pair-sections-wire.ts` said their fence guards against a run of backticks
and gave a backtick fence in their examples;
`selectFence` has fenced with `=` since `b111fc376`,
before either sheet was written.
The section sheet's fence test fed a fixture holding backticks only,
so a builder that ignored the content and fenced with a fixed `=====` passed it:
that mutant survived,
the block sheet's twin was caught,
and the control survived.
The fixture now carries a setext underline,
and the test first checks that the line it reads is the fence around that fixture;
the mutant is now caught by both files' runs.

## D17: the recovery round re-asks only the last round's unreadable seats

Status:
fixed with P2 (`005692e11`,
guard `3549b74be`);
the README described the old behaviour until the P1 documentation commit,
which now says what P2 does.
`stage-quorum.ts` set `unreadable = answeredBadly` each round,
so a seat that answered unreadably in an earlier round was not re-asked.

## D18: repo docs name a corpus variable nothing reads

Status:
fixed in `f921e8f0e`.
The variable belonged to the uncommitted fork the pull-request 386 runs used on 2026-08-29;
production has read `TRANSLATION_REPAIR_CORPUS_CLONE_DIR` and `TRANSLATION_REPAIR_CORPUS_COMMIT`
since `b0a79eb66` (2026-09-01).
The runbook now names those;
the handover keeps its history with a note saying so.
`doc/runbook/translation-repair-corpus-pass.md:38` and `doc/handover/translation-repair-handover-2026-08-29.md:379`
name `TRANSLATION_REPAIR_CORPUS_DIR`.

## D19: ALL-CAPS emphasis left in untouched paragraphs

Status:
fixed in `069b4df46`.
Emphasis became sentence case,
bold where the stress carries meaning;
the three seat states `producer-silence.ts` prints became code spans in prose rather than labelled bullets.
A probe over every package Markdown file,
not only the files this finding named,
found four more in `doc/repetition.md` (moved on 2026-10-18 to `package/translation-repair/doc/repetition.md`) and four
in this ledger,
fixed in the same commit.
Kept:
the owner's words quoted in `doc/status.md` (moved on 2026-10-18 to `package/translation-repair/doc/status.md`) and
sheet text quoted in this ledger.
`README.md`,
`doc/configuration.md` (moved on 2026-10-18 to `package/translation-repair/doc/configuration.md`),
`doc/design-commitments.md` (moved on 2026-10-18 to `package/translation-repair/doc/design-commitments.md`),
`doc/pictures.md` (moved on 2026-10-18 to `package/translation-repair/slice-reading/doc/pictures.md`),
`doc/provider-availability.md` (moved on 2026-10-18 to `package/translation-repair/doc/provider-availability.md`),
`doc/roster-changes.md` (moved on 2026-10-18 to `package/translation-repair/doc/roster-changes.md`),
`doc/seats-and-calibration.md` (moved on 2026-10-18 to `package/translation-repair/doc/seats-and-calibration.md`),
`doc/slice-context.md` (moved on 2026-10-18 to `package/translation-repair/doc/slice-context.md`) and `doc/status.md`
(moved on 2026-10-18 to `package/translation-repair/doc/status.md`) keep ALL-CAPS emphasis in paragraphs the D fixes did
not reach
(the docs agent's report of 2026-09-27 lists the lines).

## D20: pairing fields kept only for the removed preparation layer

Status:
fixed in `89a15887a`,
found while fixing D16.
`BlockPairingOutcome.outcomes` (`919e517e4`,
2026-09-11) carried every asked seat's reply,
and `PreparedBlockPairing.evidence` with its type `PreparedBlockEvidence` (`536cd7445`,
2026-09-11)
said whether a section's relations came from the cache or the roster;
both were added for the receipt and replay layer `cbedea357` removed,
and after it only tests read them.
Removed by the criterion `cbedea357` itself used:
nothing a pass or probe reaches reads them.
The pairing cache stores `{ pairs, findings }` alone,
before and after,
so no cache version moves.
The tests that read the fields now assert heard and usable counts,
findings,
exact pairs and call counts;
the census of those tests first named four files,
and the type check named two more (M33).

## D21: log-line and status names in this ledger outside code spans

Status:
fixed,
found while fixing D19.
Eighteen lines of this ledger named a line or status the package prints
(`SPEND`,
`METERS`,
`SEAT`,
`TALLY`,
`DONE`,
`ERROR`,
`INCOMPLETE`,
`REFUSED`,
`SURVIVED`,
`[PASS]`,
`[FAIL]`)
as bare capitals,
where the other docs span them;
one of them was written on 2026-09-28 closing D11.
One line named a log file bare as well.
A probe over every package Markdown file found none outside this ledger.

## D24: the README's provider section counted a culled model and named a candidate by a label the ledger reuses

Status:
fixed 2026-09-28,
found re-reading the section against the code after the provider changes.
The section said Bedrock serves four models here;
four cards name a Bedrock id (`model-cards.ts`),
but `hf:openai/gpt-oss-120b` carries `owner-culled` and left every role on 2026-09-24,
which the same section says two sentences earlier,
so three are in use.
It also cited "A2's 432-second" serial run with no source,
where this ledger's A2 is a page-assembly finding;
the run is Candidate A2 in `doc/planning/translation-repair-interface-candidates-a-d.md` at the repository root,
now named.
Every other claim in the section holds against the code:
`SYNTHETIC_PER_MODEL_CONCURRENCY` is 5 (`synthetic-client.ts`),
Synthetic serves two roster models (GLM-5.3-Flash is `synthetic-withheld`,
gpt-oss-120b `owner-culled`),
Hyper,
OpenRouter and Bedrock set no per-model ceiling (`Number.POSITIVE_INFINITY` in each client),
and `PROVIDER_ORDER` is Synthetic,
Bedrock,
Hyper,
OpenRouter (`provider-name.ts`).

## D28: the document map's current status stopped at 2026-09-06

Status:
fixed 2026-09-29,
found the same day while scoping D26.
`doc/handover/translation-repair-document-map.md` opens with a "Current status" section
that still describes the legacy pipeline as read on 2026-09-04 and 2026-09-06
and names the 2026-09-06 snapshot as the place to start,
while the package has since been audited and rebuilt through this ledger.
Fix:
rewrite the section from the current handover and the package README,
each claim checked against the code or a log,
and have it point at the package docs that now carry the current state.

What was done:

-   The map's "Current status" now carries its date,
    the providers,
    the audit in progress and where its record lives,
    and the last pass with why none has launched since.
    Each claim was read from its source:
    the pass log's section on TianqiChen66621's read for the run and its class,
    the owner's words from the pass log's audit section,
    `PROVIDER_ORDER` as the README states it,
    and the absence of a later pass from `~/temp/agent`,
    where no run log or runs directory is newer than `TianqiChen66621.log`,
    and from the pass log,
    ledger and snapshot,
    none of which names a class one hundred eighty-eight.
-   Its reading list starts at the current snapshot's checkpoints and "What to do next",
    adds this ledger,
    the prevention doc and the README's list of package docs,
    and names the pass log's newest heading with the date it was read rather than its place,
    since the pass log runs oldest first to the Mio23 launch of 2026-09-16 and newest first after it.
-   Three more stale claims on the same two pages:
    the map said no new pass is authorized,
    where the index has said since `c4e65a545` (2026-09-04) that corpus runs on the legacy pipeline are authorized;
    the map said the handover links three dated snapshots,
    where the index links four historical ones and the current one;
    and the index described the snapshot as it stood on 2026-09-06
    and put the kill-and-relaunch rule in the package README,
    whose section of that name moved to `doc/configuration.md` (moved on 2026-10-18 to
    `package/translation-repair/doc/configuration.md`) with the README's split (`bc0050366`,
    2026-09-16).
-   Every relative link in the two files resolves
    (`~/temp/agent/audit-glossary-fix/local-links.mjs`,
    53 links,
    shown first to report a planted missing file and a missing fragment).

No guard can tell a current status paragraph from a stale one,
so the prevention is a habit,
recorded under M55,
which found the snapshot's own next steps stopped the same way.

## D30: the runbooks and troubleshooting docs sat outside the living set

Status:
fixed 2026-09-29,
found the same day while linting the translation-repair documents outside the living set for D27.
The runbooks and troubleshooting docs are the references a pass is run and debugged by,
yet D25 and D26 read neither:
they carried 37 task-list citations and 6 zone-less clock times,
and the corpus-pass runbook quoted a tool message the tool no longer prints.
What was done:

-   Each citation was named from the earlier session's task record,
    or replaced by the commit that landed it (`b6ea1cc51`,
    `fd4f7546f`);
    issue 541,
    which the root-order troubleshooting doc links,
    is real and stays.
    A recorded monitor alarm that quoted a task number became a paraphrase,
    since the monitor was the session's own script.
-   Times the same record states in UTC now say so,
    and the typography fix's landing was checked against `846f9ff6d`
    (04:34 UTC).
    Two times in the run-invalidation doc were a predicted overnight launch whose zone was never set;
    `pass13` in fact stopped at 17:14 UTC by the watcher's log,
    so the sentences now say what happened in words,
    without a clock time.
-   The runbook's quoted `NO ROUND LINE` message now matches `run-timing-report.ts`.
-   A scan for stray backticks and asterisks in the rendered text,
    over every translation-repair doc outside the pinned archives
    (`~/temp/agent/audit-glossary-fix/stray-contexts.mjs`),
    found literal backticks where code was meant:
    in the invisible-characters doc,
    two bare triple backticks and a `git grep` pattern whose span closed at its first escaped backtick;
    in the package audit,
    two spans that closed the same way,
    and a third stray that was their knock-on.
    A backslash does not escape a backtick inside a CommonMark code span;
    each now uses a longer backtick run as its delimiter,
    and the linter's `--fix` then broke the prose those spans had hidden.

Guard:
`src/living-docs.test-fixture.ts` (moved on 2026-10-18 to
`package/translation-repair/source-scan/src/living-docs.test-fixture.ts`) adds the runbooks and troubleshooting docs as
a kind,
the task-list-number guard reads them with issue 541 allowed,
and the clock guard reads them.
A citation planted in the corpus-pass runbook failed the first,
a zone-less time planted in a troubleshooting doc failed the second,
and both files were restored from the commit.

## D34: seven clauses and names in the history archive that said more or less than the files held

Status:
fixed 2026-10-15 in `d6f7f9ae12` by the round n167,
found the same day by the third independent reading of the history archive (n165,
`s236/review.md`),
which read the fixes of the second reading (`730c503608`,
2026-10-13)
and found all ten of its findings fixed,
the cuts exact,
and seven low faults.
The archive is `doc/handover/translation-repair-history*.md` at the repository root;
its own wording is moved and never rewritten.
The owner's rule that no mistake is too small to fix made all seven fixes and not reports.

- Two provenance-note clauses claimed that the notes name sections they do not.
  The clause in the first editor calibration file now says what the note of 2026-08-25 says:
  it counts six sections aged out of the working handover and moved unedited,
  and names the three kept there,
  and the clause says where the other three went.
  The clause in the second file now says that the note of 2026-08-26 says this file's sections moved unedited.
  The evidence is the message of `e17c133648`
  (it moved the availability record,
  the imaginary blocker,
  the record numbers,
  the naturalness lane,
  the missing silent-model list and the superseded first reading of the deepseek anomaly,
  and kept the zero-rounds,
  slice-count and 14-slice sections),
  the headings of the files,
  and `2e67f872a8`,
  whose message says 25 closed sections moved verbatim.
- The top index entry for 2026-08-24 to 2026-08-25 left out the subjects of four files,
  and the entry for 2026-08-15 to 2026-08-19 left out two subjects
  (the removal of the proportional aligner,
  and the state record after the displacement screen was rebuilt).
- The see-also clause of the roster-sweep file pointed at a caveat and not at figures.
  A search of the 16 files of the group for figures taken from a partial sweep prefix found none,
  so the clause now says only what the ratio-guard file holds:
  the sweep and the warning not to read band and tail endpoints off a partial prefix.
- The see-also clause of the scope-correction file named no file.
  It now names the three files whose sections gate or time a full pass or its relaunch.
- The name of the second editor calibration file listed seven sections.
  It is now `TRH-2026-08-24-to-25-editor-calibration-reseat-evidence-the-14-slice-run-and-the-second-calibration.md`
  (`TRH-` stands for `translation-repair-history-`),
  which names the reseat evidence,
  the run and the second calibration.
- The name of the window trial file left out its largest section,
  the pre-flight and plumbing,
  and now names it.
- The top index now names the subject of each file of 250 lines or more in every entry,
  not only the two entries the reading named,
  and drops its one pointer by position.

The brief's verb for that last check is M225:
n167 first fixed six more entries,
read the verb,
and reverted them to proposals,
and the lead applied the six proposals after checking that each subject is the title of a file.
Those clauses had no reading by an agent that wrote none of them.

Guard:
the checks of the round,
each shown by its author to report a planted fault:
the index check over 12 groups,
the section lists of 11 indexes,
542 links and 610 mentions,
no archive body line added and no heading changed against `c3c0c8dda7`,
the five doc guard tests,
and the Markdown lint on the ten files.

## D36: the ledger named `everyStageHeard` as the cache's reader, and the function no longer exists

Status:
fixed on 2026-10-22 (UTC) by in-place notes,
found by the implementer of the review fixes of the XingZ6018 work.
`audit-ledger-which-seats-a-quorum-counts.md` names `everyStageHeard` at four places,
in the accounts of B141 and B160:
"passed `everyStageHeard` and was written to the refine cache",
"passed `everyStageHeard`",
"fail `everyStageHeard`",
and "failed `everyStageHeard`".
`b423e15be8` removed the function,
because the refine write gate and the read gate came to share one predicate,
and the ledger had no word of it.
The current names are `refinementIsResumable` and `silentStagesOfSettlement` in `stage/src/stage-silence.ts` (B820).

The fix:
each account keeps the name as written,
since it is the history of what the code was called when the defect stood,
and gains a note giving the current names and the date of the replacement (2026-10-22).
Nothing else in the package's docs or READMEs names `everyStageHeard`
(`rg` over the package and the repository's `doc` and `package` trees found only these four places in the ledger,
and the entries of this round).

Guard:
none can tell a name that was replaced from one that was never written,
so the prevention is a habit:
a commit that removes an exported name searches the docs for it and leaves an in-place note
where a historical record keeps it (WR6).

Recurrence:
`mistake-prevention-docs-and-references.md`,
"Current-state docs".

## D37: five comments and docs of the XingZ6018 fixes said something the code or the rulings did not

Status:
fixed on 2026-10-22 (UTC) in `9d6095ae7e`,
found by notes N1 and N2 of the review of the fixes and by its reading of the comment round (`855963d613`).

- `corpus-pass/src/corpus-run/pass-entry.ts` said the base signal is one the pass never aborts,
  "so only a per-entry timeout ever fires".
  The dry stop made `dryStop.signal` the base signal.
- `slice-cache/src/corpus-run/entry-error-outcome.ts` documented `aborted` as whether the hard-ceiling abort fired,
  and the TALLY line prints it as `aborted=`.
  A dry stop sets it too,
  so a reader scanning TALLY lines for hard-cap hits was misled.
  The doc now says the field reports the dry stop as well.
- The TSDoc of `EveryProviderDryError` and its `messageNamesOnly` comment said the message holds fixed sentences plus the
  meter states and the hold lengths.
  It also states a time of day (B819),
  and "the day start decided at the start of this run" was wrong for a day start learned on a retry.
- The Synthetic client's comment kept Synthetic in the third tier "until a bench says otherwise".
  That condition is in no ruling.
  It now states the bench fact:
  Synthetic's probes of 2026-10-22 showed `low` turning thinking off,
  while `high` and `max` do not separate (GLM-5.3-Flash `high` and `max` about 430 to 740 reasoning tokens
  on a question where NeuralWatt's copy of the model separated `low` 116 to 143,
  `high` 169 to 222 and `max` 355 to 579).
- The NeuralWatt card comment cited a probe "by the parent session",
  which means nothing to a later reader,
  and the raw numbers stood nowhere.
  They now stand in the section "Thinking-parameter evidence" of the provider README,
  and the comment cites that.
  The shared contract's comment on Anthropic now words the clearance as the owner's ruling,
  with no direct client for it yet.

Guard:
none for the wording.

Recurrence:
`mistake-prevention-docs-and-references.md`,
"Current-state docs".
