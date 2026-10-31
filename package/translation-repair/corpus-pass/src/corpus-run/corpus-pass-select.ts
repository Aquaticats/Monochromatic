import {
  type CorpusPin,
  listCorpusPeople,
} from '@monochromatic-dev/translation-repair-corpus-source/ts';
import type { AttemptMap, } from '@monochromatic-dev/translation-repair-glossary/ts';
import { wallClockNow, } from '@monochromatic-dev/translation-repair-markup/ts';
import {
  askedAmong,
  type CommandLineOf,
  readOnlyIds,
} from '@monochromatic-dev/translation-repair-run/ts';
import { printRunLine, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  passIncompleteLine,
  passOnlyLines,
  type PassPairs,
} from './corpus-pass-lines.ts';
import { orderPendingEntries, } from './corpus-pass-order.ts';
import {
  collectEligiblePairs,
  type CorpusPair,
  listResumableEntries,
} from '@monochromatic-dev/translation-repair-slice-cache/ts';

//region Corpus pass select
// Which entries a pass runs and in what order, with what it could not pair said
// on the way.

/**
 Chooses the entries a pass runs: every complete unsettled pair at the pin, or
 only those the command line names, ordered to resume cached progress first.

 @param line - pass's command line, which may restrict the entries

 @param pin - clone and commit the corpus is read at

 @param done - entries already carrying an artifact or a decline record

 @param attempts - attempt counts from earlier runs

 @param sliceCacheDir - root of the per-entry slice caches, where cached progress is found

 @param wallClock - clock the time of each page-absent line is read on; the wall clock unless a test fixes it

 @returns Pairs to run, in run order, and the complete pairs the walk
 found, which the pass's closing line counts its artifacts and declines
 over; under `--only` the walk read the named entries alone, and says so

 @throws {@link StatedRefusalError} when `--only` names an entry the corpus does not hold or names none

 @example
 ```ts
 const { pending, pairs, } = await selectPendingEntries({ line, pin, done, attempts, sliceCacheDir, },);
 ```
 */
export async function selectPendingEntries(
  {
    line,
    pin,
    done,
    attempts,
    sliceCacheDir,
    wallClock = wallClockNow,
  }: {
    readonly line: CommandLineOf<'corpus-pass'>;
    readonly pin: CorpusPin;
    readonly done: ReadonlySet<string>;
    readonly attempts: AttemptMap;
    readonly sliceCacheDir: string;
    readonly wallClock?: () => Date;
  },
): Promise<{
  readonly pending: readonly CorpusPair[];
  readonly pairs: PassPairs;
}> {
  /**
   Every person id at the pinned commit.
   */
  const people = await listCorpusPeople({ pin, },);

  /**
   Entry ids this invocation is restricted to, empty when unrestricted.
   */
  const onlyIds = readOnlyIds({ line, },);

  for (const onlyLine of passOnlyLines({ onlyIds, },))
    printRunLine({ line: onlyLine, },);

  /**
   Complete unsettled pairs, already-settled sizes (ordering needs these:
   ranking runs over the REMAINING entries, so without knowing what each band
   already settled every run would restart each band at rank zero), and the
   entries missing a side at the pin.

   ONLY A MISSING OBJECT DROPS OUT, and it is printed. Any other read
   failure propagates: until 2026-08-26 every read failure read as the
   expected missing side, so a clone that had gone away shrank the corpus
   to nothing without a line saying so.
   */
  const {
    eligible,
    settled,
    incomplete,
    paired,
  } = await collectEligiblePairs({
    ids: askedAmong({
      asked: [...onlyIds,],
      known: people,
      source: '--only',
      within: 'the corpus at the pin',
    },),
    done,
    pin,
  },);
  for (const gap of incomplete)
    printRunLine({
      line: passIncompleteLine({
        gap,
        wallClock,
      },),
    },);

  /**
   Ids with cached slices from an earlier aborted run. These resume first so
   an in-flight large document finishes before a fresh entry starts, rather
   than every large entry taking one partial attempt while none settles.

   NO PROGRESS GUARANTEE IS CLAIMED HERE, and one used to be: this said a
   cap-abort always completes at least one new slice, which is false. An abort
   can land before the first persistence, and the slices a lane deliberately
   leaves uncached, the unfilled and the unheard, produce no cache entry
   however long they took. What actually bounds it is that a stuck entry
   surfaces: `repairChunk` degrades and persists rather than throwing on a
   lost quorum, the translate lane's refusal counter bounds its retries within
   a slice, and an entry that keeps failing writes a repeated same-entry ERROR
   line across runs, which is read by inspection.
   */
  const resumableIds = await listResumableEntries({ dir: sliceCacheDir, },);

  return {
    // In run order (`corpus-pass-order.ts`).
    pending: orderPendingEntries({
      eligible,
      settled,
      resumableIds,
      attempts,
    },),
    // THE WALK'S OWN PAIRS, the pending ones and those finished before whose
    // two pages the pin holds, over every entry or the named ones alone.
    pairs: {
      walked: (onlyIds.size === 0) ? 'every-entry' : 'named-entries',
      ids: new Set(paired,),
    },
  };
}

//endregion Corpus pass select
