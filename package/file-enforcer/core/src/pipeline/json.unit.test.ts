import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  formatJsonObject,
  isJsonObject,
  mergeFlatJson,
  mergeObjectDefaults,
  omitJsonKey,
  parseJsonObject,
} from './json.ts';

await describe({
  name: '',
  children: [
    //region isJsonObject

    describe({
      name: isJsonObject.name,
      children: [
        it({
          name: 'accepts a plain object',
          fn: async () => {
            expect(isJsonObject({ a: 1, },),).toBe(true,);
          },
        },),
        it({
          name: 'rejects arrays, null, and primitives',
          fn: async () => {
            expect(isJsonObject([],),).toBe(false,);
            expect(isJsonObject(null,),).toBe(false,);
            expect(isJsonObject('s',),).toBe(false,);
          },
        },),
      ],
    },),

    //endregion isJsonObject

    //region parseJsonObject

    describe({
      name: parseJsonObject.name,
      children: [
        it({
          name: 'parses object text',
          fn: async () => {
            expect(parseJsonObject({ content: '{"a":1}', label: 'x', },),).toEqual({ a: 1, },);
          },
        },),
        it({
          name: 'throws when content is an array',
          fn: async () => {
            expect(() => parseJsonObject({ content: '[1]', label: 'x', },),).toThrow();
          },
        },),
        it({
          name: 'throws when content is invalid JSON',
          fn: async () => {
            expect(() => parseJsonObject({ content: 'nope', label: 'x', },),).toThrow();
          },
        },),
      ],
    },),

    //endregion parseJsonObject

    //region formatJsonObject

    describe({
      name: formatJsonObject.name,
      children: [
        it({
          name: 'formats with two-space indent and no trailing newline',
          fn: async () => {
            expect(formatJsonObject({ value: { a: 1, }, },),).toBe('{\n  "a": 1\n}',);
          },
        },),
      ],
    },),

    //endregion formatJsonObject

    //region omitJsonKey

    describe({
      name: omitJsonKey.name,
      children: [
        it({
          name: 'removes a key while preserving remaining order',
          fn: async () => {
            expect(formatJsonObject({ value: omitJsonKey({ object: { a: 1, b: 2, c: 3, }, key: 'b', }, ), },),)
              .toBe(formatJsonObject({ value: { a: 1, c: 3, }, },),);
          },
        },),
        it({
          name: 'is a no-op for an absent key',
          fn: async () => {
            expect(omitJsonKey({ object: { a: 1, }, key: 'z', },),).toEqual({ a: 1, },);
          },
        },),
      ],
    },),

    //endregion omitJsonKey

    //region mergeFlatJson

    describe({
      name: mergeFlatJson.name,
      children: [
        it({
          name: 'updates existing keys in place and appends new keys in order',
          fn: async () => {
            expect(formatJsonObject({
              value: mergeFlatJson({ base: { a: 1, b: 2, }, set: { b: false, c: true, }, },),
            },),)
              .toBe(formatJsonObject({ value: { a: 1, b: false, c: true, }, },),);
          },
        },),
        it({
          name: 'unions array members, keeping existing strings then deduplicated additions',
          fn: async () => {
            expect(mergeFlatJson({
              base: { list: ['a', 'b',], },
              arrayUnion: { list: ['b', 'c',], },
            },),)
              .toEqual({ list: ['a', 'b', 'c',], },);
          },
        },),
        it({
          name: 'drops non-string members from an existing array before union',
          fn: async () => {
            expect(mergeFlatJson({
              base: { list: ['a', 7, true,], },
              arrayUnion: { list: ['b',], },
            },),)
              .toEqual({ list: ['a', 'b',], },);
          },
        },),
        it({
          name: 'treats a non-array existing value as empty for union',
          fn: async () => {
            expect(mergeFlatJson({ base: { list: 'scalar', }, arrayUnion: { list: ['x',], }, },),)
              .toEqual({ list: ['x',], },);
          },
        },),
      ],
    },),

    //endregion mergeFlatJson

    //region mergeObjectDefaults

    describe({
      name: mergeObjectDefaults.name,
      children: [
        it({
          name: 'adds only absent keys and keeps existing values',
          fn: async () => {
            expect(mergeObjectDefaults({ base: { a: 1, }, defaults: { a: 9, b: 2, }, },),)
              .toEqual({ a: 1, b: 2, },);
          },
        },),
      ],
    },),

    //endregion mergeObjectDefaults
  ],
},);
