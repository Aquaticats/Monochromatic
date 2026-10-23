/**
 Boundary test for the damage sample command.

 The command reads every settled artifact of a runs directory and asks the
 roster about each region it draws, so what is checked here is what only the
 built command can show with no provider key in its environment: what it
 prints and exits with before any call. Its procedures have their own suites.

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
} from '@monochromatic-dev/module-test/ts';

import { digestPipeline, } from '@monochromatic-dev/translation-repair-glossary';
import {
  type ChildRun,
  runBuiltCommand,
} from '@monochromatic-dev/translation-repair-adjudication/ts/child-environment.test-fixture.ts';
import { PROBE_ENTRY, } from './built-entry.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { writeSettledV2, } from '@monochromatic-dev/translation-repair-artifact/ts/corpus-run/settled-v2-pool.test-fixture.ts';
import {
  CONFLICTING_POOL_SAYS,
  CONFLICTING_POOL_VARIABLES,
} from '@monochromatic-dev/translation-repair-artifact-pool/ts/corpus-run/pool-policy.test-fixture.ts';

/**
 Exit code `reportingRefusals` sets for a stated refusal.
 */
const REFUSED_AS_STATED = 6;

/**
 Runs the built command over a runs directory holding one settled entry.

 @param shippedSlices - how many slices, from the first, the entry's repair lane replaced the archive's wording at

 @returns The run and what the runs directory held afterwards

 @example
 ```ts
 const { run, held, } = await drawOverOneEntry({ shippedSlices: 1, },);
 ```
 */
async function drawOverOneEntry(
  { shippedSlices, }: { readonly shippedSlices: number; },
): Promise<{
  readonly run: ChildRun;
  readonly runsDir: string;
  readonly held: readonly string[];
}> {
  await using scratch = await scratchDir({ prefix: 'damage-sample-built-', },);
  await writeSettledV2({
    runsDir: scratch.path,
    entryId: 'mittens',
    shippedSlices,
  },);

  // The keys are withheld by `runBuiltCommand`, which builds the child's
  // whole environment from the parent's without any `_API_KEY` variable.
  const run = await runBuiltCommand({
    command: 'damage-sample',
    packageEntry: PROBE_ENTRY,
    args: [],
    env: { TRANSLATION_REPAIR_RUNS_DIR: scratch.path, },
  },);
  return {
    run,
    runsDir: scratch.path,
    held: (await readdir(scratch.path,)).toSorted(),
  };
}

/**
 Lines the pool's census prints before the command's own.

 @returns The two lines

 @example
 ```ts
 const lines = await censusLines();
 ```
 */
async function censusLines(): Promise<readonly string[]> {
  /**
   Digest of the built output the command runs the census from.
   */
  const { digest, } = await digestPipeline({ dir: join(
    import.meta.dirname,
    '../../dist/final/node',
  ), },);
  return [
    `POOL read by pipeline ${digest}`,
    'POOL 1 entry across 1 pipeline generation',
  ];
}

await describe({
  name: 'damage-sample as built',
  children: [
    it({
      name: 'REFUSES as stated and exits 6, naming the key variables, when a region is drawn and no provider key is set, '
        + 'after printing the pool',
      fn: async () => {
        const {
          run,
          held,
        } = await drawOverOneEntry({ shippedSlices: 1, },);

        expect(run,).toEqual({
          code: REFUSED_AS_STATED,
          stdout: [
            ...await censusLines(),
            'DAMAGE pool 1 shipped region across both lanes, seed damage-round-one',
            'DAMAGE 0 shipped rows had no incumbent wording and are not drawn from',
            '',
          ].join('\n',),
          stderr: 'damage-sample: TRANSLATION_REPAIR_NEURALWATT_API_KEY, TRANSLATION_REPAIR_SYNTHETIC_API_KEY, '
            + 'TRANSLATION_REPAIR_CHARM_HYPER_API_KEY, '
            + 'TRANSLATION_REPAIR_AMAZON_BEDROCK_API_KEY, DEEPSEEK_API_KEY, TRANSLATION_REPAIR_XIAOMI_MIMO_API_KEY, TRANSLATION_REPAIR_CLAUDE_API_KEY '
            + 'or TRANSLATION_REPAIR_OPENROUTER_API_KEY is not set; '
            + 'run under mise so sops injects it\n',
        },);
        expect(held,).toEqual(['artifacts',],);
      },
    },),
    it({
      name: 'REFUSES a command-line argument the command does not read, exits 6 and prints the refusal alone',
      fn: async () => {
        const run = await runBuiltCommand({
          command: 'damage-sample',
          packageEntry: PROBE_ENTRY,
          args: ['extra',],
          env: {},
        },);

        expect(run,).toEqual({
          code: REFUSED_AS_STATED,
          stdout: '',
          stderr: 'damage-sample: this command takes no argument but its flags, and was given "extra". '
            + 'Usage: damage-sample\n',
        },);
      },
    },),
    it({
      name: 'REFUSES as stated and exits 6 when no entry has settled, writing no sheet',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'damage-sample-built-', },);
        await mkdir(join(
          scratch.path,
          'artifacts',
        ),);

        const run = await runBuiltCommand({
          command: 'damage-sample',
          packageEntry: PROBE_ENTRY,
          args: [],
          env: { TRANSLATION_REPAIR_RUNS_DIR: scratch.path, },
        },);

        expect(run.code,).toBe(REFUSED_AS_STATED,);
        expect(run.stdout,).toBe('',);
        expect(run.stderr.startsWith('damage-sample: No entry has settled yet, so there is nothing to pool.\n',),).toBe(true,);
        expect(await readdir(scratch.path,),).toEqual(['artifacts',],);
      },
    },),
    it({
      name: 'REFUSES as stated and exits 6 when the settled entries ship no replacement, writing no sheet that would '
        + 'block the next run',
      fn: async () => {
        const {
          run,
          held,
        } = await drawOverOneEntry({ shippedSlices: 0, },);

        expect(run,).toEqual({
          code: REFUSED_AS_STATED,
          stdout: [
            ...await censusLines(),
            '',
          ].join('\n',),
          stderr: 'damage-sample: the settled entries ship no replacement over an archive wording to draw from '
            + '(0 shipped rows had no incumbent wording and are not drawn from), so no sheet '
            + 'is written; a sheet with no item would be kept and refuse the next run\n',
        },);
        expect(held,).toEqual(['artifacts',],);
      },
    },),
    it({
      name: 'REFUSES A REQUIRED COMMIT AND A MIXED POOL ASKED FOR TOGETHER in its own words, read from the '
        + 'environment it runs in, exiting 6 and writing nothing',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'damage-sample-built-', },);
        await mkdir(join(
          scratch.path,
          'artifacts',
        ),);

        const run = await runBuiltCommand({
          command: 'damage-sample',
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
          stderr: `damage-sample: ${CONFLICTING_POOL_SAYS}\n`,
        },);
        expect(await readdir(scratch.path,),).toEqual(['artifacts',],);
      },
    },),
    it({
      name: 'REFUSES as stated and exits 6, naming the missing directory, when the runs directory holds no artifacts directory',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'damage-sample-built-', },);

        const run = await runBuiltCommand({
          command: 'damage-sample',
          packageEntry: PROBE_ENTRY,
          args: [],
          env: { TRANSLATION_REPAIR_RUNS_DIR: scratch.path, },
        },);

        expect(run,).toEqual({
          code: REFUSED_AS_STATED,
          stdout: '',
          stderr: `damage-sample: there is no artifacts directory at ${join(scratch.path, 'artifacts',)}; name the `
            + 'runs directory a pass settled entries into with TRANSLATION_REPAIR_RUNS_DIR\n',
        },);
      },
    },),
  ],
},);
