import { join, } from 'node:path';

import type { Logger, } from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import type { ArchiveOriginalSpan, } from '@monochromatic-dev/translation-repair-corpus-source/ts';
import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import { artifactFileNameOf, } from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import type { SettledArtifact, } from '@monochromatic-dev/translation-repair-artifact/ts';
import { writeFileAtomic, } from '@monochromatic-dev/translation-repair-glossary/ts';
import { wallClockNow, } from '@monochromatic-dev/translation-repair-markup/ts';
import {
  assertFinalNaturalnessComplete,
  defectsLine,
  type DestinationCheck,
  destinationsLine,
  finalSelectionFindings,
  type PublishDefect,
  publishFixedPage,
} from '@monochromatic-dev/translation-repair-page-check/ts';
import { printRunLine, } from '@monochromatic-dev/translation-repair-typography/ts';

//region Pass entry persistence
// Publication precedes artifact persistence so artifact remains done sentinel:
// a pass never skips an entry whose page write did not complete.

/**
 Publishes one settled page, then persists artifact that makes entry skippable.

 @param artifact - settled evidence and chosen wordings

 @param slices - preparation spans used to splice page

 @param archiveText - English page before changes

 @param sourceText - original page used for destination check

 @param entryId - corpus entry being persisted

 @param publishDir - mirrored page root

 @param artifactsDir - settled artifact root

 @param l - entry logger

 @returns Destination comparison from the published page, and the content
 checks it shipped failing

 @example
 ```ts
 const { destinations, defects, } = await persistSettledEntry({ artifact, slices, archiveText, sourceText, entryId, publishDir, artifactsDir, l, },);
 ```
 */
export async function persistSettledEntry(
  {
    artifact,
    slices,
    archiveText,
    sourceText,
    entryId,
    publishDir,
    artifactsDir,
    l,
    archiveOriginalSpans,
  }: ForeignBorrowed<{
    readonly artifact: SettledArtifact;
    readonly slices: readonly ChunkPair[];
    readonly archiveText: string;
    readonly sourceText: string;
    readonly entryId: string;
    readonly publishDir: string;
    readonly artifactsDir: string;
    readonly l: Logger;
    readonly archiveOriginalSpans?: readonly ArchiveOriginalSpan[];
  }>,
): Promise<{
  readonly destinations: DestinationCheck;
  readonly defects: readonly PublishDefect[];
}> {
  // FIRST MUTATION SITS BELOW THIS LINE. A contest decline is not approval;
  // it ships as recorded evidence the reading judges, never as a refusal.
  for (const finding of finalSelectionFindings({ artifact, },))
    l.warn(`entry ${entryId}: ${finding}`,);
  assertFinalNaturalnessComplete({ artifact, },);

  /**
   Page write and its source-destination comparison.
   */
  const published = await publishFixedPage({
    artifact,
    slices,
    archiveText,
    sourceText,
    entryId,
    publishDir,
    l,
    ...((archiveOriginalSpans === undefined) ? {} : { archiveOriginalSpans, }),
  },);
  await writeFileAtomic({
    path: join(
      artifactsDir,
      artifactFileNameOf({ entryId, },),
    ),
    text: `${JSON.stringify(
      artifact,
      undefined,
      2,
    )}\n`,
  },);
  return {
    destinations: published.destinations,
    defects: published.defects,
  };
}

/**
 Prints the lines a pass writes to stdout for one settled entry: its tally,
 its destination counts, and the content checks its page shipped failing.

 COUNTS ONLY ON STDOUT; the addresses themselves are in the run log. Split out
 of `pass-entry.ts` at its line budget when the defects line joined them.

 @param entryId - entry settled

 @param tally - its tally line

 @param published - what persisting it reported

 @param attempt - number of the entry's attempt that settled, as the pass's attempts file holds it, which the
 destinations and defects lines carry as the tally line does

 @param wallClock - clock the destinations and defects lines' time is read on; the wall clock unless a test fixes it

 @example
 ```ts
 printSettledLines({ entryId, tally, published: await persistSettledEntry({ ... },), attempt, },);
 ```
 */
export function printSettledLines(
  {
    entryId,
    tally,
    published,
    attempt,
    wallClock = wallClockNow,
  }: {
    readonly entryId: string;
    readonly tally: string;
    readonly attempt: number;
    readonly wallClock?: () => Date;
    readonly published: Readonly<{
      destinations: DestinationCheck;
      defects: readonly PublishDefect[];
    }>;
  },
): void {
  printRunLine({ line: tally, },);
  printRunLine({
    line: destinationsLine({
      entryId,
      destinations: published.destinations,
      attempt,
      wallClock,
    },),
  },);
  /**
   Defects line, empty for a clean page.
   */
  const defects = defectsLine({
    entryId,
    defects: published.defects,
    attempt,
    wallClock,
  },);
  // A PAGE THAT SHIPPED WITH DEFECTS says so beside its tally, for a grep.
  if (defects !== '')
    printRunLine({ line: defects, },);
}

//endregion Pass entry persistence
