/**
 Tests for the collection of shipped regions across the settled artifacts of a
 runs directory.

 THE POOL IS A DENOMINATOR, and a draw that mistakes a directory for a pool
 reports a rate over nothing, so a runs directory with no artifacts directory
 is refused in words. The pool's census prints its lines through the console,
 which each case diverts.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary';
import { collectShippedRegions, } from '../../dist/final/node/index.mjs';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { divertingConsoleLog, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/console-log-capture.test-fixture.ts';
import {
  SHIPPED_ADDITION,
  writeSettledV2,
} from '@monochromatic-dev/translation-repair-artifact/ts/corpus-run/settled-v2-pool.test-fixture.ts';
import {
  NO_POOL_POLICY,
  UNASKED_REPOSITORY,
} from '@monochromatic-dev/translation-repair-artifact-pool/ts/corpus-run/pool-policy.test-fixture.ts';

await describe({
  name: collectShippedRegions.name,
  // ONE AT A TIME: its cases divert the process-wide `console.log`.
  concurrency: 1,
  children: [
    it({
      name: 'COLLECTS the region the repair lane shipped, with the archive wording and the original passage of its slice',
      fn: async (ctx) => {
        await using scratch = await scratchDir({ prefix: 'damage-pool-', },);
        await writeSettledV2({
          runsDir: scratch.path,
          entryId: 'mittens',
          shippedSlices: 1,
        },);
        using _capture = divertingConsoleLog({ sinon: ctx.sinon, },);

        const census = await collectShippedRegions({ runsDir: scratch.path, policy: NO_POOL_POLICY, repository: UNASKED_REPOSITORY, },);

        expect(census,).toEqual({
          regions: [
            {
              entryId: 'mittens',
              lane: 'repair',
              sliceIndex: 0,
              regionId: 'repair#0',
              sourceText: '## 第一节\n\n猫猫在窗台上睡觉。',
              incumbentText: '## Section one\n\nThe cat sleeps on the sill.',
              shippedText: `## Section one\n\nThe cat sleeps on the sill.${SHIPPED_ADDITION}`,
              pageRelation: census.regions[0]?.pageRelation,
            },
          ],
          filledWithoutIncumbent: 0,
        },);
      },
    },),
    it({
      name: 'COLLECTS no region from an entry whose lanes shipped nothing',
      fn: async (ctx) => {
        await using scratch = await scratchDir({ prefix: 'damage-pool-', },);
        await writeSettledV2({
          runsDir: scratch.path,
          entryId: 'mittens',
          shippedSlices: 0,
        },);
        using _capture = divertingConsoleLog({ sinon: ctx.sinon, },);

        expect(await collectShippedRegions({ runsDir: scratch.path, policy: NO_POOL_POLICY, repository: UNASKED_REPOSITORY, },),).toEqual({
          regions: [],
          filledWithoutIncumbent: 0,
        },);
      },
    },),
    it({
      name: 'REFUSES as stated, naming the missing directory, when the runs directory holds no artifacts directory',
      fn: async (ctx) => {
        await using scratch = await scratchDir({ prefix: 'damage-pool-', },);
        using _capture = divertingConsoleLog({ sinon: ctx.sinon, },);

        const refusal = await rejectionOf(async function collectAbsent(): Promise<void> {
          await collectShippedRegions({ runsDir: scratch.path, policy: NO_POOL_POLICY, repository: UNASKED_REPOSITORY, },);
        },);

        expect(refusal,).toBeInstanceOf(StatedRefusalError,);
        expect(String(refusal,),).toBe(
          `StatedRefusalError: there is no artifacts directory at ${join(scratch.path, 'artifacts',)}; name the `
            + 'runs directory a pass settled entries into with TRANSLATION_REPAIR_RUNS_DIR',
        );
      },
    },),
  ],
},);
