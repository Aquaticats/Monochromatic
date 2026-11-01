/**
 Curated JSONC conformance corpus against VS Code JSONC semantics: JSON plus
 `//` and block comments and trailing commas, but not JSON5 (no single quotes,
 unquoted keys, or hex). Valid cases parse to an expected value; invalid cases
 throw.

 The shared cases live in `fixtures/jsonc-conformance.json`, which the Rust crate
 `monochromatic-jsonc-edit` reads too, so both maintained implementations answer
 the same supported-behavior contract. A task in each package fails when the two
 copies drift.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import type { StringJsonc, } from '@monochromatic-dev/jsonc-edit-core-ts/ts/brand.ts';
import {
  COMMENT_ABSENT,
  type JsoncEditState,
  jsoncGetComment,
  jsoncGetKeyComment,
  jsoncGetValue,
  jsoncKeys,
  jsoncSet,
  jsoncStringify,
  parseJsonc,
  parseJsoncEdit,
} from '@monochromatic-dev/jsonc-edit-core-ts/ts';
import type { JsonValue, } from 'type-fest';

import fixtures from '../fixtures/jsonc-conformance.json' with { type: 'json', };

const asJsonc = (source: string,): StringJsonc => source as StringJsonc;

type ValidCase = {
  readonly input: string;
  readonly value: JsonValue;
};

const validCases: readonly ValidCase[] = [
  { input: '{}', value: {}, },
  { input: '[]', value: [], },
  { input: '{ "a": 1, }', value: { a: 1, }, },
  { input: '[1, 2, 3,]', value: [1, 2, 3,], },
  { input: '{ "a": 1 } // trailing line comment', value: { a: 1, }, },
  { input: '// leading line comment\n{ "a": 1 }', value: { a: 1, }, },
  { input: '{ /* block */ "a": 1 }', value: { a: 1, }, },
  { input: '{\n  "a": 1, // inline\n  "b": 2\n}', value: { a: 1, b: 2, }, },
  {
    input: String.raw`{ "n": -1.5e3, "s": "x\u0041", "b": true, "z": null }`,
    value: {
      n: -1_500,
      s: 'xA',
      b: true,
      z: null,
    },
  },
  {
    input: '{ "nested": { "deep": [true, false, null] } }',
    value: { nested: { deep: [true, false, null,], }, },
  },
];

const invalidCases: readonly string[] = [
  '{ \'a\': 1 }',
  '{ a: 1 }',
  '42',
  '"bare string"',
  '{ "a": 1 ',
  '{ "a": 1 "b": 2 }',
  '{ "a": }',
  '[1 2]',
  '{ "a": 0x1F }',
  '{ "a": 1, , }',
  '{ /* open comment\n"a": 1 }',
];

/**
 One shared fixture case that must parse, with the shape both implementations agree on.
 */
type SharedValidCase = {
  readonly name: string;
  readonly source: string;
  readonly rootKind: string;
  readonly members: number;
};

/**
 What:     Marks a fixture expectation of "no comment here".
 Why:      The repo models absence with a sentinel rather than a nullish value, so the JSON null the
           fixture file carries is decoded into this symbol before any assertion compares it.
 */
const FIXTURE_ABSENT: unique symbol = Symbol('shared fixture expects no comment');

/**
 What:     A fixture comment expectation after decoding: text, or the absence sentinel.
 Why:      Keeps assertions null-free while still expressing "no comment".
 */
type FixtureComment = string | typeof FIXTURE_ABSENT;

/* oxlint-disable no-restricted-syntax/no-nullish-union -- the shared fixture is an external JSON document, fixtures/jsonc-conformance.json, which spells "no comment" as null and is read identically by the Rust crate; these declarations mirror that file rather than model repo state. */
/**
 What:     The JSON shape one fixture comment field has on disk.
 Why:      Naming it once keeps the mirrored nullish form in a single scoped place.
 */
type FixtureCommentJson = string | null;

/**
 One shared fixture address expectation: `null` means "no comment here".
 */
type SharedExpectation = {
  readonly path: readonly (string | number)[];
  readonly key: FixtureCommentJson;
  readonly value: FixtureCommentJson;
};

/**
 One shared fixture case describing where comments must land.
 */
type SharedOwnershipCase = {
  readonly name: string;
  readonly source: string;
  readonly root: FixtureCommentJson;
  readonly at: readonly SharedExpectation[];
};
/* oxlint-enable no-restricted-syntax/no-nullish-union */

/**
 What:     Decode one fixture comment field into text or the absence sentinel.
 Why:      The fixture's absence spelling is JSON null; narrowing by typeof keeps this module's
           comparisons free of nullish literals while still rejecting the wrong field type.
 
 @param value - Fixture field read from JSON.

 @returns The comment text, or FIXTURE_ABSENT when the fixture expects none.
 */
const decodeComment = (value: FixtureCommentJson,): FixtureComment => {
  if ((typeof value) === 'string')
    return value;
  if (value === null)
    return FIXTURE_ABSENT;
  throw new TypeError(`fixture comment expectation must be a string or null, got ${JSON.stringify(value)}`);
};

/**
 What:     Decode one comment the API returned into the same vocabulary as the fixture.
 Why:      Both sides then compare as sentinel-or-text, with no nullish value involved.
 
 @param comment - Comment query result, which is COMMENT_ABSENT when nothing is attached.

 @returns The comment text, or FIXTURE_ABSENT when the query found none.
 */
const actualComment = (comment: { type: string; text: string } | symbol,): FixtureComment =>
  (comment === COMMENT_ABSENT)
    ? FIXTURE_ABSENT
    : (comment as { text: string }).text;

/**
 What:     One shared round-trip case: a source plus the comment bodies that must still be attached
 after emission and a reparse.
 */
type SharedRoundTripCase = {
  readonly name: string;
  readonly source: string;
  readonly commentBodies: readonly string[];
};

/**
 What:     One shared root-shape case: an address, a replacement kind and whether the edit is refused.
 */
type SharedRootShapeCase = {
  readonly name: string;
  readonly path: readonly (string | number)[];
  readonly valueKind: string;
  readonly refused: boolean;
};

/**
 What:     Collect every comment body attached anywhere in a document.
 Why:      The round-trip contract is stated as bodies that must still be attached afterwards, which
 needs one walk over keys, values and the root rather than a query per address.

 @param state - Document to walk.
 @param path - Address the walk is currently at.
 @param into - Accumulator, filled in place so the walk stays tail-shaped.
 */
const collectBodies = (
  state: JsoncEditState,
  path: readonly (string | number)[],
  into: string[],
): void => {
  const valueComment = jsoncGetComment({ state, path: [...path,], },);
  if (valueComment !== COMMENT_ABSENT)
    into.push(valueComment.text,);
  if ((typeof path.at(-1,)) === 'string') {
    const keyComment = jsoncGetKeyComment({ state, path: [...path,], },);
    if (keyComment !== COMMENT_ABSENT)
      into.push(keyComment.text,);
  }
  const value = jsoncGetValue({ state, path: [...path,], },);
  // Scalars and JSON null have no members, and `jsoncKeys` throws rather than returning nothing for
  // them, so the walk stops here. `instanceof Object` is false for null without naming it.
  if (!(value instanceof Object))
    return;
  if (Array.isArray(value,)) {
    value.forEach(function walkElement(_: unknown, index: number,): void {
      collectBodies(state, [...path, index,], into,);
    },);
    return;
  }
  const keys = jsoncKeys({ state, path: [...path,], },);
  if (keys === undefined)
    return;
  for (const key of keys)
    collectBodies(state, [...path, key,], into,);
};

/**
 What:     Build the replacement value a root-shape case names.
 Why:      The contract is about the replacement's shape, so the fixture names a kind instead of
 spelling a literal both suites would have to agree on.

 @param kind - Fixture kind name.

 @returns A value of that shape.
 */
const valueForKind = (kind: string,): JsonValue => {
  if (kind === 'null')
    return null;
  if (kind === 'number')
    return 7;
  if (kind === 'string')
    return '';
  if (kind === 'array')
    return [];
  if (kind === 'record')
    return {};
  throw new Error(`fixture names an unknown value kind ${kind}`);
};

/**
 What:     The outcome of trying an edit that the contract may refuse.
 Why:      A refusal is an expected outcome for some cases, so it travels as a value; the message
 comes along so the case can assert the refusal names the root.
 */
type RefusalProbe = {
  readonly refused: boolean;
  readonly message: string;
};

/**
 What:     Try an edit and report whether the package refused it.
 Why:      Both outcomes are contractual, so neither may be allowed to fail the case by throwing.

 @param attempt - The edit to try.

 @returns Whether it was refused, and the thrown message when it was.
 */
const attemptRefusal = (attempt: () => unknown,): RefusalProbe => {
  try {
    attempt();
    return { refused: false, message: '', };
  } catch (error) {
    return { refused: true, message: (error as Error).message, };
  }
};

await describe({
  name: 'JSONC conformance',
  children: [
    it({
      name: 'valid JSONC documents parse to the expected value',
      fn: async () => {
        for (const sample of validCases) {
          const got = jsoncGetValue({
            state: parseJsoncEdit({ source: asJsonc(sample.input,), },),
            path: [],
          },);
          expect(got,).toEqual(sample.value,);
        }
      },
    },),
    it({
      name: 'invalid or JSON5-only documents throw',
      fn: async () => {
        for (const source of invalidCases)
          expect(() => {
            parseJsonc({ source: asJsonc(source,), },);
          },).toThrow();
      },
    },),
    describe({
      name: 'shared fixtures',
      children: [
        it({
          name: 'every shared valid source parses with the agreed root shape',
          fn: async () => {
            for (const sample of fixtures.valid as readonly SharedValidCase[]) {
              const state = parseJsoncEdit({ source: asJsonc(sample.source,), },);
              const value = jsoncGetValue({ state, path: [], },);
              const isRecord = !Array.isArray(value);
              expect(isRecord ? 'record' : 'array',).toBe(sample.rootKind,);
              const members = isRecord
                ? Object.keys(value as Record<string, unknown>,).length
                : (value as readonly unknown[]).length;
              expect(members,).toBe(sample.members,);
            }
          },
        },),
        it({
          name: 'every shared invalid source throws',
          fn: async () => {
            for (const sample of fixtures.invalid as readonly { name: string; source: string }[])
              expect(() => {
                parseJsonc({ source: asJsonc(sample.source,), },);
              },).toThrow();
          },
        },),
        it({
          name: 'shared comment ownership matches on the root, keys and values',
          fn: async () => {
            for (const sample of fixtures.commentOwnership as readonly SharedOwnershipCase[]) {
              const state = parseJsoncEdit({ source: asJsonc(sample.source,), },);
              expect(
                actualComment(jsoncGetComment({ state, path: [], },),),
              ).toBe(decodeComment(sample.root,),);
              for (const expectation of sample.at) {
                const path = [...expectation.path,];
                expect(
                  actualComment(jsoncGetComment({ state, path, },),),
                ).toBe(decodeComment(expectation.value,),);
                const finalSegment = expectation.path.at(-1,);
                if ((typeof finalSegment) === 'string') {
                  expect(
                    actualComment(jsoncGetKeyComment({ state, path, },),),
                  ).toBe(decodeComment(expectation.key,),);
                }
              }
            }
          },
        },),
        it({
          name: 'shared number spellings survive canonical output',
          fn: async () => {
            for (const sample of fixtures.numberSpelling as readonly {
              name: string;
              source: string;
              emittedContains: readonly string[];
            }[]) {
              const text = jsoncStringify({
                state: parseJsoncEdit({ source: asJsonc(sample.source,), },),
              },);
              for (const needle of sample.emittedContains)
                expect(text.includes(needle,),).toBe(true,);
            }
          },
        },),
        it({
          name: 'shared number equality expectations hold for parsed values',
          fn: async () => {
            for (const sample of fixtures.numberEquality as readonly {
              name: string;
              left: string;
              right: string;
              equal: boolean;
              exactIdentityOnly?: boolean;
            }[]) {
              // TypeScript stores IEEE 754 binary64 values, so a case that needs exact mathematical
              // identity is asserted here only through the preserved source spelling. The Rust crate
              // asserts every case through its exact number identity.
              if (sample.exactIdentityOnly === true)
                continue;
              const state = parseJsoncEdit({
                source: asJsonc(`{"l":${sample.left},"r":${sample.right}}`,),
              },);
              const left = jsoncGetValue({ state, path: ['l',], },);
              const right = jsoncGetValue({ state, path: ['r',], },);
              expect(Object.is(left, right,) || (left === right),).toBe(sample.equal,);
            }
          },
        },),
        it({
          name: 'shared round-trip cases keep every comment body',
          fn: async () => {
            for (const sample of fixtures.roundTrip as readonly SharedRoundTripCase[]) {
              const emitted = jsoncStringify({
                state: parseJsoncEdit({ source: asJsonc(sample.source,), },),
              },);
              const reparsed = parseJsoncEdit({ source: asJsonc(emitted,), },);
              const bodies: string[] = [];
              collectBodies(reparsed, [], bodies,);
              for (const wanted of sample.commentBodies)
                expect(bodies.includes(wanted,),).toBe(true,);
            }
          },
        },),
        it({
          name: 'shared root-shape cases refuse non-container roots',
          fn: async () => {
            for (const sample of fixtures.rootShape as readonly SharedRootShapeCase[]) {
              const state = parseJsoncEdit({ source: asJsonc('{ "a": 1 }',), },);
              const probe = attemptRefusal(() => jsoncSet({
                state,
                path: [...sample.path,],
                value: valueForKind(sample.valueKind,),
              },),);
              expect(probe.refused,).toBe(sample.refused,);
              if (sample.refused)
                expect(probe.message.includes('root',),).toBe(true,);
            }
          },
        },),
        it({
          name: 'shared canonical layout matches byte for byte',
          fn: async () => {
            for (const sample of fixtures.canonicalLayout as readonly {
              name: string;
              source: string;
              emitted: string;
            }[]) {
              const emitted = jsoncStringify({
                state: parseJsoncEdit({ source: asJsonc(sample.source,), },),
              },);
              expect(emitted,).toBe(sample.emitted,);
            }
          },
        },),
        it({
          name: 'the shared nesting boundary accepts and rejects the fixture depths',
          fn: async () => {
            const { accepted, rejected, } = fixtures.depth;
            expect(rejected,).toBe(accepted + 1,);
            const nest = (depth: number,): string => `${'['.repeat(depth,)}0${']'.repeat(depth,)}`;
            expect(parseJsonc({ source: asJsonc(nest(accepted,),), },).kind,).toBe('array',);
            expect(() => {
              parseJsonc({ source: asJsonc(nest(rejected,),), },);
            },).toThrow('nesting too deep',);
          },
        },),
      ],
    },),
  ],
},);
