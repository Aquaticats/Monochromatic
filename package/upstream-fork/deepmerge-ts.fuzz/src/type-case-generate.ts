/**
 Generator for the type-level soundness corpus.

 Draws merge argument lists from a fixed seed, runs `deepmerge` on each, and
 emits `./type-soundness.generated.ts`: one case per draw, each calling
 `deepmerge` on the same arguments written as TypeScript literals and then
 assigning the runtime result, also written as a literal, to a variable of
 the call's static result type. `lint:types` therefore proves, for every
 case, that the value deepmerge-ts returns fits the type it declares.

 Inputs are plain literals (no `as const`, no annotations), so the corpus
 exercises inference-driven types; declared unions and index signatures,
 whose known divergences live in `./type-known-defect.unit.test.ts`, never
 appear. The runtime result is assigned through an intermediate variable so
 excess-property checks cannot fire on sound but wider types.

 Run through `mise run //package/upstream-fork/deepmerge-ts.fuzz:generate:type-cases`.

 @module
 */

import { writeFile, } from 'node:fs/promises';

import {
  array,
  boolean,
  constant,
  constantFrom,
  integer,
  letrec,
  oneof,
  sample,
  string,
  tuple,
  uniqueArray,
  type Arbitrary,
} from 'fast-check';

import { keyOfEntry, } from './arbitraries.ts';
import { target, } from './target.ts';

/**
 Seed of the committed corpus; change it only together with a regeneration.
 */
const CORPUS_SEED = 20_260_924;

/**
 Number of committed widened-literal cases.
 */
const CORPUS_SIZE = 600;

/**
 Seed of the committed `as const` cases, distinct so they draw other inputs.
 */
const CONST_CORPUS_SEED = 20_260_925;

/**
 Number of committed `as const` cases.
 */
const CONST_CORPUS_SIZE = 400;

/**
 Deepest container nesting in one generated argument.
 */
const MAX_NESTING = 3;

/**
 Most entries per generated container.
 */
const MAX_ENTRIES = 3;

/**
 Most arguments per `deepmerge` call.
 */
const MAX_ARGUMENTS = 3;

/**
 Recursive generator scope for emittable literals.
 */
const literalScope = letrec<{
  value: unknown;
  container: unknown
}>(function build(tie,) {
  return {
    value: oneof(
      {
        maxDepth: MAX_NESTING,
        depthIdentifier: 'literal',
      },
      constant(undefined,),
      constant(null,),
      boolean(),
      integer({
        min: -2,
        max: 9,
      },),
      string({
        maxLength: 2,
        unit: constantFrom(
          'a',
          'b',
          'z',
        ),
      },),
      integer({
        min: 0,
        max: 9,
      },)
        .map(function toDate(offset,) {
          return new Date(offset,);
        },),
      tie('container',),
    ),
    container: oneof(
      { depthIdentifier: 'literal', },
      uniqueArray(
        tuple(
          constantFrom(
            'a',
            'b',
            'c',
            '__proto__',
            'constructor',
          ),
          tie('value',),
        ),
        {
          maxLength: MAX_ENTRIES,
          selector: keyOfEntry,
        },
      )
        .map(function toRecord(entries,) {
          return Object.defineProperties(
            {},
            Object.fromEntries(entries.map(function toDescriptor([key, value,],) {
              return [
                key,
                {
                  configurable: true,
                  enumerable: true,
                  value,
                  writable: true,
                },
              ];
            },),),
          );
        },),
      array(
        tie('value',),
        { maxLength: MAX_ENTRIES, },
      ),
      array(
        tie('value',),
        { maxLength: MAX_ENTRIES, },
      )
        .map(function toSet(items,) {
          return new Set(items,);
        },),
      // One key family per Map: TypeScript infers a Map's key type from its first entry.
      ...[
        constantFrom<unknown>(
          'k',
          'm',
        ),
        constantFrom<unknown>(
          1,
          2,
        ),
      ].map(function mapOf(keyFamily,) {
        return uniqueArray(
          tuple(
            keyFamily,
            tie('value',),
          ),
          {
            maxLength: MAX_ENTRIES,
            selector: keyOfEntry,
          },
        )
          .map(function toMap(entries,) {
            return new Map(entries,);
          },);
      },),
    ),
  };
},);

/**
 Emittable literal values: JSON scalars, `undefined`, dates, and containers.
 */
const literalArbitrary: Arbitrary<unknown> = literalScope.value;

/**
 Write a value as a TypeScript expression whose inferred type matches what a
 consumer writing that literal would get.

 @param value - Value built from {@link literalArbitrary} or returned by
   deepmerge-ts for such inputs.

 @returns TypeScript source for the value.

 @example
 ```ts
 emitLiteral({ a: [1,], }); // '{ "a": [1] }'
 ```
 */
export function emitLiteral(value: unknown,): string {
  if (value === undefined)
    return 'undefined';
  if ((value === null) || ((typeof value) === 'boolean')
    || ((typeof value) === 'number')
    || ((typeof value) === 'string'))
    return JSON.stringify(value,);
  if (value instanceof Date)
    return `new Date(${String(value.getTime(),)})`;
  if (Array.isArray(value,)) {
    return value.length === 0
      ? '([] as never[])'
      : `[${value.map(emitLiteral,)
        .join(', ',)}]`;
  }
  if (value instanceof Set) {
    return value.size === 0
      ? 'new Set<never>()'
      : `new Set([${[...value,].map(emitLiteral,)
        .join(', ',)}])`;
  }
  if (value instanceof Map) {
    return value.size === 0
      ? 'new Map<never, never>()'
      : `mapOf([${[...value,].map(function entry([key, entryValue,],) {
        return `[${emitLiteral(key,)}, ${emitLiteral(entryValue,)}]`;
      },)
        .join(', ',)}])`;
  }
  if (((typeof value) === 'object') && (value !== null)) {
    // `__proto__` must be a computed key, or the literal sets the prototype instead.
    return `{ ${Object.keys(value,)
      .map(function property(key,) {
      return `${key === '__proto__' ? '["__proto__"]' : JSON.stringify(key,)}: ${emitLiteral(Reflect.get(
        value,
        key,
      ),)}`;
    },)
      .join(', ',)} }`;
  }
  throw new TypeError(`emitLiteral: no literal form for ${typeof value}`,);
}

/**
 Write one `deepmerge` argument with a `const` assertion where TypeScript
 allows one: object, array, string, number, and boolean literals.

 @param value - Top-level argument.

 @returns Argument source keeping literal and tuple types.

 @example
 ```ts
 emitConstArgument([1,]); // '[1] as const'
 ```
 */
export function emitConstArgument(value: unknown,): string {
  if (Array.isArray(value,) && (value.length === 0))
    return '[] as const';
  if ((value === undefined) || (value === null)
    || (value instanceof Date)
    || (value instanceof Set)
    || (value instanceof Map))
    return emitLiteral(value,);
  return `${emitLiteral(value,)} as const`;
}

/**
 Draw fixed-seed argument lists.

 @param seed - Corpus seed.
 
 @param size - Number of lists.

 @returns Argument lists.

 @example
 ```ts
 const lists = drawArguments({ seed: 1, size: 2, });
 ```
 */
function drawArguments({
  seed,
  size,
}: {
  readonly seed: number;
  readonly size: number
},): readonly (readonly unknown[])[] {
  return sample(
    array(
      literalArbitrary,
      {
        minLength: 1,
        maxLength: MAX_ARGUMENTS,
      },
    ),
    {
      numRuns: size,
      seed,
    },
  );
}

/**
 Emit the whole corpus module.

 @returns Source of `./type-soundness.generated.ts`.

 @example
 ```ts
 const source = emitCorpus();
 ```
 */
export function emitCorpus(): string {
  /**
   Fixed-seed argument lists.
   */
  const draws = drawArguments({
    seed: CORPUS_SEED,
    size: CORPUS_SIZE,
  },);
  /**
   One case per draw.
   */
  const cases = draws.map(function emitCase(values,) {
    /**
     Runtime result the static type must admit.
     */
    const result: unknown = target.deepmerge(...values,);
    return [
      '  () => {',
      `    const merged = deepmerge(${values.map(emitLiteral,)
        .join(', ',)});`,
      `    const value = ${emitLiteral(result,)};`,
      '    const check: typeof merged = value;',
      '    return { merged, value: check, };',
      '  },',
    ].join('\n',);
  },);
  /**
   `as const` cases: the result literal is contextually typed by the static
   result type, so literal types and tuples must match exactly and excess
   properties surface keys the type omits.
   */
  const constCases = drawArguments({
    seed: CONST_CORPUS_SEED,
    size: CONST_CORPUS_SIZE,
  },)
    .map(function emitConstCase(values,) {
    /**
     Runtime result the static type must admit.
     */
      const result: unknown = target.deepmerge(...values,);
      return [
        '  () => {',
        `    const merged = deepmerge(${values.map(emitConstArgument,)
          .join(', ',)});`,
      // Contextually typed, so an empty array must stay a bare literal to match an empty tuple.
        `    const check: typeof merged = ${emitLiteral(result,)
          .replaceAll(
          '([] as never[])',
          '[]',
        )};`,
        '    return { merged, value: check, };',
        '  },',
      ].join('\n',);
    },);
  return [
    `// Generated by src/type-case-generate.ts (seeds ${String(CORPUS_SEED,)} and ${String(CONST_CORPUS_SEED,)}). Do not edit.`,
    '// Regenerate: mise run //package/upstream-fork/deepmerge-ts.fuzz:generate:type-cases',
    '// Each case type-checks only if the runtime result fits the static result type.',
    'import { deepmerge, } from \'deepmerge-ts\';',
    '',
    'export type TypeCase = () => { readonly merged: unknown; readonly value: unknown; };',
    '',
    '// The Map constructor infers its value type from the first entry only; array literals union',
    '// their element types, so this helper types a mixed-value Map the way an annotation would.',
    'function mapOf<Entry extends readonly [unknown, unknown,],>(entries: readonly Entry[],): Map<Entry[0], Entry[1]> {',
    '  return new Map(entries,);',
    '}',
    '',
    'export const TYPE_CASES: readonly TypeCase[] = [',
    ...cases,
    '];',
    '',
    'export const CONST_TYPE_CASES: readonly TypeCase[] = [',
    ...constCases,
    '];',
    '',
  ].join('\n',);
}

if (import.meta.main)
  await writeFile(
    new URL(
      'type-soundness.generated.ts',
      import.meta.url,
    ),
    emitCorpus(),
  );
