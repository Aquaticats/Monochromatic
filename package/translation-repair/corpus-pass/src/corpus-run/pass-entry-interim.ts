import {
  dirname,
  join,
} from 'node:path';

import type { Logger, } from '@monochromatic-dev/logger-core/ts';

import type {
  ArtifactConsolidateSlice,
  ProjectedLanes,
} from '@monochromatic-dev/translation-repair-artifact/ts';
import type { PreparedDocumentPair, } from '@monochromatic-dev/translation-repair-corpus-source/ts';
import { refusalText, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  INTERIM_DIR,
  type InterimReason,
  interimPageDigest,
  type PipelineDigest,
  removeInterimEntry,
  writeInterimEntry,
} from '@monochromatic-dev/translation-repair-glossary/ts';
import type { ArtifactLaneSelection, } from '@monochromatic-dev/translation-repair-lane-contest/ts';
import {
  errorClassOf,
  lifecycleStamp,
  wallClockNow,
} from '@monochromatic-dev/translation-repair-markup/ts';
import { endsTheRun, } from '@monochromatic-dev/translation-repair-stage/ts';
import { TranslationRepairInterruptedError, } from '@monochromatic-dev/translation-repair-translate-stage/ts';
import { printRunLine, } from '@monochromatic-dev/translation-repair-typography/ts';
import { shipPageOfStop, } from './pass-entry-interim-page.ts';
import { pageStandingOf, } from './pass-entry-page-standing.ts';

//region Pass entry interim
// THE PART OF AN ENTRY'S PIPELINE THAT SURVIVES ITS STOP. `runEntryPipeline` keeps the slicing, the lanes, the contest
// and the consolidation as locals of its `try`, and its `catch` cannot see them; this holder is what each stage hands the
// entry as it finishes, so a stop in the middle of the page still knows what was settled. Used when every provider goes
// dry (the owner's answer of 2026-10-29, "Ship interim, resumable"), when a stage heard no voice at all (B1183), and
// since B1220 for every other error too ("no matter what, we must ship"): the entry ships what it has and stays open.

/**
 What an entry had finished when it stopped, filled in stage order.

 @example
 ```ts
 const progress: EntryProgress = startEntryProgress();
 ```
 */
export type EntryProgress = {
  /**
   The slicing both lanes run over, once preparation, the admission and the fold of carried passages are done.
   */
  prepared?: PreparedDocumentPair;

  /**
   Archive English the slicing addresses, once the slicing exists.
   */
  archiveText?: string;

  /**
   The comparison both lanes' ledgers derive, once both lanes returned; the only part of the projection a page reads.
   */
  projected?: Pick<ProjectedLanes, 'comparison'>;

  /**
   What the contest decided per slice, once it returned.
   */
  contestSlices?: Extract<ArtifactLaneSelection, { readonly kind: 'contested'; }>['slices'];

  /**
   Consolidation records, in the order the slices finished.
   */
  readonly consolidated: ArtifactConsolidateSlice[];

  /**
   Set when the settled page was written, after which a stop ships no interim page: it would replace the better page
   with the worse and leave an interim record beside the entry's page (B1220).
   */
  settledPageWritten?: true;

  /**
   Set once the fold of carried passages has run on the slicing in `prepared`. Before it, the admission of the source-only
   passages has not been read, so such a passage ships nothing: the archive may already render it (B1244).
   */
  folded?: true;
};

/**
 Starts an empty holder for one entry's progress.

 @returns Holder that has seen no stage finish

 @example
 ```ts
 const progress = startEntryProgress();
 ```
 */
export function startEntryProgress(): EntryProgress {
  return { consolidated: [], };
}

/**
 Directory an entry's interim record goes in when none is named, beside the artifacts directory as the pass's runs
 layout names it.

 @param artifactsDir - directory of settled artifacts

 @returns Interim directory beside it

 @example
 ```ts
 const interimDir = interimDirBeside({ artifactsDir, },);
 ```
 */
export function interimDirBeside({ artifactsDir, }: { readonly artifactsDir: string; },): string {
  return join(
    dirname(artifactsDir,),
    INTERIM_DIR,
  );
}

/**
 Why an error that ended an entry earns an interim page, which is every error.

 THREE REASONS, ONE RULE. The every-provider-dry refusal ends the run (the owner's answer of 2026-10-29, "Ship interim,
 resumable"). A stage that heard no voice at all (`TranslationRepairInterruptedError` with the reason
 `provider-unavailable`) is the same outage seen from one stage (B1183). Every other error, a guard that threw, a store
 that failed, a stage that raised, ends the entry's work the same way, and "no matter what, we must ship" leaves no
 class that ships nothing (B1220). Its record names the class of the error.

 @param error - what ended the entry

 @returns Reason the interim record carries

 @example
 ```ts
 const reason = interimReasonOf({ error: dryRefusal, },); // => 'dry-stop'
 ```
 */
function interimReasonOf({ error, }: { readonly error: unknown; },): InterimReason {
  if (endsTheRun({ error, },))
    return 'dry-stop';
  if ((error instanceof TranslationRepairInterruptedError) && (error.reason === 'provider-unavailable'))
    return 'provider-unavailable';
  return 'entry-error';
}

/**
 Ships the interim page of an entry that stopped, whatever stopped it, and records it as interim.

 NEVER THROWS, because it runs in the entry's `catch` beside the tally of the stop, which must still be printed and the
 stop still end the run where it ends it: a page that cannot be written is named in the log with the class of the error
 and the stop stands as it was.

 @param error - what ended the entry

 @param progress - what the entry had finished

 @param entryId - entry stopped

 @param sourceText - whole original page, read for the destinations the page links to

 @param targetText - whole archive English as the entry read it, which a stop before the slicing ships as the page

 @param publishDir - root of the mirrored tree

 @param interimDir - directory the record is written into

 @param tip - repository head the pass runs at

 @param pipelineDigest - built pipeline that ships the page

 @param corpusSha - corpus commit the entry was read at

 @param attempt - number of the entry's attempt that stopped, which the entry's lines carry

 @param wallClock - clock the line's time is read on; the wall clock unless a test fixes it

 @param l - entry logger

 @example
 ```ts
 await shipInterimOnStop({ error, progress, entryId, sourceText, targetText, publishDir, interimDir, tip, pipelineDigest, corpusSha, attempt, l, },);
 ```
 */
export async function shipInterimOnStop(
  {
    error,
    progress,
    entryId,
    sourceText,
    targetText,
    publishDir,
    interimDir,
    tip,
    pipelineDigest,
    corpusSha,
    attempt,
    wallClock = wallClockNow,
    l,
  }: {
    readonly error: unknown;
    readonly progress: EntryProgress;
    readonly entryId: string;
    readonly sourceText: string;
    readonly targetText: string;
    readonly publishDir: string;
    readonly interimDir: string;
    readonly tip: string;
    readonly pipelineDigest: PipelineDigest;
    readonly corpusSha: string;
    readonly attempt: number;
    readonly wallClock?: () => Date;
    readonly l: Logger;
  },
): Promise<void> {
  /**
   Why this stop is interim.
   */
  const reason = interimReasonOf({ error, },);
  /**
   Class of the error, which every line of this stop carries.
   */
  const errorClass = errorClassOf({ error, },);
  /**
   What the stop was, in the words the log gives it.
   */
  const stopped = (reason === 'dry-stop')
    ? 'every provider went dry'
    : ((reason === 'provider-unavailable') ? 'a stage heard no voice' : `an error ended the entry (${errorClass})`);
  if (progress.settledPageWritten === true) {
    l.warn(
      `entry ${entryId}: ${stopped} after its settled page was written, so that page stands and no interim page is `
        + `shipped (class=${errorClass})`,
    );
    return;
  }
  if (progress.prepared === undefined) {
    l.warn(
      `entry ${entryId}: ${stopped} before the entry was sliced, so the archive is shipped as the interim page `
        + `(class=${errorClass})`,
    );
  }
  try {
    /**
     The page that stands where this one would go, which a better page keeps (B1242, B1243).
     */
    const standing = await pageStandingOf({
      publishDir,
      interimDir,
      entryId,
      l,
    },);
    /**
     The page as written and what each of its slices carries.
     */
    const shipped = await shipPageOfStop({
      progress,
      sourceText,
      targetText,
      entryId,
      publishDir,
      standing,
      l,
    },);
    if (!shipped.written) {
      l.warn(
        (standing.kind === 'interim')
          ? `entry ${entryId}: ${stopped}, and an interim page with more settled slices (${
            String(standing.settledSlices,)
          }) stands than this attempt settled (${
            String(shipped.settledSlices,)
          }), so that page and its record stay (class=${errorClass})`
          : `entry ${entryId}: ${stopped}, and the settled page of an earlier attempt stands (a page no interim record `
            + `describes, whose artifact is not written), so that page stays and no interim record is written `
            + `(class=${errorClass})`,
      );
      return;
    }
    // THE RECORD AFTER THE PAGE it describes, so it never names a page that was not written.
    await writeInterimEntry({
      interimDir,
      record: {
        id: entryId,
        tip,
        pipelineDigest,
        corpusSha,
        timestamp: wallClock()
          .toISOString(),
        reason,
        errorClass,
        settledSlices: shipped.settledSlices,
        archiveSlices: shipped.archiveSlices,
        sourceSlices: shipped.sourceSlices,
        pageSha256: interimPageDigest({ pageText: shipped.pageText, },),
      },
    },);
    printRunLine({
      line: `INTERIM ${entryId} ${
        lifecycleStamp({
          attempt,
          wallClock,
        },)
      } settled=${String(shipped.settledSlices,)} archive=${String(shipped.archiveSlices,)} `
        + `source=${String(shipped.sourceSlices,)} reason=${reason} class=${errorClass}`,
    },);
  }
  catch (failure) {
    l.error(
      `entry ${entryId}: ${stopped} and the interim page could not be shipped, so the archive stands `
        + `(class=${errorClass}; ${refusalText({ error: failure, },)})`,
    );
  }
}

/**
 Removes the interim record of an entry that has just settled, whose artifact now stands.

 NEVER THROWS, because the artifact is already on disk: a failure here raised into the entry's `catch` would print
 `status=ERROR` for an entry every later reader finds settled. A record left behind is named in the log, and the settled
 entry is skipped by every later pass whatever the record says.

 @param interimDir - directory of interim records

 @param entryId - entry settled

 @param l - entry logger

 @example
 ```ts
 await retireInterimRecord({ interimDir, entryId, l, },);
 ```
 */
export async function retireInterimRecord(
  {
    interimDir,
    entryId,
    l,
  }: {
    readonly interimDir: string;
    readonly entryId: string;
    readonly l: Logger;
  },
): Promise<void> {
  try {
    await removeInterimEntry({
      interimDir,
      entryId,
    },);
  }
  catch (failure) {
    l.warn(`entry ${entryId}: settled, and its interim record could not be removed (${refusalText({ error: failure, },)})`,);
  }
}

//endregion Pass entry interim
