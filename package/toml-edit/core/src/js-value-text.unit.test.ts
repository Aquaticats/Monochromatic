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
  TomlTypeError,
} from '@monochromatic-dev/toml-edit-core';

/**
 Canonical options for direct built-artifact encoder calls.
 */
const OPTIONS = emptyTomlEdit().canonical;

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
