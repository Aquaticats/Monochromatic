import type {
  ArchiveOriginalSpan,
  PreparationIdentity,
} from '@monochromatic-dev/translation-repair-corpus-source/ts';
import type { RepairTranslationResult, } from '@monochromatic-dev/translation-repair-assembly/ts';
import type { TranslateDocumentResult, } from '@monochromatic-dev/translation-repair-translate-lane/ts';
import type { ArtifactConsolidation, } from './artifact-two-lane-consolidate.ts';
import type { ArtifactConsistency, } from './artifact-two-lane-consistency.ts';
import type { ArtifactPageAssembly, } from './artifact-two-lane-page-assembly.ts';
import type { ArtifactLaneSelection, } from '@monochromatic-dev/translation-repair-lane-contest/ts';
import type {
  ArtifactComparisonRow,
  ArtifactDeliveryRow,
} from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import type { PipelineDigest, } from '@monochromatic-dev/translation-repair-glossary/ts';

//region Artifact version 2 contract
// What a settled entry records once BOTH lanes run over one preparation.
//
// Version 1 recorded one lane at the top level: a status, a repaired text, a
// set of issues. Appending a second lane beside those would answer the open
// question invisibly, since every existing reader would go on taking the
// top-level fields as "the output" and never learn a second lane existed. So
// there is no top level any more. Both lanes are nested, neither is first, and
// the field that would name a winner says out loud that nobody has picked one.
//
// THE PREPARATION IS RECORDED ONCE, not per lane. Both lanes ran over one
// slicing by construction, and storing it twice would let a later reader meet
// two copies and have to decide which is real.
//
// WHAT THIS IS NOT: a self-contained record of the run. It stores measurements
// of the two documents rather than their canonical manifest, so a reader
// holding only this file can check that the recorded identity is SHAPED like an
// identity and nothing more. Recomputing it needs the corpus at the recorded
// commit. That asymmetry is deliberate and is stated here so nobody later reads
// a syntax check as a verification.

/**
 Generation twelve, which sealed the archive-original spans; the pass writes
 generation fourteen (`ARTIFACT_SCHEMA_VERSION_V14`).

 A LITERAL rather than a reference to the writer's current version, so the
 type says which generation it is and a later bump cannot quietly re-label it.

 WHAT MOVED FROM ELEVEN: a span the archive's translators' note calls the
 English original is SEALED out of every slice and ships as the archive has
 it (the owner's rule of 2026-09-08, `archive-original-note.ts`), and the
 preparation records the sealed spans as `archiveOriginalSpans`. A reader
 rebuilding the slicing of a file from this generation on seals again; one
 rebuilding an older file seals nothing, since that slicing never did.
 */
export const ARTIFACT_SCHEMA_VERSION_V12 = 12;

/**
 Generation that records what the page-level assembly guard did to the
 composed page.

 WHAT MOVED FROM TWELVE: the page the polish, the consolidation and the
 contest compose is run through the same footnote assembly guard each lane
 runs, and the artifact carries `pageAssembly`, the slices the guard trimmed
 with the text the page carries, the slices it took back, and its findings.
 A reader composing the page applies that section first. An older file
 carries no section and its page is composed as before.
 */
export const ARTIFACT_SCHEMA_VERSION_V13 = 13;

/**
 Generation fourteen: it records the effective wider bench for every
 naturalness review, so windowed and confirmation rounds retain their runtime
 quorum.
 */
export const ARTIFACT_SCHEMA_VERSION_V14 = 14;

/**
 Generation fifteen, the one the first consistency pass wrote, still read and
 no longer written (generation seventeen is): a slice where the repair lane recorded
 `not-applicable` (an archive-absent insertion) and the translate lane
 therefore alone had a passage to work on is recorded as a `sole-lane`
 contest verdict, with no ballots and no roster asked.

 WHAT MOVED FROM FOURTEEN: such a slice used to go to the roster like any
 other contest, and a roster that missed quorum left the passage unshipped
 (ledger L6). A reader of this generation takes the `sole-lane` verdict from
 the comparison row and refuses a roster verdict on it. A reader of fourteen
 or earlier takes the contest exactly as recorded, since those files asked
 the roster and the verdict is derived from its ballots.

 THE TENSE FINDING IS THIS GENERATION'S ALONE. A contest slice that went to the
 roster and a consolidation slice that ships wording may carry
 `tenseCandidates`, the present-tense candidates the judges were shown for the
 texts of that slice, only for an entry whose `info.yml` records a death and
 only when a text has one. It is a finding for the audit trail, counted toward
 no verdict, no flawed archive and no run-off. A reader of fourteen or earlier
 refuses the key; a reader of this generation refuses it on a `sole-lane`
 slice, which asked no judge, and on a consolidation slice that ships no
 wording of its own.

 THE CONSISTENCY SECTION JOINED THIS GENERATION before any pass ran at it, so
 it moved no number: a reader of fifteen requires the `consistency` section
 (the page-level pass's groups, ballots and changed slices) and refuses it in
 an older file. No pass had written a generation fifteen artifact, so none
 exists without it.
 */
export const ARTIFACT_SCHEMA_VERSION_V15 = 15;

/**
 Generation sixteen, still read and no longer written (generation seventeen
 is): the consistency section records the page-level pass
 as it counts ballots by the span of the place's own English they locate, runs
 off a tie for first, and carries the chosen rendering by writers and a choice
 among candidates.

 WHAT MOVED FROM FIFTEEN, all inside the `consistency` section
 (`corpus-run/artifact-two-lane-consistency.ts` states each): a located place
 carries the words and the offset the English holds, not a judge's string; a
 frame's slot is no condition of being located; a group the bench was asked
 about may carry `verdict.runOff`, and every group carries `carries`, one record
 per receiving place with its candidates and the ballots that chose among them;
 and the outcome `kept-current` and the winner reasons `run-off-no-quorum` and
 `run-off-tied` exist. A reader of this generation refuses the generation
 fifteen spellings of the slot reasons (`slot-no-agreement`, `slot-empty`,
 `slot-not-found`, `slot-ambiguous`, `slot-whole`), which no pass of this
 generation gives. A reader of fifteen refuses `runOff`, `carries`, a located
 place's `start`, the outcome `kept-current` and the two run-off reasons, and
 recounts a section exactly as the first pass to write one counted it, so a
 generation fifteen artifact reads as it was written.

 Everything else the generation fifteen record holds (the sole-lane verdict, the
 tense finding) is this generation's too.

 A section of this generation records an overlapped place with no name for the
 place it gave way to, so its reader keeps the check the section was written
 under: a replacement, a refused replacement or any located place of another
 group in the slice is enough for an overlapped place to stand beside.

 No pass wrote this generation: the last pass before it wrote fifteen, and
 seventeen landed before the next pass ran (ledger entry for the fourth
 consistency review). Its reader recounts ballots and finds a frame's slot with
 the locator as it now stands, whose reading of marks at the edge of a quote and
 of the joints of a word has changed since, so a section some other tool wrote
 under the earlier reading may be refused. No generation-gated locator is kept
 for files that do not exist.
 */
export const ARTIFACT_SCHEMA_VERSION_V16 = 16;

/**
 Generation seventeen, the one the pass wrote until generation eighteen: a place of the consistency section that ended
 `overlapped` names the place it gave way to, and a located place records the
 single marks the English holds beside its words.

 WHAT MOVED FROM SIXTEEN, all inside the `consistency` section
 (`corpus-run/artifact-two-lane-consistency.ts` states it): an occurrence that
 ended `overlapped` carries `gaveWayTo`, the position of the other group on the
 page and the label of its place, and no other occurrence carries the key. A
 reader of this generation resolves the name: the group and the place exist, the
 group is another one, the place stands in the same slice and either was
 replaced or refused by the floors or has a located rendering. Generation
 sixteen recorded an overlapped place with nothing to say what it overlapped, so
 a reader of it accepted one beside any located place of another group in its
 slice; that check stays for a generation sixteen file, and a reader of sixteen
 refuses `gaveWayTo`.

 A located rendering of this generation also stores `markBefore` and
 `markAfter` when present: the single mark (one of the eight the pass counts: the
 straight apostrophe, the opening and closing curly single mark, the modifier letter
 apostrophe, the low and the reversed single mark and both single angle marks;
 one character) the English holds directly before and directly
 after the span, as the English writes it, absent where none stands there. The
 span never covers one, except an apostrophe the locator proved
 (`consistency-proven-apostrophes.ts`): that mark belongs to its word, so the
 recorded words start or end with it and no `markBefore` or `markAfter` holds it.
 The splice is unavailable for two places that differ in a mark beside the span,
 whose words hold different quotation marks, or that hold a different number of
 block breaks (a blank line, or a line break the Markdown parse reads as a new
 block), so a reader of this generation recounts each splice with the marks compared
 (`planConsistencyEdit` with `marks: 'compared'`). A splice that brings a proven
 apostrophe to an edge where the receiving words hold none is a replacement, and
 whether the mark is still an apostrophe on the page the pass makes is decided by the
 pass, with every replacement made (`corpus-run/pass-consistency-apply-gains.ts`):
 a splice whose carried mark is not proven there is left out, and its place ends
 `kept-current`. A reader of sixteen and of
 fifteen refuses both keys and recounts a splice by the words alone
 (`marks: 'unrecorded'`), which is how those generations were written.

 WHAT A READER OF THIS GENERATION CHECKS OF A PROVEN APOSTROPHE: the recorded
 words of a located rendering may start or end with one apostrophe-like single
 mark with a letter or a digit directly inside it (the straight apostrophe, the
 closing curly mark or the modifier letter apostrophe), and the reader refuses
 recorded words that start or end with any other single mark (the opening curly
 mark, two marks, a mark beside a space). It holds no English, so it cannot verify
 that the mark was proven: the closing mark of a quotation recorded inside the
 words would read the same, and that case is left to the planner, which is
 recounted from the recorded words and marks. A reader of sixteen and of fifteen
 applies none of this.

 A UNIT MAY BE CARRIED IN PART, which no earlier section could record. The places
 of one block share one bench record: the same candidates, ballots and decision
 are recorded under each of them. When the splice is among the candidates because
 code can move the chosen words into some places of the block, the places code
 cannot move them into (their plan is `unavailable`) keep their words, and a
 reader of this generation reads each place from the record alone: a place whose
 recomputed plan is `unavailable` is `kept-current` after a splice was chosen (and
 ends as a writer's paragraph decides when one was chosen), a place whose plan is
 `replace` ends in `replaced`, `kept-floor` or `overlapped`, or in `kept-current`
 when the splice was chosen and would bring an apostrophe to an edge where the
 place's words hold none (the reader can check that this outcome is allowed and
 cannot check whether the mark was proven on the page, which the record does not
 hold) and then only with the reason the pass left the place alone, `keptBecause`, one
 of a closed set that the reader requires and cannot check, and a splice
 candidate under a place whose plan is `unavailable` is accepted only when
 another place of the group, recorded under the same candidates, ballots and
 decision, has a `replace` plan, since a block with no such place was offered no
 splice. A reader of sixteen and of fifteen keeps the older rule, that a splice
 candidate for a place whose plan is `unavailable` is refused and a place whose
 plan is `replace` or whose carry chose a splice or a writer's paragraph is never
 `kept-current`, which is how those sections were written.

 Everything else the generation sixteen record holds is this generation's too.

 A reader of this generation counts the finalists of a run-off, and the slot of a frame, under the rules the passes that wrote
 it used, which are the rules before generation eighteen: only the labels tied for the highest count are run off, and a slot
 counts only when it was given beside a quote that is the located words.
 */
export const ARTIFACT_SCHEMA_VERSION_V17 = 17;

/**
 Generation twenty, the one the pass wrote until generation twenty-one: a naturalness review round whose
 reachable seats are exactly its bench quorum closes on one voice fewer than those seats (owner, 2026-10-22, "Let the grace open";
 `reachableQuorum` in `stage-reachable-quorum.ts`), which the readers of generation nineteen and earlier do not apply to files written
 before.

 WHAT MOVED FROM NINETEEN, in the verdict a reader derives and in no field it records. A round over a bench of seven with three seats
 out of reach and three usable seats was `quorum-not-met` under nineteen, which needed all four live seats, and is decided by its
 seats here, which need three. A bench whose reachable seats are a quorum of two keeps the two-voice floor, so it still needs both.

 WHY A GENERATION AND NOT THE READER ALONE: a reader recomputes each round's verdict from its seats and refuses a record whose stored
 verdict differs (`parseNaturalnessReviewRound`), so a file of nineteen that recorded `quorum-not-met` on such a round would be refused
 under the new rule. A reader gates the rule on the generation the file records (`artifactGenerationLetsGraceOpenOnLastSeat`).

 A BUILD OF THIS GENERATION REFUSES TO RESUME A RUN DIRECTORY STAMPED NINETEEN OR EARLIER (`assertResumableSchemaGeneration`,
 `pass-schema-guard.ts`): a pass settles only into a directory whose files it writes itself. That is expected and no loss, since
 `readSettledArtifact` still reads those files, each under its own generation's rules.

 Everything else the generation nineteen record holds is this generation's too, so the key vocabulary is nineteen's. The cache keys
 of the passes do not read the generation; the caches whose gathers the rule moves carry their own version bumps.
 */
export const ARTIFACT_SCHEMA_VERSION_V20 = 20;

/**
 Generation twenty-one, the one the pass writes (`artifact-two-lane-build.ts`, `pass-schema-guard.ts`): a slice that
 kept its incumbent because the bench that decides it read below its floor says so in its record, as the key
 `benchBelowFloor` set to true, on the lane contest slice, the consolidation slice and an unfilled translate outcome
 (2026-10-23, after the XingZ6020 read).

 WHAT MOVED FROM TWENTY, in what a reader can tell and in one optional key. A lane contest slice of `quorum-not-met`
 with no ballots, a consolidation slice of `incumbent-only` with no gate, and a translate outcome of `unfilled` each
 read the same whether a roster was asked and declined or the floor kept the slice from being asked at all. Under
 twenty-one the second case carries the key and the first never does, so a reader of the record alone tells them
 apart. A record of twenty or earlier carries no key and is read as before, where an absent key says nothing about why
 the slice kept its incumbent.

 WHY A GENERATION AND NOT AN OPTIONAL KEY OF TWENTY: the readers of the three records require their exact keys
 (`requireExactKeys`), so a reader built for twenty refuses a record that carries the key, which is the situation
 `tenseCandidates` was in when generation fifteen added it. And an absent key means "unknown" under twenty and "a real
 decline" under twenty-one, which a reader cannot work out from the fields alone. A reader gates the key on the
 generation the file records (`artifactGenerationRecordsBenchFloor`).

 A BUILD OF THIS GENERATION REFUSES TO RESUME A RUN DIRECTORY STAMPED TWENTY OR EARLIER
 (`assertResumableSchemaGeneration`, `pass-schema-guard.ts`), and `readSettledArtifact` still reads those files, each
 under its own generation's rules.

 Everything else the generation twenty record holds is this generation's too, so the key vocabulary is twenty's. The
 cache keys of the passes do not read the generation, and nothing the key records goes into one.
 */
export const ARTIFACT_SCHEMA_VERSION_V21 = 21;

/**
 Generation twenty-two, the one the pass writes (`artifact-two-lane-build.ts`, `pass-schema-guard.ts`): a consolidation
 slice says why the archive keeps it, and says when the translate lane's winner ships because the final round could not
 seat two judges (owner, 2026-10-29, "Ship the two-ballot text (Recommended)", after the XingZ6022 read).

 WHAT MOVED FROM TWENTY-ONE, in what a record can say and in the shape of `shipped`. A consolidation slice that kept the
 archive recorded `{ kind: 'archive' }` whether no wording passed the rule, the final round could not seat two judges
 and no wording two judges chose existed, or the slate seated two or more judges and declined every wording; the
 publish defect then said "no wording passed" for all three. Under twenty-two the record carries the key `reason`
 (`no-wording-passed`, `final-round-unseated` or `slate-declined`) on that shipped kind, and a slice whose final round
 could not seat two judges and whose translate lane held a text two judges chose, passing the rule, records the new
 shipped kind `translate-winner` with that text.

 WHY A GENERATION AND NOT AN OPTIONAL KEY OF TWENTY-ONE: the reader of `shipped` requires its exact keys
 (`requireExactKeys`) and a closed set of kinds, so a reader built for twenty-one refuses both, and an absent `reason`
 means "unknown" under twenty-one and "recorded" under twenty-two. A reader gates both on the generation the file
 records (`artifactGenerationRecordsTranslateWinner`).

 A BUILD OF THIS GENERATION REFUSES TO RESUME A RUN DIRECTORY STAMPED TWENTY-ONE OR EARLIER
 (`assertResumableSchemaGeneration`, `pass-schema-guard.ts`), and `readSettledArtifact` still reads those files, each
 under its own generation's rules.

 Everything else the generation twenty-one record holds is this generation's too, so the key vocabulary is
 twenty-one's. The cache keys of the passes do not read the generation, and nothing the key records goes into one.
 */
export const ARTIFACT_SCHEMA_VERSION_V22 = 22;

/**
 Generation nineteen, the one the pass wrote until generation twenty: the recount of the
 `consistency` section reads containment of the judges' quotes and the inline code of a span as the locator reads them (review of
 2026-10-18), which the readers of generation eighteen and earlier do not apply to files written before.

 A READER OF THIS GENERATION DERIVES A NATURALNESS VERDICT UNDER THE RULE OF 2026-09-09, which generation twenty replaced: a round
 whose reachable seats are exactly its bench quorum needs every one of them.

 WHAT MOVED FROM EIGHTEEN, in the recount and in no field it records. A quote is contained in another only where no joint of a longer
 word stands at an edge of the match, the locator's own test of an apostrophe, a hyphen or a joint of digits (`containsWords`,
 `consistency-contains-words.ts`), so the candidate spans of a place and the slot words of a frame differ for quotes such as `cat` and
 `cat's bowl`: two judges who quoted `cat's bowl` and one who quoted `cat` agreed on `cat` in eighteen and agree on no shorter words
 here. A line ending pads inline code as a space does (`codeContentOf`, `consistency-markup-ranges.ts`), so the locator refuses a word
 beside code padded by line endings, and a slot recorded beside such code is no stand.

 WHY A GENERATION AND NOT THE RECOUNT ALONE: a file of eighteen was right under the rules its pass counted by, and the stricter recount
 would refuse the agreement it recorded. A reader gates each reading on the generation the file records, so a stored artifact is read
 under the rules it was written by (`artifactGenerationCutsContainmentAtJoints`,
 `artifactGenerationReadsLineEndingsAsCodePadding`). Only the recount of a stored artifact is gated: the pass, which locates places in
 a live page, always reads the locator's rules.

 A BUILD OF THIS GENERATION REFUSES TO RESUME A RUN DIRECTORY STAMPED EIGHTEEN OR EARLIER (`assertResumableSchemaGeneration`,
 `pass-schema-guard.ts`): a pass settles only into a directory whose files it writes itself. That is expected and no loss, since
 `readSettledArtifact` still reads those files, each under its own generation's rules.

 Everything else the generation eighteen record holds is this generation's too. The shape of the section is eighteen's, so the key
 vocabulary is eighteen's. No cache key of the passes reads the generation or the field: the consistency cache digests the sheet and its
 own version (`CONSISTENCY_CACHE_VERSION`), which did not move, since the counting of quotes is not stored.
 */
export const ARTIFACT_SCHEMA_VERSION_V19 = 19;

/**
 Generation eighteen, the one the pass wrote until generation nineteen: the finalists of a run-off and the slot of a frame are counted
 by the rules the lead adopted on 2026-10-15 and 2026-10-18, which the readers of generation seventeen and earlier do not apply to
 files written before them.

 WHAT MOVED FROM SEVENTEEN, in the recount of the `consistency` section and in no field it records: a first round with no strict
 majority and a lone leader is run off among the leader and the labels with the second-highest count (`runOffFinalistsOf`), where
 seventeen ran off only the labels tied for the highest count (`runOffTiedFinalistsOf`), so a group of two, one and one over four
 usable ballots records a run-off among the three labels here and none in seventeen; and a slot a strict majority of the usable
 ballots gave beside any quote that contains the located words counts for the frame (`SlotAgreement` `containing-quote`), where
 seventeen counted only a slot given beside a quote that is the located words (`exact-quote`). A reader of this generation requires
 the run-off its ballots call for and refuses a missing one, and a reader of seventeen refuses a run-off recorded for a lone
 leader, which no pass of that generation recorded.

 A CARRY'S RUN-OFF IS REQUIRED THE SAME WAY: a reader of this generation refuses a carry recorded with a first count that reached its
 quorum and gave no candidate a strict majority, and no run-off, where the finalists rule calls for one. A reader of seventeen and
 earlier reads such a carry as it was written, since the runs of those generations recorded a split and went no further.

 A LOCATED PLACE OF A FRAME RECORDS `definitions` WHERE THE PAGE'S DEFINITIONS MOVE ITS SLOT: the link and footnote definitions the pass
 read the slot with, the smallest set that gives the slot (`definitionsMovingSlotOf`), absent where the page's definitions change
 nothing. The file holds no page to rebuild them from, so the recount of this generation reads the slot with the recorded definitions
 and refuses definitions that are malformed or move no slot. A reader of seventeen and earlier refuses the key and recounts the slot over
 the span alone, which is a limit it documents (ledger B661): a span that holds a reference defined in another slice is read as plain
 text there.

 A READER OF THIS GENERATION RECOUNTS CONTAINMENT AND INLINE CODE BY THE READINGS THE PASS THAT WROTE IT USED, which generation
 nineteen replaced (review of 2026-10-18): a quote is contained in another where no letter, digit or combining mark stands beside the
 match, a joint of a longer word being no bar, so `cat` is contained in `cat's bowl`; and a literal space alone pads inline code, so a
 line ending beside it is a shown separator and a word beside code padded by line endings is a stand. Neither moved a field.
 A carry the bench was not asked (a decision of no alternative) is refused when it records a run-off, which no pass writes; that
 refusal applies to every generation, this one included.

 Everything else the generation seventeen record holds is this generation's too. The shape of the section is that of seventeen
 with the one key `definitions` added, so the key vocabulary is generation seventeen's and that key. No cache key of the passes
 reads the generation or the field: the consistency cache digests the sheet and its own version
 (`CONSISTENCY_CACHE_VERSION`), which did not move.
 */
export const ARTIFACT_SCHEMA_VERSION_V18 = 18;

/**
 Generation before archive-original spans were sealed; the archive's front
 matter stood as published from here.

 WHAT MOVED FROM TEN: the archive's front matter stands as published where
 the archive translated it (the owner's rule of 2026-09-08), so the
 preparation of such an entry carries NO metadata slice and records
 `frontMatterAuthority: 'archive'`. A reader rebuilding the slicing must
 know not to add slice zero, and it reads that off the record rather than
 recomputing the rule, so a later change to the rule cannot re-slice an
 older file.
 */
export const ARTIFACT_SCHEMA_VERSION_V11 = 11;

/**
 Generation before the archive's front matter stood as published; the
 absolute naturalness reviewer was shown every body block from here.
 */
export const ARTIFACT_SCHEMA_VERSION_V10 = 10;

/**
 Generation before reviewers were shown every body block.
 */
export const ARTIFACT_SCHEMA_VERSION_V9 = 9;

/**
 Generation before two bounded correction transitions became auditable.
 */
export const ARTIFACT_SCHEMA_VERSION_V8 = 8;

/**
 Generation before exact-text absolute naturalness review became auditable.
 */
export const ARTIFACT_SCHEMA_VERSION_V7 = 7;

/**
 Generation before source-backed contest eligibility became auditable.
 */
export const ARTIFACT_SCHEMA_VERSION_V6 = 6;

/**
 Generation before final post-consolidation body polish became auditable.
 */
export const ARTIFACT_SCHEMA_VERSION_V5 = 5;

/**
 Generation before front matter became explicit slice zero.
 */
export const ARTIFACT_SCHEMA_VERSION_V4 = 4;

/**
 Generation that renamed the three index and critic keys and left the
 per-slice index alone. Still read, no longer written.

 A MIXTURE, and that is the whole reason it needs its own row in the key
 table: it spells the arrays the way generation 4 does and the index the way
 generation 2 did.
 */
export const ARTIFACT_SCHEMA_VERSION_V3 = 3;

/**
 Generation this same shape was first written under, still read and no longer
 written.

 THE SHAPE DID NOT MOVE. Versions 2, 3 and 4 record the same two lanes, the
 same comparison and the same lane selection. They differ in four key
 spellings, which `artifact-key-vocabulary.ts` holds and a reader selects by
 the version the file records.

 The version moved anyway, twice, because a key rename IS a shape change and a
 version that does not move on one is the failure this field exists to end.
 */
export const ARTIFACT_SCHEMA_VERSION_V2 = 2;

/**
 Every generation carrying this two-lane shape, oldest first.

 ONE AUTHORITY FOR THE FAMILY, because two places decide something about it:
 the reader that accepts a body, and the dispatch that chooses that reader for
 a file. Those two lists drifting apart is not a refusal but a WRONG ANSWER,
 and it already happened once: generation 3 was minted, the reader learned it,
 the dispatch did not, and every generation 3 artifact reaching the dispatch
 was reported as a generation nothing reads.

 NAMED ONE BY ONE RATHER THAN AS A RANGE, since a generation belongs here once
 someone has checked that `artifact-key-vocabulary.ts` spells it, not because
 its number falls between two others.

 @example
 ```ts
 if (TWO_LANE_GENERATIONS.includes(version,)) { ... }
 ```
 */
export const TWO_LANE_GENERATIONS: readonly number[] = [
  ARTIFACT_SCHEMA_VERSION_V2,
  ARTIFACT_SCHEMA_VERSION_V3,
  ARTIFACT_SCHEMA_VERSION_V4,
  ARTIFACT_SCHEMA_VERSION_V5,
  ARTIFACT_SCHEMA_VERSION_V6,
  ARTIFACT_SCHEMA_VERSION_V7,
  ARTIFACT_SCHEMA_VERSION_V8,
  ARTIFACT_SCHEMA_VERSION_V9,
  ARTIFACT_SCHEMA_VERSION_V10,
  ARTIFACT_SCHEMA_VERSION_V11,
  ARTIFACT_SCHEMA_VERSION_V12,
  ARTIFACT_SCHEMA_VERSION_V13,
  ARTIFACT_SCHEMA_VERSION_V14,
  ARTIFACT_SCHEMA_VERSION_V15,
  ARTIFACT_SCHEMA_VERSION_V16,
  ARTIFACT_SCHEMA_VERSION_V17,
  ARTIFACT_SCHEMA_VERSION_V18,
  ARTIFACT_SCHEMA_VERSION_V19,
  ARTIFACT_SCHEMA_VERSION_V20,
  ARTIFACT_SCHEMA_VERSION_V21,
  ARTIFACT_SCHEMA_VERSION_V22,
];

/**
 Generation of parsed two-lane artifact.

 @example
 ```ts
 const generation: TwoLaneArtifactGeneration = 5;
 ```
 */
export type TwoLaneArtifactGeneration =
  | typeof ARTIFACT_SCHEMA_VERSION_V2
    | typeof ARTIFACT_SCHEMA_VERSION_V3
    | typeof ARTIFACT_SCHEMA_VERSION_V4
    | typeof ARTIFACT_SCHEMA_VERSION_V5
    | typeof ARTIFACT_SCHEMA_VERSION_V6
    | typeof ARTIFACT_SCHEMA_VERSION_V7
    | typeof ARTIFACT_SCHEMA_VERSION_V8
    | typeof ARTIFACT_SCHEMA_VERSION_V9
    | typeof ARTIFACT_SCHEMA_VERSION_V10
    | typeof ARTIFACT_SCHEMA_VERSION_V11
    | typeof ARTIFACT_SCHEMA_VERSION_V12
    | typeof ARTIFACT_SCHEMA_VERSION_V13
    | typeof ARTIFACT_SCHEMA_VERSION_V14
    | typeof ARTIFACT_SCHEMA_VERSION_V15
    | typeof ARTIFACT_SCHEMA_VERSION_V16
    | typeof ARTIFACT_SCHEMA_VERSION_V17
    | typeof ARTIFACT_SCHEMA_VERSION_V18
    | typeof ARTIFACT_SCHEMA_VERSION_V19
    | typeof ARTIFACT_SCHEMA_VERSION_V20
    | typeof ARTIFACT_SCHEMA_VERSION_V21
    | typeof ARTIFACT_SCHEMA_VERSION_V22;

/**
 Narrows numeric artifact version to known two-lane generation.

 @param value - parsed schema version

 @returns Whether this two-lane reader knows generation

 @example
 ```ts
 if (isTwoLaneArtifactGeneration(artifact.artifactSchemaVersion)) read(artifact);
 ```
 */
export function isTwoLaneArtifactGeneration(
  value: number,
): value is TwoLaneArtifactGeneration {
  return TWO_LANE_GENERATIONS.includes(value,);
}

/**
 Reports whether generation requires auditable consolidation polish records.

 @param generation - known two-lane artifact generation

 @returns Whether every consolidation slice must carry polish field

 @example
 ```ts
 artifactGenerationRequiresPolish({ generation: 7, });
 ```
 */
export function artifactGenerationRequiresPolish(
  { generation, }: { readonly generation: TwoLaneArtifactGeneration; },
): boolean {
  return generation >= ARTIFACT_SCHEMA_VERSION_V6;
}

/**
 Reports whether generation binds final wording to absolute naturalness review.

 @param generation - known two-lane artifact generation

 @returns Whether consolidation polish must carry absolute review audit

 @example
 ```ts
 artifactGenerationRequiresNaturalnessReview({ generation: 8, });
 ```
 */
export function artifactGenerationRequiresNaturalnessReview(
  { generation, }: { readonly generation: TwoLaneArtifactGeneration; },
): boolean {
  return generation >= ARTIFACT_SCHEMA_VERSION_V8;
}

/**
 Reports whether generation binds every correction transition by digest.

 @param generation - known two-lane artifact generation

 @returns Whether review audit requires correction chain

 @example
 ```ts
 artifactGenerationRequiresNaturalnessCorrectionChain({ generation: 9, });
 ```
 */
export function artifactGenerationRequiresNaturalnessCorrectionChain(
  { generation, }: { readonly generation: TwoLaneArtifactGeneration; },
): boolean {
  return generation >= ARTIFACT_SCHEMA_VERSION_V9;
}

/**
 Reports whether generation showed the absolute reviewer every body block
 of the candidate, so that recorded paragraph counts and digests are of
 those blocks rather than of the refinable paragraphs alone.

 @param generation - known two-lane artifact generation

 @returns Whether reviewed paragraphs are every body block

 @example
 ```ts
 artifactGenerationReviewsEveryBodyBlock({ generation: 10, });
 ```
 */
export function artifactGenerationReviewsEveryBodyBlock(
  { generation, }: { readonly generation: TwoLaneArtifactGeneration; },
): boolean {
  return generation >= ARTIFACT_SCHEMA_VERSION_V10;
}

/**
 Reports whether generation settles a slice that only one lane applies to by
 rule, with no roster asked, rather than by a contest over both lanes.

 @param generation - known two-lane artifact generation

 @returns Whether a sole-lane contest verdict is recorded and expected

 @example
 ```ts
 artifactGenerationSettlesSoleLane({ generation: 15, });
 ```
 */
export function artifactGenerationSettlesSoleLane(
  { generation, }: { readonly generation: TwoLaneArtifactGeneration; },
): boolean {
  return generation >= ARTIFACT_SCHEMA_VERSION_V15;
}

/**
 Reports whether the runs of a generation told their models the recorded date
 of death and showed their judges the tense candidates, and so record those
 candidates on a contest slice that went to the roster and on a consolidation
 slice that ships wording. A run of an earlier generation read no `info.yml`,
 so a replay of one must not add the date to the identity block its models saw.

 @param generation - known two-lane artifact generation

 @returns Whether the recorded death and the tense finding belong to the generation's runs

 @example
 ```ts
 artifactGenerationRecordsTense({ generation: 15, });
 ```
 */
export function artifactGenerationRecordsTense(
  { generation, }: { readonly generation: TwoLaneArtifactGeneration; },
): boolean {
  return generation >= ARTIFACT_SCHEMA_VERSION_V15;
}

/**
 Reports whether generation records the page-level consistency section.

 @param generation - known two-lane artifact generation

 @returns Whether the section is required, and refused in an older file

 @example
 ```ts
 artifactGenerationRecordsConsistency({ generation: 15, });
 ```
 */
export function artifactGenerationRecordsConsistency(
  { generation, }: { readonly generation: TwoLaneArtifactGeneration; },
): boolean {
  return generation >= ARTIFACT_SCHEMA_VERSION_V15;
}

/**
 Reports whether generation counts the consistency ballots by the span of the
 place's own English they locate, runs off a split first round and carries the
 chosen rendering by writers, so its section has the sixteenth shape. An older
 section is recounted by the exact text of the quotes, as it was written.

 @param generation - known two-lane artifact generation

 @returns Whether the section is of the sixteenth shape

 @example
 ```ts
 artifactGenerationCountsBySpan({ generation: 16, });
 ```
 */
export function artifactGenerationCountsBySpan(
  { generation, }: { readonly generation: TwoLaneArtifactGeneration; },
): boolean {
  return generation >= ARTIFACT_SCHEMA_VERSION_V16;
}

/**
 Reports whether generation records, for a place of the consistency section
 that ended overlapped, the place it gave way to, so that its reader resolves
 the name. An older section recorded the outcome alone and is read under the
 weaker check it was written under.

 @param generation - known two-lane artifact generation

 @returns Whether an overlapped place of the section names the place it gave way to

 @example
 ```ts
 artifactGenerationNamesOverlap({ generation: 17, });
 ```
 */
export function artifactGenerationNamesOverlap(
  { generation, }: { readonly generation: TwoLaneArtifactGeneration; },
): boolean {
  return generation >= ARTIFACT_SCHEMA_VERSION_V17;
}

/**
 Reports whether generation records, for a located place of the consistency
 section, the single marks the English holds directly beside its span, so that
 its reader compares them when it recounts a splice. An older section kept no
 mark and is recounted by the words of the places alone.

 @param generation - known two-lane artifact generation

 @returns Whether a located place may carry `markBefore` and `markAfter`, and a splice is recounted with the marks compared

 @example
 ```ts
 artifactGenerationRecordsMarks({ generation: 17, });
 ```
 */
export function artifactGenerationRecordsMarks(
  { generation, }: { readonly generation: TwoLaneArtifactGeneration; },
): boolean {
  return generation >= ARTIFACT_SCHEMA_VERSION_V17;
}

/**
 Reports whether generation runs off a first round with no strict majority and a lone leader, among the leader and the labels with
 the second-highest count. An older section was written when only the labels tied for the highest count were run off, and its
 reader keeps that rule: such a group records no run-off, and one that records a run-off for a lone leader is refused.

 @param generation - known two-lane artifact generation

 @returns Whether the finalists of a run-off include the labels next behind a lone leader

 @example
 ```ts
 artifactGenerationRunsOffLoneLeader({ generation: 18, });
 ```
 */
export function artifactGenerationRunsOffLoneLeader(
  { generation, }: { readonly generation: TwoLaneArtifactGeneration; },
): boolean {
  return generation >= ARTIFACT_SCHEMA_VERSION_V18;
}

/**
 Reports whether generation counts the slot of a frame from every quote that contains the located words, as the place itself is
 located by containment. An older section counted only a slot given beside a quote that is the located words, and its reader keeps
 that rule.

 @param generation - known two-lane artifact generation

 @returns Whether a slot given beside a longer quote that contains the located words counts toward the frame's slot

 @example
 ```ts
 artifactGenerationReadsSlotsBesideContainingQuotes({ generation: 18, });
 ```
 */
export function artifactGenerationReadsSlotsBesideContainingQuotes(
  { generation, }: { readonly generation: TwoLaneArtifactGeneration; },
): boolean {
  return generation >= ARTIFACT_SCHEMA_VERSION_V18;
}

/**
 Reports whether generation requires a carry whose first count reached its quorum and gave no candidate a strict majority to record
 the run-off the finalists rule asks for. An older section is read as it was written: such a carry ends at the split there.

 @param generation - known two-lane artifact generation

 @returns Whether a carry that names finalists and records no run-off is refused

 @example
 ```ts
 artifactGenerationRequiresCarryRunOff({ generation: 18, });
 ```
 */
export function artifactGenerationRequiresCarryRunOff(
  { generation, }: { readonly generation: TwoLaneArtifactGeneration; },
): boolean {
  return generation >= ARTIFACT_SCHEMA_VERSION_V18;
}

/**
 Reports whether generation records, on a located place of a frame, the page definitions that move its slot, and reads the slot with
 them. An older section records none and is recounted over the span alone, and its reader refuses the key.

 @param generation - known two-lane artifact generation

 @returns Whether a located rendering may carry `definitions`, and the slot is recounted with them

 @example
 ```ts
 artifactGenerationRecordsDefinitions({ generation: 18, });
 ```
 */
export function artifactGenerationRecordsDefinitions(
  { generation, }: { readonly generation: TwoLaneArtifactGeneration; },
): boolean {
  return generation >= ARTIFACT_SCHEMA_VERSION_V18;
}

/**
 Reports whether generation recounts containment of the judges' quotes as the locator reads it: a quote is contained in another only
 where no joint of a longer word (an apostrophe or a hyphen between letters, a full stop between digits) stands at an edge of the
 match, so `cat` is not contained in `cat's bowl`. An older section was written when only a letter, a digit or a combining mark beside
 the match barred containment, and its reader keeps that reading, so the agreement it recorded is not refused for a rule it was not
 written under.

 @param generation - known two-lane artifact generation

 @returns Whether a joint of a longer word bars a quote from containing another in the recount of the candidate spans and the slot words

 @example
 ```ts
 artifactGenerationCutsContainmentAtJoints({ generation: 19, });
 ```
 */
export function artifactGenerationCutsContainmentAtJoints(
  { generation, }: { readonly generation: TwoLaneArtifactGeneration; },
): boolean {
  return generation >= ARTIFACT_SCHEMA_VERSION_V19;
}

/**
 Reports whether generation recounts inline code as CommonMark reads it, with each line ending that pads the code read as a space, so
 the locator refuses a word beside code padded by line endings. An older section was written when a literal space alone padded code,
 and its reader keeps that reading.

 @param generation - known two-lane artifact generation

 @returns Whether a slot is located in a span with the line endings that pad inline code read as spaces

 @example
 ```ts
 artifactGenerationReadsLineEndingsAsCodePadding({ generation: 19, });
 ```
 */
export function artifactGenerationReadsLineEndingsAsCodePadding(
  { generation, }: { readonly generation: TwoLaneArtifactGeneration; },
): boolean {
  return generation >= ARTIFACT_SCHEMA_VERSION_V19;
}

/**
 Reports whether generation derives a naturalness verdict with the last-seat grace: a round whose reachable seats are exactly its
 bench quorum closes on one voice fewer than those seats, never under two (owner, 2026-10-22). An older round was decided when such a bench needed
 every live seat, and its reader keeps that rule, so the verdict it recorded is not refused for a rule it was not written under.

 @param generation - known two-lane artifact generation

 @returns Whether `reachableQuorum` reads the round with its last-seat grace on

 @example
 ```ts
 artifactGenerationLetsGraceOpenOnLastSeat({ generation: 20, });
 ```
 */
export function artifactGenerationLetsGraceOpenOnLastSeat(
  { generation, }: { readonly generation: TwoLaneArtifactGeneration; },
): boolean {
  return generation >= ARTIFACT_SCHEMA_VERSION_V20;
}

/**
 Reports whether generation may record a unit carried in part: a splice chosen
 for a block whose places are not all ones code can move the chosen words into,
 the others `kept-current`. An older section was written when one such place
 withheld the splice from its whole block, and its reader keeps that rule.

 @param generation - known two-lane artifact generation

 @returns Whether a place whose plan is `unavailable` may sit under a splice candidate its block shares, and ends `kept-current` when the splice is chosen

 @example
 ```ts
 artifactGenerationCarriesPartialUnits({ generation: 17, });
 ```
 */
export function artifactGenerationCarriesPartialUnits(
  { generation, }: { readonly generation: TwoLaneArtifactGeneration; },
): boolean {
  return generation >= ARTIFACT_SCHEMA_VERSION_V17;
}

/**
 Reports whether generation records, on the slice it kept for want of a bench, that the bench read below its floor:
 the lane contest slice, the consolidation slice and an unfilled translate outcome carry `benchBelowFloor` from this
 generation, and an older record refuses the key.

 @param generation - known two-lane artifact generation

 @returns Whether the key belongs to the generation's records

 @example ```ts artifactGenerationRecordsBenchFloor({ generation: 21, }); ```
 */
export function artifactGenerationRecordsBenchFloor(
  { generation, }: { readonly generation: TwoLaneArtifactGeneration; },
): boolean {
  return generation >= ARTIFACT_SCHEMA_VERSION_V21;
}

/**
 Reports whether generation records, on a consolidation slice, why the archive keeps it (the `reason` of the archive
 shipped kind) and the shipped kind `translate-winner`; an older record refuses both.

 @param generation - known two-lane artifact generation

 @returns Whether the key and the kind belong to the generation's records

 @example ```ts artifactGenerationRecordsTranslateWinner({ generation: 22, }); ```
 */
export function artifactGenerationRecordsTranslateWinner(
  { generation, }: { readonly generation: TwoLaneArtifactGeneration; },
): boolean {
  return generation >= ARTIFACT_SCHEMA_VERSION_V22;
}

/**
 Generation-specific fields exact reader requires.

 @param generation - known two-lane artifact generation

 @returns Polish and absolute-review requirements for generation, and the
 quorum rule its review rounds were decided under

 @example
 ```ts
 const requirements = artifactGenerationReadingRequirements({ generation: 8, });
 ```
 */
export function artifactGenerationReadingRequirements(
  { generation, }: { readonly generation: TwoLaneArtifactGeneration; },
): {
  readonly polishRequired: boolean;
  readonly reviewRequired: boolean;
  readonly correctionChainRequired: boolean;
  readonly everyBodyBlockReviewed: boolean;
  readonly quorumBasisRequired: boolean;
  readonly lastSeatGrace: boolean;
  readonly tenseRecorded: boolean;
  readonly benchFloorRecorded: boolean;
  readonly translateWinnerRecorded: boolean;
} {
  return {
    polishRequired: artifactGenerationRequiresPolish({ generation, }),
    reviewRequired: artifactGenerationRequiresNaturalnessReview({ generation, }),
    correctionChainRequired: artifactGenerationRequiresNaturalnessCorrectionChain({ generation, }),
    everyBodyBlockReviewed: artifactGenerationReviewsEveryBodyBlock({ generation, }),
    quorumBasisRequired: generation >= ARTIFACT_SCHEMA_VERSION_V14,
    lastSeatGrace: artifactGenerationLetsGraceOpenOnLastSeat({ generation, }),
    tenseRecorded: artifactGenerationRecordsTense({ generation, }),
    benchFloorRecorded: artifactGenerationRecordsBenchFloor({ generation, }),
    translateWinnerRecorded: artifactGenerationRecordsTranslateWinner({ generation, }),
  };
}

/**
 What the one field this schema does not describe may hold.

 JSON's `null` is deliberately NOT among these. It is absence spelled as a
 value, which is the thing this whole generation exists to stop recording, and
 the writer controls every byte that reaches this field: a configuration with
 nothing to say about a setting leaves the key out. A reader meeting a null
 here has met an artifact this schema did not write.

 @example
 ```ts
 const value: ArtifactJsonValue = { retries: 2, };
 ```
 */
export type ArtifactJsonValue =
  | boolean
    | number
    | string
    | readonly ArtifactJsonValue[]
    | { readonly [key: string]: ArtifactJsonValue; };

/**
 The slicing both lanes ran over, recorded once.

 @example
 ```ts
 const preparation: SettledPreparation = { identity, sliceCount: 12, ... };
 ```
 */
export type SettledPreparation = {
  /**
   Name this slicing gives itself, which every row is joined under.
   */
  readonly identity: PreparationIdentity;

  /**
   Archive English of the whole entry, as it stood before either lane ran.

   STORED BECAUSE IT COSTS ALMOST NOTHING AND BUYS THE FILE ITS OWN MEANING.
   Measured over the archived artifacts it adds 0.6 and 0.9 percent: an
   artifact is judge exchanges, findings and ledger rows, not text. The
   alternative, a hash plus the corpus commit, saves that fraction by charging
   every future reader a checkout pinned to the right commit, which is the
   dependency generation identity exists to remove.

   IT TRAVELS WITH {@link SettledPreparation.identity} OR NOT AT ALL. Stored
   text nobody can check against the slicing that produced it is a record with
   no standing, so the hash is what makes the text evidence rather than a copy.

   Decided in `doc/decision/artifact-stores-the-archive-text.md`.
   */
  readonly archiveText: string;

  /**
   Slices it produced, which every per-slice list here is out of.
   */
  readonly sliceCount: number;

  /**
   Present when the archive's front matter stood and no metadata slice was
   made (the owner's rule of 2026-09-08); absent when the lanes rendered
   slice zero, and on every file written before generation eleven.
   */
  readonly frontMatterAuthority?: 'archive';

  /**
   Present when the archive's translators' note sealed one or more spans as
   the English original (the owner's rule of 2026-09-08), each in archive
   text offsets with the note that sealed it; absent when nothing was
   sealed, and on every file written before generation twelve.
   */
  readonly archiveOriginalSpans?: readonly ArchiveOriginalSpan[];

  /**
   Original document length, in UTF-16 code units, for eyeballing an entry's
   size. NOT the band input, which is `sourceBytes`.
   */
  readonly sourceChars: number;

  /**
   Archive translation length, on the same measure.
   */
  readonly targetChars: number;

  /**
   Original document length in UTF-8 BYTES, which is what band classification
   takes and what the character counts are routinely mistaken for.
   */
  readonly sourceBytes: number;

  /**
   Section pairs alignment produced.
   */
  readonly alignmentPairCount: number;

  /**
   What alignment observed about the two documents' structure.

   Recorded on the preparation rather than under either lane: both ran over
   this one alignment, and counting these per lane would count one defect in
   the archive twice.
   */
  readonly alignmentFindings: readonly string[];

  /**
   Which original block the roster said each translation block renders, per
   aligned section.

   THE MOST CONSEQUENTIAL DECISION IN A RUN, and until this field the only one
   a settled entry did not keep. It decides which original each slice is
   judged against, no later stage can repair a wrong one, and the cache that
   held it is discarded once the entry settles. Recovering it meant racing a
   live run before it settled.

   OPTIONAL WITHIN VERSION 2 rather than a version 3, on the precedent
   {@link SettledPreparation.archiveText} set: a reader that meets it
   unrecorded understands the artifact completely, so refusing the whole
   generation over an added field would buy nothing. Recorded in
   `doc/decision/artifact-stores-the-block-pairing.md`.

   ABSENT MEANS NOBODY WAS ASKED, which in practice means the artifact was
   written before this field existed: every production entry runs through the
   roster shell. Present and EMPTY means the roster was asked and agreed
   nothing anywhere, which is a different fact. A section missing from a
   present list had no pairing consumed for it, and which reason applies is
   legible from {@link SettledPreparation.alignmentFindings} rather than
   from here: it may have been trivial enough that nobody was asked, it may
   have fallen back to scoring, or its round may have gone unanswered.
   */
  readonly blockPairing?: readonly ArtifactSectionPairing[];

  /**
   Which decider chose the aligned sections, and what it chose.

   THE OTHER HALF OF THE PAIRING RECIPE. The `blockPairing` field is keyed by
   aligned section index, and those indices only mean something under the
   section alignment that was in force. A reader rebuilding the slicing
   needs both, and until this field it had one.

   ALWAYS WRITTEN BY THE BUILDER, unlike `blockPairing`: the deterministic
   aligner deciding the sections is the ordinary production case, so an
   absent field could not tell "the aligner decided" from "written before
   the field existed". Optional here only because every artifact settled
   before it was added lacks it, which a reader reports as unrecorded.
   */
  readonly sectionPairing?: ArtifactSectionAlignment;
};

/**
 One committed correspondence between the two sides' sections, as version 2
 records it.

 FROZEN UNDER A VERSION 2 NAME rather than reusing the live `SectionPair`,
 for the reason {@link ArtifactSectionPairing} gives.

 @example
 ```ts
 const pair: ArtifactSectionCorrespondence = { source: 2, target: 3, };
 ```
 */
export type ArtifactSectionCorrespondence = {
  /**
   Original-side section index.
   */
  readonly source: number;

  /**
   Translation-side section index.
   */
  readonly target: number;
};

/**
 How the aligned sections were decided, as version 2 records it.

 @example
 ```ts
 const alignment: ArtifactSectionAlignment = { kind: 'deterministic', };
 ```
 */
export type ArtifactSectionAlignment = {
  /**
   The deterministic aligner chose the sections, by shape or by heading
   scoring; no pairing was supplied.
   */
  readonly kind: 'deterministic';
} | {
  /**
   A supplied pairing chose them, which in production is the roster's
   section round.
   */
  readonly kind: 'supplied';

  /**
   Correspondences it committed to, in document order.
   */
  readonly pairs: readonly ArtifactSectionCorrespondence[];
};

/**
 One aligned section's pairing, as version 2 records it.

 FROZEN UNDER A VERSION 2 NAME rather than reusing the live `BlockPair`, which
 is the rule everywhere in this schema that is not evidence: a later field on
 the live type would otherwise silently change what an artifact claiming
 version 2 means. Drift makes the writer stop compiling, which is the moment
 the version question should be asked.

 @example
 ```ts
 const pairing: ArtifactSectionPairing = { sectionIndex: 0, pairs: [{ source: 0, target: 0, },], };
 ```
 */
export type ArtifactSectionPairing = {
  /**
   Aligned section this answers about, which every other index of this pairing is local to.
   */
  readonly sectionIndex: number;

  /**
   Correspondences the roster agreed on, in document order.
   */
  readonly pairs: readonly {
    /**
     Original-side block index within this section.
     */
    readonly source: number;

    /**
     Translation-side block index within this section.
     */
    readonly target: number;
  }[];
};

/**
 One lane's raw result beside the ledger derived from it.

 BOTH, rather than either alone. The ledger is what a reader compares, because
 it has been checked against that lane's own document; the raw result is the
 evidence behind it, and a ledger with its evidence discarded cannot answer
 why a slice went the way it did.

 @example
 ```ts
 const lane: SettledLane<RepairTranslationResult> = { result, delivery, };
 ```
 */
export type SettledLane<TResult,> = {
  /**
   Exactly what the lane returned.

   Typed by the lane's LIVE shape rather than frozen under a version 2 name,
   unlike the unions a reader dispatches on. These are evidence: they are
   large, they grow by addition, and a reader takes the fields it knows and
   leaves the rest. A change that REMOVES or REPURPOSES a field here is a
   version 3 all the same.
   */
  readonly result: TResult;

  /**
   One row per prepared slice, saying what this lane's document carries.
   */
  readonly delivery: readonly ArtifactDeliveryRow[];
};

/**
 Everything one settled entry records, once both lanes have run.

 @example
 ```ts
 const artifact: SettledArtifact = buildSettledTwoLaneArtifact({ ... },);
 ```
 */
export type SettledArtifact = {
  /**
   Which generation this is, stated rather than inferred from which fields
   happen to be present.
   */
  readonly artifactSchemaVersion: typeof ARTIFACT_SCHEMA_VERSION_V22;

  /**
   Corpus entry this covers.
   */
  readonly id: string;

  /**
   Repository head when the pass started, as PROVENANCE: it says where the
   code came from and never what ran.
   */
  readonly tip: string;

  /**
   Built output that ran, as IDENTITY: this is the field two artifacts must
   share before their results may be pooled.
   */
  readonly pipelineDigest: PipelineDigest;

  /**
   Corpus commit the two texts were read at.
   */
  readonly corpusSha: string;

  /**
   Model call configuration this run used, whose inside this schema
   deliberately does not describe.
   */
  readonly callConfig: Readonly<Record<string, ArtifactJsonValue>>;

  /**
   Time the entry took, both lanes included, on `monotonicMs`.
   */
  readonly durationMs: number;

  /**
   When the artifact was written.
   */
  readonly timestamp: string;

  /**
   Slicing both lanes ran over.
   */
  readonly preparation: SettledPreparation;

  /**
   The two lanes, nested and unordered in meaning: neither is the output.
   */
  readonly lanes: {
    /**
     Lane that mends the archive's English.
     */
    readonly repair: SettledLane<RepairTranslationResult>;

    /**
     Lane that renders every slice afresh from the original.
     */
    readonly translate: SettledLane<TranslateDocumentResult>;
  };

  /**
   The two lanes compared slice by slice, derived rather than supplied.

   Persisted for the reader's convenience and RECOMPUTED by any reader that
   cares, which then refuses a disagreement: a stored comparison is a claim
   about two ledgers that are stored beside it, so nothing has to trust it.
   */
  readonly comparison: readonly ArtifactComparisonRow[];

  /**
   Which lane ships, as the roster settled it or as nobody having asked.

   A STATED PENDING STATE rather than an absent field. Leaving it out would
   make "no decision yet" and "this artifact predates the question" the same
   absence, which is the defect class this whole generation exists to end.
   */
  readonly laneSelection: ArtifactLaneSelection;

  /**
   What the consolidation settled, or a stated absence saying it never ran.

   SEPARATE FROM `laneSelection` because it answers a later question. The
   contest picks between the two lanes; this asks whether a rendering neither
   lane produced is better than the winner, and it runs only where the contest
   already settled something.
   */
  readonly consolidation: ArtifactConsolidation;

  /**
   What the page-level assembly guard did to the composed page.
   */
  readonly pageAssembly: ArtifactPageAssembly;

  /**
   What the page-level consistency pass did to the settled page: the
   repeated wording it found, the ballots it counted, and the slices it changed.
   Written from generation fifteen on.
   */
  readonly consistency: ArtifactConsistency;
};

//endregion Artifact version 2 contract
