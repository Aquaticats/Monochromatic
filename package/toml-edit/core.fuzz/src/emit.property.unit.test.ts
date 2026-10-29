/**
 Emitter and seam properties, exercised through the package's unstable `_`
 exports on the built artifact.
 
 - `_encodeKey`: any key name round-trips to a single key segment, never an
   accidental dotted path.
 - `_jsValueToTomlText`: any JSON value either rejects (null) with
   `TomlTypeError` or re-emits to a semantically equal value.
 - `_emitContentNode`: re-emitting an unchanged value node reparses equal, and
   for non-string scalars the parse-time spelling is byte-preserved. This is
   the load-bearing datetime-spelling check: the `getStaticTOMLValue` round-trip
   oracle is blind to a datetime kind or spelling change, so this property
   asserts the raw spelling directly.
 - `_emitStringValue`: re-emitting a parsed string node preserves its value
   across the escaping boundary.
 - `_emitDocument`: with no edits, output is byte-identical to the source.
 
 Run plan and seed policy: see `../fuzz-budget.ts`.
 
 @module
 */

import {
  assert,
  asyncProperty,
  constantFrom,
  type JsonValue,
  jsonValue,
  oneof,
  string,
} from 'fast-check';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  _emitContentNode,
  _emitStringValue,
  _encodeKey,
  _jsValueToTomlText,
  _emitDocument,
  emptyTomlEdit,
  parseTomlEdit,
  tomlGetNode,
  tomlKeys,
  TomlTypeError,
} from '@monochromatic-dev/toml-edit-core';

import { fuzzRunPlan, } from './fuzz-budget.ts';
import { documentArbitrary, } from './arb-documents.ts';
import {
  semanticEquals,
  semanticModel,
} from './equality.ts';
import { scalarSampleArbitrary, } from './arb-values.ts';
import type { ValueSample, } from './arb-types.ts';
import { basicStringLiteral, } from './escape.ts';

//region Setup

/**
 Run plan resolved once for every property in this file.
 */
const RUN = fuzzRunPlan();

/**
 Canonical options drawn from a fresh empty state, used by the emitters.
 */
const CANONICAL = emptyTomlEdit().canonical;

/**
 Pinned control-character examples for the emitContentNode property: re-emitting
 these once produced raw control bytes that the parser then rejected (the
 emitter escaped only the named control escapes). Kept as permanent regressions.
 */
const CONTROL_EXAMPLES: readonly (readonly [ValueSample])[] = [
  [{
    text: basicStringLiteral({ content: '\u007F', },),
    value: '\u007F',
  },],
  [{
    text: basicStringLiteral({ content: '\u0000', },),
    value: '\u0000',
  },],
  [{
    text: basicStringLiteral({ content: '\u001F', },),
    value: '\u001F',
  },],
];

/**
 Adversarial key-name arbitrary spanning bare, empty, dotted-looking,
 numeric-looking, quote-bearing, and unicode names (no lone surrogates).
 */
const keyNameArbitrary = oneof(
  string({
    unit: constantFrom(
      'a',
      'Z',
      '0',
      '_',
      '-',
      '.',
      ' ',
      '"',
      '\\',
      '#',
      'é',
      '😀',
    ),
    maxLength: 12,
  },),
  constantFrom(
    '',
    'a.b',
    '123',
    '3.14',
    'has space',
    'quote"x',
    'é€',
  ),
);

/**
 Prototype-shadowing key names that cannot faithfully round-trip through the
 external `getStaticTOMLValue` oracle used by {@link projectProbe}: it decodes
 tables into plain objects by assignment, so a `__proto__` key sets the
 prototype instead of an own property (and `constructor` / `prototype` shadow
 inherited members). The package's own reader handles these correctly (see
 value-materialize.ts prototype safety, exercised by the unit tests); they are
 excluded from the round-trip arbitrary only so the oracle comparison stays
 fair, without weakening the assertion.
 */
const PROTOTYPE_KEYS: ReadonlySet<string> = new Set([
  '__proto__',
  'constructor',
  'prototype',
],);

/**
 Recursively drop prototype-shadowing keys from a generated JSON value.
 Structural walk over the JSON tree (objects and arrays).
 
 @returns Same value with any prototype-shadowing keys removed.
 */
function stripPrototypeKeys(value: JsonValue,): JsonValue {
  if (Array.isArray(value,)) {
    return value.map(function eachElement(child,) {
      return stripPrototypeKeys(child,);
    },);
  }
  if ((value !== null) && ((typeof value) === 'object')) {
    return Object.fromEntries(
      Object.entries(value,)
        .filter(function keep([key,],): boolean {
          return !PROTOTYPE_KEYS.has(key,);
        },)
        .map(function recurse([key, child,],): readonly [string, JsonValue,] {
          return [
            key,
            stripPrototypeKeys(child ?? null,),
          ];
        },),
    );
  }
  return value;
}

/**
 JSON arbitrary with prototype-shadowing keys removed, feeding the
 `_jsValueToTomlText` round-trip property.
 */
const protoSafeJsonValue = jsonValue().map(function sanitize(value,) {
  return stripPrototypeKeys(value,);
},);

/**
 Basic-string content arbitrary stressing the escaping boundary.
 */
const stringContentArbitrary = string({
  unit: constantFrom(
    'a',
    '"',
    '\\',
    '\n',
    '\t',
    ' ',
    'é',
    '😀',
  ),
  maxLength: 16,
},);

/**
 Project the `probe` value out of a single-key document.
 
 @returns Native projection of the value bound to `probe`.
 */
function projectProbe(source: string,): unknown {
  /**
   Whole-document projection; `probe` is the only key these properties set.
   */
  const model = semanticModel({ source, },) as Record<string, unknown>;
  return model.probe;
}

/**
 Outcome of {@link encodeOrSkip}: encoded text, or a skip for a rejected input.
 */
type EncodeResult =
  | { readonly skipped: true; }
    | { readonly text: string; };

/**
 Encode `input`, or skip when a null member makes the encoder reject.
 
 Skips only for a `TomlTypeError` (the documented rejection of null and
 similar). Any other throw propagates and fails the property, so an unexpected
 error class still surfaces rather than being treated as a skip.
 
 @returns Encoded text, or a skip marker for a legitimately rejected input.
 */
function encodeOrSkip(input: unknown,): EncodeResult {
  try {
    return {
      text: _jsValueToTomlText({
        input,
        options: CANONICAL,
      },),
    };
  }
  catch (caught: unknown) {
    if (caught instanceof TomlTypeError) return { skipped: true, };
    throw caught;
  }
}

//endregion Setup

await describe({
  name: 'emit seams',
  children: [
    it({
      name: '_encodeKey round-trips any key name to a single decoded segment',
      timeout: RUN.timeout,
      fn: async () => {
        await assert(
          asyncProperty(keyNameArbitrary, async function roundTrip(name,) {
            /**
             Encoded key spelling, bare or quoted.
             */
            const spelling = _encodeKey({ key: name, },);
            /**
             Top-level keys of a document using the encoded spelling.
             */
            const keys = tomlKeys({ edit: parseTomlEdit({ source: `${spelling} = 1\n`, },), },);
            expect(keys.length,).toBe(1,);
            expect(keys[0],).toBe(name,);
          },),
          RUN.params,
        );
      },
    },),

    it({
      // Regression: the from-scratch key encoder (encodeKey) and value encoder
      // (encodeStringWithStyle) emitted control scalars raw, producing invalid
      // TOML the runner rejected (toml-test encoder/key and encoder/string cases).
      // These seams differ from the parsed-node _emitStringValue path above.
      name: 'control scalars escape through _encodeKey and _jsValueToTomlText',
      timeout: RUN.timeout,
      fn: async () => {
        for (const [sample,] of CONTROL_EXAMPLES) {
          /**
           Control scalar drawn from a pinned example.
           */
          const control = sample.value;
          if ((typeof control) !== 'string') continue;
          /**
           Quoted-key round-trip: the encoded key must reparse to the same name.
           */
          const keyName = `a${control}b`;
          const keys = tomlKeys({ edit: parseTomlEdit({ source: `${_encodeKey({ key: keyName, },)} = 1\n`, },), },);
          expect(keys.includes(keyName,),).toBe(true,);
          /**
           Value round-trip: the encoded string must reparse to the same value.
           */
          const result = encodeOrSkip(`x${control}y`,);
          expect('text' in result,).toBe(true,);
          if ('text' in result)
            expect(
              semanticEquals({
                left: projectProbe(`probe = ${result.text}\n`,) as never,
                right: `x${control}y` as never,
              },),
            ).toBe(true,);
        }
      },
    },),

    it({
      name: '_jsValueToTomlText re-emits a JSON value to an equal value or rejects null',
      timeout: RUN.timeout,
      fn: async () => {
        await assert(
          asyncProperty(protoSafeJsonValue, async function roundTrip(input,) {
            /**
             Encoded text, or a skip marker when a null member is rejected.
             */
            const result = encodeOrSkip(input,);
            if ('skipped' in result) return;
            expect(
              semanticEquals({
                left: projectProbe(`probe = ${result.text}\n`,) as never,
                right: input as never,
              },),
            ).toBe(true,);
          },),
          RUN.params,
        );
      },
    },),

    it({
      name: '_emitContentNode reparses to the same value kind (catches datetime/number kind drift)',
      timeout: RUN.timeout,
      fn: async () => {
        await assert(
          asyncProperty(scalarSampleArbitrary, async function preserve(sample,) {
            /**
             Parse-time value node for the generated scalar.
             */
            const node = tomlGetNode({
              edit: parseTomlEdit({ source: `probe = ${sample.text}\n`, },),
              path: ['probe',],
            },);
            if ((!('type' in node)) || (node.type !== 'TOMLValue')) return;
            /**
             Canonical re-emission of the unchanged node.
             */
            const emitted = _emitContentNode({
              node,
              options: CANONICAL,
            },);
            /**
             Re-parsed node, whose kind must match the original; a structure or
             kind change (local-date to offset-datetime, integer to float) is a
             silent corruption the value oracle alone would not catch.
             */
            const reNode = tomlGetNode({
              edit: parseTomlEdit({ source: `probe = ${emitted}\n`, },),
              path: ['probe',],
            },);
            expect('type' in reNode,).toBe(true,);
            if ('type' in reNode) {
              expect(reNode.type,).toBe('TOMLValue',);
              if (reNode.type === 'TOMLValue') expect(reNode.kind,).toBe(node.kind,);
            }
            if (sample.value !== undefined) {
              expect(
                semanticEquals({
                  left: projectProbe(`probe = ${emitted}\n`,) as never,
                  right: sample.value,
                },),
              ).toBe(true,);
            }
          },),
          {
            ...RUN.params,
            examples: [...CONTROL_EXAMPLES,],
          },
        );
      },
    },),

    it({
      name: '_emitContentNode preserves nested array and inline-table contents',
      fn: async () => {
        for (const value of [
          '[ 1, 2, ]',
          '[ [ 1, 2, ], [ 3, ], ]',
          '[ { x = 1 }, { x = 2 } ]',
          '[\n  1,\n  2,\n]',
        ]) {
          const source = `probe = ${value}\n`;
          const node = tomlGetNode({
            edit: parseTomlEdit({ source, },),
            path: ['probe',],
          },);
          if ((!('type' in node)) || (node.type !== 'TOMLArray'))
            throw new Error('Expected array node from array-valued probe',);
          const emitted = _emitContentNode({ node, options: CANONICAL, },);
          expect(semanticEquals({
            left: semanticModel({ source, },),
            right: semanticModel({ source: `probe = ${emitted}\n`, },),
          },),).toBe(true,);
        }
      },
    },),

    it({
      name: '_emitStringValue re-emits a parsed string to the same value',
      timeout: RUN.timeout,
      fn: async () => {
        await assert(
          asyncProperty(stringContentArbitrary, async function reemit(content,) {
            /**
             Parse-time string node for a basic-string literal of `content`.
             */
            const node = tomlGetNode({
              edit: parseTomlEdit({ source: `probe = ${basicStringLiteral({ content, },)}\n`, },),
              path: ['probe',],
            },);
            if ((!('type' in node)) || (node.type !== 'TOMLValue') || (node.kind !== 'string')) return;
            expect(projectProbe(`probe = ${_emitStringValue({ node, },)}\n`,),).toBe(content,);
          },),
          RUN.params,
        );
      },
    },),

    it({
      name: '_emitDocument with no edits is byte-identical to the source',
      timeout: RUN.timeout,
      fn: async () => {
        await assert(
          asyncProperty(documentArbitrary, async function identity(source,) {
            expect(_emitDocument({ edit: parseTomlEdit({ source, },), },),).toBe(source,);
          },),
          RUN.params,
        );
      },
    },),
  ],
},);
