/**
 Tests for the procedure a corpus pass runs, over everything a process would
 hand it: the environment, the clock, the git tip, the client and the
 settlement of one entry are all scripted, and the corpus is a throwaway
 repository of invented pages, so no case reads the operator's clone, runs
 directory or cache, or reaches a network.

 THE ORDER IS WHAT THESE HOLD: the lock before anything is read, the guards
 before anything is printed, the START line and the launch notes before the
 required providers are asked, the client built once and only after them, the
 plan returning before any page is written, and the pages republished before
 the first entry runs.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  mkdir,
  readdir,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { nonNullishOrThrow, } from '@monochromatic-dev/module-or-throw/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { prepareDocumentPair, } from '@monochromatic-dev/translation-repair-adjudication';
import {
  lockRunsDir,
  prepareRunsLayout,
  PROVIDERS_LEFT_OFF,
  runClientFrom,
  RunsDirectoryBusyError,
} from '@monochromatic-dev/translation-repair-run';
import { EveryProviderDryError, } from '@monochromatic-dev/translation-repair-provider';
import type { EntryOutcome, } from '@monochromatic-dev/translation-repair-slice-cache';
import {
  digestPipeline,
  StatedRefusalError,
  writeDeclinedEntry,
} from '@monochromatic-dev/translation-repair-glossary';
import {
  type CorpusPassInput,
  passProvidersOffLines,
  runCorpusPassOver,
} from '../../dist/final/node/index.mjs';
import {
  scratchDir,
  scratchDirWith,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { lineOf, } from '@monochromatic-dev/translation-repair-run/ts/corpus-run/command-line.test-fixture.ts';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import { SEAT_HYPER_OPENROUTER_VISION_EDITOR, } from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { relayingConsoleLog, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/console-log-capture.test-fixture.ts';
import { makeCorpusPassClone, } from './corpus-pass-clone.test-fixture.ts';
import { NO_OUTSIDE_READS, } from './pass-outside-reads.test-fixture.ts';
import { NO_PICTURE_SOURCES, } from './pass-picture-sources.test-fixture.ts';
import { settledArtifactText, } from '@monochromatic-dev/translation-repair-artifact/ts/corpus-run/settled-artifact.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';
import { PASS_FALLBACK_OVERLAP, } from '@monochromatic-dev/translation-repair-slice-cache/ts/corpus-run/slice-overlap-dial.test-fixture.ts';

/**
 Tip the scripted git read answers.
 */
const TIP = 'f'.repeat(40,);

/**
 Key a case hands the run's environment, which belongs to no provider.
 */
const INVENTED_KEY = 'a-key-no-provider-issued';

/**
 Instant the scripted clock keeps reading once its list is spent.
 */
const LAST_INSTANT = 1_042;

/**
 What a case watched the procedure do.
 */
type Watched = {
  /**
   Entries handed to the settlement, in order.
   */
  readonly settled: string[];

  /**
   Prompt payload directories the client was asked for, in order.
   */
  readonly clients: string[];

  /**
   What was on disk the moment the first entry was settled.
   */
  readonly atFirstSettle: { stalePageThere: boolean; };
};

/**
 The fields of the procedure's input a case may set itself.
 */
type Overrides = {
  /**
   The pass's monotonic clock, for a case that moves it by hand.
   */
  readonly now?: () => number;

  /**
   Note naming the straggler window.
   */
  readonly graceNote?: string;

  /**
   Note naming the writer rounds' window.
   */
  readonly writerNote?: string;

  /**
   Environment the required providers' keys are read from.
   */
  readonly env?: CorpusPassInput['env'];

  /**
   HTTP the required providers' meters are read over.
   */
  readonly transport?: CorpusPassInput['transport'];

  /**
   What building the client does.
   */
  readonly newClient?: CorpusPassInput['newClient'];

  /**
   What settling one entry does.
   */
  readonly settle?: CorpusPassInput['settle'];
};

/**
 Transport a case must never reach, since no model is asked.

 @returns Never

 @throws Error always
 */
async function unreachedTransport(): ReturnType<CorpusPassInput['transport']> {
  throw new Error('the pass reached a transport this case never answers',);
}

/**
 Runs the procedure over a throwaway runs directory, corpus and build, with
 every effect scripted, and reports what it printed and did.

 @param typed - arguments after the command
 @param clone - throwaway corpus clone and its commit
 @param runsDir - throwaway runs directory
 @param pipelineDir - throwaway directory standing for the built pipeline
 @param watched - where to note what the procedure did
 @param overrides - fields of the procedure's input a case sets itself

 @returns What the procedure was handed back, once it ends

 @example
 ```ts
 await runPass({ typed: [], clone, runsDir, pipelineDir, watched, overrides: {}, },);
 ```
 */
async function runPass(
  {
    typed,
    clone,
    runsDir,
    pipelineDir,
    watched,
    overrides,
  }: {
    readonly typed: readonly string[];
    readonly clone: {
      readonly cloneDir: string;
      readonly commitSha: string;
    };
    readonly runsDir: string;
    readonly pipelineDir: string;
    readonly watched: Watched;
    readonly overrides: Overrides;
  },
): Promise<void> {
  /**
   Instants the scripted clock reads, then reads for ever.
   */
  const instants = [
    1_000,
    1_000,
    1_042,
  ];

  await runCorpusPassOver({
    line: lineOf({
      command: 'corpus-pass',
      typed,
    },),
    runsDir,
    graceNote: '',
    writerNote: '',
    pinSetting: {
      pin: clone,
      cloneDirSource: 'fallback',
      commitSource: 'fallback',
    },
    hardCapMs: 25_200_000,
    spendCeilingUsd: 20,
    driftAllowed: false,
    pipelineDir,
    readTip: async function readTip() {
      return TIP;
    },
    env: {},
    transport: unreachedTransport,
    newClient: function newClient({ promptPayloadDir, },) {
      watched.clients.push(promptPayloadDir,);
      return runClientFrom({
        env: { TRANSLATION_REPAIR_SYNTHETIC_API_KEY: INVENTED_KEY, },
        transport: unreachedTransport,
      },);
    },
    settle: async function settle({
      entry,
      artifactsDir,
      publishDir,
    },): Promise<EntryOutcome> {
      if (watched.settled.length === 0) {
        watched.atFirstSettle.stalePageThere = (await readdir(
          publishDir,
          { recursive: true, },
        )).includes(join(
          'people',
          'mittens',
          'page.en.md',
        ),);
      }
      watched.settled.push(entry.id,);
      await writeFile(
        join(
          artifactsDir,
          `${entry.id}.json`,
        ),
        '{}',
      );
      return { kind: 'settled', };
    },
    outsideReads: NO_OUTSIDE_READS,
    pictureSources: NO_PICTURE_SOURCES,
    now: function now() {
      return instants.shift() ?? LAST_INSTANT;
    },
    ...overrides,
    overlap: PASS_FALLBACK_OVERLAP,
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);
}

/**
 Throwaway directory standing for the built pipeline, holding one built file.

 @returns The directory and the digest taken over it

 @example
 ```ts
 const { dir, digest, fileCount, } = await builtPipeline();
 ```
 */
async function builtPipeline(): Promise<{
  readonly dir: string;
  readonly digest: string;
  readonly fileCount: number;
} & AsyncDisposable> {
  return await scratchDirWith({
    prefix: 'corpus-pass-build-',
    setup: async function built({ path, },): Promise<{
      readonly dir: string;
      readonly digest: string;
      readonly fileCount: number;
    }> {
      await writeFile(
        join(
          path,
          'naps.mjs',
        ),
        'export const naps = 1;\n',
      );
      /**
       Digest and file count the procedure takes over it.
       */
      const stamp = await digestPipeline({ dir: path, },);
      return {
        dir: path,
        digest: stamp.digest,
        fileCount: stamp.fileCount,
      };
    },
  },);
}

/**
 Fresh record of what a case watches.

 @returns Nothing seen yet

 @example
 ```ts
 const watched = nothingWatched();
 ```
 */
function nothingWatched(): Watched {
  return {
    settled: [],
    clients: [],
    atFirstSettle: { stalePageThere: true, },
  };
}

/**
 Lines a launch prints to say which providers it leaves off, after the required-provider lines and before the first entry.
 */
const OFF_LINES: readonly string[] = passProvidersOffLines({ off: PROVIDERS_LEFT_OFF, },);

await describe({
  name: runCorpusPassOver.name,
  concurrency: 1,
  children: [
    it({
      name: 'RUNS EVERY PENDING ENTRY ONCE and closes on what it finished of what it set out to run, over the '
        + 'build and the tip it was handed',
      fn: async (ctx) => {
        using printed = relayingConsoleLog({ sinon: ctx.sinon, },);
        await using clone = await makeCorpusPassClone({ entries: [{
          id: 'tabby',
          sourceText: '猫睡觉。\n',
          targetText: 'The cat naps.\n',
        },], },);
        await using runs = await scratchDir({ prefix: 'corpus-pass-run-', },);
        await using built = await builtPipeline();
        /**
         What the procedure did.
         */
        const watched = nothingWatched();

        await runPass({
          typed: [],
          clone,
          runsDir: runs.path,
          pipelineDir: built.dir,
          watched,
          overrides: {},
        },);

        expect(printed.lines,).toEqual([
          `START tip=${TIP} pipeline=${built.digest} files=${String(built.fileCount,)} pending=1 done=0 `
            + 'soft=259200000ms hard=25200000ms',
          ...OFF_LINES,
          'DONE processed=1 of pending=1; artifacts=1/1 declined=0 unpairedArtifacts=0 elapsed=42ms',
        ],);
        expect(watched.settled,).toEqual(['tabby',],);
        expect(watched.clients,).toEqual([
          join(
            runs.path,
            'prompt-payloads',
          ),
        ],);
      },
    },),
    it({
      name: 'CLOSES OVER THE COMPLETE PAIRS ITS OWN WALK FOUND, counting an entry declined before apart and out '
        + 'of the pairs still to settle, and an entry with no English page in neither',
      fn: async (ctx) => {
        using printed = relayingConsoleLog({ sinon: ctx.sinon, },);
        await using clone = await makeCorpusPassClone({ entries: [
          {
            id: 'biscuit',
            sourceText: '猫睡觉。\n',
            targetText: 'The cat naps.\n',
          },
          {
            id: 'mittens',
            sourceText: '猫睡觉。\n',
          },
          {
            id: 'mochi',
            sourceText: '猫睡觉。\n',
            targetText: 'The cat naps.\n',
          },
          {
            id: 'tabby',
            sourceText: '猫睡觉。\n',
            targetText: 'The cat naps.\n',
          },
        ], },);
        await using runs = await scratchDir({ prefix: 'corpus-pass-run-', },);
        await using built = await builtPipeline();
        /**
         Paths the procedure keeps under the runs directory.
         */
        const layout = await prepareRunsLayout({ runsDir: runs.path, },);
        await writeDeclinedEntry({
          declinedDir: layout.declinedDir,
          record: {
            id: 'mochi',
            tip: TIP,
            pipelineDigest: built.digest,
            corpusSha: clone.commitSha,
            timestamp: '2026-09-08T21:00:00.000Z',
            reason: 'archive-original',
            note: 'The cat wrote this herself.',
          },
        },);

        await runPass({
          typed: [],
          clone,
          runsDir: runs.path,
          pipelineDir: built.dir,
          watched: nothingWatched(),
          overrides: {},
        },);

        expect(printed.lines,).toEqual([
          `INCOMPLETE mittens: target page absent at the pin (corpus read failed for ${clone.commitSha}:`
            + 'people/mittens/page.en.md (missing-object); the commit has no such path: check the path, or pin a '
            + 'commit that has it.)',
          `START tip=${TIP} pipeline=${built.digest} files=${String(built.fileCount,)} pending=2 done=1 `
            + 'soft=259200000ms hard=25200000ms',
          ...OFF_LINES,
          'DONE processed=2 of pending=2; artifacts=2/2 declined=1 unpairedArtifacts=0 elapsed=42ms',
        ],);
      },
    },),
    it({
      name: 'COUNTS AN ARTIFACT SETTLED AT AN EARLIER PIN for an entry this pin does not hold apart, as an '
        + 'artifact with no complete pair, never among the pairs, so the artifacts never outnumber the pairs',
      fn: async (ctx) => {
        using printed = relayingConsoleLog({ sinon: ctx.sinon, },);
        await using clone = await makeCorpusPassClone({ entries: [{
          id: 'tabby',
          sourceText: '猫睡觉。\n',
          targetText: 'The cat naps.\n',
        },], },);
        await using runs = await scratchDir({ prefix: 'corpus-pass-run-', },);
        await using built = await builtPipeline();
        /**
         Paths the procedure keeps under the runs directory.
         */
        const layout = await prepareRunsLayout({ runsDir: runs.path, },);
        await writeFile(
          join(
            layout.artifactsDir,
            'ghost.json',
          ),
          JSON.stringify({
            ...JSON.parse(settledArtifactText({
              prepared: prepareDocumentPair({
                sourceText: '猫睡觉。\n',
                targetText: 'The cat naps.\n',
                includeFrontMatter: true,
                sealArchiveOriginal: true,
              },),
              entryId: 'ghost',
            },),),
            pipelineDigest: built.digest,
          },),
        );

        await runPass({
          typed: [],
          clone,
          runsDir: runs.path,
          pipelineDir: built.dir,
          watched: nothingWatched(),
          overrides: {},
        },);

        // The republish lines the earlier artifact draws (its pair is read at
        // a commit this clone lacks) are another case's; this one reads the
        // run's opening and closing lines.
        expect(printed.lines.filter(function openingOrClosing(line,): boolean {
          return line.startsWith('START ',) || line.startsWith('DONE ',);
        },),).toEqual([
          `START tip=${TIP} pipeline=${built.digest} files=${String(built.fileCount,)} pending=1 done=1 `
            + 'soft=259200000ms hard=25200000ms',
          'DONE processed=1 of pending=1; artifacts=1/1 declined=0 unpairedArtifacts=1 elapsed=42ms',
        ],);
      },
    },),
    it({
      name: 'COUNTS THE ARTIFACT OF A FINISHED ENTRY WHOSE ENGLISH PAGE THE PIN LACKS apart, as an artifact with no '
        + 'complete pair, though the pin holds its original',
      fn: async (ctx) => {
        using printed = relayingConsoleLog({ sinon: ctx.sinon, },);
        await using clone = await makeCorpusPassClone({ entries: [
          {
            id: 'mittens',
            sourceText: '猫睡觉。\n',
          },
          {
            id: 'tabby',
            sourceText: '猫睡觉。\n',
            targetText: 'The cat naps.\n',
          },
        ], },);
        await using runs = await scratchDir({ prefix: 'corpus-pass-run-', },);
        await using built = await builtPipeline();
        /**
         Paths the procedure keeps under the runs directory.
         */
        const layout = await prepareRunsLayout({ runsDir: runs.path, },);
        await writeFile(
          join(
            layout.artifactsDir,
            'mittens.json',
          ),
          JSON.stringify({
            ...JSON.parse(settledArtifactText({
              prepared: prepareDocumentPair({
                sourceText: '猫睡觉。\n',
                targetText: 'The cat naps.\n',
                includeFrontMatter: true,
                sealArchiveOriginal: true,
              },),
              entryId: 'mittens',
            },),),
            pipelineDigest: built.digest,
          },),
        );

        await runPass({
          typed: [],
          clone,
          runsDir: runs.path,
          pipelineDir: built.dir,
          watched: nothingWatched(),
          overrides: {},
        },);

        // The republish lines the finished entry's artifact draws are another
        // case's; this one reads the run's opening and closing lines.
        expect(printed.lines.filter(function openingOrClosing(line,): boolean {
          return line.startsWith('START ',) || line.startsWith('DONE ',);
        },),).toEqual([
          `START tip=${TIP} pipeline=${built.digest} files=${String(built.fileCount,)} pending=1 done=1 `
            + 'soft=259200000ms hard=25200000ms',
          'DONE processed=1 of pending=1; artifacts=1/1 declined=0 unpairedArtifacts=1 elapsed=42ms',
        ],);
      },
    },),
    it({
      name: 'CLOSES OVER THE PAIRS AMONG THE NAMED ENTRIES under --only, saying so, since its walk read them '
        + 'alone',
      fn: async (ctx) => {
        using printed = relayingConsoleLog({ sinon: ctx.sinon, },);
        await using clone = await makeCorpusPassClone({ entries: [
          {
            id: 'biscuit',
            sourceText: '猫睡觉。\n',
            targetText: 'The cat naps.\n',
          },
          {
            id: 'tabby',
            sourceText: '猫睡觉。\n',
            targetText: 'The cat naps.\n',
          },
        ], },);
        await using runs = await scratchDir({ prefix: 'corpus-pass-run-', },);
        await using built = await builtPipeline();

        await runPass({
          typed: [
            '--only',
            'tabby',
          ],
          clone,
          runsDir: runs.path,
          pipelineDir: built.dir,
          watched: nothingWatched(),
          overrides: {},
        },);

        expect(printed.lines,).toEqual([
          'ONLY tabby (chosen by hand in place of every pending pair at the pin; if still pending it runs in '
            + 'the pass\'s own order; run this into a throwaway TRANSLATION_REPAIR_RUNS_DIR so a hand-picked '
            + 'entry never enters a pool later draws treat as natural accumulation)',
          `START tip=${TIP} pipeline=${built.digest} files=${String(built.fileCount,)} pending=1 done=0 `
            + 'soft=259200000ms hard=25200000ms',
          ...OFF_LINES,
          'DONE processed=1 of pending=1; artifacts=1/1 declined=0 (pairs counted over the entries --only named, '
            + 'not every pair at the pin) elapsed=42ms',
        ],);
      },
    },),
    it({
      name: 'PRINTS EVERY LAUNCH NOTE it was handed after the START line, and the providers found wet',
      fn: async (ctx) => {
        using printed = relayingConsoleLog({ sinon: ctx.sinon, },);
        await using clone = await makeCorpusPassClone({ entries: [], },);
        await using runs = await scratchDir({ prefix: 'corpus-pass-run-', },);
        await using built = await builtPipeline();

        await runPass({
          typed: [
            '--require-providers',
            'hyper',
          ],
          clone,
          runsDir: runs.path,
          pipelineDir: built.dir,
          watched: nothingWatched(),
          overrides: {
            graceNote: 'a straggler note',
            writerNote: 'a writer note',
            env: { TRANSLATION_REPAIR_CHARM_HYPER_API_KEY: INVENTED_KEY, },
            transport: async function answerCredits() {
              return {
                status: 200,
                bodyText: '{"balance":243}',
              };
            },
          },
        },);

        expect(printed.lines,).toEqual([
          `START tip=${TIP} pipeline=${built.digest} files=${String(built.fileCount,)} pending=0 done=0 `
            + 'soft=259200000ms hard=25200000ms',
          'a straggler note',
          'a writer note',
          'REQUIRED-PROVIDERS hyper status=wet',
          ...OFF_LINES,
          'DONE processed=0 of pending=0; artifacts=0/0 declined=0 unpairedArtifacts=0 elapsed=0ms',
        ],);
      },
    },),
    it({
      name: 'HANDS THE CLIENT WHAT THE PASS HAS LEFT, unbounded before the loop starts and the hard cap or less once '
        + 'an entry runs, so the seat reader waits for NeuralWatt\'s return only inside it',
      fn: async (ctx) => {
        using _printed = relayingConsoleLog({ sinon: ctx.sinon, },);
        await using clone = await makeCorpusPassClone({ entries: [
          {
            id: 'tabby',
            sourceText: '猫睡觉。\n',
            targetText: 'The cat naps.\n',
          },
        ], },);
        await using runs = await scratchDir({ prefix: 'corpus-pass-run-', },);
        await using built = await builtPipeline();
        /**
         The bound the client was handed and what it read before the loop and at the first entry.
         */
        const seen: { before?: number; during?: number; } = {};

        await runPass({
          typed: [],
          clone,
          runsDir: runs.path,
          pipelineDir: built.dir,
          watched: nothingWatched(),
          overrides: {
            newClient: function newClient({ passLeftMs, },) {
              seen.before = passLeftMs();
              return runClientFrom({
                env: { TRANSLATION_REPAIR_SYNTHETIC_API_KEY: INVENTED_KEY, },
                transport: unreachedTransport,
                passLeftMs,
              },);
            },
            settle: async function settle({ entry, client, artifactsDir, },): Promise<EntryOutcome> {
              // Conditional write keeps an absent bound absent, which the assertions then fail on.
              if (client.passLeftMs !== undefined)
                seen.during = client.passLeftMs();
              await writeFile(
                join(
                  artifactsDir,
                  `${entry.id}.json`,
                ),
                '{}',
              );
              return { kind: 'settled', };
            },
          },
        },);

        expect(seen.before,).toBe(Number.POSITIVE_INFINITY,);
        expect(seen.during,).toBeLessThanOrEqual(25_200_000,);
        expect(seen.during,).toBeGreaterThan(0,);
      },
    },),
    it({
      name: 'COUNTS THE SOFT BUDGET DOWN ON THE PASS\'S CLOCK in what the client is handed: the first entry reads the '
        + '420 min hard cap (the soft budget is far longer), and once the pass\'s clock has run to 320 min short of '
        + 'the soft budget the bound is exactly what the soft budget has left, falling with the clock',
      fn: async (ctx) => {
        using _printed = relayingConsoleLog({ sinon: ctx.sinon, },);
        await using clone = await makeCorpusPassClone({ entries: [
          {
            id: 'tabby',
            sourceText: '猫睡觉。\n',
            targetText: 'The cat naps.\n',
          },
        ], },);
        await using runs = await scratchDir({ prefix: 'corpus-pass-run-', },);
        await using built = await builtPipeline();
        /**
         The pass's monotonic clock, held still until the case moves it.
         */
        const clock = { now: 1_000, };
        /**
         What the client read at the first entry, then after the clock ran to 20 min short of the soft budget,
         then a minute later.
         */
        const seen: number[] = [];

        await runPass({
          typed: [],
          clone,
          runsDir: runs.path,
          pipelineDir: built.dir,
          watched: nothingWatched(),
          overrides: {
            now: function now() {
              return clock.now;
            },
            newClient: function newClient({ passLeftMs, },) {
              return runClientFrom({
                env: { TRANSLATION_REPAIR_SYNTHETIC_API_KEY: INVENTED_KEY, },
                transport: unreachedTransport,
                passLeftMs,
              },);
            },
            settle: async function settle({ entry, client, artifactsDir, },): Promise<EntryOutcome> {
              seen.push(client.passLeftMs?.() ?? Number.NaN,);
              // 259,200,000 ms of soft budget less 20 min (1,200,000 ms) left.
              clock.now += 258_000_000;
              seen.push(client.passLeftMs?.() ?? Number.NaN,);
              clock.now += 60_000;
              seen.push(client.passLeftMs?.() ?? Number.NaN,);
              await writeFile(
                join(
                  artifactsDir,
                  `${entry.id}.json`,
                ),
                '{}',
              );
              return { kind: 'settled', };
            },
          },
        },);

        expect(seen,).toEqual([25_200_000, 1_200_000, 1_140_000,],);
      },
    },),
    it({
      name: 'ENDS ON THE PLAN LINE under --plan, with the client built and no entry settled',
      fn: async (ctx) => {
        using printed = relayingConsoleLog({ sinon: ctx.sinon, },);
        await using clone = await makeCorpusPassClone({ entries: [{
          id: 'tabby',
          sourceText: '猫睡觉。\n',
          targetText: 'The cat naps.\n',
        },], },);
        await using runs = await scratchDir({ prefix: 'corpus-pass-run-', },);
        await using built = await builtPipeline();
        /**
         What the procedure did.
         */
        const watched = nothingWatched();

        await runPass({
          typed: ['--plan',],
          clone,
          runsDir: runs.path,
          pipelineDir: built.dir,
          watched,
          overrides: {},
        },);

        expect(printed.lines,).toEqual([
          `START tip=${TIP} pipeline=${built.digest} files=${String(built.fileCount,)} pending=1 done=0 `
            + 'soft=259200000ms hard=25200000ms',
          ...OFF_LINES,
          `PLAN ok tip=${TIP} pipeline=${built.digest} client=constructed pending=1 first=tabby`,
        ],);
        expect(watched.settled,).toEqual([],);
        expect(watched.clients.length,).toBe(1,);
      },
    },),
    it({
      name: 'REMOVES THE PAGE OF A DECLINED ENTRY BEFORE THE FIRST ENTRY RUNS, and counts the decline among those '
        + 'finished before it',
      fn: async (ctx) => {
        using printed = relayingConsoleLog({ sinon: ctx.sinon, },);
        await using clone = await makeCorpusPassClone({ entries: [{
          id: 'tabby',
          sourceText: '猫睡觉。\n',
          targetText: 'The cat naps.\n',
        },], },);
        await using runs = await scratchDir({ prefix: 'corpus-pass-run-', },);
        await using built = await builtPipeline();
        /**
         Paths the procedure keeps under the runs directory.
         */
        const layout = await prepareRunsLayout({ runsDir: runs.path, },);
        await writeDeclinedEntry({
          declinedDir: layout.declinedDir,
          record: {
            id: 'mittens',
            tip: TIP,
            pipelineDigest: built.digest,
            corpusSha: clone.commitSha,
            timestamp: '2026-09-08T21:00:00.000Z',
            reason: 'archive-original',
            note: 'The cat wrote this herself.',
          },
        },);
        await mkdir(
          join(
            layout.publishDir,
            'people',
            'mittens',
          ),
          { recursive: true, },
        );
        await writeFile(
          join(
            layout.publishDir,
            'people',
            'mittens',
            'page.en.md',
          ),
          'A page an earlier crash left.\n',
        );
        /**
         What the procedure did.
         */
        const watched = nothingWatched();

        await runPass({
          typed: [],
          clone,
          runsDir: runs.path,
          pipelineDir: built.dir,
          watched,
          overrides: {},
        },);

        expect(watched.atFirstSettle.stalePageThere,).toBe(false,);
        expect(printed.lines[0],).toBe(
          `START tip=${TIP} pipeline=${built.digest} files=${String(built.fileCount,)} pending=1 done=1 `
            + 'soft=259200000ms hard=25200000ms',
        );
      },
    },),
    it({
      name: 'REFUSES A RUNS DIRECTORY ANOTHER PASS HOLDS before it prints or reads anything',
      fn: async (ctx) => {
        using printed = relayingConsoleLog({ sinon: ctx.sinon, },);
        await using clone = await makeCorpusPassClone({ entries: [], },);
        await using runs = await scratchDir({ prefix: 'corpus-pass-run-', },);
        await using built = await builtPipeline();
        await using _held = await lockRunsDir({ runsDir: runs.path, },);
        /**
         What the procedure did.
         */
        const watched = nothingWatched();

        await expect(runPass({
          typed: [],
          clone,
          runsDir: runs.path,
          pipelineDir: built.dir,
          watched,
          overrides: {},
        },),).rejects.toBeInstanceOf(RunsDirectoryBusyError,);
        expect(printed.lines,).toEqual([],);
        expect(watched.clients,).toEqual([],);
      },
    },),
    it({
      name: 'REFUSES AN ARTIFACT NOTHING CAN PLACE before it prints, builds a client or settles anything',
      fn: async (ctx) => {
        using printed = relayingConsoleLog({ sinon: ctx.sinon, },);
        await using clone = await makeCorpusPassClone({ entries: [{
          id: 'tabby',
          sourceText: '猫睡觉。\n',
          targetText: 'The cat naps.\n',
        },], },);
        await using runs = await scratchDir({ prefix: 'corpus-pass-run-', },);
        await using built = await builtPipeline();
        /**
         Paths the procedure keeps under the runs directory.
         */
        const layout = await prepareRunsLayout({ runsDir: runs.path, },);
        await writeFile(
          join(
            layout.artifactsDir,
            'mittens.json',
          ),
          '{}',
        );
        /**
         What the procedure did.
         */
        const watched = nothingWatched();

        await expect(runPass({
          typed: [],
          clone,
          runsDir: runs.path,
          pipelineDir: built.dir,
          watched,
          overrides: {},
        },),).rejects.toBeInstanceOf(StatedRefusalError,);
        expect(printed.lines,).toEqual([
          'POOL mittens.json records an id that is not its file name (absent); treating it as unplaceable',
        ],);
        expect(watched.clients,).toEqual([],);
        expect(watched.settled,).toEqual([],);
      },
    },),
    it({
      name: 'REFUSES A REQUIRED PROVIDER THAT HAS NO KEY after the START line and before any client is built',
      fn: async (ctx) => {
        using printed = relayingConsoleLog({ sinon: ctx.sinon, },);
        await using clone = await makeCorpusPassClone({ entries: [], },);
        await using runs = await scratchDir({ prefix: 'corpus-pass-run-', },);
        await using built = await builtPipeline();
        /**
         What the procedure did.
         */
        const watched = nothingWatched();

        await expect(runPass({
          typed: [
            '--require-providers',
            'hyper',
          ],
          clone,
          runsDir: runs.path,
          pipelineDir: built.dir,
          watched,
          overrides: {},
        },),).rejects.toThrow('required provider hyper is not ready: key missing',);
        expect(printed.lines,).toEqual([
          `START tip=${TIP} pipeline=${built.digest} files=${String(built.fileCount,)} pending=0 done=0 `
            + 'soft=259200000ms hard=25200000ms',
        ],);
        expect(watched.clients,).toEqual([],);
      },
    },),
    it({
      name: 'STOPS THE RUN WHEN THE ROUTER FINDS EVERY PROVIDER DRY: the entry in flight is stopped by the abort the '
        + 'refusal raised, no further entry starts, the entries already settled stay counted, and one STOP line names '
        + 'the cause',
      fn: async (ctx) => {
        using printed = relayingConsoleLog({ sinon: ctx.sinon, },);
        await using clone = await makeCorpusPassClone({ entries: [
          {
            id: 'tabby',
            sourceText: '猫睡觉。\n',
            targetText: 'The cat naps.\n',
          },
          {
            id: 'mittens',
            sourceText: '猫玩耍。\n',
            targetText: 'The cat plays.\n',
          },
          {
            id: 'whiskers',
            sourceText: '猫吃饭。\n',
            targetText: 'The cat dines.\n',
          },
        ], },);
        await using runs = await scratchDir({ prefix: 'corpus-pass-run-', },);
        await using built = await builtPipeline();
        /**
         What the procedure did.
         */
        const watched = nothingWatched();
        /**
         What the entry that met the dry router saw its base signal become.
         */
        const seen: { reason?: unknown; } = {};

        await runPass({
          typed: [],
          clone,
          runsDir: runs.path,
          pipelineDir: built.dir,
          watched,
          overrides: {
            newClient: function dryClient() {
              /**
               Router refusal every call meets.
               */
              const dry = new EveryProviderDryError({ measured: 'meters read every provider dry', },);
              return {
                ...runClientFrom({
                  env: { TRANSLATION_REPAIR_SYNTHETIC_API_KEY: INVENTED_KEY, },
                  transport: unreachedTransport,
                },),
                chatText: async () => {
                  throw dry;
                },
              };
            },
            settle: async function settle({ entry, client, artifactsDir, baseSignal, },): Promise<EntryOutcome> {
              watched.settled.push(entry.id,);
              if (watched.settled.length === 1) {
                await writeFile(
                  join(
                    artifactsDir,
                    `${entry.id}.json`,
                  ),
                  '{}',
                );
                return { kind: 'settled', };
              }
              await rejectionOf(async function ask(): Promise<unknown> {
                return await client.chatText({
                  modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
                  messages: [],
                  signal: baseSignal,
                },);
              },);
              seen.reason = baseSignal.reason;
              return { kind: 'stopped', };
            },
          },
        },);

        expect(watched.settled.length,).toBe(2,);
        expect(seen.reason,).toBeInstanceOf(EveryProviderDryError,);
        expect(printed.lines.filter(function isStop(line,): boolean {
          return line.startsWith('STOP ',);
        },),).toEqual([
          `STOP every provider is out of budget, so no further entry starts; the entries already settled keep their `
            + `pages and the entry in flight is left resumable. Cause: ${(seen.reason as Error).message}`,
        ],);
        expect(printed.lines.at(-1,),).toContain('DONE processed=1 of pending=3; artifacts=1/3',);
      },
    },),
    it({
      name: 'STOPS THE RUN WHEN AN ENTRY IS ENDED BY THE REFUSAL WITHOUT A GUARDED CALL, as the seat reading ends it before '
        + 'any chat call: the entry hands the refusal to the run, no further entry starts and one STOP line names the cause',
      fn: async (ctx) => {
        using printed = relayingConsoleLog({ sinon: ctx.sinon, },);
        await using clone = await makeCorpusPassClone({ entries: [
          {
            id: 'tabby',
            sourceText: '猫睡觉。\n',
            targetText: 'The cat naps.\n',
          },
          {
            id: 'mittens',
            sourceText: '猫玩耍。\n',
            targetText: 'The cat plays.\n',
          },
        ], },);
        await using runs = await scratchDir({ prefix: 'corpus-pass-run-', },);
        await using built = await builtPipeline();
        /**
         What the procedure did.
         */
        const watched = nothingWatched();
        await runPass({
          typed: [],
          clone,
          runsDir: runs.path,
          pipelineDir: built.dir,
          watched,
          overrides: {
            settle: async function settle({ entry, endRun, },): Promise<EntryOutcome> {
              watched.settled.push(entry.id,);
              nonNullishOrThrow(endRun,)({
                error: new EveryProviderDryError({ measured: 'the seat reading found every meter dry', },),
              },);
              return { kind: 'stopped', };
            },
          },
        },);

        expect(watched.settled.length,).toBe(1,);
        /**
         The stop lines the run printed.
         */
        const stops = printed.lines.filter(function isStop(line,): boolean {
          return line.startsWith('STOP ',);
        },);
        expect(stops,).toHaveLength(1,);
        expect(stops[0],).toContain('the seat reading found every meter dry',);
      },
    },),
    it({
      name: 'GIVES EVERY ENTRY THE RUN\'S ANSWER TO WHETHER IT PARKED FOR A FAILURE: the entry that hands the gate a '
        + 'refusal naming a return 5 ms away is told the run is parked for it, an unrelated failure is not, and the '
        + 'entry is attempted again after the park',
      fn: async (ctx) => {
        using printed = relayingConsoleLog({ sinon: ctx.sinon, },);
        await using clone = await makeCorpusPassClone({ entries: [
          {
            id: 'tabby',
            sourceText: '猫睡觉。\n',
            targetText: 'The cat naps.\n',
          },
        ], },);
        await using runs = await scratchDir({ prefix: 'corpus-pass-run-', },);
        await using built = await builtPipeline();
        /**
         What the procedure did.
         */
        const watched = nothingWatched();
        /**
         What the entry was told about its park, per attempt.
         */
        const told: { forRefusal?: unknown; forOther?: unknown; } = {};
        await runPass({
          typed: [],
          clone,
          runsDir: runs.path,
          pipelineDir: built.dir,
          watched,
          overrides: {
            settle: async function settle({ entry, endRun, parked, },): Promise<EntryOutcome> {
              watched.settled.push(entry.id,);
              if (watched.settled.length === 1) {
                /**
                 The refusal that names a return the pass can wait for.
                 */
                const dry = new EveryProviderDryError({
                  measured: 'the seat reading found every meter dry',
                  neuralwatt: { kind: 'at', msOfDay: 69_308_000, },
                  neuralwattReturnsInMs: 5,
                },);
                nonNullishOrThrow(endRun,)({ error: dry, },);
                told.forRefusal = nonNullishOrThrow(parked,)({ error: dry, },);
                told.forOther = nonNullishOrThrow(parked,)({ error: new Error('connection reset',), },);
              }
              return { kind: 'stopped', };
            },
          },
        },);

        expect(told,).toEqual({
          forRefusal: { parked: true, returnsInMs: 5, returnsAt: '19:15:08 UTC', },
          forOther: { parked: false, },
        },);
        expect(watched.settled,).toEqual(['tabby', 'tabby',],);
        expect(printed.lines.filter(function isStop(line,): boolean {
          return line.startsWith('STOP ',);
        },),).toEqual([],);
      },
    },),
  ],
},);
