/**
 Tests for cross-path effective-value projection: queries at a path
 that intersects pending state get the post-edit JS value via
 longest-prefix-first walking plus sub-tree synthesis.
 
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
} from '@monochromatic-dev/toml-edit-core';

await describe({
  name: 'cross-path effective-value resolution',
  children: [
    it({
      name: 'reads a sub-path after inline-table Case C extension',
      fn: async () => {
        const e0 = parseTomlEdit({ source: 'foo = {}\n', },);
        const e1 = tomlSet({ edit: e0, path: ['foo', 'x',], value: 1, },);
        expect(tomlGetValue({ edit: e1, path: ['foo', 'x',], },),).toBe(1,);
      },
    },),

    it({
      name: 'reads both new and existing entries after inline-table extension',
      fn: async () => {
        const e0 = parseTomlEdit({ source: 'foo = { a = 1 }\n', },);
        const e1 = tomlSet({ edit: e0, path: ['foo', 'b',], value: 2, },);
        expect(tomlGetValue({ edit: e1, path: ['foo', 'a',], },),).toBe(1,);
        expect(tomlGetValue({ edit: e1, path: ['foo', 'b',], },),).toBe(2,);
      },
    },),

    it({
      name: 'reads intermediate object after deep top-level path-create',
      fn: async () => {
        const e0 = emptyTomlEdit();
        const e1 = tomlSet({ edit: e0, path: ['a', 'b', 'c',], value: 42, },);
        expect(tomlGetValue({ edit: e1, path: ['a', 'b', 'c',], },),).toBe(42,);
        expect(tomlGetValue({ edit: e1, path: ['a', 'b',], },),).toEqual({ c: 42, },);
      },
    },),

    it({
      name: 'most-specific pending edit wins over an ancestor edit',
      fn: async () => {
        const e0 = parseTomlEdit({ source: 'arr = [10, 20, 30]\n', },);
        const e1 = tomlSet({ edit: e0, path: ['arr',], value: [100, 200, 300,], },);
        const e2 = tomlSet({ edit: e1, path: ['arr', 0,], value: 999, },);
        expect(tomlGetValue({ edit: e2, path: ['arr', 0,], },),).toBe(999,);
      },
    },),

    it({
      name: 'inline-table Case C extension is readable at the parent path',
      fn: async () => {
        const e0 = parseTomlEdit({ source: 'foo = {}\n', },);
        const e1 = tomlSet({ edit: e0, path: ['foo', 'x',], value: 1, },);
        // The edited inline table is now the current value; reads reflect it.
        expect(tomlGetValue({ edit: e1, path: ['foo',], },),).toEqual({ x: 1, },);
      },
    },),

    it({
      name: 'array-element delete: sub-path read returns post-delete value',
      fn: async () => {
        const e0 = parseTomlEdit({ source: 'arr = [10, 20, 30]\n', },);
        const e1 = tomlDelete({ edit: e0, path: ['arr', 1,], },);
        expect(tomlGetValue({ edit: e1, path: ['arr', 0,], },),).toBe(10,);
        expect(tomlGetValue({ edit: e1, path: ['arr', 1,], },),).toBe(30,);
      },
    },),
  ],
},);
