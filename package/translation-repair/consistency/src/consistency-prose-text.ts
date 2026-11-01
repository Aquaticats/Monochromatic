import type { Nodes, } from 'mdast';

import {
  type CodePadding,
  type LabelledLink,
  markupRangesOf,
  NO_MARKUP_RANGES,
} from './consistency-markup-ranges.ts';
import { withDefinitions, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  type DeepReadonlyData,
  parseSliceBody,
  requireMdxRefusal,
} from '@monochromatic-dev/translation-repair-document-structure/ts';

//region Consistency prose text
// THE PROSE OF AN ENGLISH, with every other stretch blanked and no offset moved.
// A reader reads the words of a page, and a single quote that stands in the string
// of a component (`photos={['a.webp']}`), in code, in a comment or in the
// destination of a link is no quotation mark of those words. The reading of which
// single marks are apostrophes (`consistency-proven-apostrophes.ts`) looks for a
// partner of a mark in the whole English, so it must not find one in a stretch the
// page does not read as words. The prose here is what the Markdown grammar of the
// slice (`parse-slice-body.ts`) calls text: the stretches of the English that its
// text nodes cover, with a link's destination, an address written bare, code, a
// component's markup and attributes, a comment, an expression and the markers of
// emphasis left out. Each unit of a stretch left out becomes a space, except a line
// break, which stays, so the result is as long as the English, has its lines where
// they were, and every offset of the one is the offset of the other.
//
// THE DELIMITERS OF EMPHASIS COME FROM THE SAME PARSE (`emphasisDelimitersOf`). The emphasis,
// strong and delete nodes of the tree end where the grammar says they end, so the characters
// that open and close them are known by offset, and a delimiter that closes is one a node ends
// on. A lone asterisk or underscore that opens nothing and closes nothing is no delimiter: it
// is a character of the prose, kept as it is. Readers that need to know whether a mark stands
// after a closing emphasis (`consistency-closing-marks.ts`, `consistency-word-joints.ts`) take the
// answer from here and not from the shape of the characters around it. An English the grammar
// refuses has no delimiters.
//
// A PAGE THE GRAMMAR REFUSES IS READ AS TEXT. The grammar is strict and an English
// with a stray brace is refused. Then nothing can be said of which stretch is prose,
// and the English is returned whole: every mark is a possible partner, which can only
// keep a mark from being proven and never prove one it should not.

/**
 A node of the tree the slice grammar reads, which it hands over read-only.
 */
type TreeNode = DeepReadonlyData<Nodes>;

/**
 Every node of a tree, walked with a work stack rather than by recursion, since a tree is as deep as the nesting a page
 writes.

 @param root - the tree's top node

 @returns The tree's nodes, in no promised order

 @example
 ```ts
 const nodes = nodesOf({ root, },);
 ```
 */
function nodesOf({ root, }: { readonly root: TreeNode; },): readonly TreeNode[] {
  /**
   Nodes met so far, which is what the function returns.
   */
  const met: TreeNode[] = [];
  /**
   Nodes still to visit.
   */
  const pending: TreeNode[] = [root,];
  // Each node in turn, until the stack is empty.
  for (let node = pending.pop(); node !== undefined; node = pending.pop()) {
    met.push(node,);
    if ('children' in node)
      pending.push(...(node.children as readonly TreeNode[]),);
  }
  return met;
}

/**
 One stretch of the English that is prose.
 */
type ProseRange = {
  /**
   Offset where the stretch starts.
   */
  readonly start: number;

  /**
   Exclusive offset where it ends.
   */
  readonly end: number;
};

/**
 Where a node of the Markdown tree stands in the English it was read from.

 @param node - a node of the tree

 @returns The stretch as a list of zero or one; none for a node the parser gave no position

 @example
 ```ts
 const [stretch,] = stretchOf({ node, },);
 ```
 */
function stretchOf({ node, }: { readonly node: TreeNode; },): readonly ProseRange[] {
  /**
   Where the parser put the node.
   */
  const { position, } = node;
  if (position === undefined)
    return [];
  /**
   Offset where the node starts.
   */
  const { offset: start, } = position.start;
  /**
   Offset where the node ends.
   */
  const { offset: end, } = position.end;
  return ((start === undefined) || (end === undefined))
    ? []
    : [
      {
        start,
        end,
      },
    ];
}

/**
 The stretches of the English that its text nodes cover, in the order they stand.

 @param english - the English the tree was read from

 @param nodes - every node of the tree

 @returns The stretches, text inside a link whose source is a bare address left out

 @example
 ```ts
 const ranges = proseRangesOf({ english, nodes: nodesOf({ root, },), },);
 ```
 */
function proseRangesOf(
  {
    english,
    nodes,
  }: {
    readonly english: string;
    readonly nodes: readonly TreeNode[];
  },
): readonly ProseRange[] {
  /**
   The stretches of the links written as a bare address or in angle brackets, which hold their destination as their text.
   */
  const barred = nodes
    .filter(function isLink(node,): boolean {
      return node.type === 'link';
    },)
    .flatMap(function stretchOfLink(node,): readonly ProseRange[] {
      return stretchOf({ node, },)
        .filter(function startsWithoutBracket({ start, },): boolean {
          return english.charAt(start,) !== '[';
        },);
    },);
  return nodes
    .filter(function isText(node,): boolean {
      return node.type === 'text';
    },)
    .flatMap(function stretchOfText(node,): readonly ProseRange[] {
      return stretchOf({ node, },);
    },)
    .filter(function isOutsideBareLinks(text,): boolean {
      return !barred.some(function holdsText(link,): boolean {
        return (link.start <= text.start) && (text.end <= link.end);
      },);
    },)
    .toSorted(function inOrder(
      left,
      right,
    ): number {
      return left.start - right.start;
    },);
}

/**
 The Markdown tree of an English.

 THE `try` COVERS ALL OF `parseSliceBody`, which is reading a slice body: its three offset-preserving masks (invisible
 lines, HTML comments, lone container tags) and then the parse. The engine's stack exhaustion is read as the grammar's
 refusal at the parse call inside `parseSliceBody` and nowhere else, so a failure of a mask, which is no refusal, propagates.

 @param english - the English of a slice

 @param definitions - the page's definitions (`pageDefinitionsOf`), parsed after the English so a reference in it resolves; empty
 for an English read alone

 @returns The root of the tree as a list of one, none when the grammar refuses the English

 @throws {@link Error} when reading fails for a reason other than the grammar's own refusal

 @example
 ```ts
 const [root,] = treeOf({ english: 'Fill the bowl.', definitions: '', },);
 ```
 */
function treeOf(
  {
    english,
    definitions,
  }: {
    readonly english: string;
    readonly definitions: string;
  },
): readonly TreeNode[] {
  try {
    /**
     The parse of the English, with the definitions after it.
     */
    const { root, } = parseSliceBody({
      text: withDefinitions({
        text: english,
        definitions,
      },),
    },);
    return [root,];
  }
  catch (error) {
    // Only the grammar's own refusal is an English to be read as text; anything else keeps propagating.
    requireMdxRefusal({ error, },);
    return [];
  }
}

/**
 A stretch left out of the prose: every unit a space and every line break kept, so the
 lines of the English stay where they were.

 @param text - the stretch

 @returns Text as long as the stretch with only spaces and line breaks in it

 @example
 ```ts
 const blank = blankedOf({ text: '``\nit\n``', },); // '  \n  \n  '
 ```
 */
export function blankedOf({ text, }: { readonly text: string; },): string {
  return text
    .split('\n',)
    .map(function blankLine(line,): string {
      return ' '.repeat(line.length,);
    },)
    .join('\n',);
}

/**
 The offsets of the delimiters of emphasis, strong emphasis and strikethrough in an English, by the parse of the Markdown
 grammar: the characters `*`, `_` and `~` that open or close such a stretch, which the prose blanks. A lone asterisk, an
 underscore inside a word and a mark in code are characters of the text, no delimiter.
 */
export type EmphasisDelimiters = {
  /**
   Offsets of every delimiter, opening and closing.
   */
  readonly offsets: ReadonlySet<number>;

  /**
   Offsets of the closing delimiters alone: the last unit of an emphasis, the last two of strong emphasis, the last one or
   two of strikethrough.
   */
  readonly closing: ReadonlySet<number>;
};

/**
 What the grammar read of an English, the grammar refusing it, no stretch is a delimiter.
 */
const NO_DELIMITERS: EmphasisDelimiters = {
  offsets: new Set(),
  closing: new Set(),
};

/**
 The node types that hold a stretch between delimiters.
 */
const DELIMITED_NODE_TYPES: ReadonlySet<string> = new Set([
  'emphasis',
  'strong',
  'delete',
],);

/**
 The characters a delimiter is made of.
 */
const DELIMITER_CHARACTERS: ReadonlySet<string> = new Set([
  '*',
  '_',
  '~',
],);

/**
 How many units one delimiter of a node takes at either end.

 @param english - the English the tree was read from

 @param node - an emphasis, strong or strikethrough node

 @param start - offset where the node starts

 @returns Two for strong emphasis, one for emphasis, and for strikethrough the length of the tilde run the node starts with, one or two

 @example
 ```ts
 const width = delimiterWidthOf({ english: '~~cat~~', node, start: 0, },); // 2
 ```
 */
function delimiterWidthOf(
  {
    english,
    node,
    start,
  }: {
    readonly english: string;
    readonly node: TreeNode;
    readonly start: number;
  },
): number {
  if (node.type === 'strong')
    return 2;
  if (node.type === 'emphasis')
    return 1;
  return (english.charAt(start + 1,) === '~') ? 2 : 1;
}

/**
 The delimiters of every emphasis, strong and strikethrough node of a tree.

 @param english - the English the tree was read from

 @param nodes - every node of the tree

 @returns The offsets of the delimiters, the closing ones apart

 @example
 ```ts
 const { closing, } = delimitersOf({ english: 'a *b* c', nodes, },); // holds 4
 ```
 */
function delimitersOf(
  {
    english,
    nodes,
  }: {
    readonly english: string;
    readonly nodes: readonly TreeNode[];
  },
): EmphasisDelimiters {
  /**
   Every delimiter, mutated only inside this function.
   */
  const offsets = new Set<number>();
  /**
   The closing delimiters, mutated only inside this function.
   */
  const closing = new Set<number>();
  for (const node of nodes) {
    if (!DELIMITED_NODE_TYPES.has(node.type,))
      continue;
    /**
     The stretch the node covers in the English, none for a node with no position.
     */
    const [stretch,] = stretchOf({ node, },);
    if (stretch === undefined)
      continue;
    /**
     How many units a delimiter of this node takes.
     */
    const width = delimiterWidthOf({
      english,
      node,
      start: stretch.start,
    },);
    for (let step = 0; step < width; step += 1) {
      /**
       Offset of the step-th unit of the opening delimiter.
       */
      const opening = stretch.start + step;
      /**
       Offset of the step-th unit of the closing delimiter, counted from the end.
       */
      const closer = (stretch.end - 1) - step;
      if (DELIMITER_CHARACTERS.has(english.charAt(opening,),))
        offsets.add(opening,);
      if (DELIMITER_CHARACTERS.has(english.charAt(closer,),)) {
        offsets.add(closer,);
        closing.add(closer,);
      }
    }
  }
  return {
    offsets,
    closing,
  };
}

/**
 What one parse of an English gives: its prose, and the delimiters of its emphasis.
 */
export type ProseView = {
  /**
   The prose, as long as the English, with every other stretch blanked.
   */
  readonly prose: string;

  /**
   The stretches the prose keeps, in order and cut off where the English ends. Everything else is blanked, which an English the
   grammar refuses has none of.
   */
  readonly stretches: readonly ProseRange[];

  /**
   Whether the grammar refused the English, so no parse says which stretch is prose and the prose is the English itself.
   */
  readonly refused: boolean;

  /**
   The delimiters of emphasis, strong emphasis and strikethrough, which the prose blanks.
   */
  readonly emphasis: EmphasisDelimiters;

  /**
   The stretches of the images (`![pic](u)`, `![pic][r]`), none where the grammar refuses the English. An image shows a reader a
   picture between the words around it.
   */
  readonly images: readonly ProseRange[];

  /**
   The stretches of the rows of tables, none where the grammar refuses the English. The pipes and the padding of a row are
   written between the cells and show a reader the cell's edge.
   */
  readonly tableRows: readonly ProseRange[];

  /**
   The stretches of the content of inline code, between its backticks, none where the grammar refuses the English. A reader sees
   the code word.
   */
  readonly codeContents: readonly ProseRange[];

  /**
   The stretches of the links, references and images whose label shows nothing (`[](u)`, `[]()`, `![](u)`), none where the grammar
   refuses the English. The text on each side of one meets as if it were not there.
   */
  readonly emptyLinks: readonly ProseRange[];

  /**
   The stretches of the links and images whose label shows marks and spaces and no letter or digit (`[-](u)`), none where the
   grammar refuses the English.
   */
  readonly markOnlyLinks: readonly ProseRange[];

  /**
   The stretches of the links and images whose label shows closing round or square brackets and closing curly double quotation
   marks and nothing else (`[\]](u)`), none where the grammar refuses the English.
   */
  readonly closingLinks: readonly ProseRange[];

  /**
   The stretches of the resources of the links, references and images: from the end of a label to the end of the link, and the whole
   of an image, none where the grammar refuses the English. A line break inside one is part of the link.
   */
  readonly linkResources: readonly ProseRange[];

  /**
   The links and references whose label shows letters or digits or marks (`[kit](u)`, `[-](u)`, `[kit][r]`), each with the stretch
   of its opening square bracket and the stretch of its resource, none where the grammar refuses the English. The label is what a
   reader sees there and the two stretches are not (n173).
   */
  readonly labelledLinks: readonly LabelledLink[];
};

/**
 The prose of an English and the delimiters of its emphasis, read from one parse.

 @param english - the English of a slice

 @param definitions - the page's definitions (`pageDefinitionsOf`), none for an English read alone: a reference in the English
 whose definition stands in another slice is then plain text

 @param codePadding - what pads the content of inline code: a space and a line ending as CommonMark reads it, or a literal space alone
 as an artifact written before line endings padded code was read; a space and a line ending where absent

 @returns The prose (the English itself where the grammar refuses it), the stretches it keeps and the delimiters (none where the
 grammar refuses)

 @example
 ```ts
 const { prose, emphasis, } = proseViewOf({ english: 'The *kittens*’ bowl.', },);
 ```
 */
export function proseViewOf(
  {
    english,
    definitions = '',
    codePadding = 'spaces-and-line-endings',
  }: {
    readonly english: string;
    readonly definitions?: string;
    readonly codePadding?: CodePadding;
  },
): ProseView {
  /**
   The tree of the English, none when the grammar refuses it.
   */
  const [root,] = treeOf({
    english,
    definitions,
  },);
  if (root === undefined) {
    return {
      prose: english,
      stretches: [],
      refused: true,
      emphasis: NO_DELIMITERS,
      ...NO_MARKUP_RANGES,
    };
  }
  /**
   Every node of the tree.
   */
  const nodes = nodesOf({ root, },);
  /**
   Pieces of the result: the prose between the blanks, and the blanks.
   */
  const pieces: string[] = [];
  /**
   Offset up to which the pieces hold the English.
   */
  const cursor = { at: 0, };
  /**
   The stretches the prose keeps, mutated only inside this function.
   */
  const stretches: ProseRange[] = [];
  for (const range of proseRangesOf({
    english,
    nodes,
  },)) {
    /**
     Offset the prose of this stretch starts at, past anything an earlier stretch covered.
     */
    const from = Math.max(
      range.start,
      cursor.at,
    );
    /**
     Where the stretch ends, cut off where the English ends: the parse also covers the definitions after it.
     */
    const stop = Math.min(
      range.end,
      english.length,
    );
    if (from < stop) {
      stretches.push({
        start: from,
        end: stop,
      },);
    }
    pieces.push(
      blankedOf({
        text: english.slice(
          cursor.at,
          from,
        ),
      },),
      english.slice(
        from,
        range.end,
      ),
    );
    cursor.at = Math.max(
      cursor.at,
      range.end,
    );
  }
  pieces.push(blankedOf({ text: english.slice(cursor.at,), },),);
  return {
    prose: pieces.join('',),
    stretches,
    refused: false,
    emphasis: delimitersOf({
      english,
      nodes,
    },),
    ...markupRangesOf({
      english,
      nodes,
      codePadding,
    },),
  };
}

/**
 How many English texts the readers of one parse remember: the places of one slice ask for the delimiters and for the units not
 read as words of the one English, and each is parsed once and not again for the next place.
 */
const REMEMBERED_VIEWS = 4;

/**
 One English whose parse was read, and what the parse made of it.
 */
type RememberedView = {
  /**
   The English the view was read from.
   */
  readonly english: string;

  /**
   The definitions it was read with.
   */
  readonly definitions: string;

  /**
   The padding of inline code it was read with.
   */
  readonly codePadding: CodePadding;

  /**
   What the parse made of it.
   */
  readonly view: ProseView;
};

/**
 The views of the most recent English texts, the newest last. They hold only what is read from the English, so remembering them
 changes no answer.
 */
const rememberedViews: RememberedView[] = [];

/**
 The prose view of an English, parsed once for each of the English texts most recently asked about.

 @param english - the English of a slice

 @param definitions - the page's definitions (`pageDefinitionsOf`), none for an English read alone

 @param codePadding - what pads the content of inline code, which is part of what the view holds and so of what is remembered; a space
 and a line ending where absent

 @returns What `proseViewOf` gives for it

 @example
 ```ts
 const { prose, } = rememberedViewOf({ english: 'a *b*’s c', },);
 ```
 */
export function rememberedViewOf(
  {
    english,
    definitions = '',
    codePadding = 'spaces-and-line-endings',
  }: {
    readonly english: string;
    readonly definitions?: string;
    readonly codePadding?: CodePadding;
  },
): ProseView {
  /**
   The view read before for an equal English, equal definitions and an equal padding of code, none when there is none.
   */
  const known = rememberedViews.find(function isSame(entry,): boolean {
    return (entry.english === english)
      && (entry.definitions === definitions)
      && (entry.codePadding === codePadding);
  },);
  if (known !== undefined)
    return known.view;
  /**
   What the parse makes of the English.
   */
  const view = proseViewOf({
    english,
    definitions,
    codePadding,
  },);
  rememberedViews.push({
    english,
    definitions,
    codePadding,
    view,
  },);
  if (rememberedViews.length > REMEMBERED_VIEWS)
    rememberedViews.shift();
  return view;
}

/**
 The delimiters of the emphasis of an English, by the parse.

 @param english - the English of a slice

 @param definitions - the page's definitions (`pageDefinitionsOf`), none for an English read alone: a reference in the English whose
 definition stands in another slice is then plain text, and the brackets of a link that it is not stop its label from holding a
 delimiter of emphasis that the link would hold (n173)

 @returns The offsets of the delimiters, the closing ones apart; none where the grammar refuses the English

 @example
 ```ts
 const { closing, } = emphasisDelimitersOf({ english: 'a *b*’s c', },); // holds 4
 ```
 */
export function emphasisDelimitersOf(
  {
    english,
    definitions = '',
  }: {
    readonly english: string;
    readonly definitions?: string;
  },
): EmphasisDelimiters {
  // An English with none of the three characters has no delimiter, so the parse is not made for it.
  if (![...DELIMITER_CHARACTERS,].some(function isHeld(character,): boolean {
    return english.includes(character,);
  },))
    return NO_DELIMITERS;
  /**
   What one parse of the English makes of it, remembered for the next place that asks.
   */
  const view = rememberedViewOf({
    english,
    definitions,
  },);
  return view.emphasis;
}

//endregion Consistency prose text
