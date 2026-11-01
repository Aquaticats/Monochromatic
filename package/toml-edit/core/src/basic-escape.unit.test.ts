import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  _emitStringValue,
  emptyTomlEdit,
  parseTomlEdit,
  tomlGetNode,
  tomlGetValue,
  tomlSet,
  tomlStringify,
} from '@monochromatic-dev/toml-edit-core';

/**
 Exercise basic-string escaping through the built setter and serializer.

 @param value - JS string requiring TOML encoding.

 @returns TOML assignment containing the encoded value.

 @example
 ```ts
 emitAssignedString('line\nnext');
 ```
 */
function emitAssignedString(value: string,): string {
  return tomlStringify({ edit: tomlSet({
    edit: emptyTomlEdit(),
    path: ['value',],
    value,
  },), },);
}

await describe({
  name: 'basic string escaping',
  children: [
    it({
      name: 'emits every named control escape and re-parses to the same value',
      fn: async () => {
        const value = '\b\t\n\f\r';
        const emitted = emitAssignedString(value,);
        expect(emitted,).toBe(`value = """\n${String.raw`\b\t\n\f\r`}"""\n`,);
        expect(tomlGetValue({
          edit: parseTomlEdit({ source: emitted, },),
          path: ['value',],
        },),).toBe(value,);
      },
    },),
    it({
      name: 'uses uppercase padded escapes for unnamed controls and DEL',
      fn: async () => {
        const value = '\u0000\u001A\u001F\u007F';
        const emitted = emitAssignedString(value,);
        expect(emitted,).toBe(`${String.raw`value = "\u0000\u001A\u001F\u007F"`}\n`,);
        expect(tomlGetValue({
          edit: parseTomlEdit({ source: emitted, },),
          path: ['value',],
        },),).toBe(value,);
      },
    },),
    it({
      name: 'escapes a quote and backslash in a single-line string',
      fn: async () => {
        const value = String.raw`a"b\c`;
        expect(emitAssignedString(value,),).toBe(`${String.raw`value = "a\"b\\c"`}\n`,);
      },
    },),
    it({
      name: 'preserves bare quotes and escapes a triple-quote run in multiline strings',
      fn: async () => {
        for (const [source, expected,] of [
          ['v = """\na"b"""\n', String.raw`"""a"b"""`,],
          ['v = """\na\\\"""b"""\n', String.raw`"""a\"\"\"b"""`,],
        ] as const) {
          const edit = parseTomlEdit({ source, },);
          const node = tomlGetNode({ edit, path: ['v',], },);
          if ((!('type' in node)) || (node.type !== 'TOMLValue') || (node.kind !== 'string'))
            throw new Error('Expected parsed multiline string node',);
          const emitted = _emitStringValue({ node, },);
          expect(emitted,).toBe(expected,);
          expect(tomlGetValue({
            edit: parseTomlEdit({ source: `v = ${emitted}\n`, },),
            path: ['v',],
          },),).toBe(tomlGetValue({ edit, path: ['v',], },),);
        }
      },
    },),
    it({
      name: 'preserves parsed literal and single-line basic string styles',
      fn: async () => {
        for (const [source, expected,] of [
          [`${String.raw`v = 'a\b'`}\n`, String.raw`'a\b'`,],
          ["v = '''\nhello\nworld'''\n", "'''hello\nworld'''",],
          [`${String.raw`v = "a\t b"`}\n`, String.raw`"a\t b"`,],
        ] as const) {
          const edit = parseTomlEdit({ source, },);
          const node = tomlGetNode({ edit, path: ['v',], },);
          if ((!('type' in node)) || (node.type !== 'TOMLValue') || (node.kind !== 'string'))
            throw new Error('Expected parsed string node',);
          const emitted = _emitStringValue({ node, },);
          expect(emitted,).toBe(expected,);
          expect(tomlGetValue({
            edit: parseTomlEdit({ source: `v = ${emitted}\n`, },),
            path: ['v',],
          },),).toBe(tomlGetValue({ edit, path: ['v',], },),);
        }
      },
    },),
  ],
},);
