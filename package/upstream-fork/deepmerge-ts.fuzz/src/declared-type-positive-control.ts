/**
 Positive control for the declared-type check (`./declared-type-check.ts`).

 A clean corpus run only means something if the check can fail. This script
 draws cases, keeps the ones that type-check, then swaps one leaf of each
 runtime result for a value of another primitive kind (number to string,
 string or boolean to number) and checks both sets in one module: every
 untampered case must still pass, and a tampered case must fail unless the
 static type at the swapped leaf admits the new kind. The report lists the
 uncaught tampered cases with their sources, so each one can be read.

 On seed 7331 the check caught 42 of 45 tampered cases; all three misses
 were `deepmergeInto` into an empty target type, which the void overload
 pinned in `./type-known-defect-declared.unit.test.ts` leaves typed `{}`.

 ```sh
 node src/declared-type-positive-control.ts
 ```

 @module
 */

import {
  join,
  resolve,
} from 'node:path';

import {
  checkCases,
  type RunCase,
} from './declared-type-check.ts';
import { drawCases, } from './declared-type-generate.ts';

/**
 Seed of the control draw.
 */
const CONTROL_SEED = 7_331;

/**
 Draws before the untampered pre-check filters them.
 */
const CONTROL_DRAWS = 80;

/**
 Package root, resolved from this file.
 */
const PACKAGE_ROOT = resolve(
  import.meta.dirname,
  '..',
);

/**
 Result of swapping the first leaf of a value.
 */
type Tampered = {
  readonly value: unknown;
  readonly done: boolean
};

/**
 Swap one primitive leaf for a value of another kind.

 @param leaf - Primitive leaf.

 @returns Replacement, or `undefined` when the leaf is not swappable.

 @example
 ```ts
 swapLeaf(1); // 'tampered'
 ```
 */
function swapLeaf(leaf: unknown,): Tampered {
  if ((typeof leaf) === 'number')
    return {
      done: true,
      value: 'tampered',
    };
  if (((typeof leaf) === 'string') || ((typeof leaf) === 'boolean'))
    return {
      done: true,
      value: 987_654,
    };
  return {
    done: false,
    value: leaf,
  };
}

/**
 Copy a result with its first swappable leaf (depth first, insertion order)
 replaced, preserving arrays, Sets, Maps, and symbol keys.

 @param value - Runtime result or a part of it.

 @returns Copy and whether a leaf was swapped.

 @example
 ```ts
 tamperFirstLeaf({ a: [1,], }); // { value: { a: ['tampered',], }, done: true, }
 ```
 */
function tamperFirstLeaf(value: unknown,): Tampered {
  if (Array.isArray(value,) || (value instanceof Set)) {
    /**
     Elements with at most the first swappable one replaced.
     */
    const elements = [...value,].reduce<{
      readonly items: readonly unknown[];
      readonly done: boolean
    }>(
      function step(
        state,
        item,
      ) {
        if (state.done)
          return {
            done: true,
            items: [
              ...state.items,
              item,
            ],
          };
      /**
       Element after a swap attempt.
       */
        const next = tamperFirstLeaf(item,);
        return {
          done: next.done,
          items: [
            ...state.items,
            next.value,
          ],
        };
      },
      {
        done: false,
        items: [],
      },
    );
    return {
      done: elements.done,
      value: Array.isArray(value,) ? elements.items : new Set(elements.items,),
    };
  }
  if (value instanceof Map) {
    /**
     Entries with at most the first swappable value replaced.
     */
    const entries = [...value,].reduce<{
      readonly items: readonly (readonly [
        unknown,
        unknown
      ])[];
      readonly done: boolean
    }>(
      function step(
        state,
        [key, entryValue,],
      ) {
        if (state.done)
          return {
            done: true,
            items: [
              ...state.items,
              [
                key,
                entryValue,
              ],
            ],
          };
      /**
       Entry value after a swap attempt.
       */
        const next = tamperFirstLeaf(entryValue,);
        return {
          done: next.done,
          items: [
            ...state.items,
            [
              key,
              next.value,
            ],
          ],
        };
      },
      {
        done: false,
        items: [],
      },
    );
    return {
      done: entries.done,
      value: new Map(entries.items,),
    };
  }
  if ((value instanceof Date) || (value instanceof RegExp))
    return {
      done: false,
      value,
    };
  if (((typeof value) === 'object') && (value !== null)) {
    /**
     Own entries with at most the first swappable value replaced.
     */
    const entries = Reflect.ownKeys(value,)
      .reduce<{
        readonly items: readonly (readonly [
          PropertyKey,
          unknown
        ])[];
        readonly done: boolean
      }>(
      function step(
        state,
        key,
      ) {
      /**
       Property value.
       */
        const propertyValue: unknown = Reflect.get(
          value,
          key,
        );
        if (state.done)
          return {
            done: true,
            items: [
              ...state.items,
              [
                key,
                propertyValue,
              ],
            ],
          };
      /**
       Property value after a swap attempt.
       */
        const next = tamperFirstLeaf(propertyValue,);
        return {
          done: next.done,
          items: [
            ...state.items,
            [
              key,
              next.value,
            ],
          ],
        };
      },
      {
        done: false,
        items: [],
      },
      );
    return {
      done: entries.done,
      value: Object.fromEntries(entries.items,),
    };
  }
  return swapLeaf(value,);
}

/**
 Run the control and throw when the check misses what it must catch.

 @throws When an untampered case fails, stray diagnostics appear, or no tampered case is caught.

 @example
 ```ts
 await runPositiveControl();
 ```
 */
export async function runPositiveControl(): Promise<void> {
  /**
   Scratch directory of the control.
   */
  const dir = join(
    PACKAGE_ROOT,
    'dist',
    'declared-type',
    'positive-control',
  );
  /**
   Drawn cases.
   */
  const drawn = drawCases({
    seed: CONTROL_SEED,
    size: CONTROL_DRAWS,
    unions: true,
  },);
  /**
   Untampered pre-check.
   */
  const precheck = await checkCases({
    cases: drawn,
    dir: join(
      dir,
      'precheck',
    ),
  },);
  /**
   Ids failing untampered, excluded from the control.
   */
  const failing = new Set(precheck.failures
    .map(function idOf(failure,) {
      return failure.id;
    },),);
  /**
   Cases that pass untampered.
   */
  const clean = drawn.filter(function passes(
    _case,
    id,
  ) {
    return !failing.has(id,);
  },);
  /**
   Tampered copies of the clean cases that had a swappable leaf.
   */
  const tampered: readonly RunCase[] = clean.flatMap(function tamper(runCase,) {
    /**
     Result after the swap attempt.
     */
    const swapped = tamperFirstLeaf(runCase.result,);
    return swapped.done ? [{
      drawn: runCase.drawn,
      result: swapped.value,
    },] : [];
  },);
  /**
   Combined check: clean cases first, tampered after.
   */
  const combined = await checkCases({
    cases: [
      ...clean,
      ...tampered,
    ],
    dir: join(
      dir,
      'combined',
    ),
  },);
  /**
   Failures among the clean cases, which must be none.
   */
  const cleanFailures = combined.failures
    .filter(function isClean(failure,) {
      return failure.id < clean.length;
    },);
  /**
   Tampered ids that the check flagged.
   */
  const caught = new Set(combined.failures
    .filter(function isTampered(failure,) {
      return failure.id >= clean.length;
    },)
    .map(function idOf(failure,) {
      return failure.id;
    },),);
  /**
   Tampered cases the check did not flag, for reading.
   */
  const missed = tampered.flatMap(function missedCase(
    _case,
    index,
  ) {
    return caught.has(clean.length + index,) ? [] : [clean.length + index,];
  },);
  console.log(`positive control: ${String(clean.length,)} clean cases pass: ${String(cleanFailures.length === 0,)}; tampered caught ${String(caught.size,)}/${String(tampered.length,)}; missed ids ${missed.join(', ',) || 'none'} (sources in ${join(
    dir,
    'combined',
    'cases.ts',
  )})`,);
  if ((cleanFailures.length > 0) || (combined.stray
    .length
    > 0))
    throw new Error(`clean cases failed or stray diagnostics appeared: ${JSON.stringify({
      cleanFailures,
      stray: combined.stray,
    },)}`,);
  if (caught.size === 0)
    throw new Error('positive control: no tampered case was caught, so a clean run proves nothing',);
}

if (import.meta.main)
  await runPositiveControl();
