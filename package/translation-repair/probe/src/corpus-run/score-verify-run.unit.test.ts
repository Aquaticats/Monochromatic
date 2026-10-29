/**
 Tests for the report `score-verify` prints over a graded sheet and its
 manifest.

 Each case writes its own runs directory, hands it to `printVerifyScore` and
 reads the whole of what was printed, or the refusal that stopped it.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { writeFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary';
import { RunJsonUnreadableError, } from '@monochromatic-dev/translation-repair-shared-contract';
import { printVerifyScore, } from '../../dist/final/node/index.mjs';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { divertingConsoleLog, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/console-log-capture.test-fixture.ts';
import {
  catManifestText,
  catSheetText,
} from '@monochromatic-dev/translation-repair-lane-contest/ts/corpus-run/score-sheets.test-fixture.ts';

/**
 The note every report ends with.
 */
const LABEL_NOTE = 'NOTE read the labels as the drawing task meant them. On the verification '
  + 'sheet they name what a READER already believed, so the control line is the one that decides '
  + 'gating. On the damage sample they name what the PROBE said, so probe-flagged carries its '
  + 'precision and probe-silent carries its MISSES: a Y there is damage the probe did not see.';

/**
 Writes a sheet and a manifest under one basename.

 @param runsDir - directory the case owns

 @param basename - name the two files share

 @param sheet - sheet text, absent to leave the sheet unwritten

 @param manifest - manifest text, absent to leave the manifest unwritten

 @example
 ```ts
 await writeSheetFiles({ runsDir, basename: 'probe-verify', sheet: '', manifest: '{}', },);
 ```
 */
async function writeSheetFiles(
  {
    runsDir,
    basename,
    sheet,
    manifest,
  }: {
    readonly runsDir: string;
    readonly basename: string;
    readonly sheet?: string;
    readonly manifest?: string;
  },
): Promise<void> {
  if (sheet !== undefined)
    await writeFile(
      join(
        runsDir,
        `${basename}-sheet.md`,
      ),
      sheet,
      'utf8',
    );
  if (manifest !== undefined)
    await writeFile(
      join(
        runsDir,
        `${basename}-manifest.json`,
      ),
      manifest,
      'utf8',
    );
}

await describe({
  name: printVerifyScore.name,
  concurrency: 1,
  children: [
    it({
      name: 'PRINTS a line per set and the label note for a sheet whose manifest names two sets',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        await using scratch = await scratchDir({ prefix: 'score-verify-run-', },);
        await writeSheetFiles({
          runsDir: scratch.path,
          basename: 'napping',
          sheet: catSheetText({ grades: ['Y', 'N', '[Not enough]', 'Y',], },),
          manifest: catManifestText({
            kinds: [
              'control',
              'control',
              'control',
              'flagged',
            ],
          },),
        },);

        await printVerifyScore({
          runsDir: scratch.path,
          basename: 'napping',
        },);

        expect(printed.lines,).toStrictEqual([
          'control  flags=3 realDamage=1 invented=1 unscored=1 precision=0.500',
          'flagged  flags=1 realDamage=1 invented=0 unscored=0 precision=1.000',
          LABEL_NOTE,
        ],);
      },
    },),

    it({
      name: 'SAYS the sheet and the manifest hold no item, rather than printing the note alone',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        await using scratch = await scratchDir({ prefix: 'score-verify-run-', },);
        await writeSheetFiles({
          runsDir: scratch.path,
          basename: 'napping',
          sheet: '# nothing graded here\n',
          manifest: catManifestText({ kinds: [], },),
        },);

        await printVerifyScore({
          runsDir: scratch.path,
          basename: 'napping',
        },);

        expect(printed.lines,).toStrictEqual([
          'NONE the sheet and the manifest hold no item, so there is no set to report',
          LABEL_NOTE,
        ],);
      },
    },),

    it({
      name: 'REFUSES in its own words a sheet that is not there, naming its path and ENOENT',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        await using scratch = await scratchDir({ prefix: 'score-verify-run-', },);
        await writeSheetFiles({
          runsDir: scratch.path,
          basename: 'napping',
          manifest: catManifestText({ kinds: [], },),
        },);

        /**
         What reporting a run with no sheet raised.
         */
        const refusal = await rejectionOf(async function reportsWithoutSheet(): Promise<void> {
          await printVerifyScore({
            runsDir: scratch.path,
            basename: 'napping',
          },);
        },);

        expect(refusal,).toBeInstanceOf(StatedRefusalError,);
        expect(String(refusal,),).toBe(
          `StatedRefusalError: cannot read the graded sheet at ${scratch.path}/napping-sheet.md (ENOENT); `
            + 'name the runs directory that holds it with TRANSLATION_REPAIR_RUNS_DIR, or the sheet with '
            + 'VERIFY_SHEET_BASENAME',
        );
        expect(printed.lines,).toStrictEqual([],);
      },
    },),

    it({
      name: 'REFUSES a manifest that is not there, naming its base name and ENOENT',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        await using scratch = await scratchDir({ prefix: 'score-verify-run-', },);
        await writeSheetFiles({
          runsDir: scratch.path,
          basename: 'napping',
          sheet: catSheetText({ grades: ['Y',], },),
        },);

        /**
         What reporting a run with no manifest raised.
         */
        const refusal = await rejectionOf(async function reportsWithoutManifest(): Promise<void> {
          await printVerifyScore({
            runsDir: scratch.path,
            basename: 'napping',
          },);
        },);

        expect(refusal,).toBeInstanceOf(RunJsonUnreadableError,);
        expect(String(refusal,),).toBe(
          'RunJsonUnreadableError: could not read napping-manifest.json as JSON (ENOENT)',
        );
        expect(printed.lines,).toStrictEqual([],);
      },
    },),

    it({
      name: 'REFUSES a sheet and a manifest of different lengths before printing any set',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        await using scratch = await scratchDir({ prefix: 'score-verify-run-', },);
        await writeSheetFiles({
          runsDir: scratch.path,
          basename: 'napping',
          sheet: catSheetText({ grades: ['Y', 'N',], },),
          manifest: catManifestText({ kinds: ['control',], },),
        },);

        /**
         What reporting two files of different lengths raised.
         */
        const refusal = await rejectionOf(async function reportsMismatch(): Promise<void> {
          await printVerifyScore({
            runsDir: scratch.path,
            basename: 'napping',
          },);
        },);

        expect(refusal,).toBeInstanceOf(StatedRefusalError,);
        expect(String(refusal,),).toBe(
          'StatedRefusalError: sheet carries 2 items and manifest carries 1; a positional join between '
            + 'them would mislabel verdicts, so neither file describes the other',
        );
        expect(printed.lines,).toStrictEqual([],);
      },
    },),
  ],
},);
