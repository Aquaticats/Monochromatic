/**
 Tests for `corpus-pass` as an operator meets it: the built command in a
 child process, read for what it prints and how it exits.

 NO CASE RAN THIS COMMAND BEFORE. Every child runs through
 `runBuiltWithoutKeys`, which removes every variable whose name ends in
 `_API_KEY` and every variable whose name starts `TRANSLATION_REPAIR_` from the
 child's environment before the case adds the locations it needs, so a case
 can neither spend nor read the operator's runs directory, lookup cache or
 corpus clone. The runs directory and the lookup cache are scratch
 directories and the corpus clone is a throwaway repository of invented pages.

 WHAT THE COMMAND DOES NOT LET A CASE BUILD: a run to its end. The run client
 is built before the `--plan` return and before the queue, whatever the corpus
 holds, and a child without a key stops there with a stated refusal. A child
 given a placeholder key would construct real provider clients, so no case
 does; the run's later stages are reached in the unit tests of the modules
 that hold them.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { readFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { lockRunsDir, } from '@monochromatic-dev/translation-repair-run';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import {
  ENTRIES,
  ENTRIES_WITH_A_GAP,
  NO_KEY_REFUSAL,
  pinLine,
  PROVIDERS_OFF_LINES,
  REFUSED_AS_STATED,
  runPass,
  startLine,
  WRITER_GRACE_BUILT_IN,
} from './corpus-pass-built.test-fixture.ts';
import { makeCorpusPassClone, } from './corpus-pass-clone.test-fixture.ts';

/**
 What the lock file a live pass wrote records of its holder.

 @param text - the lock file's text

 @returns The holder's process id and the instant it took the lock

 @throws {@link Error} when the text is not a lock holder, which the case that wrote it rules out

 @example
 ```ts
 const holder = lockHolderOf({ text: '{"pid":7,"startedAt":"2026-10-06T05:06:50.929Z"}', },);
 ```
 */
function lockHolderOf({ text, }: { readonly text: string; },): {
  readonly pid: number;
  readonly startedAt: string;
} {
  /**
   The text as JSON, of unknown shape.
   */
  const parsed: unknown = JSON.parse(text,);
  if (((typeof parsed) !== 'object') || (parsed === null))
    throw new Error('the lock file does not record a holder',);

  /**
   Process id the holder recorded.
   */
  const pid: unknown = Reflect.get(
    parsed,
    'pid',
  );

  /**
   Instant the holder recorded.
   */
  const startedAt: unknown = Reflect.get(
    parsed,
    'startedAt',
  );
  if (((typeof pid) === 'number') && ((typeof startedAt) === 'string')) {
    return {
      pid,
      startedAt,
    };
  }
  throw new Error('the lock file does not record a holder',);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: 'corpus-pass as built',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'REFUSES A RUN WITH NO PROVIDER KEY at exit 6 on the run client, after the START line and the '
            + 'one note a built-in launch prints, and writes nothing to stderr but the refusal',
          fn: async () => {
            await using clone = await makeCorpusPassClone({ entries: ENTRIES, },);
            await using runs = await scratchDir({ prefix: 'corpus-pass-runs-', },);
            await using cache = await scratchDir({ prefix: 'corpus-pass-cache-', },);

            /**
             What the command wrote.
             */
            const run = await runPass({
              args: [],
              runsDir: runs.path,
              cacheDir: cache.path,
              clone,
              settings: {},
            },);

            expect(run.code,).toBe(REFUSED_AS_STATED,);
            expect(run.stdout,).toBe([
              await startLine({
                pending: 2,
                hardMs: 25_200_000,
              },),
              WRITER_GRACE_BUILT_IN,
              pinLine({ clone, },),
              ...PROVIDERS_OFF_LINES,
              '',
            ].join('\n',),);
            expect(run.stderr,).toBe(NO_KEY_REFUSAL,);
          },
        },),

        it({
          name: 'NAMES THE ENTRIES IT WAS ASKED FOR, THE PAIR IT COULD NOT READ AND EVERY LIMIT A LAUNCH CHANGED, in '
            + 'the order the pass prints them, before it refuses for want of a key',
          fn: async () => {
            await using clone = await makeCorpusPassClone({ entries: ENTRIES_WITH_A_GAP, },);
            await using runs = await scratchDir({ prefix: 'corpus-pass-runs-', },);
            await using cache = await scratchDir({ prefix: 'corpus-pass-cache-', },);

            /**
             What the command wrote with a cap, a spend ceiling and both windows overridden.
             */
            const run = await runPass({
              args: [
                '--only',
                'tabby,mittens',
                '--plan',
              ],
              runsDir: runs.path,
              cacheDir: cache.path,
              clone,
              settings: {
                TRANSLATION_REPAIR_HARD_CAP_MINUTES: '1.5',
                TRANSLATION_REPAIR_RUN_SPEND_CEILING_USD: '5',
                TRANSLATION_REPAIR_STRAGGLER_GRACE_MS: '90000',
                TRANSLATION_REPAIR_WRITER_GRACE_MS: '60000',
              },
            },);

            expect(run.code,).toBe(REFUSED_AS_STATED,);
            expect(run.stdout,).toBe([
              'ONLY mittens,tabby (chosen by hand in place of every pending pair at the pin; those still pending '
                + 'run in the pass\'s own order; run this into a throwaway TRANSLATION_REPAIR_RUNS_DIR so a hand-picked '
                + 'entry never enters a pool later draws treat as natural accumulation)',
              [
                `INCOMPLETE mittens: target page absent at the pin (corpus read failed for ${clone.commitSha}:`,
                'people/mittens/page.en.md (missing-object); the commit has no such path: ',
                'check the path, or pin a commit that has it.)',
              ].join('',),
              await startLine({
                pending: 1,
                hardMs: 90_000,
              },),
              'CAP OVERRIDDEN by TRANSLATION_REPAIR_HARD_CAP_MINUTES: entries run under 1.5 minutes rather '
                + 'than the built-in 420',
              'SPEND CEILING OVERRIDDEN by TRANSLATION_REPAIR_RUN_SPEND_CEILING_USD: new entries stop once '
                + 'this run has spent 5 USD on any one of openrouter, deepseek, xiaomi rather than the built-in 20',
              'STRAGGLER GRACE OVERRIDDEN by TRANSLATION_REPAIR_STRAGGLER_GRACE_MS: rounds abandon stragglers '
                + '90000ms after quorum rather than the built-in 120000ms',
              'WRITER GRACE OVERRIDDEN by TRANSLATION_REPAIR_WRITER_GRACE_MS: writer rounds (editor, refiner, '
                + 'translate, produceConsolidations, archive-block-review) abandon stragglers 60000ms after quorum rather than the '
                + '90000ms every other round waits',
              pinLine({ clone, },),
              'CAP TOO TIGHT: an attempt runs 90000ms, which is not longer than the 460000ms one model '
                + 'exchange is allowed (360000ms by default, 460000ms for a Claude exchange). Attempts are cut before '
                + 'an exchange can return, so no slice caches, every '
                + 'attempt reports no progress, and the queue drops the entry as stalled after its second try. '
                + 'Raise the ceiling above one exchange to buy anything, or keep it here to exercise the stall '
                + 'path deliberately.',
              ...PROVIDERS_OFF_LINES,
              '',
            ].join('\n',),);
            expect(run.stderr,).toBe(NO_KEY_REFUSAL,);
          },
        },),

        it({
          name: 'REFUSES A FLAG IT DOES NOT READ with the usage line at exit 6 and prints nothing to stdout',
          fn: async () => {
            await using clone = await makeCorpusPassClone({ entries: ENTRIES, },);
            await using runs = await scratchDir({ prefix: 'corpus-pass-runs-', },);
            await using cache = await scratchDir({ prefix: 'corpus-pass-cache-', },);

            /**
             What the command wrote for a flag it does not declare.
             */
            const run = await runPass({
              args: ['--bogus',],
              runsDir: runs.path,
              cacheDir: cache.path,
              clone,
              settings: {},
            },);

            expect(run.code,).toBe(REFUSED_AS_STATED,);
            expect(run.stdout,).toBe('',);
            expect(run.stderr,).toBe(
              'corpus-pass: --bogus is not a flag this command reads. Usage: corpus-pass [--only <entry ids>] '
                + '[--require-providers <providers of neuralwatt, synthetic, bedrock, hyper, openrouter>] [--plan]\n',
            );
          },
        },),

        it({
          name: 'REFUSES AN ENTRY THE CORPUS DOES NOT HOLD at exit 6, after naming the entries chosen by hand',
          fn: async () => {
            await using clone = await makeCorpusPassClone({ entries: ENTRIES, },);
            await using runs = await scratchDir({ prefix: 'corpus-pass-runs-', },);
            await using cache = await scratchDir({ prefix: 'corpus-pass-cache-', },);

            /**
             What the command wrote for an entry no page of the corpus belongs to.
             */
            const run = await runPass({
              args: [
                '--only',
                'nobody',
              ],
              runsDir: runs.path,
              cacheDir: cache.path,
              clone,
              settings: {},
            },);

            expect(run.code,).toBe(REFUSED_AS_STATED,);
            expect(run.stdout,).toBe(
              'ONLY nobody (chosen by hand in place of every pending pair at the pin; if still pending it runs in '
                + 'the pass\'s own order; run this into a throwaway TRANSLATION_REPAIR_RUNS_DIR so a hand-picked '
                + 'entry never enters a pool later draws treat as natural accumulation)\n',
            );
            expect(run.stderr,).toBe(
              'corpus-pass: --only asks for "nobody", which the corpus at the pin does not hold\n',
            );
          },
        },),

        it({
          name: 'REFUSES A RUNS DIRECTORY A LIVE PASS HOLDS at exit 6 with the holder named, before it prints or reads anything',
          fn: async () => {
            await using clone = await makeCorpusPassClone({ entries: ENTRIES, },);
            await using runs = await scratchDir({ prefix: 'corpus-pass-runs-', },);
            await using cache = await scratchDir({ prefix: 'corpus-pass-cache-', },);

            // This process holds the lock, so the holder the child meets is alive.
            await using _held = await lockRunsDir({ runsDir: runs.path, },);

            /**
             What the lock file this process wrote records.
             */
            const holder = lockHolderOf({ text: await readFile(
              join(
                runs.path,
                'pass.lock',
              ),
              'utf8',
            ), },);

            /**
             What the command wrote against the held directory.
             */
            const run = await runPass({
              args: [],
              runsDir: runs.path,
              cacheDir: cache.path,
              clone,
              settings: {},
            },);

            expect(run.code,).toBe(REFUSED_AS_STATED,);
            expect(run.stdout,).toBe('',);
            expect(run.stderr,).toBe(
              `corpus-pass: Another pass is running in ${runs.path}.\n`
                + `  process ${String(holder.pid,)}, since ${holder.startedAt}\n`
                + '\n'
                + 'Two passes sharing one runs directory do not conflict loudly. They\n'
                + 'overwrite each other\'s attempt counts, delete each other\'s cached\n'
                + 'slices whenever their pipelines differ, and the later write of any\n'
                + 'entry simply replaces the earlier one. Every one of those looks like\n'
                + 'ordinary output.\n'
                + '\n'
                + 'Point this run at another directory with TRANSLATION_REPAIR_RUNS_DIR,\n'
                + 'or stop the other pass. A lock whose process is gone is taken over\n'
                + 'automatically. It names a process running now that started when the lock was taken, so the '
                + 'holder is alive.\n',
            );
          },
        },),

        it({
          name: 'REFUSES A REQUIRED PROVIDER NO KEY SERVES at exit 6 by name, after the START line and the notes',
          fn: async () => {
            await using clone = await makeCorpusPassClone({ entries: ENTRIES, },);
            await using runs = await scratchDir({ prefix: 'corpus-pass-runs-', },);
            await using cache = await scratchDir({ prefix: 'corpus-pass-cache-', },);

            /**
             What the command wrote when a provider is required and unkeyed.
             */
            const run = await runPass({
              args: [
                '--require-providers',
                'hyper',
              ],
              runsDir: runs.path,
              cacheDir: cache.path,
              clone,
              settings: {},
            },);

            expect(run.code,).toBe(REFUSED_AS_STATED,);
            expect(run.stdout,).toBe([
              await startLine({
                pending: 2,
                hardMs: 25_200_000,
              },),
              WRITER_GRACE_BUILT_IN,
              pinLine({ clone, },),
              '',
            ].join('\n',),);
            expect(run.stderr,).toBe('corpus-pass: required provider hyper is not ready: key missing\n',);
          },
        },),

        it({
          name: 'REFUSES A REQUIRED PROVIDER IT DOES NOT KNOW at exit 6, naming the value and the four it accepts',
          fn: async () => {
            await using clone = await makeCorpusPassClone({ entries: ENTRIES, },);
            await using runs = await scratchDir({ prefix: 'corpus-pass-runs-', },);
            await using cache = await scratchDir({ prefix: 'corpus-pass-cache-', },);

            /**
             What the command wrote when the flag names a provider that does not exist.
             */
            const run = await runPass({
              args: [
                '--require-providers',
                'foo',
              ],
              runsDir: runs.path,
              cacheDir: cache.path,
              clone,
              settings: {},
            },);

            expect(run.code,).toBe(REFUSED_AS_STATED,);
            expect(run.stdout,).toBe([
              await startLine({
                pending: 2,
                hardMs: 25_200_000,
              },),
              WRITER_GRACE_BUILT_IN,
              pinLine({ clone, },),
              '',
            ].join('\n',),);
            expect(run.stderr,).toBe(
              'corpus-pass: --require-providers accepts only neuralwatt, synthetic, bedrock, hyper, openrouter, and "foo" '
                + 'is none of them\n',
            );
          },
        },),

        it({
          name: 'REFUSES AN UNREADABLE STRAGGLER WINDOW at exit 6 with nothing on stdout, before it takes the lock',
          fn: async () => {
            await using clone = await makeCorpusPassClone({ entries: ENTRIES, },);
            await using runs = await scratchDir({ prefix: 'corpus-pass-runs-', },);
            await using cache = await scratchDir({ prefix: 'corpus-pass-cache-', },);

            /**
             What the command wrote for a window that is not a number.
             */
            const run = await runPass({
              args: [],
              runsDir: runs.path,
              cacheDir: cache.path,
              clone,
              settings: { TRANSLATION_REPAIR_STRAGGLER_GRACE_MS: 'abc', },
            },);

            expect(run.code,).toBe(REFUSED_AS_STATED,);
            expect(run.stdout,).toBe('',);
            expect(run.stderr,).toBe(
              'corpus-pass: TRANSLATION_REPAIR_STRAGGLER_GRACE_MS must be a whole number of milliseconds from 1 '
                + 'to 2147483647, and "abc" is not; leave it unset to run under the built-in window\n',
            );
            /**
             Where the lock would stand had the pass claimed the directory.
             */
            const lockPath = join(
              runs.path,
              'pass.lock',
            );
            await expect(readFile(
              lockPath,
              'utf8',
            ),).rejects.toThrow('ENOENT',);
          },
        },),
      ],
    },),
  ],
},);
