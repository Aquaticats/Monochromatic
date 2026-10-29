import type {
  Root,
  RootContent,
} from 'mdast';
import { FootnoteRewriteError, } from './footnote-rewrite-error.ts';
import {
  gfmMarkerSpans,
  type GfmMarkerSpan,
} from './gfm-marker-spans.ts';
import { headingFloorOf, } from './heading-floor-store.ts';
import type { DeepReadonlyData, } from './readonly-data.ts';

//region Unpositioned runs
// The autolink-literal transform of `mdast-util-gfm-autolink-literal` replaces
// one text node with text and link nodes that carry no position, wherever its
// pattern finds a literal micromark did not tokenize (ledger B123). This module
// places such a run in the raw text and says which marker lexemes of that raw
// lie in the run's text, as against a link's own URL. The footnote marker
// reader (`active-footnote-markers.ts`) and the document's footnote graph
// (`footnote-graph.ts`) both read runs through it.

/**
 Any node of the parsed tree, its root among them.

 @example
 ```ts
 const work: TreeNode[] = [root,];
 ```
 */
export type TreeNode = DeepReadonlyData<Root | RootContent>;

/**
 Raw span in the masked parser input, the end exclusive.

 @example
 ```ts
 const span: RawBounds = { start: 0, end: 4, };
 ```
 */
type RawBounds = {
  /**
   First offset of the span.
   */
  readonly start: number;
  /**
   Offset after the span's last character.
   */
  readonly end: number;
};

/**
 This node carries no raw bounds, distinct from bounds that read zero.

 @example
 ```ts
 if (nodeBounds(node,) === NO_NODE_BOUNDS) joinUnpositionedRun();
 ```
 */
export const NO_NODE_BOUNDS: unique symbol = Symbol('no raw bounds on a node MDAST rebuilt unpositioned',);

/**
 Raw bounds of a syntax node, where the parser kept them.

 @param node - any node the walk holds

 @returns Node's raw offsets in the masked parser input, or
 {@link NO_NODE_BOUNDS} where a transform rebuilt the node without
 positions

 @example
 ```ts
 const bounds = nodeBounds(node,);
 ```
 */
export function nodeBounds(node: TreeNode,): RawBounds | typeof NO_NODE_BOUNDS {
  /**
   Node's raw start.
   */
  const start = node.position
    ?.start
    .offset;
  /**
   Exclusive end of this node's raw span.
   */
  const end = node.position
    ?.end
    .offset;
  return ((start === undefined) || (end === undefined)) ? NO_NODE_BOUNDS : {
    start,
    end,
  };
}

/**
 Whether a link is an autolink literal, whose only text is its own URL: one
 micromark tokenized spans exactly that text, with no label bracket before
 it, and one the autolink-literal transform built carries no span at all.
 A GFM marker shape inside such text is part of the URL (ledger B123), and the
 destination reader (`corpus-run/dropped-destinations.ts`) takes the same test
 to know an address prose ran into, which it cuts where the prose resumed.
 An angle autolink is no literal here: its span starts at its `<`, and
 {@link isAngleAutolink} tells it, which only a walk under the plain grammar
 meets (the strict grammar refuses the page, so the marker reader never does).

 @param node - node a walk is about to descend into

 @returns Whether the node's children are URL text a marker walk must not
 read for GFM shapes

 @example
 ```ts
 if (!isAutolinkLiteral(node,)) descend();
 ```
 */
export function isAutolinkLiteral(node: TreeNode,): boolean {
  if (node.type !== 'link')
    return false;
  /**
   The link's own span, absent on a link the transform built.
   */
  const bounds = nodeBounds(node,);
  if (bounds === NO_NODE_BOUNDS)
    return true;
  /**
   First child of the link, absent under an empty label.
   */
  const [first,] = node.children;
  if (first === undefined)
    return false;
  /**
   Span of that child, which a label bracket sets off from the link's start.
   */
  const firstBounds = nodeBounds(first,);
  return (firstBounds !== NO_NODE_BOUNDS) && (firstBounds.start === bounds.start);
}

/**
 Whether a link is an angle autolink, `<https://cat.example/x>`, which the
 plain grammar tokenizes and the strict grammar refuses, so only a graph read
 under the plain grammar meets one. Its node span starts at the `<`, one
 character before its text, so {@link isAutolinkLiteral} does not take it for
 a literal; a link with a label starts at the `[`, and no other link starts
 at a `<`. A GFM marker shape inside its text is part of the URL.

 @param node - node a walk is about to descend into

 @param text - exact masked body supplied to the parser, which the node's span indexes

 @returns Whether the node is a link whose raw begins at an angle bracket

 @example
 ```ts
 if (!isAngleAutolink({ node, text, },)) descend();
 ```
 */
export function isAngleAutolink(
  {
    node,
    text,
  }: {
    readonly node: TreeNode;
    readonly text: string;
  },
): boolean {
  if (node.type !== 'link')
    return false;
  /**
   The link's own span, absent on a link the transform built.
   */
  const bounds = nodeBounds(node,);
  return (bounds !== NO_NODE_BOUNDS) && (text[bounds.start] === '<');
}

/**
 Zone of a bracket inside a link's own text, which is the literal the link
 was built from, a URL or an address, and never part of a reference.

 @example
 ```ts
 const leaf: RunLeaf = { value: 'www.cat.example', zone: LINK_TEXT_ZONE, };
 ```
 */
const LINK_TEXT_ZONE = -1;

/**
 Decoded text of one node of an unpositioned run, with the zone its brackets
 fall in.

 @example
 ```ts
 const leaf: RunLeaf = { value: ' naps [^9]', zone: 1, };
 ```
 */
type RunLeaf = {
  /**
   Decoded value the transform gave the node.
   */
  readonly value: string;
  /**
   Count of links standing before this text in the run, so two text nodes
   share a zone exactly when no link separates them, or
   {@link LINK_TEXT_ZONE} for a link's own text.
   */
  readonly zone: number;
};

/**
 Flattens the members of an unpositioned run into their decoded text, in
 source order. The transform replaces one text node with text nodes and
 links of one text child each, so the leaves' values joined are that text
 node's decoded value.

 @param members - siblings the run holds

 @returns One leaf per text node, a link's text marked as URL

 @throws FootnoteRewriteError when a member is neither text nor a link of
 text alone, a shape the transform does not build and this module cannot place

 @example
 ```ts
 const leaves = runLeaves({ members, },);
 ```
 */
function runLeaves({ members, }: { readonly members: readonly DeepReadonlyData<RootContent>[]; },): RunLeaf[] {
  /**
   Owned result list.
   */
  const leaves: RunLeaf[] = [];
  /**
   Links met so far, the zone of the text that follows them.
   */
  let links = 0;
  for (const member of members) {
    if (member.type === 'text') {
      leaves.push({
        value: member.value,
        zone: links,
      },);
      continue;
    }
    if (member.type !== 'link')
      throw new FootnoteRewriteError({ kind: 'position', },);
    for (const inner of member.children) {
      if (inner.type !== 'text')
        throw new FootnoteRewriteError({ kind: 'position', },);
      leaves.push({
        value: inner.value,
        zone: LINK_TEXT_ZONE,
      },);
    }
    links += 1;
  }
  return leaves;
}

/**
 Zone of every occurrence of one bracket in a run's decoded text, in order.

 @param leaves - decoded text of the run

 @param bracket - opening or closing square bracket

 @returns One zone per occurrence

 @example
 ```ts
 const openZones = bracketZones({ leaves, bracket: '[', },);
 ```
 */
function bracketZones(
  {
    leaves,
    bracket,
  }: {
    readonly leaves: readonly RunLeaf[];
    readonly bracket: '[' | ']';
  },
): number[] {
  /**
   Owned result list.
   */
  const zones: number[] = [];
  for (const {
    value,
    zone,
  } of leaves) {
    for (let at = value.indexOf(bracket,); at !== (-1); at = value.indexOf(
      bracket,
      at + 1,
    ))
      zones.push(zone,);
  }
  return zones;
}

/**
 Spellings of one square bracket as a character reference, read between the
 ampersand and the semicolon.

 @example
 ```ts
 const spellings: BracketSpellings = { names: ['lbrack',], decimal: '91', hexadecimal: '5b', };
 ```
 */
type BracketSpellings = {
  /**
   Names the entity table gives the bracket (`character-entities` 2.0.2).
   */
  readonly names: readonly string[];
  /**
   Decimal code of the bracket, without the zeros a reference may pad it with.
   */
  readonly decimal: string;
  /**
   Hexadecimal code of the bracket in lower case, without padding zeros.
   */
  readonly hexadecimal: string;
};

/**
 Character references that decode to the opening square bracket.

 @example
 ```ts
 const named = OPENING_BRACKET_SPELLINGS.names.includes('lsqb',);
 ```
 */
const OPENING_BRACKET_SPELLINGS: BracketSpellings = {
  names: [
    'lbrack',
    'lsqb',
  ],
  decimal: '91',
  hexadecimal: '5b',
};

/**
 Character references that decode to the closing square bracket.

 @example
 ```ts
 const named = CLOSING_BRACKET_SPELLINGS.names.includes('rsqb',);
 ```
 */
const CLOSING_BRACKET_SPELLINGS: BracketSpellings = {
  names: [
    'rbrack',
    'rsqb',
  ],
  decimal: '93',
  hexadecimal: '5d',
};

/**
 Most digits micromark reads in a decimal character reference
 (`characterReferenceDecimalSizeMax`); a longer one stays text.

 @example
 ```ts
 const reads = '0000091'.length <= MAX_DECIMAL_REFERENCE_DIGITS;
 ```
 */
const MAX_DECIMAL_REFERENCE_DIGITS = 7;

/**
 Most digits micromark reads in a hexadecimal character reference
 (`characterReferenceHexadecimalSizeMax`); a longer one stays text.

 @example
 ```ts
 const reads = '00005b'.length <= MAX_HEXADECIMAL_REFERENCE_DIGITS;
 ```
 */
const MAX_HEXADECIMAL_REFERENCE_DIGITS = 6;

/**
 Longest character reference that can spell a bracket, counted after its
 ampersand: a number sign, seven decimal digits and the semicolon, which a
 number sign, an `x` and six hexadecimal digits equal in length.

 @example
 ```ts
 const fits = '#0000091;'.length === LONGEST_BRACKET_REFERENCE;
 ```
 */
const LONGEST_BRACKET_REFERENCE = 9;

/**
 Whether an ampersand that no backslash escapes opens a character reference
 that decodes to one square bracket, as micromark's `characterReference`
 construct reads it: a name of the bracket, or a number sign and its decimal
 code, or a number sign, an `x` and its hexadecimal code, then a semicolon.

 @param region - raw text of the run

 @param offset - offset of an unescaped ampersand in the region

 @param bracket - opening or closing square bracket

 @returns Whether the decoded text holds that bracket for this reference

 @example
 ```ts
 const spelled = referenceSpells({ region: 'paws &#91; naps', offset: 5, bracket: '[', },);
 ```
 */
function referenceSpells(
  {
    region,
    offset,
    bracket,
  }: {
    readonly region: string;
    readonly offset: number;
    readonly bracket: '[' | ']';
  },
): boolean {
  /**
   Offset of the first character after the ampersand.
   */
  const bodyStart = offset + 1;
  /**
   Raw text after the ampersand, as far as the longest such reference and
   its semicolon reach.
   */
  const head = region.slice(
    bodyStart,
    bodyStart + LONGEST_BRACKET_REFERENCE,
  );
  /**
   Length of the reference's body, absent where no semicolon ends one.
   */
  const bodyEnd = head.indexOf(';',);
  if (bodyEnd === (-1))
    return false;
  /**
   Name or number between the ampersand and the semicolon.
   */
  const body = head.slice(
    0,
    bodyEnd,
  );
  /**
   Spellings of the bracket asked about.
   */
  const spellings = (bracket === '[') ? OPENING_BRACKET_SPELLINGS : CLOSING_BRACKET_SPELLINGS;
  if (!body.startsWith('#',))
    return spellings
      .names
      .includes(body,);
  /**
   Whether the number is written in hexadecimal.
   */
  const hexadecimal = body.startsWith('#x',) || body.startsWith('#X',);
  /**
   Most digits micromark reads in this base; a longer number stays text.
   */
  const width = hexadecimal ? MAX_HEXADECIMAL_REFERENCE_DIGITS : MAX_DECIMAL_REFERENCE_DIGITS;
  /**
   Digits as written, in lower case and padded with zeros to that width, so
   an empty or overlong number and one holding any other character differ
   from the padded code.
   */
  const padded = body.slice(hexadecimal ? '#x'.length : '#'.length,)
    .toLowerCase()
    .padStart(
    width,
    '0',
    );
  /**
   The bracket's own code in that base.
   */
  const code = hexadecimal ? spellings.hexadecimal : spellings.decimal;
  return padded === code.padStart(
    width,
    '0',
  );
}

/**
 Gives each occurrence of one bracket in a run's raw region the zone of the
 same occurrence in the run's decoded text. Decoding a text node (character
 escapes, line prefixes and suffixes) neither adds, drops nor reorders a
 square bracket, and the markup a region may hold beside that text holds
 none. A character reference that spells the bracket is counted as an
 occurrence, since the decoded text holds a bracket for it, and is given no
 offset, since no marker lexeme opens or closes at one. So the two sequences
 pair one to one.

 @param region - raw text of the run

 @param bracket - opening or closing square bracket

 @param zones - zone of each occurrence in the decoded text

 @returns Zone by raw offset within the region, written brackets only

 @throws FootnoteRewriteError when the raw and the decoded text still hold
 different counts of the bracket, so no occurrence can be placed

 @example
 ```ts
 const openZones = zoneByRawOffset({ region, bracket: '[', zones: bracketZones({ leaves, bracket: '[', },), },);
 ```
 */
function zoneByRawOffset(
  {
    region,
    bracket,
    zones,
  }: {
    readonly region: string;
    readonly bracket: '[' | ']';
    readonly zones: readonly number[];
  },
): ReadonlyMap<number, number> {
  /**
   Owned result.
   */
  const byOffset = new Map<number, number>();
  /**
   Raw occurrences paired so far, written and spelled alike.
   */
  let paired = 0;
  for (let at = 0; at < region.length; at += 1) {
    /**
     Raw character at this offset.
     */
    const character = region[at];
    if ((character === '\\') && ((region[at + 1] === '\\') || (region[at + 1] === '&'))) {
      // A backslash takes a following backslash or ampersand with it as a
      // character escape: the pair is passed over whole, so the second
      // backslash escapes nothing and the ampersand opens no reference.
      at += 1;
      continue;
    }
    /**
     Whether the bracket itself is written at this offset.
     */
    const written = character === bracket;
    if ((!written) && ((character !== '&') || (!referenceSpells({
      region,
      offset: at,
      bracket,
    },))))
      continue;
    /**
     Zone of the decoded occurrence this raw one pairs with.
     */
    const zone = zones[paired];
    if (zone === undefined)
      throw new FootnoteRewriteError({ kind: 'position', },);
    if (written)
      byOffset.set(
        at,
        zone,
      );
    paired += 1;
  }
  if (paired !== zones.length)
    throw new FootnoteRewriteError({ kind: 'position', },);
  return byOffset;
}

/**
 Marker lexemes of one unpositioned run that lie in its text: those whose
 two brackets fall in text no link separates. A lexeme that opens or closes
 in a link's own text, or spans a link, is URL to the parse and stays out.

 @param members - siblings the run holds

 @param region - raw text between the run's positioned neighbours, or its
 parent's edges

 @returns Lexemes in source order, their offsets relative to the region

 @throws FootnoteRewriteError when the members or their brackets cannot be
 placed in the raw region

 @example
 ```ts
 const spans = runSpans({ members, region: '，www.cat.example [^9]', },);
 ```
 */
function runSpans(
  {
    members,
    region,
  }: {
    readonly members: readonly DeepReadonlyData<RootContent>[];
    readonly region: string;
  },
): readonly GfmMarkerSpan[] {
  /**
   Decoded text of the run, each piece with its zone.
   */
  const leaves = runLeaves({ members, },);
  /**
   Zone of each opening bracket of the region, by raw offset.
   */
  const openZones = zoneByRawOffset({
    region,
    bracket: '[',
    zones: bracketZones({
      leaves,
      bracket: '[',
    },),
  },);
  /**
   Zone of each closing bracket of the region, by raw offset.
   */
  const closeZones = zoneByRawOffset({
    region,
    bracket: ']',
    zones: bracketZones({
      leaves,
      bracket: ']',
    },),
  },);
  return gfmMarkerSpans({ text: region, },)
    .filter(function liesInText(span,): boolean {
      /**
       Zone the lexeme opens in.
       */
      const openZone = openZones.get(span.startOffset,);
      /**
       Zone the lexeme closes in.
       */
      const closeZone = closeZones.get(span.endOffset - 1,);
      if ((openZone === undefined) || (closeZone === undefined))
        throw new Error('unreachable: a marker lexeme opens at a square bracket and ends at one, and every bracket of its region was given a zone',);
      return (openZone === closeZone) && (openZone !== LINK_TEXT_ZONE);
    },);
}

/**
 Raw bounds a run takes from its parent where no positioned sibling bounds
 it. They may reach past the run's own text into the parent's markup, which
 is harmless only where that markup can hold no square bracket: nothing for
 a paragraph, the markers of an emphasis, a strong or a strikethrough, a
 heading's number signs or underline, a table cell's pipes and padding, and
 a JSX element's tags once its attributes are left out. After the last
 attribute an opening tag holds whitespace and its closing angle bracket
 alone (`micromark-extension-mdx-jsx`, `attributeBefore`). A setext heading the parser started at the definitions
 before it starts after them (`heading-floors.ts`), whose brackets are not the heading's.

 @param parent - node whose child list the run opens or closes

 @returns Bounds the run's raw region may take

 @throws FootnoteRewriteError when the parent is of a type whose markup this
 module cannot clear of brackets, or carries no span, or is a JSX element
 with an attribute that carries none

 @example
 ```ts
 const { start, } = runEdges(parent,);
 ```
 */
function runEdges(parent: TreeNode,): RawBounds {
  /**
   The parent's own span.
   */
  const bounds = nodeBounds(parent,);
  if (bounds === NO_NODE_BOUNDS)
    throw new FootnoteRewriteError({ kind: 'position', },);
  if (parent.type === 'heading') {
    return {
      start: Math.max(
        bounds.start,
        headingFloorOf({ heading: parent, },),
      ),
      end: bounds.end,
    };
  }
  if ((parent.type === 'paragraph')
    || (parent.type === 'emphasis')
    || (parent.type === 'strong')
    || (parent.type === 'delete')
    || (parent.type === 'tableCell'))
    return bounds;
  if (parent.type !== 'mdxJsxTextElement')
    throw new FootnoteRewriteError({ kind: 'position', },);
  /**
   Last attribute of the opening tag, absent on a tag that has none.
   */
  const lastAttribute = parent.attributes
    .at(-1,);
  if (lastAttribute === undefined)
    return bounds;
  /**
   Offset after that attribute, past every attribute value of the tag.
   */
  const attributesEnd = lastAttribute.position
    ?.end
    .offset;
  if (attributesEnd === undefined)
    throw new FootnoteRewriteError({ kind: 'position', },);
  return {
    start: attributesEnd,
    end: bounds.end,
  };
}

/**
 One unpositioned run with the marker lexemes its text holds.

 @example
 ```ts
 const run: UnpositionedRun = { opener, regionStart: 0, regionEnd: 21, spans: [], };
 ```
 */
export type UnpositionedRun = {
  /**
   First member of the run, the node a source-order walk meets first.
   */
  readonly opener: DeepReadonlyData<RootContent>;
  /**
   Origin of the run's raw region in the masked parser input.
   */
  readonly regionStart: number;
  /**
   Offset after the run's raw region, where its positioned neighbour or its
   parent's edge begins. The region holds the run's text and, at most, the
   parent's own markup beside it, which holds no square bracket and no
   full-width marker bracket.
   */
  readonly regionEnd: number;
  /**
   Lexemes lying in the run's text, their offsets relative to the region.
   */
  readonly spans: readonly GfmMarkerSpan[];
};

/**
 Reads every unpositioned run among one parent's children. A run's raw is
 the source between its positioned neighbours, or its parent's edges at
 either end.

 @param parent - node whose children are read

 @param text - exact masked body supplied to the parser

 @returns Runs in source order, none for a parent whose children all kept
 their positions

 @throws FootnoteRewriteError when a run cannot be bounded or placed

 @example
 ```ts
 const runs = unpositionedRuns({ parent: paragraph, text, },);
 ```
 */
export function unpositionedRuns(
  {
    parent,
    text,
  }: {
    readonly parent: Extract<TreeNode, { readonly children: unknown; }>;
    readonly text: string;
  },
): UnpositionedRun[] {
  /**
   Owned result list.
   */
  const runs: UnpositionedRun[] = [];
  /**
   Members of the run being read, empty between runs.
   */
  let members: DeepReadonlyData<RootContent>[] = [];
  /**
   Span of the latest positioned child, absent before the first.
   */
  let before: RawBounds | typeof NO_NODE_BOUNDS = NO_NODE_BOUNDS;
  for (const child of parent.children) {
    /**
     Span of this child where the parser kept it.
     */
    const bounds = nodeBounds(child,);
    if (bounds === NO_NODE_BOUNDS) {
      members.push(child,);
      continue;
    }
    /**
     First member of the run this positioned child ends, absent where no
     run is open.
     */
    const [opener,] = members;
    if (opener !== undefined) {
      /**
       Raw start of the run.
       */
      const regionStart = (before === NO_NODE_BOUNDS)
        ? runEdges(parent,)
          .start
        : before.end;
      runs.push({
        opener,
        regionStart,
        regionEnd: bounds.start,
        spans: runSpans({
          members,
          region: text.slice(
            regionStart,
            bounds.start,
          ),
        },),
      },);
      members = [];
    }
    before = bounds;
  }
  /**
   First member of a run that reaches the end of the child list, absent
   where the last child kept its position.
   */
  const [opener,] = members;
  if (opener !== undefined) {
    /**
     Raw start of that run.
     */
    const regionStart = (before === NO_NODE_BOUNDS)
      ? runEdges(parent,)
        .start
      : before.end;
    /**
     Raw end of that run, the parent's own edge.
     */
    const { end: regionEnd, } = runEdges(parent,);
    runs.push({
      opener,
      regionStart,
      regionEnd,
      spans: runSpans({
        members,
        region: text.slice(
          regionStart,
          regionEnd,
        ),
      },),
    },);
  }
  return runs;
}

//endregion Unpositioned runs
