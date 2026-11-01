/**
 Tests for the judged-contest ledger.

 WHAT THIS GUARDS is the reason the module exists: a roster question asked
 after a run should be answerable from what the run already paid for. The
 cases therefore check that the TEXT survives, not merely that a file appears.

 THE UNSET CASE IS NOT AN EDGE CASE. Every unit run and every probe sets no
 run directory, so writing nothing there is the ordinary path, and a module
 that threw or wrote into the working tree instead would break both.

 THE FAILURE CASE MATTERS MORE THAN THE SUCCESS ONE. Telemetry that can fail a
 slice is worse than no telemetry, so a write into an impossible location must
 leave the caller undisturbed.

 Model identifiers come from the catalog. Candidate text here is cat-themed
 invention, never corpus wording.

 @module
 */

import {
  readdir,
  readFile,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  inEntryLogContext,
  inSliceLogContext,
} from '@monochromatic-dev/translation-repair-typography';
import { compareCodePoints, } from '@monochromatic-dev/translation-repair-markup';
import {
  type Candidate,
  recordContest,
  type SelectionBallot,
} from '../dist/final/node/index.mjs';
import {
  SEAT_BEDROCK_ONLY_VISION_UNSEATED,
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_HYPER_TEXT_BEDROCK,
  SEAT_HYPER_VISION,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { runsDirPointedAt, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/runs-dir-pointed.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

/**
 Directory the recorder writes under, matching the module under test.
 */
const LEDGER_SUBDIR = 'ledger';

/**
 Logger the recorder writes its own warnings through.
 */
const l = tagged({ tag: 'candidate-ledger-test', },);

/**
 Two candidates, one written alone and one by several models together.
 */
const CANDIDATES: readonly Candidate<{ readonly text: string; }>[] = [
  {
    producer: {
      kind: 'model',
      modelId: SEAT_BEDROCK_ONLY_VISION_UNSEATED,
    },
    value: { text: 'The cat slept on the warm flagstones.', },
    rendered: 'The cat slept on the warm flagstones.',
  },
  {
    producer: {
      kind: 'composite',
      contributors: [
        SEAT_HYPER_OPENROUTER_UNMEASURED,
        SEAT_HYPER_VISION,
      ],
    },
    value: { text: 'A cat lay sleeping on sun-warmed stone.', },
    rendered: 'A cat lay sleeping on sun-warmed stone.',
  },
];

/**
 Ballots as the judges cast them, reasons included.
 */
const BALLOTS: readonly SelectionBallot[] = [
  {
    modelId: SEAT_HYPER_TEXT_BEDROCK,
    best: 2,
    reason: 'Candidate 2 reads more naturally in English.',
    weight: 1,
    selfVote: false,
  },
  {
    modelId: SEAT_HYPER_OPENROUTER_UNMEASURED,
    best: 2,
    reason: 'Candidate 2 is mine and I still think it is better.',
    weight: 0.5,
    selfVote: true,
  },
];

/**
 Lists the ledger directory, reporting an absent one as empty.

 @param dir - run directory written into

 @returns File names, empty where nothing was written

 @example
 ```ts
 const names = await namesUnder({ dir, },);
 ```
 */
async function namesUnder(
  { dir, }: { readonly dir: string; },
): Promise<readonly string[]> {
  try {
    return await readdir(join(
      dir,
      LEDGER_SUBDIR,
    ),);
  } catch (error) {
    // AN ABSENT DIRECTORY IS THE EXPECTED ANSWER in the cases that assert
    // nothing was written, so the class is named at debug rather than raised.
    l.debug(`no ledger directory: ${String(error,)}`,);
    return [];
  }
}

/**
 Reads every ledger file a run directory holds, in judging order.

 @param dir - run directory written into

 @returns Parsed rounds, empty where nothing was written

 @example
 ```ts
 const rounds = await ledgerIn({ dir, },);
 ```
 */
async function ledgerIn(
  { dir, }: { readonly dir: string; },
): Promise<readonly unknown[]> {
  /**
   Files the recorder wrote, empty where it wrote none.
   */
  const names = await namesUnder({ dir, },);

  return await Promise.all(names
    .toSorted(function byName(
      left,
      right,
    ): number {
      return compareCodePoints({
        left,
        right,
      },);
    },)
    .map(async function one(name,): Promise<unknown> {
      return JSON.parse(await readFile(
        join(
          dir,
          LEDGER_SUBDIR,
          name,
        ),
        'utf8',
      ),) as unknown;
    },),);
}

/**
 Records the shared fixture contest into a directory.

 @param selectedIndex - outcome to record for this round

 @param mark - names the case, for the throwaway directory

 @returns Rounds the recorder left behind

 @example
 ```ts
 const rounds = await recordedRounds({ selectedIndex: 2, mark: 'kept', },);
 ```
 */
async function recordedRounds(
  {
    selectedIndex,
    mark,
  }: {
    readonly selectedIndex: number | 'declined';
    readonly mark: string;
  },
): Promise<readonly unknown[]> {
  /**
   Throwaway this case writes into.
   */
  await using scratch = await scratchDir({ prefix: `ledger-${mark}-`, },);
  const dir = scratch.path;

  await recordContest({
    task: 'render this passage',
    candidates: CANDIDATES,
    ballots: BALLOTS,
    selectedIndex,
    l,
    contestLedger: {
      kind: 'kept',
      runsDir: dir,
    },
  },);

  return await ledgerIn({ dir, },);
}

await describe({
  name: recordContest.name,
  // ONE AT A TIME: the cases showing the recorder writes where its ledger says
  // point the one process-wide runs directory variable elsewhere, and a
  // restore finishing out of order could leave a sibling's value set (ledger B79).
  concurrency: 1,
  children: [
    it({
      name: 'KEEPS the text each model wrote, which is the whole reason this '
        + 'exists: a standing says a seat was rarely preferred and cannot say '
        + 'whether what it wrote was wrong',
      fn: async () => {
        expect((await recordedRounds({
          selectedIndex: 2,
          mark: 'text',
        },)).map(function texts(round,): unknown {
          return (round as { candidates: readonly { rendered: string; }[]; })
            .candidates
            .map(function rendered(candidate,): string {
              return candidate.rendered;
            },);
        },),)
          .toEqual([[
            'The cat slept on the warm flagstones.',
            'A cat lay sleeping on sun-warmed stone.',
          ],],);
      },
    },),

    it({
      name: 'NAMES every model behind a joint candidate, so a seat that only '
        + 'ever agrees with its peers is still findable in the record',
      fn: async () => {
        expect((await recordedRounds({
          selectedIndex: 2,
          mark: 'producers',
        },)).map(function producers(round,): unknown {
          return (round as { candidates: readonly { producers: readonly string[]; }[]; })
            .candidates
            .map(function names(candidate,): readonly string[] {
              return candidate.producers;
            },);
        },),)
          .toEqual([[
            [SEAT_BEDROCK_ONLY_VISION_UNSEATED,],
            [
              SEAT_HYPER_OPENROUTER_UNMEASURED,
              SEAT_HYPER_VISION,
            ],
          ],],);
      },
    },),

    it({
      name: 'KEEPS each judge reason verbatim, since the reason is what says '
        + 'whether a candidate lost on a defect or on taste',
      fn: async () => {
        expect((await recordedRounds({
          selectedIndex: 2,
          mark: 'reasons',
        },)).map(function reasons(round,): unknown {
          return (round as { ballots: readonly { reason: string; }[]; })
            .ballots
            .map(function reason(ballot,): string {
              return ballot.reason;
            },);
        },),)
          .toEqual([[
            'Candidate 2 reads more naturally in English.',
            'Candidate 2 is mine and I still think it is better.',
          ],],);
      },
    },),

    it({
      name: 'RECORDS a declined round as declined, rather than inventing a '
        + 'winner from the leading candidate',
      fn: async () => {
        expect((await recordedRounds({
          selectedIndex: 'declined',
          mark: 'declined',
        },)).map(function chosen(round,): unknown {
          return (round as { selectedIndex: number | string; }).selectedIndex;
        },),)
          .toEqual(['declined',],);
      },
    },),

    it({
      name: 'WRITES nothing at all with its ledger unkept, even while the process\'s runs directory variable names '
        + 'a directory, which is what every unit run and every probe launched without one does',
      fn: async () => {
        /**
         Throwaway the process's variable names, which must stay empty.
         */
        await using scratch = await scratchDir({ prefix: 'ledger-unset-', },);
        const dir = scratch.path;
        using pointed = runsDirPointedAt({ path: dir, },);

        await recordContest({
          task: 'render this passage',
          candidates: CANDIDATES,
          ballots: BALLOTS,
          selectedIndex: 2,
          l,
          contestLedger: { kind: 'unkept', },
        },);

        expect(await namesUnder({ dir, },),).toEqual([],);
      },
    },),

    it({
      name: 'WRITES UNDER THE RUNS DIRECTORY ITS LEDGER NAMES, not under the one the process\'s variable names',
      fn: async () => {
        /**
         Runs directory the ledger names.
         */
        await using kept = await scratchDir({ prefix: 'ledger-kept-', },);
        /**
         Runs directory the process's variable names, which must stay empty.
         */
        await using elsewhere = await scratchDir({ prefix: 'ledger-elsewhere-', },);
        using pointed = runsDirPointedAt({ path: elsewhere.path, },);

        await recordContest({
          task: 'render this passage',
          candidates: CANDIDATES,
          ballots: BALLOTS,
          selectedIndex: 2,
          l,
          contestLedger: {
            kind: 'kept',
            runsDir: kept.path,
          },
        },);

        expect({
          kept: (await namesUnder({ dir: kept.path, },)).length,
          elsewhere: await namesUnder({ dir: elsewhere.path, },),
        },).toEqual({
          kept: 1,
          elsewhere: [],
        },);
      },
    },),

    it({
      name: 'REFUSES to raise when the write cannot happen, because telemetry '
        + 'that can fail a slice is worse than telemetry that is missing',
      fn: async () => {
        /**
         Throwaway with a FILE where the recorder expects a directory, so the
         write cannot succeed.
         */
        await using scratch = await scratchDir({ prefix: 'ledger-blocked-', },);
        const dir = scratch.path;
        await writeFile(
          join(
            dir,
            LEDGER_SUBDIR,
          ),
          'not a directory',
          'utf8',
        );

        await recordContest({
          task: 'render this passage',
          candidates: CANDIDATES,
          ballots: BALLOTS,
          selectedIndex: 2,
          l,
          contestLedger: {
            kind: 'kept',
            runsDir: dir,
          },
        },);

        expect((await ledgerIn({ dir, },)).length,)
          .toBe(0,);
      },
    },),
    it({
      name: 'NAMES the entry, pipeline, attempt, lane and slice a contest was judged for, so a reader can join it '
        + 'to the artifact that shipped it and to the attempt that judged it (ledger A11, B960)',
      fn: async () => {
        /**
         Throwaway this case writes into.
         */
        await using scratch = await scratchDir({ prefix: 'ledger-context-', },);
        const dir = scratch.path;
        await inEntryLogContext({
          entry: 'Tabby',
          generation: 'nap-3',
          attempt: 3,
          run: async function judgeTheEntry(): Promise<void> {
            await inSliceLogContext({
              lane: 'translate',
              sliceIndex: 4,
              run: async function judgeTheSlice(): Promise<void> {
                await recordContest({
                  task: 'render this passage',
                  candidates: CANDIDATES,
                  ballots: BALLOTS,
                  selectedIndex: 2,
                  l,
                  contestLedger: {
                    kind: 'kept',
                    runsDir: dir,
                  },
                },);
              },
            },);
          },
        },);
        expect((await ledgerIn({ dir, },)).map(function contextOf(round,): unknown {
          return (round as { readonly context?: unknown; }).context;
        },),).toStrictEqual([{
          entry: 'Tabby',
          generation: 'nap-3',
          attempt: '3',
          lane: 'translate',
          slice: '4',
        },],);
      },
    },),
  ],
},);
