import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  dedup,
  getJsonProperty,
} from '../../dist/final/node/index.mjs';

await describe({
  name: '',
  children: [
    //region dedup

    describe({
      name: dedup.name,
      children: [
        it({
          name: 'removes duplicate lines preserving first occurrence order',
          fn: async () => {
            expect(dedup('a\nb\na\nc\nb',),).toBe('a\nb\nc',);
          },
        },),
        it({
          name: 'returns empty string unchanged',
          fn: async () => {
            expect(dedup('',),).toBe('',);
          },
        },),
        it({
          name: 'returns single line unchanged',
          fn: async () => {
            expect(dedup('only-line',),).toBe('only-line',);
          },
        },),
        it({
          name: 'preserves all lines when none are duplicated',
          fn: async () => {
            expect(dedup('alpha\nbeta\ngamma',),).toBe('alpha\nbeta\ngamma',);
          },
        },),
        it({
          name: 'handles consecutive duplicate lines',
          fn: async () => {
            expect(dedup('x\nx\nx\ny',),).toBe('x\ny',);
          },
        },),
        it({
          name: 'treats lines with different whitespace as distinct',
          fn: async () => {
            expect(dedup('  a\na\n a',),).toBe('  a\na\n a',);
          },
        },),
        it({
          name: 'preserves blank lines but deduplicates them',
          fn: async () => {
            expect(dedup('a\n\nb\n\nc',),).toBe('a\n\nb\nc',);
          },
        },),
      ],
    },),

    //endregion dedup

    //region getJsonProperty

    describe({
      name: getJsonProperty.name,
      children: [
        it({
          name: 'extracts a top-level string property',
          fn: async () => {
            /** JSON with a simple string value */
            const json = JSON.stringify({ name: 'Alice', },);
            expect(getJsonProperty({ path: ['name',], content: json, },),).toBe('Alice',);
          },
        },),
        it({
          name: 'extracts a nested property via array path',
          fn: async () => {
            /** JSON with nested structure */
            const json = JSON.stringify({ settings: { theme: 'dark', }, },);
            expect(getJsonProperty({
              path: ['settings', 'theme',],
              content: json,
            },),)
              .toBe('dark',);
          },
        },),
        it({
          name: 'stringifies non-string values (number)',
          fn: async () => {
            /** JSON with a numeric value that should be stringified */
            const json = JSON.stringify({ count: 42, },);
            expect(getJsonProperty({ path: ['count',], content: json, },),).toBe('42',);
          },
        },),
        it({
          name: 'stringifies array values with pretty printing',
          fn: async () => {
            /** JSON with an array value */
            const json = JSON.stringify({ items: [1, 2, 3,], },);
            expect(getJsonProperty({ path: ['items',], content: json, },),).toBe(
              JSON.stringify([1, 2, 3,], null, 2,),
            );
          },
        },),
        it({
          name: 'stringifies object values with pretty printing',
          fn: async () => {
            /** JSON with a nested object to extract */
            const json = JSON.stringify({ nested: { a: 1, b: 2, }, },);
            expect(getJsonProperty({ path: ['nested',], content: json, },),).toBe(
              JSON.stringify({ a: 1, b: 2, }, null, 2,),
            );
          },
        },),
        it({
          name: 'returns undefined for missing paths',
          fn: async () => {
            /** JSON where the requested path does not exist */
            const json = JSON.stringify({ exists: true, },);
            // dot-prop returns undefined for missing keys; getJsonProperty passes it through
            expect(getJsonProperty({
              path: ['missing',],
              content: json,
            },),)
              .toBeUndefined();
          },
        },),
        it({
          name: 'throws on invalid JSON input',
          fn: async () => {
            expect(() => getJsonProperty({ path: ['key',], content: 'not-json', },))
              .toThrow();
          },
        },),
        it({
          name: 'handles boolean values',
          fn: async () => {
            /** JSON with a boolean value */
            const json = JSON.stringify({ active: true, },);
            expect(getJsonProperty({ path: ['active',], content: json, },),).toBe(
              'true',
            );
          },
        },),
        it({
          name: 'handles null values',
          fn: async () => {
            /** JSON with an explicit null */
            const json = JSON.stringify({ value: null, },);
            expect(getJsonProperty({ path: ['value',], content: json, },),).toBe('null',);
          },
        },),
        it({
          name: 'handles deeply nested paths',
          fn: async () => {
            /** JSON with three levels of nesting */
            const json = JSON.stringify({ a: { b: { c: 'deep', }, }, },);
            expect(getJsonProperty({
              path: ['a', 'b', 'c',],
              content: json,
            },),)
              .toBe('deep',);
          },
        },),
        it({
          name: 'indexes into arrays by numeric segment',
          fn: async () => {
            /** JSON with an array of objects */
            const json = JSON.stringify({
              items: [{ name: 'first', }, { name: 'second', },],
            },);
            expect(getJsonProperty({
              path: ['items', 0, 'name',],
              content: json,
            },),)
              .toBe('first',);
          },
        },),
        it({
          name: 'addresses keys containing literal dots via array segments',
          fn: async () => {
            /** JSON where a single key contains a literal dot */
            const json = JSON.stringify({ 'a.b': 'value', },);
            /** Array form treats `'a.b'` as one segment, avoiding the dot-as-separator parse */
            expect(getJsonProperty({ path: ['a.b',], content: json, },),).toBe('value',);
          },
        },),
      ],
    },),
    //endregion getJsonProperty
  ],
},);
