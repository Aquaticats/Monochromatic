import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import type { BlockShape, } from './translate-skeleton.ts';

//region Translate validation block comparison
// The block half of `validateTranslatedSlice`, split from `translate-validate.ts`
// at its line budget along the seam it already had: these helpers compare a
// candidate's block skeleton with the floor it must carry and write the
// findings, and the validator calls only `compareBlocks`. The archive block
// review's shape check (`archive-revision-shape.ts`) reads the same two
// helpers, `describeBlocks` and `sameShape`, rather than its own copies of
// them (audit area six, 2026-09-28).
/**
 Renders one block for a finding.

 @param shape - block to describe

 @returns Kind with its distinguishing detail

 @example
 ```ts
 const label = describeBlock({ kind: 'heading', detail: 'level 2', },);
 ```
 */
function describeBlock(shape: BlockShape,): string {
  return (shape.detail === '') ? shape.kind : `${shape.kind} (${shape.detail})`;
}

/**
 Renders a block sequence for a finding.

 @param blocks - blocks in document order

 @returns Comma-separated description, or a word for none

 @example
 ```ts
 const label = describeBlocks({ blocks, },);
 ```
 */
export function describeBlocks({ blocks, }: { readonly blocks: readonly BlockShape[]; },): string {
  if (blocks.length === 0)
    return 'nothing';
  return blocks.map(describeBlock,)
    .join(', ',);
}

/**
 Whether one block sequence appears inside another, in order.

 MATCHED BY KIND AND DETAIL, so a heading of another level does not stand in
 for the one the page carries.

 @param floor - sequence that has to appear

 @param candidate - sequence to look for it in

 @returns Whether every block of `floor` was found, in order

 @example
 ```ts
 const held = appearsInOrder({ floor, candidate, },);
 ```
 */
function appearsInOrder(
  {
    floor,
    candidate,
  }: {
    readonly floor: readonly BlockShape[];
    readonly candidate: readonly BlockShape[];
  },
): boolean {
  /**
   How far into `floor` the candidate got.
   */
  const matched = candidate.reduce(
    function advance(
      cursor: number,
      shape: BlockShape,
    ): number {
      /**
       Block the floor wants next, absent once the whole floor is matched.
       */
      const wanted = floor[cursor];

      /**
       Whether this block is the one the floor wants next.
       */
      const wantedMatches = (wanted !== undefined)
        && (wanted.kind === shape.kind)
        && (wanted.detail === shape.detail);
      return wantedMatches ? cursor + 1 : cursor;
    },
    0,
  );
  return matched >= floor.length;
}

/**
 Whether two block sequences are the same shape, kind for kind and detail
 for detail.

 @param left - one sequence

 @param right - other sequence

 @returns Whether every block matches its counterpart

 @example
 ```ts
 sameShape({ left: source, right: candidate, },);
 ```
 */
export function sameShape(
  {
    left,
    right,
  }: {
    readonly left: readonly BlockShape[];
    readonly right: readonly BlockShape[];
  },
): boolean {
  return (left.length === right.length)
    && left.every(function matches(
      block,
      index,
    ): boolean {
      /**
       Counterpart block in the other sequence.
       */
      const other = right[index];
      return (other !== undefined)
        && (other.kind === block.kind)
        && (other.detail === block.detail);
    },);
}

/**
 Whether the floor's blocks are all of kinds the original has, so any surplus
 is a split of the original's blocks rather than something added.

 @param floor - blocks the candidate is asked to carry

 @param source - original's blocks

 @returns Whether every floor block has a kind and detail the original has

 @example
 ```ts
 splitOnly({ floor: page.blocks, source: expected.blocks, },);
 ```
 */
function splitOnly(
  {
    floor,
    source,
  }: {
    readonly floor: readonly BlockShape[];
    readonly source: readonly BlockShape[];
  },
): boolean {
  return floor.every(function hasKind(block,): boolean {
    return source.some(function sameKind(candidate,): boolean {
      return (candidate.kind === block.kind)
        && (candidate.detail === block.detail);
    },);
  },);
}

/**
 Findings for a block skeleton that does not carry the floor's.

 THE PAGE IS A FLOOR, NOT A CEILING, and two references are why. Measured over
 68 settled slice records, the archive and the Chinese carry the same block
 sequence at 48, the archive carries more at 11 and fewer at 7, and those
 seven are two different things: an archive that MERGED Chinese paragraphs
 into a better shape, and an archive simply MISSING blocks the Chinese
 carries. Anchoring to either alone breaks the other case, so a candidate has
 to carry the floor's blocks and may add one only where the Chinese has more.

 WITH THE ORIGINAL AS THE FLOOR THIS IS TODAY'S EXACT MATCH. A floor of the
 original with a ceiling of the original's own length admits one sequence, the
 original's.
 EITHER RENDERING, the owner's decision of 2026-09-07
 (`doc/decision/translation-repair-block-floor.md`): a candidate shaped
 exactly as the original is a faithful rendering of it whatever shape the
 archive chose, so it passes beside one shaped as the page. The Huasheng poem
 is two `<br/>` paragraphs in the source and five paragraphs in the archive,
 every producer followed the source, and the entry stopped with nothing
 valid; 34 of the 92 archives carry more top-level blocks than their source.
 What stays refused is a shape that is neither reference's, which is what a
 dropped passage looks like. THE ORIGINAL'S SHAPE COUNTS ONLY WHERE THE PAGE'S
 SURPLUS IS MORE BLOCKS OF THE ORIGINAL'S OWN KINDS, a split: a page whose
 extra blocks are of a kind the original lacks (the sixth consolidation bed's
 html and blockquote against one paragraph, an archive's blockquote that says
 a passage was left by someone) is carrying something a split cannot explain,
 and a candidate shaped as the original would drop it.

 THE PAGE'S SUBSTITUTE AND THE ORIGINAL'S OWN KIND (class thirty-two,
 2026-09-16): Mio's archive ends with a farewell paragraph where the original
 ends with a poem in a block quote, the pairing set the two against each
 other, and a ceiling of max(page, original) left room for one block. The
 consolidation sheet asks a producer to add a block to carry what the
 original has and the archive left out, and the quote is exactly that, so
 the ceiling also admits the page's blocks plus every original block of a
 kind the page has no block of. What the page rendered in a kind of its own
 is kept; what it has no block of the kind for may follow.

 @param floor - blocks the candidate has to carry, the page's where there is
 one and the original's where there is not

 @param floorName - what a finding calls that sequence

 @param source - original's blocks, which set the ceiling with the floor

 @param candidate - candidate's blocks

 @returns One finding per rule the candidate's shape breaks

 @example
 ```ts
 const findings = compareBlocks({ floor, floorName, source, candidate, },);
 ```
 */
export function compareBlocks(
  {
    floor,
    floorName,
    source,
    candidate,
  }: {
    readonly floor: readonly BlockShape[];
    readonly floorName: string;
    readonly source: readonly BlockShape[];
    readonly candidate: readonly BlockShape[];
  },
): readonly string[] {
  if (
    splitOnly({
      floor,
      source,
    },)
      && sameShape({
        left: source,
        right: candidate,
      },)
  )
    return [];

  /**
   Original blocks of a kind the floor has no block of, which the page
   rendered as nothing of their kind and a candidate may carry beside the
   page's blocks.
   */
  const substituted = source.filter(function unrenderedKind(block,): boolean {
    return !floor.some(function sameKind(carried,): boolean {
      return (carried.kind === block.kind)
        && (carried.detail === block.detail);
    },);
  },);

  /**
   Most blocks any reference asks for: the page's, the original's, or the
   page's with the original blocks it has no kind for.
   */
  const ceiling = Math.max(
    floor.length,
    source.length,
    floor.length + substituted.length,
  );

  /**
   Sentence telling the author what may follow the floor's blocks, empty
   where the page has a block of every original kind.
   */
  const allowance = (substituted.length === 0)
    ? ''
    : ` The ORIGINAL's ${describeBlocks({ blocks: substituted, },)} has no block of its kind on the ${
      floorName
    }, so carry the ${floorName}'s blocks and add it after them in the ORIGINAL's own kind.`;

  /**
   Finding for a candidate the floor is not inside.
   */
  const missing = appearsInOrder({
    floor,
    candidate,
  },)
    ? []
    : [
      `The ${floorName} is ${String(floor.length,)} ${
        wordForCount({
          count: floor.length,
          one: 'block',
          many: 'blocks',
        },)
      } (${describeBlocks({ blocks: floor, },)}) and your translation is ${
        String(candidate.length,)
      } (${describeBlocks({ blocks: candidate, },)}). Every block of the ${floorName} has `
        + `to appear in your translation, of the same kind and in the same order.${allowance}`,
    ];

  /**
   Finding for a candidate carrying more blocks than anything asks for.
   */
  const surplus = (candidate.length <= ceiling)
    ? []
    : [
      `Your translation is ${String(candidate.length,)} ${
        wordForCount({
          count: candidate.length,
          one: 'block',
          many: 'blocks',
        },)
      } (${
        describeBlocks({ blocks: candidate, },)
      }) and the ${floorName} is ${String(floor.length,)}. Add a block only to `
        + 'carry something the ORIGINAL has and the text you are replacing left out.',
    ];

  return [
    ...missing,
    ...surplus,
  ];
}

/**
 Findings for a candidate that carries fewer quoted passages, at every
 depth, than its floor.

 READ OFF THE PARSE RATHER THAN THE TOP-LEVEL BLOCK LIST (ledger B110).
 `compareBlocks` compares only the top-level block sequence, so a candidate
 may keep every top-level block, same kind and same order, while a
 blockquote nested inside a container tag, a list, a footnote, or another
 blockquote is turned into prose underneath it. The strict parse already
 counts blockquotes at every depth (`SliceSkeleton.quotedPassages`), so the
 check is one comparison, called beside `compareBlocks` rather than inside
 it so that it answers whatever the block comparison found.

 @param floorName - what a finding calls the reference the candidate has to
 carry

 @param floorQuotedPassages - reference's blockquotes at every depth

 @param candidateQuotedPassages - candidate's blockquotes at every depth

 @returns One finding where the candidate carries fewer, empty otherwise

 @example
 ```ts
 const findings = quotedPassageFloorFindings({ floorName: 'PAGE AS IT STANDS', floorQuotedPassages: 1, candidateQuotedPassages: 0, },);
 ```
 */
export function quotedPassageFloorFindings(
  {
    floorName,
    floorQuotedPassages,
    candidateQuotedPassages,
  }: {
    readonly floorName: string;
    readonly floorQuotedPassages: number;
    readonly candidateQuotedPassages: number;
  },
): readonly string[] {
  if (candidateQuotedPassages >= floorQuotedPassages)
    return [];
  return [
    `The ${floorName} carries ${String(floorQuotedPassages,)} ${
      wordForCount({
        count: floorQuotedPassages,
        one: 'quoted passage',
        many: 'quoted passages',
      },)
    } (blockquotes at any depth, including inside a container tag, a list, a footnote, or another blockquote) `
      + `and your translation carries ${String(candidateQuotedPassages,)}. Every quoted passage of the ${floorName} `
      + 'has to stay a quoted passage in your translation, where it stands.',
  ];
}

//endregion Translate validation block comparison
