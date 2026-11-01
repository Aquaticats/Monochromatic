import { readFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import {
  readPackageDocPaths,
  REPOSITORY_ROOT,
} from './living-docs.test-fixture.ts';

//region Markdown line width
// THE WIDTH OF A MARKDOWN LINE OF THE FAMILY (ledger B1359). The repository's rule for prose is lines under 120
// columns, broken at semantic boundaries, and neither `markdown-lint-clean` nor `doc-line-limit` reads the width, so
// the re-checks of rounds w404 and w406 each found hundreds of wide lines by hand. What is read here is the same
// thing the rule says: a line of 120 code points or more is wide, unless it could not be broken, which is the one
// exception the rule names (a link or a code span cannot be broken, since a break inside either shows as literal
// text). A fenced code block is literal text and is left alone. The scan is one pass over each line, with no pattern.

/**
 Width at which a line is too wide: the rule is under 120 columns, so a line of this many code points or more is wide.
 */
export const WIDE_FROM = 120;

/**
 What `indexOf` answers where the text is not there.
 */
const NOT_FOUND = -1;

/**
 Words that mark a bullet list item.
 */
const BULLET_MARKERS: ReadonlySet<string> = new Set([
  '-',
  '*',
  '+',
]);

/**
 Characters that close the number of an ordered list item.
 */
const NUMBER_CLOSERS: ReadonlySet<string> = new Set([
  '.',
  ')',
]);

/**
 Number of code points of a line.

 @param line - one line of text

 @returns Code points, which is what a column counts for text without wide characters

 @example
 ```ts
 const width = widthOf({ line: 'cat', },); // 3
 ```
 */
function widthOf({ line, }: { readonly line: string; },): number {
  /**
   Code points seen.
   */
  let width = 0;
  for (const character of line) {
    if (character !== '')
      width += 1;
  }
  return width;
}

/**
 Index just past the unbreakable span that opens at a position of a line: a code span, an inline link, a reference link
 or the whole of a reference definition.

 @param line - one line of Markdown

 @param at - position of a character

 @returns Index after the span that opens at the position, or the position itself where none opens there

 @example
 ```ts
 const end = spanEnd({ line: 'See [cat](nap.md) now', at: 4, },); // 17
 ```
 */
function spanEnd({
  line,
  at,
}: {
  readonly line: string;
  readonly at: number
},): number {
  /**
   Character at the position.
   */
  const opener = line.charAt(at,);
  if (opener === '`') {
    /**
     Position of the code span's closing backtick.
     */
    const close = line.indexOf(
      '`',
      at + 1,
    );
    return (close === NOT_FOUND) ? at : (close + 1);
  }
  if (opener !== '[')
    return at;
  /**
   Position of the bracket that closes the link text.
   */
  const textEnd = line.indexOf(
    ']',
    at + 1,
  );
  if (textEnd === NOT_FOUND)
    return at;
  /**
   Character after the link text, which says what kind of link it is.
   */
  const next = line.charAt(textEnd + 1,);
  if (next === ':')
    return line.length;
  /**
   Bracket that closes the destination or the label, matching the opener that follows the text.
   */
  const closer = (next === '(') ? ')' : ((next === '[') ? ']' : '');
  if (closer === '')
    return at;
  /**
   Position of that closing bracket.
   */
  const close = line.indexOf(
    closer,
    textEnd + 2,
  );
  return (close === NOT_FOUND) ? at : (close + 1);
}

/**
 The text of a line after its list marker, where it has one.

 @param line - one line of Markdown

 @returns The line without its indent and marker (`-`, `*`, `+`, or a number with `.` or `)`), or without its indent
 alone where it has no marker

 @example
 ```ts
 const body = afterListMarker({ line: '  - The cat naps.', },); // 'The cat naps.'
 ```
 */
function afterListMarker({ line, }: { readonly line: string; },): string {
  /**
   The line without its indent.
   */
  const body = line.trimStart();
  /**
   The first word, which is the marker where the line is a list item.
   */
  const [marker = '',] = body.split(
    ' ',
    1,
  );
  /**
   The marker without its closing `.` or `)`, which is a number for an ordered item.
   */
  const number = marker.slice(
    0,
    -1,
  );
  /**
   Whether the first word is a bullet or a number with its closing mark.
   */
  const isMarker = BULLET_MARKERS.has(marker,)
    || ((number !== '') && Number.isInteger(Number(number,),)
      && NUMBER_CLOSERS.has(marker.slice(-1,),));
  return isMarker ? body.slice(marker.length + 1,) : body;
}

/**
 Whether a line has a place to break outside a code span and a link, after its list marker and its first word.

 @param line - one line of Markdown outside a fence

 @returns True when a break could bring the line under the width

 @example
 ```ts
 const breakable = couldBreak({ line: 'The cat naps on the sill, and wakes.', },); // true
 ```
 */
function couldBreak({ line, }: { readonly line: string; },): boolean {
  /**
   The line after its list marker, without trailing spaces.
   */
  const body = afterListMarker({ line, },)
    .trim();
  return (function walkBody(): boolean {
    /**
     Whether a word has been passed, so that a space after it is a place to break.
     */
    let seenWord = false;
    /**
     Position walked.
     */
    let at = 0;
    while (at < body.length) {
      /**
       End of the span that opens here, which is the position itself where none does.
       */
      const end = spanEnd({
        line: body,
        at,
      },);
      if (end > at) {
        seenWord = true;
        at = end;
        continue;
      }
      if (body.charAt(at,) !== ' ')
        seenWord = true;
      else if (seenWord)
        return true;
      at += 1;
    }
    return false;
  })();
}

/**
 The lines of a Markdown text that are wide and could be made narrower, outside fenced code blocks.

 @param text - Markdown text

 @returns One-based line numbers, in order

 @example
 ```ts
 const wide = wideLinesOf({ text: 'A short line.\n', },); // []
 ```
 */
export function wideLinesOf({ text, }: { readonly text: string; },): readonly number[] {
  /**
   Whether the walk is inside a fenced code block.
   */
  let inFence = false;
  /**
   Line numbers found wide.
   */
  const found: number[] = [];
  /**
   The lines of the text.
   */
  const lines = text.split('\n',);
  for (const [at, line,] of lines.entries()) {
    if (line.trimStart()
      .startsWith('```',)) {
      inFence = !inFence;
      continue;
    }
    if ((!inFence) && (widthOf({ line, },) >= WIDE_FROM)
      && couldBreak({ line, },))
      found.push(at + 1,);
  }
  return found;
}

/**
 Every Markdown file the rule is read over: the family's package-level docs and the runbook of the corpus pass.

 @returns Paths from the repository root, in order

 @example
 ```ts
 const paths = await readWidthDocPaths();
 ```
 */
export async function readWidthDocPaths(): Promise<readonly string[]> {
  return [
    ...(await readPackageDocPaths()),
    join(
      'doc',
      'runbook',
      'translation-repair-corpus-pass.md',
    ),
  ];
}

/**
 The number of wide lines of each file that has any, by path from the repository root.

 @returns Paths with their counts, sorted by path

 @example
 ```ts
 const counts = await wideLineCounts();
 ```
 */
export async function wideLineCounts(): Promise<ReadonlyMap<string, number>> {
  /**
   Each path with the text of its file.
   */
  const files = await Promise.all((await readWidthDocPaths()).toSorted()
    .map(async function withText(path,): Promise<readonly [
      string,
      string
    ]> {
      return [
        path,
        await readFile(
          join(
            REPOSITORY_ROOT,
            path,
          ),
          'utf8',
        ),
      ];
    },),);
  /**
   Counts by path, for the files that have any wide line.
   */
  const counts = new Map<string, number>();
  for (const [path, text,] of files) {
    /**
     Wide lines of this file.
     */
    const wide = wideLinesOf({ text, },);
    if (wide.length > 0) {
      counts.set(
        path,
        wide.length,
      );
    }
  }
  return counts;
}

//endregion Markdown line width
