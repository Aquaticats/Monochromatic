/**
 Tests for the queue a corpus pass runs its entries through: each attempt is
 counted and persisted before it starts, runs under its entry's name, and
 the run stops at the soft budget or the spend allowance.

 Every effect is injected: the settlement is scripted, the clock is a list of
 instants and the runs directory is a throwaway, so no case reads a corpus or
 calls a provider.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  mkdir,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  EveryProviderDryError,
  type ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import { WritingBenchUnreachableError, } from '@monochromatic-dev/translation-repair-run';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import { SEAT_HYPER_OPENROUTER_VISION_EDITOR, } from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { resetRunSpend, } from '@monochromatic-dev/translation-repair-shared-contract';
import { currentLogContext, } from '@monochromatic-dev/translation-repair-typography';
import {
  type AttemptMap,
  readAttemptMap,
} from '@monochromatic-dev/translation-repair-glossary';
import {
  createPassDryStop,
  runPassQueue,
} from '../../dist/final/node/index.mjs';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { relayingConsoleLog, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/console-log-capture.test-fixture.ts';

/**
 Generation every case's attempts run under.
 */
const GENERATION = `sha256-tree-v1:${'c'.repeat(64,)}`;

/**
 A pair of invented pages.

 @param id - entry id

 @returns The pair

 @example
 ```ts
 const pair = pairOf({ id: 'tabby', },);
 ```
 */
function pairOf({ id, }: { readonly id: string; },): {
  readonly id: string;
  readonly sourceText: string;
  readonly targetText: string;
} {
  return {
    id,
    sourceText: '猫睡觉。\n',
    targetText: 'The cat naps.\n',
  };
}

/**
 A clock reading each instant of a list in turn, and the last one for ever.

 @param instants - what it reads, first first

 @returns The clock

 @example
 ```ts
 const now = clockReading({ instants: [0, 5,], },);
 ```
 */
function clockReading({ instants, }: { readonly instants: readonly number[]; },): () => number {
  /**
   Readings not yet taken.
   */
  const remaining = [...instants,];

  /**
   Instant every reading after the list answers.
   */
  const last = instants.at(-1,) ?? 0;
  return function now(): number {
    return remaining.shift() ?? last;
  };
}

/**
 Makes one chat call through a guarded client and returns the failure it raises.

 @param client - guarded client

 @param signal - call's abort handle

 @returns Failure the call raised

 @example
 ```ts
 const failure = await dryCallOf({ client: dryClient, signal: dryStop.signal, },);
 ```
 */
async function dryCallOf(
  {
    client,
    signal,
  }: {
    readonly client: ModelClient;
    readonly signal: AbortSignal;
  },
): Promise<unknown> {
  return await rejectionOf(async function dryCall(): Promise<unknown> {
    return await client.chatText({
      modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
      messages: [],
      signal,
    },);
  },);
}

/**
 Chat call whose router finds every provider dry.

 @returns Never

 @throws {@link EveryProviderDryError} always

 @example
 ```ts
 await refuseDry();
 ```
 */
async function refuseDry(): Promise<never> {
  throw new EveryProviderDryError({ measured: 'meters read synthetic dry, openrouter dry', },);
}

await describe({
  name: runPassQueue.name,
  concurrency: 1,
  children: [
    it({
      name: 'COUNTS AND PERSISTS AN ATTEMPT BEFORE IT RUNS, and settles each entry under its own name and the '
        + 'build, so a crash still records that the attempt happened',
      fn: async (ctx) => {
        resetRunSpend();
        using _printed = relayingConsoleLog({ sinon: ctx.sinon, },);
        await using scratch = await scratchDir({ prefix: 'corpus-pass-queue-', },);
        /**
         Where the counts are persisted.
         */
        const attemptsPath = join(
          scratch.path,
          'attempts.json',
        );
        /**
         Counts this run keeps.
         */
        const attempts: AttemptMap = new Map([
          [
            'tabby',
            2,
          ],
        ],);
        /**
         What each settlement saw when it started.
         */
        const seen: {
          readonly entry: string;
          readonly persisted: ReadonlyMap<string, number>;
          readonly context: string;
          readonly generation: string;
        }[] = [];

        await runPassQueue({
          pending: [
            pairOf({ id: 'tabby', },),
            pairOf({ id: 'mittens', },),
          ],
          attempts,
          attemptsPath,
          sliceCacheDir: join(
            scratch.path,
            'slice-cache',
          ),
          pipelineDigest: GENERATION,
          dryStop: createPassDryStop(),
          softBudgetMs: 1_000,
          spendCeilingUsd: 20,
          start: 0,
          now: clockReading({ instants: [0,], },),
          settle: async function settle({ entry, },) {
            seen.push({
              entry: entry.id,
              persisted: await readAttemptMap(attemptsPath,),
              context: currentLogContext().entry,
              generation: currentLogContext().generation,
            },);
            return { kind: 'settled', };
          },
        },);

        expect(seen,).toEqual([
          {
            entry: 'tabby',
            persisted: new Map([
              [
                'tabby',
                3,
              ],
            ],),
            context: 'tabby',
            generation: GENERATION,
          },
          {
            entry: 'mittens',
            persisted: new Map([
              [
                'tabby',
                3,
              ],
              [
                'mittens',
                1,
              ],
            ],),
            context: 'mittens',
            generation: GENERATION,
          },
        ],);
        expect(attempts,).toEqual(new Map([
          [
            'tabby',
            3,
          ],
          [
            'mittens',
            1,
          ],
        ],),);
      },
    },),
    it({
      name: 'STOPS AT THE SOFT BUDGET before the next entry starts, and says how long the run had gone',
      fn: async (ctx) => {
        resetRunSpend();
        using printed = relayingConsoleLog({ sinon: ctx.sinon, },);
        await using scratch = await scratchDir({ prefix: 'corpus-pass-queue-', },);
        /**
         Entries settled.
         */
        const settled: string[] = [];

        await runPassQueue({
          pending: [
            pairOf({ id: 'tabby', },),
            pairOf({ id: 'mittens', },),
          ],
          attempts: new Map(),
          attemptsPath: join(
            scratch.path,
            'attempts.json',
          ),
          sliceCacheDir: join(
            scratch.path,
            'slice-cache',
          ),
          pipelineDigest: GENERATION,
          dryStop: createPassDryStop(),
          softBudgetMs: 1_000,
          spendCeilingUsd: 20,
          start: 100,
          now: clockReading({ instants: [
            100,
            1_100,
          ], },),
          settle: async function settle({ entry, },) {
            settled.push(entry.id,);
            return { kind: 'settled', };
          },
        },);

        expect(settled,).toEqual(['tabby',],);
        expect(printed.lines,).toEqual(['SOFT budget reached after 1000ms; not starting new entries',],);
      },
    },),
    it({
      name: 'STOPS BEFORE THE FIRST ENTRY under an allowance of nothing, and writes no attempt',
      fn: async (ctx) => {
        resetRunSpend();
        using printed = relayingConsoleLog({ sinon: ctx.sinon, },);
        await using scratch = await scratchDir({ prefix: 'corpus-pass-queue-', },);
        /**
         Entries settled.
         */
        const settled: string[] = [];
        /**
         Counts this run keeps.
         */
        const attempts: AttemptMap = new Map();

        await runPassQueue({
          pending: [pairOf({ id: 'tabby', },),],
          attempts,
          attemptsPath: join(
            scratch.path,
            'attempts.json',
          ),
          sliceCacheDir: join(
            scratch.path,
            'slice-cache',
          ),
          pipelineDigest: GENERATION,
          dryStop: createPassDryStop(),
          softBudgetMs: 1_000,
          spendCeilingUsd: 0,
          start: 0,
          now: clockReading({ instants: [0,], },),
          settle: async function settle({ entry, },) {
            settled.push(entry.id,);
            return { kind: 'settled', };
          },
        },);

        expect(settled,).toEqual([],);
        expect(attempts,).toEqual(new Map(),);
        expect(printed.lines.length,).toBe(1,);
      },
    },),
    it({
      name: 'QUEUES A SECOND ATTEMPT FOR AN ENTRY THAT CACHED ONE SLICE under its own directory in the slice cache, '
        + 'and DROPS ONE THAT CACHED NONE, each said in the singular where it counts one',
      fn: async (ctx) => {
        resetRunSpend();
        using printed = relayingConsoleLog({ sinon: ctx.sinon, },);
        await using scratch = await scratchDir({ prefix: 'corpus-pass-queue-', },);
        /**
         Slice cache the attempts write into.
         */
        const sliceCacheDir = join(
          scratch.path,
          'slice-cache',
        );
        /**
         Attempts made, by entry.
         */
        const made: string[] = [];

        await runPassQueue({
          pending: [
            pairOf({ id: 'tabby', },),
            pairOf({ id: 'mittens', },),
          ],
          attempts: new Map(),
          attemptsPath: join(
            scratch.path,
            'attempts.json',
          ),
          sliceCacheDir,
          pipelineDigest: GENERATION,
          dryStop: createPassDryStop(),
          softBudgetMs: 1_000,
          spendCeilingUsd: 20,
          start: 0,
          now: clockReading({ instants: [0,], },),
          settle: async function settle({ entry, },) {
            made.push(entry.id,);
            /**
             Attempts made so far on this entry, this one included.
             */
            const triesOfEntry = made.filter(function isThisEntry(id,): boolean {
              return id === entry.id;
            },).length;
            if ((entry.id === 'tabby') && (triesOfEntry === 1)) {
              await mkdir(
                join(
                  sliceCacheDir,
                  'tabby',
                ),
                { recursive: true, },
              );
              await writeFile(
                join(
                  sliceCacheDir,
                  'tabby',
                  '0.json',
                ),
                '{}',
              );
            }
            return { kind: 'resumable-failure', };
          },
        },);

        expect(made,).toEqual([
          'tabby',
          'mittens',
          'tabby',
        ],);
        expect(printed.lines,).toEqual([
          'REATTEMPT tabby queued: 1 more cache record than it had, so the next attempt starts further along',
          'STALLED mittens: its 0 cache records are what it started with, so a further attempt in this '
            + 'invocation would repeat it',
          'STALLED tabby: its 1 cache record is what it started with, so a further attempt in this '
            + 'invocation would repeat it',
        ],);
      },
    },),
    it({
      name: 'QUEUES A SECOND ATTEMPT FOR AN ENTRY THAT CACHED TWO SLICES and says so in the plural, then drops it '
        + 'when the second attempt caches nothing more',
      fn: async (ctx) => {
        resetRunSpend();
        using printed = relayingConsoleLog({ sinon: ctx.sinon, },);
        await using scratch = await scratchDir({ prefix: 'corpus-pass-queue-', },);
        /**
         Slice cache the attempts write into.
         */
        const sliceCacheDir = join(
          scratch.path,
          'slice-cache',
        );
        /**
         Where the entry's slices are cached.
         */
        const entryDir = join(
          sliceCacheDir,
          'tabby',
        );
        /**
         Attempts made.
         */
        let made = 0;

        await runPassQueue({
          pending: [pairOf({ id: 'tabby', },),],
          attempts: new Map(),
          attemptsPath: join(
            scratch.path,
            'attempts.json',
          ),
          sliceCacheDir,
          pipelineDigest: GENERATION,
          dryStop: createPassDryStop(),
          softBudgetMs: 1_000,
          spendCeilingUsd: 20,
          start: 0,
          now: clockReading({ instants: [0,], },),
          settle: async function settle() {
            made += 1;
            if (made === 1) {
              await mkdir(
                entryDir,
                { recursive: true, },
              );
              await writeFile(
                join(
                  entryDir,
                  '0.json',
                ),
                '{}',
              );
              await writeFile(
                join(
                  entryDir,
                  '1.json',
                ),
                '{}',
              );
            }
            return { kind: 'resumable-failure', };
          },
        },);

        expect(made,).toBe(2,);
        expect(printed.lines,).toEqual([
          'REATTEMPT tabby queued: 2 more cache records than it had, so the next attempt starts further along',
          'STALLED tabby: its 2 cache records are what it started with, so a further attempt in this '
            + 'invocation would repeat it',
        ],);
      },
    },),
    it({
      name: 'STOPS BEFORE THE NEXT ENTRY ONCE EVERY PROVIDER READ DRY, says so in one line naming the cause, and queues no '
        + 'second attempt, so the run ships what it settled instead of starting an entry no provider can serve',
      fn: async (ctx) => {
        resetRunSpend();
        using printed = relayingConsoleLog({ sinon: ctx.sinon, },);
        await using scratch = await scratchDir({ prefix: 'corpus-pass-queue-', },);
        /**
         Gate between the run and its provider, as the pass builds it.
         */
        const dryStop = createPassDryStop();
        /**
         Client whose every call meets the dry router.
         */
        const rawClient: ModelClient = {
          chatText: refuseDry,
          chatJson: refuseDry,
          quotas: async () => {
            throw new Error('quotas unused',);
          },
        };
        /**
         Same client behind the gate.
         */
        const dryClient = dryStop.guard({ client: rawClient, },);
        /**
         Entries attempted.
         */
        const made: string[] = [];

        await runPassQueue({
          pending: [
            pairOf({ id: 'tabby', },),
            pairOf({ id: 'mittens', },),
          ],
          attempts: new Map(),
          attemptsPath: join(
            scratch.path,
            'attempts.json',
          ),
          sliceCacheDir: join(
            scratch.path,
            'slice-cache',
          ),
          pipelineDigest: GENERATION,
          dryStop,
          softBudgetMs: HANG_STOP_MS,
          spendCeilingUsd: 20,
          start: 0,
          now: clockReading({ instants: [0,], },),
          settle: async function settle({ entry, },) {
            made.push(entry.id,);
            await dryCallOf({ client: dryClient, signal: dryStop.signal, },);
            return { kind: 'stopped', };
          },
        },);

        expect(made,).toEqual(['tabby',],);
        expect(printed.lines.filter(function isStop(line,): boolean {
          return line.startsWith('STOP ',);
        },),).toHaveLength(1,);
        expect(printed.lines.some(function isReattempt(line,): boolean {
          return line.startsWith('REATTEMPT',);
        },),).toBe(false,);
        expect(printed.lines.at(-1,),).toContain('every provider',);
        expect(printed.lines.at(-1,),).toContain('meters read synthetic dry, openrouter dry',);
      },
    },),
    it({
      name: 'PARKS UNTIL NEURALWATT\'S RETURN AND RESUMES THE INTERRUPTED ENTRY when every provider read dry and the '
        + 'return is 301 s away, inside what the pass has left: no STOP line, no INCOMPLETE, REATTEMPT or STALLED '
        + 'line for the interrupted attempt, and the entries behind it keep their order',
      fn: async (ctx) => {
        resetRunSpend();
        using printed = relayingConsoleLog({ sinon: ctx.sinon, },);
        await using scratch = await scratchDir({ prefix: 'corpus-pass-queue-', },);
        /**
         Milliseconds the park waited for.
         */
        const waits: number[] = [];
        const dryStop = createPassDryStop({
          passLeftMs: () => 3_600_000,
          // A frozen gate clock: the park waits the whole 301 s the refusal named.
          now: () => 1_000,
          waitMs: async function waitMs({ ms, },): Promise<void> {
            waits.push(ms,);
          },
        },);
        /**
         Client whose every call meets the dry router, naming NeuralWatt's return 301 s away.
         */
        async function refuseDryNamingReturn(): Promise<never> {
          throw new EveryProviderDryError({
            measured: 'meters read every provider dry',
            neuralwatt: { kind: 'at', msOfDay: 69_308_000, },
            neuralwattReturnsInMs: 301_000,
          },);
        }
        const dryClient = dryStop.guard<ModelClient>({
          client: {
            chatText: refuseDryNamingReturn,
            chatJson: refuseDryNamingReturn,
            quotas: async () => {
              throw new Error('quotas unused',);
            },
          },
        },);
        /**
         Entries attempted.
         */
        const made: string[] = [];
        await runPassQueue({
          pending: [pairOf({ id: 'tabby', },), pairOf({ id: 'mittens', },),],
          attempts: new Map(),
          attemptsPath: join(
            scratch.path,
            'attempts.json',
          ),
          sliceCacheDir: join(
            scratch.path,
            'slice-cache',
          ),
          pipelineDigest: GENERATION,
          dryStop,
          softBudgetMs: HANG_STOP_MS,
          spendCeilingUsd: 20,
          start: 0,
          now: clockReading({ instants: [0,], },),
          settle: async function settle({ entry, },) {
            made.push(entry.id,);
            if (made.length === 1) {
              await dryCallOf({ client: dryClient, signal: dryStop.signal, },);
              return { kind: 'stopped', };
            }
            return { kind: 'settled', };
          },
        },);

        expect(made,).toEqual(['tabby', 'tabby', 'mittens',],);
        expect(waits,).toEqual([301_000,],);
        expect(dryStop.tripped(),).toBe(false,);
        expect(printed.lines.some(function isStopOrVerdict(line,): boolean {
          return line.startsWith('STOP ',) || line.startsWith('REATTEMPT',) || line.startsWith('STALLED',)
            || line.startsWith('INCOMPLETE',);
        },),).toBe(false,);
      },
    },),
    it({
      name: 'PARKS AND REQUEUES THE ENTRY A WRITING BENCH ERROR ENDED when the error carries NeuralWatt\'s return '
        + 'inside what the pass has left: the entry is attempted again after the wait with no verdict printed for '
        + 'the first attempt, and an error without a return ends the entry as before',
      fn: async (ctx) => {
        resetRunSpend();
        using printed = relayingConsoleLog({ sinon: ctx.sinon, },);
        await using scratch = await scratchDir({ prefix: 'corpus-pass-queue-', },);
        /**
         Milliseconds the park waited for.
         */
        const waits: number[] = [];
        const dryStop = createPassDryStop({
          passLeftMs: () => 3_600_000,
          now: () => 1_000,
          waitMs: async function waitMs({ ms, },): Promise<void> {
            waits.push(ms,);
          },
        },);
        /**
         Entries attempted.
         */
        const made: string[] = [];
        await runPassQueue({
          pending: [pairOf({ id: 'tabby', },), pairOf({ id: 'mittens', },),],
          attempts: new Map(),
          attemptsPath: join(
            scratch.path,
            'attempts.json',
          ),
          sliceCacheDir: join(
            scratch.path,
            'slice-cache',
          ),
          pipelineDigest: GENERATION,
          dryStop,
          softBudgetMs: HANG_STOP_MS,
          spendCeilingUsd: 20,
          start: 0,
          now: clockReading({ instants: [0,], },),
          settle: async function settle({ entry, },) {
            made.push(entry.id,);
            if (made.length === 1) {
              dryStop.trip({
                error: new WritingBenchUnreachableError({
                  phase: 'lanes',
                  clauses: ['editors 0 of 3 reachable, floor 2',],
                  neuralwattReturnsInMs: 301_000,
                  neuralwattReturnsAt: '19:15:08 UTC',
                  neuralwattDryness: { kind: 'spent-day', dayStart: '2026-10-22T19:15:08.000Z', returnsInMs: 301_000, },
                },),
              },);
              return { kind: 'resumable-failure', };
            }
            return { kind: 'settled', };
          },
        },);

        expect(made,).toEqual(['tabby', 'tabby', 'mittens',],);
        expect(waits,).toEqual([301_000,],);
        expect(dryStop.tripped(),).toBe(false,);
        expect(printed.lines.some(function isStopOrVerdict(line,): boolean {
          return line.startsWith('STOP ',) || line.startsWith('REATTEMPT',) || line.startsWith('STALLED',);
        },),).toBe(false,);
      },
    },),
    it({
      name: 'PRINTS THE STOP LINE EVEN WHEN THE ENTRY THAT MET THE DRY ROUTER WAS THE LAST ONE, once, so the cause is '
        + 'on the log however the queue ended',
      fn: async (ctx) => {
        resetRunSpend();
        using printed = relayingConsoleLog({ sinon: ctx.sinon, },);
        await using scratch = await scratchDir({ prefix: 'corpus-pass-queue-', },);
        /**
         Gate between the run and its provider, as the pass builds it.
         */
        const dryStop = createPassDryStop();
        await runPassQueue({
          pending: [pairOf({ id: 'tabby', },),],
          attempts: new Map(),
          attemptsPath: join(
            scratch.path,
            'attempts.json',
          ),
          sliceCacheDir: join(
            scratch.path,
            'slice-cache',
          ),
          pipelineDigest: GENERATION,
          dryStop,
          softBudgetMs: HANG_STOP_MS,
          spendCeilingUsd: 20,
          start: 0,
          now: clockReading({ instants: [0,], },),
          settle: async function settle() {
            dryStop.trip({ error: new EveryProviderDryError({ measured: 'meters read synthetic dry', },), },);
            return { kind: 'stopped', };
          },
        },);

        expect(printed.lines.filter(function isStop(line,): boolean {
          return line.startsWith('STOP ',);
        },),).toHaveLength(1,);
        expect(printed.lines.at(-1,),).toContain('meters read synthetic dry',);
      },
    },),
    it({
      name: 'PRINTS NO REATTEMPT OR STALLED LINE FOR AN ENTRY THE DRY ROUTER STOPPED even when a layer filed its failure '
        + 'as a resumable error and it cached a record, and queues no second attempt: the stop line follows alone',
      fn: async (ctx) => {
        resetRunSpend();
        using printed = relayingConsoleLog({ sinon: ctx.sinon, },);
        await using scratch = await scratchDir({ prefix: 'corpus-pass-queue-', },);
        /**
         Slice cache the attempts write into.
         */
        const sliceCacheDir = join(
          scratch.path,
          'slice-cache',
        );
        /**
         Gate between the run and its provider, as the pass builds it.
         */
        const dryStop = createPassDryStop();
        /**
         Entries attempted.
         */
        const made: string[] = [];
        await runPassQueue({
          pending: [
            pairOf({ id: 'tabby', },),
            pairOf({ id: 'mittens', },),
          ],
          attempts: new Map(),
          attemptsPath: join(
            scratch.path,
            'attempts.json',
          ),
          sliceCacheDir,
          pipelineDigest: GENERATION,
          dryStop,
          softBudgetMs: HANG_STOP_MS,
          spendCeilingUsd: 20,
          start: 0,
          now: clockReading({ instants: [0,], },),
          settle: async function settle({ entry, },) {
            made.push(entry.id,);
            await mkdir(
              join(
                sliceCacheDir,
                entry.id,
              ),
              { recursive: true, },
            );
            await writeFile(
              join(
                sliceCacheDir,
                entry.id,
                '0.json',
              ),
              '{}',
            );
            dryStop.trip({ error: new EveryProviderDryError({ measured: 'meters read synthetic dry', },), },);
            return { kind: 'resumable-failure', };
          },
        },);

        expect(made,).toEqual(['tabby',],);
        expect(printed.lines.filter(function isReattemptOrStalled(line,): boolean {
          return line.startsWith('REATTEMPT',) || line.startsWith('STALLED',);
        },),).toEqual([],);
        expect(printed.lines.filter(function isStop(line,): boolean {
          return line.startsWith('STOP ',);
        },),).toHaveLength(1,);
      },
    },),
  ],
},);
