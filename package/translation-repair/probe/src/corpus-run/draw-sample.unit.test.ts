/**
 Boundary test for the draw sample command.

 The command reads every settled artifact of a runs directory and the corpus
 page of each, so what is checked here is what only the built command can
 show: what it prints and exits with on a throwaway runs directory, over a
 throwaway corpus clone, with no provider key in its environment. Its
 procedures have their own suites.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  mkdir,
  readdir,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { digestPipeline, } from '@monochromatic-dev/translation-repair-glossary';
import { makeNamingArchive, } from '@monochromatic-dev/translation-repair-corpus-pass/ts/archive-naming.test-fixture.ts';
import {
  type ChildRun,
  runBuiltCommand,
} from '@monochromatic-dev/translation-repair-adjudication/ts/child-environment.test-fixture.ts';
import { PROBE_ENTRY, } from './built-entry.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { writeSettledArtifact, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/settled-v1-pool.test-fixture.ts';
import {
  CONFLICTING_POOL_SAYS,
  CONFLICTING_POOL_VARIABLES,
} from '@monochromatic-dev/translation-repair-artifact-pool/ts/corpus-run/pool-policy.test-fixture.ts';

/**
 Exit code `reportingRefusals` sets for a stated refusal.
 */
const REFUSED_AS_STATED = 6;

/**
 Seed the gate draw's files are named after.
 */
const FINAL_SEED = 'milestone-three-precision-round-three';

/**
 Seed the preliminary draw's files are named after.
 */
const PRELIMINARY_SEED = `${FINAL_SEED}-preliminary`;

/**
 Runs the built command over a runs directory and a corpus clone that hold one
 settled entry of two accepted issues.

 @param argv - arguments after the script path

 @param repairRecorded - whether the settled issues carry a recorded repair

 @param settle - whether the runs directory holds the artifact; false leaves its artifacts directory empty

 @returns Run, runs directory it was given, and what that directory held afterwards

 @example
 ```ts
 const { run, } = await drawOverOneEntry({ argv: [], settle: true, repairRecorded: false, },);
 ```
 */
async function drawOverOneEntry(
  {
    argv,
    settle,
    repairRecorded,
  }: {
    readonly argv: readonly string[];
    readonly settle: boolean;
    readonly repairRecorded: boolean;
  },
): Promise<{
  readonly run: ChildRun;
  readonly runsDir: string;
  readonly held: readonly string[];
}> {
  await using scratch = await scratchDir({ prefix: 'draw-sample-built-', },);
  await using archive = await makeNamingArchive({
    before: '猫\n',
    after: '猫猫在窗台上睡觉。\n',
    relPath: 'people/mittens/page.md',
  },);
  await mkdir(
    join(
      scratch.path,
      'artifacts',
    ),
    { recursive: true, },
  );
  if (settle) {
    await writeSettledArtifact({
      runsDir: scratch.path,
      entryId: 'mittens',
      issueIds: [
        'adjudicated/purr-one',
        'adjudicated/purr-two',
      ],
      repairRecorded,
    },);
  }

  // The keys are withheld by `runBuiltCommand`, which builds the child's
  // whole environment from the parent's without any `_API_KEY` variable.
  const run = await runBuiltCommand({
    command: 'draw-sample',
    packageEntry: PROBE_ENTRY,
    args: argv,
    env: {
      TRANSLATION_REPAIR_RUNS_DIR: scratch.path,
      TRANSLATION_REPAIR_CORPUS_CLONE_DIR: archive.pin.cloneDir,
      TRANSLATION_REPAIR_CORPUS_COMMIT: archive.pin.commitSha,
    },
  },);
  return {
    run,
    runsDir: scratch.path,
    held: (await readdir(scratch.path,)).toSorted(),
  };
}

await describe({
  name: 'draw-sample as built',
  children: [
    it({
      name: 'DRAWS a preliminary sample over one settled entry, prints the pool and the draw and exits 0',
      fn: async () => {
        const {
          run,
          runsDir,
          held,
        } = await drawOverOneEntry({
          argv: [],
          settle: true,
          repairRecorded: false,
        },);
        const { digest, } = await digestPipeline({ dir: join(
          import.meta.dirname,
          '../../dist/final/node',
        ), },);

        expect(run,).toEqual({
          code: 0,
          stdout: [
            `POOL read by pipeline ${digest}`,
            'POOL 1 entry across 1 pipeline generation',
            'POOL band=small entries=1 contributing=1 accepted=2 perEntry=mittens:2',
            'POOL band=medium entries=0 contributing=0 accepted=0 perEntry=',
            'POOL band=large entries=0 contributing=0 accepted=0 perEntry=',
            `SAMPLE final=false seed=${PRELIMINARY_SEED} pool=2 drawn=2 unrecordedRepairs=2 unrecordedInPool=2 `
              + `out=${join(runsDir, `grading-sheet-${PRELIMINARY_SEED}.md`,)} `
              + `repairOut=${join(runsDir, `repair-sheet-${PRELIMINARY_SEED}.md`,)} `
              + `manifest=${join(runsDir, `sample-manifest-${PRELIMINARY_SEED}.json`,)}`,
            '',
          ].join('\n',),
          stderr: '',
        },);
        expect(held,).toEqual([
          'artifacts',
          `grading-sheet-${PRELIMINARY_SEED}.md`,
          `repair-sheet-${PRELIMINARY_SEED}.md`,
          `sample-manifest-${PRELIMINARY_SEED}.json`,
        ],);
      },
    },),
    it({
      name: 'DRAWS a final sample once every sampled issue carries a recorded repair, writing the gate sheets',
      fn: async () => {
        const {
          run,
          runsDir,
          held,
        } = await drawOverOneEntry({
          argv: ['--final',],
          settle: true,
          repairRecorded: true,
        },);
        const { digest, } = await digestPipeline({ dir: join(
          import.meta.dirname,
          '../../dist/final/node',
        ), },);

        expect(run,).toEqual({
          code: 0,
          stdout: [
            `POOL read by pipeline ${digest}`,
            'POOL 1 entry across 1 pipeline generation',
            'POOL band=small entries=1 contributing=1 accepted=2 perEntry=mittens:2',
            'POOL band=medium entries=0 contributing=0 accepted=0 perEntry=',
            'POOL band=large entries=0 contributing=0 accepted=0 perEntry=',
            `SAMPLE final=true seed=${FINAL_SEED} pool=2 drawn=2 unrecordedRepairs=0 unrecordedInPool=0 `
              + `out=${join(runsDir, `grading-sheet-${FINAL_SEED}.md`,)} `
              + `repairOut=${join(runsDir, `repair-sheet-${FINAL_SEED}.md`,)} `
              + `manifest=${join(runsDir, `sample-manifest-${FINAL_SEED}.json`,)}`,
            '',
          ].join('\n',),
          stderr: '',
        },);
        expect(held,).toEqual([
          'artifacts',
          `grading-sheet-${FINAL_SEED}.md`,
          `repair-sheet-${FINAL_SEED}.md`,
          `sample-manifest-${FINAL_SEED}.json`,
        ],);
      },
    },),
    it({
      name: 'REFUSES a flag the command does not read, exits 6 and prints the refusal alone',
      fn: async () => {
        const run = await runBuiltCommand({
          command: 'draw-sample',
          packageEntry: PROBE_ENTRY,
          args: ['--bogus',],
          env: {},
        },);

        expect(run,).toEqual({
          code: REFUSED_AS_STATED,
          stdout: '',
          stderr: 'draw-sample: --bogus is not a flag this command reads. Usage: draw-sample [--final]\n',
        },);
      },
    },),
    it({
      name: 'REFUSES as stated and exits 6 when no entry has settled, writing no sheet',
      fn: async () => {
        const {
          run,
          held,
        } = await drawOverOneEntry({
          argv: [],
          settle: false,
          repairRecorded: false,
        },);

        expect(run.code,).toBe(REFUSED_AS_STATED,);
        expect(run.stdout,).toBe('',);
        expect(run.stderr,).toBe(
          [
            'draw-sample: No entry has settled yet, so there is nothing to pool.',
            '',
            'This THROWS rather than returning an empty pool, because every caller',
            'of this function goes on to compute a rate. A rate over zero entries',
            'is this module\'s own failure mode taken to its limit: a denominator',
            'quietly shrunk, here all the way to nothing, while the number above it',
            'still renders. Accumulate entries under the required pipeline, or',
            'require an earlier commit that the settled entries actually contain.',
            '',
          ].join('\n',),
        );
        expect(held,).toEqual(['artifacts',],);
      },
    },),
    it({
      name: 'REFUSES A REQUIRED COMMIT AND A MIXED POOL ASKED FOR TOGETHER in its own words, read from the '
        + 'environment it runs in, exiting 6 and writing nothing',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'draw-sample-built-', },);
        await mkdir(join(
          scratch.path,
          'artifacts',
        ),);

        const run = await runBuiltCommand({
          command: 'draw-sample',
          packageEntry: PROBE_ENTRY,
          args: [],
          env: {
            TRANSLATION_REPAIR_RUNS_DIR: scratch.path,
            ...CONFLICTING_POOL_VARIABLES,
          },
        },);

        expect(run,).toEqual({
          code: REFUSED_AS_STATED,
          stdout: '',
          stderr: `draw-sample: ${CONFLICTING_POOL_SAYS}\n`,
        },);
        expect(await readdir(scratch.path,),).toEqual(['artifacts',],);
      },
    },),
    it({
      name: 'REFUSES as stated and exits 6, naming the missing directory, when the runs directory holds no artifacts directory',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'draw-sample-built-', },);

        const run = await runBuiltCommand({
          command: 'draw-sample',
          packageEntry: PROBE_ENTRY,
          args: [],
          env: { TRANSLATION_REPAIR_RUNS_DIR: scratch.path, },
        },);

        expect(run,).toEqual({
          code: REFUSED_AS_STATED,
          stdout: '',
          stderr: `draw-sample: there is no artifacts directory at ${join(scratch.path, 'artifacts',)}; name the `
            + 'runs directory a pass settled entries into with TRANSLATION_REPAIR_RUNS_DIR\n',
        },);
      },
    },),
    it({
      name: 'REFUSES as stated and exits 6 a final draw whose sampled issues carry no recorded repair, writing no sheet',
      fn: async () => {
        const {
          run,
          held,
        } = await drawOverOneEntry({
          argv: ['--final',],
          settle: true,
          repairRecorded: false,
        },);
        const { digest, } = await digestPipeline({ dir: join(
          import.meta.dirname,
          '../../dist/final/node',
        ), },);

        expect(run,).toEqual({
          code: REFUSED_AS_STATED,
          stdout: [
            `POOL read by pipeline ${digest}`,
            'POOL 1 entry across 1 pipeline generation',
            'POOL band=small entries=1 contributing=1 accepted=2 perEntry=mittens:2',
            'POOL band=medium entries=0 contributing=0 accepted=0 perEntry=',
            'POOL band=large entries=0 contributing=0 accepted=0 perEntry=',
            '',
          ].join('\n',),
          stderr: 'draw-sample: refusing a final draw: 2 of 2 sampled issues carry no recorded repair, so repair '
            + 'quality cannot be measured over this sample. Those artifacts predate repair recording; move them '
            + 'aside and rerun the pass into a fresh artifacts directory.\n',
        },);
        expect(held,).toEqual(['artifacts',],);
      },
    },),
  ],
},);
