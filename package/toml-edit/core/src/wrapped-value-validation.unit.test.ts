/**
 Rejections for tagged wrapper payloads that cannot be represented as the TOML
 kind they claim.

 A wrapper payload used to reach the emitted document as raw text, so each
 rejection below also proves the document keeps its original text instead of
 gaining an injected key, table, comment, or quote.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  emptyTomlEdit,
  parseTomlEdit,
  tomlFloat,
  tomlInteger,
  tomlLocalDate,
  tomlLocalDateTime,
  tomlLocalTime,
  tomlSet,
  tomlStringify,
  TomlTypeError,
} from '@monochromatic-dev/toml-edit-core';

/**
 Document every rejection runs against, so the test can prove it survives
 untouched.
 */
const BASE_SOURCE = 'kept = 1\n';

/**
 Key the wrapped value is written to.
 */
const KEY = 'k';

/**
 Wrapper-shaped values that must be rejected, one per kind and failure mode.
 */
const REJECTED_VALUES = [
  {
    name: 'integer payload that would inject a key line',
    tomlKind: 'integer',
    value: '1\nbad = "x"',
  },
  {
    name: 'integer payload carrying a trailing newline',
    tomlKind: 'integer',
    value: '1\n',
  },
  {
    name: 'integer payload carrying a quote',
    tomlKind: 'integer',
    value: '1"',
  },
  {
    name: 'integer payload of the wrong type (string)',
    tomlKind: 'integer',
    value: '1',
  },
  {
    name: 'integer payload that is not integral',
    tomlKind: 'integer',
    value: 1.5,
  },
  {
    name: 'integer payload beyond the exact integer range',
    tomlKind: 'integer',
    value: 1e21,
  },
  {
    name: 'float payload that would inject a table header',
    tomlKind: 'float',
    value: '1.5\n[injected]',
  },
  {
    name: 'float payload carrying a trailing newline',
    tomlKind: 'float',
    value: '1.5\n',
  },
  {
    name: 'float payload carrying a quote',
    tomlKind: 'float',
    value: '1.5"',
  },
  {
    name: 'float payload of the wrong type (string)',
    tomlKind: 'float',
    value: '1.5',
  },
  {
    name: 'float payload of the wrong type (bigint)',
    tomlKind: 'float',
    value: 1n,
  },
  {
    name: 'local-date payload that would inject a key line',
    tomlKind: 'local-date',
    value: '2026-01-01\nbad = "x"',
  },
  {
    name: 'local-date payload that would append a comment',
    tomlKind: 'local-date',
    value: '2026-01-01 # x',
  },
  {
    name: 'local-date payload carrying a trailing newline',
    tomlKind: 'local-date',
    value: '2026-01-01\n',
  },
  {
    name: 'local-date payload carrying a quote',
    tomlKind: 'local-date',
    value: '2026-01-01"',
  },
  {
    name: 'local-date payload of the wrong type (number)',
    tomlKind: 'local-date',
    value: 20_260_101,
  },
  {
    name: 'local-date payload of the wrong type (Date)',
    tomlKind: 'local-date',
    value: new Date('2026-05-14T00:00:00.000Z',),
  },
  {
    name: 'local-date payload naming an impossible month',
    tomlKind: 'local-date',
    value: '2026-13-01',
  },
  {
    name: 'local-date payload naming an impossible day',
    tomlKind: 'local-date',
    value: '2026-02-30',
  },
  {
    name: 'local-date-time payload that would inject a key line',
    tomlKind: 'local-date-time',
    value: '2026-05-14T10:00:00\nbad = "x"',
  },
  {
    name: 'local-date-time payload that would append a comment',
    tomlKind: 'local-date-time',
    value: '2026-05-14T10:00:00 # x',
  },
  {
    name: 'local-date-time payload carrying a trailing newline',
    tomlKind: 'local-date-time',
    value: '2026-05-14T10:00:00\n',
  },
  {
    name: 'local-date-time payload carrying a quote',
    tomlKind: 'local-date-time',
    value: '2026-05-14T10:00:00"',
  },
  {
    name: 'local-date-time payload of the wrong type (number)',
    tomlKind: 'local-date-time',
    value: 0,
  },
  {
    name: 'local-date-time payload that is really an offset date-time',
    tomlKind: 'local-date-time',
    value: '2026-05-14T10:00:00Z',
  },
  {
    name: 'local-date-time payload missing its time',
    tomlKind: 'local-date-time',
    value: '2026-05-14',
  },
  {
    name: 'local-time payload that would inject a key line',
    tomlKind: 'local-time',
    value: '10:00:00\nbad = "x"',
  },
  {
    name: 'local-time payload that would append a comment',
    tomlKind: 'local-time',
    value: '10:00:00 # x',
  },
  {
    name: 'local-time payload carrying a trailing newline',
    tomlKind: 'local-time',
    value: '10:00:00\n',
  },
  {
    name: 'local-time payload carrying a quote',
    tomlKind: 'local-time',
    value: "10:00:00'",
  },
  {
    name: 'local-time payload of the wrong type (boolean)',
    tomlKind: 'local-time',
    value: true,
  },
  {
    name: 'local-time payload naming an impossible hour',
    tomlKind: 'local-time',
    value: '25:00:00',
  },
  {
    name: 'local-time payload that is really an offset time',
    tomlKind: 'local-time',
    value: '10:00:00Z',
  },
  {
    name: 'payload claiming a kind no wrapper defines',
    tomlKind: 'instant',
    value: '2026-05-14',
  },
] as const;

/**
 Wrapper factories whose payloads must keep reaching the document unchanged.
 */
const ACCEPTED_VALUES = [
  { name: 'integer from a bigint', value: tomlInteger(42n,), text: '42', },
  { name: 'integer from an integral number', value: tomlInteger(42,), text: '42', },
  { name: 'float from a fractional number', value: tomlFloat(1.5,), text: '1.5', },
  { name: 'float from a whole number', value: tomlFloat(1,), text: '1.0', },
  { name: 'float from NaN', value: tomlFloat(Number.NaN,), text: 'nan', },
  { name: 'float from positive infinity', value: tomlFloat(Infinity,), text: 'inf', },
  { name: 'float from negative infinity', value: tomlFloat(-Infinity,), text: '-inf', },
  { name: 'local date', value: tomlLocalDate('2026-05-14',), text: '2026-05-14', },
  {
    name: 'local date-time with a T delimiter',
    value: tomlLocalDateTime('2026-05-14T10:00:00',),
    text: '2026-05-14T10:00:00',
  },
  {
    name: 'local date-time with a space delimiter and fraction',
    value: tomlLocalDateTime('2026-05-14 10:00:00.5',),
    text: '2026-05-14 10:00:00.5',
  },
  {
    name: 'local date-time without seconds (TOML 1.1)',
    value: tomlLocalDateTime('2026-05-14T10:00',),
    text: '2026-05-14T10:00',
  },
  { name: 'leap-day local date', value: tomlLocalDate('2024-02-29',), text: '2024-02-29', },
  { name: 'local time', value: tomlLocalTime('10:00:00',), text: '10:00:00', },
  { name: 'local time with a fraction', value: tomlLocalTime('10:00:00.25',), text: '10:00:00.25', },
  { name: 'leap-second local time', value: tomlLocalTime('10:00:60',), text: '10:00:60', },
] as const;

/**
 Run `call` and return the diagnostic of the `TomlTypeError` it throws.
 
 @param call - Operation expected to reject its wrapped payload.
 
 @returns Message of the thrown {@link TomlTypeError}.
 
 @example
 ```ts
 rejectedMessage(() => tomlSet({ edit, path: ['k',], value: tomlInteger(1.5,), },)); // '…integer…1.5…'
 ```
 */
function rejectedMessage(call: () => unknown,): string {
  try {
    call();
  }
  catch (caught: unknown) {
    expect(caught,)
      .toBeInstanceOf(TomlTypeError,);
    if (Error.isError(caught,))
      return caught.message;
    throw caught;
  }
  throw new Error('expected the wrapped payload to be rejected',);
}

/**
 Render an accepted wrapper through {@link tomlSet} and the serializer.
 
 @param value - Wrapper input to write at {@link KEY}.
 
 @returns Emitted document text.
 
 @example
 ```ts
 emitted({ value: tomlInteger(1,), }); // 'k = 1\n'
 ```
 */
function emitted({ value, }: { readonly value: unknown; },): string {
  return tomlStringify({
    edit: tomlSet({
      edit: emptyTomlEdit(),
      path: [KEY,],
      value,
    },),
  },);
}

await describe({
  name: 'wrapped value validation',
  children: [
    ...REJECTED_VALUES.map(function rejection({
      name,
      tomlKind,
      value,
    },) {
      return it({
        name,
        fn: async () => {
          const base = parseTomlEdit({ source: BASE_SOURCE, },);
          const message = rejectedMessage(function setWrapped() {
            tomlSet({
              edit: base,
              path: [KEY,],
              value: { tomlKind, value, },
            },);
          },);
          expect(message,)
            .toContain(tomlKind,);
          /**
           The rejected call left the document exactly as parsed.
           */
          expect(tomlStringify({ edit: base, },),)
            .toBe(BASE_SOURCE,);
        },
      },);
    },),
    it({
      name: 'renders a rejected payload escaped so the diagnostic stays readable',
      fn: async () => {
        const message = rejectedMessage(function setNewline() {
          tomlSet({
            edit: emptyTomlEdit(),
            path: [KEY,],
            value: { tomlKind: 'local-date', value: '2026-01-01\nbad = "x"', },
          },);
        },);
        expect(message,)
          .toContain(String.raw`2026-01-01\nbad`,);
        expect(message.includes('\n',),)
          .toBe(false,);
      },
    },),
    it({
      name: 'treats a tomlKind without a value as an ordinary inline table',
      fn: async () => {
        expect(emitted({ value: { tomlKind: 'integer', }, },),)
          .toBe('k = { tomlKind = "integer", }\n',);
      },
    },),
    ...ACCEPTED_VALUES.map(function acceptance({
      name,
      value,
      text,
    },) {
      return it({
        name,
        fn: async () => {
          expect(emitted({ value, },),)
            .toBe(`${KEY} = ${text}\n`,);
        },
      },);
    },),
  ],
},);
