/**
 Tests for the slice census command at its boundary: the built command run in
 a child process whose environment carries no provider key, over a throwaway
 corpus repository the case builds and a scratch runs directory, so nothing
 reads the operator's corpus clone and nothing asks a model (the census asks
 none).

 The child is started by `runBuiltCommand`, which removes every variable whose
 name ends in `_API_KEY` and every one whose name starts `TRANSLATION_REPAIR_`
 before it adds the three the case names: the runs directory, the corpus clone
 and the corpus commit.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { runBuiltCommand, } from '@monochromatic-dev/translation-repair-adjudication/ts/child-environment.test-fixture.ts';
import { SLICE_CACHE_ENTRY, } from './built-entry.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import {
  FIRST_SECTION_ONLY_TARGET,
  makeCensusCorpus,
  ONE_BLOCK_PLUS_ADDED_TARGET,
  ONE_BLOCK_SOURCE,
  THREE_SECTION_SOURCE,
  THREE_SECTION_TARGET,
} from '@monochromatic-dev/translation-repair-adjudication/ts/corpus-run/slice-census-corpus.test-fixture.ts';

/**
 Exit code `reportingRefusals` sets for a stated refusal.
 */
const REFUSED_AS_STATED = 6;

await describe({
  name: 'slice-census as built',
  children: [
    it({
      name: 'PRINTS THE CENSUS of every complete pair at the pinned commit and exits 0, counting the entry that '
        + 'lacks its original as incomplete',
      fn: async () => {
        await using corpus = await makeCensusCorpus({
          files: {
            'people/mochi/page.md': THREE_SECTION_SOURCE,
            'people/mochi/page.en.md': THREE_SECTION_TARGET,
            'people/nori/page.md': THREE_SECTION_SOURCE,
            'people/nori/page.en.md': FIRST_SECTION_ONLY_TARGET,
            'people/tama/page.md': ONE_BLOCK_SOURCE,
            'people/tama/page.en.md': ONE_BLOCK_PLUS_ADDED_TARGET,
            'people/yuzu/page.en.md': THREE_SECTION_TARGET,
          },
        },);
        await using runs = await scratchDir({ prefix: 'slice-census-built-runs-', },);

        /**
         What the command wrote and how it ended.
         */
        const run = await runBuiltCommand({
          command: 'slice-census',
          packageEntry: SLICE_CACHE_ENTRY,
          env: {
            TRANSLATION_REPAIR_RUNS_DIR: runs.path,
            TRANSLATION_REPAIR_CORPUS_CLONE_DIR: corpus.cloneDir,
            TRANSLATION_REPAIR_CORPUS_COMMIT: corpus.commitSha,
          },
        },);

        expect(run,).toEqual({
          code: 0,
          stdout: [
            'CENSUS complete pairs: 3, incomplete: 1, slices: 4',
            'CENSUS carve: 0 settled entries with a complete recipe, 0 settled with a defaulted half, 0 settled with a recorded block pairing that does not fit the text, carved by the deterministic aligner, 3 deterministic baseline (0 of those hold a legacy artifact)',
            'CENSUS slice source chars: n 4, p50 24, p90 26, p99 26, max 26',
            'CENSUS slice target chars: n 4, p50 94, p90 110, p99 110, max 110',
            'CENSUS unpaired sections reaching no slice: source 3, target 1; entries: 1; chars: source 65, target 47',
            'CENSUS   nori: source sections 3 (chars: 65), target sections 1 (chars: 47)',
            'CENSUS target-only blocks: 1; entries: 1; chars: 39',
            'CENSUS   tama: blocks 1, chars 39',
            'CENSUS target-only block chars: n 1, p50 39, p90 39, p99 39, max 39',
            'CENSUS slices over 4641 target chars: 0 of 4',
            'CENSUS   widest tama: chars in one slice: 110',
            'CENSUS   widest mochi: chars in one slice: 94',
            'CENSUS   widest nori: chars in one slice: 0',
            '',
          ].join('\n',),
          stderr: '',
        },);
      },
    },),
    it({
      name: 'PRINTS THE CENSUS OF A CORPUS WITH NO ENTRY and exits 0',
      fn: async () => {
        await using corpus = await makeCensusCorpus({ files: { 'README.md': 'cats\n', }, },);
        await using runs = await scratchDir({ prefix: 'slice-census-built-runs-', },);

        /**
         What the command wrote and how it ended.
         */
        const run = await runBuiltCommand({
          command: 'slice-census',
          packageEntry: SLICE_CACHE_ENTRY,
          env: {
            TRANSLATION_REPAIR_RUNS_DIR: runs.path,
            TRANSLATION_REPAIR_CORPUS_CLONE_DIR: corpus.cloneDir,
            TRANSLATION_REPAIR_CORPUS_COMMIT: corpus.commitSha,
          },
        },);

        expect(run,).toEqual({
          code: 0,
          stdout: [
            'CENSUS complete pairs: 0, incomplete: 0, slices: 0',
            'CENSUS carve: 0 settled entries with a complete recipe, 0 settled with a defaulted half, 0 settled with a recorded block pairing that does not fit the text, carved by the deterministic aligner, 0 deterministic baseline (0 of those hold a legacy artifact)',
            'CENSUS slice source chars: n 0, p50 0, p90 0, p99 0, max 0',
            'CENSUS slice target chars: n 0, p50 0, p90 0, p99 0, max 0',
            'CENSUS unpaired sections reaching no slice: source 0, target 0; entries: 0; chars: source 0, target 0',
            'CENSUS target-only blocks: 0; entries: 0; chars: 0',
            'CENSUS target-only block chars: n 0, p50 0, p90 0, p99 0, max 0',
            'CENSUS slices over 4641 target chars: 0 of 0',
            '',
          ].join('\n',),
          stderr: '',
        },);
      },
    },),
    it({
      name: 'REFUSES AN ARGUMENT it does not read as stated and exits 6 with its line, before reading the corpus',
      fn: async () => {
        await using runs = await scratchDir({ prefix: 'slice-census-built-runs-', },);

        /**
         What the command wrote and how it ended.
         */
        const run = await runBuiltCommand({
          command: 'slice-census',
          packageEntry: SLICE_CACHE_ENTRY,
          args: ['--whiskers',],
          env: { TRANSLATION_REPAIR_RUNS_DIR: runs.path, },
        },);

        expect(run.code,).toBe(REFUSED_AS_STATED,);
        expect({
          stdout: run.stdout,
          stderr: run.stderr,
        },).toEqual({
          stdout: '',
          stderr: 'slice-census: --whiskers is not a flag this command reads. Usage: slice-census\n',
        },);
      },
    },),
  ],
},);
