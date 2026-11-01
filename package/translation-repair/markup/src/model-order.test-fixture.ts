import { compareCodePoints, } from '../dist/final/node/index.mjs';

//region Model order
// THE ORDER A TEST READS MODELS IN WHEN THE BENCH ANSWERS IN A ROTATED ORDER (n161). The roster is asked from a bench the prompt
// rotates (`stage-fanout-window.ts`), so the order in which ballots, claims and renderings come back moves whenever a sheet's
// wording does. A test of what a record holds sorts both the record and its expectation by model, in one order on every machine.

/**
 Puts things that carry a model id in order of the id, in code-point order.

 @param items - the things, left as they are

 @returns A new list of the same things, sorted by model id

 @example
 ```ts
 const ballots = sortedByModelId({ items: [{ modelId: 'b', }, { modelId: 'a', },], }); // a, then b
 ```
 */
export function sortedByModelId<ItemT extends { readonly modelId: string; },>(
  {
    items,
  }: {
    readonly items: readonly ItemT[];
  },
): ItemT[] {
  return items
    .toSorted(function byModelId(
      left,
      right,
    ): number {
      return compareCodePoints({
        left: left.modelId,
        right: right.modelId,
      },);
    },);
}

//endregion Model order
