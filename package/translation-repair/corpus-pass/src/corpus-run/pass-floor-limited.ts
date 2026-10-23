import type { FloorLimitedMemory, } from '@monochromatic-dev/translation-repair-translate-lane/ts';

//region Pass memory of floor-limited slices
// ONE MEMORY PER ENTRY, HELD FOR THE WHOLE PASS, so a later attempt of the same entry finds the slices an earlier one
// left unfilled on a select bench below its floor and does not ask them again unless the bench has grown
// (`translate-floor-limited.ts` in the translate lane holds the rule). Another entry gets its own, and a new pass starts
// with none, since the bench may be healthy by then.

/**
 The pass's memories, one per entry.

 @example
 ```ts
 const memories: FloorLimitedByEntry = floorLimitedByEntry();
 ```
 */
export type FloorLimitedByEntry = {
  readonly of: (args: { readonly entryId: string; },) => FloorLimitedMemory;
};

/**
 Opens the memories a pass hands its entries.

 @returns Memories keyed by entry, each created empty on first use

 @example
 ```ts
 const memories = floorLimitedByEntry();
 const memory = memories.of({ entryId: 'tabby', },);
 ```
 */
export function floorLimitedByEntry(): FloorLimitedByEntry {
  /**
   Memory of each entry that asked for one.
   */
  const memories = new Map<string, FloorLimitedMemory>();
  return {
    of: function ofEntry({ entryId, },): FloorLimitedMemory {
      /**
       This entry's memory, if an earlier attempt created it.
       */
      const existing = memories.get(entryId,);
      if (existing !== undefined)
        return existing;
      /**
       The entry's first memory.
       */
      const created: FloorLimitedMemory = new Map();
      memories.set(
        entryId,
        created,
      );
      return created;
    },
  };
}

//endregion Pass memory of floor-limited slices
