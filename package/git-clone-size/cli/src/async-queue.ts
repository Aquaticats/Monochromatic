/**
 Tags a task's resolved value with its pool index, without `.then`, so the
 merge loop can identify which task settled.
 
 @param index - position of the task in the pool
 
 @param task - the task promise
 
 @returns the resolved value paired with its index
 
 @example
 ```ts
 const tagged = await labeled({ index: 0, task: Promise.resolve('x') });
 ```
 */
async function labeled<T,>(
  {
    index,
    task,
  }: {
    readonly index: number;
    readonly task: Promise<T>
  },
): Promise<{
  readonly index: number;
  readonly value: T
}> {
  return {
    index,
    value: await task,
  };
}

/**
 Yields the results of a fixed set of tasks in completion order (fastest
 first), so the consumer can react to each signal the instant it lands instead
 of blocking on the slowest probe.
 
 @param tasks - task promises to merge
 
 @returns async generator of resolved values in settle order
 
 @example
 ```ts
 for await (const value of mergeAsArrived({ tasks: [slow, fast] })) {
   // `fast` is yielded before `slow`
 }
 ```
 */
export async function* mergeAsArrived<T,>(
  { tasks, }: { readonly tasks: readonly Promise<T>[]; },
): AsyncGenerator<T> {
  /**
   Outstanding labeled tasks keyed by index; entries are removed as they settle.
   */
  const pending = new Map<number, Promise<{
    readonly index: number;
    readonly value: T
  }>>();
  for (const [index, task,] of tasks.entries()) {
    pending.set(
      index,
      labeled({
        index,
        task,
      },),
    );
  }
  while (pending.size > 0) {
    /**
     The next task to settle, with its index for removal.
     */
    // oxlint-disable-next-line eslint/no-await-in-loop -- intentional: race the outstanding tasks, yield the winner, then loop to race the remainder; this IS the merge-as-arrived semantics
    const settled = await Promise.race(pending.values(),);
    pending.delete(settled.index,);
    yield settled.value;
  }
}
