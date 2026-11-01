/**
 Self-validation for the fuzz generators and the semantic-equality oracle.
 
 Before the generators are trusted to drive parser, emitter, and stateful
 properties, they must themselves be proven sound: every "valid" arbitrary
 must produce text the parser accepts, every predicted value must match the
 parser's projection, the independent escaper must agree with the parser's
 string decoding, and whole documents must round-trip byte-identically in
 splice mode. A failure here is a generator bug, not a package bug.
 
 Run plan and seed policy: see `../fuzz-budget.ts`.
 
 @module
 */

import {
  assert,
  asyncProperty,
} from 'fast-check';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  parseTomlEdit,
  tomlStringify,
} from '@monochromatic-dev/toml-edit-core';

import { fuzzRunPlan, } from './fuzz-budget.ts';
import { datetimeSampleArbitrary, } from './arb-datetimes.ts';
import { documentArbitrary, } from './arb-documents.ts';
import {
  floatSampleArbitrary,
  integerSampleArbitrary,
} from './arb-numbers.ts';
import { stringSampleArbitrary, } from './arb-strings.ts';
import type { ValueSample, } from './arb-types.ts';
import {
  booleanSampleArbitrary,
  valueTextArbitrary,
} from './arb-values.ts';
import {
  semanticEquals,
  semanticModel,
} from './equality.ts';

//region Helpers

/**
 Run plan resolved once for every property in this file.
 */
const RUN = fuzzRunPlan();

/**
 Project the value of `text` placed under a synthetic top-level key.
 
 @returns Native projection of the single generated value.
 */
function projectValue({ text, }: { readonly text: string; },): unknown {
  /**
   Whole document wrapping the value so the parser sees a complete statement.
   */
  const model = semanticModel({ source: `probe = ${text}\n`, },) as Record<string, unknown>;
  return model.probe;
}

/**
 Assert that a value sample parses and, when it predicts a value, matches it.
 
 @returns Nothing; throws via `expect` on a generator defect.
 */
function checkSample({ sample, }: { readonly sample: ValueSample; },): void {
  /**
   Parsed projection of the sample's text.
   */
  const projected = projectValue({ text: sample.text, },);
  if (sample.value !== undefined) {
    expect(
      semanticEquals({
        left: projected as never,
        right: sample.value,
      },),
    ).toBe(true,);
  }
}

//endregion Helpers

await describe({
  name: 'fuzz generators',
  children: [
    //region Scalar families

    it({
      name: 'integer samples parse and match their predicted value',
      timeout: RUN.timeout,
      fn: async () => {
        await assert(
          asyncProperty(integerSampleArbitrary, async function check(sample,) {
            checkSample({ sample, },);
          },),
          RUN.params,
        );
      },
    },),

    it({
      name: 'float samples parse and match their predicted value',
      timeout: RUN.timeout,
      fn: async () => {
        await assert(
          asyncProperty(floatSampleArbitrary, async function check(sample,) {
            checkSample({ sample, },);
          },),
          RUN.params,
        );
      },
    },),

    it({
      name: 'boolean samples parse and match their predicted value',
      timeout: RUN.timeout,
      fn: async () => {
        await assert(
          asyncProperty(booleanSampleArbitrary, async function check(sample,) {
            checkSample({ sample, },);
          },),
          RUN.params,
        );
      },
    },),

    it({
      name: 'string samples parse and the escaper agrees with the parser decoding',
      timeout: RUN.timeout,
      fn: async () => {
        await assert(
          asyncProperty(stringSampleArbitrary, async function check(sample,) {
            checkSample({ sample, },);
          },),
          RUN.params,
        );
      },
    },),

    it({
      name: 'datetime samples parse without error',
      timeout: RUN.timeout,
      fn: async () => {
        await assert(
          asyncProperty(datetimeSampleArbitrary, async function check(sample,) {
            expect(typeof projectValue({ text: sample.text, },),).not.toBe('undefined',);
          },),
          RUN.params,
        );
      },
    },),

    //endregion Scalar families

    //region Compound values and documents

    it({
      name: 'compound value text parses under a synthetic key',
      timeout: RUN.timeout,
      fn: async () => {
        await assert(
          asyncProperty(valueTextArbitrary, async function check(text,) {
            // Throws on a generator defect; the property is the no-throw itself.
            projectValue({ text, },);
          },),
          RUN.params,
        );
      },
    },),

    it({
      name: 'generated documents parse and round-trip byte-identically in splice mode',
      timeout: RUN.timeout,
      fn: async () => {
        await assert(
          asyncProperty(documentArbitrary, async function check(source,) {
            /**
             Splice-mode re-emission of the parsed document.
             */
            const text = tomlStringify({ edit: parseTomlEdit({ source, },), },);
            expect(text,).toBe(source,);
          },),
          RUN.params,
        );
      },
    },),

    //endregion Compound values and documents

    //region Oracle sanity

    it({
      name: 'semanticEquals captures TOML value identity and difference',
      timeout: RUN.timeout,
      fn: async () => {
        /**
         Pairs that must compare equal under TOML semantics.
         */
        const equalPairs: readonly (readonly [string, string])[] = [
          ['a = 0x10\n', 'a = 16\n',],
          ['a = nan\n', 'a = -nan\n',],
          ['a = 1\nb = 2\n', 'b = 2\na = 1\n',],
          ['a = -0.0\n', 'a = 0.0\n',],
        ];
        for (const [left, right,] of equalPairs) {
          expect(
            semanticEquals({
              left: semanticModel({ source: left, },),
              right: semanticModel({ source: right, },),
            },),
          ).toBe(true,);
        }
        /**
         Pairs that must compare unequal.
         */
        const unequalPairs: readonly (readonly [string, string])[] = [
          ['a = inf\n', 'a = -inf\n',],
          ['a = 1\n', 'a = 2\n',],
          ['a = "x"\n', 'a = "y"\n',],
        ];
        for (const [left, right,] of unequalPairs) {
          expect(
            semanticEquals({
              left: semanticModel({ source: left, },),
              right: semanticModel({ source: right, },),
            },),
          ).toBe(false,);
        }
      },
    },),

    //endregion Oracle sanity
  ],
},);
