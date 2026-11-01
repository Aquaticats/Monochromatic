/**
 Tests for the damage draw over a runs directory, from the pool it reads to
 the sheet pair it writes.

 THE SHEET IS BLIND AND THE MANIFEST IS NOT: the probe's verdict labels each
 item in the manifest and appears nowhere on the sheet. A pool with nothing to
 draw is refused before any client is built, and a region no prober could hear
 is never recorded as one the probe found silent. The roster is scripted, so
 nothing here reaches a provider.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  mkdir,
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

import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary';
import { sampleDamage, } from '../../dist/final/node/index.mjs';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import {
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { divertingConsoleLog, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/console-log-capture.test-fixture.ts';
import {
  modelReplyScriptedClient,
  type ScriptedModelReply,
} from '@monochromatic-dev/translation-repair-provider/ts/corpus-run/model-reply-scripted-client.test-fixture.ts';
import { writeSettledV2, } from '@monochromatic-dev/translation-repair-artifact/ts/corpus-run/settled-v2-pool.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import {
  NO_POOL_POLICY,
  UNASKED_REPOSITORY,
} from '@monochromatic-dev/translation-repair-artifact-pool/ts/corpus-run/pool-policy.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Probers every case asks.
 */
const PROBERS = [
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
] as const;

/**
 Reply every prober gives that finds nothing wrong.
 */
const FINDS_NOTHING = JSON.stringify({
  checks: [
    {
      region: 1,
      verdict: 'no-introduced-defect-found',
      category: '',
      severity: '',
      evidence: '',
      omittedText: '',
      verdictReason: '',
    },
  ],
},);

/**
 Reply every prober gives that quotes the wording the repair lane added.
 */
const QUOTES_THE_ADDITION = JSON.stringify({
  checks: [
    {
      region: 1,
      verdict: 'introduced-defect',
      category: 'accuracy/addition',
      severity: 'minor',
      evidence: 'It purrs.',
      omittedText: '',
      verdictReason: 'The cat is given a purr the original does not state.',
    },
  ],
},);

/**
 Runs the draw over a runs directory with the probers scripted.

 @param runsDir - throwaway runs directory

 @param reply - what every prober does for the first region

 @param laterReply - what every prober does for each region after the first, the same as the first when absent

 @returns Lines printed and how many clients were opened

 @example
 ```ts
 const { lines, } = await drawWith({ runsDir, reply, sinon: ctx.sinon, },);
 ```
 */
async function drawWith(
  {
    runsDir,
    reply,
    laterReply = reply,
    sinon,
  }: {
    readonly runsDir: string;
    readonly reply: ScriptedModelReply;
    readonly laterReply?: ScriptedModelReply;
    readonly sinon: Parameters<typeof divertingConsoleLog>[0]['sinon'];
  },
): Promise<{
  readonly lines: readonly string[];
  readonly opened: number;
}> {
  /**
   Clients opened so far, one per region probed.
   */
  const opened = { count: 0, };
  using capture = divertingConsoleLog({ sinon, },);
  await sampleDamage({
    policy: NO_POOL_POLICY,
    repository: UNASKED_REPOSITORY,
    runsDir,
    seed: 'damage-round-one',
    openClient: function openScriptedClient() {
      opened.count += 1;
      return {
        ...modelReplyScriptedClient({
          replies: new Map(PROBERS.map(function replyOf(modelId,) {
            return [
              modelId,
              (opened.count === 1) ? reply : laterReply,
            ] as const;
          },),),
        },).client,
        providerDryness: function noDryness() {
          return Promise.reject(new Error('providerDryness unused by the probe',),);
        },
        providerMeterStates: function noMeterStates() {
          return Promise.reject(new Error('providerMeterStates unused by the probe',),);
        },
        providerHolds: function noHolds() {
          throw new Error('providerHolds unused by the probe',);
        },
      };
    },
    proberModelIds: PROBERS,
    perCallTimeoutMs: HANG_STOP_MS,
    l: tagged({ tag: 'damage-sample-test', },),
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);
  return {
    lines: [...capture.lines,],
    opened: opened.count,
  };
}

await describe({
  name: sampleDamage.name,
  // ONE AT A TIME: its cases divert the process-wide `console.log`.
  concurrency: 1,
  children: [
    it({
      name: 'WRITES the sheet and the manifest of a drawn region the probers found nothing wrong with, '
        + 'labelled silent in the manifest and carrying no claim on the sheet',
      fn: async (ctx) => {
        await using scratch = await scratchDir({ prefix: 'damage-run-', },);
        await writeSettledV2({
          runsDir: scratch.path,
          entryId: 'mittens',
          shippedSlices: 1,
        },);

        const {
          lines,
          opened,
        } = await drawWith({
          runsDir: scratch.path,
          reply: {
            kind: 'ok',
            text: FINDS_NOTHING,
          },
          sinon: ctx.sinon,
        },);

        expect(lines.slice(-4,),).toEqual([
          'DAMAGE pool 1 shipped region across both lanes, seed damage-round-one',
          'DAMAGE 0 shipped rows had no incumbent wording and are not drawn from',
          'DAMAGE mittens repair#0 probe=silent',
          `DAMAGE wrote 1 item to ${scratch.path}/damage-sheet.md`,
        ],);
        expect(opened,).toBe(1,);
        expect((await readdir(scratch.path,)).toSorted(),).toEqual([
          'artifacts',
          'damage-manifest.json',
          'damage-sheet.md',
        ],);
        expect(await readFile(
          join(
            scratch.path,
            'damage-manifest.json',
          ),
          'utf8',
        ),).toContain('"kind": "probe-silent"',);
      },
    },),
    it({
      name: 'LABELS a region flagged when the probers corroborate damage, and says so on its line',
      fn: async (ctx) => {
        await using scratch = await scratchDir({ prefix: 'damage-run-', },);
        await writeSettledV2({
          runsDir: scratch.path,
          entryId: 'mittens',
          shippedSlices: 1,
        },);

        const { lines, } = await drawWith({
          runsDir: scratch.path,
          reply: {
            kind: 'ok',
            text: QUOTES_THE_ADDITION,
          },
          sinon: ctx.sinon,
        },);

        expect(lines.slice(-2,),).toEqual([
          'DAMAGE mittens repair#0 probe=flagged',
          `DAMAGE wrote 1 item to ${scratch.path}/damage-sheet.md`,
        ],);
        expect(await readFile(
          join(
            scratch.path,
            'damage-manifest.json',
          ),
          'utf8',
        ),).toContain('"kind": "probe-flagged"',);
        expect(await readFile(
          join(
            scratch.path,
            'damage-sheet.md',
          ),
          'utf8',
        ),).not.toContain('The cat is given a purr the original does not state.',);
      },
    },),
    it({
      name: 'OPENS one client per drawn region and writes one item for each, in the plural',
      fn: async (ctx) => {
        await using scratch = await scratchDir({ prefix: 'damage-run-', },);
        await writeSettledV2({
          runsDir: scratch.path,
          entryId: 'mittens',
          shippedSlices: 2,
        },);

        const {
          lines,
          opened,
        } = await drawWith({
          runsDir: scratch.path,
          reply: {
            kind: 'ok',
            text: FINDS_NOTHING,
          },
          sinon: ctx.sinon,
        },);

        expect(opened,).toBe(2,);
        expect(lines.at(-1,),).toBe(`DAMAGE wrote 2 items to ${scratch.path}/damage-sheet.md`,);
        expect(lines[2],).toBe('DAMAGE pool 2 shipped regions across both lanes, seed damage-round-one',);
      },
    },),
    it({
      name: 'REFUSES as stated, before any client is opened, a pool with no region to draw, writing no sheet',
      fn: async (ctx) => {
        await using scratch = await scratchDir({ prefix: 'damage-run-', },);
        await writeSettledV2({
          runsDir: scratch.path,
          entryId: 'mittens',
          shippedSlices: 0,
        },);
        /**
         Clients opened, which a refusal must leave at none.
         */
        const opened = { count: 0, };

        const refusal = await rejectionOf(async function drawNothing(): Promise<void> {
          using _capture = divertingConsoleLog({ sinon: ctx.sinon, },);
          await sampleDamage({
            policy: NO_POOL_POLICY,
            repository: UNASKED_REPOSITORY,
            runsDir: scratch.path,
            seed: 'damage-round-one',
            openClient: function refuseToOpen() {
              opened.count += 1;
              throw new Error('a refused draw opened a client',);
            },
            proberModelIds: PROBERS,
            perCallTimeoutMs: HANG_STOP_MS,
            l: tagged({ tag: 'damage-sample-test', },),
            stageSettings: SCRIPTED_STAGE_SETTINGS,
          },);
        },);

        expect(refusal,).toBeInstanceOf(StatedRefusalError,);
        expect(String(refusal,),).toBe(
          'StatedRefusalError: the settled entries ship no replacement over an archive wording to draw from '
            + '(0 shipped rows had no incumbent wording and are not drawn from), so no sheet is written; a sheet '
            + 'with no item would be kept and refuse the next run',
        );
        expect(opened.count,).toBe(0,);
        expect(await readdir(scratch.path,),).toEqual(['artifacts',],);
      },
    },),
    it({
      name: 'REFUSES as stated, writing nothing, when a sheet of an earlier draw stands, after the regions are probed',
      fn: async (ctx) => {
        await using scratch = await scratchDir({ prefix: 'damage-run-', },);
        await writeSettledV2({
          runsDir: scratch.path,
          entryId: 'mittens',
          shippedSlices: 1,
        },);
        await mkdir(scratch.path,{ recursive: true, },);
        await writeFile(
          join(
            scratch.path,
            'damage-sheet.md',
          ),
          'graded',
          'utf8',
        );

        const refusal = await rejectionOf(async function drawOverAGradedSheet(): Promise<void> {
          await drawWith({
            runsDir: scratch.path,
            reply: {
              kind: 'ok',
              text: FINDS_NOTHING,
            },
            sinon: ctx.sinon,
          },);
        },);

        expect(refusal,).toBeInstanceOf(StatedRefusalError,);
        expect(String(refusal,),).toBe(
          `StatedRefusalError: ${join(scratch.path, 'damage-sheet.md',)} already exists; grade or move it before `
            + 'rerunning, since a rerun would replace a grader\'s work',
        );
        expect((await readdir(scratch.path,)).toSorted(),).toEqual([
          'artifacts',
          'damage-sheet.md',
        ],);
      },
    },),
    it({
      name: 'LEAVES a region no prober answered for off the sheet and says it was unheard, never recording it as silent',
      fn: async (ctx) => {
        await using scratch = await scratchDir({ prefix: 'damage-run-', },);
        await writeSettledV2({
          runsDir: scratch.path,
          entryId: 'mittens',
          shippedSlices: 2,
        },);

        const { lines, } = await drawWith({
          runsDir: scratch.path,
          reply: {
            kind: 'schema-mismatch',
            rawText: '',
            detail: 'scripted silence',
          },
          laterReply: {
            kind: 'ok',
            text: FINDS_NOTHING,
          },
          sinon: ctx.sinon,
        },);

        expect(lines.slice(-4,),).toEqual([
          'DAMAGE 0 shipped rows had no incumbent wording and are not drawn from',
          'DAMAGE mittens repair#1 probe=unheard, left off the sheet',
          'DAMAGE mittens repair#0 probe=silent',
          `DAMAGE wrote 1 item to ${scratch.path}/damage-sheet.md`,
        ],);
      },
    },),
    it({
      name: 'REFUSES as stated, writing no sheet, when no prober answered for any region drawn, never recording one as silent',
      fn: async (ctx) => {
        await using scratch = await scratchDir({ prefix: 'damage-run-', },);
        await writeSettledV2({
          runsDir: scratch.path,
          entryId: 'mittens',
          shippedSlices: 1,
        },);

        const refusal = await rejectionOf(async function drawUnheard(): Promise<void> {
          await drawWith({
            runsDir: scratch.path,
            reply: {
              kind: 'schema-mismatch',
              rawText: '',
              detail: 'scripted silence',
            },
            sinon: ctx.sinon,
          },);
        },);

        expect(refusal,).toBeInstanceOf(StatedRefusalError,);
        expect(String(refusal,),).toBe(
          'StatedRefusalError: no prober answered for any region drawn, so none of them can be called silent '
            + 'and no sheet is written',
        );
        expect(await readdir(scratch.path,),).toEqual(['artifacts',],);
      },
    },),
  ],
},);
