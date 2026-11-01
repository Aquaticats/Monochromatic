/**
 Deterministic coverage driver: exercises every exported function and its
 error paths with fixed inputs, so the V8 coverage it produces is
 reproducible. Run under `NODE_V8_COVERAGE` by the `fuzz:coverage` task,
 then summarized by `coverage-report.ts`.
 
 @module
 */

import {
  cacheSize,
  clearPatternCache,
  compileMatchers,
  compilePattern,
  foldCase,
  foldCharacter,
  getPattern,
  isFastPathEligible,
  isMatch,
  matchParts,
  matcher,
  resolveMatcherOptions,
  sanitizeInputs,
  sanitizeList,
  sanitizePatterns,
  splitPattern,
} from '@monochromatic-dev/upstream-fork-matcher/ts';

//region Fixtures

/**
 Code point of the backslash escape character.
 */
const BACKSLASH_POINT = 92;

/**
 Backslash code unit shared by every escaped-pattern fixture.
 */
const escape = String.fromCodePoint(BACKSLASH_POINT,);

/**
 Failure thrown by assertion helpers when the driver diverges.
 */
const DRIVER_FAILURE = 'coverage driver diverged; driver inputs are stale';

/**
 Fixed inputs exercised across every driver section.
 */
const DRIVER_INPUTS: readonly string[] = [
  'foo',
  'bar',
  'moo',
  'unicorn',
  'UNICORN',
  '',
  'foo\nbar',
  'a*b',
  '😀',
  'école',
];

//endregion Fixtures

//region Helpers

/**
 Runs a thunk that is expected to throw, swallowing `Error` outcomes so the
 driver keeps exercising remaining paths. Re-throws anything that is not an
 `Error`.
 
 @param thunk - Operation expected to throw.
 
 @example
 ```ts
 swallow(function bad() {
   matcher(1, 'a',);
 });
 ```
 */
function swallow(thunk: () => void,): void {
  try {
    thunk();
  }
  catch (error: unknown) {
    if (!(Error.isError(error,)))
      throw error;
  }
}

//endregion Helpers

//region Exercise

/**
 Exercises input normalization, option resolution, case folding, pattern
 splitting and matching, cache eviction, set compilation, and both entry
 points end to end.
 
 @example
 ```ts
 await exercise();
 ```
 */
function exercise(): void {
  //region Normalization

  sanitizeInputs('foo',);
  /* oxlint-disable typescript/no-unsafe-type-assertion -- invalid runtime inputs mirror a mis-typed caller */
  sanitizeInputs([
    'a',
    undefined,
  ] as never,);
  /* oxlint-enable typescript/no-unsafe-type-assertion */
  sanitizePatterns('a*',);
  /* oxlint-disable typescript/no-unsafe-type-assertion -- invalid runtime patterns mirror a mis-typed caller */
  sanitizePatterns([
    undefined,
    '',
  ] as never,);
  /* oxlint-enable typescript/no-unsafe-type-assertion */
  sanitizeList({
    value: undefined,
    name: 'inputs',
    element: 'inputs',
  },);
  /* oxlint-disable typescript/no-unsafe-type-assertion -- invalid runtime inputs mirror a mis-typed caller */
  swallow(function invalidInputs() {
    sanitizeInputs(0 as never,);
  },);
  swallow(function invalidInputsElement() {
    sanitizeInputs([0] as never,);
  },);
  swallow(function invalidPatterns() {
    sanitizePatterns(null as never,);
  },);
  swallow(function invalidPatternsElement() {
    sanitizePatterns([false] as never,);
  },);
  /* oxlint-enable typescript/no-unsafe-type-assertion */

  //endregion Normalization

  //region Options

  resolveMatcherOptions(undefined,);
  resolveMatcherOptions(null,);
  resolveMatcherOptions({},);
  /* oxlint-disable typescript/no-unsafe-type-assertion -- invalid runtime options mirror a mis-typed caller */
  resolveMatcherOptions({
    caseSensitive: 1,
    allPatterns: 'yes',
  } as never,);
  /* oxlint-enable typescript/no-unsafe-type-assertion */
  resolveMatcherOptions({
    caseSensitive: true,
    allPatterns: true,
  },);

  //endregion Options

  //region Folding

  foldCharacter('é',);
  foldCharacter('ﬁ',);
  foldCharacter('ß',);
  isFastPathEligible('abc',);
  isFastPathEligible('aıb',);
  isFastPathEligible('😀',);
  foldCase({
    value: 'Unicorn',
    caseSensitive: true,
  },);
  foldCase({
    value: 'Unicorn',
    caseSensitive: false,
  },);
  foldCase({
    value: 'école',
    caseSensitive: false,
  },);
  foldCase({
    value: 'ß',
    caseSensitive: false,
  },);
  foldCase({
    value: '😀',
    caseSensitive: false,
  },);

  //endregion Folding

  //region Patterns

  splitPattern('abc',);
  splitPattern('a*b',);
  splitPattern(`a${escape}*b`,);
  splitPattern(`${escape}a${escape}b`,);
  splitPattern(`test${escape}`,);
  matchParts({
    input: 'abc',
    parts: ['abc'],
  },);
  matchParts({
    input: 'foobar',
    parts: [
      'foo',
      'bar',
    ],
  },);
  matchParts({
    input: 'a',
    parts: [
      'a',
      'a',
    ],
  },);
  matchParts({
    input: 'xaxbx',
    parts: [
      '',
      'a',
      'b',
      '',
    ],
  },);
  compilePattern({
    pattern: '!foo',
    caseSensitive: false,
  },);
  compilePattern({
    pattern: `${escape}!foo`,
    caseSensitive: true,
  },);

  //endregion Patterns

  //region Cache

  clearPatternCache();
  getPattern({
    pattern: 'same*',
    caseSensitive: false,
  },);
  getPattern({
    pattern: 'same*',
    caseSensitive: false,
  },);
  getPattern({
    pattern: 'same*',
    caseSensitive: true,
  },);
  if (cacheSize() !== 2)
    throw new Error(DRIVER_FAILURE,);

  //endregion Cache

  //region Compilation

  compileMatchers({
    patterns: [],
    options: {
      caseSensitive: false,
      allPatterns: false,
    },
  },);
  compileMatchers({
    patterns: [
      '*oo',
      '!foo',
    ],
    options: {
      caseSensitive: false,
      allPatterns: false,
    },
  },);
  compileMatchers({
    patterns: [
      'foo*',
      '*bar',
    ],
    options: {
      caseSensitive: false,
      allPatterns: true,
    },
  },);
  compileMatchers({
    patterns: [
      '!bar',
      '!baz',
    ],
    options: {
      caseSensitive: false,
      allPatterns: true,
    },
  },);

  //endregion Compilation

  //region Entry points

  /**
   Filtered inputs over the fixed driver inputs.
   */
  const kept = matcher({
    inputs: DRIVER_INPUTS,
    patterns: [
      '*o*',
      '!bar',
    ],
  },);
  if (!kept.includes('foo',))
    throw new Error(DRIVER_FAILURE,);

  matcher({
    inputs: 'moo',
    patterns: ['*oo'],
  },);
  matcher({
    inputs: DRIVER_INPUTS,
    patterns: [
      'f*',
      '*o*',
    ],
    options: { allPatterns: true, },
  },);
  matcher({
    inputs: DRIVER_INPUTS,
    patterns: [
      '!bar',
      '!baz',
    ],
    options: { allPatterns: true, },
  },);
  matcher({
    inputs: 'FOO',
    patterns: 'foo',
    options: { caseSensitive: true, },
  },);
  /* oxlint-disable typescript/no-unsafe-type-assertion -- invalid runtime entries mirror a mis-typed caller */
  swallow(function invalidEntryInputs() {
    matcher({
      inputs: 1 as never,
      patterns: 'a',
    },);
  },);
  swallow(function invalidEntryPatterns() {
    matcher({
      inputs: 'a',
      patterns: 1 as never,
    },);
  },);
  /* oxlint-enable typescript/no-unsafe-type-assertion */

  if (!isMatch({
    inputs: 'unicorn',
    patterns: 'uni*',
  },))
    throw new Error(DRIVER_FAILURE,);
  isMatch({
    inputs: DRIVER_INPUTS,
    patterns: [
      '!bar',
      '!baz',
    ],
    options: { allPatterns: true, },
  },);
  isMatch({
    inputs: [],
    patterns: '*',
  },);

  //endregion Entry points
}

//endregion Exercise

exercise();
