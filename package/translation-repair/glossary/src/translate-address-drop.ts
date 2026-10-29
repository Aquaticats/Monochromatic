import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import { howOften, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  parseSliceBody,
  requireMdxRefusal,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import { readSliceSkeleton, } from './translate-skeleton.ts';
import {
  addressCount,
  hanThirdPersonCount,
} from './translate-address-original.ts';
import { lowerCaseLatinWords, } from '@monochromatic-dev/translation-repair-markup/ts';

//region Second-person address the passage carries
// CLASS NINETY-SEVEN (yingying5, 2026-09-23). The original's closing wish
// addresses the deceased directly (愿在你的下一个世界……), the archive wrote
// it in the third person, the repair lane wrote "you" and the translate lane
// copied the archive; the contest tied two to two, the archive's line stood,
// and the gate kept it over the consolidated "you" with one ballot reading
// the Chinese as "she". yingying1 to 4 had shipped the second person. A
// pronoun the ORIGINAL writes is rendered as written where it stands (class
// eighty-one), and a rendering that turns 你 into she or they is refused
// before any judge reads it, as a dropped marker is (class ninety-two). THE
// FLOOR IS NARROW: a greeting (你好) addresses nobody, and a rendering that
// carries no pronoun at all ("Ah, wanna play?" for 干干你的) is left to the
// judges; only a rendering with no second-person pronoun and a third-person
// one in its place fails. The corpus census of 2026-09-23 found 67 of 92
// archive pages carrying 你, and of 31 aligned blocks six without "you": four
// greetings, colloquial drops or misalignments and two person switches.
//
// LEDGER F-1 (2026-09-27): read block by block where the texts have as many
// blocks, and "in its place" means a third-person pronoun beyond the ones the
// original writes there; `translate-address-original.ts` reads the original.
// Replayed over 1,264 archive slices and 3,975 would-ship slices, refusals
// fell from 25 to 13 with none added. THE COUNT HAS A LIMIT: English writes
// pronouns Chinese leaves out, so a block whose 你 is generic ("逼着你干",
// "makes you do") beside a subject-dropped description can still show a
// surplus with no switch in it (lintong slice 1 is the one left).

/**
 English second-person pronouns, lower case.
 */
const SECOND_PERSON: ReadonlySet<string> = new Set([
  'you',
  'your',
  'yours',
  'yourself',
  'yourselves',
]);

/**
 English third-person pronouns, lower case.
 */
const THIRD_PERSON: ReadonlySet<string> = new Set([
  'she',
  'he',
  'her',
  'him',
  'his',
  'hers',
  'herself',
  'himself',
  'they',
  'them',
  'their',
  'theirs',
  'themselves',
]);

/**
 Opening of an HTML comment.
 */
const COMMENT_OPEN = '<!--';

/**
 Closing of an HTML comment.
 */
const COMMENT_CLOSE = '-->';

/**
 Text with its HTML comments cut out, so a comment's words count for
 nothing.

 @param text - passage to strip

 @returns Passage without its comments

 @example
 ```ts
 withoutHtmlComments({ text: '<!-- 你 --> 猫', },); // ' 猫'
 ```
 */
export function withoutHtmlComments({ text, }: { readonly text: string; },): string {
  /**
   Kept pieces, in order.
   */
  const kept: string[] = [];
  for (let at = 0; at <= text.length;) {
    /**
     Where the next comment opens, -1 for none.
     */
    const open = text.indexOf(
      COMMENT_OPEN,
      at,
    );
    if (open === (-1)) {
      kept.push(text.slice(at,),);
      break;
    }
    kept.push(text.slice(
      at,
      open,
    ),);
    /**
     Where that comment closes, -1 for an unclosed one, which runs to the end.
     */
    const close = text.indexOf(
      COMMENT_CLOSE,
      open + COMMENT_OPEN.length,
    );
    if (close === (-1))
      break;
    at = close + COMMENT_CLOSE.length;
  }
  return kept.join('',);
}

/**
 A passage's top-level blocks as the slice grammar reads them, or the whole
 passage as one block where the grammar refuses it.

 READ OFF THE PARSE (ledger B68), not split on blank lines: a fence carrying
 a blank line between its own lines is one block, as it is to the floor, so
 the two sides' counts do not part for spacing either side's wording never
 chose. A passage the grammar refuses has no blocks to pair, and reading it
 whole is what an unequal count already falls back to.

 @internal

 @param text - passage, comments already cut

 @returns Blocks in order, each the exact source text of one top-level node

 @example
 ```ts
 blocksOrWholeText({ text: 'a\n\nb', },); // ['a', 'b']
 ```
 */
export function blocksOrWholeText({ text, }: { readonly text: string; },): readonly string[] {
  try {
    return parseSliceBody({ text, },)
      .root
      .children
      .map(function blockText(node,): string {
        return text.slice(
          nonNullishOrThrow(node.position
            ?.start
            .offset,),
          nonNullishOrThrow(node.position
            ?.end
            .offset,),
        );
      },);
  }
  catch (error) {
    // Only the grammar's own refusal reads the passage whole; anything else
    // is an unexpected state that must keep propagating.
    requireMdxRefusal({ error, },);
    return [text,];
  }
}

/**
 How many third-person pronouns an original passage writes: the Han ones,
 and the romanised TA or ta.

 @param text - original passage, comments already cut

 @returns Count of third-person pronouns

 @example
 ```ts
 thirdPersonCount({ text: '她说 ta 的其他猫', },); // 2
 ```
 */
function thirdPersonCount({ text, }: { readonly text: string; },): number {
  /**
   Romanised third-person pronouns among the passage's Latin words.
   */
  const romanised = lowerCaseLatinWords({ text, },)
    .filter(function isTa(word,): boolean {
      return word === 'ta';
    },);
  return hanThirdPersonCount({ text, },) + romanised.length;
}

/**
 A block of the original and the rendering's block in the same place.
 */
type SideBySide = {
  readonly original: string;
  readonly rendering: string;
};

/**
 One block where the address turned into narration: how often the original
 addresses someone there, how many third-person pronouns it writes there, and
 the ones the rendering carries instead.
 */
type SwitchedBlock = {
  readonly addresses: number;
  readonly written: number;
  readonly rendered: readonly string[];
};

/**
 Reads one pair of blocks for a person switch.

 @param original - block of the original, comments already cut

 @param rendering - rendering's block in the same place

 @returns Switch, or none where the block addresses nobody, the rendering
 keeps a second-person pronoun, or its third-person pronouns do not outnumber
 the original's own

 @example
 ```ts
 switchedBlock({ original: '愿你安好。', rendering: 'May she be well.', },);
 ```
 */
function switchedBlock(
  {
    original,
    rendering,
  }: {
    readonly original: string;
    readonly rendering: string;
  },
): readonly SwitchedBlock[] {
  /**
   How often the block addresses someone.
   */
  const addresses = addressCount({ text: original, },);
  if (addresses === 0)
    return [];
  /**
   Words of the rendering's block.
   */
  const words = lowerCaseLatinWords({ text: rendering, },);
  if (words.some(function secondPerson(word,): boolean {
    return SECOND_PERSON.has(word,);
  },))
    return [];
  /**
   Third-person pronouns the rendering carries.
   */
  const rendered = words.filter(function isThird(word,): boolean {
    return THIRD_PERSON.has(word,);
  },);
  /**
   Third-person pronouns the original writes in the same block.
   */
  const written = thirdPersonCount({ text: original, },);
  if (rendered.length <= written)
    return [];
  return [
    {
      addresses,
      written,
      rendered,
    },
  ];
}

/**
 Pairs the original and the rendering block by block where the slice grammar
 reads as many blocks on both sides, or the whole texts where the counts
 differ.

 @param original - original outside its comments

 @param rendering - candidate outside its comments

 @returns Blocks read side by side, or the two whole texts as one pair

 @example
 ```ts
 const pairs = blockPairs({ original: '你好。', rendering: 'Hello.', },);
 ```
 */
function blockPairs(
  {
    original,
    rendering,
  }: {
    readonly original: string;
    readonly rendering: string;
  },
): readonly SideBySide[] {
  /**
   Whole texts as one pair, named once so every fallback path reads alike.
   */
  const whole: readonly SideBySide[] = [{
    original,
    rendering,
  },];
  /**
   Blocks of the original.
   */
  const originalBlocks = blocksOrWholeText({ text: original, },);
  /**
   Blocks of the candidate.
   */
  const renderingBlocks = blocksOrWholeText({ text: rendering, },);
  if (originalBlocks.length !== renderingBlocks.length)
    return whole;
  return originalBlocks.map(function paired(
    block,
    index,
  ): SideBySide {
    return {
      original: block,
      rendering: nonNullishOrThrow(renderingBlocks[index],),
    };
  },);
}

/**
 Findings for a candidate that renders a passage the original addresses in
 the second person with a third-person pronoun and no second-person one:
 read block by block where the two texts have as many blocks, else whole, and
 refused only where the rendering's third-person pronouns outnumber the
 original's own.

 A CANDIDATE THE SLICE GRAMMAR REFUSES IS LEFT TO THE PARSE FLOOR, which
 refuses it with the parser's own account (`translate-validate.ts`). Read
 whole here, its address would be counted across blocks the switch never
 spanned, and this finding, which runs first, would stand in for the one
 that names what is wrong (ledger B100).

 @param sourceText - original slice

 @param candidateText - candidate under validation

 @returns One finding, or none where the address is kept or no surplus pronoun
 stands in its place

 @example
 ```ts
 const findings = droppedAddressFindings({ sourceText: '愿你安好。', candidateText: 'May she be well.', },);
 ```
 */
export function droppedAddressFindings(
  {
    sourceText,
    candidateText,
  }: {
    readonly sourceText: string;
    readonly candidateText: string;
  },
): readonly string[] {
  /**
   Original outside its comments.
   */
  const original = withoutHtmlComments({ text: sourceText, },);
  /**
   Candidate outside its comments.
   */
  const rendering = withoutHtmlComments({ text: candidateText, },);
  /**
   How the slice grammar reads the candidate.
   */
  const reading = readSliceSkeleton({ text: rendering, },);
  if (reading.kind === 'unparseable')
    return [];
  /**
   Blocks read side by side, or the whole texts as one pair.
   */
  const pairs = blockPairs({
    original,
    rendering,
  },);
  /**
   Blocks where the address turned into narration.
   */
  const switched = pairs.flatMap(switchedBlock,);
  if (switched.length === 0)
    return [];
  /**
   How often the original addresses someone in those blocks, and how many
   third-person pronouns it writes there.
   */
  const {
    addresses,
    written,
  } = switched.reduce(
    function addBlock(
      sum,
      block,
    ): {
      readonly addresses: number;
      readonly written: number;
    } {
      return {
        addresses: sum.addresses + block.addresses,
        written: sum.written + block.written,
      };
    },
    {
      addresses: 0,
      written: 0,
    },
  );
  /**
   Third-person pronouns the rendering carries in those blocks.
   */
  const rendered = switched.flatMap(function words(block,): readonly string[] {
    return block.rendered;
  },);
  /**
   Third-person pronouns quoted for the finding.
   */
  const quoted = [...new Set(rendered,),]
    .map(function quote(word,): string {
      return `"${word}"`;
    },)
    .join(', ',);
  /**
   How often, in words.
   */
  const times = howOften({ count: addresses, },);
  return [
    `Your translation drops the address in the second person the ORIGINAL carries: where the ORIGINAL writes 你 or 您 ${times}, your translation carries no "you" and more third-person pronouns than the ORIGINAL writes there (${quoted}: ${String(rendered.length,)} against ${String(written,)}), so a pronoun stands where the address stood. A pronoun the ORIGINAL writes is rendered as written where it stands: address the person the ORIGINAL addresses.`,
  ];
}

//endregion Second-person address the passage carries
