/**
 Tests for canonical-mode emission via `emptyTomlEdit` plus setters.
 
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
  tomlGetValue,
  tomlSet,
  tomlSetHeaderComment,
  tomlStringify,
} from '@monochromatic-dev/toml-edit-core';

await describe({
  name: 'canonical emission via emptyTomlEdit',
  children: [
    it({
      name: 'emits valid TOML that re-parses without error',
      fn: async () => {
        const e0 = emptyTomlEdit();
        const e1 = tomlSet({ edit: e0, path: ['title',], value: 'Demo', },);
        const out = tomlStringify({ edit: e1, },);
        expect(function reparse() {
          parseTomlEdit({ source: out, },);
        },)
          .not
          .toThrow();
      },
    },),

    it({
      name: 'defaults to explicit rather than dotted key creation',
      fn: async () => {
        expect(emptyTomlEdit().canonical.preferDottedKeysForCreate,).toBe(false,);
      },
    },),

    it({
      name: 'canonical overrides retain an empty source',
      fn: async () => {
        const edit = emptyTomlEdit({ canonical: { indent: 4, }, },);
        expect(edit.source,).toBe('',);
        expect(edit.mode,).toBe('canonical',);
        expect(edit.canonical.indent,).toBe(4,);
      },
    },),

    it({
      name: 'omits the final newline in a synthetic canonical document when disabled',
      fn: async () => {
        const edit = emptyTomlEdit({ canonical: { trailingNewline: false, }, },);
        const updated = tomlSet({ edit, path: ['title',], value: 'Demo', },);
        expect(tomlStringify({ edit: updated, },),).toBe('title = "Demo"',);
      },
    },),

    it({
      name: 'omits the final newline in a parsed canonical document when disabled',
      fn: async () => {
        const edit = parseTomlEdit({
          source: 'title = "Demo"\n',
          mode: 'canonical',
          canonical: { trailingNewline: false, },
        },);
        expect(tomlStringify({ edit, },),).toBe('title = "Demo"',);
      },
    },),

    it({
      name: 'adds the final newline in a parsed canonical document when enabled',
      fn: async () => {
        const edit = parseTomlEdit({ source: 'title = "Demo"', mode: 'canonical', },);
        expect(tomlStringify({ edit, },),).toBe('title = "Demo"\n',);
      },
    },),

    it({
      name: 'keeps empty canonical output empty',
      fn: async () => {
        expect(tomlStringify({ edit: emptyTomlEdit(), },),).toBe('',);
        expect(tomlStringify({
          edit: emptyTomlEdit({ canonical: { trailingNewline: false, }, },),
        },),).toBe('',);
      },
    },),

    it({
      name: 'omits terminal newlines in a header-only canonical document',
      fn: async () => {
        const edit = tomlSetHeaderComment({
          edit: emptyTomlEdit({ canonical: { trailingNewline: false, }, },),
          comment: 'Generated',
        },);
        expect(tomlStringify({ edit, },),).toBe('# Generated',);
      },
    },),

    it({
      name: 'omits multiple terminal newlines without changing multiline string content',
      fn: async () => {
        const edit = parseTomlEdit({
          source: 'note = """\nhello\nworld\n"""\n\n',
          mode: 'canonical',
          canonical: { trailingNewline: false, },
        },);
        const text = tomlStringify({ edit, },);
        expect(text.endsWith('\n',),).toBe(false,);
        expect(tomlGetValue({
          edit: parseTomlEdit({ source: text, },),
          path: ['note',],
        },),).toBe(tomlGetValue({ edit, path: ['note',], },),);
      },
    },),

    it({
      name: 'honors the newline preference in an empty-source splice state',
      fn: async () => {
        const edit = parseTomlEdit({ source: '', canonical: { trailingNewline: false, }, },);
        const updated = tomlSet({ edit, path: ['title',], value: 'Demo', },);
        expect(tomlStringify({ edit: updated, },),).toBe('title = "Demo"',);
      },
    },),

    it({
      name: 'preserves the final newline in an untouched splice document',
      fn: async () => {
        const edit = parseTomlEdit({
          source: 'title = "Demo"\n',
          canonical: { trailingNewline: false, },
        },);
        expect(tomlStringify({ edit, },),).toBe('title = "Demo"\n',);
      },
    },),

    it({
      name: 'preserves the final newline after editing a nonempty splice document',
      fn: async () => {
        const edit = parseTomlEdit({
          source: 'title = "Old"\n',
          canonical: { trailingNewline: false, },
        },);
        const updated = tomlSet({ edit, path: ['title',], value: 'New', },);
        expect(tomlStringify({ edit: updated, },),).toBe('title = "New"\n',);
      },
    },),

    it({
      name: 'multiple sets accumulate in the output',
      fn: async () => {
        const e0 = emptyTomlEdit();
        const e1 = tomlSet({ edit: e0, path: ['title',], value: 'Demo', },);
        const e2 = tomlSet({ edit: e1, path: ['version',], value: '1.0', },);
        const out = tomlStringify({ edit: e2, },);
        expect(out,).toContain('title = "Demo"',);
        expect(out,).toContain('version = "1.0"',);
      },
    },),

    it({
      name: 'header comment lands above the body',
      fn: async () => {
        const e0 = emptyTomlEdit();
        const e1 = tomlSetHeaderComment({ edit: e0, comment: 'Generated', },);
        const e2 = tomlSet({ edit: e1, path: ['title',], value: 'Demo', },);
        const out = tomlStringify({ edit: e2, },);
        expect(out.indexOf('# Generated',),).toBeLessThan(out.indexOf('title',),);
      },
    },),
  ],
},);
