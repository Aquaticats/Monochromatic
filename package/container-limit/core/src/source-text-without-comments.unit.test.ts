import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { stripSourceComments, } from './source-text-without-comments.ts';

/**
 The blanked text of a fixture, with the language whose comment syntax it holds.
 */
function blanked({
  text,
  language,
}: {
  readonly text: string;
  readonly language: 'toml' | 'typescript' | 'yaml';
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
  ],
},);
