import { ArtifactParseError, } from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import {
  artifactGenerationNamesOverlap,
  type TwoLaneArtifactGeneration,
} from './artifact-two-lane-contract.ts';
import type {
  ArtifactConsistencyGroup,
  ArtifactConsistencyOccurrence,
  ArtifactConsistencySlice,
} from './artifact-two-lane-consistency.ts';

//region Artifact consistency slices
// WHAT THE CHANGED SLICES SAY BESIDE THE PLACES THAT WERE REPLACED: one row per
// slice, a row for every slice that holds a replaced place and for no other, and
// one fate for the places of a slice, since the floors answer for a slice once.
// What each group says of itself is the coherence check's question
// (`artifact-two-lane-read-consistency-coherence.ts`).
//
// AN OVERLAPPED PLACE MUST NAME WHAT IT OVERLAPPED, FROM GENERATION SEVENTEEN.
// The record gives the group and the label of the place it gave way to, and the
// reader resolves it: the group is another group of the section, the place is
// one of that group's, in the overlapped place's slice, and it either was
// replaced or refused by the floors or has a located rendering. What no record
// holds is the span of the replacement, so the reader cannot confirm that the
// replacement truly overlapped the place it names; it confirms that the name
// points at a place that could have been overlapped. Two overlapped places may
// name each other, since two replacements that each change the words the other
// group located are both left out and each names the other's located place. An
// older generation never named it, and is read under the check it was written
// under: any replacement, refused replacement or located place of another group
// in the slice.

/**
 Whether a place was replaced or refused by the floors, or has a located
 rendering: the three things a place can be that another place gave way to.

 @param place - the place an overlapped place names

 @returns Whether an overlapped place can have given way to it

 @example
 ```ts
 const can = canBeGivenWayTo({ place, },);
 ```
 */
function canBeGivenWayTo({ place, }: { readonly place: ArtifactConsistencyOccurrence; },): boolean {
  /**
   What became of the place, and how its rendering stands.
   */
  const {
    outcome,
    rendering,
  } = place;
  if (outcome === 'replaced')
    return true;
  if (outcome === 'kept-floor')
    return true;
  return rendering.kind === 'located';
}

/**
 A place with the position of its group.

 @example
 ```ts
 const entry: PlaceEntry = { groupPosition: 0, place, };
 ```
 */
type PlaceEntry = {
  /**
   Position of the place's group among the section's groups, from zero.
   */
  readonly groupPosition: number;

  /**
   The place.
   */
  readonly place: ArtifactConsistencyOccurrence;
};

/**
 Checks that an overlapped place names a place it could have given way to.

 The reader checks that the name is present, that it names another group of the
 section and a label that group holds, that the named place stands in the
 overlapped place's slice, and that the named place was replaced, was refused by
 the floors or has a located rendering. It cannot check that the named place is
 the one the replacement truly met, because the record holds no span for a
 replacement to check a name against. So it accepts any such place of another
 group in the slice, including two left-out places that name each other, since
 the writer can produce that pair when two block replacements each change the
 words the other group located.

 @param groups - every recorded group

 @param groupPosition - position of the overlapped place's group, from zero

 @param place - the place that ended overlapped

 @param path - where the slices sit, for the parse error

 @throws {@link ArtifactParseError} when the place names none, names a group the
 section does not hold or the place's own group, names a label the group does not
 hold, names a place in another slice, or names a place that was neither replaced,
 refused nor located

 @example
 ```ts
 assertGaveWayResolves({ groups, groupPosition: 0, place, path, },);
 ```
 */
function assertGaveWayResolves(
  {
    groups,
    groupPosition,
    place,
    path,
  }: {
    readonly groups: readonly ArtifactConsistencyGroup[];
    readonly groupPosition: number;
    readonly place: ArtifactConsistencyOccurrence;
    readonly path: string;
  },
): void {
  /**
   The overlapped place, as the refusal names it.
   */
  const subject = `group ${String(groupPosition + 1,)} ${place.label} (overlapped, slice ${String(place.sliceIndex,)})`;
  /**
   The place it names.
   */
  const { gaveWayTo, } = place;
  if (gaveWayTo === undefined) {
    throw new ArtifactParseError({
      path,
      reason: `${subject} to name the place it gave way to`,
    },);
  }
  /**
   The named group, counted from one as the refusals count.
   */
  const named = String(gaveWayTo.group + 1,);
  /**
   The group the place names.
   */
  const other = groups[gaveWayTo.group];
  if (other === undefined) {
    throw new ArtifactParseError({
      path,
      reason: `${subject} to name a group the section holds, and group ${named} is not one`,
    },);
  }
  if (gaveWayTo.group === groupPosition) {
    throw new ArtifactParseError({
      path,
      reason: `${subject} to name a place of another group, and group ${named} is its own`,
    },);
  }
  /**
   The place it names, in that group.
   */
  const target = other.occurrences
    .find(function hasLabel({ label, },): boolean {
      return label === gaveWayTo.label;
    },);
  if (target === undefined) {
    throw new ArtifactParseError({
      path,
      reason: `${subject} to name a place group ${named} holds, and ${gaveWayTo.label} is not one`,
    },);
  }
  if (target.sliceIndex !== place.sliceIndex) {
    throw new ArtifactParseError({
      path,
      reason: `${subject} to name a place in its own slice, and group ${named} ${target.label} stands in slice ${
        String(target.sliceIndex,)
      }`,
    },);
  }
  if (!canBeGivenWayTo({ place: target, },)) {
    throw new ArtifactParseError({
      path,
      reason: `${subject} to name a replaced or refused place, or a place with a located rendering, and group ${named} ${target.label} is neither`,
    },);
  }
}

/**
 Checks that the places of each slice share one fate, since the floors answer
 for a slice once: no slice holds both a replaced place and a place whose
 replacement the floors refused, and a place left out for overlapping another
 replacement has something it gave way to: the place it names from generation
 seventeen, and in an older generation a replacement or a located place of
 another group in its slice.

 @param groups - every recorded group

 @param path - where the slices sit, for the parse error

 @param generation - generation the section was written under, which says whether
 an overlapped place names the place it gave way to

 @throws {@link ArtifactParseError} when a slice holds both fates, or an overlap
 has nothing it gave way to

 @example
 ```ts
 assertSliceFates({ groups, path, generation: 17, },);
 ```
 */
function assertSliceFates(
  {
    groups,
    path,
    generation,
  }: {
    readonly groups: readonly ArtifactConsistencyGroup[];
    readonly path: string;
    readonly generation: TwoLaneArtifactGeneration;
  },
): void {
  /**
   Every place of every group, with the position of its group.
   */
  const entries = groups.flatMap(function placesOf(
    group,
    groupPosition,
  ): readonly PlaceEntry[] {
    return group.occurrences
      .map(function entryOf(place,): PlaceEntry {
        return {
          groupPosition,
          place,
        };
      },);
  },);
  /**
   Whether an overlapped place names the place it gave way to in this generation.
   */
  const named = artifactGenerationNamesOverlap({ generation, },);
  // THE CHECK AN OLDER GENERATION WAS WRITTEN UNDER: an overlap needs something in its slice to overlap, a replacement
  // or a refused one (an earlier group's replacement won the words), or a located place of another group whose words the
  // replacement would have changed. Nothing in the record says which, so any located place of another group will do.
  /**
   Whether a slice holds a located place of a group other than the one a place belongs to.

   @param sliceIndex - slice asked about

   @param place - the place that ended overlapped

   @returns Whether another group located words in the slice
   */
  function holdsOthersLocated(
    {
      sliceIndex,
      place,
    }: {
      readonly sliceIndex: number;
      readonly place: ArtifactConsistencyOccurrence;
    },
  ): boolean {
    return groups.some(function locatesIn({ occurrences, },): boolean {
      if (occurrences.includes(place,))
        return false;
      return occurrences.some(function isLocatedHere({
        sliceIndex: otherSlice,
        rendering,
      },): boolean {
        return (otherSlice === sliceIndex) && (rendering.kind === 'located');
      },);
    },);
  }
  /**
   Whether a slice holds a place that ended in an outcome.

   @param sliceIndex - slice asked about

   @param outcome - outcome looked for

   @returns Whether some place of the slice ended in it
   */
  function holds({
    sliceIndex,
    outcome,
  }: {
    readonly sliceIndex: number;
    readonly outcome: string
  },): boolean {
    return entries.some(function endedIn({ place, },): boolean {
      return (place.sliceIndex === sliceIndex) && (place.outcome === outcome);
    },);
  }
  for (const {
    groupPosition,
    place,
  } of entries) {
    if ((place.outcome === 'replaced') && holds({
      sliceIndex: place.sliceIndex,
      outcome: 'kept-floor',
    },)) {
      throw new ArtifactParseError({
        path,
        reason: `one fate for slice ${String(place.sliceIndex,)}, which holds a replaced place and a place the floors refused`,
      },);
    }
    if (place.outcome !== 'overlapped')
      continue;
    if (named) {
      assertGaveWayResolves({
        groups,
        groupPosition,
        place,
        path,
      },);
      continue;
    }
    if (holds({
      sliceIndex: place.sliceIndex,
      outcome: 'replaced',
    },))
      continue;
    if (holds({
      sliceIndex: place.sliceIndex,
      outcome: 'kept-floor',
    },))
      continue;
    if (holdsOthersLocated({
      sliceIndex: place.sliceIndex,
      place,
    },))
      continue;
    throw new ArtifactParseError({
      path,
      reason: `a replacement, or a located place of another group, in slice ${String(place.sliceIndex,)} for the overlapped place to overlap`,
    },);
  }
}

/**
 Checks the changed slices against the places that were replaced.

 @param groups - every recorded group

 @param slices - the recorded changed slices

 @param path - where the slices sit, for the parse error

 @param generation - generation the section was written under, which says whether
 an overlapped place names the place it gave way to

 @throws {@link ArtifactParseError} when a slice is listed twice, a slice has no
 replaced place, a replaced place has no slice, or an overlapped place has
 nothing it gave way to

 @example
 ```ts
 assertSlicesCoherent({ groups, slices, path, generation: 17, },);
 ```
 */
export function assertSlicesCoherent(
  {
    groups,
    slices,
    path,
    generation,
  }: {
    readonly groups: readonly ArtifactConsistencyGroup[];
    readonly slices: readonly ArtifactConsistencySlice[];
    readonly path: string;
    readonly generation: TwoLaneArtifactGeneration;
  },
): void {
  /**
   Slices a replaced place sits in.
   */
  const replaced = new Set(groups.flatMap(function replacedIn(group,): readonly number[] {
    return group.occurrences
      .filter(function wasReplaced(occurrence,): boolean {
        return occurrence.outcome === 'replaced';
      },)
      .map(function sliceOf(occurrence,): number {
        return occurrence.sliceIndex;
      },);
  },),);
  /**
   Slices already named.
   */
  const named = new Set<number>();
  for (const row of slices) {
    if (named.has(row.sliceIndex,)) {
      throw new ArtifactParseError({
        path,
        reason: `one row per slice; slice ${String(row.sliceIndex,)} appears more than once`,
      },);
    }
    named.add(row.sliceIndex,);
    if (!replaced.has(row.sliceIndex,)) {
      throw new ArtifactParseError({
        path,
        reason: `a replaced place in slice ${String(row.sliceIndex,)}, which a changed slice needs`,
      },);
    }
  }
  for (const sliceIndex of replaced) {
    if (!named.has(sliceIndex,)) {
      throw new ArtifactParseError({
        path,
        reason: `a changed slice row for slice ${String(sliceIndex,)}, which holds a replaced place`,
      },);
    }
  }
  assertSliceFates({
    groups,
    path,
    generation,
  },);
}

//endregion Artifact consistency slices
