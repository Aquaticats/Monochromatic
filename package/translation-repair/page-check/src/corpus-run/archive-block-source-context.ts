import {
  alignDocumentSections,
  mostCarriedReading,
  type PairedReading,
} from '@monochromatic-dev/translation-repair-slice-reading/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import type { PreparedDocumentPair, } from '@monochromatic-dev/translation-repair-corpus-source/ts';
import {
  parseDocument,
  photoReferences,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { ArchiveBlockAlignmentMissingError, } from './archive-block-context-errors.ts';
import { archiveBlockIdentity, } from './archive-block-repair.ts';

//region Archive block source context

/**
 Source section text of the aligned pair a block sits in.

 @param alignment - section alignment preparation consumed

 @param blockId - id of the block, for the refusal

 @param pairIndex - aligned pair the block names

 @returns The pair's source text

 @throws {@link ArchiveBlockAlignmentMissingError} when the alignment holds no
 such pair, which means the preparation and the alignment disagree about the
 block's place (ledger B717)

 @example
 ```ts
 const text = alignedSourceText({ alignment, blockId: 'block/0', pairIndex: 0, },);
 ```
 */
function alignedSourceText(
  {
    alignment,
    blockId,
    pairIndex,
  }: {
    readonly alignment: ReturnType<typeof alignDocumentSections>;
    readonly blockId: string;
    readonly pairIndex: number;
  },
): string {
  /**
   Pair the block names.
   */
  const pair = alignment.pairs
    .at(pairIndex,);
  if (pair === undefined) {
    throw new ArchiveBlockAlignmentMissingError({
      blockId,
      pairIndex,
      pairCount: alignment.pairs
        .length,
    },);
  }
  return pair.source
    .text;
}

/**
 Maps each unclaimed archive block to source section it was expected to render.

 Target-only sections map to empty context because no source section can license
 factual wording there.

 @param prepared - current paired preparation

 @param pictureReadings - completed entry evidence, restricted to each section's references

 @returns Exact block identity to source-section text and corroborated picture support

 @throws {@link ArchiveBlockAlignmentMissingError} when a block names an
 aligned pair the alignment of the same texts does not hold

 @example
 ```ts
 const contexts = archiveBlockSourceContexts({ prepared, });
 ```
 */
export function archiveBlockSourceContexts(
  {
    prepared,
    pictureReadings = new Map(),
  }: {
    readonly prepared: PreparedDocumentPair;
    readonly pictureReadings?: ReadonlyMap<string, PairedReading>;
  },
): ReadonlyMap<string, string> {
  /**
   Section alignment preparation consumed.
   */
  const alignment = alignDocumentSections({
    source: parseDocument({ text: prepared.sourceText, },),
    target: parseDocument({ text: prepared.targetText, },),
    ...((prepared.sectionPairing === undefined)
      ? {}
      : { sectionPairing: prepared.sectionPairing, }),
  },);
  return new Map(prepared.unclaimedTargetBlocks
    .map(function toContext(block,): readonly [
      string,
      string
    ] {
    /**
     Source section paired with archive block location.
     */
      const sourceContext = block.location
        .kind
        === 'aligned-pair'
        ? alignedSourceText({
          alignment,
          blockId: block.blockId,
          pairIndex: block.location
            .pairIndex,
        },)
        : '';
    /**
     Names this source section alone authorizes, each once.
     */
      const names = new Set(photoReferences({ text: sourceContext, },)
        .map(function asset(reference,): string {
          return reference.assetName;
        },),);
    /**
     One transcript per corroborated picture, the one the other readers carry
     most, and never unrelated or unavailable evidence.

     EVERY READER'S TRANSCRIPT WENT HERE UNTIL 2026-09-16. On Mio13 seven of
     twelve reviewers spent their whole completion cap reasoning about the
     first chat translation against six transcripts of its screenshots and
     sent no content, while the block before it drew 640 to 1,268 tokens
     from the same seats on a prompt of the same size. The owner chose one
     transcript per picture over a doubled re-ask cap
     (`doc/decision/translation-repair-archive-review-one-transcript-2026-09-16.md`).
     */
      const support = [...names,].flatMap(function pictureSupport(assetName,): readonly string[] {
      /**
       Completed reading for this section's reference.
       */
        const reading = pictureReadings.get(assetName,);
        if (reading?.kind !== 'corroborated')
          return [];
      /**
       Transcript the other readers carry most.
       */
        const chosen = mostCarriedReading({ readings: reading.readings, },);
      /**
       Readers that agreed on the picture.
       */
        const readerCount = reading.readings
          .length;
        return [
          `CORROBORATED PICTURE SOURCE SUPPORT ${assetName} (${
          String(readerCount,)
        } ${
          wordForCount({
            count: readerCount,
            one: 'reader',
            many: 'readers',
          },)
        } agree; the transcript the others carry most)\n${chosen.text}`,
        ];
      },);
      return [
        archiveBlockIdentity({
          block,
          targetText: prepared.targetText,
        },),
        [
          sourceContext,
          ...support,
        ].join('\n\n',),
      ] as const;
    },),);
}

//endregion Archive block source context
