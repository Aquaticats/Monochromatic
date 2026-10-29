/**
 Tests for the draw over a runs directory, from the pool it reads to the
 summary line it prints.

 THE SUMMARY LINE IS THE RECORD of what a draw did: pool size, sample size,
 how many sampled and pooled issues carry no recorded repair, and where the
 three files went. A final draw over issues with no recorded repair is refused
 in the command's own words and writes nothing. The corpus is scripted and the
 runs directory is a throwaway, so nothing here reads a clone.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  readdir,
  readFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  digestPipeline,
  StatedRefusalError,
} from '@monochromatic-dev/translation-repair-glossary';
import { drawGradingSample, } from '../../dist/final/node/index.mjs';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import { builtDirectoryOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/built-package-directory.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { lineOf, } from '@monochromatic-dev/translation-repair-run/ts/corpus-run/command-line.test-fixture.ts';
import { divertingConsoleLog, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/console-log-capture.test-fixture.ts';
import { writeSettledArtifact, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/settled-v1-pool.test-fixture.ts';
import {
  NO_POOL_POLICY,
  UNASKED_REPOSITORY,
} from '@monochromatic-dev/translation-repair-artifact-pool/ts/corpus-run/pool-policy.test-fixture.ts';

/**
 Seed the gate draw's files are named after.
 */
const GATE_SEED = 'milestone-three-precision-round-three';

/**
 Corpus pin every case hands the draw, whose commit the files record.
 */
const PIN = {
  cloneDir: '/nowhere/cat-corpus',
  commitSha: 'b'.repeat(40,),
};

/**
 Reads the page of every entry as one short invented page.

 @returns Page

 @example
 ```ts
 const page = await readInventedPage();
 ```
 */
async function readInventedPage(): Promise<string> {
  return await Promise.resolve('猫猫在窗台上睡觉。\n',);
}

/**
 Runs a final draw that must refuse.

 @param runsDir - throwaway runs directory

 @returns What the draw refused with

 @example
 ```ts
 const refusal = await refusalOfFinalDraw({ runsDir, },);
 ```
 */
async function refusalOfFinalDraw(
  { runsDir, }: { readonly runsDir: string; },
): Promise<unknown> {
  return await rejectionOf(async function drawFinal(): Promise<void> {
    await drawGradingSample({
      policy: NO_POOL_POLICY,
      repository: UNASKED_REPOSITORY,
      line: lineOf({
        command: 'draw-sample',
        typed: ['--final',],
      },),
      runsDir,
      pin: PIN,
      readSource: readInventedPage,
    },);
  },);
}

/**
 Lines the pool's census prints before the band lines.

 @param entries - how many entries the pool holds, spelled as the census does

 @returns Two lines

 @example
 ```ts
 const lines = await censusLines({ entries: '1 entry', },);
 ```
 */
async function censusLines(
  { entries, }: { readonly entries: string; },
): Promise<readonly string[]> {
  /**
   Digest of the built output this suite runs the pool census from.
   */
  const { digest, } = await digestPipeline({
    dir: builtDirectoryOf({ packageEntry: import.meta.resolve('@monochromatic-dev/translation-repair-artifact-pool',), },),
  },);
  return [
    `POOL read by pipeline ${digest}`,
    `POOL ${entries} across 1 pipeline generation`,
  ];
}

await describe({
  name: drawGradingSample.name,
  // ONE AT A TIME: its cases divert the process-wide `console.log`.
  concurrency: 1,
  children: [
    it({
      name: 'PRINTS the pool, the bands and the summary of a preliminary draw, and writes the three scratch files',
      fn: async (ctx) => {
        await using scratch = await scratchDir({ prefix: 'draw-run-', },);
        await writeSettledArtifact({
          runsDir: scratch.path,
          entryId: 'mittens',
          issueIds: [
            'adjudicated/purr',
            'adjudicated/knead',
          ],
          repairRecorded: false,
        },);
        using capture = divertingConsoleLog({ sinon: ctx.sinon, },);

        await drawGradingSample({
          policy: NO_POOL_POLICY,
          repository: UNASKED_REPOSITORY,
          line: lineOf({
            command: 'draw-sample',
            typed: [],
          },),
          runsDir: scratch.path,
          pin: PIN,
          readSource: readInventedPage,
        },);

        expect(capture.lines,).toEqual([
          ...await censusLines({ entries: '1 entry', },),
          'POOL band=small entries=1 contributing=1 accepted=2 perEntry=mittens:2',
          'POOL band=medium entries=0 contributing=0 accepted=0 perEntry=',
          'POOL band=large entries=0 contributing=0 accepted=0 perEntry=',
          `SAMPLE final=false seed=${GATE_SEED}-preliminary pool=2 drawn=2 unrecordedRepairs=2 unrecordedInPool=2 `
            + `out=${join(scratch.path, `grading-sheet-${GATE_SEED}-preliminary.md`,)} `
            + `repairOut=${join(scratch.path, `repair-sheet-${GATE_SEED}-preliminary.md`,)} `
            + `manifest=${join(scratch.path, `sample-manifest-${GATE_SEED}-preliminary.json`,)}`,
        ],);
        expect((await readdir(scratch.path,)).toSorted(),).toEqual([
          'artifacts',
          `grading-sheet-${GATE_SEED}-preliminary.md`,
          `repair-sheet-${GATE_SEED}-preliminary.md`,
          `sample-manifest-${GATE_SEED}-preliminary.json`,
        ],);
      },
    },),
    it({
      name: 'RECORDS the pin\'s commit and the preliminary seed in the manifest it writes',
      fn: async (ctx) => {
        await using scratch = await scratchDir({ prefix: 'draw-run-', },);
        await writeSettledArtifact({
          runsDir: scratch.path,
          entryId: 'mittens',
          issueIds: ['adjudicated/purr',],
          repairRecorded: false,
        },);
        using _capture = divertingConsoleLog({ sinon: ctx.sinon, },);

        await drawGradingSample({
          policy: NO_POOL_POLICY,
          repository: UNASKED_REPOSITORY,
          line: lineOf({
            command: 'draw-sample',
            typed: [],
          },),
          runsDir: scratch.path,
          pin: PIN,
          readSource: readInventedPage,
        },);

        /**
         The manifest as written.
         */
        const manifest: unknown = JSON.parse(await readFile(
          join(
            scratch.path,
            `sample-manifest-${GATE_SEED}-preliminary.json`,
          ),
          'utf8',
        ),);
        expect(manifest,).toMatchObject({
          corpusSha: PIN.commitSha,
          seed: `${GATE_SEED}-preliminary`,
        },);
      },
    },),
    it({
      name: 'DRAWS a final sample under the gate seed once every sampled issue carries a recorded repair',
      fn: async (ctx) => {
        await using scratch = await scratchDir({ prefix: 'draw-run-', },);
        await writeSettledArtifact({
          runsDir: scratch.path,
          entryId: 'mittens',
          issueIds: ['adjudicated/purr',],
          repairRecorded: true,
        },);
        using capture = divertingConsoleLog({ sinon: ctx.sinon, },);

        await drawGradingSample({
          policy: NO_POOL_POLICY,
          repository: UNASKED_REPOSITORY,
          line: lineOf({
            command: 'draw-sample',
            typed: ['--final',],
          },),
          runsDir: scratch.path,
          pin: PIN,
          readSource: readInventedPage,
        },);

        expect(capture.lines.at(-1,),).toBe(
          `SAMPLE final=true seed=${GATE_SEED} pool=1 drawn=1 unrecordedRepairs=0 unrecordedInPool=0 `
            + `out=${join(scratch.path, `grading-sheet-${GATE_SEED}.md`,)} `
            + `repairOut=${join(scratch.path, `repair-sheet-${GATE_SEED}.md`,)} `
            + `manifest=${join(scratch.path, `sample-manifest-${GATE_SEED}.json`,)}`,
        );
        expect((await readdir(scratch.path,)).toSorted(),).toEqual([
          'artifacts',
          `grading-sheet-${GATE_SEED}.md`,
          `repair-sheet-${GATE_SEED}.md`,
          `sample-manifest-${GATE_SEED}.json`,
        ],);
      },
    },),
    it({
      name: 'REFUSES as stated a final draw whose sample carries issues with no recorded repair, writing nothing',
      fn: async (ctx) => {
        await using scratch = await scratchDir({ prefix: 'draw-run-', },);
        await writeSettledArtifact({
          runsDir: scratch.path,
          entryId: 'mittens',
          issueIds: ['adjudicated/purr',],
          repairRecorded: false,
        },);
        using capture = divertingConsoleLog({ sinon: ctx.sinon, },);

        const refusal = await refusalOfFinalDraw({ runsDir: scratch.path, },);

        expect(refusal,).toBeInstanceOf(StatedRefusalError,);
        expect(String(refusal,),).toBe(
          'UnmeasurableRepairError: refusing a final draw: 1 of 1 sampled issue carries no recorded repair, so repair '
            + 'quality cannot be measured over this sample. Those artifacts predate repair recording; move them '
            + 'aside and rerun the pass into a fresh artifacts directory.',
        );
        expect(capture.lines,).toEqual([
          ...await censusLines({ entries: '1 entry', },),
          'POOL band=small entries=1 contributing=1 accepted=1 perEntry=mittens:1',
          'POOL band=medium entries=0 contributing=0 accepted=0 perEntry=',
          'POOL band=large entries=0 contributing=0 accepted=0 perEntry=',
        ],);
        expect(await readdir(scratch.path,),).toEqual(['artifacts',],);
      },
    },),
    it({
      name: 'DRAWS the sample size from a pool larger than it, and counts the pool whole',
      fn: async (ctx) => {
        await using scratch = await scratchDir({ prefix: 'draw-run-', },);
        await writeSettledArtifact({
          runsDir: scratch.path,
          entryId: 'mittens',
          issueIds: Array.from(
            { length: 60, },
            function issueAt(
              _unused,
              index,
            ): string {
              return `adjudicated/purr-${String(index,)}`;
            },
          ),
          repairRecorded: false,
        },);
        using capture = divertingConsoleLog({ sinon: ctx.sinon, },);

        await drawGradingSample({
          policy: NO_POOL_POLICY,
          repository: UNASKED_REPOSITORY,
          line: lineOf({
            command: 'draw-sample',
            typed: [],
          },),
          runsDir: scratch.path,
          pin: PIN,
          readSource: readInventedPage,
        },);

        expect(capture.lines.at(-1,)?.startsWith(
          `SAMPLE final=false seed=${GATE_SEED}-preliminary pool=60 drawn=50 unrecordedRepairs=50 unrecordedInPool=60 `,
        ),).toBe(true,);
      },
    },),
    it({
      name: 'REFUSES as stated, naming the missing directory, when the runs directory holds no artifacts directory',
      fn: async (ctx) => {
        await using scratch = await scratchDir({ prefix: 'draw-run-', },);
        using _capture = divertingConsoleLog({ sinon: ctx.sinon, },);

        const refusal = await rejectionOf(async function drawAbsent(): Promise<void> {
          await drawGradingSample({
            policy: NO_POOL_POLICY,
            repository: UNASKED_REPOSITORY,
            line: lineOf({
              command: 'draw-sample',
              typed: [],
            },),
            runsDir: scratch.path,
            pin: PIN,
            readSource: readInventedPage,
          },);
        },);

        expect(refusal,).toBeInstanceOf(StatedRefusalError,);
      },
    },),
    it({
      name: 'REFUSES as stated a final draw over a pool that holds no accepted issue, writing no gate sheet an empty '
        + 'draw would then protect from the real one',
      fn: async (ctx) => {
        await using scratch = await scratchDir({ prefix: 'draw-run-', },);
        await writeSettledArtifact({
          runsDir: scratch.path,
          entryId: 'mittens',
          issueIds: [],
          repairRecorded: false,
        },);
        using _capture = divertingConsoleLog({ sinon: ctx.sinon, },);

        const refusal = await refusalOfFinalDraw({ runsDir: scratch.path, },);

        expect(refusal,).toBeInstanceOf(StatedRefusalError,);
        expect(String(refusal,),).toBe(
          'StatedRefusalError: refusing a final draw: the settled entries hold no accepted issue to sample, and a '
            + 'gate sheet with no item would be protected from overwrite and block the real draw. Settle more '
            + 'entries, or draw without --final to check the pool.',
        );
        expect(await readdir(scratch.path,),).toEqual(['artifacts',],);
      },
    },),
    it({
      name: 'DRAWS a preliminary sample of nothing from a pool that holds no accepted issue, since scratch is replaced',
      fn: async (ctx) => {
        await using scratch = await scratchDir({ prefix: 'draw-run-', },);
        await writeSettledArtifact({
          runsDir: scratch.path,
          entryId: 'mittens',
          issueIds: [],
          repairRecorded: false,
        },);
        using capture = divertingConsoleLog({ sinon: ctx.sinon, },);

        await drawGradingSample({
          policy: NO_POOL_POLICY,
          repository: UNASKED_REPOSITORY,
          line: lineOf({
            command: 'draw-sample',
            typed: [],
          },),
          runsDir: scratch.path,
          pin: PIN,
          readSource: readInventedPage,
        },);

        expect(capture.lines.at(-1,)?.startsWith(
          `SAMPLE final=false seed=${GATE_SEED}-preliminary pool=0 drawn=0 unrecordedRepairs=0 unrecordedInPool=0 `,
        ),).toBe(true,);
      },
    },),
  ],
},);
