/**
 Guards the package's comments against a backtick written with a backslash
 before it. A comment is Markdown to every reader that renders it (TSDoc in an
 editor's hover, the docs a TSDoc tool builds), and Markdown keeps a backslash
 before a backtick inside a code span or a fenced block as the character it is:
 an example fence written with a backslash before each backtick renders as
 three escaped backticks and no fence, and a template literal written with
 escaped backticks inside an example shows code that does not compile. The
 habit comes from writing a comment's text inside a template literal, where
 the escape is needed.

 To show a backtick inside a code span, the span is delimited by a longer run
 of backticks (``` `` a `b` `` ```); inside a fenced example, the code is
 written as it is. The one form kept is a code span that holds nothing but
 backslashes (`` `\` ``, or `` `\\` `` for the JSON escape of one), where the
 backslashes are the span's text and the backtick after them closes the span.

 WHAT THE SCAN READS. Every comment of every TypeScript file under `src`,
 tests included, as the parser reports them: a backslash followed by a
 backtick is a finding unless the run of backslashes it ends has a backtick
 just before it.

 THE FIXTURE CASE COMES FIRST, so the package-wide case is read against a
 scan shown able to find each form (ledger M21). Fixtures are cat-themed.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { readPackageSource, } from './family-source.test-fixture.ts';
import {
  parseSource,
  type SourceText,
} from './source-scan.test-fixture.ts';
import { expectNoFindings, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scan-findings.test-fixture.ts';

//region Comment escaped backticks
// Comment lines that write a backtick with a backslash before it, found in the
// comments the parser reports, so a template literal's own escapes are never
// read.

/**
 What a backtick written with a backslash before it looks like in a comment.
 */
const ESCAPED_BACKTICK = '\\`';

/**
 Text with the run of backslashes it ends on taken off, so the character
 before a run can be read whatever the run's length.

 @param text - text before an escaped backtick

 @returns Text up to its trailing backslashes

 @example
 ```ts
 withoutTrailingBackslashes({ text: 'a `\\', },); // 'a `'
 ```
 */
function withoutTrailingBackslashes({ text, }: { readonly text: string; },): string {
  /**
   End of the text kept, moved back over each trailing backslash.
   */
  const cursor = { end: text.length, };
  while ((cursor.end > 0) && (text.charAt(cursor.end - 1,) === '\\'))
    cursor.end -= 1;

  return text.slice(
    0,
    cursor.end,
  );
}

/**
 Places in the files given where a comment writes a backtick with a backslash
 before it, as `path:line`, one per line however many it holds.

 @param files - files read, package source and tests alike

 @returns Findings, in file order and line order within a file

 @example
 ```ts
 const lines = escapedBacktickLines({ files: await readPackageSource(), },);
 ```
 */
function escapedBacktickLines({ files, }: { readonly files: readonly SourceText[]; },): readonly string[] {
  /**
   Findings so far.
   */
  const found: string[] = [];
  for (const file of files) {
    /**
     Comments of this file, with their offsets in its text.
     */
    const { comments, } = parseSource({ file, },);
    for (const comment of comments) {
      /**
       Comment as written, its markers included.
       */
      const text = file.text.slice(
        comment.start,
        comment.end,
      );
      /**
       Line of the file the comment opens on, counted from one.
       */
      const firstLine = file.text
        .slice(
        0,
        comment.start,
        )
        .split('\n',)
        .length;
      for (const [index, line,] of text.split('\n',).entries()) {
        /**
         Whether this line escapes a backtick anywhere but at the close of a
         span that holds nothing but backslashes.
         */
        const escapes = line
          .split(ESCAPED_BACKTICK,)
          .slice(
          0,
          -1,
          )
          .some(function beforeEscape(before,): boolean {
            return !withoutTrailingBackslashes({ text: before, },)
              .endsWith('`',);
          },);
        if (escapes)
          found.push(`${file.path}:${String(firstLine + index,)}`,);
      }
    }
  }
  return found;
}

//endregion Comment escaped backticks

await describe({
  name: 'backticks escaped in comments',
  children: [
    it({
      name: 'FINDS an escaped fence, an escaped code span and a template literal escaped in an example, in TSDoc and '
        + 'line comments, and leaves a span holding one backslash, a span holding two, a longer-delimited span and a '
        + 'template\'s own escapes',
      fn: async () => {
        expect(escapedBacktickLines({
          files: [
            {
              path: 'cat.ts',
              text: [
                '/**',
                ' Naps.',
                '',
                ' @example',
                ' \\`\\`\\`ts',
                ' const nap = \\`Tabby naps again\\`;',
                ' \\`\\`\\`',
                ' */',
                'export function napOf({ cat, }: { readonly cat: string; },): string {',
                '  // the \\`cat\\` naps',
                '  return `naps \\`now\\``;',
                '}',
                '/** Keeps `\\`, `\\\\` and `` a `b` `` as they are. */',
                'export const purr = 1;',
                '',
              ].join('\n',),
              isTest: false,
            },
          ],
        },),).toEqual([
          'cat.ts:5',
          'cat.ts:6',
          'cat.ts:7',
          'cat.ts:10',
        ],);
      },
    },),
    it({
      name: 'FINDS NO COMMENT that writes a backtick with a backslash before it across the package\'s source and tests',
      fn: async () => {
        expectNoFindings({ findings: escapedBacktickLines({ files: await readPackageSource(), },), },);
      },
    },),
  ],
},);
