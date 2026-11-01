import {
  type ConsistencyCount,
  ConsistencyInvariantError,
  consistencyLabel,
  countConsistency,
  usableVoicesOf,
} from '@monochromatic-dev/translation-repair-consistency/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import type {
  ArtifactConsistencyBallot,
  ArtifactConsistencyCarry,
  ArtifactConsistencyRepeat,
  ArtifactConsistencyUnplaced,
  ConsistencyOccurrenceOutcome,
} from '@monochromatic-dev/translation-repair-artifact/ts';
import type { ConsistencyAsked, } from './pass-consistency-ask.ts';
import {
  afterRunOff,
  chosenPlaceOf,
  plannedPlace,
} from './pass-consistency-decide.ts';
import type { EdgeVerdict, } from './pass-consistency-edge-refusal.ts';
import type { ConsistencyPlace, } from './pass-consistency-group.ts';
import type { ConsistencyRunOffAsked, } from './pass-consistency-run-off.ts';
import type {
  CarryReceiver,
  DecidedGroup,
  PendingEdit,
  SettledGroup,
  UnitCarried,
} from './pass-consistency-types.ts';

//region Pass consistency settle
// WHAT A GROUP COMES TO once its ballots are counted and a split with no strict majority is run
// off among its finalists: a verdict, a rendering and a provisional outcome for each place, and the
// places the chosen rendering is still to be carried to (`receivers`). The carry
// (`pass-consistency-carry.ts`) turns the receivers of each block into a record
// for every one of them and, when the bench chooses a text other than the block's
// own, a replacement. The outcomes become final in `pass-consistency-apply.ts`,
// where replacements meet each other and the floors.

/**
 A ballot as the artifact records it.

 @param voice - ballot with its seat

 @returns The recorded ballot

 @example
 ```ts
 const ballot = recordedBallot({ voice, },);
 ```
 */
function recordedBallot(
  { voice, }: { readonly voice: ConsistencyAsked['voices'][number]; },
): ArtifactConsistencyBallot {
  return {
    seat: voice.seat,
    reason: voice.reason,
    quotes: voice.quotes
      .map(function recordedQuote(quote,): ArtifactConsistencyBallot['quotes'][number] {
        return {
          label: quote.label,
          quote: quote.quote,
          ...((quote.slot === undefined) ? {} : { slot: quote.slot, }),
        };
      },),
    choice: voice.choice,
    keep: voice.keep,
  };
}

/**
 Why the bench was asked once more: the choices reached no strict majority, and the finalists
 are named with how many of the usable ballots named each.

 @param count - what the first round's ballots came to, with finalists

 @param asked - the ballots heard

 @returns One sentence naming the finalists and their ballots, empty when the choices have no finalists

 @example
 ```ts
 const reason = runOffReasonOf({ count, asked, },); // 'the choices reached no strict majority, so the finalists are OCCURRENCE 3 (2 ballots) and OCCURRENCE 4 (2 ballots), of 4 usable in all'
 ```
 */
export function runOffReasonOf(
  {
    count,
    asked,
  }: {
    readonly count: ConsistencyCount;
    readonly asked: ConsistencyAsked;
  },
): string {
  if (count.finalists
    .length
    === 0)
    return '';
  /**
   The usable ballots, which the counts are over.
   */
  const usable = usableVoicesOf({
    voices: asked.voices,
    labels: count.renderings
      .map(function labelOf(
        _rendering,
        position,
      ): string {
        return consistencyLabel({ position, },);
      },),
  },);
  /**
   Each finalist with the ballots that named it.
   */
  const named = count.finalists
    .map(function namedOf(label,): string {
      /**
       Ballots that chose this place.
       */
      const ballots = usable.filter(function choosesIt(voice,): boolean {
        return voice.choice === label;
      },)
        .length;
      return `${label} (${String(ballots,)} ${wordForCount({
        count: ballots,
        one: 'ballot',
        many: 'ballots',
      },)})`;
    },);
  /**
   Every finalist but the last.
   */
  const beforeLast = named.toSpliced(-1,);
  /**
   The finalists as a list in prose: "A and B", or "A, B and C".
   */
  const listed = (beforeLast.length === 0) ? (named[0] ?? '') : `${beforeLast.join(', ',)} and ${named.at(-1,) ?? ''}`;
  return `the choices reached no strict majority, so the finalists are ${listed}, of ${String(usable.length,)} usable in all`;
}

/**
 Counts the first round's ballots over the places.

 @param repeat - the repeated wording or frame

 @param places - the places a slice holds whole

 @param asked - the ballots heard, the quorum and the gather's findings

 @returns What the usable ballots come to, the finalists of a split with no strict majority included

 @example
 ```ts
 const count = countAsked({ repeat, places, asked, },);
 ```
 */
export function countAsked(
  {
    repeat,
    places,
    asked,
  }: {
    readonly repeat: ArtifactConsistencyRepeat;
    readonly places: readonly ConsistencyPlace[];
    readonly asked: ConsistencyAsked;
  },
): ConsistencyCount {
  return countConsistency({
    voices: asked.voices,
    places: places.map(function placeOf(
      place,
      position,
    ) {
      return {
        label: consistencyLabel({ position, },),
        english: place.slice
          .current,
        sliceIndex: place.slice
          .sliceIndex,
        // THE PROOF OF AN APOSTROPHE READS THE PAGE the place's slice belongs to, not the slice alone.
        ...((place.slice
          .page
          === undefined) ? {} : { page: place.slice
            .page, }),
      };
    },),
    framed: repeat.kind === 'refrain',
    quorum: asked.quorum,
  },);
}

/**
 Counts what the bench said about a group and decides what each place is to
 receive.

 @param repeat - the repeated wording or frame

 @param places - the places a slice holds whole

 @param unplaced - the places no slice holds whole

 @param asked - the ballots heard, the quorum and the gather's findings

 @param runOff - what the run-off heard when the choices had finalists

 @returns The group with its verdict, renderings and provisional outcomes, and the places the chosen rendering is still to be carried to

 @throws {@link ConsistencyInvariantError} when the winner is not a located place of the group

 @example
 ```ts
 const decided = settleAsked({ repeat, places, unplaced, asked, },);
 ```
 */
export function settleAsked(
  {
    repeat,
    places,
    unplaced,
    asked,
    runOff,
  }: {
    readonly repeat: ArtifactConsistencyRepeat;
    readonly places: readonly ConsistencyPlace[];
    readonly unplaced: readonly ArtifactConsistencyUnplaced[];
    readonly asked: ConsistencyAsked;
    readonly runOff?: ConsistencyRunOffAsked;
  },
): DecidedGroup {
  /**
   What the usable ballots come to.
   */
  const count = countAsked({
    repeat,
    places,
    asked,
  },);
  /**
   Labels of the places, in order.
   */
  const labels = places.map(function labelOf(
    _place,
    position,
  ): string {
    return consistencyLabel({ position, },);
  },);
  /**
   The winner after any run-off, and the run-off as recorded.
   */
  const ruled = afterRunOff({
    count,
    labels,
    runOff: (runOff === undefined) ? [] : [runOff,],
  },);
  /**
   The winning place with its located rendering, as a list of zero or one.
   */
  const chosen = chosenPlaceOf({
    places,
    renderings: count.renderings,
    position: labels.findIndex(function isWinner(label,): boolean {
      return (ruled.winner
        .kind
        === 'chosen') && (ruled.winner
          .label
          === label);
    },),
  },);
  /**
   Every place's provisional outcome, and the receiving places among them.
   */
  const planned = places.map(function planFor(
    place,
    position,
  ) {
    return plannedPlace({
      chosen,
      place,
      position,
      count,
      framed: repeat.kind === 'refrain',
    },);
  },);
  return {
    repeat,
    places,
    unplaced,
    verdict: {
      kind: 'asked',
      ballots: asked.voices
        .map(function toBallot(voice,): ArtifactConsistencyBallot {
          return recordedBallot({ voice, },);
        },),
      usable: count.usable,
      quorum: asked.quorum,
      winner: ruled.winner,
      ...((ruled.runOff[0] === undefined) ? {} : { runOff: ruled.runOff[0], }),
    },
    renderings: count.renderings,
    outcomes: planned.map(function outcomeOf(entry,): ConsistencyOccurrenceOutcome {
      return entry.outcome;
    },),
    ...(planned.some(function isRefused({ edge, },): boolean {
      return edge !== undefined;
    },)
      ? {
        leftBecause: planned.map(function edgeOf({ edge, },): EdgeVerdict {
          return edge ?? 'unrefused';
        },),
      }
      : {}),
    edits: [],
    carries: [],
    findings: [
      ...asked.findings,
      ...((runOff === undefined)
        ? []
        : [
          `run-off asked: ${
            runOffReasonOf({
              count,
              asked,
            },)
          }`,
        ]),
      ...(runOff?.findings ?? []),
    ],
    chosen,
    receivers: planned.flatMap(function receiverOf(entry,): readonly CarryReceiver[] {
      return entry.receiver;
    },),
  };
}

/**
 Settles a decided group once its receiving places are carried to.

 @param decided - the group with its receivers

 @param carried - how the carry to each block of receiving places ended, in any order

 @returns The group with the carries' records in the order of the receivers, the replacements the chosen candidates make, and each receiver's provisional outcome

 @throws {@link ConsistencyInvariantError} when the carries are not one record per receiver

 @example
 ```ts
 const settled = settleCarried({ decided, carried, },);
 ```
 */
export function settleCarried(
  {
    decided,
    carried,
  }: {
    readonly decided: DecidedGroup;
    readonly carried: readonly UnitCarried[];
  },
): SettledGroup {
  /**
   Every receiving place's record of the carry, in the order the blocks were carried.
   */
  const records = carried.flatMap(function recordsOf(entry,): readonly ArtifactConsistencyCarry[] {
    return entry.carries;
  },);
  if (records.length
    !== decided.receivers
      .length) {
    throw new ConsistencyInvariantError({ invariant:
      `the pass carries to each receiving place once, and the number of carries differs from the number of receiving places (carried: ${String(records.length,)}, receiving: ${String(decided.receivers
        .length,)})`, },);
  }
  /**
   The records in the order of the receivers, each place's own.
   */
  const ordered = decided.receivers
    .map(function recordFor(receiver,): ArtifactConsistencyCarry {
      /**
       The records that name this receiver.
       */
      const [own, ...others] = records.filter(function namesIt({ label, },): boolean {
        return label === receiver.label;
      },);
      if ((own === undefined) || (others.length > 0)) {
        throw new ConsistencyInvariantError({ invariant:
          `the pass carries to each receiving place once, and the records that name ${receiver.label} number ${String(others.length + ((own === undefined) ? 0 : 1),)}`, },);
      }
      return own;
    },);
  /**
   The positions of the places whose carry makes a replacement.
   */
  const replacing = new Set(carried.flatMap(function positionsOf(entry,): readonly number[] {
    return entry.edits
      .flatMap(function positionOf(edit,): readonly number[] {
        return edit.placePositions;
      },);
  },),);
  return {
    repeat: decided.repeat,
    places: decided.places,
    unplaced: decided.unplaced,
    verdict: decided.verdict,
    renderings: decided.renderings,
    outcomes: decided.outcomes
      .map(function outcomeOf(
        outcome,
        position,
      ): ConsistencyOccurrenceOutcome {
        return replacing.has(position,) ? 'replaced' : outcome;
      },),
    ...((decided.leftBecause === undefined) ? {} : { leftBecause: decided.leftBecause, }),
    edits: carried.flatMap(function editsOf(entry,): readonly PendingEdit[] {
      return entry.edits;
    },),
    carries: ordered,
    findings: [
      ...decided.findings,
      ...carried.flatMap(function findingsOf(entry,): readonly string[] {
        return entry.findings;
      },),
    ],
  };
}

//endregion Pass consistency settle
