import { containerFootnoteDisagreementsOf, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  blocksOfPage,
  blocksOfSource,
  PageSyntaxError,
  readerOfPage,
  readerOfSource,
  withoutLinkTargets,
} from './source-reading-account.ts';
import { blockDifferenceOf, } from './source-reading-blocks.ts';

//region Source reading difference
// WHETHER A READER OF THE PAGE SEES WHAT A READER OF THE SOURCE SEES. The owner ruled on 2026-10-25 ("Add the comparison")
// that the page check compares the two readings, after the search of the third round found 1407 of 11423 texts that
// the encoder writes so that a reader sees other text, with no check to say so. The comparison is the oracle the unit
// tests of the encoder use (`source-reading-account.ts`): the text of every leaf of the page read by the strict grammar,
// against the text of every leaf of the source read as plain Markdown with GFM, math and raw HTML.
//
// THREE ANSWERS. `same` is the common case and says nothing. `differs` names why, for a defect that ships with the
// page and never stops it (a run always ships, the owner, 2026-09-27). `undecided` is the one place the oracle cannot
// speak for the ruling: the two-readings ruling takes the source's footnote reading at the top of a text and the page's
// inside a block quote or a list item and after a footnote's last block. A reader of plain Markdown gives only the
// source's, so where the two readings of a footnote disagree inside a container a difference between the accounts may be
// the ruling's own choice. That is decided per top-level block (`source-reading-blocks.ts`): a block that differs on
// its own is a difference whatever stands beside a block the readings disagree on, and a block that stays undecided is
// reported in a defect of its own by the caller (`source-ships-differences.ts`), so that no page is silent. A page that
// holds a construct the strict grammar reads as syntax (an expression, a tag) shows a reader other than the text and is
// a difference.

/**
 What the comparison says of a page and the source it carries.
 */
export type ReadingDifference = {
  /**
   The answer.
   */
  readonly kind: 'differs' | 'same' | 'undecided';

  /**
   Why, in words that name no text of the page; absent for `same`.
   */
  readonly reason?: string;

  /**
   How many blocks of the source stay undecided beside the block that differs; present only for `differs`, and only when
   some block does.
   */
  readonly undecidedBlocks?: number;
};

/**
 Why a page that holds syntax is a difference.
 */
const SYNTAX_REASON = 'the page holds a construct the strict grammar reads as syntax, which shows a reader other than the text';

/**
 Why a page that shows other text than the source is a difference.
 */
const OTHER_TEXT_REASON = 'the page shows other text than the source';

/**
 Why a difference in a block with a footnote in a container is not decided.
 */
const CONTAINER_REASON = 'the footnote ranges of the two readings differ inside a block quote or a list item, where the '
  + 'two-readings ruling takes the page\'s reading, which a reader of plain Markdown cannot give';

/**
 Says whether a reader of the page sees what a reader of the source sees.

 @param source - text of a run of slices as the artifact records it

 @param carried - the same text as the page carries it after the note, encoded or as written

 @returns `same` when the accounts agree, `differs` when they do not, `undecided` when they do not and the difference
 sits where the two-readings ruling takes the page's reading

 @throws Whatever the strict grammar raises when the page does not parse, which the composition rules out before it asks

 @example
 ```ts
 const answer = readingDifferenceOf({ source: '她说 { 然后走了。', carried: String.raw`她说 \{ 然后走了。`, },);
 ```
 */
export function readingDifferenceOf(
  {
    source,
    carried,
  }: {
    readonly source: string;
    readonly carried: string;
  },
): ReadingDifference {
  try {
    if (
      withoutLinkTargets({ account: readerOfPage({ carried, },), },)
        === withoutLinkTargets({ account: readerOfSource({ source, },), },)
    )
      return { kind: 'same', };
  }
  catch (error) {
    if (error instanceof PageSyntaxError) {
      return {
        kind: 'differs',
        reason: SYNTAX_REASON,
      };
    }
    throw error;
  }
  /**
   Where the two readings of a footnote disagree inside a block quote or a list item.
   */
  const disagreements = containerFootnoteDisagreementsOf({ body: source, },);
  if (disagreements.length === 0) {
    return {
      kind: 'differs',
      reason: OTHER_TEXT_REASON,
    };
  }
  /**
   What the blocks of the source and of the page say of each other.
   */
  const {
    differs,
    undecidedBlocks,
  } = blockDifferenceOf({
    sourceBlocks: blocksOfSource({ source, },),
    pageBlocks: blocksOfPage({ carried, },),
    disagreements,
  },);
  if (differs || (undecidedBlocks === 0)) {
    return {
      kind: 'differs',
      reason: OTHER_TEXT_REASON,
      ...((undecidedBlocks === 0) ? {} : { undecidedBlocks, }),
    };
  }
  return {
    kind: 'undecided',
    reason: CONTAINER_REASON,
  };
}

//endregion Source reading difference
