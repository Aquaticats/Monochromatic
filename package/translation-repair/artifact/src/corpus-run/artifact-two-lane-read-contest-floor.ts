import { ArtifactParseError, } from '@monochromatic-dev/translation-repair-artifact-pool/ts';

//region Contest floor mark read
// The mark a contest slice carries when its round was never opened for want of a bench, in its own file so the
// contest reader stays under its line budget and the rule it states has one place.

/**
 Reads the mark of a contest slice whose round was never opened because the bench read below its floor.

 THE MARK SAYS NO ROSTER WAS ASKED, so a record carrying it holds no ballot: a slice with ballots was asked, and a mark
 on it is two records in one, refused rather than believed.

 @param value - recorded `benchBelowFloor` value, absent when the slice carries none

 @param ballotCount - ballots the record carries

 @param path - artifact path of the value, for refusals

 @returns Mark as a record fragment, empty when the slice carries none

 @throws {@link ArtifactParseError} when the value is anything but true, or the slice carries ballots

 @example
 ```ts
 const mark = parseContestFloorMark({ value: undefined, ballotCount: 0, path: 'x.benchBelowFloor', },);
 ```
 */
export function parseContestFloorMark(
  {
    value,
    ballotCount,
    path,
  }: {
    readonly value: unknown;
    readonly ballotCount: number;
    readonly path: string;
  },
): { readonly benchBelowFloor?: true; } {
  if (value === undefined)
    return {};
  if (value !== true) {
    throw new ArtifactParseError({
      path,
      reason: 'true or absent',
    },);
  }
  if (ballotCount > 0) {
    throw new ArtifactParseError({
      path,
      reason: `no ballots, since a bench read below its floor was not asked, rather than ${String(ballotCount,)}`,
    },);
  }
  return { benchBelowFloor: true, };
}

//endregion Contest floor mark read
