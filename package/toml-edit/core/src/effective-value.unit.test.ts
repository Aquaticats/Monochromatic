/**
 Tests for the read-through-edits semantics: pending `tomlSet` and
 `tomlDelete` deltas must be visible to subsequent `tomlGetValue` calls on
 the same (or branched) state.
 
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
  tomlSet,
} from '@monochromatic-dev/toml-edit-core';

await describe({
  name: 'effective-value (read-through-edits)',
  children: [
    it({
      name: 'tomlGetValue on a state returns the value set on that state',
      fn: async () => {
        const e0 = parseTomlEdit({ source: 'foo = "old"\n', },);
        const e1 = tomlSet({ edit: e0, path: ['foo',], value: 'new', },);
        expect(tomlGetValue({ edit: e1, path: ['foo',], },),).toBe('new',);
      },
    },),

    it({
      name: 'tomlGetValue on the original state is unchanged after a derived set',
      fn: async () => {
        const e0 = parseTomlEdit({ source: 'foo = "old"\n', },);
        const e1 = tomlSet({ edit: e0, path: ['foo',], value: 'new', },);
        expect(tomlGetValue({ edit: e0, path: ['foo',], },),).toBe('old',);
        expect(tomlGetValue({ edit: e1, path: ['foo',], },),).toBe('new',);
      },
    },),

    it({
      name: 'tomlGetValue on a deleted path returns undefined',
      fn: async () => {
        const e0 = parseTomlEdit({ source: 'foo = "v"\n', },);
        const e1 = tomlDelete({ edit: e0, path: ['foo',], },);
        expect(tomlGetValue({ edit: e1, path: ['foo',], },),).toBe(undefined,);
      },
    },),

    it({
      name: 'path-create set is visible on the derived state',
      fn: async () => {
        const e0 = parseTomlEdit({ source: '', },);
        const e1 = tomlSet({ edit: e0, path: ['new',], value: 42, },);
        expect(tomlGetValue({ edit: e1, path: ['new',], },),).toBe(42,);
        expect(tomlGetValue({ edit: e0, path: ['new',], },),).toBe(undefined,);
      },
    },),

    it({
      name: 'branching: trial set on a state is invisible to siblings',
      fn: async () => {
        const e0 = parseTomlEdit({ source: 'x = 1\n', },);
        const trial = tomlSet({ edit: e0, path: ['x',], value: 999, },);
        const other = tomlSet({ edit: e0, path: ['x',], value: 2, },);
        expect(tomlGetValue({ edit: trial, path: ['x',], },),).toBe(999,);
        expect(tomlGetValue({ edit: other, path: ['x',], },),).toBe(2,);
      },
    },),
  ],
},);
