/**
 Tests for `tomlGetNode`.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  parseTomlEdit,
  tomlGetNode,
  tomlSet,
  TomlPathNotFoundError,
} from '@monochromatic-dev/toml-edit-core';

await describe({
  name: tomlGetNode.name,
  children: [
    it({
      name: 'returns the value AST node for an existing keyvalue',
      fn: async () => {
        const edit = parseTomlEdit({ source: 'foo = "bar"\n', },);
        const node = tomlGetNode({ edit, path: ['foo',], },);
        if (Array.isArray(node,))
          throw new Error('expected single node',);
        if (('type' in node) && (node.type === 'TOMLValue'))
          expect(node.kind,).toBe('string',);
        else
          throw new Error('expected TOMLValue',);
      },
    },),

    it({
      name: 'returns clean array-of-tables instance and nested value nodes',
      fn: async () => {
        const edit = parseTomlEdit({ source: '[[foo]]\nname="a"\n[[foo]]\nname="b"\n', },);
        const table = tomlGetNode({ edit, path: ['foo', 1,], },);
        if ((!('type' in table)) || (table.type !== 'TOMLTable'))
          throw new Error('Expected selected array-of-tables instance',);
        expect(table.kind,).toBe('array',);
        const name = tomlGetNode({ edit, path: ['foo', 1, 'name',], },);
        if ((!('type' in name)) || (name.type !== 'TOMLValue') || (name.kind !== 'string'))
          throw new Error('Expected nested string value node',);
        expect(name.value,).toBe('b',);
        expect(() => tomlGetNode({ edit, path: ['foo', 2,], },),)
          .toThrow(TomlPathNotFoundError,);
      },
    },),

    it({
      name: 'throws TomlPathNotFoundError for missing path',
      fn: async () => {
        const edit = parseTomlEdit({ source: 'foo = 1\n', },);
        expect(function lookup() {
          tomlGetNode({ edit, path: ['missing',], },);
        },)
          .toThrow(TomlPathNotFoundError,);
      },
    },),

    it({
      name: 'throws for an edited path (no parse-time node after tomlSet)',
      fn: async () => {
        const e0 = parseTomlEdit({ source: 'foo = "old"\nbar = 1\n', },);
        const e1 = tomlSet({ edit: e0, path: ['foo',], value: 'new', },);
        // The edited value is synthetic, so it no longer maps to a parse-time node.
        expect(function lookup() {
          tomlGetNode({ edit: e1, path: ['foo',], },);
        },)
          .toThrow(TomlPathNotFoundError,);
        // An unedited sibling still resolves to its parse-time node.
        const bar = tomlGetNode({ edit: e1, path: ['bar',], },);
        if (Array.isArray(bar,) || (!('type' in bar)) || (bar.type !== 'TOMLValue'))
          throw new Error('expected TOMLValue for the unedited sibling',);
        expect(bar.value,).toBe(1,);
      },
    },),
  ],
},);
