/**
 Guards against a character a reader cannot see, or cannot tell from a space,
 standing literally in the package's TypeScript: a soft hyphen, a zero-width
 space or joiner, a bidirectional control, a byte order mark, a variation
 selector or tag character, a no-break or ideographic space, a control
 character. In a test such a character is invisible fixture data, so a reader
 cannot see what the case feeds; in production code it is an invisible member
 of a set or a pattern, so a reader cannot see what the code matches; and a
 diff shows neither. Each is written as its escape instead, which a reader
 can see and a search can find.

 HOW THEY GOT IN. A tool that writes a file from a model's parameters decoded a
 backslash-u escape the model wrote into the character it names (measured on
 2026-10-06 UTC: an edit meant to write the escape of a hyphen wrote a hyphen,
 and a message draft got a byte order mark), so an escape written through such
 a tool can land as the character itself. The scan reads the file, not the
 intent.

 WHAT THE SCAN READS. Every TypeScript file under `src`, tests included, code
 point by code point: a code point with the Unicode property
 `Default_Ignorable_Code_Point`, `White_Space` or the general category `Cc` is
 a finding, except the four a source file is written with (space, tab, line
 feed, carriage return).

 THE FIXTURE CASE COMES FIRST, so the package-wide case is read against a
 scan shown able to find each class (ledger M21). Fixtures are cat-themed;
 every unseen character in them is made with `String.fromCodePoint`, so this
 file holds none.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { readPackageSource, } from './family-source.test-fixture.ts';
import type { SourceText, } from './source-scan.test-fixture.ts';
import { expectNoFindings, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scan-findings.test-fixture.ts';

//region Literal unseen characters
// Code points no reader sees as what they are, found by their Unicode
// properties rather than by a list, so a class nobody thought of is found too.

/* oxlint-disable no-restricted-syntax/no-regex -- the input is one code point, tested against one class with no quantifier, so the test is bounded and cannot backtrack; Unicode properties have no string API */
/**
 A code point that shows nothing, or shows as a space: default ignorable,
 white space, or a control character.
 */
const UNSEEN = /^[\p{Default_Ignorable_Code_Point}\p{White_Space}\p{Cc}]$/u;
/* oxlint-enable no-restricted-syntax/no-regex */

/**
 The unseen code points a source file is written with, which stand literally
 by right: space, tab, line feed and carriage return.
 */
const WRITTEN_WITH: ReadonlySet<number> = new Set([
  0x20,
  0x09,
  0x0A,
  0x0D,
],);

/**
 Places in the files given where an unseen character stands literally, as
 `path:line:column U+XXXX`, the column counted in UTF-16 units from one.

 @param files - files read, package source and tests alike

 @returns Findings, in file order and text order within a file

 @example
 ```ts
 const places = unseenCharacters({ files: await readPackageSource(), },);
 ```
 */
function unseenCharacters({ files, }: { readonly files: readonly SourceText[]; },): readonly string[] {
  /**
   Findings so far.
   */
  const found: string[] = [];
  for (const file of files) {
    /**
     Line and column of the code point being read, both counted from one.
     */
    const place = {
      line: 1,
      column: 1,
    };
    for (const character of file.text) {
      /**
       The code point's number.
       */
      const point = character.codePointAt(0,);
      if (point === undefined)
        throw new Error('unreachable: a string iterator yields no empty character',);
      if ((UNSEEN.test(character,)) && (!WRITTEN_WITH.has(point,))) {
        found.push(`${file.path}:${String(place.line,)}:${String(place.column,)} U+${
          point.toString(16,)
            .toUpperCase()
            .padStart(
              4,
              '0',
            )
        }`,);
      }
      if (character === '\n') {
        place.line += 1;
        place.column = 1;
        continue;
      }
      place.column += character.length;
    }
  }
  return found;
}

//endregion Literal unseen characters

await describe({
  name: 'unseen characters written literally',
  children: [
    it({
      name: 'FINDS a soft hyphen, a zero-width space, a joiner, a bidirectional isolate, a byte order mark, a '
        + 'variation selector, a tag character past U+FFFF, a no-break space, an ideographic space and two control '
        + 'characters, and leaves their escapes, a tab, a carriage return and a visible full-width comma',
      fn: async () => {
        /**
         Writes a code point as the character it is.

         @param point - code point

         @returns Character
         */
        function literal(point: number,): string {
          return String.fromCodePoint(point,);
        }
        expect(unseenCharacters({
          files: [
            {
              path: 'cat.ts',
              text: [
                `export const nap = 'cat${literal(0xAD,)}nap';`,
                `export const purr = 'pu${literal(0x20_0B,)}rr${literal(0x20_0D,)}';`,
                `// a${literal(0x20_66,)} sunny${literal(0x20_69,)} sill`,
                `${literal(0xFE_FF,)}export const meow = 'meow${literal(0xFE_0F,)}${literal(0xE_00_41,)}';`,
                `export const den = 'cat${literal(0xA0,)}den${literal(0x30_00,)}';`,
                `export const bell = 'ring${literal(0x0B,)}${literal(0x85,)}';`,
                String.raw`export const kept = 'cat\u00ADnap\u200B\uFEFF\u3000';`,
                'export const tab = \'cat\tnap，\';\r',
                '',
              ].join('\n',),
              isTest: false,
            },
          ],
        },),).toEqual([
          'cat.ts:1:24 U+00AD',
          'cat.ts:2:24 U+200B',
          'cat.ts:2:27 U+200D',
          'cat.ts:3:5 U+2066',
          'cat.ts:3:12 U+2069',
          'cat.ts:4:1 U+FEFF',
          'cat.ts:4:27 U+FE0F',
          'cat.ts:4:28 U+E0041',
          'cat.ts:5:24 U+00A0',
          'cat.ts:5:28 U+3000',
          'cat.ts:6:26 U+000B',
          'cat.ts:6:27 U+0085',
        ],);
      },
    },),
    it({
      name: 'FINDS NO UNSEEN CHARACTER written literally across the package\'s source and tests',
      fn: async () => {
        expectNoFindings({ findings: unseenCharacters({ files: await readPackageSource(), },), },);
      },
    },),
  ],
},);
