/**
 Collection matcher factories: assert uniformity across every element of an
 array actual. Holds the chai plugin that registers `allStrictEqual`,
 `allDeepEqual`, and `satisfyAll` Assertion methods, plus the
 {@link buildCollectionMatchers} builder that surfaces them as Jest-style
 matchers (`toAllBe`, `toAllEqual`, `toSatisfyAll`).
 
 Each assertion anchors on the first element: since strict (`===`) and deep
 (`eql`) equality are both transitive, "every element equals the first" is
 exactly "all elements are mutually equal". Pass/fail and negation are
 delegated to chai's own `this.assert`, so `.not` works without manual
 branching. Split from `expect-matchers.ts` to keep each module under the
 line-count limit.
 
 @module
 */

import type { MatcherSet, } from './expect-matchers.ts';

//region Chai augmentation

declare global {
  // Declaration merging into chai's ambient global `Chai.Assertion` requires a
  // `namespace` augmentation; an ES module export cannot reopen the interface.
  // oxlint-disable-next-line typescript/no-namespace -- chai augmentation must reopen the ambient `Chai` namespace
  namespace Chai {
    /* oxlint-disable typescript/consistent-type-definitions -- declaration merging into chai's ambient `Assertion` requires `interface`; a `type` alias cannot be reopened. */
    /**
     Collection assertions merged into chai's `Assertion`, consumed by the
     `toAllBe`, `toAllEqual`, and `toSatisfyAll` matchers.
     */
    interface Assertion {
      /**
       Asserts every element of an array actual strictly equals the first
       (mirrors `toBe`). Throws when the actual is not an array or has fewer
       than two elements.
       */
      readonly allStrictEqual: () => Assertion;

      /**
       Asserts every element of an array actual deep-equals the first
       (mirrors `toEqual`). Throws when the actual is not an array or has fewer
       than two elements.
       */
      readonly allDeepEqual: () => Assertion;

      /**
       Asserts a predicate holds for every element of an array actual. An empty
       array passes vacuously, mirroring `Array.prototype.every`.
       */
      readonly satisfyAll: (predicate: (value: unknown,) => boolean,) => Assertion;
    }
    /* oxlint-enable typescript/consistent-type-definitions */
  }
}

//endregion Chai augmentation

//region Collection plugin

/**
 Minimum element count for an all-equal assertion; comparing fewer than two
 values is meaningless and signals a test-author mistake, so it throws rather
 than passing vacuously.
 */
const MIN_ALL_EQUAL_VALUES = 2;

/**
 Chai plugin registering the collection assertions consumed by
 {@link buildCollectionMatchers}. Each method reads the array actual from the
 assertion's `object` flag, computes a single uniformity boolean, then hands
 pass/fail and negation to chai's `this.assert`. Takes only `chai` (the second
 `use` argument is redundant with `chai.util`) so it stays a single-parameter
 declaration.
 
 @param chai - chai module passed by `use`, source of `Assertion` and `util`
 
 @throws TypeError when an all-equal actual is not an array, or when a
 `satisfyAll` predicate is not a function
 
 @throws RangeError when an all-equal actual has fewer than two elements
 
 @mutates chai - `chai.Assertion.addMethod`, `chaiUtil.eql`, and `chaiUtil.flag` may change framework state or inspect caller hooks.
 
 @example
 ```ts
 use(collectionMatchersPlugin);
 expect([1, 1, 1]).to.allStrictEqual();
 ```
 */
export function collectionMatchersPlugin(chai: Chai.ChaiStatic,): void {
  /**
   Chai assertion utilities (`flag`, `eql`), aliased once; calls stay bound to
   this object so the methods are never referenced unbound.
   */
  const chaiUtil = chai.util;

  /**
   Reads the array actual from an assertion, throwing on misuse shared by the
   two all-equal methods.
   
   @param assertion - chai assertion whose `object` flag holds the actual
   
   @returns array actual narrowed to `readonly unknown[]`
   
   @throws TypeError when the actual is not an array
   
   @throws RangeError when the actual has fewer than two elements
   
   @mutates assertion - `chaiUtil.flag` may inspect assertion state and caller-defined hooks.
   */
  function readAllEqualActual(assertion: Chai.AssertionStatic,): readonly unknown[] {
    /**
     Raw assertion subject pulled from `object` flag before array narrowing.
     */
    const actual: unknown = chaiUtil.flag(
      assertion,
      'object',
    );
    if (!Array.isArray(actual,)) {
      throw new TypeError('all-equal matchers expect an array actual',);
    }
    if (actual.length < MIN_ALL_EQUAL_VALUES) {
      throw new RangeError(
        'all-equal matchers expect at least two values to compare',
      );
    }
    return actual;
  }

  chai.Assertion
    .addMethod(
    'allStrictEqual',
    function allStrictEqual(this: Chai.AssertionStatic,): void {
        /**
         Array actual with misuse already rejected.
         */
      const values = readAllEqualActual(this,);
        /**
         Anchor element; every other element is compared against it.
         */
      const [first, ...rest] = values;
        /**
         Whether every element strictly equals the anchor.
         */
      const allEqual = rest.every(function isStrictEqual(value,) {
        return value === first;
      },);
      this.assert(
        allEqual,
        'expected every value to strictly equal #{exp}',
        'expected values not to all strictly equal #{exp}',
        first,
        values.find(function isStrictDifferent(value,) {
          return value !== first;
        },),
        true,
      );
    },
    );

  chai.Assertion
    .addMethod(
    'allDeepEqual',
    function allDeepEqual(this: Chai.AssertionStatic,): void {
        /**
         Array actual with misuse already rejected.
         */
      const values = readAllEqualActual(this,);
        /**
         Anchor element; every other element is deep-compared against it.
         */
      const [first, ...rest] = values;
        /**
         Whether every element deep-equals the anchor.
         */
      const allEqual = rest.every(
          /**
           Compares one retained assertion value with anchor.
           
           @param value - Retained value inspected by Chai.
           
           @returns whether value deeply equals anchor.
           
           @mutates value - `chaiUtil.eql` may inspect caller getters or proxy traps.
           */
        function isDeepEqual(value,) {
          return chaiUtil.eql(
            value,
            first,
          );
        },);
      this.assert(
        allEqual,
        'expected every value to deep-equal #{exp}',
        'expected values not to all deep-equal #{exp}',
        first,
        values.find(
            /**
             Finds first retained assertion value differing from anchor.
             
             @param value - Retained value inspected by Chai.
             
             @returns whether value differs from anchor.
             
             @mutates value - `chaiUtil.eql` may inspect caller getters or proxy traps.
             */
          function isDeepDifferent(value,) {
            return !chaiUtil.eql(
              value,
              first,
            );
          },),
        true,
      );
    },
    );

  chai.Assertion
    .addMethod(
    'satisfyAll',
    function satisfyAll(
      this: Chai.AssertionStatic,
      predicate: (value: unknown,) => boolean,
    ): void {
      if ((typeof predicate) !== 'function') {
        throw new TypeError('toSatisfyAll expects a predicate function',);
      }
        /**
         Raw assertion subject pulled from `object` flag before array narrowing.
         */
      const actual: unknown = chaiUtil.flag(
        this,
        'object',
      );
      if (!Array.isArray(actual,)) {
        throw new TypeError('toSatisfyAll expects an array actual',);
      }
        /**
         Whether the predicate holds for every element; empty arrays pass.
         */
      const pass = actual.every(function satisfies(value: unknown,) {
        return predicate(value,);
      },);
      this.assert(
        pass,
        'expected every value to satisfy the predicate',
        'expected at least one value not to satisfy the predicate',
        true,
        pass,
      );
    },
    );
}

//endregion Collection plugin

//region Collection matcher builder

/**
 Collection matcher subset of {@link MatcherSet}: the array-actual assertions
 that compare every element.
 */
export type CollectionMatcherSet = Pick<
  MatcherSet,
  'toAllBe' | 'toAllEqual' | 'toSatisfyAll'
  >;

/**
 Builds the collection matcher subset from a chai Assertion instance. The
 assertion already carries the array actual (and any `.not` flag), so the
 matchers delegate straight to the plugin methods registered by
 {@link collectionMatchersPlugin}.
 
 @param a - chai assertion (may have `.not` flag set)
 
 @returns object with `toAllBe`, `toAllEqual`, and `toSatisfyAll`
 
 @example
 ```ts
 const matchers = buildCollectionMatchers({ a: chaiExpect([1, 1, 1]) });
 matchers.toAllBe();
 ```
 */
export function buildCollectionMatchers(
  {
    a,
  }: {
    readonly a: Chai.Assertion;
  },
): CollectionMatcherSet {
  return {
    toAllBe: function toAllBe(): void {
      a.to
        .allStrictEqual();
    },

    toAllEqual: function toAllEqual(): void {
      a.to
        .allDeepEqual();
    },

    toSatisfyAll: function toSatisfyAll(predicate: (value: unknown,) => boolean,): void {
      a.to
        .satisfyAll(predicate,);
    },
  };
}

//endregion Collection matcher builder
