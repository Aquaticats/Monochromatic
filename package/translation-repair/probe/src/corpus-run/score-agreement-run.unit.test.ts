/**
 Tests for the report `score-agreement` prints over a graded sheet, its
 manifest and its pre-grades.

 Each case writes its own runs directory, hands it and a command line to
 `printGradeReport` and reads the whole of what was printed, or the refusal
 that stopped it.

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
import { SheetBindingError, } from '@monochromatic-dev/translation-repair-lane-contest';
import { DEFAULT_SAMPLE_SEED, } from '@monochromatic-dev/translation-repair-house-rule';
import { printGradeReport, } from '../../dist/final/node/index.mjs';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { lineOf, } from '@monochromatic-dev/translation-repair-run/ts/corpus-run/command-line.test-fixture.ts';
import { divertingConsoleLog, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/console-log-capture.test-fixture.ts';
import {
  catDrawDigest,
  catDrawSheetText,
  catPreGradesText,
  catSampleManifestText,
} from '@monochromatic-dev/translation-repair-lane-contest/ts/corpus-run/score-sheets.test-fixture.ts';

/**
 Draw seed every fixture sheet declares.
 */
const SEED = 'round-cats';

/**
 Corpus commit every fixture sheet declares.
 */
const CORPUS = 'abc1234';

/**
 Grades of the five-item sheet: two real defects, one false positive, one
 duplicate and one the grader declined.
 */
const GRADES = [
  'Y',
  'N',
  'Duplicate',
  '[Not enough context]',
  'Y',
];

/**
 Precision line of that sheet.
 */
const PRECISION_LINE = 'PRECISION items=5 gradeable=4 scored=3 realDefects=2 strict=0.500 excluded=0.667 '
  + 'lenient=0.750 duplicates=3 unscored=4';

/**
 Pre-grades of that sheet, disagreeing at the second position.
 */
const PRE_GRADES = [
  'real-defect',
  'real-defect',
  'duplicate',
  'unscored',
  'real-defect',
];

/**
 Agreement line those pre-grades give.
 */
const AGREEMENT_LINE = 'AGREEMENT compared=3 agreed=2 rate=0.667 disagreed=2';

/**
 Note printed when no manifest sits beside the sheet.
 */
const NO_MANIFEST_NOTE = 'NOTE no manifest found beside this sheet, so nothing proves the pre-grades this '
  + 'report reads describe the same draw. Both files are joined by POSITION and neither prints an issue id.';

/**
 Writes one file under a runs directory.

 @param runsDir - directory the case owns

 @param name - file name

 @param text - contents

 @returns Path written

 @example
 ```ts
 const path = await put({ runsDir, name: 'sheet.md', text: '', },);
 ```
 */
async function put(
  {
    runsDir,
    name,
    text,
  }: {
    readonly runsDir: string;
    readonly name: string;
    readonly text: string;
  },
): Promise<string> {
  /**
   Where the file goes.
   */
  const path = join(
    runsDir,
    name,
  );
  await writeFile(
    path,
    text,
    'utf8',
  );
  return path;
}

/**
 What reporting a sheet under the given command line raised.

 @param runsDir - runs directory the case owns

 @param typed - arguments after the runner's name

 @returns Refusal

 @example
 ```ts
 const refusal = await refusalOfReport({ runsDir, typed: ['--sheet', sheet,], },);
 ```
 */
async function refusalOfReport(
  {
    runsDir,
    typed,
  }: {
    readonly runsDir: string;
    readonly typed: readonly string[];
  },
): Promise<unknown> {
  return await rejectionOf(async function reportsRefused(): Promise<void> {
    await printGradeReport({
      runsDir,
      line: lineOf({ command: 'score-agreement', typed, },),
    },);
  },);
}

await describe({
  name: printGradeReport.name,
  concurrency: 1,
  children: [
    it({
      name: 'PRINTS precision and agreement from the default files, with no note where the draw digest binds the pair',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        await using scratch = await scratchDir({ prefix: 'score-agreement-run-', },);
        await put({
          runsDir: scratch.path,
          name: `grading-sheet-${DEFAULT_SAMPLE_SEED}.md`,
          text: catDrawSheetText({
            seed: SEED,
            corpusSha: CORPUS,
            drawDigest: catDrawDigest({ seed: SEED, corpusSha: CORPUS, count: 5, },),
            grades: GRADES,
          },),
        },);
        await put({
          runsDir: scratch.path,
          name: `sample-manifest-${SEED}.json`,
          text: catSampleManifestText({ seed: SEED, corpusSha: CORPUS, count: 5, digested: true, },),
        },);
        await put({
          runsDir: scratch.path,
          name: `pre-grades-${SEED}.json`,
          text: catPreGradesText({ verdicts: PRE_GRADES, },),
        },);

        await printGradeReport({
          runsDir: scratch.path,
          line: lineOf({ command: 'score-agreement', typed: [], },),
        },);

        expect(printed.lines,).toStrictEqual([
          PRECISION_LINE,
          AGREEMENT_LINE,
        ],);
      },
    },),

    it({
      name: 'SAYS the binding rests on the header where the manifest carries no draw digest',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        await using scratch = await scratchDir({ prefix: 'score-agreement-run-', },);
        const sheet = await put({
          runsDir: scratch.path,
          name: 'sheet.md',
          text: catDrawSheetText({ seed: SEED, corpusSha: CORPUS, grades: GRADES, },),
        },);
        await put({
          runsDir: scratch.path,
          name: `sample-manifest-${SEED}.json`,
          text: catSampleManifestText({ seed: SEED, corpusSha: CORPUS, count: 5, digested: false, },),
        },);

        await printGradeReport({
          runsDir: scratch.path,
          line: lineOf({ command: 'score-agreement', typed: ['--sheet', sheet,], },),
        },);

        expect(printed.lines.length,).toBe(3,);
        expect(printed.lines[0]?.startsWith('NOTE sheet and manifest agree on seed and corpus pin',),).toBe(true,);
        expect(printed.lines.slice(1,),).toStrictEqual([
          PRECISION_LINE,
          'AGREEMENT none: no pre-grades recorded for this draw',
        ],);
      },
    },),

    it({
      name: 'SAYS no manifest was found beside the sheet where none was named and none is there',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        await using scratch = await scratchDir({ prefix: 'score-agreement-run-', },);
        const sheet = await put({
          runsDir: scratch.path,
          name: 'sheet.md',
          text: catDrawSheetText({ seed: SEED, corpusSha: CORPUS, grades: GRADES, },),
        },);

        await printGradeReport({
          runsDir: scratch.path,
          line: lineOf({ command: 'score-agreement', typed: ['--sheet', sheet,], },),
        },);

        expect(printed.lines,).toStrictEqual([
          NO_MANIFEST_NOTE,
          PRECISION_LINE,
          'AGREEMENT none: no pre-grades recorded for this draw',
        ],);
      },
    },),

    it({
      name: 'NAMES every rate n/a and every list none for a sheet with no item, and compares nothing against no pre-grade',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        await using scratch = await scratchDir({ prefix: 'score-agreement-run-', },);
        const sheet = await put({
          runsDir: scratch.path,
          name: 'sheet.md',
          text: catDrawSheetText({ seed: SEED, corpusSha: CORPUS, grades: [], },),
        },);
        const preGrades = await put({
          runsDir: scratch.path,
          name: 'pre-grades.json',
          text: catPreGradesText({ verdicts: [], },),
        },);

        await printGradeReport({
          runsDir: scratch.path,
          line: lineOf({
            command: 'score-agreement',
            typed: [
              '--sheet',
              sheet,
              '--pre-grades',
              preGrades,
            ],
          },),
        },);

        expect(printed.lines,).toStrictEqual([
          NO_MANIFEST_NOTE,
          'PRECISION items=0 gradeable=0 scored=0 realDefects=0 strict=n/a excluded=n/a lenient=n/a '
            + 'duplicates=none unscored=none',
          'AGREEMENT compared=0 agreed=0 rate=n/a disagreed=none',
        ],);
      },
    },),

    it({
      name: 'REFUSES a sheet that declares no draw seed, before reading anything beside it',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        await using scratch = await scratchDir({ prefix: 'score-agreement-run-', },);
        const sheet = await put({
          runsDir: scratch.path,
          name: 'sheet.md',
          text: catDrawSheetText({ corpusSha: CORPUS, grades: GRADES, },),
        },);

        /**
         What reporting the sheet raised.
         */
        const refusal = await refusalOfReport({
          runsDir: scratch.path,
          typed: ['--sheet', sheet,],
        },);

        expect(refusal,).toBeInstanceOf(SheetBindingError,);
        expect(String(refusal,).startsWith(`SheetBindingError: detection sheet ${sheet} declares no draw seed`,),)
          .toBe(true,);
        expect(printed.lines,).toStrictEqual([],);
      },
    },),

    it({
      name: 'REFUSES a manifest of another draw than the sheet, before printing precision',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        await using scratch = await scratchDir({ prefix: 'score-agreement-run-', },);
        const sheet = await put({
          runsDir: scratch.path,
          name: 'sheet.md',
          text: catDrawSheetText({ seed: SEED, corpusSha: CORPUS, grades: GRADES, },),
        },);
        const manifest = await put({
          runsDir: scratch.path,
          name: 'manifest.json',
          text: catSampleManifestText({ seed: 'round-dogs', corpusSha: CORPUS, count: 5, digested: false, },),
        },);

        /**
         What reporting the pair raised.
         */
        const refusal = await refusalOfReport({
          runsDir: scratch.path,
          typed: [
            '--sheet',
            sheet,
            '--manifest',
            manifest,
          ],
        },);

        expect(refusal,).toBeInstanceOf(SheetBindingError,);
        expect(String(refusal,).startsWith(`SheetBindingError: detection sheet ${sheet} and manifest ${manifest} `,),)
          .toBe(true,);
        expect(printed.lines,).toStrictEqual([],);
      },
    },),

    it({
      name: 'REFUSES in its own words a sheet that is not there, naming its path, ENOENT and --sheet',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        await using scratch = await scratchDir({ prefix: 'score-agreement-run-', },);

        /**
         Sheet nobody wrote.
         */
        const sheet = join(
          scratch.path,
          'sheet.md',
        );

        /**
         What reporting it raised.
         */
        const refusal = await refusalOfReport({
          runsDir: scratch.path,
          typed: ['--sheet', sheet,],
        },);

        expect(refusal,).toBeInstanceOf(StatedRefusalError,);
        expect(String(refusal,),).toBe(
          `StatedRefusalError: cannot read the graded sheet at ${sheet} (ENOENT); name the sheet that exists with `
            + '--sheet, or put the default one in the runs directory',
        );
        expect(printed.lines,).toStrictEqual([],);
      },
    },),

    it({
      name: 'REFUSES a manifest named with --manifest that is not there, rather than scoring without it',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        await using scratch = await scratchDir({ prefix: 'score-agreement-run-', },);
        const sheet = await put({
          runsDir: scratch.path,
          name: 'sheet.md',
          text: catDrawSheetText({ seed: SEED, corpusSha: CORPUS, grades: GRADES, },),
        },);

        /**
         Manifest nobody wrote.
         */
        const manifest = join(
          scratch.path,
          'manifest.json',
        );

        /**
         What reporting it raised.
         */
        const refusal = await refusalOfReport({
          runsDir: scratch.path,
          typed: [
            '--sheet',
            sheet,
            '--manifest',
            manifest,
          ],
        },);

        expect(refusal,).toBeInstanceOf(StatedRefusalError,);
        expect(String(refusal,),).toBe(
          `StatedRefusalError: --manifest names ${manifest}, which is not there; name a sample manifest that `
            + 'exists, or leave the flag out to look beside the sheet',
        );
        expect(printed.lines,).toStrictEqual([],);
      },
    },),

    it({
      name: 'REFUSES pre-grades named with --pre-grades that are not there, after printing precision',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        await using scratch = await scratchDir({ prefix: 'score-agreement-run-', },);
        const sheet = await put({
          runsDir: scratch.path,
          name: 'sheet.md',
          text: catDrawSheetText({ seed: SEED, corpusSha: CORPUS, grades: GRADES, },),
        },);

        /**
         Pre-grades nobody wrote.
         */
        const preGrades = join(
          scratch.path,
          'pre-grades.json',
        );

        /**
         What reporting it raised.
         */
        const refusal = await refusalOfReport({
          runsDir: scratch.path,
          typed: [
            '--sheet',
            sheet,
            '--pre-grades',
            preGrades,
          ],
        },);

        expect(refusal,).toBeInstanceOf(StatedRefusalError,);
        expect(String(refusal,),).toBe(
          `StatedRefusalError: --pre-grades names ${preGrades}, which is not there; name a pre-grades file that `
            + 'exists, or leave the flag out to look beside the sheet',
        );
        expect(printed.lines,).toStrictEqual([NO_MANIFEST_NOTE, PRECISION_LINE,],);
      },
    },),
  ],
},);
