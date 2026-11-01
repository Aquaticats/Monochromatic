# Audit ledger: the measured model, the author and the order of the re-ask

The second re-check of 2026-10-31 (UTC) of the re-ask of another model
(the ledger
[the reach of the re-ask](audit-ledger-the-reach-of-the-re-ask-and-the-repair-of-another-model.md)),
and the rulings that followed it:
the gate (B1140),
the measurement that re-asks the model it measures (B1141),
the model that answered for a seat as the author of its text (B1142),
the holds a run decides when it seats a role (B1143),
the order of the stand-ins by role standing (B1144),
the second-person words reaching the other translator (B1145),
the findings that name the answerer (B1146),
the bench of the repair (B1147),
the benches derived from the pools (B1148),
the statements of this record that were stale (B1149),
the four gaps of the role reader (B1150),
the drift tests that read a build (B1151),
a comment about a refusal (B1152),
and what stays open with the slips (B1153).

The third re-check of 2026-10-31 (UTC) of this record,
with its fixes and the statements of this file it corrected in place,
is in the entries from B1160 to B1169 of
[the third re-check and the cache versions it moved](audit-ledger-the-third-re-check-of-the-re-ask-and-the-cache-versions-it-moved.md).

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B1140: the gate: a role test that read nothing, a sheet outside the writers and an index past its bound

Status:
fixed on 2026-10-31 (UTC) in `0db13efc7e` (test-fixture and stage;
red `8abe94b012`),
`ef4161e1d2` (artifact)
and `f1ec23ca63` (docs).

- The stage role test (`stage/src/reask-role.unit.test.ts`) named `gatherStageVoices` as the callee of
  `pair-sections-stage.ts`,
  which calls `runWindowedRounds` and never the gather.
  The reader returned `[]` for a callee the source did not call,
  and the case that expected a role got nothing to compare and said nothing of why.
  The brief said to fix the reader and not the expectation.
  The reader is fixed:
  `rolesNamedAtCalls` now throws when the source never names the callee,
  since a test that names the wrong function is not a file with no call.
  The expectation is also corrected,
  because it named the wrong function:
  the test now names `runWindowedRounds`.
  This departs from the brief's wording by also changing the test;
  the reader alone would have made the old test fail and left it red.
  The red case is `8abe94b012`,
  which reads a source that names `runWindowedRounds` and asks for `gatherStageVoices`.
  `98c1f9f39f` committed the stage test red while its message said it passed.
- The sheet "translate repair for another translator" revises a translation,
  so it is a writer:
  `artifact/src/sheet-translators-notes.unit.test.ts` lists it among `WRITERS`,
  with the reason of B698 and B702:
  it returns wording that replaces the translation,
  so a note it keeps stands carried and one it leaves out is put back after the slice.
  The case that failed was the one the brief named;
  the message of `ef4161e1d2` takes its "failed at its second case" from the brief and does not re-run it.
- `audit-ledger.md` stood at 998 lines,
  and listing the files of this round would have passed 1,000.
  Its seven weight-floors bullets (76 lines) moved into an index of their own,
  [Audit ledger: the weight floors removed](audit-ledger-weight-floors-removed.md),
  which the main index lists in five lines,
  the way the process mistakes left it on 2026-10-29.
  The seven files now say they are part of that index.
  The main index stood at 944 lines with the files of this round listed
  (the first count of 943 was a miscount;
  B1169).
  The areas index lists the reach ledger,
  which it lacked,
  and the new index.
  The by-id index lists ids and needs no change for a moved bullet.

Cache versions:
none moves.

## B1141: a measurement re-asked a rival of the model it measures

Status:
fixed on 2026-10-31 (UTC) in `7179618d23` (78 files;
reds `ec515a9fb1` provider,
`9484424f35` stage,
`a535680bf4` slice-reading
and `5b97f0b90a` source-scan and test-fixture).
The lead's ruling:
a probe,
a bench,
a calibration or a census scores the model it asks,
so a reasoning runaway there is re-asked of that model itself,
on another provider and then nudged as the no-seat path of B1081 does,
and never of another model,
whose answer would be recorded as the measured model's and scored as its own.
B1103 applied this to the picture reader of `reader-measure-run.ts` only.

What it does now:

- The mode is `ReaskMode`:
  `other-model` for the pass,
  `measured-model` for a measurement.
  A launch names it in `readStageSettings({ env, reask })`,
  where it is required and has no default,
  so a launch cannot forget it.
  The gather,
  the windowed rounds,
  the translate repair and the picture reader take it as a value and put it on the `ReaskSeat`
  (`reask`).
  That the mode is required in all of them was not so when this entry was written:
  `ReaskSeat.reask` and the `reask` of `readPastRefusal` were optional,
  an absent mode meant another model,
  and the scan read neither (B1163 closes it);
  the three places that re-ask read it through one function (`measuresModelOf`):
  the provider (`reaskElsewhereNudged`),
  the stage recovery round (`askSeatItself`)
  and the picture re-ask (`reaskCut`).
  A measurement logs at info that it asks the model itself and does not raise the B1081 warning.
- Every launch,
  by command.
  The pass (`corpus-pass`) names `other-model`.
  The benches (`bench`) name `measured-model`:
  `checker-sensitivity`,
  `editor-width-probe`,
  `producer-calibrate`,
  `roster-bench`,
  `sentinel-probe`,
  `translate-probe`
  and `window-trial-probe`,
  and the editor calibration (`editor-calibrate-run.ts`),
  whose settings are a literal and name `measured-model` there.
  `reader-measure-run.ts` hands the picture reader no bench,
  so the picture re-ask asks the reader itself (B1103).
  The coverage census names it in `coverage-probe` and `coverage-control-probe`.
  The probes (`probe`) name it in `audit-sensitivity`,
  `damage-sample`,
  `editor-fidelity-pool-verdict`,
  `editor-fidelity-probe`,
  `judge-fidelity-probe`,
  `probe-relabel`,
  `probe-sensitivity`,
  `probe-verify`,
  `recall-benchmark`
  and `rendering-audit-settled`.
  That is twenty calls of `readStageSettings` in twenty files,
  nineteen `measured-model` and the pass `other-model`,
  and the one literal of the editor calibration.
- The translate probe asks its editors in the `editor` role and no longer in the roster role.
- The guard:
  `source-scan/src/measurement-mode.unit.test.ts` reads the mode each launch names
  (the packages of the measurements `measured-model`,
  every other package `other-model`),
  fails if a measurement package names `other-model` anywhere,
  and fails if a production call of `gatherStageVoices`,
  `runWindowedRounds` or `repairInvalidCandidates` passes the mode as anything but the launch's own
  (`stageSettings.reask` or a parameter named `reask`).
  A measurement written later therefore cannot take the mode of the pass without a finding.
  Out of its reach:
  a settings object built as a literal,
  which the type forces to name `reask`,
  and a stage that receives the mode under another name.

Cache versions:
none moves.
No stored key or record changes;
the mode decides which model a runaway seat is re-asked of,
and a re-ask is bought under the prompt of the model it goes to.

## B1142: the model that answered for a runaway seat is the author of what it wrote

Status:
fixed on 2026-10-31 (UTC) in `98d5b63eb3` (25 files;
reds `79f848f552` stage and `69c982305d`,
eight cases in eight files).
The owner's "Other model,
own name" (2026-10-31) and the lead's rulings:
the own text of a model weighs half,
a checker that wrote the text it checks counts as its author,
one judge alone never decides,
and a re-ask must not give one model two voices.
B1102 recorded a revision by another model as its own candidate in the translate repair;
the same had to hold wherever a re-ask answers for a seat.

What it does now:

- A heard voice (`HeardVoice`) names the model that answered in `modelId`
  and,
  apart,
  the seat it answered for in `seat`.
  `seatOfVoice` reads the second,
  and `answererOf` and `heardVoiceOf` (`stage/src/stage-answerer.ts`) read the first.
  The recovery round marks the voice it returns with `answeredBy`
  (the model it asked,
  or the one that model was itself re-asked of).
  The gather logs both the seat and the answerer.
- Weight,
  half weight on a model's own text,
  the check of a model on the text it wrote,
  the tallies and the stored records read the answerer:
  the editor candidates and the refiner replies are credited to the author but ordered by the seat;
  a judge's ballot in the translate stage is weighed under the model that cast it;
  translation candidates are placed at the seat and produced by their author;
  the contest ballot,
  the naturalness review seat,
  the consolidation gate and the pairing log lines read `answererOf`.
- The claims still stop two voices of one model (B1080):
  a model that answered for one seat is not asked for another seat of the question,
  and a seat that ran away is counted unusable while its answerer is counted usable
  (`seat-tally.ts`).
- Stored consistency voices stay under their seat,
  so a stored answer still fits the bench (`recordFits`),
  and gain the optional key `answeredBy`;
  the artifact ballot of the first round of a group names the answerer,
  and the voices of a gather are ordered by seat.
  The ballots of a carry and of its run-off,
  the ballots of a page's run-off and the writers of a carry candidate
  still recorded the seat when this entry was written,
  and the entry said the artifact named the answerer without
  that limit (B1160 closes it).

B1107 turned around.
B1107 pinned that no model of the checker order is an editor or a refiner,
because a re-ask recorded the answer of another model as the checker's voice and the voice did not carry who answered.
The voice now carries it:
`wroteTheText` in `repair-edit-stages.ts` is computed from the voice's `modelId`,
which is the answerer.
A checker-order model seated among the editors later would therefore check its own text as its author and not at full
weight,
with the pin of `seat-role-pool.unit.test.ts` kept as a second layer and no longer the only one.

Cache versions:
`CONSISTENCY_CACHE_VERSION` 9 to 10,
because a stored voice of the page consistency gathers (ballot,
choice and writer kinds) gains the optional key `answeredBy`;
its Version 10 paragraph in `consistency-cache-version.ts` says so.
This entry said no other version moves,
because the stored translate,
consolidate,
repair and refine records hold candidates by wording and producer,
and a producer is a model id of the same shape whoever it names,
and a slice key digests the prompt and the bench.
That was wrong:
the producer a record names is now the answerer where it was the seat,
so the same key is weighed and checked by another model than before,
and the five stages that read it move their versions (B1168).

## B1143: a re-ask honoured the holds of a role but not the holds the run decides when it seats it

Status:
fixed on 2026-10-31 (UTC) in `b8bc752846` (16 files;
red `4fe68921ba`).
The owner's "Honour holds (Recommended)" (2026-10-30):
a runaway is re-asked of a model its card does not hold out of the role.
Two holds were applied by the run when it seats a bench and by no card:
Qwen3.8-27B is withheld from every bench but the editors and the refiners,
and Kimi-K3 is withheld from the select and slate seats.
A re-ask drew from the pool of the role and could ask either.

What it does now:

- The card of Qwen3.8-27B carries the hold `hyper-slow`,
  which `run-seats.ts` applied from a set of its own
  (`HYPER_SLOW_JUDGES` is now the `holdSet` of that hold).
- `ROUTE_TIME_HOLDS_OF_ROLE` in `seat-role.ts` writes once per role the holds the run applies at seating time:
  `hyper-slow` for every role but the editors and the refiners,
  `hyper-slow-select` for the select and slate roles.
  `reaskHoldsKeepingOutOf` and `reaskPoolOf` give the re-ask the pool of the role less those models,
  and `alternateModelsFor` names the hold that left a model out.
- The roles are eleven:
  the nine of B1100 and `select-judge` and `slate-judge`,
  which carry the static holds of the wide and late roles.
  The slates of the translate lane,
  the editors,
  the refiners and the polish name `select-judge`,
  and the consolidation slate names `slate-judge`.
  A critic or a panel re-ask may still ask Kimi-K3.
- The static exclusion is exact:
  a model is on the bench,
  excluded as the bench,
  or withheld,
  and a withheld model is excluded.
  The three provider holds (openrouter,
  Synthetic and the dry meters) are closed by the router's `NoProviderForModelError`,
  and the comment in `seat-role.ts` that said route holds apply "without a table" is corrected:
  the router closes those three,
  and only `run-seats.ts` read the two holds named here.
- The guard:
  `run/src/corpus-run/seat-role-pool.unit.test.ts` derives the benches of eight readings of the meters
  (`judgeSeatsFor` with Synthetic,
  Hyper and OpenRouter each wet or dry)
  and fails when a model the run withheld is in the re-ask pool of the role while some provider would serve it.
  The Synthetic-dry reading is one of the eight.

B1101 said the one exception to a pool equal to the bench is "a seat the run withheld from the bench of a reading",
asked because the gather's bench excludes it and the pool does not.
That exception is gone for the holds this entry names:
the pool no longer holds a model the run withheld from the role.
It stands only for a model the pool does not name for another reason,
and none is known.

Cache versions:
none moves;
the roles and holds decide only which other model a runaway seat is re-asked of.

## B1144: the stand-ins of an editor or a refiner are asked in the order of their standing

Status:
decided and fixed on 2026-10-31 (UTC) in `92b8756112` (red `fedfcb8265`).
The owner's answer of 2026-10-31 to the order of the alternates:
"Order by role standing (Recommended)".
For the editor and refiner roles only,
the alternates are ordered by that role's standing in the editor calibration of 2026-09-01,
best first;
models never measured in the role follow in the order of their lowest share of runaways;
every other role keeps the lowest share of runaways (B1075).
Holds apply first (B1143).

The owner was shown this result,
and the test pins both orders:

- Editor stand-ins:
  Qwen3.8-27B (17.3),
  Kimi-K3 (14.2),
  minimax-m3 (8.4),
  gemma-4-26b (7.0),
  then e2b and Mercury.
- Refiner stand-ins:
  glm-5.3 (12.1),
  Kimi-K3 (7.8),
  Qwen3.8-27B (7.7),
  gemma-4-26b (3.4),
  then e2b and Mercury.

`ROLE_STANDINGS` in `shared-contract/src/seat-role-standing.ts` is one typed table of those shares,
cited to the calibration (the models that left the roster since are left out),
and `standingOrderOf` reads it.
`alternateOrderOf` in `provider/src/reask-alternate-models.ts` is the one place that decides the order:
the models a role has a standing for,
best first,
then the others by the lowest share of runaways.
A role with no standing keeps the share alone.

Cache versions:
none moves.

## B1145: second-person words reached the translator that was not the author

Status:
fixed on 2026-10-31 (UTC) in `594d9d1262` (red `f0d363bbe9`).
The owner's "Other model,
own name" (2026-10-31):
when another translator revises a translation,
the follow-up it is handed must read as written to it,
and not carry words addressed to the author.
The turn was made from the author's by replacing `Your ` and `your ` with the article across the whole text,
which also rewrote a word of the model's own quoted text,
and the rules said the original passage was rendered by another translator,
which clashed with the sheet's term for the original.

What it does now:

- `articleForPossessive` swaps the possessive for the article outside double quotes and backticks only.
  Three cases in `translate-repair-another-wording.unit.test.ts` hold it:
  the swap outside quotations only,
  a second-person check over findings,
  and one name for the text under revision.
  The second case as first written fed four hand-typed strings (not five) and imported no finding producer,
  so it held nothing of the producers;
  B1167 replaces it with one that reads their output.
- The rules say the translation to revise was written by another translator.
- Five findings that spoke to the author in other words are written without a second person for both turns:
  the lines the rendering carries (`line-structure-guard.ts`,
  `bilingual-pair-bound.ts`),
  the wording as it stands,
  the line breaks that were merged,
  the title as it was rendered (`translate-declared-link-name.ts`),
  and the evidence shown to the translator (`translate-sheet-leak.ts`).
  The case of f0d363bbe9 that checks them was written with neutral strings typed by hand and read no producer,
  and its comment said they were "as the source of each writes it today",
  which was not so when it was written,
  since the five sources were reworded later,
  in `594d9d1262`;
  it was not shown red against the old wording until B1167.
- The 4 tests that pin the reworded strings follow.

Cache versions:
none moves.
A finding is a log and artifact note and no stored key or record,
and the follow-up turn of the other model is a prompt of its own that was never replayed from the author's.

## B1146: a repair finding named the author and not the model that answered for it

Status:
fixed on 2026-10-31 (UTC) in `594d9d1262`.
The unheard,
unable,
as-intended and unresolved findings of `translate-repair.ts` named the author,
and since B1102 the answer can come from another translator.
They now name the author and the model that answered,
as `<author> -> <answerer>`,
as the revised-by-another finding already did,
so a reader of the log or the artifact sees which model's words each finding records.
A finding of an author that answered itself is unchanged.

Cache versions:
none moves.

## B1147: the repair bench is the heard authors, and so a second chance and not a second voice

Status:
recorded on 2026-10-31 (UTC).
The bench the translate repair draws its alternates from was the translators that were heard in the gather,
and the pool is the translators the cards allow,
so the repair bench was narrower than the gather bench,
which also held the seats that ran away.
That left a hole,
which B1161 closes:
a voice carried the model that answered and not the seat it answered for,
so a seat that ran away left the bench and could be asked to revise the text of the model that answered for it.
The bench now holds the seat and the answerer of every voice.
This is a second chance and not a second voice:
the repair asks an author about its own findings,
and when that author ran away it is asked of a translator who wrote no candidate for the slice,
whose answer is recorded under its own name (B1102,
B1142).
The shared record of the claims (`createReaskClaims`) is taken once per question,
by the gather and by the repair,
so a translator that took a runaway seat in the gather is never asked again for the same slice in the repair.
Two voices of one model cannot arise;
a heard author is asked about its findings once,
and the case in `translate-stage/src/reask-role.unit.test.ts` pins that the record is shared.

Cache versions:
none moves.

## B1148: the run's benches and the pool of a role were two lists

Status:
fixed on 2026-10-31 (UTC) in `2cf8691409` (5 files).
`run-config.ts` wrote the roster,
translators,
writers,
wide seats,
late judges and readers as lists of its own,
and the pool of each role was derived apart in the shared contract,
with a drift test between them (B1100).
The six benches now come from `chatBenchOf`,
which is the pool of the role (`rolePoolOf`) less the decision-only seats
(and,
for the readers,
the models that read no picture),
so each list has one source with the pool a re-ask draws from.
The alias `UNMEASURED_UNTIL_SEATED` is gone,
and the five hold sets whose documentation carried the history of a seat
(`WRITER_UNMEASURED`,
`TRANSLATOR_DROPPED`,
`WIDE_SEAT_DROPPED`,
`LATE_JUDGE_DROPPED`,
`READER_UNMEASURED`)
are comments now and no longer code,
since nothing but that documentation named them once the benches came from the pools
(the dead-code scan found all five).
The history is kept word for word under the name of its hold.
The holds of the editors and the refiners are the row of the writers,
those of the select seats the wide row and those of the slate seats the late row,
so a hold added to one is added to its twin.
The drift test is kept:
`seat-role-pool.unit.test.ts` compares the benches and the pools in eight readings,
and `shared-contract/src/seat-role.unit.test.ts` spells every role's holds out again apart from the table.
That case passed on first writing,
so it is not shown to fail by editing the table (GFP not met).
The drift case of the run package was not shown to fail either.

Cache versions:
none moves;
the benches are the same lists.

## B1149: statements of this record that were stale or overstated

Status:
fixed on 2026-10-31 (UTC) in the ledger and in three tests.

- B1101 said B1072 found the same weights on another route repeat a runaway "in 6 of 7 direct DeepSeek calls".
  B1072 found seven direct DeepSeek calls that ran to the cap,
  six of which came after a NeuralWatt answer of the same model failed the same way.
  How many NeuralWatt runaways were re-asked on DeepSeek and answered usably was not counted,
  so no share of repeats is known.
  B1101 is corrected in place.
- B1101 listed where another model is asked and omitted the bench commands and the editor calibration's lane.
  They ask the measured model itself since B1141,
  so no other model is asked there.
- B1101's item "the picture readers of a small bench" is no path of a real run:
  `image-reading-pair.ts` hands `readerModelIds`,
  and the run seats every model that reads a picture (`RUN_READER_MODELS`),
  so a pass reads each picture with all of them.
  Only a command that seats fewer readers has a small bench,
  and the one that does (`reader-measure-run.ts`) asks the reader itself.
- The third index,
  `audit-ledger-areas-of-the-audit-and-the-files-filed-under-each.md`,
  lacked the reach ledger;
  it lists it now beside the runaways ledger.
- The runaways ledger said in B1079 that the translate probe seats the roster
  and that the editors and the refiners stand in `roster`;
  notes of 2026-10-31 say what they are now,
  and its line 19 of 131 columns is wrapped.
- `repair/src/reask-role.unit.test.ts` said in its header and in two case names that the editors and the refiners
  are the roster role and that their slates are the wide judges;
  they are the roles `editor` and `refiner`,
  and their slates the select judges.
- B1106 said "two real faults" and listed three:
  the refiners' slate call,
  the consolidation slate argument and the window trial's slate call.
  Read again on 2026-10-31,
  the three moves changed no value at run time;
  they were defensive.
- B1106 said each fix came "after its red case".
  `be93348926` did not:
  its declaration-order case passed on first writing,
  and `cbea1aa6c3` now has that case beside the three that fail.
  That case was vacuous as `cbea1aa6c3` wrote it:
  it stayed green with the declaring-word check replaced by a
  test that always holds,
  since the equals sign of the declaration lay within the reach of the first use;
  B1166 gives it teeth.
- The line of this ledger that passed 120 columns (the header's link) is wrapped.

Cache versions:
none moves.

## B1150: the reader of the role a call names had four gaps

Status:
fixed on 2026-10-31 (UTC) in `6b8e8b45f5` (red `cbea1aa6c3`,
with three of its four cases failing against the reader of `be93348926`).

- A call whose argument is only a spread read `none`,
  so a case that expected `none` of such a call held vacuously.
  It reads `only a spread` now,
  which a case accepts by name.
  `translate-stage/src/reask-role.unit.test.ts` pins `translate-retry.ts` on it:
  the file names no role itself,
  its run-off and second judging spread the judging they were handed,
  and the committed reader returned `["not an object literal","none"]` for it.
- A spread of a call with an object argument (`...build({ a: 1 })`) was taken for an inline object whose keys can be
  read.
  It counts as able to replace the role,
  as any spread but an inline object or a conditional of one does.
- A role written as a computed key (`['role']: 'checker'`) was not read.
- The case be93348926 lacked:
  a name passed to a call before its declaration does not stand for the declaration.
  It passes,
  and is kept;
  as written it was vacuous (B1149,
  B1166).

Cache versions:
none moves.

## B1151: the drift tests read a build that `test:unit` does not make, and may not read source instead

Status:
answered on 2026-10-31 (UTC);
`2cf8691409` changed the two tests to import source and `7749a15d76` reverts that.
Two drift tests imported `dist/final/node/index.mjs` of the run and bench packages:
`run/src/corpus-run/seat-role-pool.unit.test.ts` and `bench/src/corpus-run/editor-lane-pool.unit.test.ts`.
The root `test:unit` task (`mise.toml`) runs the test files without building,
and only a package's `buildAndTest` task builds first,
so a drift test that read the build compared the source with whatever build was left.
`2cf8691409` made both import the source files,
and the oxlint rule `test-import/require-eventual-artifact` rejects a test that imports package source
(`doc/planning/oxlint-test-import-eventual-artifact.md`),
so both import the build again;
loosening the rule needs the owner's approval and none was asked.
The run case also imports the shared contract by its package name and not its `/ts` subpath,
since the identity of the checker order needs the one build that the run build and the test both read.
So `test:unit` does not build,
and a drift test reads the last build until `buildAndTest` builds again.
The suites of this round ran after every package was rebuilt in dependency order.
What would remove the risk is a `test:unit` that depends on the build of its package,
and that stays open for the lead (B1153).

Cache versions:
none moves.

## B1152: a refusal is a fact about the model that gave it

Status:
fixed on 2026-10-31 (UTC) in `e980f539cb` (slice-reading).
A comment after a cut reading called a refusal the roll of the model,
as if it were a fact about the picture.
It says a refusal is a fact about the model that gave it and not about the picture.
No behaviour changes.

Cache versions:
none moves.

## B1153: what stays open from the second round, and the slips

Status:
open,
for the lead and the owner.

- The pool of a role is read from the cards and the checker order and the standings table,
  and the route-time holds are written once in `seat-role.ts` and applied from `run-seats.ts`.
  The drift test reads eight readings of three meters;
  a run-time hold of a fourth kind would have to be added to both.
- The standings of B1144 are the calibration of 2026-09-01 as a typed table.
  A later calibration is not read by the code;
  the table has to be re-cited.
- A model asked outside the gather and the repair for the same question is still in neither record (B1109).
- The reading of how often another model is asked (B1101) is from the benches and not counted in a pass.
- The tests of the packages that import their own build can read a stale one (B1151).
- A `test:unit` that builds its package first would remove the stale-build risk of B1151.
- Gates:
  every package suite the changes reach ran once after the dists were rebuilt,
  and the files that failed were rerun alone;
  the first full runs found five failing cases in four files
  (the lines that list the models left out in the provider and stage tests,
  the claims test,
  and the consistency cache version with its key),
  and the first oxlint runs found findings in twelve packages.
  The whole source-scan suite ran twice at the end,
  because the first run found four modules with no test of their own,
  five unreached constants,
  a blankness test,
  a function example,
  ten leading articles,
  a locale ordering and a duplicated client;
  the second run is green.
- Slips in this round's commit messages:
  `5b97f0b90a` says 19 launches and 28 calls where the scan reads 20 files and about 25 calls;
  `9484424f35` says 7 failing cases where 6 failed;
  `7179618d23` says 389 insertions where the diff,
  with its new file,
  has 416;
  `92b8756112` says "1 new,
  3 changed" for 3 files in all;
  `1a92b7502e` says 9 TSDoc lines where 10 lost their article;
  `2cf8691409` says the drift tests read source,
  which the lint rule forbids and `7749a15d76` reverts (B1151);
  `594d9d1262` carries stray backslash-escaped quotes;
  `ef4161e1d2` takes "failed at its second case" from the brief;
  the five findings of B1145 were reworded before their test was written,
  the drift case of B1148 was not shown to fail,
  and the red cases of B1142 share commits with the stubs they need.
