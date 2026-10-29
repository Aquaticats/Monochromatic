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
  if ((candidate === null) || (candidate === undefined))
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

//region Pulling

/**
 Pull function over one source iterator: the item the source produced, or an
 already-settled `done` marker once the source reported `done`.

 @typeParam Element - element type after awaiting an iterated item
 */
export type SourcePuller<Element> = () =>
  | IteratorResult<Element | Promise<Element>, unknown>
    | Promise<IteratorResult<Element | Promise<Element>, unknown>>;

/**
 Whether one pulled item reports `done` the way upstream `p-map` reads it:
 through optional chaining and truthiness, so a non-conforming source's
 truthy `done` ends the pull exactly like a conforming one.

 @param item - Pulled item as the source produced it.

 @returns Whether the item's `done` is truthy.

 @example
 ```ts
 readsAsDone({ done: true, },); // => true
 ```
 */
function readsAsDone(item: unknown,): boolean {
// mutation-test-disable-next-line conditional, logical -- a nullish answer that is not guarded here fails on the same read of `done` with the same TypeError message (`reading 'done'`) in the puller instead of in the consumer, and the run rejects with that error either way
  if ((item === null) || (item === undefined))
// mutation-test-disable-next-line boolean -- a nullish answer that reads as done only latches the puller, and the run has already rejected with the TypeError of the consumer's own read of `done` by then
    return false;
  return Boolean((item as { readonly done?: unknown; }).done,);
}

/**
 Whether one pulled answer carries a callable `then`, read the way upstream
 `p-map` reads it: through optional chaining, so a nullish answer is not a
 thenable.

 @param answer - Pulled answer as the source produced it.

 @returns Whether the answer is a thenable.

 @example
 ```ts
 isThenable(Promise.resolve(1,),); // => true
 ```
 */
function isThenable(answer: unknown,): boolean {
// mutation-test-disable-next-line conditional -- whether an answer is wrapped in a native promise only decides whether an asynchronous source also latches `done` when its pull settles: both consumers set their own `done` guard in the reaction that directly follows the latch's on the same promise, with no job between them, so no pull can see a latch their guard lacks; a nullish answer is awaited in one turn wrapped or not
  if ((answer === null) || (answer === undefined))
// mutation-test-disable-next-line boolean -- whether an answer is wrapped in a native promise only decides whether an asynchronous source also latches `done` when its pull settles: both consumers set their own `done` guard in the reaction that directly follows the latch's on the same promise, with no job between them, so no pull can see a latch their guard lacks; a nullish answer is awaited in one turn wrapped or not
    return false;
// mutation-test-disable-next-line conditional, equality -- whether an answer is wrapped in a native promise only decides whether an asynchronous source also latches `done` when its pull settles: both consumers set their own `done` guard in the reaction that directly follows the latch's on the same promise, with no job between them, so no pull can see a latch their guard lacks; a synchronous answer is awaited in one turn wrapped or not
  return (typeof (answer as { readonly then?: unknown; }).then) === 'function';
}

/**
 Wraps one source iterator in a puller that stops pulling once the source has
 reported `done`, as `p-map` 7.1.0 does.

 The `done` is final, like `for await`: a source like a queue may block in
 `next()` after it is exhausted, and anything it produces after `done` is
 dropped. A synchronous source answers right away, so its `done` latches
 before the next pull can start; an asynchronous source latches as soon as
 its pull settles.

 @typeParam Element - element type after awaiting an iterated item

 @param iterator - Source iterator to pull from.

 @returns Puller returning the source's own result, or an already-settled
 `done` marker after the source reported `done`.

 @example
 ```ts
 const pull = createPuller(selectIterator([1,],),);
 ```
 */
export function createPuller<Element>(iterator: SourceIterator<Element>,): SourcePuller<Element> {
  /**
   Latch recording that the source reported `done`.
   */
  const latch = { isSourceDone: false, };

  /**
   Latches `done` when the pulled item reports it.

   @param item - Pulled item as the source produced it.
   */
  function markDone(item: unknown,): void {
    if (readsAsDone(item,))
      latch.isSourceDone = true;
  }

  /**
   Latches `done` once one pull settles; a rejection is the caller's to
   handle, so it is only logged here.

   @param pull - Native promise of the pulled item.
   */
  async function latchWhenSettled(pull: Promise<unknown>,): Promise<void> {
    try {
      markDone(await pull,);
    }
// mutation-test-disable-next-line block, string -- logging only: the caller of the puller handles the same rejection, and the log output is not observable through the public API
    catch (error) {
      // mutation-test-disable-next-line string -- logging only: the caller of the puller handles the same rejection, and the log output is not observable through the public API
      /**
       Logger carrying the puller's name as its tag.
       */
      const log = tagged({ tag: createPuller.name, },);
// mutation-test-disable-next-line string -- logging only: the caller of the puller handles the same rejection, and the log output is not observable through the public API
      log.debug(`pull rejected, left to the caller: ${caughtValueText(error,)}`,);
    }
  }

  return function pullNext(): ReturnType<SourcePuller<Element>> {
    if (latch.isSourceDone)
      return {
        done: true,
        value: undefined,
      };

    /**
     The source's own answer: an item for a synchronous source, a promise of
     one for an asynchronous source.
     */
    const result = iterator.next();
    markDone(result,);

    // A synchronous source answers right away, so its `done` was read above
    // and the answer goes back untouched.
// mutation-test-disable-next-line conditional, boolean -- whether an answer is wrapped in a native promise only decides whether an asynchronous source also latches `done` when its pull settles: both consumers set their own `done` guard in the reaction that directly follows the latch's on the same promise, with no job between them, so no pull can see a latch their guard lacks; a synchronous answer is awaited in one turn wrapped or not
    if (!isThenable(result,))
      return result;

    // An asynchronous source settles later, so its `done` is read off the
    // pull once it settles. The caller keeps awaiting the pull itself rather
    // than a derived promise, so an asynchronous source pays no extra
    // microtask per element. The rejection is handled here too, so a pull
    // never escapes as an unhandled rejection. A thenable that is not a
    // promise is turned into one first, so it is subscribed to once, like
    // `for await` does, while a native promise is returned as is.
    /**
     The pull as a native promise, so the `done` latch and the caller share a
     single subscription to the source's thenable.
     */
    const promise = Promise.resolve(result,);
    void latchWhenSettled(promise,);
    return promise;
  };
}

//endregion Pulling

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
