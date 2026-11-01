/**
 Invariant properties that need no model.

 - Inputs are never mutated: every `deepmerge` argument and every
   `deepmergeInto` source matches its pre-merge snapshot afterwards.
 - No prototype pollution: `Object.prototype` gains no own key, even with
   `__proto__`, `constructor`, and `prototype` keys in the inputs.
 - `undefined` arguments are neutral: dropping them changes nothing.
 - Two inputs cyclic along the same shallow chain merge without throwing
   into a result whose chain loops back to the result itself, the case
   upstream's `tests/deepmerge-circular.test.ts` specifies.

 Associativity is deliberately absent: n-ary merging resolves a position to
 its last value as soon as kinds differ anywhere, so `deepmerge(null, [x], [])`
 is `[]` while the pairwise nesting gives `[x]`; the README contrasts this
 "smart merge" with classic pairwise merging on purpose.

 Cycles stay a few levels deep on purpose: deep cyclic inputs are covered by
 machine-local tests only, see `doc/handover/deepmerge-ts-hardening.md`.

 Run plan and seed policy: see `./fuzz-budget.ts`.

 @module
 */

import {
  array,
  assert,
  integer,
  property,
} from 'fast-check';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  treeArbitraries,
} from './arbitraries.ts';
import { mergeArgumentsArbitrary, } from './argument-arbitraries.ts';
import { fuzzRunPlan, } from './fuzz-budget.ts';
import {
  NO_MISMATCH,
  shapeMismatch,
  snapshot,
  snapshotObject,
} from './shape.ts';
import { target, } from './target.ts';

//region Constants and arbitraries

/**
 Run plan resolved once for every property in this file.
 */
const RUN = fuzzRunPlan();

/**
 Most levels between a cyclic record and its self-reference.
 */
const MAX_CYCLE_DEPTH = 3;

/**
 Exotic argument lists, `__proto__` keys included.
 */
const exoticArguments = mergeArgumentsArbitrary({ exotic: true, },);

/**
 Exotic records, for the associativity triple and `deepmergeInto` sources.
 */
const exoticTrees = treeArbitraries({ exotic: true, },);

/**
 Own keys of `Object.prototype` before any merge ran, the pollution baseline.
 */
const OBJECT_PROTOTYPE_KEYS = Reflect.ownKeys(Object.prototype,);

/**
 Build a record whose `self` chain loops back to it after `depth` levels.

 @param depth - Levels between the record and its self-reference.
 @param extra - Additional leaf stored beside the chain.

 @returns Cyclic record; the caller must never pass it to the tree helpers.

 @example
 ```ts
 const looped = cyclicRecord({ depth: 1, extra: 0, });
 ```
 */
function cyclicRecord({ depth, extra, }: { readonly depth: number; readonly extra: unknown; },): object {
  /**
   Record the chain returns to.
   */
  const root: Record<string, unknown> = { extra, };
  /**
   Deepest record of the chain, which receives the back-reference.
   */
  const tail = Array
    .from({ length: depth, },)
    .reduce<Record<string, unknown>>((node,) => {
      /**
       Next link of the chain.
       */
      const next: Record<string, unknown> = {};
      node.self = next;
      return next;
    }, root,);
  tail.self = root;
  return root;
}

//endregion Constants and arbitraries

await describe({
  name: 'deepmerge-ts invariants',
  children: [
    it({
      name: 'deepmerge never mutates its arguments',
      timeout: RUN.timeout,
      fn: async () => {
        assert(
          property(exoticArguments, function argumentsUnchanged(values,) {
            /**
             Pre-merge copies of every argument.
             */
            const before = values.map((value,) => snapshot(value,));
            target.deepmerge(...values,);
            expect(shapeMismatch({ actual: values, expected: before, },),).toBe(NO_MISMATCH,);
          },),
          RUN.params,
        );
      },
    },),
    it({
      name: 'deepmergeInto never mutates its sources',
      timeout: RUN.timeout,
      fn: async () => {
        assert(
          property(
            treeArbitraries({ exotic: false, objectLeaves: false, undefinedLeaves: false, },).record,
            array(exoticTrees.record, { minLength: 1, maxLength: 3, },),
            function sourcesUnchanged(generated, sources,) {
              /**
               Pre-merge copies of every source.
               */
              const before = sources.map((source,) => snapshot(source,));
              /**
               Private target this run may mutate.
               */
              const mutableTarget = snapshotObject(generated,);
              target.deepmergeInto(mutableTarget, ...sources,);
              expect(shapeMismatch({ actual: sources, expected: before, },),).toBe(NO_MISMATCH,);
            },
          ),
          RUN.params,
        );
      },
    },),
    it({
      name: 'no entry point without FastUnsafe pollutes Object.prototype',
      timeout: RUN.timeout,
      fn: async () => {
        assert(
          property(exoticArguments, function prototypeUntouched(values,) {
            target.deepmerge(...values,);
            target.deepmergeCustom({ maxDepth: 2, },)(...values,);
            target.deepmergeInto({}, ...values,);
            expect(shapeMismatch({ actual: Reflect.ownKeys(Object.prototype,), expected: OBJECT_PROTOTYPE_KEYS, },),)
              .toBe(NO_MISMATCH,);
          },),
          RUN.params,
        );
      },
    },),
    it({
      name: 'undefined arguments are neutral',
      timeout: RUN.timeout,
      fn: async () => {
        assert(
          property(
            exoticArguments,
            array(integer({ min: 0, max: 4, },), { maxLength: 3, },),
            function undefinedIsNeutral(values, positions,) {
              /**
               Arguments with `undefined` spliced in at generated positions.
               */
              const padded = positions.reduce<readonly unknown[]>(
                (list, position,) => [...list.slice(0, position,), undefined, ...list.slice(position,),],
                values,
              );
              expect(shapeMismatch({ actual: target.deepmerge(...padded,), expected: target.deepmerge(...values,), },),)
                .toBe(NO_MISMATCH,);
            },
          ),
          RUN.params,
        );
      },
    },),
    it({
      name: 'inputs cyclic along the same shallow chain merge into a result with that cycle',
      timeout: RUN.timeout,
      fn: async () => {
        assert(
          property(
            integer({ min: 0, max: MAX_CYCLE_DEPTH, },),
            exoticTrees.tree,
            exoticTrees.tree,
            function cyclesStayBounded(depth, firstExtra, secondExtra,) {
              /**
               Merge of two inputs cyclic along the same chain; it must reach
               itself through that chain.
               */
              const merged: unknown = target.deepmerge(
                cyclicRecord({ depth, extra: firstExtra, },),
                cyclicRecord({ depth, extra: secondExtra, },),
              );
              /**
               Node reached by following `self` once per chain link plus one.
               */
              const reached = Array
                .from({ length: depth + 1, },)
                .reduce<unknown>((node,) => Reflect.get(node as object, 'self',), merged,);
              expect(reached,).toBe(merged,);
            },
          ),
          RUN.params,
        );
      },
    },),
    it({
      name: 'a shallow cyclic input merged with a non-cyclic tree at the same key keeps every key',
      timeout: RUN.timeout,
      fn: async () => {
        assert(
          property(
            integer({ min: 0, max: MAX_CYCLE_DEPTH, },),
            exoticTrees.record,
            exoticTrees.tree,
            function oneSidedCycle(depth, otherSelf, extra,) {
              /**
               Non-cyclic input colliding with the cycle at `self`.
               */
              const other = { self: otherSelf, beside: extra, };
              /**
               Merge that routes the non-cyclic side through cycle resolution.
               */
              const merged: unknown = target.deepmerge(cyclicRecord({ depth, extra, },), other,);
              expect(Object.getPrototypeOf(merged,),).toBe(Object.prototype,);
              expect(Reflect.ownKeys(merged as object,),).toEqual(['extra', 'self', 'beside',],);
            },
          ),
          RUN.params,
        );
      },
    },),
    it({
      name: 'deepmergeInto a shallow cyclic target from a non-cyclic tree at the same key keeps every key',
      timeout: RUN.timeout,
      fn: async () => {
        assert(
          property(
            integer({ min: 0, max: MAX_CYCLE_DEPTH, },),
            exoticTrees.record,
            exoticTrees.tree,
            function oneSidedCycleInto(depth, otherSelf, extra,) {
              /**
               Cyclic target, freshly built so the merge may mutate it.
               */
              const looped = cyclicRecord({ depth, extra, },);
              target.deepmergeInto(looped, { self: otherSelf, beside: extra, },);
              expect(Reflect.ownKeys(looped,),).toEqual(['extra', 'self', 'beside',],);
            },
          ),
          RUN.params,
        );
      },
    },),
  ],
},);
