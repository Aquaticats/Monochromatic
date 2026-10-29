import type { ChatMessage, } from '@monochromatic-dev/llm-llm-type/ts';

import { withoutTranslatorsNotes, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import { isIndexPairingWire, } from './index-pair-list.ts';
import { selectFence, } from './prompt-fence.ts';
import type { ReadonlyEntry, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Block pairing wire
// PAIRING IS THE ONE JOB HERE THAT SCORING CANNOT DO.
//
// The deterministic aligner has three signals and this corpus exhausts all
// three: block kind is constant when every block is a paragraph, Chinese and
// English prose share no Latin tokens, and length is a weak tiebreaker even
// once its expansion is estimated per document rather than assumed. Measured on
// `saurikissa`, that reaches four correct pairings in eight and goes no further
// at any length weight.
//
// The errors that remain are one-to-two correspondences, where a translation
// splits a paragraph the original keeps whole. Reading two languages and saying
// which passage renders which is comprehension, and
// `doc/decision/llm-assisted-block-pairing.md` decides it is done by a model.
//
// A REFUSED PAIRING MUST NOT SILENTLY PROCEED, which was already demanded of
// the section aligner: a wrong pairing manufactures issues rather than skipping
// work, so it is worse than no pairing. Everything this file parses is checked
// against the block counts it was built from, and anything that does not hold
// throws rather than being repaired into something plausible.

/**
 Signals a pairing a model returned that cannot be used as one.

 @example
 ```ts
 throw new BlockPairingError({ message: 'pair 3 moves backwards on the original side', },);
 ```
 */
export class BlockPairingError extends Error {
  /**
   Names the class for callers matching on it.
   */
  public override readonly name = 'BlockPairingError';

  /**
   @param message - what about the returned pairing cannot be used
   */
  public constructor({ message, }: { readonly message: string; },) {
    super(message,);
  }
}

/**
 The refusal a catch around {@link readBlockPairing} holds, for a catch that
 treats an unusable reply as a lost voice.

 ONE NARROWING PER CLASS, as `requireFrontMatterRefusal` (`front-matter.ts`)
 is for its splitter: the reader raises every refusal as a
 {@link BlockPairingError}, so a rethrow written in the catch is a statement
 only a broken reader reaches. It stands here once, where a case reaches it.

 @param error - what the catch caught

 @returns Refusal

 @throws The caught value unchanged when it is anything but a pairing
 refusal, an unexpected state that must keep propagating

 @example
 ```ts
 const refusal = requireBlockPairingRefusal({ error, },);
 ```
 */
export function requireBlockPairingRefusal({ error, }: { readonly error: unknown; },): BlockPairingError {
  if (error instanceof BlockPairingError)
    return error;
  throw error;
}

/**
 One block on one side, as the sheet numbers it.

 @example
 ```ts
 const block: NumberedBlock = { index: 0, text: 'The tabby dozed by the stove.', };
 ```
 */
export type NumberedBlock = {
  /**
   Position in document order, zero-based, as the sheet shows it.
   */
  readonly index: number;

  /**
   The block's own text.
   */
  readonly text: string;
};

/**
 One committed correspondence between the two sides.

 @example
 ```ts
 const pair: BlockPair = { source: 2, target: 3, };
 ```
 */
export type BlockPair = {
  /**
   Original-side block index.
   */
  readonly source: number;

  /**
   Translation-side block index.
   */
  readonly target: number;
};

/**
 What a model returns for one document pair.

 Unpaired blocks are ABSENT rather than listed against a sentinel, because a
 sentinel invites a model to pair everything and mark the doubtful ones, which
 is the behaviour this exists to prevent.

 @example
 ```ts
 const wire: BlockPairingWire = { pairs: [{ source: 0, target: 0, },], };
 ```
 */
export type BlockPairingWire = {
  /**
   Correspondences the model committed to, in document order.
   */
  readonly pairs: readonly BlockPair[];
};

/**
 Renders one side's blocks as a numbered, fenced list.

 @param blocks - blocks in document order

 @param fence - fence no block text can reproduce

 @returns Sheet section listing every block against its index

 @example
 ```ts
 const section = renderBlocks({ blocks, fence: '=====', },);
 ```
 */
function renderBlocks(
  {
    blocks,
    fence,
  }: {
    readonly blocks: readonly NumberedBlock[];
    readonly fence: string;
  },
): string {
  return blocks
    .map(function toEntry(block,): string {
      return `[${String(block.index,)}]\n${fence}\n${block.text}\n${fence}`;
    },)
    .join('\n\n',);
}

/**
 Builds the sheet asking one model to pair two documents' blocks.

 WHAT THE PICTURES SAY TRAVELS WITH THE QUESTION (class thirty-four,
 2026-09-16). An archive block that translates a picture's words (a chat, a
 post, a note) has no counterpart among the original's blocks, and a sheet
 that shows only the two block lists cannot tell that block from a rendering
 of whatever original block stands where the picture stands: Mio17 paired the
 archive's chat translation with the poem's quote, the floor then had room for
 one blockquote, and the entry stopped. Only 2 of the 49 picture pages label
 such a block, so the sheet is shown the pictures' transcripts and told what
 they mean for pairing.

 @param sourceBlocks - original blocks in document order

 @param targetBlocks - translation blocks in document order

 @param pictureContext - transcripts of the pictures this section shows,
 rendered the way every other sheet carries them, absent or empty when the
 section shows none or nobody read them

 @returns Messages for one pairing call

 @example
 ```ts
 const messages = buildBlockPairingMessages({ sourceBlocks, targetBlocks, },);
 ```
 */
export function buildBlockPairingMessages(
  {
    sourceBlocks,
    targetBlocks,
    pictureContext = '',
  }: {
    readonly sourceBlocks: readonly NumberedBlock[];
    readonly targetBlocks: readonly NumberedBlock[];
    readonly pictureContext?: string;
  },
): readonly ChatMessage[] {
  // THE MODEL IS SHOWN THE TRANSLATION'S BLOCKS WITHOUT THE NOTES THE ARCHIVE'S
  // TRANSLATORS LEFT EACH OTHER (`withoutTranslatorsNotes`): no reader sees one,
  // so a note may not make a block look like a rendering of an original block
  // or make a block of its own. Every index and the length of the list stay as
  // the caller numbered them, and the pairs the model returns name those
  // indices, so a block that held nothing but a note is listed empty. The
  // original's own comments stay in view.
  /**
   Texts whose comments are the original's own.
   */
  const originals = sourceBlocks
    .map(function toText(block,): string {
      return block.text;
    },)
    .join('\n',);
  /**
   Translation blocks as the model reads them, each at its index.
   */
  const shownTargetBlocks = targetBlocks.map(function shown(block,): NumberedBlock {
    return {
      index: block.index,
      text: withoutTranslatorsNotes({
        text: block.text,
        original: originals,
      },),
    };
  },);

  /**
   Fence chosen against every block this sheet carries.

   Both sides are arbitrary prose and either may contain a run of equals
   signs (a setext heading underline is one), so a fixed fence would let a
   block close its own listing and have the rest read as sheet structure.
   `selectFence` fenced with equals signs from `b111fc376`, before this sheet
   was written, yet this comment said backticks until ledger D16.
   */
  const fence = selectFence({
    texts: [
      ...sourceBlocks.map(function toText(block,): string {
        return block.text;
      },),
      ...targetBlocks.map(function toText(block,): string {
        return block.text;
      },),
      ...shownTargetBlocks.map(function toText(block,): string {
        return block.text;
      },),
      pictureContext,
    ],
  },);

  /**
   Rule the sheet adds when the section shows pictures somebody read.
   */
  const pictureRule = (pictureContext === '')
    ? ''
    : 'WHAT THE PICTURES SAY. The original shows pictures, and their words are '
      + 'transcribed under WHAT THE PICTURES HERE SAY. A translation block that renders '
      + 'what a picture says (a chat, a post, a note, a caption) translates THE PICTURE, '
      + 'not any original block: LEAVE IT OUT, even where an original block stands in the '
      + 'same place, because that original block says something else.\n\n';

  return [
    {
      role: 'system',
      content: `You pair the paragraphs of an ORIGINAL document with the paragraphs of a `
        + `TRANSLATION of it. Return only which original block each translation block `
        + `renders.\n\n`
        + `PAIR ONLY WHAT CORRESPONDS. A translation may split one original paragraph `
        + `into several, merge several into one, add a paragraph the original never had, `
        + `or omit one entirely. Where a block has no counterpart, LEAVE IT OUT: an `
        + `omitted block is a correct answer and a wrong pairing is worse than none, `
        + `because later stages will report differences between two passages that were `
        + `never about the same thing.\n\n${pictureRule}`
        + `WHERE ONE ORIGINAL BLOCK IS RENDERED BY TWO TRANSLATION BLOCKS, pair the same `
        + `original index with each of them.\n\n`
        + `ORDER IS PRESERVED. Both documents say things in the same order, so your `
        + `pairs must never move backwards on either side.\n\n`
        + `Return JSON: {"pairs":[{"source":0,"target":0}]} with indices exactly as `
        + `numbered below.`,
    },
    {
      role: 'user',
      content: `ORIGINAL BLOCKS\n\n${
        renderBlocks({
          blocks: sourceBlocks,
          fence,
        },)
      }\n\nTRANSLATION BLOCKS\n\n${
        renderBlocks({
          blocks: shownTargetBlocks,
          fence,
        },)
      }${
        (pictureContext === '')
          ? ''
          : `\n\nWHAT THE PICTURES HERE SAY\n\n${fence}\n${pictureContext}\n${fence}`
      }`,
    },
  ];
}

/**
 Whether a parsed value has the shape of a pairing.

 SHAPE ONLY. Whether the pairing is usable is
 {@link readBlockPairing}'s question, because that needs the block counts.

 @param value - parsed model reply

 @returns Whether it is a {@link BlockPairingWire}

 @example
 ```ts
 const ok = isBlockPairingWire({ pairs: [], },);
 ```
 */
export function isBlockPairingWire(value: unknown,): value is BlockPairingWire {
  return isIndexPairingWire(value,);
}

/**
 Chunk-local indices of the blocks whose order carries no meaning: the
 footnote definitions on each side.

 @example
 ```ts
 const freeOrder: FreeOrderBlocks = { source: new Set([ 7, 8, ],), target: new Set([ 11, 12, ],), };
 ```
 */
export type FreeOrderBlocks = {
  /**
   Original-side definition indices.
   */
  readonly source: ReadonlySet<number>;

  /**
   Translation-side definition indices.
   */
  readonly target: ReadonlySet<number>;
};

/**
 No block exempt from the order rule, the reader's default.
 */
const NO_FREE_ORDER: FreeOrderBlocks = {
  source: new Set<number>(),
  target: new Set<number>(),
};

/**
 Whether a pair joins two definitions, two body blocks, or one of each.

 @param pair - pair to classify

 @param freeOrder - definition indices, empty when the caller named none

 @returns Class

 @example
 ```ts
 pairClass({ pair: { source: 7, target: 12, }, freeOrder, },);
 // => 'definition'
 ```
 */
function pairClass(
  {
    pair,
    freeOrder,
  }: {
    readonly pair: BlockPair;
    readonly freeOrder: FreeOrderBlocks;
  },
): 'definition' | 'body' | 'mixed' {
  /**
   Whether the original side is a definition.
   */
  const sourceFree = freeOrder.source
    .has(pair.source,);
  /**
   Whether the translation side is a definition.
   */
  const targetFree = freeOrder.target
    .has(pair.target,);
  if (sourceFree && targetFree)
    return 'definition';
  if (sourceFree || targetFree)
    return 'mixed';
  return 'body';
}

/**
 Whether an index names one of a side's blocks: a whole number from zero up
 to, and not including, how many blocks the side carries.

 @param index - block index a pair names

 @param count - blocks the side carries

 @returns Whether the side has a block at that index

 @example
 ```ts
 namesBlock({ index: 3, count: 1, },);
 // => false
 ```
 */
function namesBlock(
  {
    index,
    count,
  }: {
    readonly index: number;
    readonly count: number;
  },
): boolean {
  return Number.isInteger(index,)
    && (index >= 0)
    && (index < count);
}

/**
 Refuses a pairing that names a block its side does not carry.

 THE ONE WORDING of this refusal. A pairing reaches the blocks three ways: a
 model's reply ({@link readBlockPairing}), a record the block-pairing cache
 kept (`prepareBlockPairing`), and a settled artifact's recipe carved over a
 text the run may have parsed into other blocks (`blockPairingToSteps`). Each
 checks here, so a pairing that does not fit its blocks reads the same
 wherever it was found.

 @param pairs - correspondences about to be read against the blocks

 @param sourceCount - original blocks

 @param targetCount - translation blocks

 @throws BlockPairingError naming the first block no side carries, and how
 many blocks that side has

 @example
 ```ts
 assertPairsNameBlocks({ pairs: [{ source: 0, target: 3, },], sourceCount: 2, targetCount: 1, },);
 // throws: pairing names translation block 3, and there are 1
 ```
 */
export function assertPairsNameBlocks(
  {
    pairs,
    sourceCount,
    targetCount,
  }: {
    readonly pairs: readonly BlockPair[];
    readonly sourceCount: number;
    readonly targetCount: number;
  },
): void {
  for (const pair of pairs) {
    if (
      !namesBlock({
        index: pair.source,
        count: sourceCount,
      },)
    )
      throw new BlockPairingError({
        message: `pairing names original block ${String(pair.source,)}, and there are ${String(sourceCount,)}`,
      },);
    if (
      !namesBlock({
        index: pair.target,
        count: targetCount,
      },)
    )
      throw new BlockPairingError({
        message: `pairing names translation block ${String(pair.target,)}, and there are ${String(targetCount,)}`,
      },);
  }
}

/**
 Reads a model's pairing, refusing anything that cannot be used as one.

 REFUSES RATHER THAN REPAIRS. A pairing that runs backwards, names a block
 that does not exist, or names the same correspondence twice is not a
 near-miss to be tidied up: it is evidence the model did not do the task, and
 using part of it would put mismatched passages in front of the critics
 exactly as before.

 ONE ORIGINAL MAY APPEAR TWICE, because a translation splitting a paragraph is
 the correspondence this exists to express, and one TRANSLATION block may
 appear twice, because a translation merging two paragraphs is another. The
 same correspondence may not be named twice, adjacent or not, since the
 agreement over voices counts each naming as a vote.

 @param value - parsed model reply

 @param sourceCount - original blocks the sheet numbered

 @param targetCount - translation blocks the sheet numbered

 @returns Pairs in document order

 @throws BlockPairingError when the reply is not a usable pairing

 @example
 ```ts
 const pairs = readBlockPairing({ value, sourceCount: 4, targetCount: 5, },);
 ```
 */
export function readBlockPairing(
  {
    value,
    sourceCount,
    targetCount,
    freeOrder = NO_FREE_ORDER,
  }: {
    readonly value: unknown;
    readonly sourceCount: number;
    readonly targetCount: number;
    readonly freeOrder?: FreeOrderBlocks;
  },
): readonly BlockPair[] {
  if (!isBlockPairingWire(value,))
    throw new BlockPairingError({ message: 'reply is not a pairing: expected {"pairs":[{"source":n,"target":n}]}', },);

  /**
   Pairs in the order the model gave them.
   */
  const { pairs, } = value;
  assertPairsNameBlocks({
    pairs,
    sourceCount,
    targetCount,
  },);

  // MONOTONE ON BOTH SIDES, AND A REPEAT ON EITHER IS A REAL CORRESPONDENCE.
  //
  // A translation may SPLIT one original across several blocks, which repeats
  // the original, and it may MERGE several originals into one block, which
  // repeats the translation. Both happen in this corpus.
  //
  // AN EARLIER VERSION REFUSED THE MERGE, on the reasoning that a passage
  // renders one place. A live run refuted it: on `lintong` all six models
  // independently paired one translation block with two originals, every reply
  // was refused, and the entry fell back to scoring and collapsed to a single
  // slice. Six voices agreeing on a structure is evidence about the documents,
  // not six identical mistakes.
  //
  // What stays forbidden is going BACKWARDS, since both documents say things in
  // the same order, and standing still on BOTH sides at once, which repeats a
  // correspondence already made rather than describing a new one.
  //
  // FOOTNOTE DEFINITIONS ARE THE EXCEPTION TO THE ORDER, since a page renders
  // its notes by reference order and two documents may define the same notes
  // in a different order and both be right (the third `yuki418330012` launch
  // of 2026-09-08 lost six of eight voices to this rule when the archive had
  // renumbered its two notes). A definition pairs only with a definition, and
  // the order rule reads over the body pairs alone.
  for (const [at, pair,] of pairs.entries()) {
    /**
     Pair before this one, absent at the first position.
     */
    const previous = pairs[at - 1];
    if (pairClass({
      pair,
      freeOrder,
    },) === 'mixed')
      throw new BlockPairingError({
        message: `pairing pairs a footnote definition with a body block at position ${String(at,)}`,
      },);
    if (previous === undefined)
      continue;
    if ((pair.source === previous.source) && (pair.target === previous.target))
      throw new BlockPairingError({
        message: `pairing repeats the same correspondence at position ${String(at,)}`,
      },);
  }
  /**
   The body pairs with their positions, the ones the order rule reads.
   */
  const bodyPairs = [ ...pairs.entries(), ]
    .filter(function isBody([
      ,
      pair,
    ]: ReadonlyEntry<number, BlockPair>,): boolean {
      return pairClass({
        pair,
        freeOrder,
      },) === 'body';
    },);
  for (const [step, [at, pair,],] of bodyPairs.entries()) {
    /**
     Body pair before this one, absent at the first.
     */
    const previous = bodyPairs[step - 1]?.[1];
    if (previous === undefined)
      continue;
    if (pair.source < previous.source)
      throw new BlockPairingError({
        message: `pairing moves backwards on the original side at position ${String(at,)}`,
      },);
    if (pair.target < previous.target)
      throw new BlockPairingError({
        message: `pairing moves backwards on the translation side at position ${String(at,)}`,
      },);
  }
  // A correspondence is named once, wherever it stands. The check of each pair
  // against the one before it refuses an adjacent repeat, and a body pair
  // named again after others steps backwards; a definition pair is outside
  // the order rule, so a voice could name one again after another pair, and
  // the agreement over voices counts every naming as a vote, so one voice
  // would stand for two.
  /**
   Correspondences met so far, by original and translation block.
   */
  const named = new Set<string>();
  for (const [at, pair,] of pairs.entries()) {
    /**
     Both blocks this correspondence names, as one key.
     */
    const key = `${String(pair.source,)},${String(pair.target,)}`;
    if (named.has(key,))
      throw new BlockPairingError({
        message: `pairing repeats the same correspondence at position ${String(at,)}`,
      },);
    named.add(key,);
  }
  return pairs;
}

//endregion Block pairing wire
