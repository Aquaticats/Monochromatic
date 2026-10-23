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
//
// A LIST MARKER ALONE ON A LINE, RIGHT AFTER A FOOTNOTE DEFINITION, ENDS THE PARAGRAPH TOO. The strict grammar
// keeps a footnote definition open for the line after it and tries a new container there as if no paragraph were open
// (`micromark@4.0.3`, `lib/initialize/document.js`, `checkNewContainers`: the interrupt flag is set only when every
// open container continued; `micromark-core-commonmark@2.0.4`, `lib/list.js`, `start` and `atMarker`: with the flag
// unset, a marker with nothing after it starts a list, and so does an ordered marker that is not 1). A line of `*`,
// `-`, `+` or a number and a dot or a parenthesis alone is then an empty list item, and the line after it starts a
// block, an ES module among them; the plain reading, which has no footnote definition to keep open, keeps the marker as
// a line of the paragraph.
//
// A LINE OF INVISIBLE CHARACTERS ENDS THE PARAGRAPH THE SAME WAY, so a marker alone on a later line of the
// node starts a list as well.
//
// THE DEFINITION NEED NOT START THE PARAGRAPH. The strict grammar lets a footnote definition interrupt a paragraph, at
// any indentation (the indented code that would otherwise cap it is off in MDX), so a line of `[^label]:` inside a
// paragraph of the plain reading opens one for the lines after it, which stay a lazy line of its paragraph and keep the
// definition open to the line that ends them.
//
// THE LINE NEED NOT BE IN A PARAGRAPH AT THE TOP. A line at the first column ends every container it stands in, so one
// that follows such a line in a list item or a block quote (a lazy line of its paragraph) starts a module as well, and
// so does one in a raw HTML block, which the plain reading keeps whole across blank lines and the strict grammar, which
// has no such block, reads as paragraphs.

/* oxlint-disable no-restricted-syntax/no-regex -- the input is one line tested whole against one class, so the test is
bounded and cannot backtrack; the Unicode categories have no string API */
/**
 A line of separators and format characters only, or no characters at all, which shows nothing.
 */
const INVISIBLE_LINE = /^[\p{Z}\p{Cf}\t]*$/u;
/* oxlint-enable no-restricted-syntax/no-regex */

/**
 The characters a bullet list marker is.
 */
const BULLET_MARKERS = '*+-';

/**
 The characters that end an ordered list marker.
 */
const ORDERED_MARKER_ENDS = '.)';

/**
 How many digits an ordered list marker has at most.
 */
const ORDERED_MARKER_DIGITS = 9;

/**
 How many columns of indentation may stand before a list marker.
 */
const MARKER_INDENT = 3;

/**
 How many columns a tab takes in the indentation of a line.
 */
const TAB_COLUMNS = 4;

/**
 The characters a footnote label may not hold: the white space micromark ends a label at (a space, a tab and the line
 endings) and an opening bracket.
 */
const LABEL_BREAKS = ' \t\r\n[';

/**
 Index in a footnote definition's line where the label starts, after the bracket and the caret.
 */
const FOOTNOTE_LABEL_START = 2;

/**
 What a flow block starts with when MDX reads it as an ES module: `import` or `export` and one space.
 */
const ES_MODULE_STARTS: readonly string[] = [
  'import ',
  'export ',
];

/**
 Node types whose later lines can start a module: the paragraph and the setext heading, whose lines the plain reading
 keeps together, and the raw HTML block, which it keeps whole across blank lines.
 */
const LINE_BLOCKS: ReadonlySet<string> = new Set([
  'paragraph',
  'heading',
  'html',
],);

/**
 The indentation of a line.

 @param line - line, with no line ending

 @returns Index of the first character after the indentation, and the columns the indentation takes

 @example
 ```ts
 const indent = indentOf({ line: '  \t*', },);
 ```
 */
function indentOf(
  { line, }: { readonly line: string; },
): {
  readonly at: number;
  readonly columns: number;
} {
  /**
   Index of the first character after the indentation, and the columns the indentation takes.
   */
  const indent = {
    at: 0,
    columns: 0,
  };
  while ((indent.at < line.length) && ' \t'.includes(line.charAt(indent.at,),)) {
    indent.columns += (line.charAt(indent.at,) === '\t') ? TAB_COLUMNS : 1;
    indent.at += 1;
  }
  return indent;
}

/**
 Whether a line starts a footnote definition: a label of a caret and characters that are no white space and no bracket,
 and a colon, after any indentation.

 @param line - line, with no line ending

 @returns Whether the strict grammar reads the line as the first line of a footnote definition

 @example
 ```ts
 const starts = startsFootnoteLine({ line: '[^1]: 注', },);
 ```
 */
export function startsFootnoteLine({ line, }: { readonly line: string; },): boolean {
  /**
   Index of the first character after the indentation.
   */
  const { at, } = indentOf({ line, },);
  /**
   The line from its first character after the indentation.
   */
  const body = line.slice(at,);
  if (!body.startsWith('[^',))
    return false;
  /**
   Index of the bracket that closes the label.
   */
  const close = body.indexOf(']',);
  if (close <= FOOTNOTE_LABEL_START)
    return false;
  /**
   The label, without its caret.
   */
  const label = body.slice(
    FOOTNOTE_LABEL_START,
    close,
  );
  /**
   Whether a character of the label is white space or an opening bracket.
   */
  const isBroken = Array
    .from(label,)
    .some(function breaksLabel(character,): boolean {
      return LABEL_BREAKS.includes(character,);
    },);
  return (body.charAt(close + 1,) === ':') && (!isBroken);
}

/**
 Whether a line is a list marker and nothing else: a bullet, or one to nine digits and a dot or a parenthesis, with
 up to three columns of indentation before it and white space after it.

 @param line - line, with no line ending

 @returns Whether the strict grammar reads an empty list item there when no paragraph is open

 @example
 ```ts
 const marker = isEmptyMarkerLine({ line: ' 2)', },);
 ```
 */
export function isEmptyMarkerLine({ line, }: { readonly line: string; },): boolean {
  /**
   Index of the first character after the indentation, and the columns the indentation takes.
   */
  const indent = indentOf({ line, },);
  if (indent.columns > MARKER_INDENT)
    return false;
  /**
   Index after the marker, before the spaces and tabs that end the line.
   */
  const end = { at: line.length, };
  while ((end.at > indent.at) && ' \t'.includes(line.charAt(end.at - 1,),))
    end.at -= 1;
  /**
   What stands between the indentation and the white space at the end.
   */
  const marker = line.slice(
    indent.at,
    end.at,
  );
  if (marker.length === 0)
    return false;
  if (marker.length === 1)
    return BULLET_MARKERS.includes(marker,);
  /**
   The digits before the end of an ordered marker.
   */
  const digits = marker.slice(
    0,
    -1,
  );
  /**
   Whether the marker ends with a dot or a parenthesis.
   */
  const hasEnd = ORDERED_MARKER_ENDS.includes(marker.slice(-1,),);
  /**
   Whether everything before the end is a digit.
   */
  const isNumber = Array
    .from(digits,)
    .every(function isDigit(character,): boolean {
      return (character >= '0') && (character <= '9');
    },);
  return [
    hasEnd,
    isNumber,
    digits.length <= ORDERED_MARKER_DIGITS,
  ].every(Boolean,);
}

/**
 Whether MDX reads the node as an ES module: a paragraph or a setext heading at the first column of the document's own
 level that starts with `import ` or `export `.

 @param source - text

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
 Where the lines of a paragraph, setext heading or raw HTML block start that follow a line of invisible characters or
 no characters and begin with `import ` or `export ` at the first column, each of which the strict grammar reads as an
 ES module.

 @param source - text

 @param node - node of the parse

 @param range - where the node stands

 @param isNested - whether the text is a block read apart from the document, none of whose lines stands at the first
 column of it

 @param afterFootnote - whether the node follows a footnote definition on the next line or opens one, so that a line of
 a list marker alone also ends its paragraph for the strict grammar; a line inside the node that starts a footnote
 definition does the same for the lines after it

 @returns Index of the first letter of each such line, in text order

 @example
 ```ts
 const at = laterModuleLines({ source, node, range, isNested: false, },);
 ```
 */
export function laterModuleLines(
  {
    source,
    node,
    range,
    isNested,
    afterFootnote = false,
  }: {
    readonly source: string;
    readonly node: TreeNode;
    readonly range: TextRange;
    readonly isNested: boolean;
    readonly afterFootnote?: boolean;
  },
): readonly number[] {
  if (isNested)
    return [];
  if (!LINE_BLOCKS.has(node.type,))
    return [];
  /**
   Index of the first letter of each line found.
   */
  const found: number[] = [];
  /**
   Whether the strict grammar tries a new list at a marker alone on the line read next: the node follows or opens a
   footnote definition, or a line read starts one inside the node or ends the paragraph.
   */
  const marker = { startsList: afterFootnote, };
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
    if ((INVISIBLE_LINE.test(line,) || (marker.startsList && isEmptyMarkerLine({ line, },))) && startsAny({
      text: source,
      at: lineEnd + 1,
      candidates: ES_MODULE_STARTS,
    },))
      found.push(lineEnd + 1,);
    marker.startsList ||= INVISIBLE_LINE.test(line,) || startsFootnoteLine({ line, },);
    lineStart = lineEnd + 1;
  }
  return found;
}

/**
 Writes the first letter of an ES module line as a character reference, which shows the same and starts no module.

 @param source - text

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

/**
 The edits that keep a node from starting an ES module: its first word and the first word of each later line, as
 character references.

 @param source - text

 @param node - node of the parse

 @param range - where the node stands

 @param isTopLevel - whether the node's parent is the root of the document

 @param isNested - whether the text is a block read apart from the document

 @param afterFootnote - whether the node follows a footnote definition on the next line or opens one

 @returns Edits, none for a node that starts no module

 @example
 ```ts
 const edits = moduleEdits({ source, node, range, isTopLevel: true, isNested: false, },);
 ```
 */
export function moduleEdits(
  {
    source,
    node,
    range,
    isTopLevel,
    isNested,
    afterFootnote = false,
  }: {
    readonly source: string;
    readonly node: TreeNode;
    readonly range: TextRange;
    readonly isTopLevel: boolean;
    readonly isNested: boolean;
    readonly afterFootnote?: boolean;
  },
): readonly TextEdit[] {
  /**
   Index of the first letter of the node, when it starts a module.
   */
  const first = startsEsModule({
    source,
    node,
    isTopLevel,
    range,
  },)
    ? [range.start,]
    : [];
  /**
   Index of the first letter of each later line that starts one.
   */
  const later = laterModuleLines({
    source,
    node,
    range,
    isNested,
    afterFootnote,
  },);
  return [
    ...first,
    ...later,
  ].map(function letter(at,): TextEdit {
    return moduleLetterEdit({
      source,
      at,
    },);
  },);
}

//endregion Source ships module line
