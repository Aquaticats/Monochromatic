import {
  rendersAsNothing,
  wordForCount,
} from '@monochromatic-dev/translation-repair-typography/ts';
import {
  type BlockShape,
  describeBlocks,
  readSliceSkeleton,
  sameShape,
} from '@monochromatic-dev/translation-repair-glossary/ts';

//region Archive revision shape
// THE SEVENTY-SEVENTH CLASS (zheermao2, 2026-09-21). The archive block review
// asks reviewers to revise one archive-only block and admits any revision
// that keeps the contributor identities. On the one-line label introducing
// the English rendering of an email exchange ("English translation of the
// preceding email conversation:"), one reviewer answered with a four-block
// letter, its third paragraph cut mid-sentence, and the selection judges
// chose it over the label as "the actual email translation content required
// for the block". The block's own shape is the deterministic bound the lanes
// already hold their candidates to (`translate-validate.ts`): a revision of
// one paragraph is one paragraph, a revised blockquote is a blockquote. A
// removal (an empty revision) is a shape of its own the review allows.
//
// A QUOTED PASSAGE NESTED BELOW THE TOP LEVEL IS COMPARED TOO (ledger B110).
// `sameShape` reads only the top-level block sequence, so a revision keeping
// the block's own shape could still turn a blockquote nested inside a
// container tag, a list, a footnote, or another blockquote into prose;
// `quotedPassageRevisionFindings` reads the parse's own any-depth count.

/**
 Finding prefix a revision withheld on shape is recorded under.
 */
export const REVISION_SHAPE_REFUSED = 'archive-revision-refused';

/**
 A block sequence as a refusal says it: the count, then each block's shape,
 as the translate lanes' block floor says it (`translate-validate-blocks.ts`).
 It once said the shapes alone, "the block is paragraph" (ledger T8,
 eighteenth batch).

 @param blocks - blocks in document order

 @returns Count and shapes, or a word for none

 @example
 ```ts
 countedBlocks({ blocks: [{ kind: 'paragraph', detail: '', },], },); // '1 block (paragraph)'
 ```
 */
function countedBlocks({ blocks, }: { readonly blocks: readonly BlockShape[]; },): string {
  if (blocks.length === 0)
    return 'nothing';
  return `${String(blocks.length,)} ${
    wordForCount({
      count: blocks.length,
      one: 'block',
      many: 'blocks',
    },)
  } (${describeBlocks({ blocks, },)})`;
}

/**
 Findings for a revision that carries fewer quoted passages, at every depth,
 than the block it revises.

 ASKED ONLY OF A REVISION WITH THE BLOCK'S OWN SHAPE. `sameShape` compares
 only the top-level block sequence, so a revision may keep it kind for kind
 while a blockquote nested inside a container tag, a list, a footnote, or
 another blockquote is turned into prose underneath it (ledger B110, the
 twin of the translate floor's `quotedPassageFloorFindings`); a revision of
 another shape is already withheld for its shape.

 @param modelId - reviewer who wrote the revision, named in the finding

 @param blockQuotedPassages - block's blockquotes at every depth

 @param revisionQuotedPassages - revision's blockquotes at every depth

 @returns One finding where the revision carries fewer, empty otherwise

 @example
 ```ts
 const findings = quotedPassageRevisionFindings({ modelId, blockQuotedPassages: 1, revisionQuotedPassages: 0, },);
 ```
 */
function quotedPassageRevisionFindings(
  {
    modelId,
    blockQuotedPassages,
    revisionQuotedPassages,
  }: {
    readonly modelId: string;
    readonly blockQuotedPassages: number;
    readonly revisionQuotedPassages: number;
  },
): readonly string[] {
  if (revisionQuotedPassages >= blockQuotedPassages)
    return [];
  return [
    `${REVISION_SHAPE_REFUSED} (${modelId}): the block carries ${String(blockQuotedPassages,)} ${
      wordForCount({
        count: blockQuotedPassages,
        one: 'quoted passage',
        many: 'quoted passages',
      },)
    } and the revision carries ${
      String(revisionQuotedPassages,)
    }; a revision keeps every quoted passage the block carries, including one nested inside a container tag, a `
      + 'list, a footnote, or another blockquote',
  ];
}

/**
 Why a revision cannot replace the block it revises, when its shape is not
 the block's own or it carries fewer quoted passages at any depth.

 A RULE THAT CANNOT READ THE BLOCK SAYS NOTHING: an archive block the slice
 grammar refuses has no shape to hold a revision to, so the revision passes
 to the judges as before. A revision the grammar refuses is withheld, since
 it could not be published either way.

 @param modelId - reviewer who wrote the revision, named in the finding

 @param blockText - archive block under review

 @param replacementText - revision as it would ship, empty for a removal

 @returns Findings withholding the revision, empty when it may stand

 @throws {@link Error} when the revision shows a reader nothing and is not
 empty, since the review stage reads such a revision as the empty removal
 before any revision reaches this check, and a second reading here would
 hold it to a shape the removal does not have

 @example
 ```ts
 const findings = revisionShapeFindings({ modelId, blockText, replacementText, },);
 ```
 */
export function revisionShapeFindings(
  {
    modelId,
    blockText,
    replacementText,
  }: {
    readonly modelId: string;
    readonly blockText: string;
    readonly replacementText: string;
  },
): readonly string[] {
  // A REMOVAL IS A SHAPE OF ITS OWN the review allows.
  if (replacementText === '')
    return [];
  if (rendersAsNothing({ text: replacementText, },)) {
    throw new Error(
      `unreachable: ${modelId}'s revision shows a reader nothing and is not empty, though the review stage reads `
        + 'such a revision as the empty removal before the slate is built',
    );
  }
  /**
   Shape of the block under review.
   */
  const block = readSliceSkeleton({ text: blockText, },);
  if (block.kind !== 'read')
    return [];
  /**
   Blocks of the block under review.
   */
  const blockShapes = block.skeleton
    .blocks;
  /**
   Shape of the revision.
   */
  const revision = readSliceSkeleton({ text: replacementText, },);
  if (revision.kind !== 'read')
    return [
      `${REVISION_SHAPE_REFUSED} (${modelId}): the revision does not parse (${revision.detail}); the block is ${
        countedBlocks({ blocks: blockShapes, },)
      }`,
    ];
  /**
   Blocks of the revision.
   */
  const revisionShapes = revision.skeleton
    .blocks;
  if (!sameShape({
    left: blockShapes,
    right: revisionShapes,
  },))
    {
    return [
      `${REVISION_SHAPE_REFUSED} (${modelId}): the block is ${
        countedBlocks({ blocks: blockShapes, },)
      } and the revision is ${
        countedBlocks({ blocks: revisionShapes, },)
      }; a revision keeps the block's own shape`,
    ];
  }
  return quotedPassageRevisionFindings({
    modelId,
    blockQuotedPassages: block.skeleton
      .quotedPassages,
    revisionQuotedPassages: revision.skeleton
      .quotedPassages,
  },);
}

//endregion Archive revision shape
