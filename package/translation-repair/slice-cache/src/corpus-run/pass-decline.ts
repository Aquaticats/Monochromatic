import { allInInputOrder, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  lifecycleStamp,
  monotonicMs,
  textsInCodePointOrder,
  wallClockNow,
} from '@monochromatic-dev/translation-repair-markup/ts';
import { rm, } from 'node:fs/promises';

import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';

import {
  archiveNoteReadingsOf,
  type ArchiveOriginalReading,
  archiveOriginalReadingOf,
} from '@monochromatic-dev/translation-repair-corpus-source/ts';
import {
  printRunLine,
  rethrowUnlessMissingPath,
  runLogger,
  sliceTagged,
} from '@monochromatic-dev/translation-repair-typography/ts';
import { parseDocument, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  declinedEntryIds,
  type PipelineDigest,
  writeDeclinedEntry,
} from '@monochromatic-dev/translation-repair-glossary/ts';
import type {
  CorpusPair,
  EntryOutcome,
} from './pass-entry-contract.ts';
import { fixedPagePath, } from '@monochromatic-dev/translation-repair-page-check/ts';

//region Pass decline
// The one way an entry leaves the pass without an artifact and without a
// failure: the archive's note says the whole page is the author's own English
// (the owner's rule of 2026-09-08, `archive-original-note.ts`), so nothing is
// bought and the archive stands. Split out of `pass-entry.ts` at its line
// budget.

/**
 Reads what an entry's archive notes say about whose text the page is, and
 logs every note with its reading, warning on one that speaks of an English
 original in a wording no mark knows, which seals nothing (ledger E12).

 @param entry - corpus pair, text already read

 @param l - entry logger, which a test hands a capturing one so the warning
 an unmarked claim must raise can be read

 @returns Reading, off the archive as inherited

 @example
 ```ts
 const reading = entryArchiveOriginalOf({ entry, l, },);
 ```
 */
export function entryArchiveOriginalOf(
  {
    entry,
    l,
  }: {
    readonly entry: CorpusPair;
    readonly l: Logger;
  },
): ArchiveOriginalReading {
  /**
   Entry logger, tagged with this function's name.
   */
  const el = tagged({
    l,
    tag: entryArchiveOriginalOf.name,
  },);
  /**
   The archive as inherited.
   */
  const document = parseDocument({ text: entry.targetText, },);
  for (const {
    note,
    reading,
  } of archiveNoteReadingsOf({ document, },)) {
    if (reading === 'unmarked-original-claim') {
      el.warn(
        `ARCHIVE NOTE entry=${entry.id} reading=${reading}: the note speaks of an English original in a `
          + `wording no mark reads, so nothing is sealed; add its mark if it seals (${note})`,
      );
    } else
      el.info(`ARCHIVE NOTE entry=${entry.id} reading=${reading}: ${note}`,);
  }
  return archiveOriginalReadingOf({ document, },);
}

/**
 Removes a page standing for an entry the pipeline declines, which only a
 crash between an earlier page write and its artifact write leaves (ledger
 A16c).

 @param publishDir - root of the mirrored tree

 @param entryId - entry declined

 @returns Whether a page stood there

 @throws Whatever the removal raised other than ENOENT, since a page left
 standing would ship where the archive must

 @example
 ```ts
 const removed = await removeLeftoverPage({ publishDir, entryId, },);
 ```
 */
async function removeLeftoverPage(
  {
    publishDir,
    entryId,
  }: {
    readonly publishDir: string;
    readonly entryId: string;
  },
): Promise<boolean> {
  try {
    await rm(fixedPagePath({
      publishDir,
      entryId,
    },),);
    return true;
  }
  catch (error) {
    rethrowUnlessMissingPath({ error, },);
    return false;
  }
}

/**
 Records that the pipeline declined an entry, and says so on the run log and
 the tally.

 RECORDED, NEVER SILENT: the record is what the next pass skips on, and the
 archive page stands as the output, untouched. A PAGE AN EARLIER CRASH LEFT
 GOES FIRST, before the record: a crash between the two then leaves the page
 gone and the entry still pending, never a recorded decline with a page
 standing that no later pass would revisit.

 @param entry - entry declined

 @param declinedDir - directory the record is written into

 @param publishDir - root of the mirrored tree, where no page may stand for it

 @param tip - repository head the pass runs at

 @param pipelineDigest - built pipeline that declined it

 @param corpusSha - corpus commit the pass read the entry at, which the
 record names

 @param note - archive's note that decided it

 @param startedAt - `monotonicMs` reading when the entry started, for the tally's duration

 @param attempt - number of the entry's attempt the decline belongs to, as the pass's attempts file holds it, which the
 tally line carries beside its clock time (ledger B960)

 @param wallClock - clock the tally line's time is read on; the wall clock unless a test fixes it

 @returns Declined outcome

 @throws `LifecycleAttemptError` when the attempt is not a whole number from 1, after the decline record is
 written and before the tally line is printed

 @example
 ```ts
 return recordEntryDecline({ entry, declinedDir, publishDir, tip, pipelineDigest, corpusSha: pin.commitSha, note, startedAt: t0, attempt, },);
 ```
 */
export async function recordEntryDecline(
  {
    entry,
    declinedDir,
    publishDir,
    tip,
    pipelineDigest,
    corpusSha,
    note,
    startedAt,
    attempt,
    wallClock = wallClockNow,
  }: {
    readonly entry: CorpusPair;
    readonly declinedDir: string;
    readonly publishDir: string;
    readonly tip: string;
    readonly pipelineDigest: PipelineDigest;
    readonly corpusSha: string;
    readonly note: string;
    readonly startedAt: number;
    readonly attempt: number;
    readonly wallClock?: () => Date;
  },
): Promise<EntryOutcome> {
  /**
   Entry logger, which names the attempt the decline is recorded in (ledger B963).
   */
  const el = sliceTagged({ l: tagged({ tag: entry.id, l: runLogger, },), },);
  if (await removeLeftoverPage({
    publishDir,
    entryId: entry.id,
  },))
    el.warn(`entry ${entry.id}: removed a page an earlier crash left, since the archive's note says the archive ships`,);
  await writeDeclinedEntry({
    declinedDir,
    record: {
      id: entry.id,
      tip,
      pipelineDigest,
      corpusSha,
      timestamp: new Date().toISOString(),
      reason: 'archive-original',
      note,
    },
  },);
  el.info(
    `ARCHIVE ORIGINAL entry=${entry.id}: the archive's note says the whole page is the author's own `
      + `English (${note}), so the pipeline declines to repair it and the archive stands`,
  );
  printRunLine({
    line: `TALLY ${entry.id} status=DECLINED ${
      lifecycleStamp({
        attempt,
        wallClock,
      },)
    } reason=archive-original ms=${String(monotonicMs() - startedAt,)}`,
  },);
  return { kind: 'declined', };
}

/**
 Removes the page standing for every entry a runs directory has declined,
 which a decline recorded before its page removal existed can have left
 (ledger A16c); the pass runs it before any entry, since a declined entry is
 never visited again.

 @param declinedDir - directory of decline records

 @param publishDir - root of the mirrored tree

 @returns Entries whose page was removed, sorted

 @example
 ```ts
 const removed = await removeDeclinedPages({ declinedDir, publishDir, },);
 ```
 */
export async function removeDeclinedPages(
  {
    declinedDir,
    publishDir,
  }: {
    readonly declinedDir: string;
    readonly publishDir: string;
  },
): Promise<readonly string[]> {
  /**
   Entries declined here, each with whether a page stood for it.
   */
  const outcomes = await allInInputOrder({
    members: [...await declinedEntryIds({ declinedDir, },),]
      .map(async function removeFor(entryId,): Promise<{
        readonly entryId: string;
        readonly removed: boolean;
      }> {
        return {
          entryId,
          removed: await removeLeftoverPage({
            publishDir,
            entryId,
          },),
        };
      },),
  },);
  /**
   Entries whose page was removed.
   */
  const removed = textsInCodePointOrder({ texts: outcomes
    .filter(function wasRemoved({ removed: gone, },): boolean {
      return gone;
    },)
    .map(function idOf({ entryId, },): string {
      return entryId;
    },), },);
  for (const entryId of removed) {
    sliceTagged({ l: tagged({ tag: entryId, l: runLogger, },), },)
      .warn(`entry ${entryId}: removed a page standing for a declined entry, since the archive's note says the archive ships`,);
  }
  return removed;
}

//endregion Pass decline
