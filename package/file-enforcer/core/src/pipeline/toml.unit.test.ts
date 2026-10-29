import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  editTomlKey,
  getTomlProperty,
} from '../../dist/final/node/index.mjs';

await describe({
  name: '',
  children: [
    //region getTomlProperty

    describe({
      name: getTomlProperty.name,
      children: [
        it({
          name: 'extracts a top-level string property',
          fn: async () => {
            /** TOML with a top-level string value */
            const toml = 'name = "Alice"';
            expect(getTomlProperty({ path: ['name',], content: toml, },),).toBe('Alice',);
          },
        },),
        it({
          name: 'extracts a value nested under a section header',
          fn: async () => {
            /** TOML with a section header */
            const toml = '[settings]\ntheme = "dark"';
            expect(getTomlProperty({
              path: ['settings', 'theme',],
              content: toml,
            },),)
              .toBe('dark',);
          },
        },),
        it({
          name: 'stringifies integer values',
          fn: async () => {
            const toml = 'count = 42';
            expect(getTomlProperty({ path: ['count',], content: toml, },),).toBe('42',);
          },
        },),
        it({
          name: 'stringifies float values',
          fn: async () => {
            const toml = 'ratio = 3.14';
            expect(getTomlProperty({ path: ['ratio',], content: toml, },),).toBe('3.14',);
          },
        },),
        it({
          name: 'stringifies boolean true',
          fn: async () => {
            const toml = 'active = true';
            expect(getTomlProperty({ path: ['active',], content: toml, },),).toBe(
              'true',
            );
          },
        },),
        it({
          name: 'stringifies boolean false',
          fn: async () => {
            const toml = 'active = false';
            expect(getTomlProperty({ path: ['active',], content: toml, },),).toBe(
              'false',
            );
          },
        },),
        it({
          name: 'stringifies array values with pretty printing',
          fn: async () => {
            const toml = 'items = [1, 2, 3]';
            expect(getTomlProperty({ path: ['items',], content: toml, },),).toBe(
              JSON.stringify([1, 2, 3,], null, 2,),
            );
          },
        },),
        it({
          name: 'stringifies inline-table values with pretty printing',
          fn: async () => {
            const toml = 'pkg = { name = "foo", version = "1.0" }';
            expect(getTomlProperty({ path: ['pkg',], content: toml, },),).toBe(
              JSON.stringify({ name: 'foo', version: '1.0', }, null, 2,),
            );
          },
        },),
        it({
          name: 'indexes into array-of-tables by numeric segment',
          fn: async () => {
            /** TOML with array-of-tables blocks */
            const toml = '[[fruits]]\nname = "apple"\n\n[[fruits]]\nname = "orange"';
            expect(getTomlProperty({
              path: ['fruits', 0, 'name',],
              content: toml,
            },),)
              .toBe('apple',);
          },
        },),
        it({
          name: 'returns undefined for missing paths',
          fn: async () => {
            const toml = 'exists = true';
            expect(getTomlProperty({
              path: ['missing',],
              content: toml,
            },),)
              .toBeUndefined();
          },
        },),
        it({
          name: 'addresses keys containing literal dots via array segments',
          fn: async () => {
            /** TOML with a quoted key containing a literal dot */
            const toml = '"a.b" = "value"';
            /** Array form treats the segment as one key, not nested a.b */
            expect(getTomlProperty({ path: ['a.b',], content: toml, },),).toBe('value',);
          },
        },),
        it({
          name: 'throws on invalid TOML input',
          fn: async () => {
            expect(() => getTomlProperty({ path: ['key',], content: '[[invalid', },))
              .toThrow();
          },
        },),
      ],
    },),

    //endregion getTomlProperty

    //region editTomlKey

    describe({
      name: editTomlKey.name,
      children: [
        it({
          name: 'no-op identity edit produces byte-identical output',
          fn: async () => {
            /** Splice mode preserves bytes when the new value matches the existing one */
            const toml = 'name = "Alice"\n';
            expect(editTomlKey({
              content: toml,
              path: ['name',],
              value: 'Alice',
            },),)
              .toBe(toml,);
          },
        },),
        it({
          name: 'updates an existing scalar and preserves surrounding comments',
          fn: async () => {
            /** TOML with a comment that must survive the edit */
            const toml = '# Package metadata\nname = "old"\n# Tail comment\n';
            const result = editTomlKey({
              content: toml,
              path: ['name',],
              value: 'new',
            },);
            expect(result.includes('name = "new"',),).toBe(true,);
            expect(result.includes('# Package metadata',),).toBe(true,);
            expect(result.includes('# Tail comment',),).toBe(true,);
          },
        },),
        it({
          name: 'updates a value nested under a section header',
          fn: async () => {
            const toml = '[settings]\ntheme = "dark"\n';
            const result = editTomlKey({
              content: toml,
              path: ['settings', 'theme',],
              value: 'light',
            },);
            expect(result.includes('theme = "light"',),).toBe(true,);
            expect(result.includes('[settings]',),).toBe(true,);
          },
        },),
        it({
          name: 'creates a missing top-level key',
          fn: async () => {
            const toml = 'existing = "yes"\n';
            const result = editTomlKey({
              content: toml,
              path: ['added',],
              value: 'new-value',
            },);
            expect(result.includes('added = "new-value"',),).toBe(true,);
            expect(result.includes('existing = "yes"',),).toBe(true,);
          },
        },),
        it({
          name: 'creates a missing key inside an existing section',
          fn: async () => {
            const toml = '[settings]\ntheme = "dark"\n';
            const result = editTomlKey({
              content: toml,
              path: ['settings', 'fontSize',],
              value: 14,
            },);
            expect(result.includes('fontSize = 14',),).toBe(true,);
          },
        },),
        it({
          name: 'updates an array-of-tables element',
          fn: async () => {
            const toml = '[[fruits]]\nname = "apple"\n\n[[fruits]]\nname = "orange"\n';
            const result = editTomlKey({
              content: toml,
              path: ['fruits', 0, 'name',],
              value: 'banana',
            },);
            expect(result.includes('name = "banana"',),).toBe(true,);
            expect(result.includes('name = "orange"',),).toBe(true,);
          },
        },),
        it({
          name: 'writes an inline-table-shaped object value',
          fn: async () => {
            const toml = 'placeholder = "x"\n';
            const result = editTomlKey({
              content: toml,
              path: ['pkg',],
              value: { name: 'foo', version: '1.0', },
            },);
            expect(result.includes('pkg',),).toBe(true,);
            expect(result.includes('name',),).toBe(true,);
            expect(result.includes('foo',),).toBe(true,);
          },
        },),
        it({
          name: 'throws on invalid source TOML',
          fn: async () => {
            expect(() =>
              editTomlKey({
                content: '[[unclosed',
                path: ['key',],
                value: 'x',
              },)
            )
              .toThrow();
          },
        },),
      ],
    },),
    //endregion editTomlKey
  ],
},);
