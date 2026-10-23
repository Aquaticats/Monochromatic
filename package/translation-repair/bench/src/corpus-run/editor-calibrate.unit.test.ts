/**
 Tests for the editor calibration runner: the built command, run in a child
 process with every provider key withheld and its corpus read from a
 throwaway clone, so it can reach its first refusal and no model.

 THE CHILD'S ENVIRONMENT. `runBuiltCommand` drops every variable whose
 name ends in `_API_KEY` and every variable whose name starts
 `TRANSLATION_REPAIR_` before a case adds the locations it needs
 (`environmentWithoutKeys` in `child-environment.test-fixture.ts`).

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { RUN_ROSTER, } from '@monochromatic-dev/translation-repair-run';
import { EDITOR_LANE_EDITORS, } from '../../dist/final/node/index.mjs';
import { runBuiltCommand, } from '@monochromatic-dev/translation-repair-adjudication/ts/child-environment.test-fixture.ts';
import { BENCH_ENTRY, } from './built-entry.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { makeBenchClone, } from '@monochromatic-dev/translation-repair-corpus-pass/ts/corpus-run/editor-bench-clone.test-fixture.ts';

/**
 Exit code `reportingRefusals` sets for a stated refusal.
 */
const REFUSED_AS_STATED = 6;

/**
 Line the command prints when no provider key is set.
 */
const NO_KEY_LINE = 'editor-calibrate: TRANSLATION_REPAIR_NEURALWATT_API_KEY, TRANSLATION_REPAIR_SYNTHETIC_API_KEY, '
  + 'TRANSLATION_REPAIR_CHARM_HYPER_API_KEY, '
  + 'TRANSLATION_REPAIR_AMAZON_BEDROCK_API_KEY, DEEPSEEK_API_KEY, TRANSLATION_REPAIR_XIAOMI_MIMO_API_KEY, TRANSLATION_REPAIR_CLAUDE_API_KEY or '
  + 'TRANSLATION_REPAIR_OPENROUTER_API_KEY is not set; '
  + 'run under mise so sops injects it\n';

/**
 Models the lane seats as editors and as judges, which the header counts.
 */
const EDITORS = String(EDITOR_LANE_EDITORS.length,);
const JUDGES = String(RUN_ROSTER.length,);

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: 'editor-calibrate as built',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'REFUSES as stated and exits 6 for want of a provider key, after the sample and the header and '
            + 'before any call',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'editor-calibrate-', },);
            await using clone = await makeBenchClone();

            /**
             What the command wrote with every key withheld.
             */
            const run = await runBuiltCommand({
              command: 'editor-calibrate',
              packageEntry: BENCH_ENTRY,
              args: [],
              env: {
                TRANSLATION_REPAIR_RUNS_DIR: scratch.path,
                TRANSLATION_REPAIR_LOOKUP_CACHE_DIR: `${scratch.path}/lookup-cache`,
                TRANSLATION_REPAIR_CORPUS_CLONE_DIR: clone.path,
                TRANSLATION_REPAIR_CORPUS_COMMIT: clone.commitSha,
              },
            },);

            expect(run.code,).toBe(REFUSED_AS_STATED,);
            expect(run.stdout,).toBe(
              `editor-calibrate: 1 slice, ${EDITORS} models editing and ${JUDGES} judging each, 4 slices in flight\n`
                + 'straggler window 300000ms (calibration default)\n',
            );
            expect(run.stderr,).toBe(NO_KEY_LINE,);
          },
        },),

        it({
          name: 'PRINTS the override as the straggler window source and the writer window note when a launch sets both',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'editor-calibrate-', },);
            await using clone = await makeBenchClone();

            /**
             What the command wrote with both windows overridden.
             */
            const run = await runBuiltCommand({
              command: 'editor-calibrate',
              packageEntry: BENCH_ENTRY,
              args: ['3',],
              env: {
                TRANSLATION_REPAIR_RUNS_DIR: scratch.path,
                TRANSLATION_REPAIR_LOOKUP_CACHE_DIR: `${scratch.path}/lookup-cache`,
                TRANSLATION_REPAIR_CORPUS_CLONE_DIR: clone.path,
                TRANSLATION_REPAIR_CORPUS_COMMIT: clone.commitSha,
                TRANSLATION_REPAIR_SLICE_OVERLAP: '1',
                TRANSLATION_REPAIR_STRAGGLER_GRACE_MS: '1234',
                TRANSLATION_REPAIR_WRITER_GRACE_MS: '5678',
              },
            },);

            expect(run.code,).toBe(REFUSED_AS_STATED,);
            expect(run.stdout,).toBe(
              `editor-calibrate: 1 slice, ${EDITORS} models editing and ${JUDGES} judging each, 1 slice in flight\n`
                + 'straggler window 1234ms (TRANSLATION_REPAIR_STRAGGLER_GRACE_MS override)\n'
                + 'WRITER GRACE OVERRIDDEN by TRANSLATION_REPAIR_WRITER_GRACE_MS: writer rounds (editor, refiner, '
                + 'translate, produceConsolidations, archive-block-review) abandon stragglers 5678ms after quorum rather than the 1234ms '
                + 'every other round waits\n',
            );
            expect(run.stderr,).toBe(NO_KEY_LINE,);
          },
        },),

        it({
          name: 'REFUSES a count that is no whole number as stated and exits 6 with nothing on stdout',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'editor-calibrate-', },);

            /**
             What the command wrote given a count of letters.
             */
            const run = await runBuiltCommand({
              command: 'editor-calibrate',
              packageEntry: BENCH_ENTRY,
              args: ['four',],
              env: {
                TRANSLATION_REPAIR_RUNS_DIR: scratch.path,
                TRANSLATION_REPAIR_LOOKUP_CACHE_DIR: `${scratch.path}/lookup-cache`,
              },
            },);

            expect(run.code,).toBe(REFUSED_AS_STATED,);
            expect(run.stdout,).toBe('',);
            expect(run.stderr,).toBe(
              'editor-calibrate: slices must be a whole number written in digits, at most 9007199254740991, '
                + 'and "four" is not one\n',
            );
          },
        },),

        it({
          name: 'REFUSES an overlap variable that is no whole number as stated and exits 6 before the sample is drawn',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'editor-calibrate-', },);

            /**
             What the command wrote with the overlap variable misspelled as a word.
             */
            const run = await runBuiltCommand({
              command: 'editor-calibrate',
              packageEntry: BENCH_ENTRY,
              args: [],
              env: {
                TRANSLATION_REPAIR_RUNS_DIR: scratch.path,
                TRANSLATION_REPAIR_LOOKUP_CACHE_DIR: `${scratch.path}/lookup-cache`,
                TRANSLATION_REPAIR_SLICE_OVERLAP: 'several',
              },
            },);

            expect(run.code,).toBe(REFUSED_AS_STATED,);
            expect(run.stdout,).toBe('',);
            expect(run.stderr,).toBe(
              'editor-calibrate: TRANSLATION_REPAIR_SLICE_OVERLAP must be a whole number written in digits, '
                + 'at most 9007199254740991, and several is not one\n',
            );
          },
        },),

        it({
          name: 'REFUSES a flag it does not read as stated and exits 6, naming the flag and the usage',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'editor-calibrate-', },);

            /**
             What the command wrote given one flag it does not declare.
             */
            const run = await runBuiltCommand({
              command: 'editor-calibrate',
              packageEntry: BENCH_ENTRY,
              args: ['--bogus',],
              env: {
                TRANSLATION_REPAIR_RUNS_DIR: scratch.path,
                TRANSLATION_REPAIR_LOOKUP_CACHE_DIR: `${scratch.path}/lookup-cache`,
              },
            },);

            expect(run.code,).toBe(REFUSED_AS_STATED,);
            expect(run.stdout,).toBe('',);
            expect(run.stderr,).toBe(
              'editor-calibrate: --bogus is not a flag this command reads. Usage: editor-calibrate [<slices>]\n',
            );
          },
        },),

        it({
          name: 'REFUSES a corpus clone that does not exist as stated and exits 6 with nothing on stdout',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'editor-calibrate-', },);

            /**
             What the command wrote when its clone directory is absent.
             */
            const run = await runBuiltCommand({
              command: 'editor-calibrate',
              packageEntry: BENCH_ENTRY,
              args: [],
              env: {
                TRANSLATION_REPAIR_RUNS_DIR: scratch.path,
                TRANSLATION_REPAIR_LOOKUP_CACHE_DIR: `${scratch.path}/lookup-cache`,
                TRANSLATION_REPAIR_CORPUS_CLONE_DIR: `${scratch.path}/no-clone`,
                TRANSLATION_REPAIR_CORPUS_COMMIT: 'a'.repeat(40,),
              },
            },);

            expect(run.code,).toBe(REFUSED_AS_STATED,);
            expect(run.stdout,).toBe('',);
            expect(run.stderr,).toBe(
              'editor-calibrate: corpus read failed for people/ at aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa (unreadable-clone); '
                + 'git could not open the clone: check that the directory exists, is the top of a git repository, '
                + 'and is one git may read.\n',
            );
          },
        },),
      ],
    },),
  ],
},);
