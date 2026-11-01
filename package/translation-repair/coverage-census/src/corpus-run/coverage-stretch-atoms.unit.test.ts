/**
 Tests the tokens the coverage census reads a cold stretch's bundle text by
 (ledger T8), in one linear pass: a word, a string or template literal read
 whole, a bracketed group read whole through everything it nests, any other
 character as a mark, blank space and comments skipped, and the places the
 pass refuses to read for certain, which keep a stretch cold. The texts are
 written as the unminified coverage build writes code; names are cat-themed
 invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { atomAt, } from '../../dist/final/node/index.mjs';

/**
 Every token of a text in order, each as its kind and what it is written
 as, ending with the token that ended the read.

 @param text - text read from its first character

 @returns Each token's kind and text, then `['end']` or `['unreadable']`
 */
function atomsOf({ text, }: { readonly text: string; },): readonly (readonly string[])[] {
  const read: (readonly string[])[] = [];
  for (let from = 0; from <= text.length;) {
    const atom = atomAt({
      text,
      from,
    },);
    if ((atom.kind === 'end') || (atom.kind === 'unreadable')) {
      read.push([atom.kind,],);
      break;
    }
    read.push([
      atom.kind,
      text.slice(
        atom.start,
        atom.end,
      ),
    ],);
    from = atom.end;
  }
  return read;
}

await describe({
  name: atomAt.name,
  children: [
    it({
      name: 'READS A WORD, A BRACKETED GROUP AS ONE TOKEN THROUGH WHAT IT NESTS, AND ANY OTHER CHARACTER AS A MARK, a '
        + 'closing bracket with nothing open among them, with each token\'s offsets',
      fn: async () => {
        expect(atomAt({
          text: 'throw new Error("nap")',
          from: 5,
        },),).toEqual({
          kind: 'word',
          start: 6,
          end: 9,
        },);
        expect(atomsOf({ text: 'throw new Error("a ) cat", { cause: paws[0] }) ;', },),).toEqual([
          ['word', 'throw',],
          ['word', 'new',],
          ['word', 'Error',],
          ['group', '("a ) cat", { cause: paws[0] })',],
          ['mark', ';',],
          ['end',],
        ],);
        expect(atomsOf({ text: `} else if (lap[0] === \`\${cat}\`) { a ) b ] c`, },),).toEqual([
          ['mark', '}',],
          ['word', 'else',],
          ['word', 'if',],
          ['group', `(lap[0] === \`\${cat}\`)`,],
          ['mark', '{',],
          ['word', 'a',],
          ['mark', ')',],
          ['word', 'b',],
          ['mark', ']',],
          ['word', 'c',],
          ['end',],
        ],);
      },
    },),
    it({
      name: 'READS A STRING AND A TEMPLATE WHOLE: a quote behind a backslash, the other kind of quote, a substitution '
        + 'holding a string with a closing brace in it, a template nested in a substitution, and a template mark '
        + 'behind a backslash',
      fn: async () => {
        expect(atomsOf({ text: String.raw`"a \" cat's" + 'a "lap"'`, },),).toEqual([
          ['text', String.raw`"a \" cat's"`,],
          ['mark', '+',],
          ['text', '\'a "lap"\'',],
          ['end',],
        ],);
        /**
         A template as a bundle writes one: two substitutions, the second
         holding a template of its own, then a template mark and a
         substitution's opening each behind a backslash.
         */
        const template = `\`nap \${paw("}")} of \${\`cat \${name}\`} \\\` and \\\${not} one\``;
        expect(atomsOf({ text: `${template} done`, },),).toEqual([
          ['text', template,],
          ['word', 'done',],
          ['end',],
        ],);
      },
    },),
    it({
      name: 'SKIPS BLANK SPACE AND COMMENTS before a token and inside a group, and reads the end where nothing else '
        + 'is left; A LINE COMMENT ENDS AT ANY LINE END, so nothing after one hides in it, and the two rarer line '
        + 'ends, which are no blank space here, are read as marks',
      fn: async () => {
        expect([
          ' \t/* nap ) " ` */ // doze ( \' `\n throw',
          '// doze\r throw',
          '// doze\u2028throw',
          '// doze\u2029throw',
        ].map((text,) => atomsOf({ text, },)),).toEqual([
          [['word', 'throw',], ['end',],],
          [['word', 'throw',], ['end',],],
          [['mark', '\u2028',], ['word', 'throw',], ['end',],],
          [['mark', '\u2029',], ['word', 'throw',], ['end',],],
        ],);
        expect(atomsOf({ text: '(a, /* ) */ b // )\n) c', },),).toEqual([
          ['group', '(a, /* ) */ b // )\n)',],
          ['word', 'c',],
          ['end',],
        ],);
        expect([
          '',
          ' \n\t',
          '/* nap */ // doze',
        ].map((text,) => atomsOf({ text, },)),).toEqual([[['end',],], [['end',],], [['end',],],],);
      },
    },),
    it({
      name: 'READS A NAME OF ASCII LETTERS, DIGITS, AN UNDERSCORE AND A DOLLAR SIGN, ending it at any other letter',
      fn: async () => {
        expect(atomsOf({ text: 'Nap_1$ napé', },),).toEqual([
          ['word', 'Nap_1$',],
          ['word', 'nap',],
          ['mark', 'é',],
          ['end',],
        ],);
      },
    },),
    it({
      name: 'REFUSES TO READ, FOR CERTAIN, A SLASH THAT OPENS NO COMMENT, which is a division or a pattern, at its own '
        + 'level, inside a group and inside a substitution',
      fn: async () => {
        expect([
          'laps / 2',
          '(laps / 2)',
          `\`\${laps / 2}\``,
          '(name.replace(/[")]/g, ""))',
        ].map((text,) => atomsOf({ text, },)),).toEqual([
          [['word', 'laps',], ['unreadable',],],
          [['unreadable',],],
          [['unreadable',],],
          [['unreadable',],],
        ],);
      },
    },),
    it({
      name: 'REFUSES TO READ WHAT NEVER CLOSES OR CLOSES SOMETHING ELSE: a string with no closing quote or one on a '
        + 'later line, a template, a group, a block comment at its own level and inside a group, a string inside '
        + 'a group, a substitution, and a bracket closed by another\'s closer',
      fn: async () => {
        expect([
          '"never closed',
          '"closed on\nanother line"',
          '`never closed',
          '(never closed',
          '/* never closed',
          '(paws /* never closed',
          '(paws, "never closed)',
          `\`never \${closed`,
          '(paws[0)',
          '({ lap )',
        ].map((text,) => atomsOf({ text, },)),).toEqual([
          [['unreadable',],],
          [['unreadable',],],
          [['unreadable',],],
          [['unreadable',],],
          [['unreadable',],],
          [['unreadable',],],
          [['unreadable',],],
          [['unreadable',],],
          [['unreadable',],],
          [['unreadable',],],
        ],);
      },
    },),
  ],
},);
