/**
 Apply an async function to each item one after another, in order, and collect the results.

 The calls are chained through a reduction instead of a loop or `Promise.all`, so the next call starts only after the
 previous one finished; use it where running them at once would load the host more than the work is worth.

 @param items - values to process, in order
 
 @param run - starts the work for one item
 
 @returns the results, in the order of the items
 
 @example
 ```ts
 await mapInSequence({ items: [1, 2], run: async (item) => item * 2 });
 ```
 */
export function mapInSequence<Item, Result>({
  items,
  run,
}: {
  readonly items: readonly Item[];
  readonly run: (item: Item) => Promise<Result>;
}): Promise<readonly Result[]> {
  return items.reduce(
    async function chain(
      earlier: Promise<readonly Result[]>,
      item: Item
    ): Promise<readonly Result[]> {
      /**
       Results of the items before this one.
       */
      const done = await earlier;
      return [
        ...done,
        await run(item)
      ];
    },
    Promise.resolve<readonly Result[]>([]),
  );
}
