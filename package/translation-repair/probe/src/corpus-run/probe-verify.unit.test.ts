/**
 Tests for the probe verify runner: the built command, run with every
 provider key withheld so it can never reach a model.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  readFile,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { formatVerifySheet, } from '../../dist/final/node/index.mjs';
import { runBuiltCommand, } from '@monochromatic-dev/translation-repair-adjudication/ts/child-environment.test-fixture.ts';
import { PROBE_ENTRY, } from './built-entry.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { scratchPlaces, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/keyless-built-command.test-fixture.ts';

/**
 Exit code `reportingRefusals` sets when a run file would not read.
 */
const COULD_NOT_READ = 4;

/**
 Manifest the runner indexes its damaged positions into, in the runs directory.
 */
const MANIFEST_NAME = 'sample-manifest-milestone-three-precision-round-three.json';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: 'probe-verify as built',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'STOPS at exit 4 naming the manifest when the runs directory holds none, before any client is built',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'probe-verify-', },);

            /**
             What the command wrote with every key withheld.
             */
            const run = await runBuiltCommand({
              command: 'probe-verify',
              packageEntry: PROBE_ENTRY,
              env: scratchPlaces({
                runsDir: scratch.path,
                scratchDir: scratch.path,
              },),
            },);

            expect(run.code,).toBe(COULD_NOT_READ,);
            expect(run.stdout,).toBe('',);
            expect(run.stderr,).toBe(
              `probe-verify: could not read ${MANIFEST_NAME} as JSON (ENOENT)\n`
                + '  Nothing was read past that file, so this run was not examined. Re-run the pass to '
                + 'rewrite it, or name a run directory that has it.\n',
            );
          },
        },),

        it({
          name: 'WRITES an empty sheet and manifest and exits 0 when the manifest draws no position, asking no model',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'probe-verify-', },);
            await writeFile(
              join(
                scratch.path,
                MANIFEST_NAME,
              ),
              JSON.stringify({
                seed: 'cat-seed',
                corpusSha: 'b'.repeat(40,),
                items: [],
              },),
              'utf8',
            );

            /**
             What the command wrote with every key withheld.
             */
            const run = await runBuiltCommand({
              command: 'probe-verify',
              packageEntry: PROBE_ENTRY,
              env: scratchPlaces({
                runsDir: scratch.path,
                scratchDir: scratch.path,
              },),
            },);

            expect(run.code,).toBe(0,);
            expect(run.stderr,).toBe('',);
            expect(run.stdout,).toBe(
              'VERIFY probing 0 damaged and 0 control regions, issues withheld\n'
                + `VERIFY wrote 0 items to ${scratch.path}/probe-verify-sheet.md\n`
                + 'NOTE the sheet is blind and its manifest is not. Grade the sheet without opening the '
                + 'manifest, or the answer stops meaning anything.\n',
            );
            expect(
              await readFile(
                join(
                  scratch.path,
                  'probe-verify-manifest.json',
                ),
                'utf8',
              ),
            ).toBe('{\n  "items": []\n}',);
            expect(
              await readFile(
                join(
                  scratch.path,
                  'probe-verify-sheet.md',
                ),
                'utf8',
              ),
            ).toBe(formatVerifySheet({ items: [], },),);
          },
        },),

        it({
          name: 'REFUSES an argument it does not read as stated and exits 6, naming the argument and the usage',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'probe-verify-', },);

            /**
             What the command wrote given one argument.
             */
            const run = await runBuiltCommand({
              command: 'probe-verify',
              packageEntry: PROBE_ENTRY,
              args: ['--bogus',],
              env: scratchPlaces({
                runsDir: scratch.path,
                scratchDir: scratch.path,
              },),
            },);

            expect(run.code,).toBe(6,);
            expect(run.stdout,).toBe('',);
            expect(run.stderr,).toBe(
              'probe-verify: --bogus is not a flag this command reads. Usage: probe-verify\n',
            );
          },
        },),
      ],
    },),
  ],
},);
