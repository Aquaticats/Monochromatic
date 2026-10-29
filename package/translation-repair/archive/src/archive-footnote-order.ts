import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import {
  activeFootnoteMarkers,
  parseDocument,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  type DocumentNode,
  normalizeFootnoteIdentifier,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  type FootnoteProtectedRange,
  overlapsFootnoteProtection,
} from '@monochromatic-dev/translation-repair-page-pass/ts';
import { definitionLabelsOf, } from '@monochromatic-dev/translation-repair-stage/ts';

//region Archive footnote definition order
// THE ARCHIVE'S DEFINITIONS STAND IN THE ORIGINAL'S ORDER once their labels
// are the original's, so the block pairing, which never steps backwards, pairs
// each with its counterpart and the slices carry both. A page renders its
// notes by reference order, so moving the definition blocks changes nothing a
// reader sees. Only a contiguous run of definitions is reordered; a page
// whose definitions are interleaved with prose keeps its shape, named.

/**
 What reordering the definitions did.

 @example
 ```ts
 const reordered: ReorderedDefinitions = reorderFootnoteDefinitions({ text, order: [ '1', '2', ], },);
 ```
 */
export type ReorderedDefinitions = {
  /**
   Text with the definitions in the asked order, or as it came.
   */
  readonly text: string;

  /**
   Whether the text moved.
   */
  readonly changed: boolean;

  /**
   Why the text stands when it does and the order asked for differs.
   */
  readonly note?: string;

  /**
   Caller must withhold the entire composed operation, including any preceding rename.
   */
  readonly blockedByProtection?: true;
};

/**
 One definition block with its place in the order asked for.
 */
type RankedDefinition = {
  /**
   The block.
   */
  readonly node: DocumentNode;

  /**
   Its label's place in the order asked for, or past the end in document
   order for a label the order does not name.
   */
  readonly rank: number;
};

/**
 Zone a parsed footnote definition block carries.
 */
const DEFINITION_ZONE = 'footnote-definition';

/**
 Openers of the two comment forms a page carries, HTML and JSX.
 */
const COMMENT_OPENERS = [
  '<!--',
  '{/*',
] as const;

/**
 Characters a tail may hold before its first content: spaces, tabs and line ends.
 */
const BLANK_CHARACTERS = ' \t\r\n';

/**
 Columns a tab counts as in an indent.
 */
const TAB_COLUMNS = 4;

/**
 Columns of indent at which a line continues the definition above it.
 */
const CONTINUATION_INDENT = 4;

/**
 Labels of a text's definitions, in the order the text carries them.

 @param text - document to read

 @returns Labels in document order

 @example
 ```ts
 definitionLabelOrder({ text: sourceText, },);
 // => ['1', '2']
 ```
 */
export function definitionLabelOrder(
  { text, }: { readonly text: string; },
): readonly string[] {
  return activeFootnoteMarkers({ text, },)
    .filter(function definition(marker,): boolean {
      return marker.kind === 'definition';
    },)
    .map(function label(marker,): string { return marker.rawLabel; },);
}

/**
 Whether the text after a definition's last block opens with a comment that
 continues that definition.

 A COMMENT NO READER SEES ENDS NO BLOCK. The parse masks an HTML comment to
 whitespace and reads a JSX comment as a block of its own, so a note on the
 line under a definition, or in its indented continuation paragraph, lies
 after the block's end and moves with nothing. Such a comment continues the
 definition when it starts on the same line or the next one, or when it is
 indented four columns, a tab counting as four, as a continuation paragraph
 is. A comment on a line of its own past a blank line and indented less is a
 block of its own: a note closing the page after the last footnote stays where
 it stands. Content that is no comment is another block, whatever line it
 starts on.

 @param tail - text from the last definition's end to the text's end

 @returns Whether the first content in the tail is a comment that continues
 the definition; a tail of whitespace alone does not

 @example
 ```ts
 tailContinuesDefinition({ tail: '\n  <!-- 大家看一下 -->\n', },); // true
 tailContinuesDefinition({ tail: '\n\n<!-- 大家看一下 -->\n', },); // false
 ```
 */
function tailContinuesDefinition({ tail, }: { readonly tail: string; },): boolean {
  /**
   Offset of the first character that is no space, tab or line end, minus one
   when the tail holds none; every character before it is one code unit.
   */
  const contentAt = Array.from(tail,)
    .findIndex(function isContent(character,): boolean {
      return !BLANK_CHARACTERS.includes(character,);
    },);
  if (contentAt === (-1))
    return false;
  if (!COMMENT_OPENERS.some(function opensAt(opener,): boolean {
    return tail.startsWith(
      opener,
      contentAt,
    );
  },))
    return false;
  /**
   The blank stretch before the comment.
   */
  const lead = tail.slice(
    0,
    contentAt,
  );
  /**
   Columns the comment's line is indented.
   */
  const indent = Array.from(lead.slice(lead.lastIndexOf('\n',) + 1,),)
    .reduce(
    function columns(
      sum,
      character,
    ): number {
      return sum + ((character === '\t') ? TAB_COLUMNS : 1);
    },
    0,
    );
  /**
   Line feeds before the comment, a blank line being two.
   */
  const breaks = lead.split('\n',)
    .length
    - 1;
  return (breaks < 2) || (indent >= CONTINUATION_INDENT);
}

/**
 Moves a text's footnote definition blocks into the order of their labels
 in `order`, labels the order does not name keeping their place after the
 ones it does.

 @param text - document whose definitions move

 @param order - labels in the order wanted

 @param protectedRanges - original-English intervals in current text coordinates

 @returns Text, moved or standing, and why it stands: a move that would
 change how the page parses its definitions, or leave a comment that continues
 the last definition behind, stands with a note

 @throws FootnoteRewriteError when the moved text's markers disagree with the
 parser's reading of them

 @example
 ```ts
 reorderFootnoteDefinitions({ text: 'A[^1] B[^2].\n\n[^2]: two\n\n[^1]: one\n', order: [ '1', '2', ], },);
 // => { text: 'A[^1] B[^2].\n\n[^1]: one\n\n[^2]: two\n', changed: true, }
 ```
 */
export function reorderFootnoteDefinitions(
  {
    text,
    order,
    protectedRanges = [],
  }: {
    readonly text: string;
    readonly order: readonly string[];
    readonly protectedRanges?: readonly FootnoteProtectedRange[];
  },
): ReorderedDefinitions {
  /**
   Every block of the text.
   */
  const {
    nodes,
    containers,
  } = parseDocument({ text, },);
  /**
   The definition blocks, in document order.
   */
  const definitions = nodes.filter(function isDefinition(node,): boolean {
    return node.zone === DEFINITION_ZONE;
  },);
  /**
   The first definition, where the region that moves begins.
   */
  const [first, second,] = definitions;
  /**
   The last definition, where that region ends.
   */
  const last = definitions.at(-1,);
  if ((first === undefined)
    || (second === undefined)
    || (last === undefined))
    return {
      text,
      changed: false,
    };
  // ORDER FIRST (ledger B55): definitions that already follow the order need
  // no move, so no note saying a move was withheld; the checks that refuse a
  // move come after, where a move is wanted. Read after them, an archive whose
  // definitions already followed the original's carried "definitions stand"
  // as a finding whenever prose sat among them.
  /**
   Original label ranks use the same normalization as correspondence and rewriting.
   */
  const normalizedOrder = order.map(function key(label,): string {
    return normalizeFootnoteIdentifier({ identifier: label, },);
  },);
  /**
   Each definition with its rank.
   */
  const ranked = definitions.map(function toRanked(
    node,
    index,
  ): RankedDefinition {
    /**
     The block's label, empty when it opens with none.
     */
    const label = definitionLabelsOf({ node, },)[0] ?? '';
    /**
     Its place in the order asked for, absent as -1.
     */
    const place = normalizedOrder.indexOf(normalizeFootnoteIdentifier({ identifier: label, },),);
    return {
      node,
      rank: (place === (-1)) ? (order.length + index) : place,
    };
  },);
  /**
   The definitions in the order asked for.
   */
  const sorted = ranked.toSorted(function byRank(
    left,
    right,
  ): number {
    return left.rank - right.rank;
  },);
  if (sorted.every(function stays(
    entry,
    index,
  ): boolean {
    return entry.node === definitions[index];
  },))
    return {
      text,
      changed: false,
    };
  /**
   Whether a block that is not a definition sits among them.
   */
  const interleaved = nodes.some(function sitsAmong(node,): boolean {
    return (node.zone !== DEFINITION_ZONE)
      && (node.startOffset >= first.startOffset)
      && (node.startOffset < last.endOffset);
  },);
  if (interleaved)
    return {
      text,
      changed: false,
      note: 'the footnote definitions are interleaved with other blocks, so they keep their order',
    };
  /**
   Containers crossing definition boundaries cannot move as independent block fragments.
   */
  const splitContainer = containers.some(function crosses(container,): boolean {
    return (container.openerStartOffset < last.endOffset) && (container.closerEndOffset > first.startOffset)
      && (!definitions.some(function contains(node,): boolean {
        return (node.startOffset <= container.openerStartOffset) && (node.endOffset >= container.closerEndOffset);
      },));
  },);
  if (splitContainer)
    return {
      text,
      changed: false,
      note: 'footnote definitions share container delimiters, so they keep their order',
    };
  if (overlapsFootnoteProtection({
    startOffset: first.startOffset,
    endOffset: last.endOffset,
    protectedRanges,
  },))
    return {
      text,
      changed: false,
      blockedByProtection: true,
      note: 'definition movement would touch protected English-original bytes or their declaration',
    };
  /**
   Every original gap is preserved once, in its existing separator position.
   */
  const gaps = definitions.slice(1,)
    .map(function gap(
      node,
      index,
    ): string {
      return text.slice(
        nonNullishOrThrow(definitions[index],)
          .endOffset,
        node.startOffset,
      );
    },);
  if (gaps.some(function containsContent(gap,): boolean {
    for (const character of gap) {
      if (!' \t\r\n'.includes(character,))
        return true;
    }
    return false;
  },))
    return {
      text,
      changed: false,
      note: 'footnote definition gaps contain non-blank content, so they keep their order',
    };
  // THE TEXT AFTER THE LAST DEFINITION IS READ LIKE THE GAPS: a note under the
  // last definition ends no block, so it would stay behind in the tail and
  // follow whichever definition the move puts last.
  if (tailContinuesDefinition({ tail: text.slice(last.endOffset,), },))
    return {
      text,
      changed: false,
      note: 'a comment continues the last footnote definition past its block, so the definitions keep their order',
    };
  /**
   Changed definitions, with unrelated separator bytes neither dropped nor repeated.
   */
  const region = sorted.map(function placed(
    entry,
    index,
  ): string {
    return `${index === 0 ? '' : nonNullishOrThrow(gaps[index - 1],)}${text.slice(
      entry.node
        .startOffset,
      entry.node
        .endOffset,
    )}`;
  },)
    .join('',);
  /**
   Candidate stays local until syntax and every complete definition block are rechecked.
   */
  const rewritten = `${text.slice(
    0,
    first.startOffset,
  )}${region}${text.slice(last.endOffset,)}`;
  activeFootnoteMarkers({ text: rewritten, },);
  /**
   Actual output definition boundaries must still match the complete copied blocks.
   */
  const reparsed = parseDocument({ text: rewritten, },)
    .nodes
    .filter(function definition(node,): boolean {
      return node.zone === DEFINITION_ZONE;
    },);
  // A MOVE THAT WOULD CHANGE THE PARSE IS REFUSED LIKE ANY OTHER (ledger T8,
  // eighteenth batch): the definitions stand, said in a note, and the rename
  // before it stands too. It once threw, which kept every relabel of the page
  // back with it; a bounded probe over generated pages read 156 such moves in
  // 2,000, one shape being a definition whose fence was written unindented,
  // whose lines a paragraph definition moved after it takes as its own.
  if ((reparsed.length !== sorted.length) || reparsed.some(function differs(
    node,
    index,
  ): boolean {
    return node.text
      !== sorted[index]
        ?.node
        .text;
  },))
    return {
      text,
      changed: false,
      note: 'moving the footnote definitions would change how the page parses them, so they keep their order',
    };
  return {
    text: rewritten,
    changed: true,
  };
}

//endregion Archive footnote definition order
