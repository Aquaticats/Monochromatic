/**
 Tests for `resolveByPath` and dotted-key handling.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  parseTomlEdit,
  tomlGetValue,
} from '@monochromatic-dev/toml-edit-core';

await describe({
  name: 'resolveByPath via tomlGetValue',
  children: [
    it({
      name: 'dotted-key form resolves under separate path segments',
      fn: async () => {
        const source = 'a.b.c = 1\n';
        const edit = parseTomlEdit({ source, },);
        expect(tomlGetValue({ edit, path: ['a', 'b', 'c',], },),).toBe(1,);
      },
    },),

    it({
      name: 'quoted key with dots resolves under the literal dotted segment',
      fn: async () => {
        const source = '"a.b" = 1\n';
        const edit = parseTomlEdit({ source, },);
        expect(tomlGetValue({ edit, path: ['a.b',], },),).toBe(1,);
        expect(tomlGetValue({ edit, path: ['a', 'b',], },),).toBe(undefined,);
      },
    },),

    it({
      name: 'mixed quoted and bare key segments',
      fn: async () => {
        const source = 'a."b.c".d = 1\n';
        const edit = parseTomlEdit({ source, },);
        expect(tomlGetValue({ edit, path: ['a', 'b.c', 'd',], },),).toBe(1,);
      },
    },),

    it({
      name: 'array-of-tables: index resolves the correct instance',
      fn: async () => {
        const source = '[[fruits]]\nname = "apple"\n\n[[fruits]]\nname = "pear"\n';
        const edit = parseTomlEdit({ source, },);
        expect(tomlGetValue({ edit, path: ['fruits', 0, 'name',], },),).toBe('apple',);
        expect(tomlGetValue({ edit, path: ['fruits', 1, 'name',], },),).toBe('pear',);
      },
    },),

    it({
      name: 'inline table key access',
      fn: async () => {
        const source = 'point = { x = 1, y = 2 }\n';
        const edit = parseTomlEdit({ source, },);
        expect(tomlGetValue({ edit, path: ['point', 'x',], },),).toBe(1,);
        expect(tomlGetValue({ edit, path: ['point', 'y',], },),).toBe(2,);
      },
    },),

    it({
      name: 'array element by index',
      fn: async () => {
        const source = 'arr = [10, 20, 30]\n';
        const edit = parseTomlEdit({ source, },);
        expect(tomlGetValue({ edit, path: ['arr', 0,], },),).toBe(10,);
        expect(tomlGetValue({ edit, path: ['arr', 2,], },),).toBe(30,);
      },
    },),

    it({
      name: 'nested table: [a.b]',
      fn: async () => {
        const source = '[a.b]\nx = 1\n';
        const edit = parseTomlEdit({ source, },);
        expect(tomlGetValue({ edit, path: ['a', 'b', 'x',], },),).toBe(1,);
      },
    },),
  ],
},);
