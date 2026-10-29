/**
 Tests for the JS-to-TOML value coercion path used by `tomlSet`.
 
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
  tomlLocalDate,
  tomlLocalDateTime,
  tomlLocalTime,
  tomlSet,
  tomlStringify,
  TomlTypeError,
} from '@monochromatic-dev/toml-edit-core';

await describe({
  name: 'JS-to-TOML value coercion',
  children: [
    it({
      name: 'tomlLocalDate produces a local-date assignment',
      fn: async () => {
        const e1 = tomlSet({
          edit: parseTomlEdit({ source: '', },),
          path: ['d',],
          value: tomlLocalDate('2026-05-14',),
        },);
        expect(tomlStringify({ edit: e1, },),).toContain('d = 2026-05-14',);
      },
    },),

    it({
      name: 'tomlLocalDateTime produces a local-date-time assignment',
      fn: async () => {
        const e1 = tomlSet({
          edit: parseTomlEdit({ source: '', },),
          path: ['t',],
          value: tomlLocalDateTime('2026-05-14T10:00:00',),
        },);
        expect(tomlStringify({ edit: e1, },),).toContain('t = 2026-05-14T10:00:00',);
      },
    },),

    it({
      name: 'tomlLocalTime produces a local-time assignment',
      fn: async () => {
        const e1 = tomlSet({
          edit: parseTomlEdit({ source: '', },),
          path: ['t',],
          value: tomlLocalTime('10:00:00',),
        },);
        expect(tomlStringify({ edit: e1, },),).toContain('t = 10:00:00',);
      },
    },),

    it({
      name: 'boolean coerces to true/false',
      fn: async () => {
        const e1 = tomlSet({
          edit: parseTomlEdit({ source: '', },),
          path: ['enabled',],
          value: true,
        },);
        expect(tomlStringify({ edit: e1, },),).toContain('enabled = true',);
      },
    },),

    it({
      name: 'array of primitives renders inline',
      fn: async () => {
        const e1 = tomlSet({
          edit: parseTomlEdit({ source: '', },),
          path: ['xs',],
          value: [1, 2, 3,],
        },);
        const out = tomlStringify({ edit: e1, },);
        expect(out,).toContain('xs = [ 1, 2, 3, ]',);
      },
    },),

    it({
      name: 'array re-set preserves the raw spelling of equal parsed elements',
      fn: async () => {
        const edit = tomlSet({
          edit: parseTomlEdit({ source: 'arr = [0x10, 0x20]\n', },),
          path: ['arr',],
          value: [16, 32,],
        },);
        expect(tomlStringify({ edit, },),).toBe('arr = [ 0x10, 0x20, ]\n',);
        expect(tomlGetValue({ edit, path: ['arr',], },),).toEqual([16, 32,],);
      },
    },),

    it({
      name: 'wrapped scalar reads expose plain JS values',
      fn: async () => {
        const base = emptyTomlEdit();
        const count = tomlSet({ edit: base, path: ['count',], value: tomlInteger(42n,), },);
        const ratio = tomlSet({ edit: count, path: ['ratio',], value: tomlFloat(1,), },);
        const meeting = tomlSet({
          edit: ratio,
          path: ['meeting',],
          value: tomlLocalDateTime('2026-05-14T10:00:00',),
        },);
        expect(tomlGetValue({ edit: meeting, path: ['count',], },),).toBe(42,);
        expect(tomlGetValue({ edit: meeting, path: ['ratio',], },),).toBe(1,);
        expect(tomlGetValue({ edit: meeting, path: ['meeting',], },),).toBe('2026-05-14T10:00:00',);
        expect(tomlStringify({ edit: meeting, },),).toContain('ratio = 1.0',);
      },
    },),

    it({
      name: 'coerces a structurally tagged float string to a numeric read',
      fn: async () => {
        const edit = tomlSet({
          edit: emptyTomlEdit(),
          path: ['ratio',],
          value: { tomlKind: 'float', value: '1.5', },
        },);
        expect(tomlGetValue({ edit, path: ['ratio',], },),).toBe(1.5,);
        expect(tomlStringify({ edit, },),).toBe('ratio = 1.5\n',);
      },
    },),

    it({
      name: 'null and undefined throw TomlTypeError',
      fn: async () => {
        const edit = parseTomlEdit({ source: '', },);
        expect(function setNull() {
          tomlSet({ edit, path: ['x',], value: null, },);
        },)
          .toThrow(TomlTypeError,);
        expect(function setUndefined() {
          tomlSet({ edit, path: ['x',], value: undefined, },);
        },)
          .toThrow(TomlTypeError,);
      },
    },),
    it({
      name: 'rejects null and undefined nested inside arrays and tables',
      fn: async () => {
        const edit = emptyTomlEdit();
        expect(() => tomlSet({ edit, path: ['values',], value: [null,], },),)
          .toThrow('Cannot encode null as TOML; use tomlDelete to remove a key',);
        expect(() => tomlSet({ edit, path: ['table',], value: { child: undefined, }, },),)
          .toThrow('Cannot encode undefined as TOML; use tomlDelete to remove a key',);
      },
    },),
  ],
},);
