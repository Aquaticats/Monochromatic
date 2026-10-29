/**
 Deterministic coverage driver fixtures: sources and mappers with fixed
 behaviors, so `coverage-driver.ts` exercises every reachable branch
 reproducibly.
 
 @module
 */

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import {
  pMapSkip,
  pMapStop,
} from '@monochromatic-dev/upstream-fork-p-map/ts';

//region Mappers

/**
 Fixed input list the ordered-map and stream fixtures walk.
 */
export const MAP_INPUTS: readonly number[] = [
  0,
  1,
  2,
];

/**
 Failure thrown by the synchronous-throw mapper fixture.
 */
export const THROWN_FAILURE: Error = new Error('thrown',);

/**
 Failure thrown by the failing-iterable fixture.
 */
export const ITERABLE_FAILURE: Error = new Error('iterable failed',);

/**
 Mapper returning its input unchanged.
 
 @param value - Input handed back unchanged.
 
 @returns Same value it was called with.
 
 @example
 ```ts
 identityMapper(2,); // => 2
 ```
 */
export function identityMapper(value: number,): number {
  return value;
}

/**
 Mapper dropping the middle input with {@link pMapSkip}.
 
 @param value - Input under test.
 
 @returns The input, or {@link pMapSkip} for the middle one.
 
 @example
 ```ts
 droppingMapper(1,); // => pMapSkip
 ```
 */
export function droppingMapper(value: number,): number | typeof pMapSkip {
  return (value === 1)
    ? pMapSkip
    : value;
}

/**
 Mapper ending the iteration at the middle input with {@link pMapStop}.

 @param value - Input under test.

 @returns The input, or {@link pMapStop} for the middle one.

 @example
 ```ts
 stoppingMapper(1,); // => pMapStop
 ```
 */
export function stoppingMapper(value: number,): number | typeof pMapStop {
  return (value === 1)
    ? pMapStop
    : value;
}

/**
 Mapper failing on every input with a message naming its input.
 
 @param value - Input named in the failure message.
 
 @throws Error naming the input, on every call.
 
 @example
 ```ts
 failingMapper(0,); // throws 'mapper failed 0'
 ```
 */
export function failingMapper(value: number,): never {
  throw new Error(`mapper failed ${String(value,)}`,);
}

/**
 Mapper throwing synchronously on every input.
 
 @throws {@link THROWN_FAILURE} on every call.
 
 @example
 ```ts
 throwingMapper(); // throws THROWN_FAILURE
 ```
 */
export function throwingMapper(): never {
  throw THROWN_FAILURE;
}

/**
 Mapper returning a promise resolving to its input.
 
 @param value - Input handed back unchanged.
 
 @returns Promise of the same value.
 
 @example
 ```ts
 await promisedMapper(2,); // => 2
 ```
 */
export function promisedMapper(value: number,): Promise<number> {
  return Promise.resolve(value,);
}

//endregion Mappers

//region Sources

/**
 Opens one asynchronous iterator yielding two fixed values.
 
 @returns Async iterator of `1` then `2`.
 
 @example
 ```ts
 const iterator = openTwoValues();
 ```
 */
export function openTwoValues(): AsyncIterator<number> {
  /**
   Pull cursor shared by this iterator's `next` calls.
   */
  const cursor = {
    position: 0,
  };
  return {
    next: function twoValuesNext(): Promise<IteratorResult<number>> {
      if (cursor.position >= 2)
        return Promise.resolve({
          done: true,
          value: undefined,
        },);
      cursor.position += 1;
      return Promise.resolve({
        done: false,
        value: cursor.position,
      },);
    },
  };
}

/**
 Asynchronous source yielding two fixed values.
 
 @returns Async iterable of `1` then `2`.
 
 @example
 ```ts
 for await (const value of twoAsyncValues()) console.log(value,);
 ```
 */
export function twoAsyncValues(): AsyncIterable<number> {
  return {
    [Symbol.asyncIterator]: openTwoValues,
  };
}

/**
 Iterator `next` reporting immediate exhaustion.
 
 @returns `done` result with no value.
 
 @example
 ```ts
 const step = exhaustedNext();
 ```
 */
export function exhaustedNext(): IteratorResult<number> {
  return {
    done: true,
    value: undefined,
  };
}

/**
 Opens the synchronous slot of {@link bothSlots}, yielding nothing.
 
 @returns Exhausted synchronous iterator.
 
 @example
 ```ts
 const iterator = openSyncSlot();
 ```
 */
export function openSyncSlot(): Iterator<number> {
  return {
    next: exhaustedNext,
  };
}

/**
 Opens the asynchronous slot of {@link bothSlots}, yielding `2`.
 
 @returns Asynchronous iterator of one value.
 
 @example
 ```ts
 const iterator = openAsyncSlot();
 ```
 */
export function openAsyncSlot(): AsyncIterator<number> {
  /**
   Pull cursor shared by this iterator's `next` calls.
   */
  const cursor = {
    position: 0,
  };
  return {
    next: function asyncSlotNext(): Promise<IteratorResult<number>> {
      if (cursor.position >= 1)
        return Promise.resolve({
          done: true,
          value: undefined,
        },);
      cursor.position += 1;
      return Promise.resolve({
        done: false,
        value: 2,
      },);
    },
  };
}

/**
 Source carrying both iterator slots, yielding `2` from the async slot.
 
 @returns Input exercising the asynchronous selection path.
 
 @example
 ```ts
 const results = await pMap({ iterable: bothSlots(), mapper, });
 ```
 */
export function bothSlots(): {
  readonly [Symbol.iterator]: () => Iterator<number>;
  readonly [Symbol.asyncIterator]: () => AsyncIterator<number>;
} {
  return {
    [Symbol.iterator]: openSyncSlot,
    [Symbol.asyncIterator]: openAsyncSlot,
  };
}

/**
 Number of values the close-counting source yields before reporting `done`;
 more than one, so an early consumer exit happens while the source is still
 open.
 */
const CLOSE_SOURCE_LENGTH = 4;

/**
 Close counter shared by {@link closeCounting} fixtures.
 */
export type CloseState = {
  /**
   Number of `return` calls the source observed.
   */
  closes: number;
};

/**
 Opens one instrumented iterator yielding a single value and recording its
 shutdown.
 
 @param closeState - Counter incremented by the iterator's `return` method.
 
 @returns Iterator of several values recording its shutdown.
 
 @example
 ```ts
 const iterator = openCloseCounting({ closes: 0, });
 ```
 */
export function openCloseCounting(closeState: CloseState,): Iterator<number> {
  /**
   Pull cursor shared by this iterator's `next` calls.
   */
  const cursor = {
    position: 0,
  };
  return {
    next: function closeCountingNext(): IteratorResult<number> {
      if (cursor.position >= CLOSE_SOURCE_LENGTH)
        return {
          done: true,
          value: undefined,
        };
      cursor.position += 1;
      return {
        done: false,
        value: cursor.position,
      };
    },
    return: function closeCountingReturn(): IteratorResult<number> {
      closeState.closes += 1;
      return {
        done: true,
        value: undefined,
      };
    },
  };
}

/**
 Source whose shutdown is observable through a close counter; it stays open
 across several values so an early exit still reaches its `return` method.
 
 @param closeState - Counter incremented by the source's `return` method.
 
 @returns Iterable of several values recording its shutdown.
 
 @example
 ```ts
 const source = closeCounting({ closeState: { closes: 0, }, });
 ```
 */
export function closeCounting(
  {
    closeState,
  }: {
    /**
     Counter incremented by the source's `return` method.
     */
    readonly closeState: CloseState;
  },
): Iterable<number> {
  return {
    [Symbol.iterator]: function openCloseCountingIterable(): Iterator<number> {
      return openCloseCounting(closeState,);
    },
  };
}

/**
 Iterable yielding one value then throwing {@link ITERABLE_FAILURE}.
 
 @returns Nothing; the generator always throws after its first yield.
 
 @yields Exactly one value before throwing.
 
 @throws {@link ITERABLE_FAILURE} after the first yield.
 
 @example
 ```ts
 for await (const value of oneThenThrow()) console.log(value,);
 ```
 */
export function* oneThenThrow(): Generator<number> {
  yield 1;
  throw ITERABLE_FAILURE;
}

//endregion Sources

//region Swallowing

/**
 Runs a thunk that is expected to throw, swallowing the thrown value so the
 driver keeps exercising remaining paths.
 
 Every thrown value is swallowed: abort reasons are strings or
 `DOMException`s, not `Error`s, and the driver must not stop on them.
 
 @param thunk - Operation expected to throw.
 
 @example
 ```ts
 swallow(function bad() {
   throw new TypeError('expected',);
 });
 ```
 */
export function swallow(thunk: () => void,): void {
  try {
    thunk();
  }
  catch (error: unknown) {
    // Logged rather than discarded: the driver's own assertions detect stale
    // inputs, so an expected failure must not stop the walk, but repository
    // rules forbid silent catch blocks.
    /**
     Logger carrying this function's name as its tag.
     */
    const log = tagged({
      tag: swallow.name,
    },);
    log.warn(`swallowed expected driver failure: ${caughtValueText(error,)}`,);
  }
}

/**
 Awaits a thunk that is expected to reject, swallowing the rejection reason
 so the driver keeps exercising remaining paths.
 
 Every rejection is swallowed: abort reasons are strings or `DOMException`s,
 not `Error`s, and the driver must not stop on them.
 
 @param thunk - Async operation expected to reject.
 
 @example
 ```ts
 await swallowAsync(async function bad(): Promise<void> {
   await Promise.reject(new Error('boom',),);
 });
 ```
 */
export async function swallowAsync(thunk: () => Promise<void>,): Promise<void> {
  try {
    await thunk();
  }
  catch (error: unknown) {
    // Logged rather than discarded: the driver's own assertions detect stale
    // inputs, so an expected rejection must not stop the walk, but
    // repository rules forbid silent catch blocks.
    /**
     Logger carrying this function's name as its tag.
     */
    const log = tagged({
      tag: swallowAsync.name,
    },);
    log.warn(`swallowed expected driver rejection: ${caughtValueText(error,)}`,);
  }
}

//endregion Swallowing
