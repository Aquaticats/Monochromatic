import {
  requireExactKeys,
  requireKeyOf,
} from '../artifact-exact-guard.ts';
import {
  ArtifactParseError,
  requireCount,
  requireRecord,
  requireString,
} from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import {
  artifactGenerationCountsBySpan,
  artifactGenerationNamesOverlap,
  type TwoLaneArtifactGeneration,
} from './artifact-two-lane-contract.ts';
import {
  type ArtifactConsistencyOccurrence,
  type ArtifactConsistencyPlaceName,
  CONSISTENCY_KEPT_REASONS,
  CONSISTENCY_OCCURRENCE_OUTCOMES,
  type ConsistencyKeptReason,
  type ConsistencyOccurrenceOutcome,
} from './artifact-two-lane-consistency.ts';
import { requirePlaceBounds, } from './artifact-two-lane-read-consistency-parts.ts';
import { parseConsistencyRendering, } from './artifact-two-lane-read-consistency-rendering.ts';

//region Artifact consistency occurrence reading
// Reading one place of a recorded group, key by key. The place's rendering is
// read in `artifact-two-lane-read-consistency-rendering.ts`.

/**
 Reads what became of a place.

 @param value - recorded outcome

 @param path - where it sits, for the parse error

 @param generation - generation the section was written under

 @returns The outcome

 @throws {@link ArtifactParseError} when it is not an outcome, or is `kept-current`
 in a generation that gives no such outcome

 @example
 ```ts
 const outcome = requireOutcome({ value: record.outcome, path: `${path}.outcome`, generation: 16, },);
 ```
 */
function requireOutcome(
  {
    value,
    path,
    generation,
  }: {
    readonly value: unknown;
    readonly path: string;
    readonly generation: TwoLaneArtifactGeneration;
  },
): ConsistencyOccurrenceOutcome {
  /**
   The outcome the artifact names.
   */
  const outcome = requireKeyOf({
    value,
    record: CONSISTENCY_OCCURRENCE_OUTCOMES,
    path,
  },);
  if ((outcome === 'kept-current') && (!artifactGenerationCountsBySpan({ generation, },))) {
    throw new ArtifactParseError({
      path,
      reason: 'an outcome generation fifteen gives, which has no carry to end in kept-current',
    },);
  }
  return outcome;
}

/**
 Reads the place an overlapped place gave way to.

 @param value - recorded name, absent from a place that gave way to none

 @param outcome - what became of the place that records it

 @param path - where the name sits, for the parse error

 @param generation - generation the section was written under, which says whether
 an overlapped place names the place it gave way to

 @returns The record's `gaveWayTo` entry for the place, empty when the place gave way to none

 @throws {@link ArtifactParseError} when an overlapped place of a generation that
 records the name has none, a place that is not overlapped has one, or the name is
 not a group position and a label

 @example
 ```ts
 const named = parseGaveWayTo({ value: record.gaveWayTo, outcome, path: `${path}.gaveWayTo`, generation: 17, },);
 ```
 */
function parseGaveWayTo(
  {
    value,
    outcome,
    path,
    generation,
  }: {
    readonly value: unknown;
    readonly outcome: ConsistencyOccurrenceOutcome;
    readonly path: string;
    readonly generation: TwoLaneArtifactGeneration;
  },
): { readonly gaveWayTo?: ArtifactConsistencyPlaceName; } {
  if (!artifactGenerationNamesOverlap({ generation, },))
    return {};
  if (outcome !== 'overlapped') {
    if (value !== undefined) {
      throw new ArtifactParseError({
        path,
        reason: 'an overlapped place, the only kind that gave way to another place',
      },);
    }
    return {};
  }
  if (value === undefined) {
    throw new ArtifactParseError({
      path,
      reason: 'the place an overlapped place gave way to, which generation seventeen records',
    },);
  }
  /**
   The name as a record.
   */
  const record = requireRecord({
    value,
    path,
  },);
  requireExactKeys({
    record,
    allowed: [
      'group',
      'label',
    ],
    path,
  },);
  return {
    gaveWayTo: {
      group: requireCount({
        value: record.group,
        path: `${path}.group`,
      },),
      label: requireString({
        value: record.label,
        path: `${path}.label`,
      },),
    },
  };
}

/**
 Whether a text is one of the closed set of reasons a place is left as it was for.

 @param candidate - the text a record holds

 @returns Whether it is a reason, by own key so an inherited property never answers for a name the record carries

 @example
 ```ts
 const known = isKeptReason('gain-pairs',); // true
 ```
 */
function isKeptReason(candidate: string,): candidate is ConsistencyKeptReason {
  return Object.hasOwn(
    CONSISTENCY_KEPT_REASONS,
    candidate,
  );
}

/**
 What a reader expects of the reason a place was left as it was for.

 @param label - the place's label

 @param read - what the record holds there, text for a reason outside the set; absent where nothing is recorded

 @returns The expectation, which names the place, the closed set of reasons and what the record holds

 @example
 ```ts
 const reason = keptReasonExpected({ label: 'OCCURRENCE 2', },);
 ```
 */
export function keptReasonExpected(
  {
    label,
    read,
  }: {
    readonly label: string;
    readonly read?: string;
  },
): string {
  return `a reason ${label} was left as it was for, one of ${
    Object.keys(CONSISTENCY_KEPT_REASONS,)
      .join(', ',)
  }; the record has ${(read === undefined) ? 'none' : JSON.stringify(read,)}`;
}

/**
 Reads why a place was left as it was.

 @param value - recorded reason, absent from a place the pass did not leave as it was for a reason

 @param outcome - what became of the place that records it

 @param label - the place's label, which the refusal names

 @param path - where the reason sits, for the parse error

 @param generation - generation the section was written under, which says whether a place records a reason

 @returns The record's `keptBecause` entry for the place, empty where the place records none

 @throws {@link ArtifactParseError} when a place that is not kept-current records one, or the reason is not text or is not
 one of the closed set

 @example
 ```ts
 const kept = parseKeptBecause({ value: record.keptBecause, outcome, label: 'OCCURRENCE 2', path: `${path}.keptBecause`, generation: 17, },);
 ```
 */
function parseKeptBecause(
  {
    value,
    outcome,
    label,
    path,
    generation,
  }: {
    readonly value: unknown;
    readonly outcome: ConsistencyOccurrenceOutcome;
    readonly label: string;
    readonly path: string;
    readonly generation: TwoLaneArtifactGeneration;
  },
): { readonly keptBecause?: ConsistencyKeptReason; } {
  if ((!artifactGenerationNamesOverlap({ generation, },)) || (value === undefined))
    return {};
  if (outcome !== 'kept-current') {
    throw new ArtifactParseError({
      path,
      reason: 'a kept-current place, the only kind the pass leaves as it was for a reason',
    },);
  }
  /**
   The reason as text.
   */
  const held = requireString({
    value,
    path,
  },);
  if (!isKeptReason(held,)) {
    throw new ArtifactParseError({
      path,
      reason: keptReasonExpected({
        label,
        read: held,
      },),
    },);
  }
  return { keptBecause: held, };
}

/**
 Reads one place of a group.

 @param value - recorded place

 @param path - where it sits, for the parse error

 @param generation - generation the section was written under

 @returns The place

 @throws {@link ArtifactParseError} when it is the wrong shape, its offsets run
 backward, its line is zero, its rendering or outcome belongs to the other
 generation, or the place it gave way to is absent from an overlapped place of a
 generation that records one, present on any other place, or malformed

 @example
 ```ts
 const occurrence = parseConsistencyOccurrence({ value, path: `${path}.occurrences[0]`, generation: 16, },);
 ```
 */
export function parseConsistencyOccurrence(
  {
    value,
    path,
    generation,
  }: {
    readonly value: unknown;
    readonly path: string;
    readonly generation: TwoLaneArtifactGeneration;
  },
): ArtifactConsistencyOccurrence {
  /**
   The place as a record.
   */
  const record = requireRecord({
    value,
    path,
  },);
  requireExactKeys({
    record,
    allowed: [
      'label',
      'sliceIndex',
      'startOffset',
      'endOffset',
      'line',
      'slotStartOffset',
      'slotEndOffset',
      'rendering',
      'outcome',
      ...(artifactGenerationNamesOverlap({ generation, },)
        ? [
          'gaveWayTo',
          'keptBecause',
        ]
        : []),
    ],
    path,
  },);
  /**
   What became of the place.
   */
  const outcome = requireOutcome({
    value: record.outcome,
    path: `${path}.outcome`,
    generation,
  },);
  /**
   Where the place starts and ends in the original.
   */
  const startOffset = requireCount({
    value: record.startOffset,
    path: `${path}.startOffset`,
  },);
  /**
   Exclusive end of the place in the original.
   */
  const endOffset = requireCount({
    value: record.endOffset,
    path: `${path}.endOffset`,
  },);
  /**
   One-based line.
   */
  const line = requireCount({
    value: record.line,
    path: `${path}.line`,
  },);
  requirePlaceBounds({
    startOffset,
    endOffset,
    line,
    path,
  },);
  /**
   The label the sheet gave the place.
   */
  const label = requireString({
    value: record.label,
    path: `${path}.label`,
  },);
  return {
    label,
    sliceIndex: requireCount({
      value: record.sliceIndex,
      path: `${path}.sliceIndex`,
    },),
    startOffset,
    endOffset,
    line,
    ...((record.slotStartOffset === undefined)
      ? {}
      : {
        slotStartOffset: requireCount({
          value: record.slotStartOffset,
          path: `${path}.slotStartOffset`,
        },),
      }),
    ...((record.slotEndOffset === undefined)
      ? {}
      : {
        slotEndOffset: requireCount({
          value: record.slotEndOffset,
          path: `${path}.slotEndOffset`,
        },),
      }),
    rendering: parseConsistencyRendering({
      value: record.rendering,
      path: `${path}.rendering`,
      generation,
    },),
    outcome,
    ...parseGaveWayTo({
      value: record.gaveWayTo,
      outcome,
      path: `${path}.gaveWayTo`,
      generation,
    },),
    ...parseKeptBecause({
      value: record.keptBecause,
      outcome,
      label,
      path: `${path}.keptBecause`,
      generation,
    },),
  };
}

//endregion Artifact consistency occurrence reading
