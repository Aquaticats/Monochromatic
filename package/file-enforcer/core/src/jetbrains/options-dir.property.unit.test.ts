/**
 Property-based fuzz tests for the JetBrains version helpers in
 `./options-dir.ts`.
 
 Properties: `compareVersionParts` is a consistent total preorder
 (reflexive, sign-antisymmetric, transitive) under its zero-padding rule;
 `parseVersionParts` is total over arbitrary product names and prefixes,
 yielding either a tuple of non-negative integers or the
 not-a-product sentinel; and a product name built from a prefix and a
 dotted numeric version parses back to that exact tuple.
 
 Run plan and seed policy: see `../fuzz-budget-fixture.ts`.
 
 @module
 */

import {
  array,
  assert,
  asyncProperty,
  constantFrom,
  nat,
  record,
  string,
} from 'fast-check';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { fuzzRunPlan, } from '../fuzz-budget-fixture.ts';
import {
  compareVersionParts,
  NOT_A_MATCHING_PRODUCT,
  parseVersionParts,
} from '../../dist/final/node/index.mjs';

//region Constants and arbitraries

/**
 Run plan resolved once for every property in this file.
 */
const RUN = fuzzRunPlan();

/**
 Arbitrary version tuple of non-negative integers.
 */
const versionTupleArbitrary = array(
  nat({ max: 100_000, },),
  { maxLength: 5, },
);

/**
 Arbitrary product-name prefix of letters.
 */
const prefixArbitrary = string({
  minLength: 1,
  maxLength: 8,
  unit: constantFrom(
    'I',
    'd',
    'e',
    'a',
    'P',
    'y',
  ),
},);

/**
 Arbitrary non-empty dotted numeric version, paired as numbers and text.
 */
const numericVersionArbitrary = array(
  nat({ max: 9_999, },),
  {
    minLength: 1,
    maxLength: 4,
  },
);

//endregion Constants and arbitraries

await describe({
  name: '',
  children: [
    //region compareVersionParts

    describe({
      name: compareVersionParts.name,
      children: [
        it({
          name: 'is reflexive and sign-antisymmetric',
          timeout: RUN.timeout,
          fn: async () => {
            await assert(
              asyncProperty(
                record({
                  left: versionTupleArbitrary,
                  right: versionTupleArbitrary,
                },),
                async function reflexiveAntisymmetric({
                  left,
                  right,
                },) {
                  expect(compareVersionParts({
                    left,
                    right: left,
                  },),).toBe(0,);
                  expect(
                    Math.sign(compareVersionParts({
                      left,
                      right,
                    },),),
                  ).toBe(
                    -Math.sign(compareVersionParts({
                      left: right,
                      right: left,
                    },),),
                  );
                },
              ),
              RUN.params,
            );
          },
        },),

        it({
          name: 'is transitive across three tuples',
          timeout: RUN.timeout,
          fn: async () => {
            await assert(
              asyncProperty(
                record({
                  a: versionTupleArbitrary,
                  b: versionTupleArbitrary,
                  c: versionTupleArbitrary,
                },),
                async function transitive({
                  a,
                  b,
                  c,
                },) {
                  /**
                   Sign of comparing a with b.
                   */
                  const ab = Math.sign(compareVersionParts({
                    left: a,
                    right: b,
                  },),);
                  /**
                   Sign of comparing b with c.
                   */
                  const bc = Math.sign(compareVersionParts({
                    left: b,
                    right: c,
                  },),);
                  if ((ab <= 0) && (bc <= 0)) {
                    expect(
                      Math.sign(compareVersionParts({
                        left: a,
                        right: c,
                      },),),
                    ).toBeLessThanOrEqual(0,);
                  }
                  if ((ab >= 0) && (bc >= 0)) {
                    expect(
                      Math.sign(compareVersionParts({
                        left: a,
                        right: c,
                      },),),
                    ).toBeGreaterThanOrEqual(0,);
                  }
                },
              ),
              RUN.params,
            );
          },
        },),
      ],
    },),

    //endregion compareVersionParts

    //region parseVersionParts

    describe({
      name: parseVersionParts.name,
      children: [
        it({
          name: 'yields a non-negative integer tuple or the sentinel, never throwing',
          timeout: RUN.timeout,
          fn: async () => {
            await assert(
              asyncProperty(
                record({
                  productName: string(),
                  prefixes: array(string(), { maxLength: 4, },),
                },),
                async function parseTotality({
                  productName,
                  prefixes,
                },) {
                  /**
                   Parsed version tuple, or the not-a-product sentinel.
                   */
                  const parsed = parseVersionParts({
                    productName,
                    prefixes,
                  },);
                  if (parsed === NOT_A_MATCHING_PRODUCT) return;
                  expect(Array.isArray(parsed,),).toBe(true,);
                  parsed.forEach(function nonNegativeInteger(part,) {
                    expect(Number.isInteger(part,),).toBe(true,);
                    expect(part,).toBeGreaterThanOrEqual(0,);
                  },);
                },
              ),
              RUN.params,
            );
          },
        },),

        it({
          name: 'parses a prefixed dotted numeric version back to its tuple',
          timeout: RUN.timeout,
          fn: async () => {
            await assert(
              asyncProperty(
                record({
                  prefix: prefixArbitrary,
                  parts: numericVersionArbitrary,
                },),
                async function parsesConstructed({
                  prefix,
                  parts,
                },) {
                  /**
                   Product directory name built from the prefix and version.
                   */
                  const productName = `${prefix}${parts.map(String,).join('.',)}`;
                  expect(
                    parseVersionParts({
                      productName,
                      prefixes: [prefix,],
                    },),
                  ).toEqual(parts,);
                },
              ),
              RUN.params,
            );
          },
        },),
      ],
    },),

    //endregion parseVersionParts
  ],
},);
