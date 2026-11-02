import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  moduleDeclarations,
  referencedNames,
} from './rust-module-scan.ts';
import { tokenizeRust } from './rust-tokens.ts';

/**
 Names a source text refers to through the roots a library crate uses.

 @param source - Rust source text

 @returns the names, sorted

 @example
 ```ts
 namesIn('use super::a::b;');
 ```
 */
function namesIn(source: string): readonly string[] {
  return [...referencedNames({
    tokens: tokenizeRust(source),
    roots: new Set(['crate', 'super', 'lib_name']),
  })].toSorted();
}

await describe({
  name: 'the module scanners',
  children: [
    describe({
      name: moduleDeclarations.name,
      children: [
        it({
          name: 'lists plain, public and restricted-visibility declarations in source order',
          fn: async () => {
            expect(moduleDeclarations(tokenizeRust('mod a; pub mod b; pub(crate) mod c;'))).toEqual([
              { name: 'a' },
              { name: 'b' },
              { name: 'c' },
            ]);
          },
        }),
        it({
          name: 'attaches a path attribute to the declaration that follows it, not to the one after',
          fn: async () => {
            expect(moduleDeclarations(tokenizeRust([
              '#[cfg(test)]',
              '#[path = "a_tests.rs"]',
              'pub(crate) mod tests;',
              'mod plain;',
            ].join('\n')))).toEqual([
              {
                name: 'tests',
                path: 'a_tests.rs',
              },
              { name: 'plain' },
            ]);
          },
        }),
        it({
          name: 'forgets a path attribute that belongs to an item other than a module declaration',
          fn: async () => {
            expect(moduleDeclarations(tokenizeRust('#[path = "x.rs"] fn f() {} mod after;'))).toEqual([{ name: 'after' }]);
          },
        }),
        it({
          name: 'skips inner attributes and does not list inline modules, but lists the declarations inside them',
          fn: async () => {
            expect(moduleDeclarations(tokenizeRust('#![cfg(unix)] mod inline { mod nested; } mod last;'))).toEqual([
              { name: 'nested' },
              { name: 'last' },
            ]);
          },
        }),
        it({
          name: 'ignores the word mod in comments and strings',
          fn: async () => {
            expect(moduleDeclarations(tokenizeRust('// mod a;\n/* mod b; */ let s = "mod c;"; mod d;'))).toEqual([{ name: 'd' }]);
          },
        }),
      ],
    }),
    describe({
      name: referencedNames.name,
      children: [
        it({
          name: 'names the segment after crate, super and the library root, and after repeated super',
          fn: async () => {
            expect(namesIn('use crate::a::f; use super::b::g; use super::super::c; let x = lib_name::d::h();')).toEqual([
              'a',
              'b',
              'c',
              'd',
            ]);
          },
        }),
        it({
          name: 'names the first segment of every item of a use group, and not what is nested inside an item',
          fn: async () => {
            expect(namesIn('use crate::{a::{x, y}, b, c::d as e, self};')).toEqual([
              'a',
              'b',
              'c',
            ]);
            expect(namesIn('use super::{Answer, answer_with};')).toEqual(['Answer', 'answer_with']);
          },
        }),
        it({
          name: 'ignores paths that start elsewhere, glob imports and the words in comments and strings',
          fn: async () => {
            expect(namesIn('use std::fs::read; use other::a; use super::*; // crate::hidden\nlet s = "crate::text";')).toEqual([]);
          },
        }),
        it({
          name: 'finds a path inside a function body and a macro call',
          fn: async () => {
            expect(namesIn('fn f() { let r = crate::inner::call(); assert!(super::sibling::check()); }')).toEqual([
              'inner',
              'sibling',
            ]);
          },
        }),
      ],
    }),
  ],
});
