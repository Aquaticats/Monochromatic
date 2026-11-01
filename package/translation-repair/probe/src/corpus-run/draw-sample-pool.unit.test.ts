/**
 Tests for the read of a runs directory into the pool a draw samples from.

 THE POOL IS A DENOMINATOR, so what is pinned is which files it reads, in what
 order and what it asks the corpus for: every admitted artifact once, by the
 page its entry names, and nothing a directory or a stray file adds. The pool's
 own census prints its lines through the console, which each case diverts.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  mkdir,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { digestPipeline, } from '@monochromatic-dev/translation-repair-glossary';
import { readDrawPool, } from '../../dist/final/node/index.mjs';
import { builtDirectoryOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/built-package-directory.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { divertingConsoleLog, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/console-log-capture.test-fixture.ts';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/rejection-of.test-fixture.ts';
import { writeSettledArtifact, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/settled-v1-pool.test-fixture.ts';
import {
  NO_POOL_POLICY,
  UNASKED_REPOSITORY,
} from '@monochromatic-dev/translation-repair-artifact-pool/ts/corpus-run/pool-policy.test-fixture.ts';

/**
 Clone and commit the draw reads each entry's source at, invented.
 */
const POOL_PIN = {
  cloneDir: '/cats/corpus/clone',
  commitSha: 'feedface'.repeat(5,),
} as const;

await describe({
  name: readDrawPool.name,
  // ONE AT A TIME: its cases divert the process-wide `console.log`.
  concurrency: 1,
  children: [
    it({
      name: 'READS every settled artifact in code point order, asking the corpus for the page of each entry once',
      fn: async (ctx) => {
        await using scratch = await scratchDir({ prefix: 'draw-pool-', },);
        await writeSettledArtifact({
          runsDir: scratch.path,
          entryId: 'tabby',
          issueIds: ['adjudicated/nap',],
          repairRecorded: false,
        },);
        await writeSettledArtifact({
          runsDir: scratch.path,
          entryId: 'mittens',
          issueIds: [
            'adjudicated/purr',
            'adjudicated/knead',
          ],
          repairRecorded: false,
        },);
        await mkdir(join(
          scratch.path,
          'artifacts',
          'backup.json',
        ),);

        /**
         Pages the corpus was asked for, in order.
         */
        const asked: string[] = [];
        using capture = divertingConsoleLog({ sinon: ctx.sinon, },);

        const pool = await readDrawPool({
          policy: NO_POOL_POLICY,
          repository: UNASKED_REPOSITORY,
          runsDir: scratch.path,
          pin: POOL_PIN,
          readSource: async function readPage({ relPath, },): Promise<string> {
            asked.push(relPath,);
            return await Promise.resolve('猫猫在窗台上睡觉。\n',);
          },
        },);
        const { digest, } = await digestPipeline({
          dir: builtDirectoryOf({ packageEntry: import.meta.resolve('@monochromatic-dev/translation-repair-artifact-pool',), },),
        },);

        expect(pool.names,).toEqual([
          'mittens.json',
          'tabby.json',
        ],);
        expect(pool.entries.map(function idOf(entry,): string {
          return entry.id;
        },),).toEqual([
          'mittens',
          'tabby',
        ],);
        expect(pool.pool.map(function issueOf(candidate,): string {
          return candidate.issueId;
        },),).toEqual([
          pool.entries[0]?.candidates[0]?.issueId,
          pool.entries[0]?.candidates[1]?.issueId,
          pool.entries[1]?.candidates[0]?.issueId,
        ],);
        expect(pool.pool.length,).toBe(3,);
        expect(pool.eligible.entryIds,).toEqual([
          'mittens',
          'tabby',
        ],);
        expect(asked.toSorted(),).toEqual([
          'people/mittens/page.md',
          'people/tabby/page.md',
        ],);
        expect(capture.lines,).toEqual([
          `POOL read by pipeline ${digest}`,
          'POOL 2 entries across 1 pipeline generation',
        ],);
      },
    },),
    it({
      name: 'REFUSES with the first entry\'s refusal in code point order when two entries are refused and the '
        + 'later one is refused first',
      fn: async (ctx) => {
        await using scratch = await scratchDir({ prefix: 'draw-pool-', },);
        await writeSettledArtifact({
          runsDir: scratch.path,
          entryId: 'tabby',
          issueIds: ['adjudicated/nap',],
          repairRecorded: false,
        },);
        await writeSettledArtifact({
          runsDir: scratch.path,
          entryId: 'mittens',
          issueIds: ['adjudicated/purr',],
          repairRecorded: false,
        },);

        /**
         Opened by the later entry's page read as it refuses, which the earlier
         entry's page read waits for.
         */
        const laterRefused = Promise.withResolvers<undefined>();
        using _capture = divertingConsoleLog({ sinon: ctx.sinon, },);

        /**
         What the pool read refused with.
         */
        const refusal = await rejectionOf({
          promise: readDrawPool({
            policy: NO_POOL_POLICY,
            repository: UNASKED_REPOSITORY,
            runsDir: scratch.path,
            pin: POOL_PIN,
            readSource: async function refusesTabbyFirst({ relPath, },): Promise<string> {
              if (relPath === 'people/tabby/page.md') {
                laterRefused.resolve(undefined,);
                throw new Error('the page of tabby was refused',);
              }
              await laterRefused.promise;
              throw new Error('the page of mittens was refused',);
            },
          },),
        },);
        expect(String(refusal,),).toBe('Error: the page of mittens was refused',);
      },
    },),
  ],
},);
