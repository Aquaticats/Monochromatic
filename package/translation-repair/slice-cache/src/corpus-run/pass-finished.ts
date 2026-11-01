import { declinedEntryIds, } from '@monochromatic-dev/translation-repair-glossary/ts';
import { artifactBackedIds, } from './pass-settled.ts';

//region Pass finished entries
// What a pass counts as done when it starts, and how many entries it reports
// processed when it ends, READ THE SAME WAY BOTH TIMES (ledger A9). The pass
// skipped every entry with an artifact or a decline record, then reported
// artifacts after the run less the size of that set: every decline already on
// disk was subtracted from the new artifacts, and a decline written this run
// never counted. A resume into a runs dir holding one decline and finishing
// nothing printed `processed=-1`.

/**
 Entry ids carrying an artifact or a decline record.

 @param artifactsDir - directory of settled artifacts

 @param declinedDir - directory of decline records

 @returns Every finished entry id

 @example
 ```ts
 const done = await finishedEntryIds({ artifactsDir, declinedDir, },);
 ```
 */
export async function finishedEntryIds(
  {
    artifactsDir,
    declinedDir,
  }: {
    readonly artifactsDir: string;
    readonly declinedDir: string;
  },
): Promise<ReadonlySet<string>> {
  return new Set([
    ...(await artifactBackedIds({ artifactsDir, },)),
    ...(await declinedEntryIds({ declinedDir, },)),
  ],);
}

/**
 Entries a run processed, for its DONE line.

 @param before - finished ids when the run started

 @param artifactsDir - directory of settled artifacts

 @param declinedDir - directory of decline records

 @returns Entries finished after the run and not before it, settled or
 declined

 @example
 ```ts
 const processed = await entriesFinishedThisRun({ before: done, artifactsDir, declinedDir, },);
 ```
 */
export async function entriesFinishedThisRun(
  {
    before,
    artifactsDir,
    declinedDir,
  }: {
    readonly before: ReadonlySet<string>;
    readonly artifactsDir: string;
    readonly declinedDir: string;
  },
): Promise<number> {
  /**
   Finished ids now, read as `before` was.
   */
  const after = await finishedEntryIds({
    artifactsDir,
    declinedDir,
  },);
  /**
   Ids the run finished.
   */
  const fresh = [...after,].filter(function isNew(entryId,): boolean {
    return !before.has(entryId,);
  },);
  return fresh.length;
}

//endregion Pass finished entries
