/**
 Tests for `tomlSetHeaderComment`.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  emptyTomlEdit,
  tomlSet,
  tomlSetHeaderComment,
  tomlStringify,
} from '@monochromatic-dev/toml-edit-core';

await describe({
  name: tomlSetHeaderComment.name,
  children: [
    it({
      name: 'string comment lands above the body in canonical mode',
      fn: async () => {
        const e0 = emptyTomlEdit();
        const e1 = tomlSetHeaderComment({ edit: e0, comment: 'Generated', },);
        const e2 = tomlSet({ edit: e1, path: ['title',], value: 'Demo', },);
        const out = tomlStringify({ edit: e2, },);
        expect(out.startsWith('# Generated',),).toBe(true,);
      },
    },),

    it({
      name: 'array of lines emits one comment line each',
      fn: async () => {
        const e0 = emptyTomlEdit();
        const e1 = tomlSetHeaderComment({ edit: e0, comment: ['one', 'two',], },);
        const e2 = tomlSet({ edit: e1, path: ['title',], value: 'Demo', },);
        const out = tomlStringify({ edit: e2, },);
        expect(out,).toBe('# one\n# two\n\ntitle = "Demo"\n',);
      },
    },),

    it({
      name: 'empty comment omits a synthetic header',
      fn: async () => {
        const edit = tomlSetHeaderComment({ edit: emptyTomlEdit(), comment: '', },);
        const updated = tomlSet({ edit, path: ['title',], value: 'Demo', },);
        expect(tomlStringify({ edit: updated, },),).toBe('title = "Demo"\n',);
      },
    },),

    it({
      name: 'omitting comment clears the header comment',
      fn: async () => {
        const e0 = emptyTomlEdit();
        const e1 = tomlSetHeaderComment({ edit: e0, comment: 'X', },);
        const e2 = tomlSetHeaderComment({ edit: e1, },);
        const e3 = tomlSet({ edit: e2, path: ['title',], value: 'Demo', },);
        const out = tomlStringify({ edit: e3, },);
        expect(out.includes('# X',),).toBe(false,);
      },
    },),
  ],
},);
