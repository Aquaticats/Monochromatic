import { types, } from '@monochromatic-dev/module-es';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

const { $, } = types.string.from.array.string.concat.with.string.sync.named;

await describe({
  name: $.name,
  children: [
    it({
      name: 'joins multiple strings with a separator',
      fn: async () => {
        expect(
          $({ strings: ['Hello', 'World',], concatWith: ', ', },),
        )
          .toBe(
            'Hello, World',
          );
      },
    },),

    it({
      name: 'joins three strings with a separator',
      fn: async () => {
        expect(
          $({ strings: ['a', 'b', 'c',], concatWith: '-', },),
        )
          .toBe(
            'a-b-c',
          );
      },
    },),

    it({
      name: 'empty array returns empty string',
      fn: async () => {
        expect($({ strings: [], concatWith: ',', },),).toBe('',);
      },
    },),

    it({
      name: 'single element array returns that element',
      fn: async () => {
        expect($({ strings: ['single',], concatWith: ',', },),).toBe('single',);
      },
    },),

    it({
      name: 'empty string separator joins without separator',
      fn: async () => {
        expect($({ strings: ['a', 'b', 'c',], concatWith: '', },),).toBe('abc',);
      },
    },),

    it({
      name: 'single character separator',
      fn: async () => {
        expect($({ strings: ['one', 'two', 'three',], concatWith: ' ', },),).toBe(
          'one two three',
        );
      },
    },),

    it({
      name: 'multi-character separator',
      fn: async () => {
        expect(
          $({ strings: ['first', 'second', 'third',], concatWith: ' - ', },),
        )
          .toBe(
            'first - second - third',
          );
      },
    },),

    it({
      name: 'whitespace separator',
      fn: async () => {
        expect($({ strings: ['1', '2', '3',], concatWith: '  ', },),).toBe('1  2  3',);
      },
    },),

    it({
      name: 'special characters as separators',
      fn: async () => {
        expect($({ strings: ['a', 'b', 'c',], concatWith: '|||', },),).toBe('a|||b|||c',);
      },
    },),

    it({
      name: 'newline as separator',
      fn: async () => {
        expect($({ strings: ['line1', 'line2', 'line3',], concatWith: '\n', },),).toBe(
          'line1\nline2\nline3',
        );
      },
    },),

    it({
      name: 'tab as separator',
      fn: async () => {
        expect($({ strings: ['col1', 'col2', 'col3',], concatWith: '\t', },),).toBe(
          'col1\tcol2\tcol3',
        );
      },
    },),

    it({
      name: 'handles empty strings in the array',
      fn: async () => {
        expect($({ strings: ['a', '', 'c',], concatWith: ',', },),).toBe('a,,c',);
      },
    },),

    it({
      name: 'handles multiple empty strings',
      fn: async () => {
        expect(
          $({ strings: ['', '', '',], concatWith: '-', },),
        )
          .toBe(
            '--',
          );
      },
    },),

    it({
      name: 'handles strings with spaces',
      fn: async () => {
        expect(
          $({ strings: ['foo bar', 'baz qux',], concatWith: ', ', },),
        )
          .toBe(
            'foo bar, baz qux',
          );
      },
    },),

    it({
      name: 'handles unicode characters',
      fn: async () => {
        expect(
          $({ strings: ['世界', '你好', '世界',], concatWith: ' ', },),
        )
          .toBe(
            '世界 你好 世界',
          );
      },
    },),

    it({
      name: 'handles emoji characters',
      fn: async () => {
        expect($({ strings: ['🚀', '🌟', '✨',], concatWith: ' ', },),).toBe('🚀 🌟 ✨',);
      },
    },),

    it({
      name: 'handles special regex characters as strings',
      fn: async () => {
        expect(
          $({ strings: ['test.*', 'pattern+', 'more?',], concatWith: '|', },),
        )
          .toBe(
            'test.*|pattern+|more?',
          );
      },
    },),

    it({
      name: 'handles numbers in array',
      fn: async () => {
        expect($({ strings: ['1', '2', '3',], concatWith: '-', },),).toBe('1-2-3',);
      },
    },),

    it({
      name: 'handles mixed content',
      fn: async () => {
        expect(
          $({ strings: ['Hello', '123', 'World',], concatWith: ' ', },),
        )
          .toBe(
            'Hello 123 World',
          );
      },
    },),

    it({
      name: 'handles very long arrays efficiently',
      fn: async () => {
        const largeArray = Array.from({ length: 100, }, (_, index,) => `item${index}`,);
        const result = $({ strings: largeArray, concatWith: ',', },);
        expect(result.startsWith('item0,item1,item2',),).toBe(true,);
        expect(result,).toContain('item50,',);
        expect(result,).toContain(',item99',);
      },
    },),

    it({
      name: 'handles very long strings',
      fn: async () => {
        const longString1 = 'a'.repeat(1_000,);
        const longString2 = 'b'.repeat(1_000,);
        const result = $({ strings: [longString1, longString2,], concatWith: '|', },);
        expect(result,).toBe(`${longString1}|${longString2}`,);
      },
    },),

    it({
      name: 'handles dot separator',
      fn: async () => {
        expect($({ strings: ['com', 'example', 'www',], concatWith: '.', },),).toBe(
          'com.example.www',
        );
      },
    },),

    it({
      name: 'handles slash separator',
      fn: async () => {
        expect($({ strings: ['home', 'user', 'documents',], concatWith: '/', },),).toBe(
          'home/user/documents',
        );
      },
    },),

    it({
      name: 'handles backslash separator',
      fn: async () => {
        expect($({ strings: ['C:', 'Users', 'Documents',], concatWith: '\\', },),).toBe(
          String.raw`C:\Users\Documents`,
        );
      },
    },),

    it({
      name: 'handles colon separator',
      fn: async () => {
        expect(
          $({ strings: ['key', 'value', 'extra',], concatWith: ':', },),
        )
          .toBe(
            'key:value:extra',
          );
      },
    },),

    it({
      name: 'handles semicolon separator',
      fn: async () => {
        expect(
          $({ strings: ['red', 'green', 'blue',], concatWith: ';', },),
        )
          .toBe(
            'red;green;blue',
          );
      },
    },),

    it({
      name: 'handles pipe separator',
      fn: async () => {
        expect($({ strings: ['A', 'B', 'C',], concatWith: '|', },),).toBe('A|B|C',);
      },
    },),

    it({
      name: 'handles ampersand separator',
      fn: async () => {
        expect($({ strings: ['foo', 'bar', 'baz',], concatWith: '&', },),).toBe(
          'foo&bar&baz',
        );
      },
    },),

    it({
      name: 'handles dash separator',
      fn: async () => {
        expect($({ strings: ['first', 'second', 'third',], concatWith: '-', },),).toBe(
          'first-second-third',
        );
      },
    },),

    it({
      name: 'handles underscore separator',
      fn: async () => {
        expect(
          $({ strings: ['variable', 'name', 'here',], concatWith: '_', },),
        )
          .toBe(
            'variable_name_here',
          );
      },
    },),

    it({
      name: 'handles equals sign separator',
      fn: async () => {
        expect($({ strings: ['key', 'value',], concatWith: '=', },),).toBe('key=value',);
      },
    },),
  ],
},);
