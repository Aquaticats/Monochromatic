import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  _emitContentNode,
  type CanonicalOptionsOverride,
  emptyTomlEdit,
  parseTomlEdit,
  tomlGetNode,
  tomlGetValue,
  TomlImmutableNodeError,
} from '@monochromatic-dev/toml-edit-core';

/**
 Render a parsed value through the built package's unstable emitter seam.

 @param value - TOML value spelling selected for the test.
 @param options - Formatting overrides that distinguish array layouts.
 @param depth - Nesting level used by indentation checks.

 @returns Re-emitted value text.

 @example
 ```ts
 renderProbe({ value: 'true', }); // 'true'
 ```
 */
function renderProbe({
  value,
  options,
  depth = 0,
}: {
  readonly value: string;
  readonly options?: CanonicalOptionsOverride;
  readonly depth?: number;
},): string {
  const node = tomlGetNode({
    edit: parseTomlEdit({ source: `probe = ${value}\n`, },),
    path: ['probe',],
  },);
  if (!('type' in node))
    throw new Error('Expected parsed value node for emitter test',);
  return _emitContentNode({
    node,
    options: { ...emptyTomlEdit().canonical, ...options, },
    depth,
  },);
}

await describe({
  name: _emitContentNode.name,
  children: [
    it({
      name: 'preserves both boolean spellings',
      fn: async () => {
        expect(renderProbe({ value: 'true', },),).toBe('true',);
        expect(renderProbe({ value: 'false', },),).toBe('false',);
      },
    },),
    it({
      name: 'uses the inclusive array element and column limits',
      fn: async () => {
        const value = '[1,2]';
        expect(renderProbe({
          value,
          options: { arrayInlineThreshold: 2, arrayInlineMaxColumns: 9, },
          depth: 1,
        },),).toBe('[ 1, 2, ]',);
        expect(renderProbe({
          value,
          options: { arrayInlineThreshold: 1, arrayInlineMaxColumns: 9, },
          depth: 1,
        },),).toBe('[\n    1,\n    2,\n  ]',);
        expect(renderProbe({
          value,
          options: { arrayInlineThreshold: 2, arrayInlineMaxColumns: 8, },
          depth: 1,
        },),).toBe('[\n    1,\n    2,\n  ]',);
      },
    },),
    it({
      name: 'indents nested multiline arrays by their depth',
      fn: async () => {
        expect(renderProbe({
          value: '[ [1,2], [3] ]',
          options: { arrayInlineThreshold: 1, arrayInlineMaxColumns: 9, },
          depth: 1,
        },),).toBe('[\n    [\n      1,\n      2,\n    ],\n    [ 3, ],\n  ]',);
      },
    },),
    it({
      name: 'keeps dotted keys and nested array depth inside inline tables',
      fn: async () => {
        const emitted = renderProbe({
          value: '{ a.b = 1, xs = [1,2] }',
          options: { arrayInlineThreshold: 1, arrayInlineMaxColumns: 9, },
          depth: 1,
        },);
        expect(emitted,).toBe('{ a.b = 1, xs = [\n      1,\n      2,\n    ], }',);
        expect(tomlGetValue({
          edit: parseTomlEdit({ source: `probe = ${emitted}\n`, },),
          path: ['probe',],
        },),).toEqual({ a: { b: 1, }, xs: [1, 2,], },);
      },
    },),
    it({
      name: 'rejects a parsed table node with a specific diagnostic',
      fn: async () => {
        const node = tomlGetNode({
          edit: parseTomlEdit({ source: '[section]\nx = 1\n', },),
          path: ['section',],
        },);
        if ((!('type' in node)) || (node.type !== 'TOMLTable'))
          throw new Error('Expected parsed table node for emitter test',);
        expect(() => _emitContentNode({
          node,
          options: emptyTomlEdit().canonical,
        },),).toThrow(TomlImmutableNodeError,);
        expect(() => _emitContentNode({
          node,
          options: emptyTomlEdit().canonical,
        },),).toThrow('emitInlineTable: expected TOMLInlineTable, got TOMLTable',);
      },
    },),
    it({
      name: 'retains empty containers and quoted inline-table keys',
      fn: async () => {
        expect(renderProbe({ value: '[]', },),).toBe('[ ]',);
        expect(renderProbe({ value: '{}', },),).toBe('{ }',);
        expect(renderProbe({ value: '{ "a.b" = 1, c = true }', },),).toBe('{ "a.b" = 1, c = true, }',);
      },
    },),
  ],
},);
