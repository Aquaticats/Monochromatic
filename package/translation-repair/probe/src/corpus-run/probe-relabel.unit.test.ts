/**
 Tests for the probe relabel runner: the built command, run with every
 provider key withheld so it can never reach a model.

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
      name: 'probe-relabel as built',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'STOPS at exit 4 naming the manifest when the runs directory holds none, before any client is built',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'probe-relabel-', },);

            /**
             What the command wrote with every key withheld.
             */
            const run = await runBuiltCommand({
              command: 'probe-relabel',
              packageEntry: PROBE_ENTRY,
              env: scratchPlaces({
                runsDir: scratch.path,
                scratchDir: scratch.path,
              },),
            },);

            expect(run.code,).toBe(COULD_NOT_READ,);
            expect(run.stdout,).toBe('',);
            expect(run.stderr,).toBe(
              `probe-relabel: could not read ${MANIFEST_NAME} as JSON (ENOENT)\n`
                + '  Nothing was read past that file, so this run was not examined. Re-run the pass to '
                + 'rewrite it, or name a run directory that has it.\n',
            );
          },
        },),

        it({
          name: 'PRINTS both counts and both notes and exits 0 when the manifest draws no position, asking no model',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'probe-relabel-', },);
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
              command: 'probe-relabel',
              packageEntry: PROBE_ENTRY,
              env: scratchPlaces({
                runsDir: scratch.path,
                scratchDir: scratch.path,
              },),
            },);

            expect(run.code,).toBe(0,);
            expect(run.stderr,).toBe('',);
            expect(run.stdout,).toBe(
              'RELABEL rebuilt 0 distinct damaged regions\n'
                + 'RELABEL gathered 0 unflagged control regions\n'
                + 'NOTE production sends issues-withheld. Compare it against issues-rendered on each region: '
                + 'a region that reports damage under one prompt only is one the other prompt talks the probe '
                + 'out of. Compare it against issues-absent: a region that reports damage only with no list '
                + 'known is one whose claims the screen dismisses as restating a prior issue. A region dark '
                + 'under all three exonerates the label and indicts the difficulty of the judgement.\n'
                + 'NOTE a control line prints positions= empty. Read the issues-absent arm across controls '
                + 'against the issues-absent arm across damaged regions: similar rates mean the unlabelled '
                + 'prober is re-reporting pre-existing defects and the damaged result proves nothing, and a '
                + 'much lower control rate means the issue list is suppressing real detections.\n',
            );
          },
        },),

        it({
          name: 'REFUSES an argument it does not read as stated and exits 6, naming the argument and the usage',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'probe-relabel-', },);

            /**
             What the command wrote given one argument.
             */
            const run = await runBuiltCommand({
              command: 'probe-relabel',
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
              'probe-relabel: --bogus is not a flag this command reads. Usage: probe-relabel\n',
            );
          },
        },),
      ],
    },),
  ],
},);
