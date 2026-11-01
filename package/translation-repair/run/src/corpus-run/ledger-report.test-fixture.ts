import {
  mkdir,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { LEDGER_DIR, } from '@monochromatic-dev/translation-repair-house-rule';
import type { ReadRound, } from '../../dist/final/node/index.mjs';

//region Ledger report fixture
// Contests a cat-themed ledger holds, and the writer that puts them in a runs
// directory the way the recorder does: one JSON file per contest under the
// ledger directory, named by a stamp and a zero-padded ordinal.

/**
 A contest two seats each wrote a candidate for, which a third judged and
 the first judge's own seat voted for itself in.

 `tabby-1` wrote the first candidate and was chosen by the disinterested
 judge `siamese-3`; `calico-2` wrote the second; `bengal-4` named nothing.
 */
export const NAP_CONTEST: ReadRound = {
  task: 'render the nap passage',
  at: '2026-08-25T01:00:00.000Z',
  candidates: [
    {
      index: 1,
      producers: ['tabby-1',],
      rendered: 'The cat naps in the sun.',
    },
    {
      index: 2,
      producers: [
        'calico-2',
      ],
      rendered: 'A cat sleeps where the sun is.',
    },
  ],
  ballots: [
    {
      modelId: 'siamese-3',
      best: 1,
      reason: 'keeps the sun and the nap',
    },
    {
      modelId: 'tabby-1',
      best: 1,
      reason: 'mine reads best',
    },
    {
      modelId: 'bengal-4',
      best: 0,
      reason: 'could not choose',
    },
  ],
  selectedIndex: 1,
};

/**
 A contest whose one candidate two seats wrote jointly, where the panel
 declined and one judge named a position the slate did not have.
 */
export const PURR_CONTEST: ReadRound = {
  task: 'render the purr passage',
  at: '2026-08-25T01:05:00.000Z',
  candidates: [
    {
      index: 1,
      producers: [
        'tabby-1',
        'calico-2',
      ],
      rendered: 'The purr rolls on.',
    },
  ],
  ballots: [
    {
      modelId: 'siamese-3',
      best: 4,
      reason: 'named a candidate the slate lacks',
    },
  ],
  selectedIndex: 'declined',
};

/**
 Closing sentence of the whole-ledger summary, which points at the seat view.
 */
export const SUMMARY_POINTER: string = 'Pass --model <id> to read one seat\'s text and the reasons judges gave. '
  + 'A low share means rarely picked as best, which is not the same as wrong.';

/**
 Sentence after the count of files that would not read, saying what the
 figures then are.
 */
export const SHORTFALL_SENTENCE: string = 'Every figure here counts only the files that could, so a seat that wrote '
  + 'into an unreadable contest is undercounted, and so is every judge who weighed it. Re-run the pass to '
  + 'rewrite them, or read the standing as a floor.';

/**
 Writes one contest under a runs directory's ledger directory.

 @param runsDir - runs directory to write under

 @param name - contest file's name

 @param contest - value to record, any JSON so a case can write a malformed one

 @example
 ```ts
 await writeContest({ runsDir, name: '2026-08-25T01-00-00.000Z-p1-000001.json', contest: NAP_CONTEST, },);
 ```
 */
export async function writeContest(
  {
    runsDir,
    name,
    contest,
  }: {
    readonly runsDir: string;
    readonly name: string;
    readonly contest: unknown;
  },
): Promise<void> {
  /**
   Ledger directory of that run.
   */
  const dir = join(
    runsDir,
    LEDGER_DIR,
  );
  await mkdir(
    dir,
    { recursive: true, },
  );
  await writeFile(
    join(
      dir,
      name,
    ),
    JSON.stringify(contest,),
  );
}

//endregion Ledger report fixture
