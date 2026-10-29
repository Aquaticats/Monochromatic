import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';
import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import { runArchiveBlockReviewStage, } from '@monochromatic-dev/translation-repair-archive/ts';
import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import type { UnclaimedTargetBlock, } from '@monochromatic-dev/translation-repair-corpus-source/ts';
import {
  hashContent,
  withoutTranslatorsNotes,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  type BenchSeating,
  sameWording,
} from '@monochromatic-dev/translation-repair-translate-floor/ts';
import type { StageSettings, } from '@monochromatic-dev/translation-repair-stage/ts';
import {
  rendersAsNothing,
  wordForCount,
} from '@monochromatic-dev/translation-repair-typography/ts';
import { ArchiveBlockContextMissingError, } from './archive-block-context-errors.ts';
import {
  keptNotesFinding,
  withBlockNotes,
} from './archive-block-notes.ts';

//region Archive block repair

/**
 Outcome of reviewing all currently unclaimed archive blocks.
 */
export type ArchiveBlocksRepairOutcome = {
  /**
   Archive text after selected revisions.
   */
  readonly targetText: string;
  /**
   Audit findings in the order the blocks were reviewed (latest first). A block
   whose review stage reported findings contributes a header naming the block
   (`archive block review findings (N): <identity>`), the N findings as the
   stage worded them, and then the line saying what became of it.
   */
  readonly findings: readonly string[];
};

/**
 Stable identity over location and exact block wording.

 @param block - structured unclaimed block

 @param targetText - archive whose offsets block indexes

 @returns Identity unaffected by edits after block

 @example
 ```ts
 const identity = archiveBlockIdentity({ block, targetText, });
 ```
 */
export function archiveBlockIdentity(
  {
    block,
    targetText,
  }: {
    readonly block: UnclaimedTargetBlock;
    readonly targetText: string;
  },
): string {
  return JSON.stringify({
    location: block.location,
    blockId: block.blockId,
    textDigest: hashContent({
      content: targetText.slice(
        block.startOffset,
        block.endOffset,
      ),
    },),
  },);
}

/**
 Line-ending characters, the only thing a block separator is made of.
 */
const LINE_ENDINGS: ReadonlySet<string> = new Set([
  '\n',
  '\r',
]);

/**
 Offset just past the run of line endings starting at an offset.

 @param text - document scanned

 @param offset - where the run may start

 @returns Offset of the first character that is no line ending, or the text's length

 @example
 ```ts
 pastLineEndings({ text: 'A\n\nB', offset: 1, },); // 3
 ```
 */
function pastLineEndings(
  {
    text,
    offset,
  }: {
    readonly text: string;
    readonly offset: number;
  },
): number {
  for (let at = offset; at < text.length; at += 1) {
    if (!LINE_ENDINGS.has(nonNullishOrThrow(text[at],),))
      return at;
  }
  return text.length;
}

/**
 Offset where the run of line endings ending at an offset starts.

 @param text - document scanned

 @param offset - offset just past the run

 @returns Offset of the run's first line ending, or the offset itself where none precedes it

 @example
 ```ts
 beforeLineEndings({ text: 'A\n\nB', offset: 3, },); // 1
 ```
 */
function beforeLineEndings(
  {
    text,
    offset,
  }: {
    readonly text: string;
    readonly offset: number;
  },
): number {
  for (let at = offset; at > 0; at -= 1) {
    if (!LINE_ENDINGS.has(nonNullishOrThrow(text[at - 1],),))
      return at;
  }
  return 0;
}

/**
 Span to cut when a review removes a block outright.

 CLASS NINETY-FOUR (XingZ627, 2026-09-23). A removal spliced as an empty
 replacement left the block's own separators standing on both sides, so the
 archive's placeholder line after the front matter left three blank lines
 behind it and the page shipped with a double blank line. The removed block
 takes the line endings that follow it, which leaves the separator before
 it to stand between its neighbours; a block that ends the document takes
 the line endings before it instead, so nothing trails.

 @param text - document the block sits in

 @param startOffset - where the block starts

 @param endOffset - offset just past the block

 @returns Span covering the block and one run of line endings beside it

 @example
 ```ts
 const span = removalSpan({ text: 'A\n\nB\n\nC', startOffset: 3, endOffset: 4, },);
 ```
 */
function removalSpan(
  {
    text,
    startOffset,
    endOffset,
  }: {
    readonly text: string;
    readonly startOffset: number;
    readonly endOffset: number;
  },
): {
  readonly startOffset: number;
  readonly endOffset: number;
} {
  /**
   Offset just past the line endings that follow the block.
   */
  const after = pastLineEndings({
    text,
    offset: endOffset,
  },);
  if (after > endOffset)
    return {
      startOffset,
      endOffset: after,
    };
  return {
    startOffset: beforeLineEndings({
      text,
      offset: startOffset,
    },),
    endOffset,
  };
}

/**
 Reviews unclaimed blocks in reverse offset order and applies selected revisions.

 EACH BLOCK IS REVIEWED IN THE PAGE THE REVISIONS ALREADY APPLIED LEAVE
 (ledger B80), so its reviewers, the quote style its revisions are restored
 to, and the footnote check on them all read the page a revision would
 join. Where two blocks' revisions conflict, the later block's, reviewed
 first, stands.

 A BLOCK'S NOTES BETWEEN TRANSLATORS SURVIVE ITS REPLACEMENT
 (`archive-block-notes.ts`): a revision, or a removal, that lacks a note the
 block held gets it back on a line of its own after the replacement, as the
 page assembly would carry it, and the finding and the log name it.

 @param client - provider client

 @param modelIds - review roster

 @param targetText - current archive document

 @param sourceContexts - expected source section per exact block identity

 @param blocks - unclaimed blocks in current preparation

 @param identityContext - declared names preparation holds, for every sheet
 the review asks (ledger B28)

 @param referenceContext - what the pages the original links say, with the
 attested lines under them

 @param signal - caller cancellation

 @param exchangeTimeoutMs - per-call bound

 @param l - pass logger

 @param beforeBlock - awaited before each block's review, handing it the roster it
 is asked of; none keeps the given one (ledger X12)

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @returns Revised text, retained identities, and audit findings

 @throws {@link ArchiveBlockContextMissingError} when `sourceContexts` holds
 no entry for a block's identity, which means the caller and the map disagree
 about the block

 @example
 ```ts
 const repaired = await repairArchiveBlocks(input);
 ```
 */
export async function repairArchiveBlocks(
  {
    client,
    modelIds,
    targetText,
    sourceContexts,
    blocks,
    identityContext,
    referenceContext,
    signal,
    exchangeTimeoutMs,
    l,
    beforeBlock,
    stageSettings,
  }: ForeignBorrowed<{
    readonly client: ModelClient;
    readonly modelIds: readonly RosterModelId[];
    readonly targetText: string;
    readonly sourceContexts: ReadonlyMap<string, string>;
    readonly blocks: readonly UnclaimedTargetBlock[];
    readonly identityContext?: string;
    readonly referenceContext?: string;
    readonly signal: AbortSignal;
    readonly exchangeTimeoutMs: number;
    readonly l: Logger;
    readonly beforeBlock?: () => Promise<BenchSeating>;
    readonly stageSettings: StageSettings;
  }>,
): Promise<ArchiveBlocksRepairOutcome> {
  /**
   Operation logger, naming the function in every line.
   */
  const repairLog = tagged({
    l,
    tag: repairArchiveBlocks.name,
  },);
  /**
   Audit findings in review order.
   */
  const findings: string[] = [];
  /* oxlint-disable no-restricted-syntax/no-function-root-let -- Reverse-offset sequential splicing carries each accepted correction forward. */
  /**
   Archive with selected corrections applied.
   */
  let revisedText = targetText;
  /* oxlint-enable no-restricted-syntax/no-function-root-let */
  /**
   Blocks ordered so replacement cannot invalidate later offsets.
   */
  const ordered = blocks.toSorted(function latestFirst(
    left,
    right,
  ): number {
    return right.startOffset - left.startOffset;
  },);
  for (const block of ordered) {
    /**
     Stable license identity for current block.
     */
    const identity = archiveBlockIdentity({
      block,
      targetText,
    },);
    /**
     Exact archive wording under review.
     */
    const blockText = targetText.slice(
      block.startOffset,
      block.endOffset,
    );
    /**
     Original section the block is reviewed against, whose own comments are
     the lanes' to carry. A section only the archive has is filed as empty; a
     block with no entry at all is a fault (ledger B717), never an empty one.
     */
    const source = sourceContexts.get(identity,);
    if (source === undefined) {
      throw new ArchiveBlockContextMissingError({
        identity,
        held: [...sourceContexts.keys(),],
      },);
    }
    /**
     The block as the reviewers' sheet shows it, without its notes.
     */
    const shown = withoutTranslatorsNotes({
      text: blockText,
      original: source,
    },);
    // A BLOCK OF NOTHING BUT NOTES IS NOT ASKED ABOUT: the sheet shows the
    // reviewers the block without its notes (`archive-block-review-wire.ts`),
    // which leaves nothing, and no reader sees what stands there. A block that
    // holds no note and shows nothing is reviewed as it always was.
    if ((shown !== blockText) && rendersAsNothing({ text: shown, },)) {
      repairLog.info(`block ${identity} holds only notes between translators and stands as the archive wrote it`,);
      findings.push(`archive block holds only notes between translators and stands: ${identity}`,);
      continue;
    }
    // EACH BLOCK IS REVIEWED BY THE ROSTER ITS HOOK HANDS OVER (ledger X12,
    // 2026-09-28): a dry-out inside the review left every later block on the
    // roster read before it.
    /* oxlint-disable no-await-in-loop -- Reverse-offset block corrections must settle in document order, each block asked of the bench as it stands. */
    /**
     Seating the hook hands this block: a roster read under a hold, or none,
     which keeps the one the review started on.
     */
    const blockSeating: BenchSeating = (beforeBlock === undefined) ? {} : await beforeBlock();
    /**
     Stage-local retained or revised outcome.
     */
    const outcome = await runArchiveBlockReviewStage({
      client,
      modelIds: blockSeating.modelIds ?? modelIds,
      sourceText: source,
      // THE PAGE THE REVISIONS ALREADY APPLIED LEAVE (ledger B80): a footnote
      // is a relation between blocks, so a revision is read in the page it
      // would join. Read in the archive as it came, two revisions each
      // dropping one marker of a note passed alone and shipped the note with
      // nothing referencing it. Later blocks go first, so this block stands
      // in that page at its archive offsets.
      targetText: revisedText,
      blockText,
      blockOffset: block.startOffset,
      priorFindings: [],
      ...((identityContext === undefined) ? {} : { identityContext, }),
      ...((referenceContext === undefined) ? {} : { referenceContext, }),
      signal,
      exchangeTimeoutMs,
      l,
      stageSettings,
    },);
    /* oxlint-enable no-await-in-loop */
    // THE STAGE'S FINDINGS ARE KEPT (ledger B714): the voices it lost to the
    // window, the revisions it withheld, the tally and what each reviewer
    // said. They were dropped here, so a block whose review lost a writer
    // recorded only that it was reviewed, and the artifact could not say that
    // zheermao20's letter block lost Qwen twice. They stand ahead of the line
    // saying what became of the block, under a header naming it, and each
    // stays as the stage worded it so a count by wording still counts voices.
    /**
     What the stage reported for this block.
     */
    const { findings: reviewFindings, } = outcome;
    /**
     How many findings that is.
     */
    const reviewFindingCount = reviewFindings.length;
    if (reviewFindingCount > 0) {
      findings.push(
        `archive block review findings (${String(reviewFindingCount,)}): ${identity}`,
        ...reviewFindings,
      );
      /**
       The findings, each quoted so one over several lines stays on one line
       of the log.
       */
      const quotedFindings = reviewFindings
        .map(function quoted(finding,): string {
          return JSON.stringify(finding,);
        },)
        .join(', ',);
      repairLog.info(
        `block ${identity} review recorded ${String(reviewFindingCount,)} ${
          wordForCount({
            count: reviewFindingCount,
            one: 'finding',
            many: 'findings',
          },)
        }: ${quotedFindings}`,
      );
    }
    if (outcome.kind === 'retained') {
      findings.push(`archive block reviewed and retained: ${identity}`);
      continue;
    }
    if (
      sameWording({
        proposal: outcome.text,
        standing: blockText,
        // READ AS PROSE whatever the block is: this review exists to remove or
        // reword archive text no source supports, and a demotion only keeps
        // the archive's bytes, so re-lining a verse block is kept out too.
        lineStructured: false,
      },)
    ) {
      // A revision that repeats its original wording, in all but layout the
      // page does not show (ledger B26), is a claimed change with no change;
      // the original is kept and the defective claim recorded.
      findings.push(`archive block revision repeated its original wording and was retained: ${identity}`);
      continue;
    }
    /**
     What replaces the block: the revision, or nothing for a removal, with
     each note between translators the block held that it lacks, and those
     notes.
     */
    const {
      text: replacementText,
      kept,
    } = withBlockNotes({
      blockText,
      replacement: outcome.text,
      original: source,
    },);
    if (kept.length > 0) {
      /**
       The notes kept, each quoted so one over several lines stays on one
       line of the log.
       */
      const quotedKept = kept.map(function quoted(note,): string {
        return JSON.stringify(note,);
      },);
      repairLog.info(
        `block ${identity} replaced with ${String(kept.length,)} ${
          wordForCount({
            count: kept.length,
            one: 'note',
            many: 'notes',
          },)
        } between translators kept: ${quotedKept.join(', ',)}`,
      );
      findings.push(keptNotesFinding({
        identity,
        kept,
      },),);
    }
    /**
     Span the replacement covers: the block alone, or the block with one of
     its separators when the review removes it and no note stands in its
     place (class ninety-four).
     */
    const span = (replacementText === '')
      ? removalSpan({
        text: revisedText,
        startOffset: block.startOffset,
        endOffset: block.endOffset,
      },)
      : {
        startOffset: block.startOffset,
        endOffset: block.endOffset,
      };
    revisedText = `${revisedText.slice(
      0,
      span.startOffset,
    )}${replacementText}${revisedText.slice(span.endOffset,)}`;
    findings.push(`archive block reviewed and revised: ${identity}`);
  }
  return {
    targetText: revisedText,
    findings,
  };
}

//endregion Archive block repair
