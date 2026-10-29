/**
 Property-based fuzz tests for the flat-JSON pipeline in `./json.ts`.
 
 Example-based coverage lives in `json.unit.test.ts`; these properties
 fuzz the wide input surfaces: `parseJsonObject` must never resolve to a
 non-object nor throw a non-`Error`; format/parse must round-trip;
 `mergeFlatJson` must deduplicate array unions; `mergeObjectDefaults`
 must never overwrite an existing key; `omitJsonKey` must drop exactly
 the named key while preserving the order of the rest.
 
 Run plan (bounded vs campaign) and seed policy: see `../fuzz-budget.ts`.
 
 @module
 */

import {
  type Arbitrary,
  array,
  assert,
  asyncProperty,
  boolean,
  constant,
  constantFrom,
  dictionary,
  integer,
  jsonValue,
  oneof,
  record,
  string,
} from 'fast-check';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { fuzzRunPlan, } from '../fuzz-budget.ts';
import {
  formatJsonObject,
  isJsonObject,
  type JsonObject,
  type JsonValue,
  mergeFlatJson,
  mergeObjectDefaults,
  omitJsonKey,
  parseJsonObject,
} from './json.ts';

//region Constants and arbitraries

/**
 Run plan resolved once for every property in this file.
 */
const RUN = fuzzRunPlan();

/**
 Label passed to `parseJsonObject`; surfaced only in its error message.
 */
const LABEL = 'fuzz';

/**
 Arbitrary flat or nested JSON object. fast-check's `JsonValue` is
 structurally the {@link JsonValue} this package models, so the cast only
 renames the type; it injects nothing the model forbids.
 */
const jsonObjectArbitrary = dictionary(
  string(),
  jsonValue(),
) as Arbitrary<JsonObject>;

/**
 Arbitrary scalar (or structured) JSON value used as a `set` override.
 */
const jsonValueArbitrary = oneof(
  string(),
  integer(),
  boolean(),
  constant(null,),
) as Arbitrary<JsonValue>;

/**
 Arbitrary `set` map of overrides keyed by string.
 */
const setArbitrary = dictionary(string(), jsonValueArbitrary,);

/**
 Arbitrary `arrayUnion` map of string-array additions keyed by string.
 */
const arrayUnionArbitrary = dictionary(string(), array(string(),),);

/**
 Arbitrary `defaults` map keyed by string.
 */
const defaultsArbitrary = dictionary(string(), jsonValueArbitrary,);

/**
 Arbitrary content for `parseJsonObject`: arbitrary text (mostly invalid
 JSON) unioned with serialized arbitrary JSON values (valid JSON that is
 sometimes an object, sometimes not).
 */
const parseContentArbitrary = oneof(
  string(),
  jsonValue().map(function serialize(value,): string {
    return JSON.stringify(value,);
  },),
);

/**
 Arbitrary object plus a key to omit, where the key is sometimes one the
 object holds and sometimes absent, so both removal and identity-copy
 paths are exercised.
 */
const omitArbitrary = jsonObjectArbitrary.chain(function withKey(object,) {
  /**
   Keys present in the generated object.
   */
  const keys = Object.keys(object,);
  /**
   Key arbitrary biased toward present keys when any exist.
   */
  const keyArbitrary = keys.length === 0
    ? string()
    : oneof(string(), constantFrom(...keys,),);
  return record({
    object: constant(object,),
    key: keyArbitrary,
  },);
},);

//endregion Constants and arbitraries

await describe({
  name: '',
  children: [
    //region parseJsonObject

    describe({
      name: parseJsonObject.name,
      children: [
        it({
          name: 'either returns a JSON object or throws an Error for arbitrary content',
          timeout: RUN.timeout,
          fn: async () => {
            await assert(
              asyncProperty(
                parseContentArbitrary,
                async function totality(content,) {
                  try {
                    /**
                     Parsed result for valid object content.
                     */
                    const parsed = parseJsonObject({
                      content,
                      label: LABEL,
                    },);
                    expect(isJsonObject(parsed,),).toBe(true,);
                  }
                  catch (caught: unknown) {
                    expect(caught,).toBeInstanceOf(Error,);
                  }
                },
              ),
              RUN.params,
            );
          },
        },),

        it({
          name: 'format then parse round-trips to a byte-identical serialization',
          timeout: RUN.timeout,
          fn: async () => {
            await assert(
              asyncProperty(
                jsonObjectArbitrary,
                async function roundTrips(object,) {
                  /**
                   Canonical serialization of the generated object.
                   */
                  const serialized = formatJsonObject({ value: object, },);
                  /**
                   Object recovered by parsing the serialization.
                   */
                  const parsed = parseJsonObject({
                    content: serialized,
                    label: LABEL,
                  },);
                  expect(formatJsonObject({ value: parsed, },),).toBe(serialized,);
                },
              ),
              RUN.params,
            );
          },
        },),
      ],
    },),

    //endregion parseJsonObject

    //region mergeFlatJson

    describe({
      name: mergeFlatJson.name,
      children: [
        it({
          name: 'array unions are deduplicated and contain every addition',
          timeout: RUN.timeout,
          fn: async () => {
            await assert(
              asyncProperty(
                record({
                  base: jsonObjectArbitrary,
                  set: setArbitrary,
                  arrayUnion: arrayUnionArbitrary,
                },),
                async function dedupesUnions({
                  base,
                  set,
                  arrayUnion,
                },) {
                  /**
                   Merge result under test.
                   */
                  const result = mergeFlatJson({
                    base,
                    set,
                    arrayUnion,
                  },);
                  expect(isJsonObject(result,),).toBe(true,);
                  for (const [key, additions,] of Object.entries(arrayUnion,)) {
                    /**
                     Merged array at this union key.
                     */
                    const merged = result[key];
                    expect(Array.isArray(merged,),).toBe(true,);
                    /**
                     Merged members narrowed to an array.
                     */
                    const members = merged as readonly JsonValue[];
                    expect(new Set(members,).size,).toBe(members.length,);
                    additions.forEach(function present(addition,) {
                      expect(members,).toContain(addition,);
                    },);
                  }
                },
              ),
              RUN.params,
            );
          },
        },),

        it({
          name: 'set overrides apply for keys not also unioned',
          timeout: RUN.timeout,
          fn: async () => {
            await assert(
              asyncProperty(
                record({
                  base: jsonObjectArbitrary,
                  set: setArbitrary,
                  arrayUnion: arrayUnionArbitrary,
                },),
                async function appliesSet({
                  base,
                  set,
                  arrayUnion,
                },) {
                  /**
                   Merge result under test.
                   */
                  const result = mergeFlatJson({
                    base,
                    set,
                    arrayUnion,
                  },);
                  for (const [key, value,] of Object.entries(set,)) {
                    if (Object.hasOwn(arrayUnion, key,)) continue;
                    expect(result[key],).toEqual(value,);
                  }
                },
              ),
              RUN.params,
            );
          },
        },),
      ],
    },),

    //endregion mergeFlatJson

    //region mergeObjectDefaults

    describe({
      name: mergeObjectDefaults.name,
      children: [
        it({
          name: 'never overwrites an existing key and adds every default key',
          timeout: RUN.timeout,
          fn: async () => {
            await assert(
              asyncProperty(
                record({
                  base: jsonObjectArbitrary,
                  defaults: defaultsArbitrary,
                },),
                async function keepsBase({
                  base,
                  defaults,
                },) {
                  /**
                   Merge result under test.
                   */
                  const result = mergeObjectDefaults({
                    base,
                    defaults,
                  },);
                  for (const [key, value,] of Object.entries(base,)) {
                    expect(result[key],).toEqual(value,);
                  }
                  for (const key of Object.keys(defaults,)) {
                    expect(Object.hasOwn(result, key,),).toBe(true,);
                  }
                },
              ),
              RUN.params,
            );
          },
        },),
      ],
    },),

    //endregion mergeObjectDefaults

    //region omitJsonKey

    describe({
      name: omitJsonKey.name,
      children: [
        it({
          name: 'drops exactly the named key and preserves the order of the rest',
          timeout: RUN.timeout,
          fn: async () => {
            await assert(
              asyncProperty(
                omitArbitrary,
                async function omits({
                  object,
                  key,
                },) {
                  /**
                   Copy without the omitted key.
                   */
                  const result = omitJsonKey({
                    object,
                    key,
                  },);
                  expect(Object.hasOwn(result, key,),).toBe(false,);
                  /**
                   Keys expected to survive, in original order.
                   */
                  const survivors = Object.keys(object,).filter(function notOmitted(candidate,) {
                    return candidate !== key;
                  },);
                  expect(Object.keys(result,),).toEqual(survivors,);
                  survivors.forEach(function preserved(survivor,) {
                    expect(result[survivor],).toEqual(object[survivor],);
                  },);
                },
              ),
              RUN.params,
            );
          },
        },),
      ],
    },),

    //endregion omitJsonKey

    //region isJsonObject

    describe({
      name: isJsonObject.name,
      children: [
        it({
          name: 'accepts non-array objects and rejects arrays, null, and primitives',
          timeout: RUN.timeout,
          fn: async () => {
            await assert(
              asyncProperty(
                oneof(
                  jsonObjectArbitrary.map(function asAccepted(value,): { readonly accepted: boolean; readonly value: unknown; } {
                    return {
                      accepted: true,
                      value,
                    };
                  },),
                  oneof(
                    array(jsonValue(),),
                    constant(null,),
                    string(),
                    integer(),
                    boolean(),
                  ).map(function asRejected(value,): { readonly accepted: boolean; readonly value: unknown; } {
                    return {
                      accepted: false,
                      value,
                    };
                  },),
                ),
                async function classifies({
                  accepted,
                  value,
                },) {
                  expect(isJsonObject(value,),).toBe(accepted,);
                },
              ),
              RUN.params,
            );
          },
        },),
      ],
    },),

    //endregion isJsonObject
  ],
},);
