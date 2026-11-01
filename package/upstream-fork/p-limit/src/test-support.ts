/**
 Test-only scheduling helpers shared by this package's unit tests.
 
 Gates make concurrency tests deterministic: each scheduled call blocks on
 its own gate, and the test releases gates one at a time in a chosen order,
 so start and settle sequences are decided by the test rather than by
 scheduler timing.
 
 @module
 */

import { wait, } from '@monochromatic-dev/async-time-core/ts';

//region Gates

/**
 One controllable completion gate: a blocked call awaits `open`, the test
 finishes it with `release`.
 
 @example
 ```ts
 const gate = createGate();
 await gate.open;
 ```
 */
export type Gate = {
  /**
   Promise settled by {@link Gate.release}.
   */
  readonly open: Promise<void>;
  /**
   Settles {@link Gate.open}, letting the blocked call continue.
   */
  readonly release: () => void;
};

/**
 Creates one openable gate.
 
 @returns Gate whose promise is pending until released.
 
 @example
 ```ts
 const gate = createGate();
 gate.release();
 await gate.open;
 ```
 */
export function createGate(): Gate {
  /**
   Deferred pair backing the gate's promise and its release.
   */
  const deferred = Promise.withResolvers<void>();
  return {
    open: deferred.promise,
    release: function release(): void {
      deferred.resolve();
    },
  };
}

//endregion Gates

//region Turns

/**
 Yields one macrotask turn.
 
 A `setTimeout` turn runs after every queued microtask, so awaiting this
 guarantees the limiter's deferred call starts and drains have all run.
 
 @example
 ```ts
 await yieldTurn();
 limit.activeCount; // reflects every admitted call
 ```
 */
export async function yieldTurn(): Promise<void> {
  await wait(
    0,
  );
}

//endregion Turns
