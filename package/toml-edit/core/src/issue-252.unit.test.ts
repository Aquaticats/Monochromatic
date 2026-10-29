/**
 Regression tests for issue #252: the delta-accumulation defects that the
 document-tree redesign resolves by construction.
 
 Each case exercises reads and serialization on the same un-reparsed state, so
 they lock in that one always-current tree keeps the two consistent.
 
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
  tomlDelete,
  tomlGetValue,
  tomlSet,
  tomlStringify,
  TomlTypeError,
} from '@monochromatic-dev/toml-edit-core';

await describe({
  name: 'issue #252 delta-accumulation regressions',
  children: [
    it({
      name: 'repeated path-create set replaces, not duplicates, the key',
      fn: async () => {
        const e0 = emptyTomlEdit();
        const e1 = tomlSet({ edit: e0, path: ['a',], value: 1, },);
        const e2 = tomlSet({ edit: e1, path: ['a',], value: 2, },);
        expect(tomlStringify({ edit: e2, },),).toBe('a = 2\n',);
        expect(tomlGetValue({ edit: e2, path: ['a',], },),).toBe(2,);
      },
    },),

    it({
      name: 'repeated in-table path-create set replaces the key',
      fn: async () => {
        const base = parseTomlEdit({ source: '[t]\n', },);
        const e1 = tomlSet({ edit: base, path: ['t', 'a',], value: 1, },);
        const e2 = tomlSet({ edit: e1, path: ['t', 'a',], value: 2, },);
        expect(tomlStringify({ edit: e2, },),).toBe('[t]\na = 2\n',);
      },
    },),

    it({
      name: 'implicit dotted-key parent reads as an object',
      fn: async () => {
        const edit = parseTomlEdit({ source: 'a.x = []\n', },);
        expect(tomlGetValue({ edit, path: ['a',], },),).toEqual({ x: [], },);
      },
    },),

    it({
      name: 'delete of an implicit dotted-key parent removes the bytes',
      fn: async () => {
        const edit = parseTomlEdit({ source: 'a.x = []\n', },);
        const deleted = tomlDelete({ edit, path: ['a',], },);
        expect(tomlStringify({ edit: deleted, },),).toBe('',);
        expect(tomlGetValue({ edit: deleted, path: ['a',], },),).toBe(undefined,);
      },
    },),

    it({
      name: 'delete of an implicit parent removes every dotted key under it',
      fn: async () => {
        const edit = parseTomlEdit({ source: 'a.x = 1\na.y = 2\nb = 3\n', },);
        const deleted = tomlDelete({ edit, path: ['a',], },);
        expect(tomlStringify({ edit: deleted, },),).toBe('b = 3\n',);
      },
    },),

    it({
      name: 'set a scalar over an implicit parent throws TomlTypeError',
      fn: async () => {
        const edit = parseTomlEdit({ source: 'a.x = 1\n', },);
        expect(function set() {
          tomlSet({ edit, path: ['a',], value: 5, },);
        },)
          .toThrow(TomlTypeError,);
      },
    },),

    it({
      name: 'set an object over an implicit parent replaces its entries',
      fn: async () => {
        const edit = parseTomlEdit({ source: 'a.x = 1\n', },);
        const e1 = tomlSet({ edit, path: ['a',], value: { y: 2, }, },);
        expect(tomlGetValue({ edit: e1, path: ['a',], },),).toEqual({ y: 2, },);
        expect(tomlStringify({ edit: e1, },),).not.toContain('a.x',);
      },
    },),

    it({
      name: 'delete of a mixed implicit parent removes dotted key and sub-table',
      fn: async () => {
        const edit = parseTomlEdit({ source: 'a.x = 1\n[a.b]\nc = 2\n', },);
        const deleted = tomlDelete({ edit, path: ['a',], },);
        expect(tomlStringify({ edit: deleted, },),).toBe('',);
        expect(tomlGetValue({ edit: deleted, path: ['a',], },),).toBe(undefined,);
      },
    },),

    it({
      name: 'delete into a prior edit is honored (no reparse)',
      fn: async () => {
        const base = parseTomlEdit({ source: 'foo = { x = 1, y = 2 }\n', },);
        const deleted = tomlDelete({ edit: base, path: ['foo', 'x',], },);
        expect(tomlGetValue({ edit: deleted, path: ['foo',], },),).toEqual({ y: 2, },);
      },
    },),

    it({
      name: 'repeated whole-table-replace does not accumulate (no reparse)',
      fn: async () => {
        const base = parseTomlEdit({ source: '[foo]\nx = 1\n', },);
        const e1 = tomlSet({ edit: base, path: ['foo',], value: { a: 1, }, },);
        const e2 = tomlSet({ edit: e1, path: ['foo',], value: { b: 2, }, },);
        expect(tomlGetValue({ edit: e2, path: ['foo',], },),).toEqual({ b: 2, },);
        /**
         Neither the original body key nor the intermediate set may survive.
         */
        const out = tomlStringify({ edit: e2, },);
        expect(out.includes('x = 1',) || out.includes('a = 1',),).toBe(false,);
      },
    },),
  ],
},);
