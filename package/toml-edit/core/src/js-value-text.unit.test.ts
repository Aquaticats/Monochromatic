import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  _jsValueToTomlText,
  emptyTomlEdit,
  parseTomlEdit,
  tomlGetNode,
  tomlGetValue,
  tomlSet,
  tomlStringify,
  TomlTypeError,
} from '@monochromatic-dev/toml-edit-core';

/**
 Canonical options for direct built-artifact encoder calls.
 */
const OPTIONS = emptyTomlEdit().canonical;

/**
 Re-set the `k` key of `source` and emit the resulting document.
 
 @param source - TOML document to parse before the edit.
 
 @param value - New value for the `k` key.
 
 @returns Emitted TOML text.
 */
function reserialize(
  { source, value, }: { readonly source: string; readonly value: unknown; },
): string {
  return tomlStringify({
    edit: tomlSet({ edit: parseTomlEdit({ source, },), path: ['k',], value, },),
  },);
}

await describe({
  name: _jsValueToTomlText.name,
  children: [
    it({
      name: 'emits both boolean values and nonfinite bare numbers',
      fn: async () => {
        expect(_jsValueToTomlText({ input: true, options: OPTIONS, },),).toBe('true',);
        expect(_jsValueToTomlText({ input: false, options: OPTIONS, },),).toBe('false',);
        expect(_jsValueToTomlText({ input: Number.NaN, options: OPTIONS, },),).toBe('nan',);
        expect(_jsValueToTomlText({ input: Infinity, options: OPTIONS, },),).toBe('inf',);
        expect(_jsValueToTomlText({ input: -Infinity, options: OPTIONS, },),).toBe('-inf',);
      },
    },),
    it({
      name: 'writes a bare negative zero as a float so the sign survives a reparse',
      fn: async () => {
        expect(_jsValueToTomlText({ input: -0, options: OPTIONS, },),).toBe('-0.0',);
        /**
         Positive zero keeps its unchanged integer spelling, so the branch covers
         the sign rather than every zero.
         */
        expect(_jsValueToTomlText({ input: 0, options: OPTIONS, },),).toBe('0',);
        const emitted = tomlStringify({
          edit: tomlSet({ edit: emptyTomlEdit(), path: ['k',], value: -0, },),
        },);
        expect(emitted,).toBe('k = -0.0\n',);
        /**
         TOML integers carry no signed zero, so only the float spelling reads
         back as negative zero.
         */
        const reparsed = tomlGetValue({
          edit: parseTomlEdit({ source: emitted, },),
          path: ['k',],
        },);
        expect(Object.is(reparsed, -0,),).toBe(true,);
      },
    },),
    it({
      name: 'rewrites a numerically equal zero when only the sign differs',
      fn: async () => {
        expect(reserialize({ source: 'k = 0\n', value: -0, },),).toBe('k = -0.0\n',);
        expect(reserialize({ source: 'k = 0.00\n', value: -0, },),).toBe('k = -0.0\n',);
        /**
         This package reads a TOML integer `-0` as positive zero, so the value
         still differs and the sign reaches the emitted text as a float.
         */
        expect(reserialize({ source: 'k = -0\n', value: -0, },),).toBe('k = -0.0\n',);
        /**
         A float already spelled with the sign is the same value, so its own
         digits stay.
         */
        expect(reserialize({ source: 'k = -0.00\n', value: -0, },),).toBe('k = -0.00\n',);
        expect(reserialize({ source: 'k = -0.0\n', value: 0, },),).toBe('k = 0\n',);
      },
    },),
    it({
      name: 'writes a bare negative zero as a float inside an array and keeps a signed element',
      fn: async () => {
        expect(tomlStringify({
          edit: tomlSet({ edit: emptyTomlEdit(), path: ['a',], value: [-0, 0,], },),
        },),).toBe('a = [ -0.0, 0, ]\n',);
        /**
         The per-element carrier reaches the same comparison, so an unsigned
         element is rewritten while a signed one keeps its digits.
         */
        expect(tomlStringify({
          edit: tomlSet({
            edit: parseTomlEdit({ source: 'a = [0, -0.00]\n', },),
            path: ['a',],
            value: [-0, -0,],
          },),
        },),).toBe('a = [ -0.0, -0.00, ]\n',);
      },
    },),
    it({
      name: 'formats empty and inline arrays and inline tables',
      fn: async () => {
        expect(_jsValueToTomlText({ input: [], options: OPTIONS, },),).toBe('[ ]',);
        expect(_jsValueToTomlText({ input: [1, 2,], options: OPTIONS, },),).toBe('[ 1, 2, ]',);
        expect(_jsValueToTomlText({ input: {}, options: OPTIONS, },),).toBe('{ }',);
        expect(_jsValueToTomlText({ input: { a: 1, b: 2, }, options: OPTIONS, },),)
          .toBe('{ a = 1, b = 2, }',);
      },
    },),
    it({
      name: 'uses inclusive array layout thresholds and nested indentation',
      fn: async () => {
        const inline = { ...OPTIONS, arrayInlineThreshold: 2, arrayInlineMaxColumns: 9, };
        const multiline = { ...OPTIONS, arrayInlineThreshold: 1, arrayInlineMaxColumns: 9, };
        expect(_jsValueToTomlText({ input: [1, 2,], options: inline, },),)
          .toBe('[ 1, 2, ]',);
        expect(_jsValueToTomlText({ input: [1, 2,], options: multiline, },),)
          .toBe('[\n  1,\n  2,\n]',);
        expect(_jsValueToTomlText({ input: { xs: [1, 2,], }, options: multiline, },),)
          .toBe('{ xs = [\n    1,\n    2,\n  ], }',);
        expect(_jsValueToTomlText({ input: [[1, 2,], [3,],], options: multiline, },),)
          .toBe('[\n  [\n    1,\n    2,\n  ],\n  [ 3, ],\n]',);
      },
    },),
    it({
      name: 'preserves parse-time numeric spelling inside an equal array',
      fn: async () => {
        const edit = parseTomlEdit({ source: 'arr = [0x10, 0x20]\n', },);
        const node = tomlGetNode({ edit, path: ['arr',], },);
        if ((!('type' in node)) || (node.type !== 'TOMLArray'))
          throw new Error('Expected parsed array node',);
        expect(_jsValueToTomlText({
          input: [16, 32,],
          options: OPTIONS,
          existing: { node, },
        },),).toBe('[ 0x10, 0x20, ]',);
      },
    },),
    it({
      name: 'preserves parsed numeric spelling only for an equal value',
      fn: async () => {
        const integer = tomlGetNode({
          edit: parseTomlEdit({ source: 'count = 0x10\n', },),
          path: ['count',],
        },);
        if ((!('type' in integer)) || (integer.type !== 'TOMLValue') || (integer.kind !== 'integer'))
          throw new Error('Expected parsed integer node',);
        expect(_jsValueToTomlText({ input: 16, options: OPTIONS, existing: { node: integer, }, },),)
          .toBe('0x10',);
        expect(_jsValueToTomlText({ input: 17, options: OPTIONS, existing: { node: integer, }, },),)
          .toBe('17',);
        const float = tomlGetNode({
          edit: parseTomlEdit({ source: 'ratio = 1.00\n', },),
          path: ['ratio',],
        },);
        if ((!('type' in float)) || (float.type !== 'TOMLValue') || (float.kind !== 'float'))
          throw new Error('Expected parsed float node',);
        expect(_jsValueToTomlText({ input: 1, options: OPTIONS, existing: { node: float, }, },),)
          .toBe('1.00',);
        /**
         Every `nan` spelling is the same value, so a re-set keeps the text.
         */
        const nan = tomlGetNode({
          edit: parseTomlEdit({ source: 'k = -nan\n', },),
          path: ['k',],
        },);
        if ((!('type' in nan)) || (nan.type !== 'TOMLValue') || (nan.kind !== 'float'))
          throw new Error('Expected parsed float node',);
        expect(_jsValueToTomlText({ input: Number.NaN, options: OPTIONS, existing: { node: nan, }, },),)
          .toBe('-nan',);
      },
    },),
    it({
      name: 'keeps a parsed literal style only when string content is unchanged',
      fn: async () => {
        const node = tomlGetNode({
          edit: parseTomlEdit({ source: "key = 'old'\n", },),
          path: ['key',],
        },);
        if ((!('type' in node)) || (node.type !== 'TOMLValue') || (node.kind !== 'string'))
          throw new Error('Expected parsed string node',);
        expect(_jsValueToTomlText({ input: 'old', options: OPTIONS, existing: { node, }, },),)
          .toBe("'old'",);
        expect(_jsValueToTomlText({ input: 'new', options: OPTIONS, existing: { node, }, },),)
          .toBe('"new"',);
      },
    },),
    it({
      name: 'rejects unsupported JS inputs with a named type error',
      fn: async () => {
        /**
         Unsupported JS symbol supplied as a TOML value.
         */
        const unsupported = Symbol('unsupported symbol passed as TOML value',);
        expect(() => _jsValueToTomlText({ input: unsupported, options: OPTIONS, },),)
          .toThrow(TomlTypeError,);
        expect(() => _jsValueToTomlText({ input: unsupported, options: OPTIONS, },),)
          .toThrow('Cannot encode symbol as TOML',);
      },
    },),
  ],
},);
