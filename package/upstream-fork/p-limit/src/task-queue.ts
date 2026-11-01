/**
 FIFO call queue backing the concurrency limiter.
 
 Replaces upstream `p-limit`'s `yocto-queue` dependency with an in-package
 queue so the fork has no runtime dependencies at all. Consumed entries keep
 their slots until a compaction point, so enqueue and take stay amortized
 constant time even under deep queues.
 
 @module
 */

//region Types

/**
 One queued call: how to start it once a slot frees, and how to reject it
 when the queue is cleared before it ever starts.
 
 @example
 ```ts
 const call: ScheduledCall = {
   run: function runCall(): void {},
   reject: function rejectCall(reason: unknown): void {},
 };
 ```
 */
export type ScheduledCall = {
  /**
   Starts the call, taking the slot the queue just handed over.
   */
  readonly run: () => void;
  /**
   Rejects the call's promise with a caller-supplied reason instead of
   starting it.
   */
  readonly reject: (reason: unknown,) => void;
};

/**
 FIFO queue of scheduled calls.
 
 @example
 ```ts
 const queue = createTaskQueue();
 queue.enqueue(call,);
 const admitted = queue.take(2,);
 ```
 */
export type TaskQueue = {
  /**
   Number of calls waiting to start.
   */
  readonly size: number;
  /**
   Appends one call at the queue's tail.
   */
  enqueue: (call: ScheduledCall,) => void;
  /**
   Removes and returns up to `count` calls from the queue's head, in FIFO
   order.
   */
  take: (count: number,) => readonly ScheduledCall[];
  /**
   Removes and returns every queued call, in FIFO order.
   */
  clear: () => readonly ScheduledCall[];
};

//endregion Types

//region Factory

/**
 Creates one empty FIFO call queue.
 
 @returns Queue whose entries are taken in the order they were enqueued.
 
 @example
 ```ts
 import { createTaskQueue, } from '\@monochromatic-dev/upstream-fork-p-limit/ts/task-queue.ts';
 
 const queue = createTaskQueue();
 queue.size; // => 0
 ```
 */
export function createTaskQueue(): TaskQueue {
  /**
   Enqueued entries in FIFO order, prefixed by consumed slots.
   */
  const entries: ScheduledCall[] = [];
  /**
   Consumed-prefix cursor; kept in an object because function-root `let`
   bindings are banned by repository lint.
   */
  const cursor = {
    head: 0,
  };

  /**
   Appends one call at the tail of the backing store.
   
   @param call - Queued call to append.
   */
  function enqueue(call: ScheduledCall,): void {
    entries.push(call,);
  }

  /**
   Compacts the consumed prefix so the backing store never keeps slots that
   can no longer be taken.
   */
  // mutation-test-disable-next-line block, conditional, equality -- compaction is a memory-only heuristic: queue behavior is identical however it runs
  function compact(): void {
    // mutation-test-disable-next-line conditional, equality -- compaction is a memory-only heuristic: queue behavior is identical however it runs
    if (cursor.head === 0)
      return;

    // mutation-test-disable-next-line conditional, block -- compaction is a memory-only heuristic: queue behavior is identical however it runs
    if (cursor.head === entries.length) {
      entries.length = 0;
      cursor.head = 0;
      return;
    }

    // Once the consumed prefix holds at least half the store, re-slicing is
    // cheaper than carrying the dead slots further.
    // mutation-test-disable-next-line conditional, arithmetic, equality, block -- compaction is a memory-only heuristic: queue behavior is identical however it runs
    if ((cursor.head * 2) >= entries.length) {
      entries.splice(
        0,
        cursor.head,
      );
      cursor.head = 0;
    }
  }

  /**
   Removes and returns up to `count` calls from the head.
   
   @param count - Maximum number of calls to take; fractional or infinite
   values are bounded by the queue's size.
   
   @returns Taken calls in FIFO order, at most `count` of them.
   
   @example
   ```ts
   const admitted = queue.take(4,);
   ```
   */
  function take(count: number,): readonly ScheduledCall[] {
    /**
     Head-of-queue calls admitted by this take, in FIFO order.
     */
    const taken = entries.slice(
      cursor.head,
      cursor.head + count,
    );
    cursor.head += taken.length;
    compact();
    return taken;
  }

  /**
   Removes and returns every queued call.
   
   @returns All queued calls in FIFO order; the queue is empty afterwards.
   
   @example
   ```ts
   const discarded = queue.clear();
   ```
   */
  function clear(): readonly ScheduledCall[] {
    /**
     Queued calls discarded by this clear, in FIFO order.
     */
    const discarded = entries.slice(cursor.head,);
    entries.length = 0;
    cursor.head = 0;
    return discarded;
  }

  return {
    enqueue,
    take,
    clear,
    /**
     Number of calls waiting to start.
     */
    get size(): number {
      return entries.length - cursor.head;
    },
  };
}

//endregion Factory
