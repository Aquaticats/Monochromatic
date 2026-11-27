import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { blankRustComments, } from './rust-comments-in-source-text.ts';

/**
 A Rust source with its comments blanked, as the scan of a Rust file reads it.

 @param text - the fixture

 @returns the text with every comment replaced by blank space

 @example
 ```ts
 const blank = blanked('fn f() {} // podman run');
 ```
 */
function blanked(text: string,): string {
  /**
   The file's characters, which blanking changes in place.
   */
  const characters = text.split('',);
  blankRustComments(characters,);
  return characters.join('',);
}

await describe({
  name: blankRustComments.name,
  children: [
    it({
      name: 'blanks a line comment and a doc comment, of all three kinds',
      fn: async () => {
        const blank = blanked('let x = 1; // podman run\n/// podman run\nfn f() {}\n//! podman run\n',);
        expect(blank.includes('podman',),).toBe(false,);
        expect(blank.includes('fn f() {}',),).toBe(true,);
      },
    },),
    it({
      name: 'blanks a block comment whole, including the comment nested inside it',
      fn: async () => {
        const blank = blanked('/* a /* b */ podman run */ x',);
        expect(blank.includes('podman',),).toBe(false,);
        expect(blank.endsWith('x',),).toBe(true,);
      },
    },),
    it({
      name: 'keeps a raw string whole, so the comment opener in it blanks nothing after it',
      fn: async () => {
        const blank = blanked('const NOTE: &str = r#"a "quote, and /* never a comment"#;\nfn f() {}',);
        expect(blank.includes('/* never a comment',),).toBe(true,);
        expect(blank.endsWith('fn f() {}',),).toBe(true,);
      },
    },),
    it({
      name: 'keeps a raw string with three marks whole, and blanks the comment after it',
      fn: async () => {
        const blank = blanked('const Q: &str = r###"a #" b // c"###; // podman run\nlet y = 2;',);
        expect(blank.includes('r###"a #" b // c"###',),).toBe(true,);
        expect(blank.includes('podman',),).toBe(false,);
        expect(blank.endsWith('let y = 2;',),).toBe(true,);
      },
    },),
    it({
      name: 'reads a char literal holding a quote as one character, not as the start of a string',
      fn: async () => {
        const blank = blanked('const Q: char = \'"\'; // podman run\nlet y = 2;',);
        expect(blank.includes('podman',),).toBe(false,);
        expect(blank.endsWith('let y = 2;',),).toBe(true,);
      },
    },),
    it({
      name: 'reads a lifetime as code, so the comment after it is still blanked',
      fn: async () => {
        const blank = blanked('fn f<\'static>(x: &\'static str) -> &\'static str { x }\n// podman run\nlet y = 2;',);
        expect(blank.includes('podman',),).toBe(false,);
        expect(blank.endsWith('let y = 2;',),).toBe(true,);
      },
    },),
    it({
      name: 'keeps every offset and line break of the file it blanks',
      fn: async () => {
        const text = 'let x = 1; // podman run\nfn f() {}\n/* podman\nrun */\nlet y = 2;';
        const blank = blanked(text,);
        expect(blank.length,).toBe(text.length,);
        expect(blank.split('\n',),).toHaveLength(text.split('\n',).length,);
      },
    },),
  ],
},);
