/**
 Tests for the `score-agreement` runner: the built command over a graded
 sheet, a manifest and pre-grades a case wrote.

 The runner reads files and prints precision and agreement, so its as-built
 cases run to the end of the report on a fixture and read the whole of what
 it printed. Every child is started by `runBuiltCommand`, whose environment
 carries no variable ending in `_API_KEY`.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { writeFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { runBuiltCommand, } from '@monochromatic-dev/translation-repair-adjudication/ts/child-environment.test-fixture.ts';
import { PROBE_ENTRY, } from './built-entry.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import {
  catDrawSheetText,
  catPreGradesText,
  catSampleManifestText,
} from '@monochromatic-dev/translation-repair-lane-contest/ts/corpus-run/score-sheets.test-fixture.ts';

/**
 Built command under test.
 */
const COMMAND = 'score-agreement';

/**
 Precision line of the five-item sheet every case grades.
 */
const PRECISION_LINE = 'PRECISION items=5 gradeable=4 scored=3 realDefects=2 strict=0.500 excluded=0.667 '
  + 'lenient=0.750 duplicates=3 unscored=4';

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

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: 'score-agreement as built',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'PRINTS precision and agreement for a named sheet, manifest and pre-grades, and exits 0',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'score-agreement-', },);

            /**
             Paths of the three files the flags name.
             */
            const paths = {
              sheet: join(
                scratch.path,
                'sheet.md',
              ),
              manifest: join(
                scratch.path,
                'manifest.json',
              ),
              preGrades: join(
                scratch.path,
                'pre-grades.json',
              ),
            };
            await writeFile(
              paths.sheet,
              catDrawSheetText({ seed: 'round-cats', corpusSha: 'abc1234', grades: GRADES, },),
              'utf8',
            );
            await writeFile(
              paths.manifest,
              catSampleManifestText({ seed: 'round-cats', corpusSha: 'abc1234', count: 5, digested: false, },),
              'utf8',
            );
            await writeFile(
              paths.preGrades,
              catPreGradesText({
                verdicts: [
                  'real-defect',
                  'real-defect',
                  'duplicate',
                  'unscored',
                  'real-defect',
                ],
              },),
              'utf8',
            );

            /**
             What the command wrote.
             */
            const run = await runBuiltCommand({
              command: COMMAND,
              packageEntry: PROBE_ENTRY,
              args: [
                '--sheet',
                paths.sheet,
                '--manifest',
                paths.manifest,
                '--pre-grades',
                paths.preGrades,
              ],
              env: { TRANSLATION_REPAIR_RUNS_DIR: scratch.path, },
            },);

            expect(run.code,).toBe(0,);
            expect(run.stderr,).toBe('',);
            expect(run.stdout,).toBe([
              'NOTE sheet and manifest agree on seed and corpus pin, but one of them carries no draw digest, so '
                + 'this join rests on the file names and the header rather than on the items. Draws taken before '
                + 'the digest existed read this way, and are scoreable; a NEW draw reading this way means the '
                + 'digest was dropped somewhere and the pairing is unproven.',
              PRECISION_LINE,
              'AGREEMENT compared=3 agreed=2 rate=0.667 disagreed=2',
              '',
            ].join('\n',),);
          },
        },),

        it({
          name: 'SAYS no pre-grades were recorded where the pre-grades file named by the draw is not there',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'score-agreement-', },);

            /**
             Sheet the flag names.
             */
            const sheet = join(
              scratch.path,
              'sheet.md',
            );
            await writeFile(
              sheet,
              catDrawSheetText({ seed: 'round-cats', corpusSha: 'abc1234', grades: GRADES, },),
              'utf8',
            );

            /**
             What the command wrote.
             */
            const run = await runBuiltCommand({
              command: COMMAND,
              packageEntry: PROBE_ENTRY,
              args: [
                '--sheet',
                sheet,
              ],
              env: { TRANSLATION_REPAIR_RUNS_DIR: scratch.path, },
            },);

            expect(run.code,).toBe(0,);
            expect(run.stderr,).toBe('',);
            expect(run.stdout,).toBe([
              'NOTE no manifest found beside this sheet, so nothing proves the pre-grades this report reads '
                + 'describe the same draw. Both files are joined by POSITION and neither prints an issue id.',
              PRECISION_LINE,
              'AGREEMENT none: no pre-grades recorded for this draw',
              '',
            ].join('\n',),);
          },
        },),

        it({
          name: 'REFUSES a flag written with no value in its own words and exits 6 before reading anything',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'score-agreement-', },);

            /**
             What the command wrote.
             */
            const run = await runBuiltCommand({
              command: COMMAND,
              packageEntry: PROBE_ENTRY,
              args: ['--sheet',],
              env: { TRANSLATION_REPAIR_RUNS_DIR: scratch.path, },
            },);

            expect(run.code,).toBe(6,);
            expect(run.stdout,).toBe('',);
            expect(run.stderr,).toBe('score-agreement: --sheet needs a value written after it. Usage: score-agreement [--sheet <graded sheet>] '
              + '[--manifest <sample manifest>] [--pre-grades <pre-grades file>]\n',);
          },
        },),

        it({
          name: 'REFUSES a manifest named with --manifest that is not there in its own words, exiting 6',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'score-agreement-', },);

            /**
             Sheet the flag names.
             */
            const sheet = join(
              scratch.path,
              'sheet.md',
            );
            await writeFile(
              sheet,
              catDrawSheetText({ seed: 'round-cats', corpusSha: 'abc1234', grades: GRADES, },),
              'utf8',
            );

            /**
             Manifest nobody wrote.
             */
            const manifest = join(
              scratch.path,
              'manifest.json',
            );

            /**
             What the command wrote.
             */
            const run = await runBuiltCommand({
              command: COMMAND,
              packageEntry: PROBE_ENTRY,
              args: [
                '--sheet',
                sheet,
                '--manifest',
                manifest,
              ],
              env: { TRANSLATION_REPAIR_RUNS_DIR: scratch.path, },
            },);

            expect(run.code,).toBe(6,);
            expect(run.stdout,).toBe('',);
            expect(run.stderr,).toBe(
              `score-agreement: --manifest names ${manifest}, which is not there; name a sample manifest that `
                + 'exists, or leave the flag out to look beside the sheet\n',
            );
          },
        },),
      ],
    },),
  ],
},);
