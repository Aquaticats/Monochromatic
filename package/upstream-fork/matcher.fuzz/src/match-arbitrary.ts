/**
 fast-check arbitraries generating matcher inputs, patterns, and options.
 Used by every property in this package.
 
 @module
 */

import {
  type Arbitrary,
  array,
  boolean,
  constant,
  constantFrom,
  integer,
  oneof,
  record,
} from 'fast-check';

//region Atoms

/**
 Alphabet for generated inputs: ASCII letters, wildcard and escape
 characters, negation, whitespace, and non-ASCII fold probes. Arrays (not
 spread strings) so grapheme clusters such as emoji stay whole units.
 */
const INPUT_ALPHABET: readonly [
  string,
  ...string[]
] = [
  'a',
  'b',
  'A',
  'B',
  '.',
  '*',
  '\\',
  '!',
  ' ',
  '\n',
  'ß',
  'ı',
  'é',
  'É',
  'Σ',
  'ς',
  '😀',
];

/**
 Alphabet for generated pattern bodies: inputs plus extra wildcards and
 escapes so overlapping and adjacent wildcards stay common. Arrays (not
 spread strings) so grapheme clusters stay whole units.
 */
const PATTERN_ALPHABET: readonly [
  string,
  ...string[]
] = [
  'a',
  'b',
  'A',
  'B',
  '*',
  '*',
  '\\',
  '\\',
  '!',
  '!',
  '*',
  ' ',
];

/**
 Generated short string over the input alphabet.
 */
const alphabetInputArb: Arbitrary<string> = array(
  constantFrom(...INPUT_ALPHABET as readonly [
    string,
    ...string[]
  ],),
  {
    minLength: 0,
    maxLength: 8,
  },
)
  .map(function joinUnits(units: readonly string[],): string {
    return units.join('',);
  },);

/**
 Generated pattern body over the pattern alphabet.
 */
const patternBodyArb: Arbitrary<string> = array(
  constantFrom(...PATTERN_ALPHABET as readonly [
    string,
    ...string[]
  ],),
  {
    minLength: 0,
    maxLength: 8,
  },
)
  .map(function joinUnits(units: readonly string[],): string {
    return units.join('',);
  },);

/**
 Generated pattern: an optional leading negation plus a body. Bodies are
 sometimes exactly `!` or empty, pinning the negation edge cases.
 */
const patternArb: Arbitrary<string> = oneof(
  constant(''),
  constant('!'),
  patternBodyArb,
  patternBodyArb.map(function negate(body: string,): string {
    return `!${body}`;
  },),
);

/**
 Generated matching options: both flags free.
 */
const optionsArb: Arbitrary<{
  readonly caseSensitive: boolean;
  readonly allPatterns: boolean;
}> = record({
  caseSensitive: boolean(),
  allPatterns: boolean(),
},);

/**
 Generated input list: zero to four inputs.
 */
const inputsArb: Arbitrary<readonly string[]> = array(
  alphabetInputArb,
  {
    minLength: 0,
    maxLength: 4,
  },
);

/**
 Generated pattern list: zero to four patterns.
 */
const patternsArb: Arbitrary<readonly string[]> = array(
  patternArb,
  {
    minLength: 0,
    maxLength: 4,
  },
);

//endregion Atoms

//region Cases

/**
 Generated matcher case: inputs, patterns, and options.
 */
export type MatcherCase = {
  /**
   Inputs to match.
   */
  readonly inputs: readonly string[];
  /**
   Patterns to match against.
   */
  readonly patterns: readonly string[];
  /**
   Matching flags.
   */
  readonly options: {
    readonly caseSensitive: boolean;
    readonly allPatterns: boolean;
  };
};

/**
 Arbitrary generating matcher cases.
 */
export const matcherCaseArb: Arbitrary<MatcherCase> = record({
  inputs: inputsArb,
  patterns: patternsArb,
  options: optionsArb,
},);

/**
 Arbitrary generating single input-plus-pattern pairs for the reference
 oracle, with a shared case-sensitivity flag.
 */
export const pairArb: Arbitrary<{
  readonly input: string;
  readonly pattern: string;
  readonly caseSensitive: boolean;
}> = record({
  input: alphabetInputArb,
  pattern: patternArb,
  caseSensitive: boolean(),
},);

/**
 Arbitrary generating non-negative small integers for cache probes.
 */
export const indexArb: Arbitrary<number> = integer({
  min: 0,
  max: 3_000,
});

//endregion Cases

export { patternArb, };
