# Audit ledger: the reach of the re-ask and the repair of another model

The owner's answers and the lead's rulings of 2026-10-31 on the re-check of the re-ask of another model
([Audit ledger:
reasoning runaways and the re-ask of another model](audit-ledger-reasoning-runaways-and-the-re-ask-of-another-model.md)):
the pool of each role (B1100),
how rarely another model is asked (B1101),
a revision by another model recorded as its own candidate (B1102),
the measurement that must measure the candidate and the picture re-ask (B1103),
the callers that named the wrong role (B1104),
the models left out written once (B1105),
the reader of the role a call names (B1106),
the checker that wrote the text it checks (B1107),
the figures that ledger quoted that were stale (B1108),
and what stays open (B1109).

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B1100: a re-ask drew its candidates by holds, which is not the list of a checker, an editor or a refiner

Status:
fixed on 2026-10-31 (UTC) in `8ef2c88ddf` (shared contract;
red `30ef4b9826`),
`9ad3359d40` (provider;
red `7b5b94cb0d`),
`bf2f906381` (run;
red `eb3ad46543`),
`37f2edb7d7` (repair;
red `c5dcf86d4f`)
and `570f2f39ed` (bench,
which holds the editor and refiner pools to the editor calibration's chairs).
The lead's ruling of 2026-10-31,
following from "Honour holds (Recommended)":
the alternates of the checker,
editor and refiner seats come only from the measured list of that role,
the checker order,
the editor calibration's editors and the refiners,
as `run-config.ts` and `run-seats.ts` derive them.
A model the measurements put out of a role is never its alternate.
The pool of each role is derived from the source its run bench comes from,
with no hand-transcribed table,
and a test fails if a role's pool and its bench derivation drift apart.

What the evidence was.
B1074 filed the editors and the refiners under the `roster` role,
whose holds are the cull and `judge-unmeasured`,
so a runaway editor could be re-asked of any roster model,
among them the two Gemma sizes the checker benchmark measured out of the checker role,
and a runaway checker of any model its card does not hold out of the cull alone.
The checker role honoured the cull and nothing else,
since no card says which models the checker benchmark measured in.

What it does now:

- The shared contract names nine roles (`seat-role.ts`):
  the seven of B1074 and `editor` and `refiner`.
  `rolePoolOf` gives the pool of a role:
  the roster less the models the holds of the role keep out,
  and for `checker` the measured order itself.
  The editor and refiner pools are the writers' holds,
  which is what the editor calibration seats in those chairs (`editor-calibrate-lane.ts`).
- The checker order moved into the shared contract as `MEASURED_CHECKER_ORDER`,
  and `RUN_CHECKER_ORDER` in `run-config.ts` is that list by identity and no copy.
  Its long note stays on `RUN_CHECKER_ORDER`.
- `alternateModelsFor` draws its candidates from the pool of the seat's role.
  A model outside the pool is left out with the hold that keeps it out,
  or with `not on the measured list of the <role> role` where no hold does.
  The editors (`repair/src/editor-gather.ts`) and the refiners (`repair/src/refine-stage.ts`) name their own roles.
- The drift tests:
  `run/src/corpus-run/seat-role-pool.unit.test.ts` holds the pool of the roster,
  translator,
  writer,
  wide-judge,
  late-judge and reader roles to `RUN_ROSTER`,
  `RUN_TRANSLATORS`,
  `RUN_WRITERS`,
  `RUN_WIDE_SEATS`,
  `RUN_LATE_JUDGES` and `RUN_READER_MODELS`,
  the checker pool to `RUN_CHECKER_ORDER` by identity,
  and the editor and refiner pools to the writers;
  `bench/src/corpus-run/editor-lane-pool.unit.test.ts` holds the editor and refiner pools to `EDITOR_LANE_EDITORS` and
  `EDITOR_LANE_REFINERS` but for the Claude route.
  Of those,
  only the identity of the checker order was red;
  the rest pass because the pools already agree with the benches,
  and they are there to fail when one moves.

How the pool reaches the seat.
The lead's ruling said the caller hands the pool in on `ReaskSeat` where the provider cannot import the run package.
The provider and the shared contract cannot import it,
and handing a pool in would thread it through `RepairModels` (about seventy files name `checkerModelIds`)
and every translate,
consolidate and consistency config.
The pool is therefore derived in the shared contract from the cards and the one list that is not a card fact,
the checker order,
which moved there so the run reads it from the contract.
A caller cannot forget it,
since the role is required where a bench is asked (B1079) and the pool follows from the role.
This departs from the ruling's wording;
open to the lead's veto (B1109).

Cache versions:
none moves.
No stored key or record shape changes,
and the pools alter only which other model a runaway seat is re-asked of.

Recurrence:
`mistake-prevention.md`,
"Cached decisions";
a role's list of models is read from the same source as its bench and a test fails when the two part.

## B1101: how rarely another model is asked, and where

Status:
decided on 2026-10-31.
The owner's answer "Accept the narrow reach (Recommended)":
on most benches no other model is free,
so the runaway falls back to M nudged;
keep the code and say in the ledger how rarely another model is asked and where it is.
B1072 and B1079 said the fall back "never fires" in one reading and "fires" in another;
this entry is the account.

The reading,
from the bench derivation and not from a run.
A re-ask leaves out the model that ran away,
the Claude route,
the decision-only seat,
the bench of the question and every model another seat of the question took (B1076,
B1080),
and draws from the pool of the role (B1100).

Where another model is asked:

- The editors:
  the pool is the writers and the bench is four editors.
- The refiners:
  the pool is the writers and the bench is three refiners.
- The checkers:
  the pool is the measured order of five and the bench is the first three of it,
  so the models of the order that are not on the bench (the Claude card is never asked).
- The picture readers of a small bench:
  a picture read by two readers leaves the other models that read pictures and are not held out of the reader role.
- Translate repair:
  the authors that were heard are the bench and the translators the cards allow are the pool,
  so the translators that wrote no candidate for the slice.

Where none is asked,
and the fall back to the model that ran away is the ordinary path:

- The roster-wide seats (pairing,
  admission,
  archive review).
- The writers of the consolidation,
  the translators of the translate lane,
  the wide seats (critics,
  panel,
  select judges) and the late judges (contest,
  gate,
  consolidation slate).
- A picture read by every model that reads pictures.

The bench of each of those is the whole pool of its role,
so no model is left to ask.
The exception is a seat the run withheld from the bench of a reading (`judgeSeatsFor`):
the gather's bench excludes it and the pool does not,
so it is asked.
How often that happens in a pass was not counted.

The cost of the narrow reach:
on the benches in the second list a reasoning runaway is re-asked of the same model on another provider,
nudged,
which B1072 found repeats the runaway in 6 of 7 direct DeepSeek calls.
The owner accepted that;
the census order (B1075) matters only on the benches in the first list.

Cache versions:
none moves.

## B1102: a runaway author in translate repair was re-asked of another model whose answer was recorded as the author's

Status:
fixed on 2026-10-31 (UTC) in `109bcbe53f` (provider;
red `47d351f737`),
`d61c5f8c97` (stage;
reds `002341b977` and `b54778147b`),
`3b59d5cee6` (translate-floor;
red `9da6ef031f`),
`8b9fa695f0` (translate-floor,
the export and the own test of the follow-up turn),
`e97e877f6b` (translate-stage)
and `9cda87bdcc` (consolidate).
The owner's answer "Other model,
own name":
in translate repair (`translate-floor/src/translate-repair.ts`),
a runaway author is re-asked of another translator,
but that model's revision is recorded as its own new candidate under its own model id,
with its own-text half weight on the slate.
The author's original candidate stays.
The prompt to the other model must not call the text "your translation";
it says whose candidate it revises.

What the evidence was.
The follow-up turn sends the author its own candidate as an assistant turn and the findings as a user turn,
in words that address the author (`your translation`).
When the author's answer ran away,
the provider re-ask sent that conversation to another model and recorded the answer as the author's voice,
so a text the author never wrote went on the slate under its name,
and the other model was told the text was its own.

What it does now:

- `ReaskSeat.otherModelMessages` hands a model other than the one asked its own conversation.
  The provider uses it for the ask of another model and keeps the question and the nudge for the fall back.
  The usable answer of such a model carries `answeredBy`
  (`ChatJsonOutcome` ok and `StageVoice` heard).
- `buildAnotherTranslatorRepairMessages` builds that conversation:
  the prior messages,
  then one user turn with the translation as plain text under "THE TRANSLATION TO REVISE",
  the findings,
  and rules that speak of another translator.
  Findings are written to the author in the second person;
  "your" is made the article for the other model.
  Other second-person words in a finding stay (B1109).
- A revision that `answeredBy` names is recorded as a candidate of that model,
  appended after the author's,
  with the finding `translate-repair-revised-by-another (<author> -> <model>)`.
  The author's candidate stays,
  unless it respells a contributor name,
  which no defence keeps.
  The model's ballot for its own text weighs half as for any text its author judges,
  since the candidate carries its id.
  Candidates of a model the roster does not list sort after the listed ones.
- The role and the record of models asked come from the caller:
  `repairInvalidCandidates` takes `role` and `reaskClaims`,
  the translate stage passes `translator` and the consolidation `writer`,
  and each makes one record for its slice that its gather (`gatherStageVoices`,
  parameter `reaskClaims`)
  and its repair share,
  so a model that took a runaway seat in the produce gather gives no second voice in the repair.
  Before this it named `translator` for every caller and made a record of its own.
- The follow-up turn moved to `translate-repair-ask.ts` (the file passed its line limit).

Tests:
`translate-repair.unit.test.ts` (two added:
the revision recorded as the other model's with the author's original beside it,
and the seat that carries the role,
the record,
the bench and a conversation without "your" or a turn of its own),
`translate-repair-ask.unit.test.ts` (new),
`reask-role.unit.test.ts` of translate-floor (drives the repair and checks the role,
the bench and the record),
`nudged-reask.unit.test.ts` (the conversation for another model,
`answeredBy`),
`stage-call-runaway.unit.test.ts`,
`stage-quorum-claims.unit.test.ts`
and the role cases of translate-stage and consolidate (the role and the one record at both steps).

Checked and found to hold:

- The outcome keeps `answeredBy` through the production chain.
  `promptUniqueClient` and `seatTallyClient` return what `reaskElsewhereNudged` returned,
  and a case of `prompt-uniqueness-client.unit.test.ts` reads `answeredBy` through the wrapper.
- The roster check of the translate stage (`assertJudgeableProducerRoster`) reads the bench of translators,
  not the candidates,
  so a candidate of a model off the bench does not end the slate.
- The half weight of a self-ballot reads the producer ids of each candidate (`stakesByIndexOf`),
  so a candidate of a model off the bench weighs half for that model as a judge.
- The slice key does not enumerate sheet builders:
  `translate-document-contract.ts` names the repair wire's resolutions in prose,
  and this change leaves that enum and the author's follow-up wording as they were.

Cache versions:
none moves.
The stored translate and consolidate records hold candidates by wording and producer,
and a candidate under a model the roster of the slate did not list has the shape of any other.
The prompt of the other model is new text,
so its digest differs from that model's own first ask and is never replayed from it.

## B1103: a measurement of one reader scored a rival's reading, and the picture re-ask had four smaller faults

Status:
fixed on 2026-10-31 (UTC) in `a2303eaf45` (slice-reading;
red `63e672410b`),
with the bench cases in `570f2f39ed` and `245c1e5ac1`.
The lead's ruling:
measurement calls keep the same-model re-ask.
`reader-measure-run.ts` calls `readPastRefusal` with no bench and no claims,
so a reading cut by the length limit was re-asked of another reader
and the rival's reading was scored as the candidate's.
A measurement must measure the candidate.

What it does now:

- `readPastRefusal` re-asks a cut reading of another reader only when the caller names the bench.
  A caller that names none (the measurement) asks the reader itself.
  `readImagePair` names the readers of the picture.
  The bench test of `reader-measure-run.ts` pins that it names neither bench nor record.
- When the reading of the other model after a cut is a refusal,
  the loop logged "M declined" and asked M again.
  It now logs the model that declined and asks that model,
  as a refusal is asked of the model that gave it.
- The picture path tested `reply.text === ''` where the chat path uses `rendersAsNothing`;
  both use `rendersAsNothing` now,
  so a whitespace-only cut reply is a runaway on both.
- The models left out and the order to ask them in are written once before the first re-ask (B1105).

Cache versions:
none moves.

## B1104: three callers named the role of a bench they did not hand in

Status:
fixed on 2026-10-31 (UTC) in `9cda87bdcc` (consolidate),
`bd8909af19` (corpus-pass;
red `8377c5b252`)
and `3b59d5cee6` (translate-floor),
with the cases red first in consolidate (`671c4f3d4f`) and corpus-pass.
The lead's ruling:
each call names the role of the bench it is handed.

- `consolidation-polish-round.ts` passed `judgeRole: 'late-judge'` with a comment about the late judges,
  but its bench is `config.judgeModelIds`,
  which is `selectJudges`,
  the wide seats.
  It names `wide-judge` and the comment says so.
- `pass-consistency-gather.ts` fixed `late-judge`,
  and `askCarryWriters` hands it `seats.translators`.
  The gather takes `role` as a required parameter;
  the choice and the run-off of a page name `late-judge` and the writers of a carry name `translator`.
- `translate-repair.ts` fixed `translator`;
  `consolidate-produce.ts` calls it with the writers,
  whose role is `writer`.
  It takes the role from the caller (B1102).

B1079's list of roles filed the polish slate and the consistency gather under `late-judge`;
it is corrected there.

Cache versions:
none moves.

## B1105: the models left out were written inside the loop that asks the others

Status:
fixed on 2026-10-31 (UTC) in `9ad3359d40` (provider;
red `7b5b94cb0d`),
`95bd9e9bc2` (stage;
red `4af8224ad8`)
and `a2303eaf45` (slice-reading).
The provider re-ask,
the recovery round and the picture re-ask wrote each model left out and why only on the line of the ask of an alternate,
so a list that left no alternate wrote none,
and an alternate with no provider wrote the same left-out text again on the line of the next.
Each now writes one line before the loop:
the model that ran away,
what each model left out was left out for,
and the order to ask the others in or `none`.
Each ask has a line of its own after it.
The warning of the fall back is unchanged.

Cache versions:
none moves.

## B1106: the reader of the role a call names read the formatter's layout and missed what a spread does

Status:
fixed on 2026-10-31 (UTC) in `4c0622ac14`,
`9945b54e79`,
`eaef656359`,
`02b861fb8c` and `be93348926` (test-fixture),
each after its red case,
with the cases of the callers named under the faults.
`rolesNamedAtCalls` found a call by the text `await callee` and read the role from the line that begins `role` at the
depth the formatter gives,
so it missed a call without `await`,
a call on one line,
a spread after the role that can replace it,
a duplicate key and a call whose argument is a variable.

What it does now:
it cuts the source into tokens with comments dropped and reads the argument object by its brackets.
It reports `<role> then a spread` where a variable,
a call or an object that names the role is spread after the role
(a spread of an inline object that names other keys,
as the conditional spreads of optional arguments are,
does not count),
`duplicate role: a, b`,
and `not an object literal`.
It takes another key than `role` (`judgeRole`) and a dotted key into an object the call passes (`judging.role`),
and `roleNamedInLiteral` reads the role of an object literal kept in a variable.

The stricter reader found two real faults in the callers:
the role of the refiners' slate call (`refine-stage.ts`) and of the argument object of the consolidation slate
(`consolidate-settle-judge.ts`) came before a spread of an opaque context that could replace it,
and the window trial's slate call the same.
Each now puts the role last.

The cases for the call sites that had none:
`stage/src/pair-sections-stage.ts` (roster),
`consolidate-settle-judge.ts` (late-judge),
`consolidation-polish-round.ts` (wide-judge),
`refine-slice-settle.ts` (wide-judge),
`translate-stage-repair.ts` (wide-judge),
`window-trial-slice.ts` (roster)
and `run/src/judge-fidelity.ts` (roster).
The text-only role test of translate-floor is replaced by one that drives the repair
and checks the bench and the record.
The token cutting lives in `reask-role-tokens.test-fixture.ts`,
which reads a template literal to its next backtick and so does not follow a backtick inside a `${}` hole;
the call sites it reads hold none.

Cache versions:
none moves.

## B1107: a checker answered by a model that wrote the text it checks

Status:
pinned on 2026-10-31 (UTC) in `eb3ad46543` (run).
The lead asked to check,
once the alternates of a checker come only from the checker list,
that no editor can check its own text at full weight in `repair-edit-stages.ts`.
That file computes `wroteTheText` from the model of the voice,
which is the checker's seat,
and a re-ask records the answer of another model as that seat's voice.

The reading:
the pool of the checker role is `RUN_CHECKER_ORDER`,
and no model of it is an editor or a refiner of `RUN_MODELS`
(the note of `RUN_MODELS` says so and `run-seats.unit.test.ts` holds Haiku 5.5 out of the editors).
`seat-role-pool.unit.test.ts` pins it for the whole order,
so an answer recorded as a checker's voice never comes from a model that wrote the text.
The voice does not carry which model answered into `wroteTheText`;
if a later seating puts a checker-order model among the editors,
the pin fails and the voice has to carry `answeredBy` into that computation.

Cache versions:
none moves.

## B1108: figures and words this ledger quoted that were stale

Status:
fixed on 2026-10-31 (UTC) in the ledger.

- B1074 quoted eight slice-cache versions as of 2026-10-30.
  Read again from the code on 2026-10-31:
  pairing 3,
  consolidate 43,
  lane contest 18,
  translate slice 36,
  repair slice 50,
  refine 19,
  consistency 9
  and page title 9
  (`CONSOLIDATE_CACHE_VERSION`,
  `LANE_CONTEST_CACHE_VERSION`,
  `TRANSLATE_SLICE_CACHE_VERSION`,
  `SLICE_CACHE_VERSION`,
  `REFINE_CACHE_VERSION`,
  `CONSISTENCY_CACHE_VERSION`,
  `PAGE_TITLE_CACHE_VERSION` and `PAIRING_CACHE_VERSION`).
- B1074 said the gather's role had the default `roster`;
  it has none since B1079.
- B1074 said "two fixtures and one case" scripted an empty length stop as unreadable;
  it is one fixture (`cf93bef02a`) and one case (`121719e570`) beside the two cases of `stage-quorum.unit.test.ts`.
- B1079's roles listed the polish slate and the consistency gather under `late-judge` (B1104),
  filed the editors and the refiners under `roster` (B1100),
  and said the fall back "never fires" (B1101).
- Three lines of this ledger passed 120 columns and are wrapped.
- `MIN_CALLS_TO_MEASURE` in the provider repeated the floor of `completion-cap.ts`;
  `reask-runaway-readings.ts` reads `MIN_PROVIDER_CALLS` and the second constant is gone.
- `chat-contract.ts` said an absent `reaskSeat` is the roster-wide role with an empty bench;
  it means no other model is asked (B1081),
  and the comment says so.

## B1109: what stays open from this round

Status:
open,
for the lead and the owner.

- The pool of a role comes from the shared contract and not from a hand on `ReaskSeat` (B1100).
  If the lead wants the caller to hand it in,
  `ReaskSeat` gains a required list and every caller of a gather passes it.
- The findings handed to another translator are written to the author in the second person.
  "your" becomes the article;
  other second-person words stay (`you`),
  and the guard that each finding speaks of the text and not to its author is a gap.
- The model that answered a checker's runaway is not carried into `wroteTheText` (B1107).
- A model that took a runaway seat of the produce gather is in the shared record for the repair,
  and a model another step of the question asked outside both is not.
- The reading of how often another model is asked (B1101) is from the benches and not counted in a pass.
- The commit messages of this round carry counts that are not all exact:
  `30ef4b9826` says three cases added where four were,
  `4c0622ac14` says five earlier cases where there were three before it,
  `bd8909af19` says five source files where four,
  and `9ad3359d40` counts passing lines of the suite output and not cases.
