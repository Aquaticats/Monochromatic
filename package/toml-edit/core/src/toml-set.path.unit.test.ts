import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  parseTomlEdit,
  tomlGetValue,
  tomlSet,
  tomlStringify,
  TomlImmutableNodeError,
} from '@monochromatic-dev/toml-edit-core';

await describe({
  name: 'indexed and nested set paths',
  children: [
    it({
      name: 'updates only the addressed array-of-tables instance',
      fn: async () => {
        const edit = parseTomlEdit({ source: '[[foo]]\nname="a"\n[[foo]]\nname="b"\n', },);
        const updated = tomlSet({ edit, path: ['foo', 1, 'name',], value: 'new', },);
        expect(tomlStringify({ edit: updated, },),).toBe('[[foo]]\nname="a"\n[[foo]]\nname="new"\n',);
        expect(tomlGetValue({ edit: updated, path: ['foo', 0, 'name',], },),).toBe('a',);
        expect(tomlGetValue({ edit: updated, path: ['foo', 1, 'name',], },),).toBe('new',);
      },
    },),
    it({
      name: 'rejects an absent array-of-tables index instead of editing a sibling',
      fn: async () => {
        const edit = parseTomlEdit({ source: '[[foo]]\nname="a"\n[[foo]]\nname="b"\n', },);
        expect(() => tomlSet({ edit, path: ['foo', 2, 'name',], value: 'wrong', },),)
          .toThrow(TomlImmutableNodeError,);
        expect(tomlStringify({ edit, },),).toBe('[[foo]]\nname="a"\n[[foo]]\nname="b"\n',);
      },
    },),
    it({
      name: 'replaces a nested array-of-tables collection in place among siblings',
      fn: async () => {
        const source = 'title = "x"\n[[a.b]]\nx = 1\n[[a.c]]\ny = 2\n[tail]\nz = 3\n';
        const edit = parseTomlEdit({ source, },);
        const updated = tomlSet({ edit, path: ['a', 'b',], value: [{ x: 9, },], },);
        expect(tomlStringify({ edit: updated, },),)
          .toBe('title = "x"\n[[a.b]]\nx = 9\n[[a.c]]\ny = 2\n[tail]\nz = 3\n',);
        expect(tomlGetValue({ edit: updated, path: ['a', 'c', 0, 'y',], },),).toBe(2,);
      },
    },),
    it({
      name: 'keeps a deeper array-of-tables header when replacing its parent collection',
      fn: async () => {
        const edit = parseTomlEdit({
          source: 'title="x"\n[[a.b]]\nx=1\n[[a.b.c]]\ny=2\n',
        },);
        const updated = tomlSet({ edit, path: ['a', 'b',], value: [{ x: 9, },], },);
        expect(tomlStringify({ edit: updated, },),)
          .toBe('title="x"\n[[a.b]]\nx = 9\n[[a.b.c]]\ny=2\n',);
      },
    },),
    it({
      name: 'creates under the deepest matching standard-table header',
      fn: async () => {
        const edit = parseTomlEdit({ source: '[a.b]\nx=1\n[a.c]\nz=2\n', },);
        const updated = tomlSet({ edit, path: ['a', 'b', 'new',], value: 3, },);
        expect(tomlStringify({ edit: updated, },),).toBe('[a.b]\nx=1\nnew = 3\n[a.c]\nz=2\n',);
      },
    },),
    it({
      name: 'prefers a nested standard table over its explicit parent',
      fn: async () => {
        const edit = parseTomlEdit({ source: '[a]\nroot = 1\n[a.b]\nx = 2\n', },);
        const updated = tomlSet({ edit, path: ['a', 'b', 'new',], value: 3, },);
        expect(tomlStringify({ edit: updated, },),).toBe('[a]\nroot = 1\n[a.b]\nx = 2\nnew = 3\n',);
      },
    },),
    it({
      name: 'chooses the deepest table even when its parent appears later',
      fn: async () => {
        const edit = parseTomlEdit({ source: '[a.b]\nx=1\n[a]\nroot=2\n', },);
        const updated = tomlSet({ edit, path: ['a', 'b', 'new',], value: 9, },);
        expect(tomlStringify({ edit: updated, },),).toBe('[a.b]\nx=1\nnew = 9\n[a]\nroot=2\n',);
      },
    },),
    it({
      name: 'replaces a standard table following a top-level key',
      fn: async () => {
        const edit = parseTomlEdit({ source: 'a=1\n[foo]\nx=2\n', },);
        const updated = tomlSet({ edit, path: ['foo',], value: { y: 9, }, },);
        expect(tomlStringify({ edit: updated, },),).toBe('a=1\n[foo]\ny = 9\n',);
      },
    },),
    it({
      name: 'rejects replacing a mixed dotted-key and table parent',
      fn: async () => {
        const edit = parseTomlEdit({ source: 'a.x=1\n[a.b]\nc=2\n', },);
        expect(() => tomlSet({ edit, path: ['a',], value: { y: 9, }, },),)
          .toThrow('tomlSet on the sibling tables at a is not supported',);
      },
    },),
    it({
      name: 'replaces implicit dotted keys before an unrelated table',
      fn: async () => {
        const edit = parseTomlEdit({
          source: 'title="x"\na.x=1\na.y=2\nother=3\n[sibling]\nz=4\n',
        },);
        const updated = tomlSet({ edit, path: ['a',], value: { q: 9, r: 10, }, },);
        expect(tomlStringify({ edit: updated, },),)
          .toBe('title="x"\nother=3\na.q = 9\na.r = 10\n[sibling]\nz=4\n',);
      },
    },),
    it({
      name: 'retains comment filler when replacing an implicit parent',
      fn: async () => {
        const edit = parseTomlEdit({ source: '# header\na.x=1\n', },);
        const updated = tomlSet({ edit, path: ['a',], value: { y: 2, }, },);
        expect(tomlStringify({ edit: updated, },),).toBe('# header\na.y = 2\n',);
      },
    },),
    it({
      name: 'preserves a partial-prefix sibling during implicit replacement',
      fn: async () => {
        const edit = parseTomlEdit({ source: 'a.b.x=1\na.c.y=2\n', },);
        const updated = tomlSet({ edit, path: ['a', 'b',], value: { z: 9, }, },);
        expect(tomlStringify({ edit: updated, },),).toBe('a.c.y=2\na.b.z = 9\n',);
      },
    },),
    it({
      name: 'separates a created key from an unterminated final source line',
      fn: async () => {
        const edit = parseTomlEdit({ source: 'a = 1', },);
        const updated = tomlSet({ edit, path: ['b',], value: 2, },);
        expect(tomlStringify({ edit: updated, },),).toBe('a = 1\nb = 2\n',);
      },
    },),
    it({
      name: 'updates the matching table without changing another table with the same key',
      fn: async () => {
        const edit = parseTomlEdit({ source: '[a]\nx=1\n[b]\nx=2\n', },);
        const updated = tomlSet({ edit, path: ['b', 'x',], value: 9, },);
        expect(tomlStringify({ edit: updated, },),).toBe('[a]\nx=1\n[b]\nx=9\n',);
      },
    },),
    it({
      name: 'replaces a nested inline-table value without changing siblings',
      fn: async () => {
        const edit = parseTomlEdit({ source: 'foo = { a = { b = 1, c = 2 }, d = 3 }\n', },);
        const updated = tomlSet({ edit, path: ['foo', 'a', 'b',], value: 9, },);
        expect(tomlGetValue({ edit: updated, path: ['foo',], },),).toEqual({
          a: { b: 9, c: 2, },
          d: 3,
        },);
        expect(tomlStringify({ edit: updated, },),).toBe('foo = { a = { b = 9, c = 2, }, d = 3, }\n',);
      },
    },),
    it({
      name: 'rejects nullish values with the path-specific outer diagnostic',
      fn: async () => {
        const edit = parseTomlEdit({ source: '', },);
        expect(() => tomlSet({ edit, path: ['x',], value: null, },),)
          .toThrow('Cannot set x to null; use tomlDelete',);
        expect(() => tomlSet({ edit, path: ['x',], value: undefined, },),)
          .toThrow('Cannot set x to undefined; use tomlDelete',);
      },
    },),
  ],
},);
