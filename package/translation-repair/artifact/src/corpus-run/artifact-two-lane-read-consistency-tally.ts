import { ArtifactParseError, } from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import type { ArtifactConsistencyGroup, } from './artifact-two-lane-consistency.ts';

//region Artifact consistency tally
// WHAT THE RECOUNT OF A GROUP AND THE RECOUNT OF A CARRY BOTH ASK OF THEIR
// BALLOTS: each seat spoke once, and which place the group's recorded winner
// names. Shared so the first shape's reader, the sixteenth shape's reader and
// the carry's recount read these two questions one way.

/**
 Checks that no seat spoke twice in a gather.

 @param seats - the seat of each ballot, in the order the ballots are recorded

 @param path - where the ballots sit, for the parse error

 @throws {@link ArtifactParseError} when a seat appears at a second position

 @example
 ```ts
 assertSeatsHeardOnce({ seats: ['hyper-cat', 'bedrock-cat',], path: 'cat.verdict.ballots', },);
 ```
 */
export function assertSeatsHeardOnce(
  {
    seats,
    path,
  }: {
    readonly seats: readonly string[];
    readonly path: string;
  },
): void {
  seats.forEach(function heardOnce(
    seat,
    at,
  ): void {
    if (seats.indexOf(seat,) !== at) {
      throw new ArtifactParseError({
        path: `${path}[${String(at,)}].seat`,
        reason: 'a seat heard once, since a gather hears each seat once',
      },);
    }
  },);
}

/**
 The position of the place a group's recorded winner names.

 @param group - the group, asked of the bench

 @returns The position, minus one where no place won or the label names no place

 @example
 ```ts
 const at = winningPositionOf({ group, },); // 0
 ```
 */
export function winningPositionOf(
  { group, }: { readonly group: ArtifactConsistencyGroup & { readonly verdict: { readonly kind: 'asked'; }; }; },
): number {
  /**
   The winner as the bench ended with it.
   */
  const { winner, } = group.verdict;
  if (winner.kind !== 'chosen')
    return (-1);
  return group.occurrences
    .findIndex(function isNamed({ label, },): boolean {
      return label === winner.label;
    },);
}

//endregion Artifact consistency tally
