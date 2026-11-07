/**
 Runs asynchronous work for each item strictly one after another, lazily, so a caller can stop reading and the rest never starts.

 The daily update's commands, steps and groups share one working tree and one toolchain, so they must not overlap,
 and a failed hard step must stop the ones after it. A `for` loop with `await` in its body says this directly
 but trips `no-await-in-loop`, whose intent is to catch work that could run in parallel; here it cannot.
 In an async generator `yield` of a promise waits for it, so the next item starts only when the consumer asks for it.

 @module
 */

/**
 Starts `run` for the first item, waits for its result, yields it, and starts the next only when asked.

 @param items - items in the order they must run

 @param run - starts the work for one item

 @returns the results in order; leaving the loop that reads them early leaves the remaining items unstarted

 @example
 ```ts
 for await (const exitCode of oneAfterAnother({ items: commands, run: runCommand, },)) {
   if (exitCode !== 0) { break; }
 }
 ```
 */
export async function* oneAfterAnother<Item, Result,>(
  {
    items,
    run,
  }: {
    readonly items: Iterable<Item>;
    readonly run: (item: Item,) => Promise<Result>;
  },
): AsyncGenerator<Result> {
  for (const item of items) {
    // Yielding the promise makes the generator wait for it before the loop moves to the next item.
    yield run(item,);
  }
}
