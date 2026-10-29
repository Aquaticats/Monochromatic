/**
 Tests for the `score-verify` runner: the built command over a runs directory
 a case wrote.

 The runner reads a graded sheet and its manifest and prints one line per set,
 so its as-built cases run to the end of the report on a fixture and read the
 whole of what it printed. Every child is started by `runBuiltCommand`, whose
 environment carries no variable ending in `_API_KEY`.

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
  catManifestText,
  catSheetText,
} from '@monochromatic-dev/translation-repair-lane-contest/ts/corpus-run/score-sheets.test-fixture.ts';

/**
 Built command under test.
 */
const COMMAND = 'score-verify';

/**
 The note every report ends with.
 */
const LABEL_NOTE = 'NOTE read the labels as the drawing task meant them. On the verification '
  + 'sheet they name what a READER already believed, so the control line is the one that decides '
  + 'gating. On the damage sample they name what the PROBE said, so probe-flagged carries its '
  + 'precision and probe-silent carries its MISSES: a Y there is damage the probe did not see.';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: 'score-verify as built',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'REPORTS each set\'s flags, verdict counts and precision, then the label note, and exits 0',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'score-verify-', },);
            await writeFile(
              join(
                scratch.path,
                'probe-verify-sheet.md',
              ),
              catSheetText({ grades: ['Y', 'N', '[Not enough]', 'Y', 'Y', 'N',], },),
              'utf8',
            );
            await writeFile(
              join(
                scratch.path,
                'probe-verify-manifest.json',
              ),
              catManifestText({
                kinds: [
                  'control',
                  'control',
                  'control',
                  'flagged',
                  'flagged',
                  'flagged',
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
              args: [],
              env: { TRANSLATION_REPAIR_RUNS_DIR: scratch.path, },
            },);

            expect(run.code,).toBe(0,);
            expect(run.stderr,).toBe('',);
            expect(run.stdout,).toBe([
              'control  flags=3 realDamage=1 invented=1 unscored=1 precision=0.500',
              'flagged  flags=3 realDamage=2 invented=1 unscored=0 precision=0.667',
              LABEL_NOTE,
              '',
            ].join('\n',),);
          },
        },),

        it({
          name: 'READS the sheet named by VERIFY_SHEET_BASENAME instead of the default',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'score-verify-', },);
            await writeFile(
              join(
                scratch.path,
                'napping-sheet.md',
              ),
              catSheetText({ grades: ['N',], },),
              'utf8',
            );
            await writeFile(
              join(
                scratch.path,
                'napping-manifest.json',
              ),
              catManifestText({ kinds: ['probe-silent',], },),
              'utf8',
            );

            /**
             What the command wrote.
             */
            const run = await runBuiltCommand({
              command: COMMAND,
              packageEntry: PROBE_ENTRY,
              args: [],
              env: {
                VERIFY_SHEET_BASENAME: 'napping',
                TRANSLATION_REPAIR_RUNS_DIR: scratch.path,
              },
            },);

            expect(run.code,).toBe(0,);
            expect(run.stderr,).toBe('',);
            expect(run.stdout,).toBe([
              'probe-silent flags=1 realDamage=0 invented=1 unscored=0 precision=0.000',
              LABEL_NOTE,
              '',
            ].join('\n',),);
          },
        },),

        it({
          name: 'EXITS 4 naming the manifest when the sheet is there and the manifest is not',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'score-verify-', },);
            await writeFile(
              join(
                scratch.path,
                'probe-verify-sheet.md',
              ),
              catSheetText({ grades: ['Y',], },),
              'utf8',
            );

            /**
             What the command wrote.
             */
            const run = await runBuiltCommand({
              command: COMMAND,
              packageEntry: PROBE_ENTRY,
              args: [],
              env: { TRANSLATION_REPAIR_RUNS_DIR: scratch.path, },
            },);

            expect(run.code,).toBe(4,);
            expect(run.stdout,).toBe('',);
            expect(run.stderr,).toBe([
              'score-verify: could not read probe-verify-manifest.json as JSON (ENOENT)',
              '  Nothing was read past that file, so this run was not examined. Re-run the pass to rewrite it, '
                + 'or name a run directory that has it.',
              '',
            ].join('\n',),);
          },
        },),

        it({
          name: 'REFUSES a sheet and a manifest of different lengths in its own words, exiting 6 with nothing printed',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'score-verify-', },);
            await writeFile(
              join(
                scratch.path,
                'probe-verify-sheet.md',
              ),
              catSheetText({ grades: ['Y', 'N',], },),
              'utf8',
            );
            await writeFile(
              join(
                scratch.path,
                'probe-verify-manifest.json',
              ),
              catManifestText({ kinds: ['control',], },),
              'utf8',
            );

            /**
             What the command wrote.
             */
            const run = await runBuiltCommand({
              command: COMMAND,
              packageEntry: PROBE_ENTRY,
              args: [],
              env: { TRANSLATION_REPAIR_RUNS_DIR: scratch.path, },
            },);

            expect(run.code,).toBe(6,);
            expect(run.stdout,).toBe('',);
            expect(run.stderr,).toBe(
              'score-verify: sheet carries 2 items and manifest carries 1; a positional join between them would '
                + 'mislabel verdicts, so neither file describes the other\n',
            );
          },
        },),

        it({
          name: 'REFUSES a runs directory with no sheet in its own words, exiting 6 with nothing printed',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'score-verify-', },);

            /**
             What the command wrote.
             */
            const run = await runBuiltCommand({
              command: COMMAND,
              packageEntry: PROBE_ENTRY,
              args: [],
              env: { TRANSLATION_REPAIR_RUNS_DIR: scratch.path, },
            },);

            expect(run.code,).toBe(6,);
            expect(run.stdout,).toBe('',);
            expect(run.stderr,).toBe(
              `score-verify: cannot read the graded sheet at ${scratch.path}/probe-verify-sheet.md (ENOENT); `
                + 'name the runs directory that holds it with TRANSLATION_REPAIR_RUNS_DIR, or the sheet with '
                + 'VERIFY_SHEET_BASENAME\n',
            );
          },
        },),
      ],
    },),
  ],
},);
