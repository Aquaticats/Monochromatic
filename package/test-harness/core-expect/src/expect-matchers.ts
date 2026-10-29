/**
 Matcher set builders for the test module's expect implementation.
 Holds the {@link MatcherSet} type and the sinon spy/stub matchers,
 and merges the core value matchers from `expect-matchers-core.ts`.
 The sinon matchers keep their Jest-compatible variadic signatures, so
 the no-rest-params exemption is scoped to this file in the oxlint
 config's `jestMatcherApiOverride`.
 
 @module
 */

import {
  expect as chaiExpect,
  use,
} from 'chai';
import chaiAsPromised from 'chai-as-promised';
import type { SinonSpy, } from 'sinon';
import sinonChai from 'sinon-chai';

import {
  buildCollectionMatchers,
  collectionMatchersPlugin,
} from './expect-matchers-collection.ts';
import { buildCoreMatchers, } from './expect-matchers-core.ts';

export { expect as chaiExpect, } from 'chai';

use(chaiAsPromised,);
use(sinonChai,);
use(collectionMatchersPlugin,);

//region Matcher set builder

/**
 Jest-style matcher methods backed by a chai Assertion instance.
 Each method delegates to the corresponding chai assertion.
 */
export type MatcherSet = {
  toAllBe: () => void;
  toAllEqual: () => void;
  toSatisfyAll: (predicate: (value: unknown,) => boolean,) => void;
  toBe: (expected: unknown,) => void;
  toBeCloseTo: (
    expected: number,
    precision?: number,
  ) => void;
  toBeDefined: () => void;
  toBeFalsy: () => void;
  toBeGreaterThan: (expected: number,) => void;
  toBeGreaterThanOrEqual: (expected: number,) => void;
  toBeInstanceOf: (expected: abstract new(...args: never) => unknown,) => void;
  toBeLessThan: (expected: number,) => void;
  toBeLessThanOrEqual: (expected: number,) => void;
  toBeNaN: () => void;
  toBeNull: () => void;
  toBeTruthy: () => void;
  toBeTypeOf: (
    expected: 'bigint' | 'boolean' | 'function' | 'number' | 'object' | 'string'
      | 'symbol' | 'undefined',
  ) => void;
  toBeUndefined: () => void;
  toContain: (expected: unknown,) => void;
  toContainEqual: (expected: unknown,) => void;
  toEqual: (expected: unknown,) => void;
  toHaveBeenCalled: () => void;
  toHaveBeenCalledExactlyOnceWith: (...args: readonly unknown[]) => void;
  toHaveBeenCalledTimes: (count: number,) => void;
  toHaveBeenCalledWith: (...args: readonly unknown[]) => void;
  toHaveBeenLastCalledWith: (...args: readonly unknown[]) => void;
  toHaveBeenNthCalledWith: (
    n: number,
    ...args: readonly unknown[]
  ) => void;
  toHaveLength: (expected: number,) => void;
  toHaveProperty: (
    path: string,
    value?: unknown,
  ) => void;
  toHaveReturned: () => void;
  toHaveReturnedTimes: (count: number,) => void;
  toHaveReturnedWith: (expected: unknown,) => void;
  toHaveLastReturnedWith: (expected: unknown,) => void;
  toHaveNthReturnedWith: (
    n: number,
    expected: unknown,
  ) => void;
  toMatch: (expected: string | RegExp,) => void;
  toMatchObject: (expected: Readonly<Record<string, unknown>>,) => void;
  toSatisfy: (predicate: (value: unknown,) => boolean,) => void;
  toStrictEqual: (expected: unknown,) => void;
  toThrow: (
    expected?: string | RegExp | (abstract new(...args: never) => unknown),
  ) => void;
};

/**
 Builds a Jest-style matcher set from a chai Assertion instance.
 Core value matchers come from {@link buildCoreMatchers}; the sinon
 spy/stub matchers are defined here because their variadic signatures
 need this file's no-rest-params exemption.
 
 @param a - Chai assertion (may have `.not` flag set)
 
 @param actual - Raw value being asserted on, cast to a sinon spy for the spy matchers
 
 @returns object with Jest-compatible matcher methods
 
 @mutates a - `a.to.be.a`, `a.to.be.above`, `a.to.be.at.least`, `a.to.be.at.most`, `a.to.be.below`, `a.to.be.closeTo`, `a.to.be.instanceOf`, `a.to.deep.equal`, `a.to.deep.include`, `a.to.equal`, `a.to.have.been.calledWith`, `a.to.have.callCount`, `a.to.have.lengthOf`, `a.to.have.nested.property`, `a.to.have.returned`, `a.to.include`, `a.to.match`, `a.to.satisfy`, and `a.to.throw` may change assertion state or inspect caller hooks.
 
 @example
 ```ts
 const matchers = buildMatchers({ a: chaiExpect(value), actual: value });
 matchers.toBe(42);
 ```
 */
export function buildMatchers(
  {
    a,
    actual,
  }: {
    readonly a: Chai.Assertion;
    readonly actual: unknown;
  },
): MatcherSet {
  return {
    // Core value matchers live in a sibling module to keep this file
    // under the line-count limit; merged in here so the set is whole.
    ...buildCoreMatchers({ a, },),

    // Collection matchers (`toAllBe`, `toAllEqual`, `toSatisfyAll`) compare
    // every element of an array actual; also a sibling module for line count.
    ...buildCollectionMatchers({ a, },),

    //region sinon-chai matchers

    toHaveBeenCalled: function toHaveBeenCalled(): void {
      // oxlint-disable-next-line no-unused-expressions -- sinon-chai property assertion
      a.to
        .have
        .been
        .called;
    },

    toHaveBeenCalledExactlyOnceWith:
      /**
       Checks first spy call arguments through Chai.
       
       @param args - Expected call arguments inspected by Chai.
       
       @mutates args - `chaiExpect(firstCallArgs,).to.deep.equal` may inspect caller getters or proxy traps.
       */
      function toHaveBeenCalledExactlyOnceWith(
        ...args: readonly unknown[]
      ): void {
      /* oxlint-disable no-unsafe-type-assertion -- actual is expected to be a sinon spy */
      /**
       Captured spy reference reused across the call-count and first-call argument assertions.
       */
        const spy = actual as SinonSpy;
      /* oxlint-enable no-unsafe-type-assertion */

        chaiExpect(
          spy.callCount,
          'expected spy to have been called exactly once',
        )
          .to
          .equal(1,);
      /**
       First spy-call arguments retained before assertion formatting.
       */
        const firstCallArgs = spy.firstCall
          .args;
        chaiExpect(firstCallArgs,)
          .to
          .deep
          .equal([...args,],);
      },

    toHaveBeenCalledTimes: function toHaveBeenCalledTimes(count: number,): void {
      a.to
        .have
        .callCount(count,);
    },

    toHaveBeenCalledWith: function toHaveBeenCalledWith(
      ...args: readonly unknown[]
    ): void {
      a.to
        .have
        .been
        .calledWith(...args,);
    },

    toHaveBeenLastCalledWith:
      /**
       Checks latest spy call arguments through Chai.
       
       @param args - Expected call arguments inspected by Chai.
       
       @mutates args - `chaiExpect(lastCall.args,).to.deep.equal` may inspect caller getters or proxy traps.
       */
      function toHaveBeenLastCalledWith(
        ...args: readonly unknown[]
      ): void {
      /* oxlint-disable no-unsafe-type-assertion -- actual is expected to be a sinon spy with lastCall property */
      /**
       Captured spy reference so the `lastCall` destructure below reads from a typed value.
       */
        const spy = actual as SinonSpy;
      /* oxlint-enable no-unsafe-type-assertion */
      /**
       Destructured most recent call so subsequent assertions read directly without re-accessing the spy.
       */
        const { lastCall, } = spy;

        chaiExpect(
          lastCall,
          'expected spy to have been called at least once',
        )
          .to
          .not
          .equal(null,);
        chaiExpect(lastCall.args,)
          .to
          .deep
          .equal([...args,],);
      },

    toHaveBeenNthCalledWith:
      /**
       Checks indexed spy call arguments through Chai.
       
       @param n - One-based call index.
       
       @param args - Expected call arguments inspected by Chai.
       
       @mutates args - `chaiExpect(nthCall.args,).to.deep.equal` may inspect caller getters or proxy traps.
       */
      function toHaveBeenNthCalledWith(
        n: number,
        ...args: readonly unknown[]
      ): void {
      /* oxlint-disable no-unsafe-type-assertion -- actual is expected to be a sinon spy with getCall method */
      /**
       Captured spy reference so `getCall` below reads from a typed value.
       */
        const spy = actual as SinonSpy;
      /* oxlint-enable no-unsafe-type-assertion */
      /**
       Captured call at the requested 1-based index, reused for the null check and argument comparison.
       */
        const nthCall = spy.getCall(n - 1,);

        chaiExpect(
          nthCall,
          `expected spy to have been called at least ${String(n,)} times`,
        )
          .to
          .not
          .equal(null,);
        chaiExpect(nthCall.args,)
          .to
          .deep
          .equal([...args,],);
      },

    toHaveReturned: function toHaveReturned(): void {
      /* oxlint-disable no-unsafe-type-assertion -- actual is expected to be a sinon spy */
      /**
       Captured spy reference so `getCalls` below reads from a typed value.
       */
      const spy = actual as SinonSpy;
      /* oxlint-enable no-unsafe-type-assertion */
      /**
       Captured returned-at-least-once flag reused inside the chai assertion below.
       */
      const hasReturned = spy.getCalls()
        .some(function didReturn(call,) {
          return call.exception
            === undefined;
        },);

      chaiExpect(
        hasReturned,
        'expected spy to have returned at least once',
      )
        .to
        .equal(
        true,
        );
    },

    toHaveReturnedTimes: function toHaveReturnedTimes(count: number,): void {
      /* oxlint-disable no-unsafe-type-assertion -- actual is expected to be a sinon spy */
      /**
       Captured spy reference so the call-list pipeline below reads from a typed value.
       */
      const spy = actual as SinonSpy;
      /* oxlint-enable no-unsafe-type-assertion */
      /**
       Captured count of returning calls used in the chai assertion below.
       */
      const returnCount = spy
        .getCalls()
        .filter(function didReturn(call,) {
          return call.exception
            === undefined;
        },)
        .length;

      chaiExpect(
        returnCount,
        `expected spy to have returned ${String(count,)} times`,
      )
        .to
        .equal(count,);
    },

    toHaveReturnedWith:
      /**
       Checks any spy return through Sinon-Chai.
       
       @param expected - Expected return value inspected by Sinon-Chai.
       
       @mutates expected - `a.to.have.returned` may inspect caller getters or proxy traps.
       */
      function toHaveReturnedWith(expected: unknown,): void {
        a.to
          .have
          .returned(expected,);
      },

    toHaveLastReturnedWith:
      /**
       Checks latest spy return through Chai.
       
       @param expected - Expected return value inspected by Chai.
       
       @mutates expected - `chaiExpect(lastCall.returnValue,).to.deep.equal` may inspect caller getters or proxy traps.
       */
      function toHaveLastReturnedWith(expected: unknown,): void {
      /* oxlint-disable no-unsafe-type-assertion -- actual is expected to be a sinon spy */
      /**
       Captured spy reference so the `lastCall` destructure below reads from a typed value.
       */
        const spy = actual as SinonSpy;
      /* oxlint-enable no-unsafe-type-assertion */
      /**
       Destructured most recent call so subsequent assertions read directly without re-accessing the spy.
       */
        const { lastCall, } = spy;

        chaiExpect(
          lastCall,
          'expected spy to have been called at least once',
        )
          .to
          .not
          .equal(null,);
        chaiExpect(lastCall.returnValue,)
          .to
          .deep
          .equal(expected,);
      },

    toHaveNthReturnedWith:
      /**
       Checks indexed spy return through Chai.
       
       @param n - One-based call index.
       
       @param expected - Expected return value inspected by Chai.
       
       @mutates expected - `chaiExpect(nthCall.returnValue,).to.deep.equal` may inspect caller getters or proxy traps.
       */
      function toHaveNthReturnedWith(
        n: number,
        expected: unknown,
      ): void {
      /* oxlint-disable no-unsafe-type-assertion -- actual is expected to be a sinon spy */
      /**
       Captured spy reference so `getCall` below reads from a typed value.
       */
        const spy = actual as SinonSpy;
      /* oxlint-enable no-unsafe-type-assertion */
      /**
       Captured call at the requested 1-based index, reused for the null check and return-value comparison.
       */
        const nthCall = spy.getCall(n - 1,);

        chaiExpect(
          nthCall,
          `expected spy to have been called at least ${String(n,)} times`,
        )
          .to
          .not
          .equal(null,);
        chaiExpect(nthCall.returnValue,)
          .to
          .deep
          .equal(expected,);
      },
    //endregion sinon-chai matchers
  };
}

//endregion Matcher set builder
