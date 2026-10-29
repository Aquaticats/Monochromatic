/**
 Scalar and compound value arbitraries for the fuzz generators.
 
 `scalarSampleArbitrary` unions every leaf family (string, integer, float,
 boolean, datetime). `valueTextArbitrary` extends that to compound values
 (inline arrays and inline tables) at bounded depth, emitting only source text
 because document round-trip properties compare parsed projections rather than
 predicted values.
 
 @module
 */

import {
  type Arbitrary,
  array,
  constantFrom,
  letrec,
  type LetrecTypedTie,
  oneof,
  record,
  uniqueArray,
} from 'fast-check';

import type { ValueSample, } from './arb-types.ts';
import { datetimeSampleArbitrary, } from './arb-datetimes.ts';
import {
  type KeySegment,
  keySegmentArbitrary,
} from './arb-keys.ts';
import {
  floatSampleArbitrary,
  integerSampleArbitrary,
} from './arb-numbers.ts';
import { stringSampleArbitrary, } from './arb-strings.ts';

/**
 Boolean value samples.
 */
export const booleanSampleArbitrary: Arbitrary<ValueSample> = constantFrom(
  {
    text: 'true',
    value: true,
  } as ValueSample,
  {
    text: 'false',
    value: false,
  } as ValueSample,
);

/**
 Scalar value arbitrary unioning every leaf family.
 */
export const scalarSampleArbitrary: Arbitrary<ValueSample> = oneof(
  stringSampleArbitrary,
  integerSampleArbitrary,
  floatSampleArbitrary,
  booleanSampleArbitrary,
  datetimeSampleArbitrary,
);

/**
 Maximum nesting depth for generated compound values, bounding both array and
 inline-table recursion so a single value never overflows the parser.
 */
const MAX_VALUE_DEPTH = 3;

/**
 Maximum element or entry count in one generated array or inline table.
 */
const MAX_COMPOUND_WIDTH = 4;

/**
 One inline-table entry: a unique key segment and the value text it holds.
 */
type InlineEntry = {
  readonly key: KeySegment;
  readonly valueText: string;
};

/**
 Render an inline array from its already-rendered element texts.
 
 @returns TOML inline-array source, `[]` when empty.
 */
function renderArray({ parts, }: { readonly parts: readonly string[]; },): string {
  return parts.length === 0 ? '[]' : `[ ${parts.join(', ',)} ]`;
}

/**
 Render an inline table from its unique entries.
 
 @returns TOML inline-table source, `{}` when empty.
 */
function renderInlineTable({ entries, }: { readonly entries: readonly InlineEntry[]; },): string {
  if (entries.length === 0) return '{}';
  /**
   Per-entry `key = value` fragments joined into the inline table body.
   */
  const fragments = entries.map(function each(entry,) {
    return `${entry.key
      .text} = ${entry.valueText}`;
  },);
  return `{ ${fragments.join(', ',)} }`;
}

/**
 Resolved key name of an inline-table entry, used as the uniqueness selector.
 
 @param entry - Inline-table entry whose decoded key name to return.
 
 @returns Entry's decoded key name.
 */
function inlineEntryName(entry: InlineEntry,): string {
  return entry.key
    .name;
}

/**
 String productions of the recursive value grammar, one per `letrec` tie.
 */
type TextGrammar = {
  readonly value: string;
  readonly leaf: string;
  readonly array: string;
  readonly inlineTable: string;
};

/**
 Recursive value-text grammar: leaves are scalars; compounds are inline arrays
 and inline tables, both capped in depth and width.
 */
const valueGrammar = letrec<TextGrammar>(
  /**
   Constructs recursive grammar through fast-check tie capability.
   
   @param tie - fast-check resolver for recursive grammar branches.
   
   @returns Arbitraries for every recursive grammar branch.
   
   @mutates tie - Invoking fast-check resolver can change caller-owned generation state.
   */
  function grammar(tie: LetrecTypedTie<TextGrammar>,) {
    return {
      value: oneof(
        { maxDepth: MAX_VALUE_DEPTH, },
        tie('leaf',),
        tie('array',),
        tie('inlineTable',),
      ),
      leaf: scalarSampleArbitrary.map(function toText(sample,) { return sample.text; },),
      array: array(
        tie('value',),
        { maxLength: MAX_COMPOUND_WIDTH, },
      )
        .map(function assemble(parts: readonly string[],) { return renderArray({ parts, },); },),
      inlineTable: uniqueArray(
        record({
          key: keySegmentArbitrary,
          valueText: tie('value',),
        },),
        {
          maxLength: MAX_COMPOUND_WIDTH,
          selector: inlineEntryName,
        },
      )
        .map(function assemble(entries: readonly InlineEntry[],) { return renderInlineTable({ entries, },); },),
    };
  },);

/**
 Value-text arbitrary spanning scalars and bounded compound structures.
 */
export const valueTextArbitrary: Arbitrary<string> = valueGrammar.value;
