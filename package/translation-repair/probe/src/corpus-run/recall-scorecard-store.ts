import { mkdir, } from 'node:fs/promises';
import { join, } from 'node:path';

import { writeFileAtomic, } from '@monochromatic-dev/translation-repair-glossary/ts';
import { stampFor, } from './probe-store.ts';

//region Recall scorecard store
// Where a recall benchmark's scorecard is kept.
//
// STAMPED AND ATOMIC, for the reason `probe-store.ts` gives for probe runs. The
// benchmark takes hours and is rerun on purpose to see whether a change moved
// the detection rate, and until this module it wrote `recall-scorecard.json`
// in place with a plain `writeFile`: every rerun destroyed the run it was
// bought to be compared against, and a crash mid-write left a truncated file
// under the only name anything looked for.

/**
 Subdirectory of the runs directory the scorecards are kept under.
 */
export const RECALL_SCORECARD_DIR = 'recall-scorecard';

/**
 Leading characters of the tip that name the file.
 */
const TIP_IN_NAME = 8;

/**
 One benchmark run's scorecard as it is kept.

 @example
 ```ts
 const record: RecallScorecardRecord = { startedAt, finishedAt, tip, corpusSha, callConfig, entriesPerBand, seedsPerEntry, scorecard, records, };
 ```
 */
export type RecallScorecardRecord = {
  /**
   When the benchmark began, which names the file.
   */
  readonly startedAt: string;

  /**
   When the scorecard was written.
   */
  readonly finishedAt: string;

  /**
   Repository tip the benchmark ran from.
   */
  readonly tip: string;

  /**
   Corpus commit the entries were read at.
   */
  readonly corpusSha: string;

  /**
   Call configuration in force.
   */
  readonly callConfig: Readonly<Record<string, unknown>>;

  /**
   Entries drawn per size band.
   */
  readonly entriesPerBand: number;

  /**
   Omissions planted per entry.
   */
  readonly seedsPerEntry: number;

  /**
   Aggregate scorecard.
   */
  readonly scorecard: Readonly<Record<string, unknown>>;

  /**
   Graded attempts behind it.
   */
  readonly records: readonly unknown[];
};

/**
 Writes a scorecard under a name no other run can claim.

 @param runsDir - runs directory the scorecard subdirectory lives under

 @param record - scorecard and the identity it was measured under

 @returns Path written

 @example
 ```ts
 const keptAt = await persistRecallScorecard({ runsDir, record, },);
 ```
 */
export async function persistRecallScorecard(
  {
    runsDir,
    record,
  }: {
    readonly runsDir: string;
    readonly record: RecallScorecardRecord;
  },
): Promise<string> {
  /**
   Directory the scorecards accumulate in.
   */
  const dir = join(
    runsDir,
    RECALL_SCORECARD_DIR,
  );
  await mkdir(
    dir,
    { recursive: true, },
  );

  /**
   Stamp and tip, so two runs from different builds or different hours
   never share a name.
   */
  const path = join(
    dir,
    `${stampFor({ startedAt: record.startedAt, },)}-${
      record.tip
        .slice(
          0,
          TIP_IN_NAME,
        )
    }.json`,
  );
  await writeFileAtomic({
    path,
    text: `${JSON.stringify(
      record,
      undefined,
      2,
    )}\n`,
  },);
  return path;
}

//endregion Recall scorecard store
