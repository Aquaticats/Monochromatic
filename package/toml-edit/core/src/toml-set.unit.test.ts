/**
 Tests for `tomlSet` and its splice-mode behaviour.
 
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
  tomlGetValue,
  tomlInteger,
  tomlSet,
  tomlStringify,
  TomlImmutableNodeError,
  TomlTypeError,
} from '@monochromatic-dev/toml-edit-core';

await describe({
  name: tomlSet.name,
  children: [
    it({
      name: 'no-op set with the same value is byte-identical',
      fn: async () => {
        const source = 'foo = "bar"\n';
        const e0 = parseTomlEdit({ source, },);
        const e1 = tomlSet({ edit: e0, path: ['foo',], value: 'bar', },);
        expect(tomlStringify({ edit: e1, },),).toBe(source,);
      },
    },),

    it({
      name: 'preserves a comment line on another key',
      fn: async () => {
        const source = '# kept\nfoo = "bar"\nbaz = "qux"\n';
        const e1 = tomlSet({
          edit: parseTomlEdit({ source, },),
          path: ['baz',],
          value: 'changed',
        },);
        expect(tomlStringify({ edit: e1, },),).toBe(
          '# kept\nfoo = "bar"\nbaz = "changed"\n',
        );
      },
    },),

    it({
      name: 'preserves trailing inline comment on the mutated key',
      fn: async () => {
        const source = 'foo = "bar"  # trailing\n';
        const e1 = tomlSet({
          edit: parseTomlEdit({ source, },),
          path: ['foo',],
          value: 'new',
        },);
        expect(tomlStringify({ edit: e1, },),).toBe(
          'foo = "new"  # trailing\n',
        );
      },
    },),

    it({
      name: 'preserves literal quote style on re-set with same value',
      fn: async () => {
        const source = "foo = 'literal'\n";
        const e1 = tomlSet({
          edit: parseTomlEdit({ source, },),
          path: ['foo',],
          value: 'literal',
        },);
        expect(tomlStringify({ edit: e1, },),).toBe(source,);
      },
    },),

    it({
      name: 'preserves hex integer raw spelling on re-set with same value',
      fn: async () => {
        const source = 'count = 0x10\n';
        const e1 = tomlSet({
          edit: parseTomlEdit({ source, },),
          path: ['count',],
          value: 16,
        },);
        expect(tomlStringify({ edit: e1, },),).toBe(source,);
      },
    },),

    it({
      name: 'underscore-formatted integers normalise on re-set (parser drops _)',
      fn: async () => {
        const source = 'size = 1_000\n';
        const e1 = tomlSet({
          edit: parseTomlEdit({ source, },),
          path: ['size',],
          value: 1_000,
        },);
        expect(tomlStringify({ edit: e1, },),).toBe('size = 1000\n',);
      },
    },),

    it({
      name: 'tomlInteger() forces integer emission for whole numbers',
      fn: async () => {
        const source = '';
        const e1 = tomlSet({
          edit: parseTomlEdit({ source, },),
          path: ['count',],
          value: tomlInteger(42,),
        },);
        expect(tomlStringify({ edit: e1, },),).toContain('count = 42',);
      },
    },),

    it({
      name: 'tomlFloat() forces float emission with .0 suffix',
      fn: async () => {
        const source = '';
        const e1 = tomlSet({
          edit: parseTomlEdit({ source, },),
          path: ['ratio',],
          value: tomlFloat(1,),
        },);
        expect(tomlStringify({ edit: e1, },),).toContain('ratio = 1.0',);
      },
    },),

    it({
      name: 'path-create adds a new top-level key at end',
      fn: async () => {
        const source = 'foo = "bar"\n';
        const e1 = tomlSet({
          edit: parseTomlEdit({ source, },),
          path: ['baz',],
          value: 'qux',
        },);
        expect(tomlStringify({ edit: e1, },),).toBe('foo = "bar"\nbaz = "qux"\n',);
      },
    },),

    it({
      name: 'path-create inserts inside an existing table',
      fn: async () => {
        const source = '[tools]\nbun = "1.2"\n';
        const e1 = tomlSet({
          edit: parseTomlEdit({ source, },),
          path: ['tools', 'node',],
          value: '22',
        },);
        const out = tomlStringify({ edit: e1, },);
        expect(out,).toContain('[tools]\nbun = "1.2"\nnode = "22"\n',);
      },
    },),

    it({
      name: 'immutable: setting on a derived state leaves the original unchanged',
      fn: async () => {
        const source = 'foo = "bar"\n';
        const e0 = parseTomlEdit({ source, },);
        const e1 = tomlSet({ edit: e0, path: ['foo',], value: 'new', },);
        expect(tomlGetValue({ edit: e0, path: ['foo',], },),).toBe('bar',);
        expect(tomlGetValue({ edit: e1, path: ['foo',], },),).toBe('new',);
        expect(tomlStringify({ edit: e0, },),).toBe(source,);
      },
    },),

    it({
      name: 'throws TomlTypeError on null',
      fn: async () => {
        expect(function setNullThrows() {
          tomlSet({
            edit: parseTomlEdit({ source: 'foo = 1\n', },),
            path: ['foo',],
            value: null,
          },);
        },)
          .toThrow(TomlTypeError,);
      },
    },),

    it({
      name: 'throws TomlTypeError on undefined',
      fn: async () => {
        expect(function setUndefinedThrows() {
          tomlSet({
            edit: parseTomlEdit({ source: 'foo = 1\n', },),
            path: ['foo',],
            value: undefined,
          },);
        },)
          .toThrow(TomlTypeError,);
      },
    },),

    // Table replace (Limitation 1)

    it({
      name: 'replaces a [foo] table body with the new object entries',
      fn: async () => {
        const source = '[foo]\nold = 3\n';
        const e1 = tomlSet({
          edit: parseTomlEdit({ source, },),
          path: ['foo',],
          value: { x: 1, y: 2, },
        },);
        expect(tomlStringify({ edit: e1, },),).toBe('[foo]\nx = 1\ny = 2\n',);
      },
    },),

    it({
      name: 'replaces [foo] body while preserving [foo.sub]',
      fn: async () => {
        const source = '[foo]\nold = 3\n[foo.sub]\nk = 1\n';
        const e1 = tomlSet({
          edit: parseTomlEdit({ source, },),
          path: ['foo',],
          value: { x: 1, },
        },);
        expect(tomlStringify({ edit: e1, },),).toBe('[foo]\nx = 1\n[foo.sub]\nk = 1\n',);
      },
    },),

    it({
      name: 'absorbs a trailing inline comment when deleting body keys',
      fn: async () => {
        const source = '[foo]\nold = 3  # bye\n';
        const e1 = tomlSet({
          edit: parseTomlEdit({ source, },),
          path: ['foo',],
          value: { x: 1, },
        },);
        expect(tomlStringify({ edit: e1, },),).toBe('[foo]\nx = 1\n',);
      },
    },),

    it({
      name: 'top-level replace keeps sub-tables intact',
      fn: async () => {
        const source = 'a = 1\n[s]\nk = 2\n';
        const e1 = tomlSet({
          edit: parseTomlEdit({ source, },),
          path: [],
          value: { b: 2, },
        },);
        expect(tomlStringify({ edit: e1, },),).toBe('b = 2\n[s]\nk = 2\n',);
      },
    },),

    it({
      name: 'array-of-tables wholesale replace emits one [[foo]] block per array element',
      fn: async () => {
        const source = '[[foo]]\nx = 1\n[[foo]]\nx = 2\n';
        const e1 = tomlSet({
          edit: parseTomlEdit({ source, },),
          path: ['foo',],
          value: [
            { y: 1, },
            { y: 2, },
          ],
        },);
        expect(tomlStringify({ edit: e1, },),).toBe('[[foo]]\ny = 1\n[[foo]]\ny = 2\n',);
      },
    },),

    it({
      name: 'array-of-tables replace shrinks the instance count (2 to 1)',
      fn: async () => {
        const source = '[[foo]]\nx = 1\n[[foo]]\nx = 2\n';
        const e1 = tomlSet({
          edit: parseTomlEdit({ source, },),
          path: ['foo',],
          value: [{ z: 9, },],
        },);
        expect(tomlStringify({ edit: e1, },),).toBe('[[foo]]\nz = 9\n',);
      },
    },),

    it({
      name: 'array-of-tables replace grows the instance count (2 to 3)',
      fn: async () => {
        const source = '[[foo]]\nx = 1\n[[foo]]\nx = 2\n';
        const e1 = tomlSet({
          edit: parseTomlEdit({ source, },),
          path: ['foo',],
          value: [
            { z: 1, },
            { z: 2, },
            { z: 3, },
          ],
        },);
        expect(tomlStringify({ edit: e1, },),).toBe(
          '[[foo]]\nz = 1\n[[foo]]\nz = 2\n[[foo]]\nz = 3\n',
        );
      },
    },),

    it({
      name: 'array-of-tables replace with empty array clears every instance',
      fn: async () => {
        const source = '[[foo]]\nx = 1\n[[foo]]\nx = 2\n';
        const e1 = tomlSet({
          edit: parseTomlEdit({ source, },),
          path: ['foo',],
          value: [],
        },);
        expect(tomlStringify({ edit: e1, },),).toBe('',);
      },
    },),

    it({
      name: 'array-of-tables replace preserves a sibling [bar] table',
      fn: async () => {
        const source = '[[foo]]\nx = 1\n[[foo]]\nx = 2\n[bar]\nz = 3\n';
        const e1 = tomlSet({
          edit: parseTomlEdit({ source, },),
          path: ['foo',],
          value: [{ y: 1, },],
        },);
        expect(tomlStringify({ edit: e1, },),).toBe('[[foo]]\ny = 1\n[bar]\nz = 3\n',);
      },
    },),

    it({
      name: 'array-of-tables replace at a nested path [[a.b]]',
      fn: async () => {
        const source = '[[a.b]]\nx = 1\n[[a.b]]\nx = 2\n';
        const e1 = tomlSet({
          edit: parseTomlEdit({ source, },),
          path: ['a', 'b',],
          value: [{ y: 1, },],
        },);
        expect(tomlStringify({ edit: e1, },),).toBe('[[a.b]]\ny = 1\n',);
      },
    },),

    it({
      name: 'array-of-tables replace with non-array value throws TomlTypeError',
      fn: async () => {
        const source = '[[foo]]\nx = 1\n[[foo]]\nx = 2\n';
        expect(function aotNonArrayThrows() {
          tomlSet({
            edit: parseTomlEdit({ source, },),
            path: ['foo',],
            value: { y: 1, },
          },);
        },)
          .toThrow(TomlTypeError,);
      },
    },),

    it({
      name: 'array-of-tables replace with a non-object element throws TomlTypeError',
      fn: async () => {
        const source = '[[foo]]\nx = 1\n[[foo]]\nx = 2\n';
        expect(function aotNonObjectElementThrows() {
          tomlSet({
            edit: parseTomlEdit({ source, },),
            path: ['foo',],
            value: [
              { y: 1, },
              42,
            ],
          },);
        },)
          .toThrow(TomlTypeError,);
      },
    },),

    it({
      name: 'sibling-tables wholesale replace still throws TomlImmutableNodeError',
      fn: async () => {
        const source = '[a.b]\nx = 1\n[a.c]\ny = 2\n';
        expect(function siblingTablesReplaceThrows() {
          tomlSet({
            edit: parseTomlEdit({ source, },),
            path: ['a',],
            value: [{ z: 3, },],
          },);
        },)
          .toThrow(TomlImmutableNodeError,);
      },
    },),

    it({
      name: 'effective sub-path read after AOT replace returns new element value',
      fn: async () => {
        const source = '[[foo]]\nx = 1\n[[foo]]\nx = 2\n';
        const e1 = tomlSet({
          edit: parseTomlEdit({ source, },),
          path: ['foo',],
          value: [
            { y: 7, },
            { y: 8, },
          ],
        },);
        expect(tomlGetValue({ edit: e1, path: ['foo', 0, 'y',], },),).toBe(7,);
        expect(tomlGetValue({ edit: e1, path: ['foo', 1, 'y',], },),).toBe(8,);
      },
    },),

    it({
      name: 'whole-AOT read after replace returns the new instances',
      fn: async () => {
        const source = '[[foo]]\nx = 1\n[[foo]]\nx = 2\n';
        const e1 = tomlSet({
          edit: parseTomlEdit({ source, },),
          path: ['foo',],
          value: [
            { y: 7, },
            { y: 8, },
          ],
        },);
        // The single always-current tree reads the parent path consistently
        // with what tomlStringify emits (no sub-path-only projection limit).
        expect(tomlGetValue({ edit: e1, path: ['foo',], },),).toEqual([
          { y: 7, },
          { y: 8, },
        ],);
      },
    },),

    it({
      name: 'immutable: AOT replace on derived state leaves original intact',
      fn: async () => {
        const source = '[[foo]]\nx = 1\n[[foo]]\nx = 2\n';
        const e0 = parseTomlEdit({ source, },);
        const e1 = tomlSet({
          edit: e0,
          path: ['foo',],
          value: [
            { y: 1, },
            { y: 2, },
          ],
        },);
        expect(tomlStringify({ edit: e0, },),).toBe(source,);
        expect(tomlStringify({ edit: e1, },),).toBe('[[foo]]\ny = 1\n[[foo]]\ny = 2\n',);
      },
    },),

    it({
      name: 'non-object value on a table throws TomlTypeError',
      fn: async () => {
        const source = '[foo]\nx = 1\n';
        expect(function nonObjectThrows() {
          tomlSet({
            edit: parseTomlEdit({ source, },),
            path: ['foo',],
            value: 42,
          },);
        },)
          .toThrow(TomlTypeError,);
      },
    },),

    it({
      name: 'empty object {} clears body and inserts nothing',
      fn: async () => {
        const source = '[foo]\nold = 3\n';
        const e1 = tomlSet({
          edit: parseTomlEdit({ source, },),
          path: ['foo',],
          value: {},
        },);
        expect(tomlStringify({ edit: e1, },),).toBe('[foo]\n',);
      },
    },),

    it({
      name: 'nested object value becomes an inline table',
      fn: async () => {
        const source = '[foo]\nold = 3\n';
        const e1 = tomlSet({
          edit: parseTomlEdit({ source, },),
          path: ['foo',],
          value: { x: { y: 1, }, },
        },);
        expect(tomlStringify({ edit: e1, },),).toBe('[foo]\nx = { y = 1, }\n',);
      },
    },),

    it({
      name: 'immutable: table replace on derived state leaves original intact',
      fn: async () => {
        const source = '[foo]\nold = 3\n';
        const e0 = parseTomlEdit({ source, },);
        const e1 = tomlSet({ edit: e0, path: ['foo',], value: { x: 1, }, },);
        expect(tomlStringify({ edit: e0, },),).toBe(source,);
        expect(tomlStringify({ edit: e1, },),).toBe('[foo]\nx = 1\n',);
      },
    },),

    // Deep path-create (Limitation 2)

    it({
      name: 'deep path-create from empty doc emits dotted key at top-level',
      fn: async () => {
        const e1 = tomlSet({
          edit: emptyTomlEdit(),
          path: ['a', 'b', 'c',],
          value: 42,
        },);
        expect(tomlStringify({ edit: e1, },),).toBe('a.b.c = 42\n',);
      },
    },),

    it({
      name: 'deep path-create into existing [a] inserts dotted key inside',
      fn: async () => {
        const source = '[a]\nx = 1\n';
        const e1 = tomlSet({
          edit: parseTomlEdit({ source, },),
          path: ['a', 'b', 'c',],
          value: 42,
        },);
        expect(tomlStringify({ edit: e1, },),).toBe('[a]\nx = 1\nb.c = 42\n',);
      },
    },),

    it({
      name: 'inline-table extension appends a new entry',
      fn: async () => {
        const source = 'foo = {}\n';
        const e1 = tomlSet({
          edit: parseTomlEdit({ source, },),
          path: ['foo', 'x',],
          value: 1,
        },);
        expect(tomlStringify({ edit: e1, },),).toBe('foo = { x = 1, }\n',);
      },
    },),

    it({
      name: 'inline-table multi-segment extension uses dotted-key form',
      fn: async () => {
        const source = 'foo = {}\n';
        const e1 = tomlSet({
          edit: parseTomlEdit({ source, },),
          path: ['foo', 'a', 'b',],
          value: 3,
        },);
        expect(tomlStringify({ edit: e1, },),).toBe('foo = { a.b = 3, }\n',);
      },
    },),

    it({
      name: 'object value at deep path becomes an inline table',
      fn: async () => {
        const e1 = tomlSet({
          edit: emptyTomlEdit(),
          path: ['a', 'b',],
          value: { c: 1, },
        },);
        expect(tomlStringify({ edit: e1, },),).toBe('a.b = { c = 1, }\n',);
      },
    },),

    it({
      name: 'numeric segment in the missing tail throws TomlImmutableNodeError',
      fn: async () => {
        expect(function numericTailThrows() {
          tomlSet({
            edit: emptyTomlEdit(),
            path: ['foo', 0, 'bar',],
            value: 1,
          },);
        },)
          .toThrow(TomlImmutableNodeError,);
      },
    },),

    it({
      name: 'top-level dotted-key insert lands before sibling table headers',
      fn: async () => {
        const source = 'a = 1\n[s]\nk = 2\n';
        const e1 = tomlSet({
          edit: parseTomlEdit({ source, },),
          path: ['b',],
          value: 2,
        },);
        expect(tomlStringify({ edit: e1, },),).toBe('a = 1\nb = 2\n[s]\nk = 2\n',);
      },
    },),

    // Dotted-key collision detection

    it({
      name: 'rejects inline-table extension that conflicts with a deeper existing key',
      fn: async () => {
        const source = 'foo = { a.b.c = 1 }\n';
        expect(function inlineCollisionDeeperThrows() {
          tomlSet({
            edit: parseTomlEdit({ source, },),
            path: ['foo', 'a', 'b',],
            value: 3,
          },);
        },)
          .toThrow(TomlImmutableNodeError,);
      },
    },),

    it({
      name: 'rejects inline-table extension that conflicts with a shorter existing key',
      fn: async () => {
        const source = 'foo = { a = 1 }\n';
        expect(function inlineCollisionShorterThrows() {
          tomlSet({
            edit: parseTomlEdit({ source, },),
            path: ['foo', 'a', 'b',],
            value: 3,
          },);
        },)
          .toThrow(TomlImmutableNodeError,);
      },
    },),

    it({
      name: 'allows deep dotted key when sibling table is strictly deeper',
      fn: async () => {
        const source = '[a]\n[a.b.deeper]\nd = 1\n';
        const e1 = tomlSet({
          edit: parseTomlEdit({ source, },),
          path: ['a', 'b', 'c',],
          value: 42,
        },);
        expect(tomlStringify({ edit: e1, },),).toContain('b.c = 42',);
      },
    },),

    it({
      name: 'allows top-level dotted key when sibling table prefix is sibling-only',
      fn: async () => {
        const source = '[a.x]\ny = 1\n';
        const e1 = tomlSet({
          edit: parseTomlEdit({ source, },),
          path: ['a', 'b', 'c',],
          value: 42,
        },);
        expect(tomlStringify({ edit: e1, },),).toContain('a.b.c = 42',);
      },
    },),

    it({
      name: 'allows inline-table extension to a sibling dotted leaf',
      fn: async () => {
        const source = 'foo = { a.c = 1 }\n';
        const e1 = tomlSet({
          edit: parseTomlEdit({ source, },),
          path: ['foo', 'a', 'b',],
          value: 3,
        },);
        expect(tomlStringify({ edit: e1, },),).toBe('foo = { a.c = 1, a.b = 3, }\n',);
      },
    },),
  ],
},);
