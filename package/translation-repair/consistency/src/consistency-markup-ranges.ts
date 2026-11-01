import { ConsistencyInvariantError, } from './consistency-invariant-error.ts';
import {
  isLinkNode,
  type LinkLabelKind,
  linkLabelEndsOpenOf,
  linkLabelKindOf,
} from './consistency-link-label.ts';
import type { TreeNode, } from '@monochromatic-dev/translation-repair-typography/ts';

//region Consistency markup ranges
// THE STRETCHES OF EIGHT KINDS OF MARKUP a parse holds. Three are read by the word test of a span edge apart from the prose
// (`consistency-shown-kinds.ts`, n160 Y2 and Y4): the images, which show a reader a picture between two words; the rows of tables,
// whose pipes and padding are written between the cells; and the content of inline code, which a reader sees as the code word it is.
// Five are read by the fold of the consistency pass (`consistency-folded-links.ts`, n168, n173): the links, references and images
// whose label shows nothing, those whose label shows marks and spaces and no letter or digit, and those whose label shows closing
// marks and nothing else (`consistency-link-label.ts`); the resource of every link; and the links with a label, with the opening
// bracket and the resource that show a reader nothing around it.

/**
 One stretch of the English, from its start to its exclusive end.
 */
export type MarkupRange = {
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
 The stretches the eight kinds of markup cover, each in the order the walk of the tree listed its nodes (no promise of position order).
 */
export type MarkupRanges = {
  /**
   The images (`![pic](u)`, `![pic][r]`).
   */
  readonly images: readonly MarkupRange[];

  /**
   The rows of tables.
   */
  readonly tableRows: readonly MarkupRange[];

  /**
   The content of inline code, between its backticks.
   */
  readonly codeContents: readonly MarkupRange[];

  /**
   The links, link references, images and image references whose label shows nothing (`[](u)`, `[](u "t")`, `[]()`, `[][r]`,
   `![](u)`), each from its first mark to its last.
   */
  readonly emptyLinks: readonly MarkupRange[];

  /**
   The links and images whose label shows marks and spaces, no letter or digit, and no closing bracket or quotation mark last
   (`[ ](u)`, `[-](u)`).
   */
  readonly markOnlyLinks: readonly MarkupRange[];

  /**
   The links and images whose label shows closing round or square brackets and closing curly double quotation marks and nothing
   else (`[\]](u)`, `[”](u)`), each from its first mark to its last.
   */
  readonly closingLinks: readonly MarkupRange[];

  /**
   The resource of every link, reference and image: from the end of the label of a link or reference to its end, and the whole of an
   image. A line break inside one is part of the markup and shows a reader nothing.
   */
  readonly linkResources: readonly MarkupRange[];

  /**
   The links and references, not the images, whose label shows letters or digits or marks, a closing mark last among them or not
   (`[kit](u)`, `[-](u)`, `[kit][r]`), each with the opening bracket it is written with and its resource (n173).
   */
  readonly labelledLinks: readonly LabelledLink[];
};

/**
 The two stretches of a link with a label that show a reader nothing, around the label that they show.
 */
export type LabelledLink = {
  /**
   The opening square bracket, before the label.
   */
  readonly opener: MarkupRange;

  /**
   The resource, from the end of the label to the end of the link: the closing bracket and the destination, the title or the
   reference.
   */
  readonly resource: MarkupRange;

  /**
   Whether the label ends in an open mark, neither a closing mark nor a letter or a digit (`[kit!](u)`, `[Kit.](u)`,
   `[kit ](u)`): the closing bracket of such a link is no closing bracket to an apostrophe after it.
   */
  readonly endsOpen: boolean;
};

/**
 What a parse that was refused holds: nothing.
 */
export const NO_MARKUP_RANGES: MarkupRanges = {
  images: [],
  tableRows: [],
  codeContents: [],
  emptyLinks: [],
  markOnlyLinks: [],
  closingLinks: [],
  linkResources: [],
  labelledLinks: [],
};

/**
 The stretch of one node.

 @param node - a node of the tree

 @returns The stretch from the start of the node to its end

 @throws {@link ConsistencyInvariantError} when the node has no start or no end offset, since the parser places every node it reads
 and a stretch left out would leave its markup in the prose

 @example
 ```ts
 const row = placeOf({ node: rowNode, },);
 ```
 */
function placeOf({ node, }: { readonly node: TreeNode; },): MarkupRange {
  /**
   Offset where the node starts, absent where the parser gave none.
   */
  const start = node.position
    ?.start
    .offset;
  /**
   Offset where the node ends, absent where the parser gave none.
   */
  const end = node.position
    ?.end
    .offset;
  if ((start === undefined) || (end === undefined)) {
    throw new ConsistencyInvariantError({
      invariant: `the ${node.type} node has no start or no end offset, and the parser places every node it reads`,
    },);
  }
  return {
    start,
    end,
  };
}

/**
 The stretches of some nodes.

 @param nodes - nodes of the tree

 @returns The stretches of the nodes, in the order the nodes were listed

 @throws {@link ConsistencyInvariantError} when a node has no start or no end offset

 @example
 ```ts
 const rows = stretchesOfNodes({ nodes: rowNodes, },);
 ```
 */
function stretchesOfNodes({ nodes, }: { readonly nodes: readonly TreeNode[]; },): readonly MarkupRange[] {
  return nodes.map(function stretchOfNode(node,): MarkupRange {
    return placeOf({ node, },);
  },);
}

/**
 The stretches of the nodes of some types.

 @param nodes - every node of the tree

 @param types - node types

 @returns The stretches of the nodes, in the order the nodes were listed

 @throws {@link ConsistencyInvariantError} when a node has no start or no end offset

 @example
 ```ts
 const rows = stretchesOfTypes({ nodes, types: ['tableRow',], },);
 ```
 */
function stretchesOfTypes(
  {
    nodes,
    types,
  }: {
    readonly nodes: readonly TreeNode[];
    readonly types: readonly string[];
  },
): readonly MarkupRange[] {
  return stretchesOfNodes({
    nodes: nodes.filter(function isOfType({ type, },): boolean {
      return types.includes(type,);
    },),
  },);
}

/**
 The stretch of the resource of a link, a reference or an image: for a link or a reference, from the end of its label
 to its end (the closing bracket, the destination and the title, or the reference), and for an image the whole of it, since an image
 shows a reader a picture and not its alt text.

 @param node - a link, a reference or an image of the tree

 @returns The stretch of the resource

 @throws {@link ConsistencyInvariantError} when the link has no start or no end offset, when its label ends in a child with no end
 offset, or when its label ends after it does, since the parser places every link and its label inside it, and a resource left
 out would leave its address in the prose

 @example
 ```ts
 const resource = resourceOfLink({ node: linkNode, },);
 ```
 */
function resourceOfLink({ node, }: { readonly node: TreeNode; },): MarkupRange {
  /**
   Where the link stands.
   */
  const {
    start,
    end,
  } = placeOf({ node, },);
  if ((node.type === 'image') || (node.type === 'imageReference')) {
    return {
      start,
      end,
    };
  }
  /**
   The children of the label, none for an empty label.
   */
  const children = ('children' in node ? (node.children as readonly TreeNode[]) : []);
  /**
   The last child of the label.
   */
  const lastChild = children.at(-1,);
  /**
   Offset where the label ends, the opening bracket's end for an empty label, absent where the last child has no end offset.
   */
  const labelEnd = (lastChild === undefined)
    ? start + 1
    : lastChild.position
      ?.end
      .offset;
  if (labelEnd === undefined) {
    throw new ConsistencyInvariantError({
      invariant: `the label of a ${node.type} node ends in a child with no end offset, and the parser places every child of a label`,
    },);
  }
  if (labelEnd > end) {
    throw new ConsistencyInvariantError({
      invariant: `the label of a ${node.type} node ends at ${String(labelEnd,)}, after the link ends at ${
        String(end,)
      }, and a label ends inside its link`,
    },);
  }
  return {
    start: labelEnd,
    end,
  };
}

/**
 The stretches of the resources of the links, references and images of a tree.

 @param nodes - every node of the tree

 @returns The stretches, in the order the nodes were listed

 @throws {@link ConsistencyInvariantError} when a link cannot be placed, as `resourceOfLink` says

 @example
 ```ts
 const resources = linkResourcesOf({ nodes, },);
 ```
 */
function linkResourcesOf({ nodes, }: { readonly nodes: readonly TreeNode[]; },): readonly MarkupRange[] {
  return nodes
    .filter(function isLink(node,): boolean {
      return isLinkNode({ node, },);
    },)
    .map(function resourceOf(node,): MarkupRange {
      return resourceOfLink({ node, },);
    },);
}

/**
 The kinds of label that show a reader something to read between the brackets: a word or a digit, or marks that close nothing.
 */
const LABELLED_KINDS: ReadonlySet<LinkLabelKind> = new Set<LinkLabelKind>([
  'marks',
  'other',
],);

/**
 The links and references of a tree whose label shows letters or digits or marks, with the two stretches of each that show a reader
 nothing: the opening square bracket and the resource (n173). A link whose label shows nothing is read whole as an empty link and one
 whose label shows closing marks alone as a closing link, so neither is listed; an image is hidden whole, so it is not either. A link
 that does not begin with a square bracket (`<https://u>`) is left out.

 @param english - the English the tree was read from, which tells a link written with square brackets from an autolink

 @param nodes - every node of the tree

 @returns The links, in the order the nodes were listed

 @throws {@link ConsistencyInvariantError} when a link cannot be placed, as `resourceOfLink` says

 @example
 ```ts
 const [link,] = labelledLinksOf({ english: '[kit](u)', nodes, },); // opener 0 to 1, resource 4 to 8
 ```
 */
function labelledLinksOf(
  {
    english,
    nodes,
  }: {
    readonly english: string;
    readonly nodes: readonly TreeNode[];
  },
): readonly LabelledLink[] {
  return nodes
    .filter(function showsLabel(node,): boolean {
      return ((node.type === 'link') || (node.type === 'linkReference'))
        && LABELLED_KINDS.has(linkLabelKindOf({ node, },),);
    },)
    .flatMap(function partsOfLink(node,): readonly LabelledLink[] {
      /**
       Where the link stands, which the parser gives every link.
       */
      const { start, } = placeOf({ node, },);
      /**
       The resource of the link.
       */
      const resource = resourceOfLink({ node, },);
      // A link that does not begin with a square bracket (an autolink) has no opening bracket to hide.
      if (english.charAt(start,) !== '[')
        return [];
      return [
        {
          opener: {
            start,
            end: start + 1,
          },
          resource,
          endsOpen: linkLabelEndsOpenOf({ node, },),
        },
      ];
    },);
}

/**
 The stretches of the links, references and images whose label shows a given kind of text.

 @param nodes - every node of the tree

 @param kind - what the label shows

 @returns The stretches of the links, in the order the nodes were listed

 @throws {@link ConsistencyInvariantError} when a link has no start or no end offset

 @example
 ```ts
 const empty = stretchesOfLabelsShowing({ nodes, kind: 'empty', },);
 ```
 */
function stretchesOfLabelsShowing(
  {
    nodes,
    kind,
  }: {
    readonly nodes: readonly TreeNode[];
    readonly kind: LinkLabelKind;
  },
): readonly MarkupRange[] {
  return stretchesOfNodes({
    nodes: nodes.filter(function showsKind(node,): boolean {
      return isLinkNode({ node, },) && (linkLabelKindOf({ node, },) === kind);
    },),
  },);
}

/**
 How many backticks a text opens with.

 @param text - the text

 @returns The count, the length of the text when it is all backticks

 @example
 ```ts
 const ticks = leadingBackticksOf({ text: '``a`b``', },); // 2
 ```
 */
function leadingBackticksOf({ text, }: { readonly text: string; },): number {
  /**
   Where the first character that is no backtick stands, none when every character is one.
   */
  const firstOther = Array.from(text,)
    .findIndex(function isNoBacktick(character,): boolean {
      return character !== '`';
    },);
  return (firstOther === (-1)) ? text.length : firstOther;
}

/**
 What pads the content of inline code on each side: `spaces-and-line-endings` for the characters CommonMark reads as a space (a space,
 and each line ending once its line endings are turned into spaces), and `spaces` for a literal space alone, which is how the
 consistency pass read it before line endings padded code and how a recount of an artifact written then reads it
 (`artifactGenerationReadsLineEndingsAsCodePadding`, `artifact-two-lane-contract.ts`).
 */
export type CodePadding = 'spaces' | 'spaces-and-line-endings';

/**
 The characters that pad the content of inline code, and that make content all padding when it holds no other, by reading.
 */
const PADDING_CHARACTERS: Readonly<Record<CodePadding, ReadonlySet<string>>> = {
  'spaces': new Set([' ',],),
  'spaces-and-line-endings': new Set([
    ' ',
    '\n',
    '\r',
  ],),
};

/**
 How many units of one side of the content of inline code are a single space or line ending, which CommonMark reads as one space.

 @param text - the content as written

 @param side - which side to read

 @param padding - what pads the content: a literal space alone, or a space and a line ending

 @returns One for a space, a line feed or a carriage return on the side, two for a carriage return followed by a line feed, none
 where the side holds none of them; under a space alone, one for a space and none otherwise

 @example
 ```ts
 const width = paddingWidthOf({ text: '\r\ntens', side: 'start', padding: 'spaces-and-line-endings', },); // 2
 ```
 */
function paddingWidthOf(
  {
    text,
    side,
    padding,
  }: {
    readonly text: string;
    readonly side: 'end' | 'start';
    readonly padding: CodePadding;
  },
): number {
  if (padding === 'spaces') {
    return (side === 'start')
      ? (text.startsWith(' ',) ? 1 : 0)
      : (text.endsWith(' ',) ? 1 : 0);
  }
  if (side === 'start') {
    if (text.startsWith(' ',) || text.startsWith('\n',))
      return 1;
    if (text.startsWith('\r\n',))
      return 2;
    return text.startsWith('\r',) ? 1 : 0;
  }
  if (text.endsWith(' ',) || text.endsWith('\r',))
    return 1;
  if (text.endsWith('\r\n',))
    return 2;
  return text.endsWith('\n',) ? 1 : 0;
}

/**
 The content of an inline code stretch as CommonMark shows it: the stretch without the backticks that open it and the same number that
 close it, and without one space on each side when the content begins and ends with a space and is not all spaces (the padding that
 lets code begin or end with a backtick). CommonMark turns each line ending of the content into a space first, so a line ending (a
 line feed, a carriage return or the two together) pads a side as a space does. Where only one side holds a space or a line ending,
 or the content is spaces and line endings alone, every one of them is shown. Under the padding of spaces alone (an artifact
 written before line endings padded code), a literal space is all that pads and a line ending is shown.

 @param english - the English the tree was read from

 @param range - the stretch of the whole code node, backticks included

 @param codePadding - what pads the content: a literal space alone, or a space and a line ending

 @returns The stretch of the content

 @example
 ```ts
 const content = codeContentOf({ english: 'a `kit` b', range: { start: 2, end: 7, }, codePadding: 'spaces', },); // 3 to 6
 const padded = codeContentOf({ english: 'a ` kit ` b', range: { start: 2, end: 9, }, codePadding: 'spaces', },); // 4 to 7
 ```
 */
function codeContentOf(
  {
    english,
    range,
    codePadding,
  }: {
    readonly english: string;
    readonly range: MarkupRange;
    readonly codePadding: CodePadding;
  },
): MarkupRange {
  /**
   How many backticks open the code, which as many close.
   */
  const ticks = leadingBackticksOf({
    text: english.slice(
      range.start,
      range.end,
    ),
  },);
  /**
   Where the content starts, never past the end of the code.
   */
  const start = Math.min(
    range.start + ticks,
    range.end,
  );
  /**
   Where the content ends, never before where it starts.
   */
  const end = Math.max(
    range.end - ticks,
    start,
  );
  /**
   The content between the backticks, as written.
   */
  const written = english.slice(
    start,
    end,
  );
  /**
   How many units of the content pad its start and its end: one for a space or a line feed or a carriage return, two for a carriage
   return and a line feed, none where the side holds neither.
   */
  const widths = {
    start: paddingWidthOf({
      text: written,
      side: 'start',
      padding: codePadding,
    },),
    end: paddingWidthOf({
      text: written,
      side: 'end',
      padding: codePadding,
    },),
  };
  /**
   The characters that pad the content under this reading.
   */
  const paddingCharacters = PADDING_CHARACTERS[codePadding];
  /**
   Whether the content holds a character that pads nothing: no space, and under the reading by line endings no line ending either.
   */
  const holdsNoSpace = Array.from(written,)
    .some(function isNoSpace(character,): boolean {
      return !paddingCharacters.has(character,);
    },);
  /**
   Whether one space or line ending pads each side: both ends are spaces or line endings and the content holds a character that is
   neither.
   */
  const padded = (widths.start > 0)
    && (widths.end > 0)
    && holdsNoSpace;
  return padded
    ? {
      start: start + widths.start,
      end: end - widths.end,
    }
    : {
      start,
      end,
    };
}

/**
 The stretches of the images, of the table rows and of the content of inline code a tree holds.

 @param english - the English the tree was read from

 @param nodes - every node of the tree

 @param codePadding - what pads the content of inline code: a space and a line ending as CommonMark reads it, or a literal space alone
 as an artifact written before line endings padded code was read; a space and a line ending where absent

 @returns The stretches of the three kinds

 @example
 ```ts
 const { images, } = markupRangesOf({ english: 'a ![pic](u) b', nodes: [], },);
 ```
 */
export function markupRangesOf(
  {
    english,
    nodes,
    codePadding = 'spaces-and-line-endings',
  }: {
    readonly english: string;
    readonly nodes: readonly TreeNode[];
    readonly codePadding?: CodePadding;
  },
): MarkupRanges {
  return {
    images: stretchesOfTypes({
      nodes,
      types: [
        'image',
        'imageReference',
      ],
    },),
    tableRows: stretchesOfTypes({
      nodes,
      types: ['tableRow',],
    },),
    codeContents: stretchesOfTypes({
      nodes,
      types: ['inlineCode',],
    },)
      .map(function contentOf(range,): MarkupRange {
        return codeContentOf({
          english,
          range,
          codePadding,
        },);
      },),
    emptyLinks: stretchesOfLabelsShowing({
      nodes,
      kind: 'empty',
    },),
    markOnlyLinks: stretchesOfLabelsShowing({
      nodes,
      kind: 'marks',
    },),
    closingLinks: stretchesOfLabelsShowing({
      nodes,
      kind: 'closing',
    },),
    linkResources: linkResourcesOf({ nodes, },),
    labelledLinks: labelledLinksOf({
      english,
      nodes,
    },),
  };
}

//endregion Consistency markup ranges
