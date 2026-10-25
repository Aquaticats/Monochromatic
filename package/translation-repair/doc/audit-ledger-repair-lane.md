# Audit ledger: the repair lane

Findings in the repair lane,
L1 to L13 and L15.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## L1: every checker verdict rests on two voices

Status:
fixed:
the ballot floor with guard `8c4fc28e1`,
fix `30e66051e`;
the measured bench with guard `f037a3eae`,
fix `f10de5198`.
Qwen3.8-27B is seated as a checker with no provider serving it,
so every reading is 2 of 3;
a 1 to 1 split is the most common outcome,
`regressed` never fired though checkers voted `worse` 55 times,
and XingZ6014 slice 87 resolved an issue on one ballot,
which shipped.
Measured on 2026-09-27:
759 of 8,788 recorded checker readings over 403 run directories resolved on one cast ballot,
756 inside a selected patch.
The checker benchmark (85 settled fixes,
and the same 85 issues against the unchanged archive text)
scored the bench that dominant reading seated,
`gemma-4-26b-a4b-it` and `google.gemma-4-e2b` with Qwen3.8-27B unserved,
at 35 fixes resolved and 9 unchanged texts wrongly resolved;
the all-wet bench scored 82 and 4,
and every bench of three from the measured order 81 or 82 and 2 to 5.
Fixed two ways:
one cast ballot resolves nothing (`MIN_RESOLUTION_BALLOTS`,
`SLICE_CACHE_VERSION` 32,
`REFINE_CACHE_VERSION` 5),
and the bench is the first three of `RUN_CHECKER_ORDER` a wet provider serves,
padded with unserved ranked seats only while fewer than three are served.
`regressed` kept no floor when this was written:
it only ranks a candidate and never ships text.
Superseded on 2026-10-25 (B917):
it ranks third in the candidate selection,
so it needs two or more ballots naming `worse`.

## L2: the archive-dispute stand-in is the archive's own wording whenever the repair lost

Status:
fixed with guard `23cbbdaaa` (owner answer 2026-09-27,
"No eligible standing");
sixteenth addendum of `doc/decision/translation-repair-ineligible-standing.md`,
read issue by issue
(the stand-in stands only where the checkers confirmed every disputing issue resolved:
1 of 87 measured).
60 of 87 disputed slices;
the translate lane then keeps it as lane agreement and no contest runs;
a withdrawn text also becomes a stand-in.

## L3: edits the checkers did not confirm ship inside a selected patch

Status:
fixed in `69c149471` (guards `737ebf9cc`,
`84255894a`;
mutation checked with a control,
five mutants caught),
on the owner's ruling of 2026-09-28,
"Revert worse-voted,
recheck" (`design-commitments.md`);
measured 2026-09-28.
An edit whose issue the checkers did not confirm and at least one voted worse is stripped by envelope,
and the reduced patch is proved once more (`repair-worse-strip.ts`):
58 issues in 31 patches over every run,
15 in 6 of 51 on TianqiChen666,
where the owner question had said 40,
counting only ties and the worse majority.
Rides inside the repair cache version 33 with a written account.
TianqiChen66616 slice 3 shipped "it left a trace of her turned to ash" (one ballot worse),
the patch having won on one resolved minor omission.
Across every run,
345 of 2,148 selected patches carried at least one edit whose issue the checkers did not confirm,
691 such issues:
1 on a worse majority,
39 on a fixed and worse tie,
651 on not-fixed;
on the TianqiChen666 runs 18 of 51,
49 issues,
none on a worse majority.
A worse-majority floor would therefore catch almost nothing.
No ruling covers it:
the owner's L2 answer rules on the standing after a lost repair,
not on a winning patch's contents,
and `design-commitments.md` commits that every stage changing shipped text is audited,
which counts against reverting unconfirmed envelopes with no recheck of the composite.
Superseded on 2026-10-25 (B922),
owner answer "Two ballots for undo too":
an edit is stripped only where its issue's tally flags it `regressed`,
two or more ballots naming `worse` with the `worse` weight exceeding the `fixed` and `not-fixed` weight together,
and one `worse` ballot strips nothing.
Repair slice version 49 retires the records written before.

## L4: the editor preservation gate can never reject anything

Status:
fixed in `b5338610e` and `c4a4fbb62` (guard `01d8bfd4c`);
owner ruled 2026-09-28,
"Markup atoms" (`design-commitments.md`);
measured 2026-09-28.
Envelopes and licensed quotes are the same quotes;
`residualTokens` is 0 on 536 of 540 regions.
Replayed over 5,733 recorded repair regions,
the structural atoms an edit changed are mostly gains an omission fix restores
(footnote references,
link destinations,
tags);
the losses mix addition removals (footnotes and inline code on `accuracy/addition` claims)
with damage (a footnote lost on a quotation-mark claim,
MDX braces lost on mistranslation claims).
The damage this finding names (a gloss dropped,
a handle and a name replaced,
an attribution deleted)
is prose inside the licensed quote,
which no markup gate sees,
so what the gate should protect inside a quote is a design choice,
not a measurement.

`markup-atom-scan.ts` reads footnote references,
link destinations,
MDX expressions,
inline code and tags
(comments included) out of an envelope fragment in one left-to-right scan:
a code span is consumed whole,
a backslash escapes,
parentheses and braces nest,
and a construct the fragment edge cuts off yields no atom.
Footnote labels follow GFM's call tokenizer,
which refuses an empty label,
a bracket,
a space or a line ending.
`markup-atom-preservation.ts` compares the atoms as a multiset,
since an accuracy edit may move a reference to its clause;
an atom inside a quote an addition claim made may go,
claim by claim rather than issue by issue (`buildRemovableQuotes`).
`applyPatchOperations` refuses a loss as `preservation-lost-markup (<kinds>)`,
naming kinds,
never atom text.
The editor sheet's markup rule is built from `MARKUP_ATOM_SHEET_NAMES`,
a Record over every kind,
so the sheet names each kind the gate enforces;
before,
it named footnote markers only.
One `ADDITION_CATEGORY` in `issue-taxonomy.ts` replaces the private copies in `reference-attest-claims.ts` and `archive-dispute.ts`.

Reach,
replayed with the built gate over the same 5,733 regions (`l4-replay.mjs`):
24 refused,
none of them by the typography restoration;
15 regions that lose an atom pass on an addition quote.
Positive control:
39 regions lose an atom by plain set difference,
24 plus 15.
The refused regions by shape (`l4-refused-markup.mjs`,
markup only):
footnote references lost on quotation-mark claims (hulicaijia s34,
shihai4h s29),
both withdrawn;
an MDX expression dropped on a mistranslation claim (shihai4h s38,
shipped);
`DottedNumber` component props rewritten on number-format claims (XingZ60 s34,
s39,
s41;
three shipped);
inline code the source renders as a heading tag,
converted to that tag (XingZ60 s87 and s88,
two shipped);
a spelling fix inside a `PhotoScroll` prop,
whose captions are visible text (noname s9,
shipped);
and two footnote references swapped between clauses across two envelopes (yuki418330012 s6,
shipped).
An HTML comment translated or deleted on untranslated-text claims (XingZ60 s6,
six shipped) is invisible to readers either way.
One footnote lost on a link-convention and omission claim (shihai4h s33) was not selected;
the package defines no rule text for `policy/link-convention`,
so it stays unclassified.

Corrected the same day:
the first version of this entry called the `DottedNumber` rewrites the damage the ruling names,
reading claim category and tag shape without the source (M31).
Against the corpus source page (`l4-source-carried.mjs`,
pin a41fc607),
the archive's `DottedNumber` props are not in the source on all six regions,
and the edit's props are the source's own on five:
those edits restore the original's props.
Every refused atom was classified the same way:
the lost footnote references,
MDX expression and deleted comments are pure removals with nothing written in their place;
the inline code,
the `PhotoScroll` prop,
the comments rewritten and the `DottedNumber` props are markup the translation authored
(the source does not carry it),
which the edit re-marked,
into the source's own form where the source carries one;
and the two swapped footnote references are the source's,
each written by the other envelope's edit.

Mutation check (`l4-mutants.json`):
the gate off,
the addition licence ignored,
every claim licensing removal,
a set compare in place of the multiset,
escapes ignored,
a spaced footnote label accepted,
and the sheet dropping a kind are each caught;
the comment-wording control survives.
Cache:
rides inside repair version 33 with an account in `repair-slice-key.ts`.

Refined for quality the same day,
under the owner's standing directive (`design-commitments.md`):
guard `9df194059` and `550603cfe`,
fix `b07f8ac48`,
sheet `ab84cbfd7`.
`markupDelta` (`markup-atom-preservation.ts`) keeps a loss unexcused unless an addition quote licenses it,
or the translation authored the atom (the source does not carry it),
it is an expression,
inline code or tag,
and the edit wrote,
one for one,
an atom of the same kind or one the source carries in its place.
Footnote references and link destinations (`MARKUP_IDENTIFIER_KINDS`) have no such excuse:
the source often carries no footnote at all,
so every archive reference would read as authored.
`settleMarkupMoves` then reads the patch as a whole:
an unexcused loss another standing edit writes has survived;
the settlement is a fixed point,
since refusing an edit withdraws what it wrote.
It runs over the edits every per-edit gate passed (`apply-patch-markup.ts`),
and refusals stay in input order.
The gate reads the whole source document where the chunk has it (`documentSourceText`),
else the chunk's source,
and the replay this entry records measured only the document case.
`EditorStageResult.preservation` carries the gate,
and the L3 strip re-applies it to the kept edits,
since the kept side of a move whose writer was stripped has lost its atom.
The editor sheet's markup rule states the refined rule,
both lists built from `MARKUP_IDENTIFIER_KINDS`.
Reach,
replayed with the built functions per slice against the corpus page (`l4-replay-settled.mjs`):
7 of 5,733 refused,
the pure removals (three deleted comments,
the three footnote references,
the MDX expression),
against 24 under the ruling's wording;
41 regions have no corpus source page and were read as all-authored.
Mutation check (`l4b-mutants.json`):
no re-marking excuse,
identifiers re-markable,
copied markup re-markable,
same-kind pairing only,
no move settlement,
a single settlement round,
the strip skipping the gate,
the strip dropping its new refusals,
and the sheet lists inverted are each caught;
the comment-wording control survives.

## L5: wrong panel acceptances

Status:
fixed or closed,
item by item in this entry (2026-09-28).
Found in the audit:
glosses of works outside the panel's apparatus list;
a supplied object outside its forced-difference line;
footnote-carried attributions judged dropped;
an MDX editor comment deleted;
鲨鲨 (a plush shark) taken for a person;
a neutral "correctly renders" claim accepted and cut into an envelope.

### Neutral acceptances

Status:
fixed,
one fix in two parts,
both needed.
Measured over every artifact under the agent runs (`l5-neutral-census.mjs`,
`l5-neutral-kinds.mjs`,
`l5-neutral-affirm.mjs`):
41 of 9,532 accepted repair-lane issues had settled at neutral,
over 30 runs;
32 cut a region,
28 shipped an edit,
and 26 of those the checkers called resolved.
Eight were claims whose own summary called the rendering accurate or correct,
with no negation
(seven on hulicaijia,
one on Carena0442);
one of them,
on hulicaijia19,
shipped an edit.
Others were real small losses filed neutral:
the same noname omission of a nuance was filed neutral and fixed in six runs.
Neutral is the severity that asserts no defect (`issue-taxonomy.ts`),
and neither sheet defined any severity.

The tally part (`e87e353ae`,
guards `266a0891a`,
`aa7f4e360`,
`c53d5c717`):
an acceptance settled at neutral is held as needs-human,
with a `neutral-held-for-human (claim)` finding,
so it reaches no editor,
envelope,
checker ballot,
recheck round or archive dispute.
A single supporter re-grading to a real severity lifts the upper median out of neutral and keeps the acceptance.
Mutation checked with a control:
the severity test,
the returned status,
the dropped finding,
and a finding raised for any vote (caught only after `aa7f4e360` added the rejected case).

The sheet part (`fb6c06cca`,
`severity-scale.ts`,
guard `925b85a8f`;
cache account `ef20e7978`,
`SLICE_CACHE_VERSION` 34):
both sheets define the scale,
minor,
major and critical after MQM
(<https://www.themqm.org/guidance/values-and-scores/>,
read 2026-09-28)
and neutral in the pipeline's own words,
since MQM's neutral marks a spot where "a different solution is warranted";
the critic is told a claim naming nothing wrong is no issue;
the panel votes such a claim unsupported,
with interpretive ambiguity and suspected source errors kept outside that rule;
and a supporter lifts a real defect filed neutral to at least minor.
Without the sheet part the tally part alone would regress the real small fixes filed neutral.
Mutation checked with a control:
dropping either sheet's scale,
either rule,
the re-grade rule,
a scale line or the carve-out each fails a guard.

Expected but unmeasured (QAB):
defining major and critical may move how many accuracy issues the panel settles at major or worse,
which the archive dispute reads;
the next run's artifacts measure it against the `l5-neutral-census.mjs` census.
Because an accepted addition can no longer settle at neutral,
the dispute rule's "addition at any severity" now means any severity from minor up in effect
(`design-commitments.md` records this as a quality refinement of the 2026-09-26 ruling).
The cross-check's needs-human now also holds a supported majority at neutral,
so its precision over accepted issues reads higher by construction on runs from this version.

### Footnote definitions outside the slice

Status:
premise refuted and the window change reverted (`80a18dd53`);
fixed in the note rule's framing (`868e848d3`,
guard `c611b525e` corrected in `657e9db35`,
mutation checked;
rides inside repair 34 and refine 5,
`e8d906387`).
`DECLARED_IDENTITY_RULES` now says a footnote marker in either document points to the note line with that label
and what the note says stands at that marker,
so content the ORIGINAL states inline and the TRANSLATION's own note
carries at its marker is not omitted,
nor the reverse;
the note lines stay vocabulary evidence otherwise.
The red guard's editor case first failed for a second reason (the editor states these rules only when the page
declares something,
and the fixture declared nothing),
so it would have stayed red after the fix;
caught on the fix run.
The translate lane's sheets carry the note lines with no framing rule at all,
and no evidence yet says they misread them,
so the clause stays on the repair sheets.
The audit said no sheet shows footnote definitions,
so a slice citing `[^5]` read as if the attribution
the note carries were dropped (sh2 slices 33 and 37).
That was never checked against a rendered sheet,
and it is false:
`entry-notes.ts` has carried every footnote definition of both documents,
as "ORIGINAL note" and "ARCHIVE note" lines of the DECLARED NAMES block,
to every sheet since `12ed82cee` (2026-09-02).
shihai4h ran on 2026-09-26;
its original has no footnotes and the archive's ten notes sit in no slice,
and the shipped extractor turns all ten,
notes 5 and 7 included,
into ARCHIVE note lines
(`l5-sh2-notes.mjs`;
the artifact does not store the identity context,
so this is the extractor rerun,
not the sheet read back).

What the notes lacked is standing,
not presence.
`DECLARED_IDENTITY_RULES` tells every sheet the note lines "establish vocabulary and titles for the terms they name
and nothing else",
and that the block "is evidence about naming ONLY";
nothing says what a footnote marker carries.
That framing is inferred,
not measured,
to be why the sh2 claims were accepted:
those runs predate stored ballot reasons,
so what the panel thought cannot be read.
The census still measures how many slices the framing matters for
(`l5-footnote-census.mjs`,
every run,
repeats included):
342 of 6,261 slices cite a source footnote defined outside the slice,
171 a target one;
the panel accepted 318 issues on those slices,
74 of them omission or addition claims,
and 85 name a footnote,
attribution,
credit,
citation or translator,
or quote a `[^` marker.
Those 85 stayed accepted with the notes on the sheet,
which fits the framing story and fits some of them being legitimate;
they are not the fix's yield.

The reverted change (`b0dd42341` to `13b936c90`,
seven commits over 21 files) ended each slice's fidelity window
with the definitions it cites from outside the window,
first from the slices,
then from each whole document.
Its wire guard passed with every lane's documents replaced by the empty string or by the other side's text,
because the notes reached the sheets through the DECLARED NAMES block regardless;
that is what exposed the premise (M32).

### The other sub-items

Each checked on a rendered sheet first (M32),
with `l5-obligatory-render.mjs` and `l5-handle-render.mjs`.

The panel's obligatory differences (Tq16 slice 20,
an addition claim against "of me" English had to state):
fixed in `64a4bf63a` (guard `1c3d0cf36`,
mutation checked).
Rendered sheets showed the critic carried a whole obligatory-difference block and the panel none of it,
only a line on conjunctions,
connectives,
pronouns and small words.
`obligatory-differences.ts` states the block once in neutral voice,
now naming a possessor and a connective too,
and both sheets carry it with a line in their own voice;
it replaces the panel's narrower line.
Rides inside repair 34,
no slice-cache file newer than the last bump remaining.

A handle's literal gloss claimed as an addition (Cu11 slice 1):
covered by today's sheets.
Both the critic and the panel render the house rule giving a romanized handle its literal meaning in parentheses
on first mention,
and the apparatus kinds naming a gloss of a name as accurate apparatus;
the run predates S5,
which put the shared apparatus list on the panel.

A heading rendered "Ann" (XZ14 slice 49):
not a wrong acceptance by any rule the owner has set.
The original heading has the shape "label:
name" and the archive renders every section heading as the name alone;
the accepted claims argue the label was dropped,
and the owner's ruling on headings is "judges decide".
The patch lost at the checkers,
so the archive heading shipped.

鲨鲨,
a plush shark,
taken for a person (hu31 slice 39):
no sheet change.
The accepted claim (settled major,
a mistranslation claim calling 鲨鲨 a person) lost at the checkers,
so the page kept the archive's wording;
the run predates stored panel reasons,
and the reason-before-vote change (owner,
2026-09-27) is the structural answer to a panel voting on a misreading.

Closed elsewhere:
Tq16 slice 0's gloss of a work (S5's `APPARATUS_KINDS`),
sh2 slice 38's MDX comment (L4).

## L6: the lane contest runs on insertion slices the repair lane does not apply to

Status:
open;
measured and designed 2026-09-28,
deferred past the TianqiChen666 launch,
since no TianqiChen666 run contested such a slice (its archive has every passage).
31 wasted contests on XingZ6014.
Across the 240 contested artifacts under the agent runs,
952 contests ran on slices whose repair outcome is `not-applicable` (every one an archive-absent insertion):
931 went to the translate lane,
5 settled neither and were consolidated,
and 16 missed quorum,
where the consolidation found no standing text and nothing shipped,
so those 16 inserted passages are missing from their pages.
Excluding such rows from the contest would also skip the consolidation and polish,
which run only on contested slices,
and those are the passages no human translated.
Design:
from a new artifact generation,
such a row takes a deterministic `sole-lane` verdict naming the lane that applies,
with no roster asked;
the consolidation treats it as a win for that lane,
the would-ship reader ships that lane's wording,
and a reader of an older generation reads the contests it recorded.

## L7: the lane contest is shown probe claims about a patch that lost

Status:
fixed in `3bed8241d` (guard `11c1e92d8`;
mutation checked with a control).
`damageClaimLinesBySlice` read every chunk's accuracy probe,
so XingZ6014 slice 3 showed the judges damage quoting "she came out as trans"
that the repair candidate,
the archive after a lost patch,
never carried.
The accuracy repair's claims now need `accuracyPatchSelected`;
the naturalness rewrite's stand either way.
Rides inside `LANE_CONTEST_CACHE_VERSION` 6.

## L8: a heard ballot with no usable verdict still counts toward quorum

Status:
fixed in `abfc69393` (guard `79b1db6b0`,
which also stopped the two stage tests pinning the defect:
each expected three heard voices from ballots that voted only on a claim the sheet never showed).
The panel and checker gathers validated with the wire guards alone,
which accept an empty ballot,
one voting only off the sheet and one whose only vote is no vote,
so each counted as heard and could close the round
(TianqiChen66616 slice 3,
a claim in needs-human on two votes beside a ballot voting "minor";
XingZ6014 slice 87,
an issue resolved on the one other ballot).
Now such a ballot is unreadable and the recovery round re-asks its seat;
a ballot usable on some claims is still heard,
since the artifacts record 388 missing verdicts,
142 unknown votes and 104 off-sheet numbers inside otherwise usable ballots,
and requiring whole ballots
would discard those votes.
Mutation checked with a control:
restoring either wire guard alone fails its stage test.
Rides inside `SLICE_CACHE_VERSION` 33 and `REFINE_CACHE_VERSION` 5.

## L9: the dispute rule reads the critic's filed severity, not the adjudicated one

Status:
fixed in `6a0f68cec`,
guard `b15ba5464`
(owner answer 2026-09-27,
"Adjudicated";
seventeenth addendum of `doc/decision/translation-repair-ineligible-standing.md`).

## L10: the attestation screen only looks at addition claims

Status:
closed as designed,
2026-09-28;
measured,
no change.
What a cited reference can answer is "the original never states this",
which is an addition claim;
a mistranslation or omission claim on an attested detail can still be right (the archive may word the detail wrongly),
and the panel sees the cited references for every claim it hears,
so the screen is a shortcut for one question,
not the only path references take.
Measured over every artifact (`l10-attested-others.mjs`):
the screen fired on two attested archive quotes in all,
and no claim of any other category touched either of them.
A lower bound,
since only quotes an addition claim hit leave a finding,
and a small sample;
the same script rerun on later runs reopens this if other categories start filing on attested details.

## L11: refinement on a slice whose patch lost gets no recheck

Status:
fixed in `462c514ee` (guard `35272d1cd`;
class one hundred eight's open half,
with H4);
owner ruled 2026-09-28,
"Recheck the rewrite" (`design-commitments.md`);
measured 2026-09-28.
Across every run,
1,218 of 2,144 refined slices were rewrites of the archive after the accuracy patch lost,
457 of them on slices with panel-accepted issues the rewrite was never shown,
and none had a checker round;
on the TianqiChen666 runs 106 of 125,
75 with accepted issues.

The retention recheck in `refine-slice-settle.ts` now rules on every accepted issue `T1` leaves open beside the confirmed ones,
and rolls the whole slice back when a confirmed issue is no longer resolved or an open one drew at least one worse ballot,
the threshold the owner ruled the same day for a patch's unconfirmed edits (L3).
A `fixed` ballot on an open issue credits nothing:
the round is a rollback gate.
Rejected and needs-human issues buy no round.
The introduced-defect probe already ran on every kept rewrite,
and its comment claiming every rewritten slice had its issues repaired is corrected.
Its role was first left as in the accuracy lane,
deciding nothing directly;
the owner's answer to that question (2026-09-28) made it a quality call,
recorded under "The probe rolls a rewrite back".
Scope beyond the question's wording:
an accepted issue a winning patch left open is checked by the same rule,
since the rewrite was never shown it either;
the ruling's mechanism,
not a new one.
Mutation check (`l11-mutants.json`):
leaving the open issues out of the round,
never rolling back on them,
a worse majority in place of one ballot,
dropping the status filter,
and crediting a fixed ballot are each caught;
the comment-wording control survives.
Cache:
rides inside refine version 5 with an account in `refine-slice-key.ts`.
Superseded on 2026-10-25 (B922),
owner answer "Two ballots for undo too":
the rollback on an open issue reads the same `regressed` flag as the strip of L3,
two or more ballots naming `worse` with the `worse` weight exceeding the rest,
and one `worse` ballot rolls nothing back.
Refine version 19 retires the records written before.

**The probe rolls a rewrite back**,
decided for quality under the owner's standing directive
(guard `9e01c7133`,
fix `d41ad44c4`).
Measured over every run (`l11-probe-census.mjs`):
175 of 2,144 kept rewrites carried a claim the screen admitted
(151 added damage,
25 removal),
against 409 of 3,081 accuracy reports;
only 9 carried two or more,
so requiring agreement would catch almost none.
`corroborated` in the report means the differential bore the quote out,
one prober's claim sufficing;
the `ClaimTotals` comment saying a second prober confirmed it was wrong and is corrected.
The one graded reading of flagged regions (`doc/planning/translation-repair-roster-calibration-2026-09-01.md`,
2026-09-03) found six of ten true,
three false and one borderline,
so rolling back reverts roughly twice as many damaged rewrites as fluent ones,
and what comes back is text a checker round or the archive already stood behind.
A rewrite with any admitted claim now keeps the text before it,
with `refine-rolled-back-by-probe (<added> added-damage and <dropped> removal claims ...)` in the findings;
its report is not attached,
because the lane contest reads `refinementDefects` as evidence against the text that ships.
The probe module's header no longer says nothing reads the report.
Over an accuracy patch the probe still decides nothing directly:
a rollback there would discard edits fixing panel-confirmed defects on the same six-in-ten signal.
Mutation check (`l11b-mutants.json`):
never rolling back,
ignoring added-damage claims,
ignoring removal claims,
and attaching the report on a rollback are each caught;
the comment-wording control survives.
Cache:
rides inside refine version 5 with an account in `refine-slice-key.ts`.

## L12: logging gaps in the repair lane

Status:
fixed,
each part guarded and mutation-checked.
A11 put the slice on the critic,
editor,
checker,
select and probe lines.
What it left,
and what the audit found beside it:
the refinement phase ran slices side by side with no slice on its lines;
panel and checker ballot irregularities reached the findings and never the log;
assembly's document-scale damage checks named no slice;
the settlement line said "unchanged" for three different settlements;
and the dispute line blamed the checkers for every refused stand-in.

- Refinement:
  `4c714af90` (guard `ca684d49e`).
    Each slice refines under `inSliceLogContext`,
  as the accuracy pass settles each slice.
- Ballot irregularities:
  `da02c1fef` (guard `152c83e41`).
    `logBallotIrregularities` warns one line per finding,
  naming the stage and the model that cast the ballot;
    the audit counted 10,
  14 and 26 on three runs' artifacts and none in their logs.
- Assembly damage:
  `7d40f198b` (guards `2e443cd1a`,
  and `44ccb69f5` for the translate assembler).
    Repair assembly logs the slices and phrase of each introduced repetition
    and the slice and words of each content loss;
    translate assembly logs repetitions,
  the only document-scale check it runs.
    The findings stay free of corpus wording;
  the log carries it.
- Settlement line:
  `179297e7a` (guard `56fcd6e03`,
  wiring guard `824113c56`).
    "Unchanged" now says which:
  the archive won,
  the patch won and wrote nothing,
    or the patch won and was refused for dropping a declared name.
- Dispute stand-in:
  `f66e96f06` (guard `cbb8f3f26`) and `3ffbcec10` (guard `05357361d`).
    A refused dispute names its refusal
    (withdrawn at assembly,
  the archive's own wording,
  or unconfirmed by the checkers)
    in the log line and in the refused-wording finding a translate author reads,
    which also said the checkers had not confirmed a withdrawn slice's text.
    A stand-in identical to the archive is now refused whatever the checkers voted.
    Measured over every artifact (`l12e-census.mjs`):
  266 artifacts,
  1,791 disputed slices,
    1,132 with every disputing issue resolved,
  none of them with unchanged text,
  so no settled slice moves.
    `ArchiveDispute` is a union now,
  so a refused dispute cannot lack its reason.

Mutation check (`l12-mutants.json`,
then `l12b-mutants.json`):
dropping the refinement context,
either ballot-log call,
the model in the ballot line,
any of the three assembly log loops,
the slice in either damage line,
either settlement branch,
either settlement wiring argument,
the archive-wording or withdrawn refusal,
and the refusal lookup in the dispute line and in the author's reason are each caught;
both controls survive.
The first run found the settlement wiring unguarded (both mutants survived),
which `824113c56` closed;
two ballot-log mutants in it did not parse and two patterns had been reformatted by lint,
so the second run redid them.
Cache:
the log lines touch no key.
The eligibility and the reason reach the translate and consolidation lanes through the refused wordings,
which no key hashes;
they ride inside translate 15 and consolidation 20 with accounts,
same slice-cache check,
same result.

## L13: stale TSDoc and comments

Status:
fixed in `99dc48335`;
one item was code,
now L15.
Each site was read against the code before rewording,
and two claims beyond the audit's were wrong too.

- `chunk-critic-phase.ts`:
  `votesStand` said the slice "must ship unchanged" and its example returned unchanged;
    standing votes are evidence since question 3,
  answer B (2026-08-16).
    `heardCriticIds` said it was "what it was asked";
  it is who was heard.
- `repair-contract.ts`:
  `nonTranslationStanding` said the slice "shipped unchanged"
    and fed a "document-level block";
  the dominance finding reports and blocks nothing.
- `house-policy.ts`:
  the measuring sheets were said to decide nothing that ships;
    checker ballots decide resolution,
  strip worse-voted edits (L3) and roll rewrites back (L11),
    and an admitted probe claim rolls a naturalness rewrite back.
- `repair-editor-stage.ts`:
  "judges that wrote none of them" (editors sit among the judges at the self-vote weight);
    also "a decline ships the fallback repair" and "only when no editor produced an operation",
    both wrong since a unanimous decline returns the untouched text (L14(c));
    and a `@throws` naming "every judge also edits",
  which the roster check never refuses.
- `editor-ensemble.ts`:
  "producers are removed downstream",
  twice;
  they vote at the self-vote weight.
- `stage-quorum.ts`:
  "no stage decided by a single model" was said to be held by the quorum;
    the quorum is a retry target,
  and a gather short after every round proceeds on what it heard.
- `preservation-check.ts` and the version 12 note in `repair-slice-key.ts`:
  the token gate was described as
    what rejects damaging edits;
  L4 measured that it sees almost nothing,
  since envelopes are the licensed quotes.
- The dispute header (`archive-dispute.ts`) and the damage-evidence note (`repair-damage-evidence.ts`)
    were already rewritten by L2 and L7.
- `refine-slice-settle.ts:187`:
  "shipped deliberately untouched" was a comment on code that acted on it (L15).

## L15: the refinement lane still stopped on standing non-translation votes

Status:
fixed in `f77363387`,
guarded red first in `d601ebec6`;
found while fixing L13.
`settleRefinedSlice` returned a slice unrefined when its non-translation votes stood,
on the reading that such a slice shipped deliberately untouched.
Question 3,
answer B (`doc/decision/translation-repair-question-answers.md`,
2026-08-16)
keeps critics as evidence "rather than deciding anything" and removes every early return they owned,
and `05ff9791e` removed the repair lane's the same day;
this one was left,
and `refine-slice-settle.unit.test.ts` was written on 2026-08-24 to defend it on the stale reading.
Over every artifact (`~/temp/agent/audit-glossary-fix/l13-standing-census.mjs`),
10 of 6,150 slices had standing votes,
the repair lane changed 8 of them,
and none of the 9 in runs that refine was refined.
The settle test now expects the rewriter to be asked for a standing slice as for any other;
the phase test that used a standing slice to mean "ineligible" uses one too short to refine.
Mutation check:
restoring the early return is caught;
the control survived.
Cache:
rides inside refine 5,
same check,
same result.
