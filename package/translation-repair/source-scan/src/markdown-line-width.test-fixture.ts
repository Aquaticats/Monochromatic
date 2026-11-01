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
// thing the rule says: a line of 120 columns or more is wide, unless it could not be broken, which is the one
// exception the rule names (a link or a code span cannot be broken, since a break inside either shows as literal
// text). A fenced code block is literal text and is left alone, and a fence is read as CommonMark reads one (ledger
// B1376). The scan is one pass over each line, with no pattern.
//
// A COLUMN IS A DISPLAY COLUMN, NOT A CODE POINT (ledger B1376). The rule says columns, and an editor or a terminal
// shows a character of East Asian Width Wide or Fullwidth in two of them, so a line of Chinese example text that is
// under 120 code points can fill more than 120 columns. The counts of the family's docs are the same under either
// measure on the day this was changed (no file holds more such lines by columns than by code points), so no allowance
// moved. THE LIMIT: the wide characters are the blocks of Unicode Standard Annex 11 that the docs and their examples can
// hold (CJK, Hangul, kana, fullwidth forms, the emoji blocks), not the whole table, since no width package is installed
// in the tree; a character outside those blocks counts one column, and a combining mark or joiner counts none.

/**
 Width at which a line is too wide: the rule is under 120 columns, so a line of this many columns or more is wide.
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

/* oxlint-disable no-restricted-syntax/no-regex -- each test is of one code point against one class with no quantifier, so it is bounded and cannot backtrack; the Unicode script and emoji properties have no string API (ledger B1376) */
/**
 Characters that take two display columns, as the East Asian Width classes Wide and Fullwidth give them: the Han, kana,
 Hangul and Bopomofo scripts without their halfwidth forms, the punctuation of the CJK block, the fullwidth forms and the
 characters that show as emoji by default.
 */
const TWO_COLUMNS = /[[\p{Script=Han}\p{Script=Hiragana}\p{Script=Katakana}\p{Script=Hangul}\p{Script=Bopomofo}]--[\uFF66-\uFFDC]]|[\u3000-\u303E\uFF01-\uFF60\uFFE0-\uFFE6]|\p{Emoji_Presentation}/v;

/**
 Characters that take no column of their own: combining marks, the zero width joiner and the emoji variation selector.
 */
const ZERO_COLUMNS = /\p{M}|\u200D|\uFE0F/u;
/* oxlint-enable no-restricted-syntax/no-regex */

/**
 Number of display columns of a line.

 @param line - one line of text

 @returns Columns the line fills: two for each wide character, none for a combining mark or a joiner, one for the rest

 @example
 ```ts
 // Three narrow characters fill three columns, and each wide one fills two.
 const width = widthOf({ line: 'cat', },);
 const wideWidth = widthOf({ line: '猫猫', },);
 ```
 */
export function widthOf({ line, }: { readonly line: string; },): number {
  /**
   Columns counted so far.
   */
  let width = 0;
  for (const character of line) {
    if (ZERO_COLUMNS.test(character,))
      continue;
    width += TWO_COLUMNS.test(character,) ? 2 : 1;
  }
  return width;
}

/**
 A fence that is open: the character it is made of and the length of its opening run.
 */
type Fence = {
  /**
   The backtick or the tilde the fence is made of.
   */
  readonly character: string;

  /**
   Length of the run that opened it, which the closing run must match or pass.
   */
  readonly length: number;
};

/**
 What a line is read as where it opens no fence.
 */
const NO_FENCE: unique symbol = Symbol('the line opens no fence',);

/**
 The run of one fence character a line opens with, after its indent, and the rest of the line.
 */
type FenceRun = Fence & {
  /**
   The text after the run.
   */
  readonly rest: string;
};

/**
 What a line is read as where it does not open with a fence character.
 */
const NO_RUN: unique symbol = Symbol('the line opens with no fence character',);

/**
 Characters a fence is made of.
 */
const FENCE_CHARACTERS: ReadonlySet<string> = new Set([
  '`',
  '~',
]);

/**
 Shortest run of fence characters that opens or closes a fence.
 */
const FENCE_RUN_MINIMUM = 3;

/**
 The run of one character a line opens with, after its indent.

 @param line - one line of Markdown

 @returns Character, length and rest of the line, or the mark of a line that does not open with a fence character

 @example
 ```ts
 const run = fenceRunOf({ line: '~~~~text', },);
 ```
 */
function fenceRunOf({ line, }: { readonly line: string; },): FenceRun | typeof NO_RUN {
  /**
   The line without its indent.
   */
  const body = line.trimStart();
  /**
   Its first character.
   */
  const character = body.charAt(0,);
  if (!FENCE_CHARACTERS.has(character,))
    return NO_RUN;
  /**
   Position of the first character that is not the one the run is made of, which is the run's length.
   */
  const stop = Array.from(body,)
    .findIndex(function endsRun(next,): boolean {
      return next !== character;
    },);
  /**
   Length of the run of that character.
   */
  const length = (stop === NOT_FOUND) ? body.length : stop;
  return {
    character,
    length,
    rest: body.slice(length,),
  };
}

/**
 The fence a line opens, as CommonMark reads it: three or more backticks or tildes, and for a backtick fence an info
 string with no backtick in it, so a line that opens with an inline code span is no fence.

 @param line - one line of Markdown outside a fence

 @returns The fence opened, or the mark of a line that opens none

 @example
 ```ts
 const opened = fenceOpenedBy({ line: '````ts', },);
 ```
 */
function fenceOpenedBy({ line, }: { readonly line: string; },): Fence | typeof NO_FENCE {
  /**
   Run the line opens with.
   */
  const run = fenceRunOf({ line, },);
  if ((run === NO_RUN) || (run.length < FENCE_RUN_MINIMUM))
    return NO_FENCE;
  /**
   The rest of the line after the run, which is the info string.
   */
  const { rest, } = run;
  /**
   Whether the info string holds a backtick.
   */
  const infoHasBacktick = rest.includes('`',);
  if ((run.character === '`') && infoHasBacktick)
    return NO_FENCE;
  return {
    character: run.character,
    length: run.length,
  };
}

/**
 Whether a line closes the open fence: a run of the same character at least as long as the opening one, and nothing but
 spaces after it.

 @param line - one line of Markdown inside the fence

 @param fence - the fence that is open

 @returns True when the line closes it

 @example
 ```ts
 const closes = closesFence({ line: '```', fence: { character: '`', length: 3, }, },);
 ```
 */
function closesFence(
  {
    line,
    fence,
  }: {
    readonly line: string;
    readonly fence: Fence;
  },
): boolean {
  /**
   Run the line opens with.
   */
  const run = fenceRunOf({ line, },);
  if (run === NO_RUN)
    return false;
  /**
   The rest of the line after the run.
   */
  const { rest, } = run;
  /**
   Whether nothing but spaces follows the run.
   */
  const nothingAfter = rest.trim() === '';
  return (run.character === fence.character)
    && (run.length >= fence.length)
    && nothingAfter;
}

/**
 Index just past the unbreakable span that opens at a position of a line: a code span, an inline link, a reference link
 or the whole of a reference definition.

 @param line - one line of Markdown

 @param at - position of a character

 @returns Index after the span that opens at the position, or the position itself where none opens there

 @example
 ```ts
 // The link opens at column four and its last bracket is the sixteenth character.
 const end = spanEnd({ line: 'See [cat](nap.md) now', at: 4, },);
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

 @returns Line without its indent and marker (`-`, `*`, `+`, or a number with `.` or `)`), or without its indent
 alone where it has no marker

 @example
 ```ts
 // The indent and the marker go, and the sentence stays.
 const body = afterListMarker({ line: '  - The cat naps.', },);
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
 // A sentence of several words has a place to break.
 const breakable = couldBreak({ line: 'The cat naps on the sill, and wakes.', },);
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
 // A short line is not wide.
 const wide = wideLinesOf({ text: 'A short line.\n', },);
 ```
 */
export function wideLinesOf({ text, }: { readonly text: string; },): readonly number[] {
  /**
   The fence the walk is inside, once it is inside one.
   */
  let fence: Fence | typeof NO_FENCE = NO_FENCE;
  /**
   Line numbers found wide.
   */
  const found: number[] = [];
  /**
   The lines of the text.
   */
  const lines = text.split('\n',);
  for (const [at, line,] of lines.entries()) {
    if (fence !== NO_FENCE) {
      // A fenced line is literal text, and only a run of the same character at least as long closes the fence.
      if (closesFence({
        line,
        fence,
      },))
        fence = NO_FENCE;
      continue;
    }
    /**
     The fence this line opens, when it opens one.
     */
    const opened = fenceOpenedBy({ line, },);
    if (opened !== NO_FENCE) {
      fence = opened;
      continue;
    }
    if ((widthOf({ line, },) >= WIDE_FROM) && couldBreak({ line, },))
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
async function readWidthDocPaths(): Promise<readonly string[]> {
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
