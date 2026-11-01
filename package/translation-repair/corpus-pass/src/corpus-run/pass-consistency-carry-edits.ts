import {
  type CarryOffer,
  ConsistencyInvariantError,
} from '@monochromatic-dev/translation-repair-consistency/ts';
import {
  sliceOfUnit,
  unitLabelsOf,
} from './pass-consistency-carry-units.ts';
import type {
  CarryUnit,
  PendingEdit,
} from './pass-consistency-types.ts';

//region Pass consistency carry edits
// WHAT A WINNING CANDIDATE DOES TO THE PAGE: the replacements it makes and, for a splice that carried a block in part, the
// finding that names the places carried and the places that kept their words. Split out of the bench's choice
// (`pass-consistency-carry-choose.ts`) so each reads on its own; neither asks a model.

/**
 The replacements a winning candidate makes.

 @param unit - the unit

 @param winner - the candidate the choices named, as a list of zero or one

 @returns A replacement of the words of each receiving place that has one for the splice, a replacement of the block for a writer's block carrying every receiving place, and none when the current block or nobody won

 @throws {@link ConsistencyInvariantError} when the splice won in a unit none of whose receiving places has words that code can move, since no splice is offered then

 @example
 ```ts
 const edits = editsOf({ unit, winner: [], },);
 ```
 */
export function editsOf(
  {
    unit,
    winner,
  }: {
    readonly unit: CarryUnit;
    readonly winner: readonly CarryOffer[];
  },
): readonly PendingEdit[] {
  /**
   The candidate that won, if any.
   */
  const [named,] = winner;
  if ((named === undefined) || (named.kind === 'current'))
    return [];
  /**
   The slice the block stands in.
   */
  const { sliceIndex, } = sliceOfUnit({ unit, },);
  if (named.kind === 'splice') {
    /**
     The replacement of each receiving place that has one; a place without one keeps its words.
     */
    const edits = unit.receivers
      .flatMap(function editOf(receiver,): readonly PendingEdit[] {
        /**
         How the chosen words are moved into this place.
         */
        const { plan, } = receiver;
        return (plan.kind === 'replace')
          ? [
            {
              kind: 'splice',
              placePositions: [receiver.position,],
              sliceIndex,
              start: plan.start,
              end: plan.end,
              text: plan.text,
            },
          ]
          : [];
      },);
    if (edits.length === 0) {
      throw new ConsistencyInvariantError({ invariant:
        `the splice won for ${unitLabelsOf({ unit, },)}, none of whose receiving places has words that code can move, and a splice is offered only when some receiving place of the block can be spliced`, },);
    }
    return edits;
  }
  return [
    {
      kind: 'block',
      placePositions: unit.receivers
        .map(function positionOf({ position, },): number {
          return position;
        },),
      sliceIndex,
      start: unit.block
        .start,
      end: unit.block
        .end,
      text: named.text,
    },
  ];
}

/**
 The finding for a block the splice carried in part, in the words of the other findings of the carry.

 @param unit - the unit

 @param winner - the candidate the choices named, as a list of zero or one

 @returns A finding, as a list of zero or one, naming the places the chosen words were moved into and the places that kept their words because code cannot move the chosen words into them; none unless the splice won and some receiving place of the block has no replacement

 @example
 ```ts
 const findings = partlyCarriedFindingOf({ unit, winner: [], },);
 ```
 */
export function partlyCarriedFindingOf(
  {
    unit,
    winner,
  }: {
    readonly unit: CarryUnit;
    readonly winner: readonly CarryOffer[];
  },
): readonly string[] {
  /**
   The candidate that won, if any.
   */
  const [named,] = winner;
  if (named?.kind !== 'splice')
    return [];
  /**
   Labels of the receiving places the splice moved the chosen words into.
   */
  const carried = unit.receivers
    .filter(function isCarried({ plan, },): boolean {
      return plan.kind === 'replace';
    },)
    .map(function labelOf({ label, },): string {
      return label;
    },);
  /**
   Labels of the receiving places that kept their words.
   */
  const kept = unit.receivers
    .filter(function isKept({ plan, },): boolean {
      return plan.kind !== 'replace';
    },)
    .map(function labelOf({ label, },): string {
      return label;
    },);
  if (kept.length === 0)
    return [];
  return [
    `consistency-partly-carried (${unitLabelsOf({ unit, },)}): the splice won, so code moved the chosen words into `
      + `${carried.join(' and ',)} and left ${kept.join(' and ',)} as it was, since code cannot move them there`,
  ];
}

//endregion Pass consistency carry edits
