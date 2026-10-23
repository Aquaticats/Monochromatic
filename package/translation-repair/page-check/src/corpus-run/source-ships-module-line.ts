import type { TreeNode, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  startsAny,
  type TextEdit,
  type TextRange,
} from './source-ships-ranges.ts';

//region Source ships module line
// THE LINES MDX READS AS AN ES MODULE: a flow block at the first column of the document's own level that starts with
// `import ` or `export ` is read by acorn as JavaScript, and prose that starts so is refused. A backslash cannot
// make it plain; the first letter written as a character reference does.
//
// A LINE OF INVISIBLE CHARACTERS BEFORE SUCH A LINE ENDS THE PARAGRAPH FOR THE STRICT GRAMMAR, which starts the module
// on the line after it, where the plain reading keeps both lines in one paragraph. This was measured on the strict
// grammar with a byte order mark, a no-break space, an ideographic space, a zero-width space, a word joiner and a
// zero-width joiner alone on the line, and with a soft hyphen, a left-to-right mark and a variation selector, which end
// no paragraph; every separator and format character is taken for invisible here, so a line the grammar keeps in the
// paragraph is written as a character reference too, which shows the same.

/* oxlint-disable no-restricted-syntax/no-regex -- the input is one line tested whole against one class, so the test is
bounded and cannot backtrack; the Unicode categories have no string API */
/**
 A line of separators and format characters only, which shows nothing.
 */
const INVISIBLE_LINE = /^[\p{Z}\p{Cf}\t]+$/u;
/* oxlint-enable no-restricted-syntax/no-regex */

/**
 What a flow block starts with when MDX reads it as an ES module: `import` or `export` and one space.
 */
const ES_MODULE_STARTS: readonly string[] = [
  'import ',
  'export ',
];

/**
 Whether MDX reads the node as an ES module: a paragraph or a setext heading at the first column of the document's own
 level that starts with `import ` or `export `.

 @param source - the text

 @param node - node of the parse

 @param isTopLevel - whether the node's parent is the root

 @param range - where the node stands

 @returns Whether the first word of the node must be written as a character reference

 @example
 ```ts
 const esm = startsEsModule({ source, node, isTopLevel: true, range, },);
 ```
 */
export function startsEsModule(
  {
    source,
    node,
    isTopLevel,
    range,
  }: {
    readonly source: string;
    readonly node: TreeNode;
    readonly isTopLevel: boolean;
    readonly range: TextRange;
  },
): boolean {
  // An ATX heading starts with its marker, so only a setext heading, whose first line is the module line, can match.
  if ((node.type !== 'paragraph') && (node.type !== 'heading'))
    return false;
  if (!isTopLevel)
    return false;
  /**
   Column the node starts at, where a node that follows indentation does not start at the first.
   */
  const column = node.position
    ?.start
    .column;
  if (column !== 1)
    return false;
  return startsAny({
    text: source,
    at: range.start,
    candidates: ES_MODULE_STARTS,
  },);
}

/**
 Where the lines of a paragraph or setext heading start that follow a line of invisible characters and begin with
 `import ` or `export `, each of which the strict grammar reads as an ES module.

 @param source - the text

 @param node - node of the parse

 @param isTopLevel - whether the node's parent is the root

 @param range - where the node stands

 @returns Index of the first letter of each such line, in text order

 @example
 ```ts
 const at = laterModuleLines({ source, node, isTopLevel: true, range, },);
 ```
 */
export function laterModuleLines(
  {
    source,
    node,
    isTopLevel,
    range,
  }: {
    readonly source: string;
    readonly node: TreeNode;
    readonly isTopLevel: boolean;
    readonly range: TextRange;
  },
): readonly number[] {
  if ((node.type !== 'paragraph') && (node.type !== 'heading'))
    return [];
  if (!isTopLevel)
    return [];
  /**
   Index of the first letter of each line found.
   */
  const found: number[] = [];
  for (
    let lineStart = range.start;
    lineStart < range.end;
  ) {
    /**
     Index of the line ending.
     */
    const lineEnd = source.indexOf(
      '\n',
      lineStart,
    );
    if ((lineEnd === (-1)) || ((lineEnd + 1) >= range.end))
      break;
    /**
     The line as the text has it.
     */
    const raw = source.slice(
      lineStart,
      lineEnd,
    );
    /**
     The line with no carriage return after it.
     */
    const line = raw.replace(
      '\r',
      '',
    );
    if (INVISIBLE_LINE.test(line,) && startsAny({
      text: source,
      at: lineEnd + 1,
      candidates: ES_MODULE_STARTS,
    },))
      found.push(lineEnd + 1,);
    lineStart = lineEnd + 1;
  }
  return found;
}

/**
 Writes the first letter of an ES module line as a character reference, which shows the same and starts no module.

 @param source - the text

 @param at - index of the letter

 @returns Edit replacing the letter

 @example
 ```ts
 const edit = moduleLetterEdit({ source: 'import a', at: 0, },);
 ```
 */
export function moduleLetterEdit(
  {
    source,
    at,
  }: {
    readonly source: string;
    readonly at: number;
  },
): TextEdit {
  return {
    start: at,
    end: at + 1,
    replacement: `&#${String(source.codePointAt(at,),)};`,
  };
}

//endregion Source ships module line
