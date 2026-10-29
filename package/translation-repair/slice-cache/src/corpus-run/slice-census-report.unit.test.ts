/**
 Tests for the slice census report: what it reads, in what order, and every
 line it prints.

 The corpus is a throwaway git repository the cases build, and the runs
 directory and the settled recipes are scripted, so nothing reads the
 operator's corpus clone or runs directory.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { CorpusReadError, } from '@monochromatic-dev/translation-repair-corpus-source';
import {
  reportSliceCensus,
  type SliceCensusRecipeReading,
} from '../../dist/final/node/index.mjs';
import { divertingConsoleLog, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/console-log-capture.test-fixture.ts';
import {
  FIRST_SECTION_ONLY_TARGET,
  makeCensusCorpus,
  ONE_BLOCK_PLUS_ADDED_TARGET,
  ONE_BLOCK_SOURCE,
  THREE_SECTION_SOURCE,
  THREE_SECTION_TARGET,
} from '@monochromatic-dev/translation-repair-adjudication/ts/corpus-run/slice-census-corpus.test-fixture.ts';

/**
 Runs directory every case's scripted resolver answers.
 */
const RUNS_DIR = '/nowhere/cat-runs';

/**
 Reading of an entry no artifact records.

 @returns Unsettled reading

 @example
 ```ts
 const reading = await unsettled();
 ```
 */
async function unsettled(): Promise<SliceCensusRecipeReading> {
  return { kind: 'unsettled', };
}

/**
 Runs directory the cases' scripted resolver finds.

 @returns Fixed directory

 @example
 ```ts
 const runsDir = await resolveFixedRuns();
 ```
 */
async function resolveFixedRuns(): Promise<string> {
  return RUNS_DIR;
}

await describe({
  name: reportSliceCensus.name,
  children: [
    it({
      name: 'PRINTS THE CENSUS of every complete pair, reading each recipe from the resolved runs directory',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
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

        /**
         Where each recipe was read from, by entry.
         */
        const reads: string[] = [];
        await reportSliceCensus({
          pin: {
            cloneDir: corpus.cloneDir,
            commitSha: corpus.commitSha,
          },
          resolveRuns: resolveFixedRuns,
          readRecipe: async function readRecipeIn(
            { entryId, runsDir, },
          ): Promise<SliceCensusRecipeReading> {
            reads.push(`${entryId} in ${runsDir}`,);
            return await unsettled();
          },
        },);

        expect(reads,).toEqual([
          `mochi in ${RUNS_DIR}`,
          `nori in ${RUNS_DIR}`,
          `tama in ${RUNS_DIR}`,
          `yuzu in ${RUNS_DIR}`,
        ],);
        expect(printed.lines,).toEqual([
          'CENSUS complete pairs: 3, incomplete: 1, slices: 4',
          'CENSUS carve: 0 settled entries with a complete recipe, 0 settled with a defaulted half, '
            + '0 settled with a recorded block pairing that does not fit the text, carved by the deterministic '
            + 'aligner, 3 deterministic baseline (0 of those hold a legacy artifact)',
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
        ],);
      },
    },),
    it({
      name: 'COUNTS EVERY LEGACY ENTRY on the carve line, including one found incomplete, and counts a settled entry '
        + 'on its own carve',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        await using corpus = await makeCensusCorpus({
          files: {
            'people/mochi/page.md': THREE_SECTION_SOURCE,
            'people/mochi/page.en.md': THREE_SECTION_TARGET,
            'people/tama/page.md': ONE_BLOCK_SOURCE,
            'people/tama/page.en.md': ONE_BLOCK_PLUS_ADDED_TARGET,
            'people/yuzu/page.en.md': THREE_SECTION_TARGET,
          },
        },);
        await reportSliceCensus({
          pin: {
            cloneDir: corpus.cloneDir,
            commitSha: corpus.commitSha,
          },
          resolveRuns: resolveFixedRuns,
          readRecipe: async function readRecipeIn({ entryId, },): Promise<SliceCensusRecipeReading> {
            if (entryId === 'tama') {
              return {
                kind: 'settled',
                corpusSha: corpus.commitSha,
                recipe: {
                  blockPairings: new Map([[
                    0,
                    [
                      {
                        source: 0,
                        target: 0,
                      },
                      {
                        source: 1,
                        target: 1,
                      },
                    ],
                  ],],),
                  unrecorded: ['sectionPairing',],
                },
              };
            }
            return { kind: 'legacy', };
          },
        },);

        expect(printed.lines.slice(0, 2,),).toEqual([
          'CENSUS complete pairs: 2, incomplete: 1, slices: 4',
          'CENSUS carve: 0 settled entries with a complete recipe, 1 settled with a defaulted half, '
            + '0 settled with a recorded block pairing that does not fit the text, carved by the deterministic '
            + 'aligner, 1 deterministic baseline (2 of those hold a legacy artifact)',
        ],);
      },
    },),
    it({
      name: 'PRINTS THE CENSUS OF A CORPUS WITH NO ENTRY',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        await using corpus = await makeCensusCorpus({ files: { 'README.md': 'cats\n', }, },);
        await reportSliceCensus({
          pin: {
            cloneDir: corpus.cloneDir,
            commitSha: corpus.commitSha,
          },
          resolveRuns: resolveFixedRuns,
          readRecipe: unsettled,
        },);

        expect(printed.lines,).toEqual([
          'CENSUS complete pairs: 0, incomplete: 0, slices: 0',
          'CENSUS carve: 0 settled entries with a complete recipe, 0 settled with a defaulted half, '
            + '0 settled with a recorded block pairing that does not fit the text, carved by the deterministic '
            + 'aligner, 0 deterministic baseline (0 of those hold a legacy artifact)',
          'CENSUS slice source chars: n 0, p50 0, p90 0, p99 0, max 0',
          'CENSUS slice target chars: n 0, p50 0, p90 0, p99 0, max 0',
          'CENSUS unpaired sections reaching no slice: source 0, target 0; entries: 0; chars: source 0, target 0',
          'CENSUS target-only blocks: 0; entries: 0; chars: 0',
          'CENSUS target-only block chars: n 0, p50 0, p90 0, p99 0, max 0',
          'CENSUS slices over 4641 target chars: 0 of 0',
        ],);
      },
    },),
    it({
      name: 'REFUSES A CLONE IT CANNOT LIST before resolving the runs directory, and prints nothing',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        await using corpus = await makeCensusCorpus({ files: { 'README.md': 'cats\n', }, },);

        /**
         Whether the runs directory was asked for.
         */
        const asked = { runs: false, };
        try {
          await reportSliceCensus({
            pin: {
              cloneDir: `${corpus.cloneDir}-gone`,
              commitSha: corpus.commitSha,
            },
            resolveRuns: async function resolveNone(): Promise<string> {
              asked.runs = true;
              return await resolveFixedRuns();
            },
            readRecipe: unsettled,
          },);
        }
        catch (error) {
          expect(error,).toBeInstanceOf(CorpusReadError,);
          expect({
            asked,
            printed: printed.lines,
          },).toEqual({
            asked: { runs: false, },
            printed: [],
          },);
          return;
        }
        throw new Error('the census listed a clone that is not there',);
      },
    },),
    it({
      name: 'STOPS WITH THE RESOLVER\'S OWN REFUSAL where no runs directory resolves, and prints nothing',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        await using corpus = await makeCensusCorpus({ files: { 'README.md': 'cats\n', }, },);
        try {
          await reportSliceCensus({
            pin: {
              cloneDir: corpus.cloneDir,
              commitSha: corpus.commitSha,
            },
            resolveRuns: async function resolveNone(): Promise<string> {
              throw new Error('no runs directory for the cats',);
            },
            readRecipe: unsettled,
          },);
        }
        catch (error) {
          expect(String(error,),).toBe('Error: no runs directory for the cats',);
          expect(printed.lines,).toEqual([],);
          return;
        }
        throw new Error('the census went on with no runs directory',);
      },
    },),
  ],
},);
