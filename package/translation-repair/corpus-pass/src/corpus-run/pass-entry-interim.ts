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
  interimPageDigest,
  type PipelineDigest,
  removeInterimEntry,
  writeInterimEntry,
} from '@monochromatic-dev/translation-repair-glossary/ts';
import type { ArtifactLaneSelection, } from '@monochromatic-dev/translation-repair-lane-contest/ts';
import {
  lifecycleStamp,
  wallClockNow,
} from '@monochromatic-dev/translation-repair-markup/ts';
import { publishInterimPage, } from '@monochromatic-dev/translation-repair-page-check/ts';
import { endsTheRun, } from '@monochromatic-dev/translation-repair-stage/ts';
import { printRunLine, } from '@monochromatic-dev/translation-repair-typography/ts';

//region Pass entry interim
// THE PART OF AN ENTRY'S PIPELINE THAT SURVIVES ITS STOP. `runEntryPipeline` keeps the slicing, the lanes, the contest
// and the consolidation as locals of its `try`, and its `catch` cannot see them; this holder is what each stage hands the
// entry as it finishes, so a stop in the middle of the page still knows what was settled. Used only when every provider
// goes dry (the owner's answer of 2026-10-29, "Ship interim, resumable"): the entry ships what it has and stays open.

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
   Both lanes' ledgers and the comparison they derive, once both lanes returned.
   */
  projected?: ProjectedLanes;

  /**
   What the contest decided per slice, once it returned.
   */
  contestSlices?: Extract<ArtifactLaneSelection, { readonly kind: 'contested'; }>['slices'];

  /**
   Consolidation records, in the order the slices finished.
   */
  readonly consolidated: ArtifactConsolidateSlice[];
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
 Ships the interim page of an entry that stopped because every provider went dry, and records it as interim.

 NEVER THROWS, because it runs in the entry's `catch` beside the tally of the stop, which must still be printed and the
 stop still end the run: a page that cannot be written is named in the log and the stop stands as it was, with the
 archive shipping for the entry.

 @param error - what ended the entry; nothing happens unless it is the every-provider-dry refusal

 @param progress - what the entry had finished

 @param entryId - entry stopped

 @param sourceText - whole original page, read for the destinations the page links to

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
 await shipInterimOnDryStop({ error, progress, entryId, sourceText, publishDir, interimDir, tip, pipelineDigest, corpusSha, attempt, l, },);
 ```
 */
export async function shipInterimOnDryStop(
  {
    error,
    progress,
    entryId,
    sourceText,
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
  if (!endsTheRun({ error, },))
    return;
  /**
   The slicing and the archive it addresses, absent where the stop came before either existed.
   */
  const {
    prepared,
    archiveText,
    projected,
    contestSlices,
    consolidated,
  } = progress;
  if ((prepared === undefined) || (archiveText === undefined)) {
    l.warn(`entry ${entryId}: every provider went dry before the entry was sliced, so no interim page is shipped`,);
    return;
  }
  try {
    /**
     What the entry had settled, in the shape the page is composed from.
     */
    const stop: Parameters<typeof publishInterimPage>[0]['stop'] = {
      consolidateSlices: [...consolidated,],
      ...((projected === undefined) ? {} : { comparison: projected.comparison, }),
      ...((contestSlices === undefined)
        ? {}
        : {
          laneSelection: {
            kind: 'contested',
            slices: contestSlices,
          },
        }),
    };
    /**
     The page as written and what each of its slices carries.
     */
    const shipped = await publishInterimPage({
      stop,
      slices: prepared.slices,
      archiveText,
      sourceText,
      entryId,
      publishDir,
      l,
      ...((prepared.archiveOriginalSpans === undefined)
        ? {}
        : { archiveOriginalSpans: prepared.archiveOriginalSpans, }),
    },);
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
        reason: 'dry-stop',
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
        + `source=${String(shipped.sourceSlices,)} reason=dry-stop`,
    },);
  }
  catch (failure) {
    l.error(
      `entry ${entryId}: every provider went dry and the interim page could not be shipped, so the archive stands `
        + `(${refusalText({ error: failure, },)})`,
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
