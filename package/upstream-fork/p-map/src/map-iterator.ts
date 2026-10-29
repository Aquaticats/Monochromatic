/**
 Source iterator selection and shutdown for the concurrent mappers.
 
 Reproduces upstream `p-map`'s iterator probing in its exact access order,
 so a hostile or exotic input throws the same failure at the same probe, and
 keeps upstream's detached iterator shutdown semantics: closing a source
 never blocks the caller and never surfaces the source's close failure.
 
 @module
 */

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { InvalidInputError, } from './errors.ts';
import {
  iteratorProbe,
  type MapInput,
} from './mapper.ts';

//region Types

/**
 Iterator pulled by the concurrent mappers: synchronous or asynchronous,
 yielding elements or element promises.
 
 @typeParam Element - element type after awaiting an iterated item
 */
export type SourceIterator<Element> =
  | Iterator<Element | Promise<Element>, unknown, undefined>
    | AsyncIterator<Element | Promise<Element>, unknown, undefined>;

/**
 One yielded source item projected deep-read-only: the runners pass it into
 their detached continuations, and repository lint requires writable
 projections to be replaced at this boundary.
 
 @typeParam Element - element type after awaiting an iterated item
 */
export type YieldedItem<Element> = {
  /**
   Absent or `false` while the source yields; the union member carrying
   `true` is filtered out before this projection is used.
   */
  readonly done?: false;
  /**
   Element or element promise this item carries.
   */
  readonly value: Element | Promise<Element>;
};

/**
 Structural view of the input whose iterator slots are callable.
 
 Selection invokes the slots exactly as upstream `p-map`'s ternary does
 (probe the async slot,
 then method-call the selected slot on the input), and
 the invocation must read as `iterable[Symbol.iterator]()` on a binding named
 `iterable`, so an engine "is not a function" diagnostic names the same
 expression upstream's does.
 
 @typeParam Element - element type after awaiting an iterated item
 */
type CallableProbe<Element> = {
  readonly [Symbol.iterator]: () => Iterator<Element | Promise<Element>>;
  readonly [Symbol.asyncIterator]: () => AsyncIterator<Element | Promise<Element>>;
};

//endregion Types

//region Selection

/**
 Validates one input against upstream `p-map`'s acceptance rule, probing the
 synchronous iterator slot first exactly as upstream does.
 
 @typeParam Element - element type after awaiting an iterated item
 
 @param iterable - Inputs as typed by the public API.
 
 @throws InvalidInputError when the input exposes neither iterator slot.
 
 @throws InvalidInputError for a `null` or `undefined` input too, carrying
 the same message as any other wrong input, as `p-map` 7.1.0 does with
 optional chaining instead of letting the engine's property-access
 diagnostic through.

 @example
 ```ts
 validateInput([1, 2,],);
 ```
 */
export function validateInput<Element>(iterable: MapInput<Element>,): void {
  /**
   The input as a caller may really supply it: the public type excludes
   nullish values, but a mis-typed caller can still pass one.
   */
  const candidate: unknown = iterable;
  if (candidate === null || candidate === undefined)
    throw new InvalidInputError(candidate,);

  /**
   Both iterator slots, probed in upstream `p-map`'s access order so an
   exotic input throws at the same probe.
   */
  const probe = iteratorProbe(iterable,);
  if ((probe[Symbol.iterator] === undefined)
    && (probe[Symbol.asyncIterator] === undefined))
    throw new InvalidInputError(iterable,);
}

/**
 Selects the iterator for one input, preferring the asynchronous slot exactly
 as upstream `p-map` does.
 
 Callers run {@link validateInput} first: upstream validates input,
 mapper,
 and concurrency before creating the iterator, and an input's iterator
 factory must not run before those checks.
 
 @typeParam Element - element type after awaiting an iterated item
 
 @param input - Inputs as typed by the public API.
 
 @returns The input's async iterator when present, else its sync iterator.
 
 @example
 ```ts
 const iterator = selectIterator([1, 2,],);
 ```
 */
export function selectIterator<Element>(input: MapInput<Element>,): SourceIterator<Element> {
  /**
   Input under its upstream name, typed with callable slots: the ternary
   below is upstream `p-map`'s selection expression verbatim, slot reads and
   method calls included.
   */
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- the probe type models both slots as optional so `validateInput` can compare them; selection invokes them exactly as upstream `p-map`'s ternary does
  const iterable = iteratorProbe(input,) as CallableProbe<Element>;
  return iterable[Symbol.asyncIterator] === undefined
    ? iterable[Symbol.iterator]()
    : iterable[Symbol.asyncIterator]();
}

//endregion Selection

//region Shutdown

/**
 Closes one source iterator through its `return` method, so the source can
 release its resources the way `for await` does on early exit.
 
 Detached on purpose: callers must not await this, so a source blocked inside
 `next()` can never block them. Upstream `p-map` swallows the close failure
 outright; this fork logs it instead of discarding it, then continues the
 same way.
 
 @typeParam Element - element type after awaiting an iterated item
 
 @param iterator - Source iterator to close.
 
 @example
 ```ts
 closeIterator(iterator,);
 ```
 */
export async function closeIterator<Element>(iterator: SourceIterator<Element>,): Promise<void> {
  try {
    await iterator.return?.();
  }
// mutation-test-disable-next-line block, string -- logging only: the close failure is swallowed either way and the log output is not observable through the public API
  catch (error) {
    // Swallowed like upstream `p-map`, because the close runs detached and a
    // rethrow would surface as an unhandled rejection; the caught value is
    // logged so the failure is never silently discarded.
    /**
     Logger carrying this function's name as its tag.
     */
    const log = tagged({
      tag: closeIterator.name,
    },);
// mutation-test-disable-next-line block, string -- logging only: the close failure is swallowed either way and the log output is not observable through the public API
    try {
      // mutation-test-disable-next-line string -- logging only: the close failure is swallowed either way and the log output is not observable through the public API
      log.warn(`ignoring iterator close failure: ${caughtValueText(error,)}`,);
    }
// mutation-test-disable-next-line block, string -- logging only: the fallback message is observably identical to the primary one through the public API
    catch (renderFailure: unknown) {
      // Rendering the close failure failed too (an adversarial value whose
      // string coercion throws); the fallback keeps this function from
      // rejecting like upstream `p-map`'s total swallow of close failures
      // while still naming the failure in the log.
// mutation-test-disable-next-line string -- logging only: the fallback message is observably identical to the primary one through the public API
      log.warn(`ignoring iterator close failure whose rendering also failed: ${caughtValueText(renderFailure,)}`,);
    }
  }
}

//endregion Shutdown
