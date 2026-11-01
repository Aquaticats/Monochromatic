import {
  type ConsistencyEdit,
  ConsistencyInvariantError,
  consistencyLabel,
  type ConsistencyRendering,
} from '@monochromatic-dev/translation-repair-consistency/ts';
import type {
  ArtifactConsistencyGroup,
  ArtifactConsistencyOccurrence,
  ArtifactConsistencyPlaceName,
  ArtifactConsistencyRendering,
  ConsistencyKeptReason,
  ConsistencyOccurrenceOutcome,
} from '@monochromatic-dev/translation-repair-artifact/ts';
import type {
  GroupEdit,
  SettledIndex,
} from './pass-consistency-apply-index.ts';
import type { SettledGroup, } from './pass-consistency-types.ts';

//region Pass consistency apply record
// HOW APPLYING THE REPLACEMENTS NAMES WHAT IT RECORDS, kept apart from the
// applying itself (`pass-consistency-apply.ts`): the name a replacement is
// reported under, the key a place's outcome is kept under, the name of the place
// an overlapped place gave way to, and the groups as the artifact records them
// once every place has its final outcome.

/**
 The name a replacement is reported under, in the words an operator reads the
 pass's log in: the group counted from one and the labels of the receiving
 places it carries.

 @param group - position of the group on the page, from zero

 @param places - positions of the receiving places in the group, from zero

 @returns Name unique across the page's replacements

 @example
 ```ts
 const id = editId({ group: 0, places: [1, 2,], },); // 'group 1 OCCURRENCE 2 and OCCURRENCE 3'
 ```
 */
export function editId(
  {
    group,
    places,
  }: {
    readonly group: number;
    readonly places: readonly number[];
  },
): string {
  return `group ${String(group + 1,)} ${
    places
      .map(function labelOf(place,): string {
        return consistencyLabel({ position: place, },);
      },)
      .join(' and ',)
  }`;
}

/**
 The key a place's final outcome is kept under.

 @param group - position of the group on the page, from zero

 @param place - position of the place in the group, from zero

 @returns The key

 @example
 ```ts
 const key = placeKey({ group: 0, place: 1, },); // '0:1'
 ```
 */
export function placeKey(
  {
    group,
    place,
  }: {
    readonly group: number;
    readonly place: number;
  },
): string {
  return `${String(group,)}:${String(place,)}`;
}

/**
 A rendering as the artifact records it.

 @param rendering - the counter's rendering

 @returns The recorded rendering

 @example
 ```ts
 const recorded = recordedRendering({ rendering, },);
 ```
 */
export function recordedRendering(
  { rendering, }: { readonly rendering: ConsistencyRendering; },
): ArtifactConsistencyRendering {
  if (rendering.kind === 'unlocated') {
    return {
      kind: 'unlocated',
      reason: rendering.reason,
    };
  }
  return {
    kind: 'located',
    quote: rendering.quote,
    start: rendering.start,
    ...((rendering.markBefore === undefined) ? {} : { markBefore: rendering.markBefore, }),
    ...((rendering.markAfter === undefined) ? {} : { markAfter: rendering.markAfter, }),
    ...((rendering.slot === undefined) ? {} : { slot: rendering.slot, }),
    ...((rendering.definitions === undefined) ? {} : { definitions: rendering.definitions, }),
  };
}

/**
 A pending replacement with the name it is reported under.

 @example
 ```ts
 const named: NamedEdit = { id: 'group 1 OCCURRENCE 2', group: 0, edit, };
 ```
 */
type NamedEdit = GroupEdit & {
  /**
   Name of the replacement.
   */
  readonly id: string;
};

/**
 One slice's replacements as the applying reads them.

 @example
 ```ts
 const named: SliceEdits = { edits: [], carriers: new Map(), pendings: new Map(), };
 ```
 */
export type SliceEdits = {
  /**
   The slice's replacements, earlier groups first.
   */
  readonly edits: readonly ConsistencyEdit[];

  /**
   Keys of the places each replacement carries, by the replacement's name.
   */
  readonly carriers: ReadonlyMap<string, readonly string[]>;

  /**
   Each pending replacement with the position of its group, by the replacement's name.
   */
  readonly pendings: ReadonlyMap<string, GroupEdit>;
};

/**
 Two replacements of one group whose words overlap.

 @example
 ```ts
 const pair: OverlappingPair = { first, second, };
 ```
 */
type OverlappingPair = {
  /**
   The earlier replacement.
   */
  readonly first: NamedEdit;

  /**
   The later replacement.
   */
  readonly second: NamedEdit;
};

/**
 Whether the words of two replacements overlap: two spans that only touch do not,
 as in the arbitration of `applyConsistencyEdits`.

 @param left - one replacement's span

 @param right - the other replacement's span

 @returns Whether a word lies in both

 @example
 ```ts
 const meet = spansOverlap({ left: { start: 0, end: 20, }, right: { start: 10, end: 40, }, },); // true
 ```
 */
function spansOverlap(
  {
    left,
    right,
  }: {
    readonly left: Pick<ConsistencyEdit, 'start' | 'end'>;
    readonly right: Pick<ConsistencyEdit, 'start' | 'end'>;
  },
): boolean {
  return (left.start < right.end) && (right.start < left.end);
}

/**
 The first pair of one group's replacements whose words overlap, none when no two
 of a group meet.

 @param named - a slice's replacements with their names and groups

 @returns The pair as a list of zero or one, the earlier replacement first

 @example
 ```ts
 const [pair,] = overlappingInOneGroup({ named, },);
 ```
 */
function overlappingInOneGroup(
  { named, }: { readonly named: readonly NamedEdit[]; },
): readonly OverlappingPair[] {
  return named
    .flatMap(function pairsWithLater(
      first,
      at,
    ): readonly OverlappingPair[] {
      return named
        .slice(at + 1,)
        .filter(function overlapsFirst(second,): boolean {
          return (second.group === first.group) && spansOverlap({
            left: first.edit,
            right: second.edit,
          },);
        },)
        .map(function pairOf(second,): OverlappingPair {
          return {
            first,
            second,
          };
        },);
    },)
    .slice(
    0,
    1,
    );
}

/**
 The replacements of one slice, named, with the places each carries and the
 group each belongs to.

 @param index - the page's groups read by slice

 @param sliceIndex - the slice

 @returns The slice's replacements, earlier groups first, with what ties each name back to its places and group

 @throws {@link ConsistencyInvariantError} when two replacements of one group overlap in the slice, naming both, since the places of one group that a block holds are carried by one replacement and the arbitration would record the later place as overlapped by its own group

 @example
 ```ts
 const { edits, carriers, pendings, } = sliceEditsOf({ index, sliceIndex: 2, },);
 ```
 */
export function sliceEditsOf(
  {
    index,
    sliceIndex,
  }: {
    readonly index: SettledIndex;
    readonly sliceIndex: number;
  },
): SliceEdits {
  /**
   The replacements addressed to each slice.
   */
  const { edits: indexedEdits, } = index;
  /**
   The replacements addressed to this slice, earlier groups first.
   */
  const addressed = indexedEdits.get(sliceIndex,) ?? [];
  /**
   The slice's replacements with their names, earlier groups first.
   */
  const named = addressed.map(function nameOf({
    group: groupPosition,
    edit,
  },): NamedEdit {
    return {
      id: editId({
        group: groupPosition,
        places: edit.placePositions,
      },),
      group: groupPosition,
      edit,
    };
  },);
  /**
   The first two replacements of one group whose words overlap, none when the group's replacements are apart.
   */
  const [overlapping,] = overlappingInOneGroup({ named, },);
  if (overlapping !== undefined) {
    /**
     The two replacements, whose names carry the group and the places.
     */
    const {
      first,
      second,
    } = overlapping;
    throw new ConsistencyInvariantError({ invariant:
      `the replacements ${first.id} and ${second.id} of slice ${String(sliceIndex,)} overlap, `
        + 'and no two replacements of one group address the same words', },);
  }
  return {
    edits: named.map(function editOf({
      id,
      edit,
    },): ConsistencyEdit {
      return {
        id,
        start: edit.start,
        end: edit.end,
        text: edit.text,
      };
    },),
    carriers: new Map(named.map(function carrierOf({
      id,
      group,
      edit,
    },): readonly [
      string,
      readonly string[],
    ] {
      return [
        id,
        edit.placePositions
          .map(function keyOf(place,): string {
            return placeKey({
              group,
              place,
            },);
          },),
      ];
    },),),
    pendings: new Map(named.map(function pendingOf({
      id,
      group,
      edit,
    },): readonly [
      string,
      GroupEdit,
    ] {
      return [
        id,
        {
          group,
          edit,
        },
      ];
    },),),
  };
}

/**
 The pending replacement a name stands for.

 @param pendings - the slice's pending replacements by name

 @param id - name of a replacement that was made

 @returns The replacement with its group's position

 @throws {@link ConsistencyInvariantError} when no pending replacement has the name

 @example
 ```ts
 const pending = pendingNamed({ pendings, id: 'group 1 OCCURRENCE 2', },);
 ```
 */
export function pendingNamed(
  {
    pendings,
    id,
  }: {
    readonly pendings: ReadonlyMap<string, GroupEdit>;
    readonly id: string;
  },
): GroupEdit {
  /**
   The pending replacement under that name.
   */
  const pending = pendings.get(id,);
  if (pending === undefined) {
    throw new ConsistencyInvariantError({ invariant:
      `edit ${id} was made and no pending replacement has its name, and every edit made is a pending replacement`, },);
  }
  return pending;
}

/**
 The name a replacement's place is given when another place gave way to it: the
 position of its group and the label of the first receiving place it carries.

 @param group - position of the replacement's group on the page, from zero

 @param edit - the replacement

 @returns The group and label of the place

 @throws {@link ConsistencyInvariantError} when the replacement carries no receiving place

 @example
 ```ts
 const name = carriedPlaceName({ group: 0, edit, },);
 ```
 */
export function carriedPlaceName(
  {
    group,
    edit,
  }: GroupEdit,
): ArtifactConsistencyPlaceName {
  /**
   The first receiving place the replacement carries.
   */
  const [first,] = edit.placePositions;
  if (first === undefined) {
    throw new ConsistencyInvariantError({ invariant:
      `a replacement of group ${String(group + 1,)} carries no receiving place, and every replacement carries at least one`, },);
  }
  return {
    group,
    label: consistencyLabel({ position: first, },),
  };
}

/**
 The replacement that won the words a later replacement overlapped: the first one
 made, in priority order, whose span meets the later one's.

 @param loser - the replacement left out for overlapping an earlier one

 @param made - the replacements that were made, in priority order

 @returns The replacement that won

 @throws {@link ConsistencyInvariantError} when no replacement that was made meets the one left out

 @example
 ```ts
 const winner = winnerOver({ loser, made, },);
 ```
 */
export function winnerOver(
  {
    loser,
    made,
  }: {
    readonly loser: ConsistencyEdit;
    readonly made: readonly ConsistencyEdit[];
  },
): ConsistencyEdit {
  /**
   The first replacement made whose span meets the one left out.
   */
  const winner = made.find(function meets(other,): boolean {
    return (loser.start < other.end) && (other.start < loser.end);
  },);
  if (winner === undefined) {
    throw new ConsistencyInvariantError({ invariant:
      `edit ${loser.id} was left out for overlapping an earlier edit, and no edit that was made overlaps it`, },);
  }
  return winner;
}

/**
 The groups as the artifact records them: every place with its label, slice,
 offsets, line, rendering and final outcome, and for an overlapped place the place
 it gave way to.

 @param settled - the groups in page order, with their provisional outcomes

 @param outcomes - what the replacements and the floors made of each place a replacement carried, by `placeKey`

 @param gaveWayTo - the place each overlapped place gave way to, by `placeKey`

 @param keptBecause - why the pass left a place as it was after a code splice it carried was left out, by `placeKey`; the
 reason the group holds of a splice refused at the edge stands for a place this does not name; none by default

 @returns The recorded groups, in the order of the settled ones

 @throws {@link ConsistencyInvariantError} when a place has no rendering or outcome, a place to be
 replaced was neither made, left out nor refused, or a place ended overlapped with no place it gave way to

 @example
 ```ts
 const groups = finalGroupsOf({ settled, outcomes: new Map(), gaveWayTo: new Map(), },);
 ```
 */
export function finalGroupsOf(
  {
    settled,
    outcomes,
    gaveWayTo,
    keptBecause = new Map<string, ConsistencyKeptReason>(),
  }: {
    readonly settled: readonly SettledGroup[];
    readonly outcomes: ReadonlyMap<string, ConsistencyOccurrenceOutcome>;
    readonly gaveWayTo: ReadonlyMap<string, ArtifactConsistencyPlaceName>;
    readonly keptBecause?: ReadonlyMap<string, ConsistencyKeptReason>;
  },
): readonly ArtifactConsistencyGroup[] {
  return settled.map(function finalGroup(
    group,
    groupPosition,
  ): ArtifactConsistencyGroup {
    return {
      repeat: group.repeat,
      occurrences: group.places
        .map(function finalPlace(
          place,
          position,
        ): ArtifactConsistencyOccurrence {
          /**
           Rendering of this place.
           */
          const rendering = group.renderings[position];
          /**
           Outcome of this place before the replacements met each other and the floors.
           */
          const provisional = group.outcomes[position];
          if ((rendering === undefined) || (provisional === undefined)) {
            throw new ConsistencyInvariantError({ invariant:
              `${consistencyLabel({ position, },)} of group ${String(groupPosition + 1,)} has no rendering `
                + 'or outcome, and both are made for every place', },);
          }
          /**
           The key this place's outcome and the place it gave way to are kept under.
           */
          const key = placeKey({
            group: groupPosition,
            place: position,
          },);
          /**
           What the replacements and the floors made of this place's replacement, if it had one.
           */
          const decided = outcomes.get(key,);
          if ((provisional === 'replaced') && (decided === undefined)) {
            throw new ConsistencyInvariantError({ invariant:
              `${consistencyLabel({ position, },)} of group ${String(groupPosition + 1,)} was to be replaced `
                + 'and no replacement was applied, left out or refused, and every pending replacement is one of the three', },);
          }
          /**
           The outcome the place ends in.
           */
          const outcome = decided ?? provisional;
          /**
           The place this place gave way to, when it gave way to one.
           */
          const gaveWay = gaveWayTo.get(key,);
          if ((outcome === 'overlapped') && (gaveWay === undefined)) {
            throw new ConsistencyInvariantError({ invariant:
              `${consistencyLabel({ position, },)} of group ${String(groupPosition + 1,)} ended overlapped `
                + 'and no place was recorded for it to have given way to, and every overlap is of a replacement or a located place', },);
          }
          /**
           What the decision of the group found at the edge of the span of this place; `unrefused` where it found nothing.
           */
          const refusedAtEdge = group.leftBecause?.[position] ?? 'unrefused';
          /**
           Why the pass left the place as it was: the reason the apply stage gave a gain it left out, else the refusal at the
           edge the decision of the group recorded.
           */
          const kept = keptBecause.get(key,) ?? refusedAtEdge;
          return {
            label: consistencyLabel({ position, },),
            sliceIndex: place.slice
              .sliceIndex,
            startOffset: place.startOffset,
            endOffset: place.endOffset,
            line: place.line,
            ...((place.slotStartOffset === undefined) ? {} : { slotStartOffset: place.slotStartOffset, }),
            ...((place.slotEndOffset === undefined) ? {} : { slotEndOffset: place.slotEndOffset, }),
            rendering: recordedRendering({ rendering, },),
            outcome,
            ...(((outcome === 'overlapped') && (gaveWay !== undefined)) ? { gaveWayTo: gaveWay, } : {}),
            ...(((outcome === 'kept-current') && (kept !== 'unrefused')) ? { keptBecause: kept, } : {}),
          };
        },),
      unplaced: group.unplaced,
      verdict: group.verdict,
      carries: group.carries,
    };
  },);
}

//endregion Pass consistency apply record
