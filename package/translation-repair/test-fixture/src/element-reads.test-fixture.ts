/**
 A list that counts how often its elements are read, so a reading's work on a list is measured without a clock: a reading
 whose work grows with the square of the list reads far more elements than a few for each element.

 @module
 */

/**
 The reads of one or more lists, counted together.
 */
export type ElementReads = {
  /**
   How many element reads the lists have answered so far, mutated only by the lists' own reads.
   */
  count: number;
};

/**
 A list whose element reads are counted: indexing, iteration and every array method read elements one by one, and each read
 adds one to the count. The list is the one given in every other respect.

 @param items - the elements the list holds

 @param reads - the count this list adds its reads to, shared with other lists to count them together

 @returns A list with the same elements that counts its reads

 @example
 ```ts
 const reads = { count: 0, };
 const list = countedList({ items: ['a', 'b',], reads, },);
 list.includes('b',); // reads.count is now 2
 ```
 */
export function countedList<ItemT,>(
  {
    items,
    reads,
  }: {
    readonly items: readonly ItemT[];
    readonly reads: ElementReads;
  },
): ItemT[] {
  return new Proxy(
    [...items,],
    {
      get(
        target,
        key,
        receiver,
      ): unknown {
        if (((typeof key) === 'string') && Number.isInteger(Number(key,),))
          reads.count += 1;
        return Reflect.get(
          target,
          key,
          receiver,
        );
      },
    },
  );
}
