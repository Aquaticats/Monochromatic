import type { TreeNode, } from '@monochromatic-dev/translation-repair-document-structure/ts';

//region Source ships spans
// WHERE IN A TEXT THE STRICT MDX GRAMMAR READS SYNTAX, read off a CommonMark parse of the same text
// (`document-structure/src/parse-markdown-math.ts`) by node positions. The parse is of the text itself and not of a
// guess at it: a node says whether the characters it spans are text a reader sees (a `text` node, a raw `html` node,
// indented code, which MDX has no such thing as, the alternative text of an image) or something the grammar reads as
// its own syntax (a fence, a code span, math, a link destination), and the second kind is never touched.

/**
 A stretch of the text, the end exclusive, in UTF-16 units.
 */
export type TextRange = {
  /**
   Index of the first character.
   */
  readonly start: number;

  /**
   Index after the last character.
   */
  readonly end: number;
};

/**
 A stretch of the text and what stands in its place.
 */
export type TextEdit = TextRange & {
  /**
   Text written instead of the stretch.
   */
  readonly replacement: string;
};

/**
 What a parse says of a text.
 */
export type SourceSpans = {
  /**
   Stretches a reader sees as text, in which a brace or a less-than sign is syntax for the strict grammar.
   */
  readonly text: readonly TextRange[];

  /**
   Stretches rewritten whole, in text order and not overlapping.
   */
  readonly edits: readonly TextEdit[];
};

/**
 Characters a backslash escapes in Markdown: the ASCII punctuation.
 */
export const ESCAPABLE = '!"#$%&\'()*+,-./:;<=>?@[\\]^_`{|}~';

/**
 How many characters `![` takes.
 */
const IMAGE_OPENER_LENGTH = 2;

/**
 What a flow block starts with when MDX reads it as an ES module: `import` or `export` and one space.
 */
const ES_MODULE_STARTS: readonly string[] = [
  'import ',
  'export ',
];

/**
 What a fenced code block starts with, as the parse positions one: the fence itself.
 */
const FENCE_MARKS: readonly string[] = [
  '```',
  '~~~',
];

/**
 Characters other than control characters and spaces that a destination holds as a percent escape, since Markdown reads
 them as the end of it or as its syntax.
 */
const DESTINATION_SYNTAX = '<>(){}\\';

/**
 Characters Markdown reads in the text of a link.
 */
const LABEL_SYNTAX = '\\[]{}<>*_`~&';

/**
 The highest code point of a character that one percent escape stands for.
 */
const ASCII_LIMIT = 0x7F;

/**
 The code point of a space, below which every character is a control character.
 */
const SPACE_CODE = 0x20;

/**
 The base percent escapes write their digits in.
 */
const HEX = 16;

/**
 Where a node's characters start and end.

 @param node - node of the parse

 @returns The range its position gives

 @throws {@link Error} when the node has no position, which the reading used never builds

 @example
 ```ts
 const { start, end, } = rangeOf({ node, },);
 ```
 */
function rangeOf({ node, }: { readonly node: TreeNode; },): TextRange {
  /**
   Where the node starts.
   */
  const start = node.position
    ?.start
    .offset;
  /**
   Where the node ends.
   */
  const end = node.position
    ?.end
    .offset;
  if ((start === undefined) || (end === undefined)) {
    throw new Error(
      `unreachable: a ${node.type} node of the plain reading carries no position, though that reading builds `
        + 'every one with one',
    );
  }
  return {
    start,
    end,
  };
}

/**
 Whether the text has one of the strings at an index.

 @param text - the text

 @param at - index to look at

 @param candidates - strings to try

 @returns Whether any of them starts at the index

 @example
 ```ts
 const fenced = startsAny({ text: '```ts', at: 0, candidates: ['```', '~~~',], },);
 ```
 */
function startsAny(
  {
    text,
    at,
    candidates,
  }: {
    readonly text: string;
    readonly at: number;
    readonly candidates: readonly string[];
  },
): boolean {
  return candidates.some(function startsHere(candidate,): boolean {
    return text.startsWith(
      candidate,
      at,
    );
  },);
}

/**
 Where the bracket closing a label stands.

 @param source - the text

 @param from - index just past the opening bracket

 @param limit - index the label cannot run past

 @returns Index of the closing bracket, or the limit when none closes within it

 @example
 ```ts
 const close = closingBracket({ source: '![a [b]](u)', from: 2, limit: 11, },);
 ```
 */
function closingBracket(
  {
    source,
    from,
    limit,
  }: {
    readonly source: string;
    readonly from: number;
    readonly limit: number;
  },
): number {
  /**
   Brackets opened inside the label and not yet closed.
   */
  const open = { depth: 1, };
  for (let at = from; at < limit; at += 1) {
    /**
     Character read.
     */
    const character = source.charAt(at,);
    if (character === '\\') {
      at += 1;
      continue;
    }
    if (character === '[')
      open.depth += 1;
    if (character === ']')
      open.depth -= 1;
    if (open.depth === 0)
      return at;
  }
  return limit;
}

/**
 Writes a character of a destination as the percent escape of its UTF-8 bytes when Markdown reads it as syntax.

 @param character - one character of the destination

 @returns The character, or its escape

 @example
 ```ts
 const written = destinationCharacter({ character: ' ', },);
 ```
 */
function destinationCharacter({ character, }: { readonly character: string; },): string {
  /**
   The character's code point.
   */
  const codePoint = character.codePointAt(0,) ?? 0;
  /**
   Whether Markdown reads the character as the end of a destination or as its syntax.
   */
  const unsafe = (codePoint <= SPACE_CODE)
    || (codePoint === ASCII_LIMIT)
    || DESTINATION_SYNTAX.includes(character,)
    || (character.trim() === '');
  if (!unsafe)
    return character;
  if (codePoint > ASCII_LIMIT)
    return encodeURIComponent(character,);
  /**
   The code point in upper-case hexadecimal.
   */
  const digits = codePoint
    .toString(HEX,)
    .toUpperCase();
  return `%${
    digits.padStart(
      2,
      '0',
    )
  }`;
}

/**
 Writes the text of an angle autolink as a link with the same text and target, without the angle brackets MDX reads
 as a tag.

 @param source - the text

 @param url - target the parse gives the autolink

 @param range - where the autolink stands

 @returns The link in Markdown's own link syntax

 @example
 ```ts
 const link = linkInsteadOfAutolink({ source, url: node.url, range, },);
 ```
 */
function linkInsteadOfAutolink(
  {
    source,
    url,
    range,
  }: {
    readonly source: string;
    readonly url: string;
    readonly range: TextRange;
  },
): string {
  /**
   What stands between the angle brackets, which is the text of the link.
   */
  const inner = source.slice(
    range.start + 1,
    range.end - 1,
  );
  /**
   The text with each character Markdown reads as syntax written with a backslash.
   */
  const label = Array.from(inner,)
    .map(function written(character,): string {
      return LABEL_SYNTAX.includes(character,) ? `\\${character}` : character;
    },)
    .join('',);
  /**
   The target with each character Markdown reads as syntax written as a percent escape.
   */
  const destination = Array.from(url,)
    .map(function written(character,): string {
      return destinationCharacter({ character, },);
    },)
    .join('',);
  return `[${label}](${destination})`;
}

/**
 Whether MDX reads the node as an ES module: a paragraph at the first column of the document's own level that starts with
 `import ` or `export `.

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
function startsEsModule(
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
  if (node.type !== 'paragraph')
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
 A node still to read, and whether it stands directly under the root.
 */
type Pending = {
  /**
   The node.
   */
  readonly node: TreeNode;

  /**
   Whether its parent is the root.
   */
  readonly isTopLevel: boolean;
};

/**
 Reads where a text holds syntax, from the parse of it.

 @param source - the text, the parse's own input apart from the comments it blanked

 @param root - parse of the text by the plain reading with the site's math

 @returns The stretches a reader sees as text and the stretches rewritten whole

 @throws {@link Error} when a node has no position, which the reading used never builds

 @example
 ```ts
 const spans = spansOf({ source, root: parseMarkdownMathBody({ body: source, },), },);
 ```
 */
export function spansOf(
  {
    source,
    root,
  }: {
    readonly source: string;
    readonly root: TreeNode;
  },
): SourceSpans {
  /**
   Stretches read as text.
   */
  const text: TextRange[] = [];
  /**
   Stretches rewritten.
   */
  const edits: TextEdit[] = [];
  /**
   Nodes still to read, the last of the root's children on top so they come out in order.
   */
  const pending: Pending[] = ('children' in root)
    ? root.children
      .map(function topLevel(node,): Pending {
        return {
          node,
          isTopLevel: true,
        };
      },)
      .toReversed()
    : [];
  for (let next = pending.pop(); next !== undefined; next = pending.pop()) {
    /**
     The node and whether it stands directly under the root.
     */
    const {
      node,
      isTopLevel,
    } = next;
    /**
     Where its characters are.
     */
    const range = rangeOf({ node, },);
    // A TEXT AND A RAW HTML NODE are what MDX reads a brace and a less-than sign in.
    if ((node.type === 'text') || (node.type === 'html')) {
      text.push(range,);
      continue;
    }
    // THE ALTERNATIVE TEXT OF AN IMAGE is text too; the destination and the title after it are not.
    if ((node.type === 'image') || (node.type === 'imageReference')) {
      text.push({
        start: range.start + IMAGE_OPENER_LENGTH,
        end: closingBracket({
          source,
          from: range.start + IMAGE_OPENER_LENGTH,
          limit: range.end,
        },),
      },);
      continue;
    }
    // CODE IS COPIED, EXCEPT THE INDENTED KIND, which MDX does not have: it reads such lines as a paragraph.
    if (node.type === 'code') {
      if (!startsAny({
        text: source,
        at: range.start,
        candidates: FENCE_MARKS,
      },))
        text.push(range,);
      continue;
    }
    // AN ANGLE AUTOLINK, which MDX reads as a tag, becomes a link; any other link keeps everything but its text.
    if ((node.type === 'link') && (source.charAt(range.start,) === '<')) {
      edits.push({
        ...range,
        replacement: linkInsteadOfAutolink({
          source,
          url: node.url,
          range,
        },),
      },);
      continue;
    }
    // A PARAGRAPH STARTING AT THE FIRST COLUMN WITH `import ` OR `export `, which MDX reads as an ES module and no
    // backslash makes plain; a character reference for the first letter does.
    if (startsEsModule({
      source,
      node,
      isTopLevel,
      range,
    },)) {
      edits.push({
        start: range.start,
        end: range.start + 1,
        replacement: `&#${String(source.codePointAt(range.start,),)};`,
      },);
    }
    if ('children' in node) {
      /**
       The node's children, the last first so they come out in order.
       */
      const children = node
        .children
        .toReversed();
      for (const child of children)
        pending.push({
          node: child,
          isTopLevel: false,
        },);
    }
  }
  return {
    text,
    edits: edits.toSorted(function byStart(
      left,
      right,
    ): number {
      return left.start - right.start;
    },),
  };
}

//endregion Source ships spans
