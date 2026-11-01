import { assertLandingInside, } from './fragment-landing-error.ts';
import {
  parseSliceBody,
  requireFrontMatterRefusal,
  requireMdxRefusal,
  splitFrontMatter,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import { withDefinitions, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  childrenAreProse,
  type TreeNode,
} from './typography-link-text.ts';
import { shownRangesOf, } from './typography-shown-view.ts';

//region Prose mask
// Which characters of a text are prose a typography rule may rewrite: the text nodes of the page parse, and nothing else.
//
// A straight quote inside a link destination or title, a bare address or an autolink, inline code of any backtick count, a
// code block of any kind, math, an MDX expression, a tag or any attribute of it, an HTML comment or front matter is not
// prose: a rule that curls it changes what the page says (a link to another address, a title lost, code that no longer
// runs, a string literal that compiles nowhere). On 2026-09-06 the yulianNyanner page shipped exactly that for a tag, after
// the typography restoration reached a blockquoted component line for the first time; the n155 review of 2026-10-15 found
// the same for the rest of the list, which the earlier scanner of backticks and tags never read.
//
// The mask is built from the parse the page assembly runs on every slice (`parse-slice-body.ts`), so what a rule may write
// and what the page shows as text are one thing. A unit is prose when a text node of that parse covers it and the parser
// read the unit as written: a comment, a lone container tag line and a line of invisible characters are turned into spaces
// before the parse, and the spaces they became are no prose. The text of a link written between brackets is prose, unless the
// label is itself an address (one token with no white space in which the autolink-literal reading finds an address): then a reader copies
// the label as the address, and a rule that curls a mark in it shows an address that is not the one that opens. The text of a
// shortcut or collapsed reference (`[dogs' bowl]`) is no prose either, since it is also the label its definition is looked up by
// (`typography-link-text.ts`). The text of a bare address, a `www.` address or an email address is the address itself, which the parser also puts in a text node, and
// it is no prose here. The text of an element between tags is prose; the tag and its attributes are not. A text node that
// runs over several lines of a quotation covers the markers at the start of each line, units no rule rewrites.
//
// A TEXT THE PAGE PARSE REFUSES HAS NO PROSE AT ALL. No parse shows what is text there, so no rule writes anything. The
// refusal is told apart from a text that merely holds no prose (`proseMaskReading`), so a reader that gets an all-false mask
// from a refusal can say so (`typography-mask-refusal.ts`): the page assembly guard withdraws a slice that breaks the parse
// (`corpus-run/page-assembly-guard.ts`), but nothing withdraws an archive slice that no lane touched and the guard reads
// whatever the lanes chose, so the silence of a typography rule on a refused slice is a finding and not a pass.
//
// A SLICE IS PARSED WITH THE PAGE'S DEFINITIONS WHEN THE CALLER HAS THEM. A reference or a footnote reference whose definition
// stands in another slice is plain text to a parse of its own slice and a label to a parse of the page (`page-definitions.ts`),
// so the definitions are appended after the slice for the parse and only the slice's own offsets are read back.
//
// The mask is computed once per text and read by every rule, so the rules agree on what prose is instead of each carrying
// its own scanner. The page pass reads each slice's mask once and hands it to its tally and its rewrite
// (`corpus-run/quote-style-unify.ts`); a text read before is answered from a short memory, which serves the repeated reads
// inside one text (`restoreTypography` and the rules it calls), since the parse costs more than the rules that read it.

/**
 A node of the tree with whether the text nodes under it are prose.
 */
type VisitedNode = {
  /**
   The node.
   */
  readonly node: TreeNode;

  /**
   Whether a text node under it is prose.
   */
  readonly prose: boolean;
};

/**
 The offsets one text node of the tree covers in the body.
 */
type ProseRange = {
  /**
   Offset where the text node starts.
   */
  readonly start: number;

  /**
   Exclusive offset where it ends.
   */
  readonly end: number;
};

/**
 What the parse of a text body says: the text the parser read and the offsets the text nodes of the tree cover.
 */
type ProseRead = {
  /**
   The body with every comment, lone container tag and invisible-only line turned into spaces of the same length.
   */
  readonly masked: string;

  /**
   The offsets of the text nodes that are prose, in the body.
   */
  readonly ranges: readonly ProseRange[];

  /**
   The offsets of the content of code and math, which a reader sees though no rule writes in it, in the body.
   */
  readonly shown: readonly ProseRange[];
};

/**
 What a text says of its prose: one flag per unit, and whether the grammar refused the text, which gives the same all-false
 mask as a text that holds no prose.
 */
export type ProseMaskReading = {
  /**
   One flag per UTF-16 unit of the text, true where a typography rule may write.
   */
  readonly mask: readonly boolean[];

  /**
   One flag per UTF-16 unit of the text, true where a reader sees the unit: the prose and the content of code and math. The
   delimiters of emphasis, strike and code, the brackets and destinations of links, tags, comments and expressions show nothing.
   */
  readonly visible: readonly boolean[];

  /**
   Whether the grammar refused the text (or its front matter), so no parse says what is prose in it.
   */
  readonly refused: boolean;
};

/**
 How many texts the memory holds: the rules read one text again and again (`restoreTypography` and the rules it calls), so a
 handful of the most recent texts answers those questions. The page pass does not lean on it: it reads each slice's mask once
 and hands it on.
 */
const REMEMBERED_MASKS = 32;

/**
 The readings of the most recent texts, by the page's definitions and then the text, as two keys of two maps, so that no text can
 be written to read as another pair. A reading is a function of the text and the definitions alone, so remembering it changes no
 answer.
 */
const rememberedMasks = new Map<string, Map<string, ProseMaskReading>>();

/**
 One text read with the definitions of one page, as the memory holds it.
 */
type RememberedPair = {
  /**
   The page's definitions.
   */
  readonly definitions: string;

  /**
   The text.
   */
  readonly text: string;
};

/**
 The pairs remembered, the oldest first, which is the order the memory forgets them in.
 */
const rememberedOrder: RememberedPair[] = [];

/**
 How many readings were built, not answered from the memory. A test reads it to see that the page pass reads each slice once.
 */
const builds = { count: 0, };

/**
 The text nodes of a tree that are prose, walked with a work stack rather than by recursion, since a tree is as deep as the
 nesting a page writes.

 @param root - the tree's top node

 @param masked - the text the parser read

 @returns The offsets the text nodes cover, as start and exclusive end

 @example
 ```ts
 const ranges = proseRangesOf({ root, masked: '*it’s*', },);
 ```
 */
function proseRangesOf(
  {
    root,
    masked,
  }: {
    readonly root: TreeNode;
    readonly masked: string;
  },
): readonly ProseRange[] {
  /**
   Ranges met so far.
   */
  const ranges: ProseRange[] = [];
  /**
   Nodes still to visit.
   */
  const pending: VisitedNode[] = [
    {
      node: root,
      prose: true,
    },
  ];
  // Each node in turn, until the stack is empty.
  for (let item = pending.pop(); item !== undefined; item = pending.pop()) {
    /**
     The node and whether a text node under it is prose.
     */
    const {
      node,
      prose,
    } = item;
    if ((node.type === 'text') && prose) {
      /**
       Where the parser started the text.
       */
      const start = node.position
        ?.start
        .offset;
      /**
       Where the parser ended the text.
       */
      const end = node.position
        ?.end
        .offset;
      if ((start !== undefined) && (end !== undefined)) {
        ranges.push({
          start,
          end,
        },);
      }
    }
    if ('children' in node) {
      /**
       Whether the text under the node is prose: the text of a link is, unless the address is the text.
       */
      const childrenProse = childrenAreProse({
        node,
        masked,
        prose,
      },);
      for (const child of node.children as readonly TreeNode[]) {
        pending.push({
          node: child,
          prose: childrenProse,
        },);
      }
    }
  }
  return ranges;
}

/**
 The stretches that start inside the body, cut off where the body ends: the parse also covers the definitions appended after the
 body, which are no part of it.

 @param ranges - stretches of the parse, of the body and the definitions

 @param length - how long the body is

 @returns The stretches of the body

 @example
 ```ts
 const kept = withinBody({ ranges: [{ start: 2, end: 9, },], length: 5, },); // [{ start: 2, end: 5, }]
 ```
 */
function withinBody(
  {
    ranges,
    length,
  }: {
    readonly ranges: readonly ProseRange[];
    readonly length: number;
  },
): readonly ProseRange[] {
  return ranges
    .filter(function startsInBody({ start, },): boolean {
      return start < length;
    },)
    .map(function clipped({
      start,
      end,
    },): ProseRange {
      return {
        start,
        end: Math.min(
          end,
          length,
        ),
      };
    },);
}

/**
 What the parse of a text body says of its prose, none when the grammar refuses the body.

 @param body - text after any front matter

 @param definitions - the page's definitions, parsed after the body so a reference in it resolves; empty for a body read alone

 @returns The text the parser read and the prose ranges as a list of one, none when the grammar refused the body; only the body's
 own offsets are in the answer

 @throws The error of any failure that is no refusal of the grammar

 @example
 ```ts
 const [read,] = proseReadOf({ body: 'The cat’s nap.', definitions: '', },);
 ```
 */
function proseReadOf(
  {
    body,
    definitions,
  }: {
    readonly body: string;
    readonly definitions: string;
  },
): readonly ProseRead[] {
  try {
    /**
     The tree of the body with the definitions after it, and the text the parser read.
     */
    const {
      root,
      masked,
    } = parseSliceBody({
      text: withDefinitions({
        text: body,
        definitions,
      },),
    },);
    return [
      {
        masked: masked.slice(
          0,
          body.length,
        ),
        ranges: withinBody({
          ranges: proseRangesOf({
            root: root as TreeNode,
            masked,
          },),
          length: body.length,
        },),
        shown: withinBody({
          ranges: shownRangesOf({
            root: root as TreeNode,
            masked,
          },),
          length: body.length,
        },),
      },
    ];
  }
  catch (error) {
    // Only the grammar's own refusal says the body has no prose; anything else keeps propagating.
    requireMdxRefusal({ error, },);
    return [];
  }
}

/**
 Where the body of a text starts, past any front matter, none when the fences are there and the YAML between them refuses to
 parse (so the text has no prose that can be told).

 @param text - the whole text

 @returns Offset of the body in the text as a list of one, none when the front matter is refused

 @throws The error of any failure that is no refusal of the front matter

 @example
 ```ts
 const [offset,] = bodyOffsetOf({ text: '---\ntitle: it’s\n---\n\nBody', },); // 20
 ```
 */
function bodyOffsetOf({ text, }: { readonly text: string; },): readonly number[] {
  try {
    /**
     Where the body starts, past the front matter.
     */
    const { bodyOffset, } = splitFrontMatter({ text, },);
    return [bodyOffset,];
  }
  catch (error) {
    requireFrontMatterRefusal({ error, },);
    return [];
  }
}

/**
 Builds the reading of a text, without the memory.

 @param text - text to mask

 @param definitions - the page's definitions, empty for a text read alone

 @returns One flag per UTF-16 unit, and whether the grammar refused the text

 @example
 ```ts
 const { mask, refused, } = buildMask({ text: 'a <b c="d">cat</b> e', definitions: '', },);
 ```
 */
function buildMask(
  {
    text,
    definitions,
  }: {
    readonly text: string;
    readonly definitions: string;
  },
): ProseMaskReading {
  builds.count += 1;
  /**
   Flags for the text, false until a text node of the parse covers the unit.
   */
  const mask: boolean[] = Array.from(
    { length: text.length, },
    function unmasked(): boolean {
      return false;
    },
  );
  /**
   Flags for what a reader sees of the text, false until prose or the content of code or math covers the unit.
   */
  const visible: boolean[] = Array.from(
    { length: text.length, },
    function unseen(): boolean {
      return false;
    },
  );
  /**
   The reading of a text the grammar refused: no prose, and no parse says what a reader sees, so every unit stands as written.
   */
  const refusal: ProseMaskReading = {
    mask,
    visible: Array.from(
      { length: text.length, },
      function seen(): boolean {
        return true;
      },
    ),
    refused: true,
  };
  /**
   Where the body starts: front matter is no prose.
   */
  const [bodyOffset,] = bodyOffsetOf({ text, },);
  if (bodyOffset === undefined)
    return refusal;
  /**
   The body, which the offsets of the parse index.
   */
  const body = text.slice(bodyOffset,);
  /**
   What the parse says of the body.
   */
  const [read,] = proseReadOf({
    body,
    definitions,
  },);
  if (read === undefined)
    return refusal;
  /**
   The text the parser read, which differs from the body where a comment, a lone tag line or an invisible-only line was turned
   into spaces.
   */
  const { masked, } = read;
  for (const {
    start,
    end,
  } of read.ranges) {
    for (let index = start; index < end; index += 1) {
      /**
       The unit as written.
       */
      const written = body.charAt(index,);
      // A unit the parser read as something else than it was written (a comment, a lone tag line) is no prose.
      if (written === masked.charAt(index,)) {
        mask[bodyOffset + index] = true;
        visible[bodyOffset + index] = true;
      }
    }
  }
  for (const {
    start,
    end,
  } of read.shown) {
    for (let index = start; index < end; index += 1) {
      // The content of code and math is shown as written, unless the parser turned the unit into something else.
      if (body.charAt(index,) === masked.charAt(index,))
        visible[bodyOffset + index] = true;
    }
  }
  return {
    mask,
    visible,
    refused: false,
  };
}

/**
 Reads each unit of a text as prose or not, and says whether the grammar refused the text.

 Prose is the text the page parse shows as text: a link's label (unless the label is itself an address), a heading, a paragraph,
 a table cell, the text of an element between tags. A link destination or title, an address, code of any kind, math, an
 expression, a tag and its attributes, a comment and front matter are not, and a text the grammar refuses has none. The mask has
 one entry per UTF-16 unit of the text, so it indexes exactly as `charAt` does. A reference or a footnote reference whose
 definition stands in another slice of the page resolves when the page's definitions are given.

 @param text - text to mask

 @param definitions - the page's definitions (`pageDefinitionsOf`), none for a text read alone

 @returns One flag per unit, true where a typography rule may write, and whether the grammar refused the text

 @example
 ```ts
 const { mask, refused, } = proseMaskReading({ text: 'a <b c="d">cat</b> e', },);
 ```
 */
export function proseMaskReading(
  {
    text,
    definitions = '',
  }: {
    readonly text: string;
    readonly definitions?: string;
  },
): ProseMaskReading {
  /**
   The readings remembered for these definitions, by text.
   */
  const ofDefinitions = rememberedMasks.get(definitions,) ?? new Map<string, ProseMaskReading>();
  /**
   The reading made before for an equal text and equal definitions.
   */
  const known = ofDefinitions.get(text,);
  if (known !== undefined)
    return known;
  /**
   The reading of this text.
   */
  const reading = buildMask({
    text,
    definitions,
  },);
  ofDefinitions.set(
    text,
    reading,
  );
  rememberedMasks.set(
    definitions,
    ofDefinitions,
  );
  rememberedOrder.push({
    definitions,
    text,
  },);
  if (rememberedOrder.length > REMEMBERED_MASKS) {
    /**
     The oldest pair remembered, which goes.
     */
    const [oldest,] = rememberedOrder.splice(
      0,
      1,
    );
    if (oldest !== undefined) {
      /**
       The readings remembered for the oldest pair's definitions.
       */
      const held = rememberedMasks.get(oldest.definitions,);
      held?.delete(oldest.text,);
      if (held?.size === 0)
        rememberedMasks.delete(oldest.definitions,);
    }
  }
  return reading;
}

/**
 The mask of a fragment of a slice as the slice reads it with the fragment in place: the slice with the fragment written over the
 region it replaces is parsed, and the flags of the fragment's own units are returned. A fragment that begins inside a tag or ends
 inside an element parses alone as something else than the slice shows.

 @param slice - the slice the fragment lands in

 @param start - offset where the replaced region starts in the slice

 @param end - exclusive offset where the replaced region ends in the slice

 @param replacement - the fragment that takes the region's place

 @param definitions - the page's definitions, none for a slice read alone

 @returns One flag per unit of the fragment, and whether the grammar refused the slice with the fragment in place

 @throws FragmentLandingError (`fragment-landing-error.ts`) when the region starts before the slice, ends before it starts, or ends past the slice

 @example
 ```ts
 const { mask, } = fragmentProseMask({ slice: 'He said <b>It\'s here</b>.', start: 8, end: 20, replacement: '<b>It\'s here', },);
 ```
 */
export function fragmentProseMask(
  {
    slice,
    start,
    end,
    replacement,
    definitions = '',
  }: {
    readonly slice: string;
    readonly start: number;
    readonly end: number;
    readonly replacement: string;
    readonly definitions?: string;
  },
): ProseMaskReading {
  assertLandingInside({
    slice: slice.length,
    start,
    end,
  },);
  /**
   The reading of the slice with the fragment in place.
   */
  const {
    mask,
    visible,
    refused,
  } = proseMaskReading({
    text: [
      slice.slice(
        0,
        start,
      ),
      replacement,
      slice.slice(end,),
    ].join('',),
    definitions,
  },);
  return {
    mask: mask.slice(
      start,
      start + replacement.length,
    ),
    visible: visible.slice(
      start,
      start + replacement.length,
    ),
    refused,
  };
}

/**
 How many masks were built, not answered from the memory since the process began. A test reads the difference over one piece of
 work to see how many texts it parsed.

 @returns The count of builds

 @example
 ```ts
 const before = proseMaskBuilds();
 ```
 */
export function proseMaskBuilds(): number {
  return builds.count;
}

//endregion Prose mask
