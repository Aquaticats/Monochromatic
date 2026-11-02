import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  everyModule,
  moduleDeclarations,
  referencedNames,
} from './rust-module-scan.ts';
import { tokenizeRust } from './rust-tokens.ts';

/**
 Names a source text refers to through the roots a library crate uses.

 @param source - Rust source text

 @param depth - module nesting of the file below the crate root

 @returns the names, sorted

 @example
 ```ts
 namesIn('use super::a::b;');
 ```
 */
function namesIn(source: string, depth = 1): readonly string[] {
  return [...referencedNames({
    tokens: tokenizeRust(source),
    roots: new Set(['crate', 'super', 'lib_name']),
    depth,
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
            expect(
              moduleDeclarations(tokenizeRust('mod a; pub mod b; pub(crate) mod c;')),
            ).toEqual([
              { name: 'a' },
              { name: 'b' },
              { name: 'c' },
            ]);
          },
        }),
        it({
          name: 'attaches a path attribute to the declaration that follows it, not to the one after',
          fn: async () => {
            expect(
              moduleDeclarations(
                tokenizeRust([
                  '#[cfg(test)]',
                  '#[path = "a_tests.rs"]',
                  'pub(crate) mod tests;',
                  'mod plain;',
                ].join('\n')),
              ),
            ).toEqual([
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
            expect(
              moduleDeclarations(tokenizeRust('#[path = "x.rs"] fn f() {} mod after;')),
            ).toEqual([{ name: 'after' }]);
          },
        }),
        it({
          name: 'skips inner attributes and does not list inline modules, but lists the declarations inside them with their enclosing modules',
          fn: async () => {
            expect(
              moduleDeclarations(tokenizeRust('#![cfg(unix)] mod inline { mod nested; } mod last;')),
            ).toEqual([
              { name: 'nested' },
              { name: 'last' },
            ]);
          },
        }),
        it({
          name: 'keeps the enclosing inline modules of a declaration, however deep, and forgets them at the closing brace',
          fn: async () => {
            expect(moduleDeclarations(tokenizeRust('pub mod outer { fn f() { let x = 1; } pub mod inner; mod deep { mod x; } } mod after;'))).toEqual([
              {
                name: 'inner',
                inline: ['outer'],
              },
              {
                name: 'x',
                inline: ['outer', 'deep'],
              },
              { name: 'after' },
            ]);
          },
        }),
        it({
          name: 'does not count brackets or braces inside string literals as structure',
          fn: async () => {
            expect(moduleDeclarations(tokenizeRust('#[doc = "["] pub mod a; mod b { fn f() { let s = "}"; } mod c; } mod d;'))).toEqual([
              { name: 'a' },
              {
                name: 'c',
                inline: ['b'],
              },
              { name: 'd' },
            ]);
          },
        }),
        it({
          name: 'lists the paths of cfg_attr path attributes as conditional paths, beside the default file',
          fn: async () => {
            expect(moduleDeclarations(tokenizeRust([
              '#[cfg_attr(unix, path = "a_unix.rs")]',
              '#[cfg_attr(windows, path = "a_windows.rs")]',
              'pub mod a;',
            ].join('\n')))).toEqual([
              {
                name: 'a',
                conditionalPaths: ['a_unix.rs', 'a_windows.rs'],
              },
            ]);
          },
        }),
        it({
          name: 'ignores the word mod in comments and strings',
          fn: async () => {
            expect(
              moduleDeclarations(tokenizeRust('// mod a;\n/* mod b; */ let s = "mod c;"; mod d;')),
            ).toEqual([{ name: 'd' }]);
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
          name: 'treats a glob of the crate root as naming every identifier of the file',
          fn: async () => {
            expect(namesIn('use crate::*; fn f() { a::g(); other::h(); }')).toContain('other');
            expect(namesIn('use crate::*; fn f() { a::g(); }')).toContain('a');
            expect(namesIn('use lib_name::*; fn f() { b::g(); }')).toContain('b');
          },
        }),
        it({
          name: 'treats super::* as the crate root only where super is the root, and not in a nested module',
          fn: async () => {
            expect(namesIn('use super::*; fn f() { a::g(); }', 1)).toContain('a');
            expect(namesIn('use super::*; fn f() { a::g(); }', 2)).toEqual([]);
            expect(namesIn('use super::super::*; fn f() { a::g(); }', 2)).toContain('a');
          },
        }),
        it({
          name: 'treats include! as reaching every module, because the included file is not read',
          fn: async () => {
            expect(namesIn('include!("body.rs");')).toEqual([everyModule]);
            expect(namesIn('include_str!("body.rs");')).toEqual([]);
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
