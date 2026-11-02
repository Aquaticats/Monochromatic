import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type RustToken,
  tokenizeRust,
} from './rust-tokens.ts';

/**
 Render tokens as `kind:text` words, so a test reads one line.

 @param tokens - tokens to render

 @returns the words

 @example
 ```ts
 words(tokenizeRust('a::b'));
 ```
 */
function words(tokens: readonly RustToken[]): readonly string[] {
  return tokens.map(function word(token) {
    return `${token.kind}:${token.text}`;
  });
}

await describe({
  name: tokenizeRust.name,
  children: [
    it({
      name: 'splits identifiers, the path separator, punctuation and numbers',
      fn: async () => {
        expect(words(tokenizeRust('use crate::a::{b, c};'))).toEqual([
          'ident:use',
          'ident:crate',
          'punct:::',
          'ident:a',
          'punct:::',
          'punct:{',
          'ident:b',
          'punct:,',
          'ident:c',
          'punct:}',
          'punct:;',
        ]);
        expect(words(tokenizeRust('x = 12u8;'))).toEqual([
          'ident:x',
          'punct:=',
          'number:12u8',
          'punct:;',
        ]);
      },
    }),
    it({
      name: 'drops line comments, block comments and nested block comments, and the paths inside them',
      fn: async () => {
        expect(words(tokenizeRust('a // use crate::x;\nb /* use crate::y; /* nested */ still comment */ c'))).toEqual([
          'ident:a',
          'ident:b',
          'ident:c',
        ]);
      },
    }),
    it({
      name: 'reads a string literal as one token, so comment markers and paths inside it are not code',
      fn: async () => {
        expect(words(tokenizeRust('#[path = "x//y.rs"] "a\\"b" mod'))).toEqual([
          'punct:#',
          'punct:[',
          'ident:path',
          'punct:=',
          'string:x//y.rs',
          'punct:]',
          'string:a\\"b',
          'ident:mod',
        ]);
      },
    }),
    it({
      name: 'reads raw strings with and without hashes, and byte strings, as single tokens',
      fn: async () => {
        expect(words(tokenizeRust('r"a\\" r#"b "quoted" use crate::z;"# b"c" br##"d"##'))).toEqual([
          'string:a\\',
          'string:b "quoted" use crate::z;',
          'string:c',
          'string:d',
        ]);
      },
    }),
    it({
      name: 'reads a raw identifier as its name',
      fn: async () => {
        expect(words(tokenizeRust('r#type'))).toEqual(['ident:type']);
      },
    }),
    it({
      name: 'drops character literals, byte characters and lifetimes without swallowing the code after them',
      fn: async () => {
        expect(words(tokenizeRust('\'a\' \'\\n\' \'\\u{41}\' b\'x\' \'\\\'\' fn f<\'a>(x: &\'a str) {}'))).toEqual([
          'ident:fn',
          'ident:f',
          'punct:<',
          'punct:>',
          'punct:(',
          'ident:x',
          'punct::',
          'punct:&',
          'ident:str',
          'punct:)',
          'punct:{',
          'punct:}',
        ]);
      },
    }),
    it({
      name: 'ends at an unterminated string or comment instead of looping',
      fn: async () => {
        expect(words(tokenizeRust('a "never closed'))).toEqual(['ident:a', 'string:never closed']);
        expect(words(tokenizeRust('b /* never closed'))).toEqual(['ident:b']);
      },
    }),
  ],
});
