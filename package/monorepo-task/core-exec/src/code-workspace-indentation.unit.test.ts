/**
 Checks that `monochromatic.code-workspace` is indented with spaces, at twice its own nesting depth.

 The root `.editorconfig` asks every file for two-space indentation, and the workspace file turns on
 `editor.formatOnSave`, so a mixed tab-and-space file rewrites unrelated lines whenever any editor touches it. Nothing
 else reads this file's indentation: the dprint configuration does not associate the `.code-workspace` extension, and
 `.editorconfig` records that no check enforces its own settings. The check therefore reads the file at the repository
 root and reports the line it fails on, which is the fastest way for a hand edit to see what to fix.

 The reader follows JSON with comments: brackets inside a string, after an escape, or after `//` do not change the depth.
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { readFile, } from 'node:fs/promises';

//region Reading brackets

/**
 Where a scan of one line stands.
 */
type BracketScan = {
  /** Whether the scan is inside a string. */
  readonly insideString: boolean;
  /** Whether the previous character escaped the character after it. */
  readonly escaped: boolean;
  /** Whether a `//` comment has started. */
  readonly inComment: boolean;
  /** Brackets opened minus brackets closed so far. */
  readonly change: number;
};

/**
 A scan that has read nothing yet.
 */
const startOfLine: BracketScan = {
  insideString: false,
  escaped: false,
  inComment: false,
  change: 0,
};

/**
 Count one character of a line: a brace or a square bracket outside a string, an escape and a `//` comment changes the
 depth, and the characters inside a string or a comment do not.

 @param scan - state after the characters before this one

 @param character - the character to count

 @param index - position of the character in the line

 @param characters - the whole line as characters, for the lookahead a comment start needs

 @returns state after this character

 @example
 ```ts
 countBracket(startOfLine, '{', 0, Array.from('{')) .change === 1;
 ```
 */
function countBracket(scan: BracketScan, character: string, index: number, characters: readonly string[]): BracketScan {
  if (scan.inComment) return scan;
  if (scan.escaped) return { ...scan, escaped: false, };
  if (scan.insideString && (character === '\\')) return { ...scan, escaped: true, };
  if (character === '"') return { ...scan, insideString: (!scan.insideString), };
  if (scan.insideString) return scan;
  if ((character === '/') && (characters[index + 1] === '/')) return { ...scan, inComment: true, };
  const opened = (character === '{') || (character === '[');
  const closed = (character === '}') || (character === ']');
  if ((!opened) && (!closed)) return scan;
  return { ...scan, change: scan.change + (opened ? 1 : (-1)), };
}

/**
 How many brackets a line opens minus how many it closes.

 @param line - one line of a JSON-with-comments file

 @returns the change the line makes to the nesting depth

 @example
 ```ts
 bracketChange('  "a": "{",') === 0;
 ```
 */
function bracketChange(line: string,): number {
  const counted = Array.from(line,)
    .reduce(function countEach(scan, character, index, characters,) {
      return countBracket(scan, character, index, characters,);
    }, startOfLine,);
  return counted.change;
}

/**
 How many closing brackets a line starts with, which is how far the line dedents from the depth it begins at.

 @param line - one line of a JSON-with-comments file

 @returns the count of leading `}` and `]` characters

 @example
 ```ts
 leadingClosers('  }],') === 2;
 ```
 */
function leadingClosers(line: string,): number {
  const characters = Array.from(line.trimStart(),);
  const firstOther = characters.findIndex(function isNotCloser(character,) {
    return (character !== '}') && (character !== ']');
  },);
  return firstOther === (-1) ? characters.length : firstOther;
}

//endregion

//region Reading the file

/**
 Indentation the editor configuration asks for: two spaces per nesting level.
 */
const indentWidth = 2;

/**
 The file this check owns.
 */
const workspacePath = 'monochromatic.code-workspace';

/**
 Lines of the workspace file.
 */
const lines = (await readFile(new URL(`../../../../${workspacePath}`, import.meta.url,), 'utf8',)).split('\n',);

/**
 One line of the file and the nesting depth it starts at.
 */
type LineDepth = {
  readonly line: string;
  readonly depth: number;
};

/**
 Each line paired with the nesting depth it starts at.

 @param sourceLines - the file's lines

 @returns one entry per line, in file order

 @example
 ```ts
 depths(['{', '  "a": 1', '}']);
 ```
 */
function depths(sourceLines: readonly string[],): readonly LineDepth[] {
  return sourceLines.reduce<LineDepth[]>(
    function place(placed, line,) {
      const above = placed.at(-1,);
      const depth = above === undefined ? 0 : (above.depth + bracketChange(above.line,));
      placed.push({ line, depth, },);
      return placed;
    },
    [],
  );
}

/**
 The indentation a line should carry: two spaces per nesting level, less one level for each bracket the line itself
 closes, because a closing bracket sits at the depth of the block it closes.

 @param options - the line and the depth it starts at

 @returns the leading whitespace the line should have

 @example
 ```ts
 expectedIndent({ line: '  ],', depth: 2 }) === '  ';
 ```
 */
function expectedIndent({ line, depth, }: {
  readonly line: string;
  readonly depth: number;
},): string {
  return ' '.repeat(indentWidth * (depth - leadingClosers(line,)),);
}

/**
 The file's lines with the depth each one starts at.
 */
const depthOfLine = depths(lines,);

/**
 Indented lines of the file: the ones a wrong leading whitespace shows up on.
 */
const indentedLines = depthOfLine.flatMap(function indentedAt({ line, depth, }, index,) {
  const indented = line.startsWith(' ') || line.startsWith('\t');
  return indented ? [{ line, depth, index, },] : [];
},);

//endregion

await describe({
  name: workspacePath,
  children: [
    it({
      name: 'has indented lines several levels deep, so the check below cannot pass on an empty selection',
      fn: async function hasIndentedLines() {
        expect(indentedLines.length,).toBeGreaterThan(0,);
        const deepest = Math.max(...depthOfLine.map(function depthOf(entry,) {
          return entry.depth;
        },),);
        expect(deepest,).toBeGreaterThan(1,);
      },
    },),
    it({
      name: 'indents every line with spaces only, at twice the depth the line starts at',
      fn: async function indentsWithSpaces() {
        const mismatched = indentedLines.flatMap(function reasonFor({ line, depth, index, },) {
          const expected = expectedIndent({ line, depth, },);
          const found = line.slice(
            0,
            line.length - line.trimStart()
              .length,
          );
          if (found === expected) return [];
          const position = `${workspacePath}:${String(index + 1,)}`;
          const wanted = `expected ${JSON.stringify(expected,)}`;
          return [`${position}: ${wanted}, found ${JSON.stringify(found,)}`];
        },);
        expect(mismatched,).toEqual([],);
      },
    },),
  ],
},);
