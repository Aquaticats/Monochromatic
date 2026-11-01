import { ConsistencyInvariantError, } from '@monochromatic-dev/translation-repair-consistency/ts';
import type {
  ArtifactConsistencyCarry,
  ArtifactConsistencyGroup,
  ArtifactConsistencyOccurrence,
  ConsistencyOccurrenceOutcome,
} from '@monochromatic-dev/translation-repair-artifact/ts';

//region Pass consistency group line
// THE LINE THE PASS LOGS FOR EACH GROUP, with every outcome counted and the reason named where a place was left as it stood
// (n161, R2). The settled TianqiChen66624 run logged `group 1 of 2: refrain verdict=asked placed=4 unplaced=0 replaced=0
// consistent=0 kept-context=0 kept-floor=0 overlapped=0 unlocated=0`: the bench was asked and nothing changed, and the line said
// nothing of why, because `chosen`, `kept-current` and `untouched` had no column and the reasons the record holds were not printed.
// The line now counts all nine outcomes, so the columns add up to the places placed, and ends with `because=` and each distinct
// reason, in the order the places stand, whenever a place ended `kept-current` or the bench was not asked, failed or named no winner.
// Every reason is the code's own decision, read from the record and not decided again here:
//
// - `not-asked:too-few-placed`, `not-asked:no-english`: the bench was not asked (`pass-consistency-unasked.ts`). A group already
//   consistent is not asked either, and its places are counted `consistent`, so it names no reason.
// - `failed:<detail>`: trouble stopped the group before the ballots could be counted; the detail names no corpus text.
// - `no-winner:<reason>`: the bench was asked and no place won (`NoWinnerReason`: `no-quorum`, `no-majority`, `chosen-unlocated`,
//   `run-off-no-quorum`, `run-off-tied`); every place stays as it stood.
// - `carry:<reason>`: a place was to receive the chosen rendering and the carry (`pass-consistency-carry-choose.ts`) left its English
//   as it stood: `no-alternative` (no text but the current one to choose among), `no-quorum`, `no-majority` (the choice among the
//   candidates reached no decision), `failed` (trouble stopped the carry), `chose-current` (the choice kept the English as it stands),
//   `splice-cannot-reach-place` (the choice was the code splice, and this place has no words code can move the chosen words into, as
//   a frame the splice cannot carry), `unrecorded` (the record holds no carry for the place, whatever reason the pass recorded).
// - `kept:<reason>`: the pass left the place as it stood, a splice was offered to the block of the place, and the record's words let
//   code move the chosen words in only in part or not at all (`ConsistencyKeptReason`: the `gain-` and `edge-` reasons). It is named
//   ahead of the carry's own decision for a place whose carry offered the splice. A place whose carry offered no splice (the writers
//   and the current text alone, or no carry at all) never prints one, since the reason answers a splice that was never offered; the
//   carry's own decision stands for it.
//
// A RECORD THE PASS CANNOT PRODUCE THROWS rather than printing a reason the record does not hold: a carry that chose a label no
// candidate of the record holds, a writer's text chosen for a place that ended `kept-current` (a writer's text that is chosen replaces
// every receiving place of its block), and a verdict that named no winner beside a replaced place each throw a
// `ConsistencyInvariantError` naming the group and the place.

/**
 The outcomes counted in the columns of the line, in the order they are printed: the six the line has always counted, then the three
 it did not.
 */
const COLUMNS: readonly ConsistencyOccurrenceOutcome[] = [
  'replaced',
  'consistent',
  'kept-context',
  'kept-floor',
  'overlapped',
  'unlocated',
  'chosen',
  'kept-current',
  'untouched',
];

/**
 How many places of a group ended in an outcome.

 @param group - the group with its final outcomes

 @param outcome - outcome counted

 @returns Number of places with that outcome

 @example
 ```ts
 const replaced = placesEndingIn({ group, outcome: 'replaced', },);
 ```
 */
function placesEndingIn(
  {
    group,
    outcome,
  }: {
    readonly group: ArtifactConsistencyGroup;
    readonly outcome: ConsistencyOccurrenceOutcome;
  },
): number {
  return group.occurrences
    .filter(function ended(occurrence,): boolean {
      return occurrence.outcome === outcome;
    },)
    .length;
}

/**
 The reason the verdict of a group gives for places left as they stood, none when the verdict gives none.

 @param group - the group with its final outcomes

 @param groupNumber - one-based number of the group, which names it in a thrown error

 @returns The reason as a list of zero or one

 @throws {@link ConsistencyInvariantError} when the verdict named no winner and a place ended replaced, since a group whose bench
 named no winner replaces no place

 @example
 ```ts
 const [reason,] = verdictReasonOf({ group, groupNumber: 1, },); // 'no-winner:no-quorum'
 ```
 */
function verdictReasonOf(
  {
    group,
    groupNumber,
  }: {
    readonly group: ArtifactConsistencyGroup;
    readonly groupNumber: number;
  },
): readonly string[] {
  /**
   What was asked of the bench and what it answered.
   */
  const { verdict, } = group;
  if (verdict.kind === 'failed')
    return [`failed:${verdict.detail}`,];
  if (verdict.kind === 'not-asked')
    return (verdict.reason === 'consistent') ? [] : [`not-asked:${verdict.reason}`,];
  /**
   The place every place followed, or why none.
   */
  const { winner, } = verdict;
  if (winner.kind !== 'none')
    return [];
  /**
   The first place that ended replaced although no place won, none when every place stood.
   */
  const [replaced,] = group.occurrences
    .filter(function ended(occurrence,): boolean {
      return occurrence.outcome === 'replaced';
    },);
  if (replaced !== undefined) {
    throw new ConsistencyInvariantError({
      invariant: `group ${String(groupNumber,)} holds a verdict that named no winner (${winner.reason}) and ${replaced.label} ended `
        + 'replaced, and a group whose bench named no winner replaces no place',
    },);
  }
  return [`no-winner:${winner.reason}`,];
}

/**
 The reason the carry gives for a place whose English stood: what the carry to the place decided.

 @param carries - the carries the record holds for the place: one, or none where the record holds none

 @param groupNumber - one-based number of the group, which names it in a thrown error

 @param label - label of the place

 @returns The reason

 @throws {@link ConsistencyInvariantError} when the carry chose a label none of its candidates holds, or chose a writer's text, which
 replaces every receiving place of its block and so cannot leave this place as it stood

 @example
 ```ts
 const reason = carryReasonOf({ carries: [carry,], groupNumber: 1, label: 'OCCURRENCE 2', },); // 'carry:no-majority'
 ```
 */
function carryReasonOf(
  {
    carries,
    groupNumber,
    label,
  }: {
    readonly carries: readonly ArtifactConsistencyCarry[];
    readonly groupNumber: number;
    readonly label: string;
  },
): string {
  /**
   The carry to the place, none where the record holds none.
   */
  const [carry,] = carries;
  if (carry === undefined)
    return 'carry:unrecorded';
  /**
   What the carry decided.
   */
  const { decision, } = carry;
  if (decision.kind === 'failed')
    return 'carry:failed';
  if (decision.kind === 'none')
    return `carry:${decision.reason}`;
  /**
   The kind of text the carry chose.
   */
  const [chosen,] = carry.candidates
    .filter(function isChosen(candidate,): boolean {
      return candidate.label === decision.label;
    },);
  if (chosen === undefined) {
    /**
     The candidates the carry recorded.
     */
    const { candidates, } = carry;
    /**
     How many candidates the carry recorded.
     */
    const recorded = candidates.length;
    throw new ConsistencyInvariantError({
      invariant: `group ${String(groupNumber,)} ${label} ended kept-current and its carry chose ${decision.label}, which is not `
        + `among the candidates the carry recorded (${String(recorded,)} recorded)`,
    },);
  }
  if (chosen.kind === 'current')
    return 'carry:chose-current';
  if (chosen.kind === 'splice')
    return 'carry:splice-cannot-reach-place';
  throw new ConsistencyInvariantError({
    invariant: `group ${String(groupNumber,)} ${label} ended kept-current and its carry chose ${decision.label}, a writer text, and `
      + 'a writer text that is chosen replaces every receiving place of its block',
  },);
}

/**
 The reason a place that ended `kept-current` was left as it stood: the reason the pass recorded for the place where the carry to
 it offered the splice, else what the carry to it decided. The pass records an edge reason for every receiving place whatever the
 carry was offered, so a recorded reason answers a splice only where one was offered; where the carry offered the writers and the
 current text alone, the carry's own decision is what ended the place.

 @param group - the group with its final outcomes

 @param groupNumber - one-based number of the group, which names it in a thrown error

 @param occurrence - the place

 @returns The reason

 @throws {@link ConsistencyInvariantError} when the carry to the place chose what the record cannot explain

 @example
 ```ts
 const reason = keptReasonOf({ group, groupNumber: 1, occurrence, },); // 'kept:gain-pairs'
 ```
 */
function keptReasonOf(
  {
    group,
    groupNumber,
    occurrence,
  }: {
    readonly group: ArtifactConsistencyGroup;
    readonly groupNumber: number;
    readonly occurrence: ArtifactConsistencyOccurrence;
  },
): string {
  /**
   The carry to the place: one, or none where the record holds none.
   */
  const carries = (group.carries ?? []).filter(function isForPlace(candidate,): boolean {
    return candidate.label === occurrence.label;
  },);
  /**
   Whether the carry offered the splice among its candidates.
   */
  const offeredSplice = carries.some(function offers({ candidates, },): boolean {
    return candidates.some(function isSplice(candidate,): boolean {
      return candidate.kind === 'splice';
    },);
  },);
  if ((occurrence.keptBecause !== undefined) && offeredSplice)
    return `kept:${occurrence.keptBecause}`;
  return carryReasonOf({
    carries,
    groupNumber,
    label: occurrence.label,
  },);
}

/**
 The distinct reasons a group's places were left as they stood, in the order the places stand, the verdict's first.

 @param group - the group with its final outcomes

 @param groupNumber - one-based number of the group, which names it in a thrown error

 @returns The reasons

 @throws {@link ConsistencyInvariantError} when the record holds a verdict or a carry the pass cannot produce

 @example
 ```ts
 const reasons = reasonsOf({ group, groupNumber: 1, },); // ['carry:no-majority']
 ```
 */
function reasonsOf(
  {
    group,
    groupNumber,
  }: {
    readonly group: ArtifactConsistencyGroup;
    readonly groupNumber: number;
  },
): readonly string[] {
  return [
    ...new Set([
      ...verdictReasonOf({
        group,
        groupNumber,
      },),
      ...group.occurrences
        .filter(function stood(occurrence,): boolean {
          return occurrence.outcome === 'kept-current';
        },)
        .map(function reasonOf(occurrence,): string {
          return keptReasonOf({
            group,
            groupNumber,
            occurrence,
          },);
        },),
    ],),
  ];
}

/**
 One line saying how a group came out: its repeat, the verdict, the places placed and not placed, the count of each outcome, and,
 where a place was left as it stood or the bench was not asked, failed or named no winner, the reasons.

 @param group - the group with its final outcomes

 @param groupNumber - one-based number of the group, as the pass numbers its groups in the log, which names the group in a thrown error

 @returns Log line text

 @throws {@link ConsistencyInvariantError} naming the group and the place when the record holds what the pass cannot produce: a carry
 that chose a label no candidate holds, a writer's text chosen for a place that ended `kept-current`, or a verdict that named no
 winner beside a replaced place

 @example
 ```ts
 const line = groupLine({ group, groupNumber: 1, },);
 ```
 */
export function groupLine(
  {
    group,
    groupNumber,
  }: {
    readonly group: ArtifactConsistencyGroup;
    readonly groupNumber: number;
  },
): string {
  /**
   The parts of the group the line names.
   */
  const {
    repeat,
    verdict,
    occurrences,
    unplaced,
  } = group;
  /**
   The reasons, none for a group whose places a column counts.
   */
  const reasons = reasonsOf({
    group,
    groupNumber,
  },);
  return [
    repeat.kind,
    `verdict=${verdict.kind}`,
    `placed=${String(occurrences.length,)}`,
    `unplaced=${String(unplaced.length,)}`,
    ...COLUMNS.map(function column(outcome,): string {
      return `${outcome}=${String(placesEndingIn({
        group,
        outcome,
      },),)}`;
    },),
    ...((reasons.length === 0) ? [] : [`because=${reasons.join(',',)}`,]),
  ].join(' ',);
}

//endregion Pass consistency group line
