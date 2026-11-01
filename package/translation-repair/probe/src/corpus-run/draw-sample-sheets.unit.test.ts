/**
 Tests for the three files one draw writes.

 THE FILES ARE A GRADER'S WORK IN PROGRESS, so what is pinned is when they
 land and when they do not: a final draw never replaces a final sheet, a final
 draw that fails part way leaves none of its files behind, and a preliminary
 draw is the labelled scratch it says it is. Each file's text is read whole.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  readdir,
  readFile,
  symlink,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { poolGeneration, } from '@monochromatic-dev/translation-repair-insertion';
import { GradedSheetExistsError, } from '@monochromatic-dev/translation-repair-glossary';
import { formatRepairSheet, } from '@monochromatic-dev/translation-repair-assembly';
import type { EligibleEntries, } from '@monochromatic-dev/translation-repair-artifact-pool';
import {
  DEFAULT_PRECISION_BAR,
  formatGradingSheet,
} from '@monochromatic-dev/translation-repair-house-rule';
import { buildSampleManifest, } from '@monochromatic-dev/translation-repair-lane-contest';
import { writeDrawSheets, } from '../../dist/final/node/index.mjs';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { bandedEntryOf, } from './banded-entry.test-fixture.ts';

/**
 Seed the gate sheets are named after.
 */
const GATE_SEED = 'milestone-three-precision-round-three';

/**
 Corpus commit the sheets record.
 */
const CORPUS_SHA = 'b'.repeat(40,);

/**
 Digest the pool's one entry recorded.
 */
const POOL_DIGEST = `sha256-tree-v1:${'c'.repeat(64,)}`;

/**
 Artifact file name of the pool's one entry.
 */
const NAMES = ['mittens.json',] as const;

/**
 Pool of one entry, as the pool's census would report it.
 */
const ELIGIBLE: EligibleEntries = {
  entryIds: ['mittens',],
  excludedIds: [],
  malformedIds: [],
  tipByEntry: new Map([[
    'mittens',
    'a'.repeat(40,),
  ],],),
  digestByEntry: new Map([[
    'mittens',
    POOL_DIGEST,
  ],],),
  selection: {
    kind: 'single-generation',
    digest: POOL_DIGEST,
  },
  report: [],
};

/**
 Sample every case writes.
 */
const SAMPLE = bandedEntryOf({
  id: 'mittens',
  band: 'small',
  count: 2,
},).candidates;

/**
 Text the first line of every preliminary sheet carries.
 */
const BANNER = '> PRELIMINARY draw over whatever has settled so far, for validating the sheets and the pool, NOT for '
  + 'final grading. It is drawn with a different seed from the gate sheet, so it is not a preview of the gate draw; '
  + 'individual items can still coincide, since a different seed reorders the pool rather than excluding anything '
  + 'from it. The final draw shifts again as the pool grows.\n\n';

/**
 Manifest of a draw under a seed.

 @param seed - seed the draw used

 @returns What the sheets and the manifest file are built from

 @example
 ```ts
 const manifest = manifestOf({ seed: GATE_SEED, },);
 ```
 */
function manifestOf(
  { seed, }: { readonly seed: string; },
): ReturnType<typeof buildSampleManifest> {
  return buildSampleManifest({
    sample: SAMPLE,
    seed,
    corpusSha: CORPUS_SHA,
    generation: poolGeneration({
      eligible: ELIGIBLE,
      names: NAMES,
    },),
  },);
}

await describe({
  name: writeDrawSheets.name,
  children: [
    it({
      name: 'WRITES the detection sheet, the repair sheet and the manifest of a preliminary draw, each under the '
        + 'banner that marks it scratch',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'draw-sheets-', },);
        const seed = `${GATE_SEED}-preliminary`;
        const manifest = manifestOf({ seed, },);
        const drawDigest = nonNullishOrThrow(manifest.drawDigest,);

        const paths = await writeDrawSheets({
          runsDir: scratch.path,
          isFinal: false,
          drawSeed: seed,
          sample: SAMPLE,
          eligible: ELIGIBLE,
          names: NAMES,
          corpusSha: CORPUS_SHA,
        },);

        expect(paths,).toEqual({
          outPath: join(scratch.path, `grading-sheet-${seed}.md`,),
          repairPath: join(scratch.path, `repair-sheet-${seed}.md`,),
          manifestPath: join(scratch.path, `sample-manifest-${seed}.json`,),
        },);
        expect(await readFile(
          paths.outPath,
          'utf8',
        ),).toBe(`${BANNER}${
          formatGradingSheet({
            sample: SAMPLE,
            seed,
            bar: DEFAULT_PRECISION_BAR,
            corpusSha: CORPUS_SHA,
            drawDigest,
          },)
        }`,);
        expect(await readFile(
          paths.repairPath,
          'utf8',
        ),).toBe(`${BANNER}${
          formatRepairSheet({
            sample: SAMPLE,
            seed,
            corpusSha: CORPUS_SHA,
            drawDigest,
          },)
        }`,);
        expect(await readFile(
          paths.manifestPath,
          'utf8',
        ),).toBe(`${JSON.stringify(
          manifest,
          undefined,
          2,
        )}\n`,);
      },
    },),
    it({
      name: 'REPLACES the files of an earlier preliminary draw without a word, since scratch is meant to be redrawn',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'draw-sheets-', },);
        const seed = `${GATE_SEED}-preliminary`;
        const draw = {
          runsDir: scratch.path,
          isFinal: false,
          drawSeed: seed,
          sample: SAMPLE,
          eligible: ELIGIBLE,
          names: NAMES,
          corpusSha: CORPUS_SHA,
        } as const;
        await writeDrawSheets(draw,);

        const again = await writeDrawSheets(draw,);

        expect((await readdir(scratch.path,)).toSorted(),).toEqual([
          `grading-sheet-${seed}.md`,
          `repair-sheet-${seed}.md`,
          `sample-manifest-${seed}.json`,
        ],);
        expect(again.outPath,).toBe(join(scratch.path, `grading-sheet-${seed}.md`,),);
      },
    },),
    it({
      name: 'WRITES the gate sheets of a final draw with no banner',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'draw-sheets-', },);
        const manifest = manifestOf({ seed: GATE_SEED, },);
        const drawDigest = nonNullishOrThrow(manifest.drawDigest,);

        const paths = await writeDrawSheets({
          runsDir: scratch.path,
          isFinal: true,
          drawSeed: GATE_SEED,
          sample: SAMPLE,
          eligible: ELIGIBLE,
          names: NAMES,
          corpusSha: CORPUS_SHA,
        },);

        expect(paths,).toEqual({
          outPath: join(scratch.path, `grading-sheet-${GATE_SEED}.md`,),
          repairPath: join(scratch.path, `repair-sheet-${GATE_SEED}.md`,),
          manifestPath: join(scratch.path, `sample-manifest-${GATE_SEED}.json`,),
        },);
        expect(await readFile(
          paths.outPath,
          'utf8',
        ),).toBe(formatGradingSheet({
          sample: SAMPLE,
          seed: GATE_SEED,
          bar: DEFAULT_PRECISION_BAR,
          corpusSha: CORPUS_SHA,
          drawDigest,
        },),);
        expect(await readFile(
          paths.repairPath,
          'utf8',
        ),).toBe(formatRepairSheet({
          sample: SAMPLE,
          seed: GATE_SEED,
          corpusSha: CORPUS_SHA,
          drawDigest,
        },),);
      },
    },),
    it({
      name: 'REFUSES a final draw while a final sheet is there, writing nothing',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'draw-sheets-', },);
        const draw = {
          runsDir: scratch.path,
          isFinal: true,
          drawSeed: GATE_SEED,
          sample: SAMPLE,
          eligible: ELIGIBLE,
          names: NAMES,
          corpusSha: CORPUS_SHA,
        } as const;
        await writeDrawSheets(draw,);

        const refusal = await rejectionOf(async function writeAgain(): Promise<void> {
          await writeDrawSheets(draw,);
        },);

        expect(refusal,).toBeInstanceOf(GradedSheetExistsError,);
        expect((await readdir(scratch.path,)).toSorted(),).toEqual([
          `grading-sheet-${GATE_SEED}.md`,
          `repair-sheet-${GATE_SEED}.md`,
          `sample-manifest-${GATE_SEED}.json`,
        ],);
      },
    },),
    it({
      name: 'REMOVES the files a final draw already wrote when a later one cannot land',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'draw-sheets-', },);
        // A link to nothing at the repair sheet's path reads as absent to the
        // existence check and refuses the exclusive create, so the detection
        // sheet lands and the repair sheet cannot.
        await symlink(
          join(scratch.path, 'nowhere',),
          join(scratch.path, `repair-sheet-${GATE_SEED}.md`,),
        );

        await rejectionOf(async function writeUnlandable(): Promise<void> {
          await writeDrawSheets({
            runsDir: scratch.path,
            isFinal: true,
            drawSeed: GATE_SEED,
            sample: SAMPLE,
            eligible: ELIGIBLE,
            names: NAMES,
            corpusSha: CORPUS_SHA,
          },);
        },);

        expect((await readdir(scratch.path,)).toSorted(),).toEqual([`repair-sheet-${GATE_SEED}.md`,],);
      },
    },),
  ],
},);
