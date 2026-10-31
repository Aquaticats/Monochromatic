/**
 Runs `task` over every item with at most `size` tasks at once,
 and returns the results in input order.

 Chunks run one after another through recursion over the chunk list,
 whose depth is the item count divided by `size`,
 so no loop awaits and no promise chain is built.

 @param items - Items to process.

 @param size - Most tasks running at once;
 values below one run one at a time.

 @param task - Work for one item.

 @returns Results in input order.

 @example
 ```ts
 await mapInChunks({ items: [1, 2, 3], size: 2, task: async (item) => item * 2 });
 ```
 */
export async function mapInChunks<Item, Result,>(
  {
    items,
    size,
    task,
  }: {
    readonly items: readonly Item[];
    readonly size: number;
    readonly task: (item: Item,) => Promise<Result>;
  },
): Promise<readonly Result[]> {
  /**
   Chunk width, at least one.
   */
  const width = Math.max(
    1,
    Math.floor(size,),
  );
  if (items.length === 0)
    return [];
  /**
   Results of the first chunk.
   */
  const first = await Promise.all(items.slice(
    0,
    width,
  )
    .map(task,),);
  /**
   Results of the remaining chunks.
   */
  const rest = await mapInChunks({
    items: items.slice(width,),
    size: width,
    task,
  },);
  return [
    ...first,
    ...rest,
  ];
}
