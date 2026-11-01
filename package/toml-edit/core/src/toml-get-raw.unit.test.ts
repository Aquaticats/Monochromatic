/**
 Tests for `tomlGetRaw`.
 
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
  tomlGetRaw,
  tomlSet,
  TomlPathNotFoundError,
  TomlSpliceUnavailableError,
} from '@monochromatic-dev/toml-edit-core';

await describe({
  name: tomlGetRaw.name,
  children: [
    it({
      name: 'returns the source slice for an existing keyvalue',
      fn: async () => {
        const edit = parseTomlEdit({ source: "key = 'literal'\n", },);
        expect(tomlGetRaw({ edit, path: ['key',], },),).toBe("'literal'",);
      },
    },),

    it({
      name: 'preserves integer raw spelling (hex)',
      fn: async () => {
        const edit = parseTomlEdit({ source: 'count = 0x10\n', },);
        expect(tomlGetRaw({ edit, path: ['count',], },),).toBe('0x10',);
      },
    },),

    it({
      name: 'returns only the clean standard table header bytes',
      fn: async () => {
        const edit = parseTomlEdit({ source: 'root = 0\n[tbl]\nx = 1\n[other]\ny = 2\n', },);
        expect(tomlGetRaw({ edit, path: ['tbl',], },),).toBe('[tbl]\n',);
      },
    },),

    it({
      name: 'spans every clean array-of-tables header in a collection',
      fn: async () => {
        const edit = parseTomlEdit({
          source: 'root = 0\n[[foo]]\na = 1\n[[foo]]\na = 2\n[[foo]]\na = 3\n',
        },);
        const raw = tomlGetRaw({ edit, path: ['foo',], },);
        /**
         Every array-of-tables instance contributes its own header.
         */
        const EXPECTED_HEADERS = 3;
        expect(raw.startsWith('[[foo]]',),).toBe(true,);
        expect(raw.split('[[foo]]',).length - 1,).toBe(EXPECTED_HEADERS,);
        expect(raw.includes('root = 0',),).toBe(false,);
      },
    },),

    it({
      name: 'reads a clean indexed array-of-tables header and its child value',
      fn: async () => {
        const edit = parseTomlEdit({ source: '[[foo]]\nname="a"\n[[foo]]\nname="b"\n', },);
        expect(tomlGetRaw({ edit, path: ['foo', 1,], },),).toBe('[[foo]]\n',);
        expect(tomlGetRaw({ edit, path: ['foo', 1, 'name',], },),).toBe('"b"',);
        expect(() => tomlGetRaw({ edit, path: ['foo', 2,], },),)
          .toThrow(TomlPathNotFoundError,);
      },
    },),

    it({
      name: 'throws TomlPathNotFoundError for missing path',
      fn: async () => {
        const edit = parseTomlEdit({ source: 'foo = 1\n', },);
        expect(function lookup() {
          tomlGetRaw({ edit, path: ['missing',], },);
        },)
          .toThrow(TomlPathNotFoundError,);
      },
    },),

    it({
      name: 'throws TomlSpliceUnavailableError in canonical mode',
      fn: async () => {
        const edit = emptyTomlEdit();
        expect(function lookup() {
          tomlGetRaw({ edit, path: ['foo',], },);
        },)
          .toThrow(TomlSpliceUnavailableError,);
        expect(() => tomlGetRaw({ edit, path: ['foo',], },),)
          .toThrow('tomlGetRaw requires splice mode; current state is canonical',);
      },
    },),

    it({
      name: 'throws for an edited path (no clean source slice after tomlSet)',
      fn: async () => {
        const e0 = parseTomlEdit({ source: "key = 'literal'\nother = 1\n", },);
        const e1 = tomlSet({ edit: e0, path: ['key',], value: 'new', },);
        // The edited value is synthetic, so no original bytes back it.
        expect(function lookup() {
          tomlGetRaw({ edit: e1, path: ['key',], },);
        },)
          .toThrow(TomlPathNotFoundError,);
        // An unedited sibling still returns its parse-time bytes.
        expect(tomlGetRaw({ edit: e1, path: ['other',], },),).toBe('1',);
      },
    },),
  ],
},);
