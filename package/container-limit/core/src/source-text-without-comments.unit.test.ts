import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  stripSourceComments,
  type SourceLanguage,
} from './source-text-without-comments.ts';

/**
 The blanked text of a fixture, with the language whose comment syntax it holds.
 */
function blanked({
  text,
  language,
}: {
  readonly text: string;
  readonly language: SourceLanguage;
},): string {
  return stripSourceComments({
    text,
    language,
  },);
}

await describe({
  name: stripSourceComments.name,
  children: [
    it({
      name: 'blanks a line comment, keeps the line after it, and keeps every offset',
      fn: async () => {
        const text = 'a // podman run\nb';
        const blank = blanked({
          text,
          language: 'typescript',
        },);
        expect(blank.length,).toBe(text.length,);
        expect(blank.includes('podman',),).toBe(false,);
        expect(blank.endsWith('b',),).toBe(true,);
      },
    },),
    it({
      name: 'blanks a block comment and keeps what follows it',
      fn: async () => {
        const blank = blanked({
          text: 'a /* podman */ b',
          language: 'typescript',
        },);
        expect(blank.includes('podman',),).toBe(false,);
        expect(blank.endsWith('b',),).toBe(true,);
      },
    },),
    it({
      name: 'keeps the text of a quoted string',
      fn: async () => {
        const blank = blanked({
          text: "spawn('podman', ['run']) // x",
          language: 'typescript',
        },);
        expect(blank.includes("spawn('podman', ['run'])",),).toBe(true,);
        expect(blank.includes('// x',),).toBe(false,);
      },
    },),
    it({
      name: 'blanks a TOML comment and keeps a hash inside a string',
      fn: async () => {
        const blank = blanked({
          text: 'run = "a #b" # podman run\nx = 1',
          language: 'toml',
        },);
        expect(blank.includes('"a #b"',),).toBe(true,);
        expect(blank.includes('podman',),).toBe(false,);
        expect(blank.endsWith('x = 1',),).toBe(true,);
      },
    },),
    it({
      name: 'blanks the node script comments of a TOML multi-line string',
      fn: async () => {
        const blank = blanked({
          text: 'x = """\n// podman run\ny\n"""',
          language: 'toml',
        },);
        expect(blank.includes('podman',),).toBe(false,);
        expect(blank.includes('\ny\n',),).toBe(true,);
      },
    },),
    it({
      name: 'reads an apostrophe in prose as one character, not the start of a string',
      fn: async () => {
        const blank = blanked({
          text: "the program's license # podman run\nkeep",
          language: 'yaml',
        },);
        expect(blank.includes('podman',),).toBe(false,);
        expect(blank.endsWith('\nkeep',),).toBe(true,);
      },
    },),
    it({
      name: 'blanks a YAML comment and keeps a hash inside a quoted scalar',
      fn: async () => {
        const blank = blanked({
          text: 'run: "a #b" # podman run',
          language: 'yaml',
        },);
        expect(blank.includes('"a #b"',),).toBe(true,);
        expect(blank.includes('podman',),).toBe(false,);
      },
    },),

    //region Rust comments

    it({
      name: 'blanks a Rust line comment and a Rust doc comment, of all three kinds',
      fn: async () => {
        const blank = blanked({
          text: 'let x = 1; // podman run\n/// podman run\nfn f() {}\n//! podman run\n',
          language: 'rust',
        },);
        expect(blank.includes('podman',),).toBe(false,);
        expect(blank.includes('fn f() {}',),).toBe(true,);
      },
    },),
    it({
      name: 'blanks a Rust block comment whole, including the comment nested inside it',
      fn: async () => {
        const blank = blanked({
          text: '/* a /* b */ podman run */ x',
          language: 'rust',
        },);
        expect(blank.includes('podman',),).toBe(false,);
        expect(blank.endsWith('x',),).toBe(true,);
      },
    },),
    it({
      name: 'keeps a Rust raw string whole, so the comment opener in it blanks nothing after it',
      fn: async () => {
        const blank = blanked({
          text: 'const NOTE: &str = r#"a "quote, and /* never a comment"#;\nfn f() {}',
          language: 'rust',
        },);
        expect(blank.includes('/* never a comment',),).toBe(true,);
        expect(blank.endsWith('fn f() {}',),).toBe(true,);
      },
    },),
    it({
      name: 'keeps a Rust raw string with three hashes whole, and blanks the comment after it',
      fn: async () => {
        const blank = blanked({
          text: 'const Q: &str = r###"a #" b // c"###; // podman run\nlet y = 2;',
          language: 'rust',
        },);
        expect(blank.includes('r###"a #" b // c"###',),).toBe(true,);
        expect(blank.includes('podman',),).toBe(false,);
        expect(blank.endsWith('let y = 2;',),).toBe(true,);
      },
    },),
    it({
      name: 'reads a Rust char literal holding a quote as one character, not as the start of a string',
      fn: async () => {
        const blank = blanked({
          text: 'const Q: char = \'"\'; // podman run\nlet y = 2;',
          language: 'rust',
        },);
        expect(blank.includes('podman',),).toBe(false,);
        expect(blank.endsWith('let y = 2;',),).toBe(true,);
      },
    },),
    it({
      name: 'reads a Rust lifetime as code, so the comment after it is still blanked',
      fn: async () => {
        const blank = blanked({
          text: 'fn f<\'static>(x: &\'static str) -> &\'static str { x }\n// podman run\nlet y = 2;',
          language: 'rust',
        },);
        expect(blank.includes('podman',),).toBe(false,);
        expect(blank.endsWith('let y = 2;',),).toBe(true,);
      },
    },),

    //endregion Rust comments
  ],
},);
