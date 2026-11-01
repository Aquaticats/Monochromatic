/**
 Case generator for declared-type soundness fuzzing.

 Each case draws declared input types (`./declared-type-arbitrary.ts`),
 values that conform to them (`./declared-type-sample.ts`), and runs
 deepmerge-ts on those values; `./declared-type-emit.ts` writes the case as
 TypeScript for `tsc` to check.

 Entry points covered: `deepmerge`, `deepmergeFastUnsafe`, `deepmergeInto`
 (the asserted target type against the mutated target), and
 `deepmergeCustom` with plain option objects.

 @module
 */

import {
  array,
  constantFrom,
  oneof,
  record,
  sample,
  tuple,
  type Arbitrary,
} from 'fast-check';

import { DECLARED_TYPE_SCOPES, } from './declared-type-arbitrary.ts';
import type { TypeNode, } from './declared-type-ast.ts';
import {
  sampleValue,
  type Sampled,
} from './declared-type-sample.ts';
import { snapshotObject, } from './shape.ts';
import { target, } from './target.ts';

/**
 Entry point exercised by a case.
 */
export type CaseKind = 'custom' | 'fast' | 'into' | 'merge';

/**
 Plain `deepmergeCustom` option objects, as source text keyed by name.
 */
export const CUSTOM_OPTIONS: Readonly<Record<string, string>> = {
  empty: '{}',
  filterValuesFalse: '{ filterValues: false }',
  implicitDefault: '{ enableImplicitDefaultMerging: true }',
  mapsFalse: '{ mergeMaps: false }',
  maxDepth2: '{ maxDepth: 2 }',
  arraysFalse: '{ mergeArrays: false }',
  recordsFalse: '{ mergeRecords: false }',
  setsFalse: '{ mergeSets: false }',
};

/**
 One drawn case before emission.
 */
export type DrawnCase = {
  readonly kind: CaseKind;
  readonly option: string;
  readonly types: readonly TypeNode[];
  readonly values: readonly unknown[];
  readonly sources: readonly string[];
};

/**
 Drawn inputs of one call: declared types, conforming values, and value sources.
 */
type DrawnInputs = Pick<DrawnCase, 'sources' | 'types' | 'values'>;

/**
 Most inputs per call.
 */
const MAX_INPUTS = 3;

/**
 Relative draw weights per entry point.
 */
const CASE_WEIGHTS: Readonly<Record<CaseKind, number>> = {
  custom: 2,
  fast: 1,
  into: 2,
  merge: 4,
};

/**
 Runtime option objects matching {@link CUSTOM_OPTIONS}.
 */
const RUNTIME_OPTIONS: Readonly<Record<string, object>> = {
  empty: {},
  filterValuesFalse: { filterValues: false, },
  implicitDefault: { enableImplicitDefaultMerging: true, },
  mapsFalse: { mergeMaps: false, },
  maxDepth2: { maxDepth: 2, },
  arraysFalse: { mergeArrays: false, },
  recordsFalse: { mergeRecords: false, },
  setsFalse: { mergeSets: false, },
};

/**
 Draw types first, then values conforming to them.

 @param head - Generator of the first input's type.
 
 @param others - Generator of the remaining inputs' types.

 @returns Arbitrary of types paired with conforming values.

 @example
 ```ts
 const pairs = typesAndValues({ head: scope.type, others: scope.type, });
 ```
 */
function typesAndValues({
  head,
  others,
}: {
  readonly head: Arbitrary<TypeNode>;
  readonly others: Arbitrary<TypeNode>
},): Arbitrary<DrawnInputs> {
  return tuple(
    head,
    array(
      others,
      {
        maxLength: MAX_INPUTS - 1,
        minLength: 1,
      },
    ),
  )
    .chain(function drawValues([first, rest,]: readonly [
      TypeNode,
      readonly TypeNode[]
    ],) {
    /**
     All input types in call order.
     */
      const types = [
        first,
        ...rest,
      ];
      return tuple(...types.map(sampleValue,),)
        .map(function pair(samples: readonly Sampled[],): DrawnInputs {
          return {
            sources: samples.map(function sourceOf(drawnSample: Sampled,) {
              return drawnSample.source;
            },),
            types,
            values: samples.map(function valueOf(drawnSample: Sampled,) {
              return drawnSample.value;
            },),
          };
        },);
    },);
}

/**
 Generator of whole cases across every entry point.

 @param unions - Whether declared types may contain unions.

 @returns Case generator.

 @example
 ```ts
 const cases = caseArbitraryFor({ unions: true, });
 ```
 */
export function caseArbitraryFor({ unions, }: { readonly unions: boolean; },): Arbitrary<DrawnCase> {
  /**
   Type generators for this mode.
   */
  const scope = DECLARED_TYPE_SCOPES[unions ? 'unions' : 'noUnions'];
  /**
   Inputs of any declared type.
   */
  const anyInputs = typesAndValues({
    head: scope.type,
    others: scope.type,
  },);
  return oneof(
    {
      arbitrary: anyInputs.map(function merge(drawn: DrawnInputs,): DrawnCase {
        return {
          ...drawn,
          kind: 'merge',
          option: 'empty',
        };
      },),
      weight: CASE_WEIGHTS.merge,
    },
    {
      arbitrary: anyInputs.map(function fast(drawn: DrawnInputs,): DrawnCase {
        return {
          ...drawn,
          kind: 'fast',
          option: 'empty',
        };
      },),
      weight: CASE_WEIGHTS.fast,
    },
    // Object sources only: a non-object source is the pinned root-level kind mismatch.
    {
      arbitrary: typesAndValues({
        head: scope.object,
        others: scope.object,
      },)
        .map(function into(drawn: DrawnInputs,): DrawnCase {
          return {
            ...drawn,
            kind: 'into',
            option: 'empty',
          };
        },),
      weight: CASE_WEIGHTS.into,
    },
    {
      arbitrary: record({
        drawn: anyInputs,
        option: constantFrom(...Object.keys(CUSTOM_OPTIONS,),),
      },)
        .map(function custom(fields: {
          readonly drawn: DrawnInputs;
          readonly option: string
        },): DrawnCase {
          return {
            ...fields.drawn,
            kind: 'custom',
            option: fields.option,
          };
        },),
      weight: CASE_WEIGHTS.custom,
    },
  );
}

/**
 Run a drawn case against the build under test.

 @param drawn - Case to run.

 @returns Runtime result: the merge result, or the mutated target for `into`.

 @throws When an `into` case's target is not an object, which the generator never draws.

 @example
 ```ts
 const result = runCase(drawn);
 ```
 */
export function runCase(drawn: DrawnCase,): unknown {
  if (drawn.kind === 'merge')
    return target.deepmerge(...drawn.values,);
  if (drawn.kind === 'fast')
    return target.deepmergeFastUnsafe(...drawn.values,);
  if (drawn.kind === 'custom')
    return target.deepmergeCustom(RUNTIME_OPTIONS[drawn.option] ?? {},)(...drawn.values,);
  /**
   Drawn target and sources.
   */
  const [first, ...sources] = drawn.values;
  if (((typeof first) !== 'object') || (first === null))
    throw new TypeError(`runCase: into target is not an object: ${String(first,)}`,);
  /**
   Private copy of the target, so the drawn value stays untouched.
   */
  const intoTarget = snapshotObject(first,);
  target.deepmergeInto(
    intoTarget,
    ...sources,
  );
  return intoTarget;
}

/**
 Draw and run cases from a seed.

 @param seed - fast-check seed.
 
 @param size - Number of cases.
 
 @param unions - Whether declared types may contain unions.

 @returns Drawn cases with their runtime results; draws whose run throws are dropped.

 @example
 ```ts
 const cases = drawCases({ seed: 1, size: 10, unions: true, });
 ```
 */
export function drawCases(
  {
    seed,
    size,
    unions,
  }: {
    readonly seed: number;
    readonly size: number;
    readonly unions: boolean
  },
): readonly {
  readonly drawn: DrawnCase;
  readonly result: unknown
}[] {
  return sample(
    caseArbitraryFor({ unions, },),
    {
      numRuns: size,
      seed,
    },
  )
    .flatMap(function run(drawn: DrawnCase,) {
      try {
        return [{
          drawn,
          result: runCase(drawn,),
        },];
      } catch (error) {
      // A throwing merge is a runtime finding for other suites, not a type case.
        console.warn(`declared-type-generate: dropped a ${drawn.kind} case whose run threw: ${String(error,)}`,);
        return [];
      }
    },);
}
