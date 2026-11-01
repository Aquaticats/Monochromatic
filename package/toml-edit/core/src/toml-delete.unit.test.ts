/**
 Tests for `tomlDelete` and its splice-mode behaviour.
 
 The trailing-inline-comment absorption case is the most likely shipping
 bug; it is gated here.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  parseTomlEdit,
  tomlDelete,
  tomlGetValue,
  tomlStringify,
} from '@monochromatic-dev/toml-edit-core';

await describe({
  name: tomlDelete.name,
  children: [
    it({
      name: 'removes a top-level key plus its trailing newline',
      fn: async () => {
        const source = 'foo = 1\nbar = 2\n';
        const e1 = tomlDelete({
          edit: parseTomlEdit({ source, },),
          path: ['foo',],
        },);
        expect(tomlStringify({ edit: e1, },),).toBe('bar = 2\n',);
      },
    },),

    it({
      name: 'absorbs a same-line trailing inline comment',
      fn: async () => {
        const source = 'foo = 1  # comment\nbar = 2\n';
        const e1 = tomlDelete({
          edit: parseTomlEdit({ source, },),
          path: ['foo',],
        },);
        expect(tomlStringify({ edit: e1, },),).toBe('bar = 2\n',);
      },
    },),

    it({
      name: 'preserves a comment line on an unrelated key',
      fn: async () => {
        const source = '# kept\nfoo = 1\nbar = 2\n';
        const e1 = tomlDelete({
          edit: parseTomlEdit({ source, },),
          path: ['bar',],
        },);
        expect(tomlStringify({ edit: e1, },),).toBe('# kept\nfoo = 1\n',);
      },
    },),

    it({
      name: 'is a no-op for a missing path',
      fn: async () => {
        const source = 'foo = 1\n';
        const e0 = parseTomlEdit({ source, },);
        const e1 = tomlDelete({ edit: e0, path: ['missing',], },);
        expect(tomlStringify({ edit: e1, },),).toBe(source,);
      },
    },),

    it({
      name: 'effective read returns undefined for a deleted key',
      fn: async () => {
        const source = 'foo = 1\n';
        const e1 = tomlDelete({
          edit: parseTomlEdit({ source, },),
          path: ['foo',],
        },);
        expect(tomlGetValue({ edit: e1, path: ['foo',], },),).toBe(undefined,);
      },
    },),

    it({
      name: 'immutable: deletion on derived state leaves original intact',
      fn: async () => {
        const source = 'foo = 1\nbar = 2\n';
        const e0 = parseTomlEdit({ source, },);
        const e1 = tomlDelete({ edit: e0, path: ['foo',], },);
        expect(tomlStringify({ edit: e0, },),).toBe(source,);
        expect(tomlStringify({ edit: e1, },),).toBe('bar = 2\n',);
      },
    },),

    // Standard-table and array-of-tables deletion (Limitation 3)

    it({
      name: 'deletes a standard [foo] table including its body',
      fn: async () => {
        const source = '[foo]\nx = 1\n';
        const e1 = tomlDelete({
          edit: parseTomlEdit({ source, },),
          path: ['foo',],
        },);
        expect(tomlStringify({ edit: e1, },),).toBe('',);
      },
    },),

    it({
      name: 'deletes [foo] while leaving sibling [bar] intact',
      fn: async () => {
        const source = '[foo]\nx = 1\n[bar]\ny = 2\n';
        const e1 = tomlDelete({
          edit: parseTomlEdit({ source, },),
          path: ['foo',],
        },);
        expect(tomlStringify({ edit: e1, },),).toBe('[bar]\ny = 2\n',);
      },
    },),

    it({
      name: 'deletes every [[foo]] instance by path ["foo"]',
      fn: async () => {
        const source = '[[foo]]\nname = "a"\n[[foo]]\nname = "b"\n';
        const e1 = tomlDelete({
          edit: parseTomlEdit({ source, },),
          path: ['foo',],
        },);
        expect(tomlStringify({ edit: e1, },),).toBe('',);
      },
    },),

    it({
      name: 'deletes [[foo]] instances while a sibling [bar] survives',
      fn: async () => {
        const source = '[[foo]]\nname = "a"\n[[foo]]\nname = "b"\n[bar]\nz = 3\n';
        const e1 = tomlDelete({
          edit: parseTomlEdit({ source, },),
          path: ['foo',],
        },);
        expect(tomlStringify({ edit: e1, },),).toBe('[bar]\nz = 3\n',);
      },
    },),

    it({
      name: 'deletes every sibling [a.x] / [a.y] sharing path ["a"]',
      fn: async () => {
        const source = '[a.b]\nx = 1\n[a.c]\ny = 2\n';
        const e1 = tomlDelete({
          edit: parseTomlEdit({ source, },),
          path: ['a',],
        },);
        expect(tomlStringify({ edit: e1, },),).toBe('',);
      },
    },),

    it({
      name: 'effective read after AOT delete returns undefined',
      fn: async () => {
        const source = '[[foo]]\nx = 1\n[[foo]]\nx = 2\n';
        const e1 = tomlDelete({
          edit: parseTomlEdit({ source, },),
          path: ['foo',],
        },);
        expect(tomlGetValue({ edit: e1, path: ['foo',], },),).toBe(undefined,);
      },
    },),

    // Array element deletion (Limitation 4)

    it({
      name: 'deletes a middle array element by index',
      fn: async () => {
        const source = 'arr = [10, 20, 30]\n';
        const e1 = tomlDelete({
          edit: parseTomlEdit({ source, },),
          path: ['arr', 1,],
        },);
        expect(tomlStringify({ edit: e1, },),).toBe('arr = [ 10, 30, ]\n',);
      },
    },),

    it({
      name: 'deleting the only element produces an empty array',
      fn: async () => {
        const source = 'arr = [10]\n';
        const e1 = tomlDelete({
          edit: parseTomlEdit({ source, },),
          path: ['arr', 0,],
        },);
        expect(tomlStringify({ edit: e1, },),).toBe('arr = [ ]\n',);
      },
    },),

    it({
      name: 'preserves a trailing inline comment after the array value bytes',
      fn: async () => {
        const source = 'arr = [10, 20]  # tail\n';
        const e1 = tomlDelete({
          edit: parseTomlEdit({ source, },),
          path: ['arr', 0,],
        },);
        expect(tomlStringify({ edit: e1, },),).toBe('arr = [ 20, ]  # tail\n',);
      },
    },),

    it({
      name: 'effective whole-array read after element delete reflects new array',
      fn: async () => {
        const source = 'arr = [10, 20, 30]\n';
        const e1 = tomlDelete({
          edit: parseTomlEdit({ source, },),
          path: ['arr', 1,],
        },);
        expect(tomlGetValue({ edit: e1, path: ['arr',], },),).toEqual([10, 30,],);
      },
    },),

    it({
      name: 'effective sub-path read after element delete reflects new index',
      fn: async () => {
        const source = 'arr = [10, 20, 30]\n';
        const e1 = tomlDelete({
          edit: parseTomlEdit({ source, },),
          path: ['arr', 1,],
        },);
        expect(tomlGetValue({ edit: e1, path: ['arr', 1,], },),).toBe(30,);
      },
    },),

    it({
      name:
        'nested-array element delete re-emits the outer array without the targeted element',
      fn: async () => {
        const source = 'outer = [[1, 2], [3, 4]]\n';
        const e1 = tomlDelete({
          edit: parseTomlEdit({ source, },),
          path: ['outer', 0, 1,],
        },);
        expect(tomlStringify({ edit: e1, },),).toBe('outer = [ [ 1, ], [ 3, 4, ], ]\n',);
      },
    },),

    it({
      name: 'nested-array element delete: leftmost element of the inner array',
      fn: async () => {
        const source = 'outer = [[1, 2], [3, 4]]\n';
        const e1 = tomlDelete({
          edit: parseTomlEdit({ source, },),
          path: ['outer', 0, 0,],
        },);
        expect(tomlStringify({ edit: e1, },),).toBe('outer = [ [ 2, ], [ 3, 4, ], ]\n',);
      },
    },),

    it({
      name: 'nested-array element delete: only-element collapses to empty inner array',
      fn: async () => {
        const source = 'outer = [[1]]\n';
        const e1 = tomlDelete({
          edit: parseTomlEdit({ source, },),
          path: ['outer', 0, 0,],
        },);
        expect(tomlStringify({ edit: e1, },),).toBe('outer = [ [ ], ]\n',);
      },
    },),

    it({
      name: '3-level nested-array element delete',
      fn: async () => {
        const source = 'outer = [[[1, 2], [3]], []]\n';
        const e1 = tomlDelete({
          edit: parseTomlEdit({ source, },),
          path: ['outer', 0, 0, 1,],
        },);
        expect(tomlStringify({ edit: e1, },),).toBe(
          'outer = [ [ [ 1, ], [ 3, ], ], [ ], ]\n',
        );
      },
    },),

    it({
      name: 'nested-array delete removes an inline-table element',
      fn: async () => {
        const source = 'outer = [[{ a = 1 }, { b = 2 }]]\n';
        const e1 = tomlDelete({
          edit: parseTomlEdit({ source, },),
          path: ['outer', 0, 1,],
        },);
        expect(tomlStringify({ edit: e1, },),).toBe('outer = [ [ { a = 1, }, ], ]\n',);
      },
    },),

    it({
      name: 'effective whole-outer-array read after nested delete reflects new shape',
      fn: async () => {
        const source = 'outer = [[1, 2], [3, 4]]\n';
        const e1 = tomlDelete({
          edit: parseTomlEdit({ source, },),
          path: ['outer', 0, 1,],
        },);
        expect(tomlGetValue({ edit: e1, path: ['outer',], },),).toEqual([
          [1,],
          [3, 4,],
        ],);
      },
    },),

    it({
      name:
        'immutable: nested-array element delete on derived state leaves original intact',
      fn: async () => {
        const source = 'outer = [[1, 2], [3, 4]]\n';
        const e0 = parseTomlEdit({ source, },);
        const e1 = tomlDelete({ edit: e0, path: ['outer', 0, 1,], },);
        expect(tomlStringify({ edit: e0, },),).toBe(source,);
        expect(tomlStringify({ edit: e1, },),).toBe('outer = [ [ 1, ], [ 3, 4, ], ]\n',);
      },
    },),

    it({
      name: 'immutable: array-element delete on derived state leaves original intact',
      fn: async () => {
        const source = 'arr = [10, 20, 30]\n';
        const e0 = parseTomlEdit({ source, },);
        const e1 = tomlDelete({ edit: e0, path: ['arr', 1,], },);
        expect(tomlStringify({ edit: e0, },),).toBe(source,);
        expect(tomlStringify({ edit: e1, },),).toBe('arr = [ 10, 30, ]\n',);
      },
    },),
  ],
},);
