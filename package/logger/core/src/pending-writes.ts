import { reportLoggerInternalError, } from './error-format.ts';

//region Completion identity bounds tracking without skipping sink deliveries

/**
 Pending completions belong to one logger and retain their existing flush deadline semantics.
 */
type PendingWrites = {
  readonly add: (write: Promise<void>,) => void;
  readonly clear: () => void;
  readonly drain: () => Promise<void>;
};

/**
 Observe rejection once per active sink operation while keeping its backend available for later records.

 @param write - sink-owned completion whose failure needs internal reporting
 */
async function observeWrite(write: Promise<void>,): Promise<void> {
  try {
    await write;
  }
  catch (error: unknown) {
    reportLoggerInternalError({
      context: 'sink write promise rejected while being tracked',
      error,
    },);
  }
}

/**
 Track actual pending operations rather than allocating continuations for every record sharing an operation.
 Every sink invocation remains the dispatcher's responsibility.

 @returns Logger-owned observations with snapshot drainage and deadline abandonment

 @example
 ```ts
 const writes = createPendingWrites();
 writes.add(sink.write(record));
 await writes.drain();
 ```
 */
export function createPendingWrites(): PendingWrites {
  /**
   Flush snapshots contain each active observation once.
   */
  const pending = new Set<Promise<void>>();
  /**
   Weak keys avoid extending a sink's completion lifetime after observation settles.
   */
  const byWrite = new WeakMap<Promise<void>, Promise<void>>();

  /**
   A timed-out observation cannot remove a newer observation of the same still-pending sink operation.
   */
  async function removeSettled({
    write,
    observed,
  }: {
    readonly write: Promise<void>;
    readonly observed: Promise<void>;
  },): Promise<void> {
    await observed;
    pending.delete(observed,);
    if (byWrite.get(write,) === observed)
      byWrite.delete(write,);
  }

  /**
   Shared completions need one observer; work abandoned by a prior flush must become trackable again.

   @param write - sink-owned completion shared by all records in its operation
   */
  function add(write: Promise<void>,): void {
    /**
     Existing observation is reusable only while the current flush view still owns it.
     */
    const existing = byWrite.get(write,);
    if ((existing !== undefined) && pending.has(existing,))
      return;
    /**
     Handled observation retains the original rejection reporting behavior.
     */
    const observed = observeWrite(write,);
    byWrite.set(
      write,
      observed,
    );
    pending.add(observed,);
    void removeSettled({
      write,
      observed,
    },);
  }

  /**
   Snapshot only work admitted before this drain, preserving the logger's existing flush boundary.
   */
  async function drain(): Promise<void> {
    await Promise.all(pending,);
  }

  /**
   Abandon observation without cancelling sink-owned writes or their rejection handlers.
   */
  function clear(): void {
    pending.clear();
  }

  return {
    add,
    clear,
    drain,
  };
}

//endregion Completion identity bounds tracking without skipping sink deliveries
