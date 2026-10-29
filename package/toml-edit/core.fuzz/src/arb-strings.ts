/**
 String value arbitraries for the fuzz generators.
 
 Basic and multiline-basic strings carry arbitrary content (including quotes,
 backslashes, control scalars, and astral characters) encoded through the
 independent escaper in `./escape.ts`. Literal and multiline-literal strings
 carry content drawn from a literal-safe alphabet so no escaping is needed and
 the projected value equals the raw content exactly.
 
 @module
 */

import {
  type Arbitrary,
  constantFrom,
  oneof,
  string,
} from 'fast-check';

import type { ValueSample, } from './arb-types.ts';
import {
  basicStringLiteral,
  literalStringLiteral,
} from './escape.ts';

/**
 Alphabet whose every scalar is safe inside a single-quoted literal string:
 no single quote, no control scalars, a spread of letters, digits, spacing,
 punctuation, and non-ASCII letters.
 */
const LITERAL_SAFE_UNIT = constantFrom(
  'a',
  'Z',
  '7',
  ' ',
  '/',
  '.',
  '-',
  '"',
  '\\',
  ':',
  '=',
  '#',
  'é',
  '€',
  '😀',
);

/**
 Arbitrary content safe for literal-string encoding.
 */
const literalSafeContent: Arbitrary<string> = string({
  unit: LITERAL_SAFE_UNIT,
  maxLength: 24,
},);

/**
 Adversarial-but-valid unit for basic-string content: every scalar that
 stresses the escaper (quote, backslash, the named control escapes, a low
 control, delete, BMP and astral letters) without lone surrogates, which are
 not valid Unicode scalars and would not round-trip.
 */
const BASIC_CONTENT_UNIT = constantFrom(
  'a',
  'Z',
  ' ',
  '"',
  '\\',
  '\b',
  '\t',
  '\n',
  '\r',
  '\f',
  '\u001F',
  '\u007F',
  'é',
  '€',
  '😀',
);

/**
 Arbitrary content for basic strings over the adversarial unit so quotes,
 backslashes, and control scalars all reach the escaper.
 */
const basicContent: Arbitrary<string> = string({
  unit: BASIC_CONTENT_UNIT,
  maxLength: 24,
},);

/**
 Basic single-line strings over arbitrary content.
 */
const basicStringArbitrary: Arbitrary<ValueSample> = basicContent.map(function build(content,) {
  return {
    text: basicStringLiteral({ content, },),
    value: content,
  };
},);

/**
 Multiline basic strings; the escaper leaves no raw newline after the opening
 delimiter, so TOML's first-newline trimming never alters the value.
 */
const multilineBasicStringArbitrary: Arbitrary<ValueSample> = basicContent.map(function build(content,) {
  /**
   Single-line escaped body wrapped in triple quotes; the inner escapes keep
   the body free of raw `"""` runs.
   */
  const inner = basicStringLiteral({ content, },)
    .slice(
    1,
    -1,
    );
  return {
    text: `"""${inner}"""`,
    value: content,
  };
},);

/**
 Literal single-quoted strings over literal-safe content.
 */
const literalStringArbitrary: Arbitrary<ValueSample> = literalSafeContent.map(function build(content,) {
  return {
    text: literalStringLiteral({ content, },),
    value: content,
  };
},);

/**
 Multiline literal strings; literal-safe content has no quote or newline, so
 no `'''` run or first-newline trim can occur.
 */
const multilineLiteralStringArbitrary: Arbitrary<ValueSample> = literalSafeContent.map(function build(content,) {
  return {
    text: `'''${content}'''`,
    value: content,
  };
},);

/**
 Deterministic string examples spanning every string family.
 */
export const STRING_EXAMPLES: readonly ValueSample[] = [
  {
    text: '""',
    value: '',
  },
  {
    text: '"plain"',
    value: 'plain',
  },
  {
    text: String.raw`"quote \" and back \\"`,
    value: 'quote " and back \\',
  },
  {
    text: String.raw`"tab\tnewline\n"`,
    value: 'tab\tnewline\n',
  },
  {
    text: String.raw`'C:\path'`,
    value: String.raw`C:\path`,
  },
  {
    text: '"""triple"""',
    value: 'triple',
  },
  {
    text: "'''lit'''",
    value: 'lit',
  },
];

/**
 String value arbitrary across basic, literal, and both multiline families.
 */
export const stringSampleArbitrary: Arbitrary<ValueSample> = oneof(
  basicStringArbitrary,
  multilineBasicStringArbitrary,
  literalStringArbitrary,
  multilineLiteralStringArbitrary,
  constantFrom(...STRING_EXAMPLES,),
);
