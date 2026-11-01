/**
 Recorded consistency groups over invented cat wording, valid in every
 relation the artifact reader recounts, so a case breaks one thing and a
 refusal can only come from it: a repeated sentence in three places that three
 judges worded alike, and a refrain in two places with a varying slot.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import type {
  ArtifactConsistencyBallot,
  ArtifactConsistencyGroup,
  ArtifactConsistencyOccurrence,
  ArtifactConsistencyVerdict,
  ConsistencyOccurrenceOutcome,
} from '../../dist/final/node/index.mjs';

//region Consistency group fixture

/**
 Characters of the repeated sentence's original wording.
 */
const WORDING_LENGTH = 4;

/**
 Quotes each of the three judges gives for the repeated sentence: the first
 place with its closing full stop, the second in other words, the third in the
 first place's words without the full stop.
 */
const NAPS_QUOTES: ArtifactConsistencyBallot['quotes'] = [
  {
    label: 'OCCURRENCE 1',
    quote: 'The cat naps.',
  },
  {
    label: 'OCCURRENCE 2',
    quote: 'The kitten naps',
  },
  {
    label: 'OCCURRENCE 3',
    quote: 'The cat naps',
  },
];

/**
 Quotes each of the three judges gives for the refrain, a slot in each.
 */
const FRAME_QUOTES: ArtifactConsistencyBallot['quotes'] = [
  {
    label: 'OCCURRENCE 1',
    quote: 'So this cat is a lazy little treasure.',
    slot: 'lazy',
  },
  {
    label: 'OCCURRENCE 2',
    quote: 'So this cat must be a chubby little treasure.',
    slot: 'chubby',
  },
];

/**
 One ballot choosing the first place and keeping none.

 @param seat - seat that answered

 @param quotes - the quotes the judge gave

 @returns The recorded ballot

 @example
 ```ts
 const ballot = firstPlaceBallot({ seat: 'glm-5.3', quotes: [], },);
 ```
 */
export function firstPlaceBallot(
  {
    seat,
    quotes,
  }: {
    readonly seat: string;
    readonly quotes: ArtifactConsistencyBallot['quotes'];
  },
): ArtifactConsistencyBallot {
  return {
    seat,
    reason: 'The first place words it best.',
    quotes,
    choice: 'OCCURRENCE 1',
    keep: [],
  };
}

/**
 The verdict of a group three judges answered alike, the first place winning.

 @param quotes - the quotes every judge gave

 @returns Verdict with three usable ballots against a quorum of two

 @example
 ```ts
 const verdict = threeAlikeVerdict({ quotes: FRAME_QUOTES, },);
 ```
 */
function threeAlikeVerdict({ quotes, }: { readonly quotes: ArtifactConsistencyBallot['quotes']; },): ArtifactConsistencyVerdict {
  return {
    kind: 'asked',
    ballots: [
      'hyper-cat',
      'bedrock-cat',
      'synthetic-cat',
    ]
      .map(function ballotOf(seat,): ArtifactConsistencyBallot {
        return firstPlaceBallot({
          seat,
          quotes,
        },);
      },),
    usable: 3,
    quorum: 2,
    winner: {
      kind: 'chosen',
      label: 'OCCURRENCE 1',
    },
  };
}

/**
 One place of the repeated sentence, in the slice of its position.

 @param position - zero-based position of the place in the group

 @param outcome - what became of the place

 @returns The recorded place, its rendering the quote the judges gave for it

 @example
 ```ts
 const place = napsPlace({ position: 1, outcome: 'replaced', },);
 ```
 */
function napsPlace(
  {
    position,
    outcome,
  }: {
    readonly position: number;
    readonly outcome: ConsistencyOccurrenceOutcome;
  },
): ArtifactConsistencyOccurrence {
  /**
   The quote the judges gave for this place.
   */
  const quoted = NAPS_QUOTES[position];
  return {
    label: `OCCURRENCE ${String(position + 1,)}`,
    sliceIndex: position,
    startOffset: position * 10,
    endOffset: (position * 10) + WORDING_LENGTH,
    line: (position * 2) + 1,
    rendering: {
      kind: 'located',
      quote: (quoted === undefined) ? '' : quoted.quote,
    },
    outcome,
  };
}

/**
 The repeated sentence in three places: the first chosen, the second replaced
 by its words, the third already carrying them.

 @returns The recorded group

 @example
 ```ts
 const group = napsGroup();
 ```
 */
export function napsGroup(): ArtifactConsistencyGroup {
  return {
    repeat: {
      kind: 'maximal',
      wording: '小猫打盹',
    },
    occurrences: [
      napsPlace({
        position: 0,
        outcome: 'chosen',
      },),
      napsPlace({
        position: 1,
        outcome: 'replaced',
      },),
      napsPlace({
        position: 2,
        outcome: 'consistent',
      },),
    ],
    unplaced: [],
    verdict: threeAlikeVerdict({ quotes: NAPS_QUOTES, },),
  };
}

/**
 The refrain in two places with a slot each: the first chosen, the second
 replaced by its pattern.

 @returns The recorded group

 @example
 ```ts
 const group = frameGroup();
 ```
 */
export function frameGroup(): ArtifactConsistencyGroup {
  return {
    repeat: {
      kind: 'refrain',
      prefix: '所以这只猫',
      suffix: '的小宝贝',
    },
    occurrences: [
      {
        label: 'OCCURRENCE 1',
        sliceIndex: 0,
        startOffset: 0,
        endOffset: 14,
        line: 1,
        slotStartOffset: 5,
        slotEndOffset: 7,
        rendering: {
          kind: 'located',
          quote: 'So this cat is a lazy little treasure.',
          slot: 'lazy',
        },
        outcome: 'chosen',
      },
      {
        label: 'OCCURRENCE 2',
        sliceIndex: 1,
        startOffset: 20,
        endOffset: 34,
        line: 3,
        slotStartOffset: 25,
        slotEndOffset: 27,
        rendering: {
          kind: 'located',
          quote: 'So this cat must be a chubby little treasure.',
          slot: 'chubby',
        },
        outcome: 'replaced',
      },
    ],
    unplaced: [],
    verdict: threeAlikeVerdict({ quotes: FRAME_QUOTES, },),
  };
}

/**
 A group with one place changed.

 @param group - the valid group

 @param position - position of the place to change

 @param over - fields of the place that take other values

 @returns The group with that place changed

 @example
 ```ts
 const broken = withPlace({ group: napsGroup(), position: 1, over: { outcome: 'untouched', }, },);
 ```
 */
export function withPlace(
  {
    group,
    position,
    over,
  }: {
    readonly group: ArtifactConsistencyGroup;
    readonly position: number;
    readonly over: {
      readonly label?: string;
      readonly sliceIndex?: number;
      readonly startOffset?: number;
      readonly endOffset?: number;
      readonly line?: number;
      readonly slotStartOffset?: number;
      readonly slotEndOffset?: number;
      readonly rendering?: ArtifactConsistencyOccurrence['rendering'];
      readonly outcome?: ConsistencyOccurrenceOutcome;
      readonly gaveWayTo?: NonNullable<ArtifactConsistencyOccurrence['gaveWayTo']>;
    };
  },
): ArtifactConsistencyGroup {
  return {
    ...group,
    occurrences: group.occurrences
      .map(function patched(
        place,
        at,
      ): ArtifactConsistencyOccurrence {
        return (at === position)
          ? {
            ...place,
            ...over,
          }
          : place;
      },),
  };
}

/**
 A group with one place replaced whole, for a place that must lack a key the
 other places carry.

 @param group - the valid group

 @param position - position of the place to replace

 @param place - the place that stands there

 @returns The group with that place replaced

 @example
 ```ts
 const broken = withPlaceReplaced({ group: frameGroup(), position: 1, place: bare, },);
 ```
 */
export function withPlaceReplaced(
  {
    group,
    position,
    place,
  }: {
    readonly group: ArtifactConsistencyGroup;
    readonly position: number;
    readonly place: ArtifactConsistencyOccurrence;
  },
): ArtifactConsistencyGroup {
  return {
    ...group,
    occurrences: group.occurrences
      .map(function standing(
        held,
        at,
      ): ArtifactConsistencyOccurrence {
        return (at === position) ? place : held;
      },),
  };
}

/**
 A group the bench was asked about, read as the recount reads it.

 @param group - a group whose verdict is the bench's answer

 @returns The group, typed as asked

 @throws {@link Error} when the group's verdict is not the bench's answer

 @example
 ```ts
 const asked = askedOf({ group: napsGroup(), },);
 ```
 */
export function askedOf(
  { group, }: { readonly group: ArtifactConsistencyGroup; },
): ArtifactConsistencyGroup & { readonly verdict: Extract<ArtifactConsistencyVerdict, { readonly kind: 'asked'; }>; } {
  /**
   The verdict the group records.
   */
  const { verdict, } = group;
  if (verdict.kind !== 'asked')
    throw new Error(`Expected a group the bench was asked about, but its verdict is ${verdict.kind}`,);
  return {
    ...group,
    verdict,
  };
}

//endregion Consistency group fixture
