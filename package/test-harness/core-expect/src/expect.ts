/**
 Jest-style `expect` function with asymmetric matchers and scoped assertion tracking.
 
 @module
 */

import {
  match as sinonMatch,
  type SinonMatcher,
} from 'sinon';

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';

import {
  buildMatchers,
  chaiExpect,
  type MatcherSet,
} from './expect-matchers.ts';

export type { MatcherSet, } from './expect-matchers.ts';

/**
 Async versions of Jest-style matchers for promise-based assertions.
 Each method returns a Promise that resolves on success or rejects on failure.
 Used by `expect(promise).rejects` and `expect(promise).resolves`.
 */
export type AsyncMatcherSet = {
  [K in keyof MatcherSet]: (...args: Parameters<MatcherSet[K]>) => Promise<void>;
};

/**
 Static list of matcher method names, extracted once to avoid
 creating a throwaway chai assertion on every async matcher wrapping call.
 
 @example
 ```ts
 for (const key of MATCHER_KEYS) {
   // iterate all matcher names
 }
 ```
 */
const MATCHER_KEYS = [
  'toAllBe',
  'toAllEqual',
  'toSatisfyAll',
  'toBe',
  'toEqual',
  'toStrictEqual',
  'toContain',
  'toContainEqual',
  'toThrow',
  'toBeCloseTo',
  'toBeGreaterThan',
  'toBeGreaterThanOrEqual',
  'toBeLessThan',
  'toBeLessThanOrEqual',
  'toBeDefined',
  'toBeUndefined',
  'toBeNaN',
  'toBeNull',
  'toBeTruthy',
  'toBeTypeOf',
  'toBeFalsy',
  'toHaveLength',
  'toHaveProperty',
  'toMatch',
  'toMatchObject',
  'toBeInstanceOf',
  'toSatisfy',
  'toHaveBeenCalled',
  'toHaveBeenCalledExactlyOnceWith',
  'toHaveBeenCalledTimes',
  'toHaveBeenCalledWith',
  'toHaveBeenLastCalledWith',
  'toHaveBeenNthCalledWith',
  'toHaveReturned',
  'toHaveReturnedTimes',
  'toHaveReturnedWith',
  'toHaveLastReturnedWith',
  'toHaveNthReturnedWith',
] as const satisfies readonly (keyof MatcherSet)[];

//region Async matcher builders

/**
 Wraps a sync `MatcherSet` builder into an async one by awaiting a value
 first, then building and calling the corresponding sync matcher.
 
 @param getValue - Async function that produces the value to assert on
 
 @returns object with async Jest-compatible matcher methods
 */
function buildAsyncMatchers(getValue: () => Promise<unknown>,): AsyncMatcherSet {
  /**
   Creates an async wrapper for a single matcher method.
   Awaits the value, builds a sync matcher set, then calls the named method.
   
   @param key - Matcher method name
   
   @returns async function with the same signature
   */
  function wrapMatcher<K extends keyof MatcherSet,>(
    key: K,
  ): (...args: Parameters<MatcherSet[K]>) => Promise<void> {
    return async function wrappedMatcher(
      ...args: Parameters<MatcherSet[K]>
    ): Promise<void> {
      /**
       Resolved value pulled out so chai and matcher construction share one realisation of the promise.
       */
      const value = await getValue();
      /**
       Fresh sync matcher set rebuilt per call to keep state private to this assertion.
       */
      const matchers = buildMatchers({
        a: chaiExpect(value,),
        actual: value,
      },);
      // oxlint-disable-next-line no-unsafe-type-assertion -- args are typed by MatcherSet[K]; cast needed for dynamic dispatch
      (matchers[key] as (...a: readonly unknown[]) => void)(...args,);
    };
  }

  /**
   Collector populated in the loop below, then cast to the precise `AsyncMatcherSet` shape on return.
   */
  const result = {} as Record<string, (...args: readonly unknown[]) => Promise<void>>;

  for (const key of MATCHER_KEYS) {
    // oxlint-disable-next-line no-unsafe-type-assertion -- Parameters<MatcherSet[K]> produces mutable tuples; narrowing to readonly unknown[] is safe since the function never mutates args
    result[key] = wrapMatcher(key,) as (...args: readonly unknown[]) => Promise<void>;
  }

  // oxlint-disable-next-line no-unsafe-type-assertion -- result is structurally identical to AsyncMatcherSet after the loop
  return result as unknown as AsyncMatcherSet;
}

/**
 Builds an async matcher set that awaits a promise's rejection,
 then runs sync matchers against the rejected value.
 
 Overrides `toThrow` because the standard matcher delegates to
 `chai.expect(value).to.throw()`, which expects a **function** to call.
 After rejection, the value is already an error object, so `toThrow`
 must check the error's message and type directly instead.
 
 @param promise - Promise expected to reject
 
 @returns object with async Jest-compatible matcher methods
 */
function buildRejectsMatchers(promise: Promise<unknown>,): AsyncMatcherSet {
  /**
   Awaits rejection and returns the error value.
   Throws if the promise resolves instead of rejecting.
   
   @returns rejected value
   
   @throws Error when the promise resolves unexpectedly
   */
  async function getRejection(): Promise<unknown> {
    try {
      await promise;
    }
    catch (error) {
      return error;
    }
    throw new Error('Expected promise to reject, but it resolved',);
  }

  /**
   Async matcher set bound to the rejection so most matchers can be reused; `toThrow` is then overridden below.
   */
  const matchers = buildAsyncMatchers(getRejection,);

  matchers.toThrow =
    /**
     Checks rejected value against optional expectation.
     
     @param expected - Optional string, regular expression, or constructor expectation.
     
     @mutates expected - `chaiExpect(error,).to.be.instanceOf` and `chaiExpect(message,).to.match` may inspect caller hooks.
     */
    async function rejectsToThrow(
      expected?: string | RegExp | (abstract new(...args: never) => unknown),
    ): Promise<void> {
    /**
     Captured rejection value reused across the string, regex, and constructor branches.
     */
      const error = await getRejection();
      if (expected === undefined) {
      /**
       getRejection already verified the promise rejected; nothing more to check
       */
        return;
      }
      if ((typeof expected) === 'string') {
      /**
       Stringified rejection used for substring containment when an expected string was supplied.
       */
        const message = caughtValueText(error,);
        chaiExpect(message,)
          .to
          .include(expected,);
      }
      else if (expected instanceof RegExp) {
      /**
       Stringified rejection used for regex matching when an expected pattern was supplied.
       */
        const message = caughtValueText(error,);
        chaiExpect(message,)
          .to
          .match(expected,);
      }
      else {
        chaiExpect(error,)
          .to
          .be
          .instanceOf(expected,);
      }
    };

  return matchers;
}

/**
 Builds an async matcher set that awaits a promise's resolution,
 then runs sync matchers against the resolved value.
 
 @param promise - Promise expected to resolve
 
 @returns resolved value forwarded to sync matchers
 */
function buildResolvesMatchers(promise: Promise<unknown>,): AsyncMatcherSet {
  /**
   Returns the promise for resolution-based matching.
   
   @returns original promise forwarded to async matchers
   */
  function getResolution(): Promise<unknown> {
    return promise;
  }

  return buildAsyncMatchers(getResolution,);
}

//endregion Async matcher builders

//region expect function

/**
 Result of calling `expect(actual)`.
 Provides Jest-style matchers, a `not` property for negation,
 and `rejects`/`resolves` for promise assertions.
 */
export type ExpectResult = MatcherSet & {
  /**
   Negated matchers: every method asserts the opposite.
   */
  readonly not: MatcherSet;
  /**
   Async matchers that await rejection, then assert on the rejected value.
   */
  readonly rejects: AsyncMatcherSet;
  /**
   Async matchers that await resolution, then assert on the resolved value.
   */
  readonly resolves: AsyncMatcherSet;
};

/**
 Asymmetric matchers attached as static properties of `expect`.
 Used inside `toHaveBeenCalledWith` and similar matchers.
 */
type ExpectStatic = {
  /**
   Matches any value. Useful as a placeholder in `toHaveBeenCalledWith`.
   
   @returns sinon matcher that matches anything
   */
  anything: () => SinonMatcher;
  /**
   Matches any instance of the given constructor.
   
   @param ctor - Constructor function to match against
   
   @returns sinon matcher that checks `instanceof`
   */
  any: (ctor: abstract new(...args: never) => unknown,) => SinonMatcher;
  /**
   Matches any array that contains all elements of the given array, in any order.
   
   @param arr - Elements that must be present in the matched array
   
   @returns sinon matcher that checks array containment
   */
  arrayContaining: (arr: readonly unknown[],) => SinonMatcher;
  /**
   Matches any object that deeply includes the given subset.
   
   @param obj - Partial object to match against
   
   @returns sinon matcher that checks for partial object match
   */
  objectContaining: (obj: Readonly<Record<string, unknown>>,) => SinonMatcher;
  /**
   Matches any string containing the given substring.
   
   @param str - Substring to match
   
   @returns sinon matcher that checks for substring presence
   */
  stringContaining: (str: string,) => SinonMatcher;
  /**
   Matches any string that matches the given regular expression.
   
   @param pattern - Regular expression to match against
   
   @returns sinon matcher that checks regex match
   */
  stringMatching: (pattern: RegExp,) => SinonMatcher;
};

/**
 Full type for the `expect` function including both the call signature
 and the static asymmetric matcher methods.
 */
type Expect = ((actual: unknown,) => ExpectResult) & ExpectStatic;

/**
 Jest-style `expect(actual)` powered by chai, chai-as-promised, and sinon-chai.
 
 Sync matchers assert immediately.
 `rejects`/`resolves` return async matchers that await the promise first.
 
 @param actual - Value or spy to assert against
 
 @returns object with Jest-compatible matcher methods, `.not`, `.rejects`, `.resolves`
 
 @mutates actual - `chaiExpect` may inspect caller-defined hooks or retain assertion state.
 
 @example
 ```ts
 expect(add(1, 2)).toBe(3);
 expect([1, 2, 3]).toContain(2);
 expect(spy).toHaveBeenCalledWith('hello');
 expect(value).not.toBeUndefined();
 await expect(promise).rejects.toBeInstanceOf(Error);
 await expect(promise).resolves.toBe(42);
 ```
 */
function expectImpl(actual: unknown,): ExpectResult {
  return {
    ...buildMatchers({
      a: chaiExpect(actual,),
      actual,
    },),

    not: buildMatchers({
      a: chaiExpect(actual,)
        .not,
      actual,
    },),
    // oxlint-disable-next-line no-unsafe-type-assertion -- cast required for Promise.race pattern
    rejects: buildRejectsMatchers(actual as Promise<unknown>,),
    // oxlint-disable-next-line no-unsafe-type-assertion -- cast required for Promise.race pattern
    resolves: buildResolvesMatchers(actual as Promise<unknown>,),
  };
}

//endregion expect function

//region Asymmetric matchers

/**
 Matches any string containing the given substring.
 For use inside `toHaveBeenCalledWith` and similar matchers.
 
 @param str - Substring to match
 
 @returns sinon matcher that checks for substring presence
 
 @example
 ```ts
 expect(spy).toHaveBeenCalledWith(expect.stringContaining('hello'));
 ```
 */
expectImpl.stringContaining = function stringContaining(str: string,): SinonMatcher {
  return sinonMatch(str,);
};

expectImpl.stringMatching =
  /**
   Matches any string that matches given regular expression.
   
   @param pattern - Regular expression to match against.
   
   @returns Sinon matcher that checks regular-expression match.
   
   @mutates pattern - `sinonMatch` may inspect regular-expression state or caller-defined hooks.
   
   @example
   ```ts
   expect(spy).toHaveBeenCalledWith(expect.stringMatching(/^hello/));
   ```
   */
  function stringMatching(pattern: RegExp,): SinonMatcher {
    return sinonMatch(pattern,);
  };

expectImpl.objectContaining =
  /**
   Matches any object that deeply includes given subset.
   
   @param obj - Partial object to match against.
   
   @returns Sinon matcher that checks for partial object match.
   
   @mutates obj - `sinonMatch` may inspect object properties, getters, or proxy traps.
   
   @example
   ```ts
   expect(spy).toHaveBeenCalledWith(expect.objectContaining({ id: 1 }));
   ```
   */
  function objectContaining(
    obj: Readonly<Record<string, unknown>>,
  ): SinonMatcher {
    return sinonMatch(obj,);
  };

/**
 Matches any value. Useful as a placeholder in `toHaveBeenCalledWith`.
 
 @returns sinon matcher that matches anything
 
 @example
 ```ts
 expect(spy).toHaveBeenCalledWith(expect.anything(), 'specific');
 ```
 */
expectImpl.anything = function anything(): SinonMatcher {
  return sinonMatch.any;
};

expectImpl.any =
  /**
   Matches any instance of given constructor.
   
   @param ctor - Constructor function to match against.
   
   @returns Sinon matcher that checks `instanceof`.
   
   @mutates ctor - `sinonMatch.instanceOf` may inspect or retain caller-provided constructor capability.
   
   @example
   ```ts
   expect(spy).toHaveBeenCalledWith(expect.any(Error));
   ```
   */
  function any(
    ctor: abstract new(...args: never) => unknown,
  ): SinonMatcher {
    return sinonMatch.instanceOf(ctor,);
  };

expectImpl.arrayContaining =
  /**
   Matches any array containing all expected elements in any order.
   
   @param arr - Elements that must be present in matched array.
   
   @returns Sinon matcher that checks array containment.
   
   @mutates arr - `arr.join` may invoke caller-defined array accessors or proxy traps.
   
   @example
   ```ts
   expect(spy).toHaveBeenCalledWith(expect.arrayContaining([1, 2]));
   ```
   */
  function arrayContaining(
    arr: readonly unknown[],
  ): SinonMatcher {
    return sinonMatch(
      function matchArrayContaining(actual: unknown,): boolean {
        if (!Array.isArray(actual,))
          return false;
        return arr.every(function isContained(item,) {
          return actual.includes(item,);
        },);
      },
      `arrayContaining([${arr.join(', ',)}])`,
    );
  };

//endregion Asymmetric matchers

/**
 Jest-style `expect` with both a call signature and static asymmetric matchers.
 `expectImpl` is cast to `Expect` to expose the static methods to TypeScript.
 */
const expect = expectImpl as Expect;

export { expect, };

//region Scoped expect with assertion tracking

/**
 Mutable assertion counter shared between a scoped `expect` instance and
 the `it` runner that checks it after the test completes.
 */
export type AssertionTracker = {
  /**
   Number of assertions actually called.
   */
  count: number;
  /**
   Expected assertion count set by `expect.assertions(n)`; absent means unchecked.
   */
  expected?: number;
  /**
   Whether `expect.hasAssertions()` was called.
   */
  requiresAtLeastOne: boolean;
};

/**
 Scoped `expect` function with assertion tracking.
 Same API as the global `expect` plus `assertions(n)` and `hasAssertions()`.
 */
export type ScopedExpect = Expect & {
  /**
   Verify that exactly `n` assertions are called during the test.
   */
  assertions: (count: number,) => void;
  /**
   Verify that at least one assertion is called during the test.
   */
  hasAssertions: () => void;
};

/**
 Wraps a matcher set so each matcher call increments the assertion counter.
 
 @param matchers - Original matcher set
 
 @param tracker - Shared counter incremented on every assertion
 
 @returns new matcher set that counts calls
 */
function wrapMatchersWithCounter(
  {
    matchers,
    tracker,
  }: {
    readonly matchers: MatcherSet;
    readonly tracker: AssertionTracker;
  },
): MatcherSet {
  /**
   Sync wrapper collector populated in the loop below, then cast to `MatcherSet` on return.
   */
  const wrapped = {} as Record<string, (...args: readonly unknown[]) => unknown>;

  for (const key of MATCHER_KEYS) {
    /**
     Captured reference to the original sync matcher so the counted wrapper can forward to it.
     */
    const original = matchers[key];
    wrapped[key] = function countedMatcher(...args: readonly unknown[]): unknown {
      tracker.count += 1;
      // oxlint-disable-next-line no-unsafe-type-assertion -- dynamic dispatch over MatcherSet methods requires cast
      return (original as (...a: readonly unknown[]) => unknown)(...args,);
    };
  }

  // oxlint-disable-next-line no-unsafe-type-assertion -- wrapped keys match MatcherSet by construction
  return wrapped as unknown as MatcherSet;
}

/**
 Wraps an async matcher set so each matcher call increments the assertion counter.
 
 @param matchers - Original async matcher set
 
 @param tracker - Shared counter incremented on every assertion
 
 @returns new async matcher set that counts calls
 */
function wrapAsyncMatchersWithCounter(
  {
    matchers,
    tracker,
  }: {
    readonly matchers: AsyncMatcherSet;
    readonly tracker: AssertionTracker;
  },
): AsyncMatcherSet {
  /**
   Async wrapper collector populated in the loop below, then cast to `AsyncMatcherSet` on return.
   */
  const wrapped = {} as Record<string, (...args: readonly unknown[]) => Promise<unknown>>;

  for (const key of MATCHER_KEYS) {
    /**
     Captured reference to the original async matcher so the counted wrapper can forward to it.
     */
    const original = matchers[key];
    wrapped[key] = async function countedAsyncMatcher(
      ...args: readonly unknown[]
    ): Promise<unknown> {
      tracker.count += 1;
      // oxlint-disable-next-line no-unsafe-type-assertion -- dynamic dispatch over AsyncMatcherSet methods requires cast
      return await (original as (...a: readonly unknown[]) => Promise<unknown>)(...args,);
    };
  }

  // oxlint-disable-next-line no-unsafe-type-assertion -- wrapped keys match AsyncMatcherSet by construction
  return wrapped as unknown as AsyncMatcherSet;
}

/**
 Creates a scoped `expect` function that tracks assertion counts.
 Used by `it` to provide per-test assertion tracking without shared global state.
 
 @returns tuple of the scoped expect function and its assertion tracker
 
 @example
 ```ts
 const [scopedExpect, tracker] = createScopedExpect();
 scopedExpect.assertions(3);
 scopedExpect(1).toBe(1);
 // after test: check tracker.count === tracker.expected
 ```
 */
export function createScopedExpect(): readonly [
  ScopedExpect,
  AssertionTracker,
] {
  /**
   Per-test counter shared between every wrapped matcher and the parent `it` runner that checks it.
   */
  const tracker: AssertionTracker = {
    count: 0,
    requiresAtLeastOne: false,
  };

  /**
   Scoped version of `expect(actual)` that wraps all matchers with assertion counting.
   
   @param actual - Value or spy to assert against
   
   @returns expect result with counted matchers
   
   @mutates actual - `chaiExpect` may inspect caller-defined hooks or retain assertion state.
   */
  function scopedExpectImpl(actual: unknown,): ExpectResult {
    /**
     Underlying unscoped expect result reused as the source for every counted wrapper variant.
     */
    const original = expectImpl(actual,);
    return {
      ...wrapMatchersWithCounter({
        matchers: original,
        tracker,
      },),
      not: wrapMatchersWithCounter({
        matchers: original.not,
        tracker,
      },),
      rejects: wrapAsyncMatchersWithCounter({
        matchers: original.rejects,
        tracker,
      },),
      resolves: wrapAsyncMatchersWithCounter({
        matchers: original.resolves,
        tracker,
      },),
    };
  }

  scopedExpectImpl.assertions = function assertions(count: number,): void {
    tracker.expected = count;
  };

  scopedExpectImpl.hasAssertions = function hasAssertions(): void {
    tracker.requiresAtLeastOne = true;
  };

  // Copy asymmetric matchers from the global expect
  scopedExpectImpl.stringContaining = expectImpl.stringContaining;
  scopedExpectImpl.stringMatching = expectImpl.stringMatching;
  scopedExpectImpl.objectContaining = expectImpl.objectContaining;
  scopedExpectImpl.anything = expectImpl.anything;
  scopedExpectImpl.any = expectImpl.any;
  scopedExpectImpl.arrayContaining = expectImpl.arrayContaining;

  return [
    scopedExpectImpl as ScopedExpect,
    tracker,
  ] as const;
}

//endregion Scoped expect with assertion tracking
