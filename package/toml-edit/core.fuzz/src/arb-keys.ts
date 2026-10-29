/**
 Key arbitraries for the fuzz generators.
 
 A `KeySegment` pairs the TOML source spelling of one key segment with its
 decoded logical name, so document generators can both emit text and predict
 the resolved path. Coverage spans bare keys (letters, digits, underscore,
 hyphen, numeric-looking), basic and literal quoted keys (empty, unicode,
 dotted-looking, float-looking), and dotted multi-segment keys.
 
 @module
 */

import {
  type Arbitrary,
  constantFrom,
  oneof,
  string,
  tuple,
  uniqueArray,
} from 'fast-check';

import { basicStringLiteral, } from './escape.ts';

/**
 One key segment: its TOML spelling and the decoded name it resolves to.
 */
export type KeySegment = {
  /**
   TOML source for this one segment (bare token or quoted literal).
   */
  readonly text: string;
  /**
   Decoded key name this segment resolves to.
   */
  readonly name: string;
};

/**
 A dotted key: its joined TOML spelling and the resolved segment path.
 */
export type DottedKey = {
  /**
   TOML source for the whole dotted key, segments joined by `.`.
   */
  readonly text: string;
  /**
   Resolved logical path, one decoded name per segment.
   */
  readonly path: readonly string[];
};

/**
 Bare-key content arbitrary over the unquoted character class. Exported so
 document generators can draw collision-free owner names for tables and
 top-level keys.
 */
export const bareName: Arbitrary<string> = string({
  unit: constantFrom(
    'a',
    'B',
    'z',
    '0',
    '9',
    '_',
    '-',
  ),
  minLength: 1,
  maxLength: 10,
},);

/**
 Bare key segments.
 */
const bareSegment: Arbitrary<KeySegment> = bareName.map(function build(name,) {
  return {
    text: name,
    name,
  };
},);

/**
 Content for quoted keys, including shapes that are special only when bare:
 empty, dotted-looking, numeric-looking, float-looking, and unicode.
 */
const quotedKeyName: Arbitrary<string> = oneof(
  string({
    unit: 'binary',
    maxLength: 12,
  },),
  constantFrom(
    '',
    'a.b',
    '123',
    '3.14',
    'has space',
    'tab\there',
    'quote"here',
    'é€',
  ),
);

/**
 Basic-quoted key segments via the independent escaper.
 */
const quotedSegment: Arbitrary<KeySegment> = quotedKeyName.map(function build(name,) {
  return {
    text: basicStringLiteral({ content: name, },),
    name,
  };
},);

/**
 Deterministic key examples spanning bare, quoted, and edge shapes.
 */
export const KEY_SEGMENT_EXAMPLES: readonly KeySegment[] = [
  {
    text: 'plain',
    name: 'plain',
  },
  {
    text: '0',
    name: '0',
  },
  {
    text: 'a_b-c',
    name: 'a_b-c',
  },
  {
    text: '""',
    name: '',
  },
  {
    text: '"a.b"',
    name: 'a.b',
  },
  {
    text: '"3.14"',
    name: '3.14',
  },
];

/**
 Single key-segment arbitrary across bare and quoted spellings.
 */
export const keySegmentArbitrary: Arbitrary<KeySegment> = oneof(
  bareSegment,
  quotedSegment,
  constantFrom(...KEY_SEGMENT_EXAMPLES,),
);

/**
 Dotted key arbitrary of one to three segments with distinct resolved names,
 so a document built from several keys cannot collide by accident.
 */
export const dottedKeyArbitrary: Arbitrary<DottedKey> = uniqueArray(
  keySegmentArbitrary,
  {
    minLength: 1,
    maxLength: 3,
    selector: function byName(segment,) { return segment.name; },
  },
)
  .map(function build(segments: readonly KeySegment[],) {
    return {
      text: segments.map(function each(segment,) { return segment.text; },)
        .join('.',),
      path: segments.map(function each(segment,) { return segment.name; },),
    };
  },);
