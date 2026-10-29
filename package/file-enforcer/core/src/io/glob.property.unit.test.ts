/**
 Property-based fuzz tests for the glob helpers in `./glob-split.ts` and
 `./glob-mirror.ts`.
 
 Properties: `firstGlobMetaIndex` returns the first metacharacter position
 measured in UTF-16 code units (or -1), with no earlier metacharacter;
 `splitGlob` never throws, returns an absolute base, and keeps the glob
 suffix a true suffix of the pattern; `mirrorGlobPath` substitutes
 captured wildcards positionally and rejects mismatched wildcard counts.
 
 Run plan and seed policy: see `../fuzz-budget-fixture.ts`.
 
 @module
 */

import {
  assert,
  asyncProperty,
  constant,
  constantFrom,
  integer,
  record,
  string,
} from 'fast-check';
import { isAbsolute, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { fuzzRunPlan, } from '../fuzz-budget-fixture.ts';
import {
  firstGlobMetaIndex,
  mirrorGlobPath,
  splitGlob,
} from '../../dist/final/node/index.mjs';

//region Constants and arbitraries

/**
 Run plan resolved once for every property in this file.
 */
const RUN = fuzzRunPlan();

/**
 Glob metacharacters, mirrored from `glob-split.ts` for independent checks.
 */
const GLOB_META_CHARS = '*?{[';

/**
 Arbitrary glob-like string drawn from an alphabet rich in
 metacharacters, separators, dots, and an astral code point, so the
 metacharacter scan and the code-unit indexing both get exercised.
 */
const globLikeArbitrary = string({
  unit: constantFrom(
    'a',
    'b',
    '/',
    '.',
    '*',
    '?',
    '{',
    '[',
    ']',
    '}',
    '\u{1F600}',
  ),
},);

/**
 Arbitrary repeated-character string over a single fixed unit, used to
 build mirror segments and captures from disjoint alphabets.
 
 @param unit - Character repeated to form the string.
 
 @returns Arbitrary string of zero or more copies of `unit`.
 
 @example
 ```ts
 const a = repeatedArbitrary('a');
 ```
 */
function repeatedArbitrary(unit: string,) {
  return string({ unit: constant(unit,), },);
}

//endregion Constants and arbitraries

await describe({
  name: '',
  children: [
    //region firstGlobMetaIndex

    describe({
      name: firstGlobMetaIndex.name,
      children: [
        it({
          name: 'returns the first metacharacter code unit, with none earlier',
          timeout: RUN.timeout,
          fn: async () => {
            await assert(
              asyncProperty(
                globLikeArbitrary,
                async function firstMetaCorrect(candidate,) {
                  /**
                   Reported first-metacharacter index.
                   */
                  const index = firstGlobMetaIndex(candidate,);
                  if (index === (-1)) {
                    for (let cursor = 0; cursor < candidate.length; cursor += 1) {
                      const char = candidate.charAt(cursor,);
                      expect(GLOB_META_CHARS.includes(char,),).toBe(false,);
                    }
                    return;
                  }
                  expect(index,).toBeGreaterThanOrEqual(0,);
                  expect(index,).toBeLessThan(candidate.length,);
                  /**
                   Character at the reported metacharacter index.
                   */
                  const metaChar = candidate.charAt(index,);
                  expect(GLOB_META_CHARS.includes(metaChar,),).toBe(true,);
                  for (let cursor = 0; cursor < index; cursor += 1) {
                    const char = candidate.charAt(cursor,);
                    expect(GLOB_META_CHARS.includes(char,),).toBe(false,);
                  }
                },
              ),
              RUN.params,
            );
          },
        },),
      ],
    },),

    //endregion firstGlobMetaIndex

    //region splitGlob

    describe({
      name: splitGlob.name,
      children: [
        it({
          name: 'returns an absolute base and a suffix of the pattern without throwing',
          timeout: RUN.timeout,
          fn: async () => {
            await assert(
              asyncProperty(
                globLikeArbitrary,
                async function splitConsistent(pattern,) {
                  /**
                   Resolved base, relative glob, and original prefix.
                   */
                  const [cwd, relativeGlob, originalPrefix,] = splitGlob(pattern,);
                  expect(typeof cwd,).toBe('string',);
                  expect(isAbsolute(cwd,),).toBe(true,);
                  expect(pattern.endsWith(relativeGlob,),).toBe(true,);
                  expect(
                    pattern.startsWith(originalPrefix,) || (originalPrefix === '.'),
                  ).toBe(true,);
                  if (firstGlobMetaIndex(pattern,) === (-1)) {
                    expect(relativeGlob,).toBe('',);
                    expect(originalPrefix,).toBe(pattern,);
                  }
                },
              ),
              RUN.params,
            );
          },
        },),
      ],
    },),

    //endregion splitGlob

    //region mirrorGlobPath

    describe({
      name: mirrorGlobPath.name,
      children: [
        it({
          name: 'substitutes one captured wildcard positionally',
          timeout: RUN.timeout,
          fn: async () => {
            await assert(
              asyncProperty(
                record({
                  pre: repeatedArbitrary('a',),
                  mid: repeatedArbitrary('c',),
                  post: repeatedArbitrary('b',),
                  preDest: repeatedArbitrary('d',),
                  postDest: repeatedArbitrary('e',),
                },),
                async function substitutes({
                  pre,
                  mid,
                  post,
                  preDest,
                  postDest,
                },) {
                  /**
                   Source pattern with one wildcard between fixed segments.
                   */
                  const sourcePattern = `${pre}*${post}`;
                  /**
                   Destination pattern with one wildcard, disjoint alphabet.
                   */
                  const destPattern = `${preDest}*${postDest}`;
                  /**
                   Concrete source path that matches the source pattern.
                   */
                  const sourcePath = `${pre}${mid}${post}`;
                  expect(
                    mirrorGlobPath({
                      sourcePattern,
                      destPattern,
                      sourcePath,
                    },),
                  ).toBe(`${preDest}${mid}${postDest}`,);
                },
              ),
              RUN.params,
            );
          },
        },),

        it({
          name: 'throws on mismatched wildcard counts',
          timeout: RUN.timeout,
          fn: async () => {
            await assert(
              asyncProperty(
                record({
                  sourceCount: integer({
                    min: 0,
                    max: 4,
                  },),
                  destCount: integer({
                    min: 0,
                    max: 4,
                  },),
                  sourcePath: string(),
                },).filter(function differ({
                  sourceCount,
                  destCount,
                },) {
                  return sourceCount !== destCount;
                },),
                async function rejectsMismatch({
                  sourceCount,
                  destCount,
                  sourcePath,
                },) {
                  /**
                   Source pattern carrying `sourceCount` wildcards.
                   */
                  const sourcePattern = `x${'*'.repeat(sourceCount,)}y`;
                  /**
                   Destination pattern carrying `destCount` wildcards.
                   */
                  const destPattern = `p${'*'.repeat(destCount,)}q`;
                  expect(function callMismatch() {
                    mirrorGlobPath({
                      sourcePattern,
                      destPattern,
                      sourcePath,
                    },);
                  },).toThrow('Wildcard count mismatch',);
                },
              ),
              RUN.params,
            );
          },
        },),
      ],
    },),

    //endregion mirrorGlobPath
  ],
},);
