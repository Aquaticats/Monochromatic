import type { ConsistencyRendering, } from '@monochromatic-dev/translation-repair-consistency/ts';
import type { RepeatGroup, } from '@monochromatic-dev/translation-repair-corpus-source/ts';
import type {
  ArtifactConsistencyRepeat,
  ArtifactConsistencyUnplaced,
  ConsistencyNotAskedReason,
  ConsistencyOccurrenceOutcome,
} from '@monochromatic-dev/translation-repair-artifact/ts';
import type { ConsistencyPlace, } from './pass-consistency-group.ts';
import type { SettledGroup, } from './pass-consistency-types.ts';

//region Pass consistency unasked
// A GROUP THE BENCH IS NOT ASKED ABOUT, OR WHOSE HANDLING FAILED: every place
// stays as it stands, and the group says why.

/**
 The places no slice holds whole, as the artifact records them.

 @param group - repeat group of the original

 @returns One entry per place the detector left unplaced

 @example
 ```ts
 const unplaced = unplacedOf({ group, },);
 ```
 */
export function unplacedOf({ group, }: { readonly group: RepeatGroup; },): readonly ArtifactConsistencyUnplaced[] {
  return group.unplaced
    .map(function toUnplaced(place,): ArtifactConsistencyUnplaced {
      return {
        startOffset: place.startOffset,
        endOffset: place.endOffset,
        line: place.line,
        reason: place.reason,
      };
    },);
}

/**
 Settles a group the bench is not asked about.

 @param repeat - the repeated wording or frame

 @param places - the places a slice holds whole

 @param unplaced - the places no slice holds whole

 @param reason - why the bench is not asked

 @returns The group with every place left as it stands, or all consistent

 @example
 ```ts
 const settled = settleUnasked({ repeat, places, unplaced, reason: 'consistent', },);
 ```
 */
export function settleUnasked(
  {
    repeat,
    places,
    unplaced,
    reason,
  }: {
    readonly repeat: ArtifactConsistencyRepeat;
    readonly places: readonly ConsistencyPlace[];
    readonly unplaced: readonly ArtifactConsistencyUnplaced[];
    readonly reason: ConsistencyNotAskedReason;
  },
): SettledGroup {
  return {
    repeat,
    places,
    unplaced,
    verdict: {
      kind: 'not-asked',
      reason,
    },
    renderings: places.map(function notLooked(): ConsistencyRendering {
      return {
        kind: 'unlocated',
        reason: 'not-asked',
      };
    },),
    outcomes: places.map(function outcomeOf(): ConsistencyOccurrenceOutcome {
      return (reason === 'consistent') ? 'consistent' : 'untouched';
    },),
    edits: [],
    carries: [],
    findings: [],
  };
}

/**
 Settles a group whose handling failed: every place stays as it stands.

 @param repeat - the repeated wording or frame

 @param places - the places a slice holds whole

 @param unplaced - the places no slice holds whole

 @param detail - what went wrong, naming no corpus text

 @returns The group with every place left as it stands

 @example
 ```ts
 const settled = settleFailed({ repeat, places, unplaced, detail: 'Error', },);
 ```
 */
export function settleFailed(
  {
    repeat,
    places,
    unplaced,
    detail,
  }: {
    readonly repeat: ArtifactConsistencyRepeat;
    readonly places: readonly ConsistencyPlace[];
    readonly unplaced: readonly ArtifactConsistencyUnplaced[];
    readonly detail: string;
  },
): SettledGroup {
  return {
    ...settleUnasked({
      repeat,
      places,
      unplaced,
      reason: 'too-few-placed',
    },),
    verdict: {
      kind: 'failed',
      detail,
    },
  };
}

//endregion Pass consistency unasked
