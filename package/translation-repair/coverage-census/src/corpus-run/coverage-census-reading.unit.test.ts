/**
 Tests for what the census does with a painted tally once the suite has run:
 the placement it asks for, the census file it writes and the report it
 prints, then the baseline reading. The placement and the git question are
 handed in and scripted. Paths and names are cat-themed invention.

 @module
 */

import { readFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary';
import {
  createCoverageTally,
  type PlacedTally,
  reportCensus,
  runnerEntrySources,
} from '../../dist/final/node/index.mjs';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { divertingConsoleLog, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/console-log-capture.test-fixture.ts';
import { recorded, } from './coverage-census.test-fixture.ts';

/**
 Package directory every case names.
 */
const PACKAGE_DIRECTORY = '/cats/package';

/**
 Build directory every case names.
 */
const DIST_DIRECTORY = '/cats/package/dist/final/node';

/**
 A placed tally with one cold stretch in a library source and one in a
 runner's entry file.

 @returns Placed tally

 @example
 ```ts
 const placed = twoStretches();
 ```
 */
function twoStretches(): PlacedTally {
  return {
    stretches: [
      recorded({
        source: 'src/nap.ts',
        startLine: 3,
        endLine: 5,
      },),
      recorded({
        source: 'src/corpus-run/slice-census.ts',
        startLine: 1,
        endLine: 1,
      },),
    ],
    invariantThrows: [],
    uncalled: [],
    loadedSources: new Set(['src/nap.ts',],),
    unloadedBundles: [],
    unloadedSources: [],
  };
}

await describe({
  name: reportCensus.name,
  children: [
    it({
      name: 'ASKS FOR THE PLACEMENT of the tally with the entry files the build names, writes the census file '
        + 'whole and prints the report with its path',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        await using report = await scratchDir({ prefix: 'coverage-census-reading-', },);
        const tally = createCoverageTally();
        const bundleMaps = {
          mapped: ['nap.mjs',],
          unmapped: [],
        };

        /**
         What the placement was asked.
         */
        const placements: {
          readonly tally: unknown;
          readonly bundleMaps: unknown;
          readonly packageDirectory: string;
          readonly distDirectory: string;
          readonly entryFiles: unknown;
        }[] = [];
        await reportCensus({
          asked: {
            testFiles: ['src/nap.unit.test.ts',],
            baseline: [],
            sources: [],
          },
          packageDirectory: PACKAGE_DIRECTORY,
          distDirectory: DIST_DIRECTORY,
          bundleMaps,
          head: 'abc123def',
          clean: false,
          passes: 4,
          tally,
          reportDirectory: report.path,
          baselines: [],
          place: async function placed(input,): Promise<PlacedTally> {
            placements.push(input,);
            return twoStretches();
          },
          editedSince: async function editedNone(): Promise<ReadonlySet<string>> {
            return new Set<string>();
          },
        },);

        expect(placements,).toEqual([{
          packageDirectory: PACKAGE_DIRECTORY,
          distDirectory: DIST_DIRECTORY,
          bundleMaps,
          tally,
          entryFiles: runnerEntrySources(),
        },],);
        expect(placements[0]?.tally,).toBe(tally,);
        expect(await readFile(
          join(
            report.path,
            'census.json',
          ),
          'utf8',
        ),).toBe(JSON.stringify(
          {
            format: 3,
            head: 'abc123def',
            clean: false,
            testFiles: ['src/nap.unit.test.ts',],
            passes: 4,
            stretches: twoStretches().stretches,
            invariantThrows: [],
            uncalled: [],
            loadedSources: ['src/nap.ts',],
            unloadedBundles: [],
            unloadedSources: [],
          },
          null,
          1,
        ),);
        expect(printed.lines,).toEqual([
          'coverage-census at abc123def with uncommitted changes: 1 test file, 4 passes',
          'library source: 1 file, 1 stretch over 3 lines, 0 functions never called',
          'entry file: 1 file, 1 stretch over 1 line, 0 functions never called',
          'invariant throws, counted apart from the cold stretches: 0 stretches',
          'bundles no test loaded: 0, carrying 0 sources and 0 physical lines',
          'library source by cold lines:',
          '  src/nap.ts: 1 stretch, 3 lines, 0 never called',
          'invariant throws by source and line (each stretch nothing but throws of an Error whose message begins '
            + '"unreachable:" or of a class whose name ends in InvariantError), with what each throws:',
          'functions never called in package source, outermost:',
          `census written to ${join(
            report.path,
            'census.json',
          )}`,
        ],);
      },
    },),
    it({
      name: 'PRINTS EACH BASELINE\'S READING AFTER THE REPORT, asking git for the baseline\'s commit under the '
        + 'package directory',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        await using report = await scratchDir({ prefix: 'coverage-census-reading-', },);

        /**
         What git was asked.
         */
        const asked: string[] = [];
        await reportCensus({
          asked: {
            testFiles: [],
            baseline: ['/cats/before.json',],
            sources: ['src/nap.ts',],
          },
          packageDirectory: PACKAGE_DIRECTORY,
          distDirectory: DIST_DIRECTORY,
          bundleMaps: {
            mapped: ['nap.mjs',],
            unmapped: [],
          },
          head: 'abc123def',
          clean: true,
          passes: 1,
          tally: createCoverageTally(),
          reportDirectory: report.path,
          baselines: [{
            path: '/cats/before.json',
            census: {
              head: 'c0ffee123',
              stretches: [recorded({
                source: 'src/nap.ts',
                startLine: 3,
                endLine: 5,
              },),],
              loadedSources: new Set(['src/nap.ts',],),
            },
          },],
          place: async function placed(): Promise<PlacedTally> {
            return twoStretches();
          },
          editedSince: async function editedNone(
            { packageDirectory, head, },
          ): Promise<ReadonlySet<string>> {
            asked.push(`${head} in ${packageDirectory}`,);
            return new Set<string>();
          },
        },);

        expect(asked,).toEqual([`c0ffee123 in ${PACKAGE_DIRECTORY}`,],);
        expect(printed.lines.slice(-3,),).toEqual([
          `census written to ${join(
            report.path,
            'census.json',
          )}`,
          'against /cats/before.json at c0ffee123: ran 0, still cold 1, cold since then 0, not loaded 0, claimed '
            + 'sources with no stretch there 0, sources edited since then 0',
          '  still cold: src/nap.ts:3-5',
        ],);
      },
    },),
    it({
      name: 'WRITES NO CENSUS FILE AND PRINTS NOTHING where the placement refuses, passing its refusal on',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        await using report = await scratchDir({ prefix: 'coverage-census-reading-', },);
        try {
          await reportCensus({
            asked: {
              testFiles: [],
              baseline: [],
              sources: [],
            },
            packageDirectory: PACKAGE_DIRECTORY,
            distDirectory: DIST_DIRECTORY,
            bundleMaps: {
              mapped: [],
              unmapped: [],
            },
            head: 'abc123def',
            clean: true,
            passes: 1,
            tally: createCoverageTally(),
            reportDirectory: report.path,
            baselines: [],
            place: async function refused(): Promise<PlacedTally> {
              throw new StatedRefusalError({ says: 'the cats placed nothing', },);
            },
            editedSince: async function editedNone(): Promise<ReadonlySet<string>> {
              return new Set<string>();
            },
          },);
        }
        catch (error) {
          expect(String(error,),).toBe('StatedRefusalError: the cats placed nothing',);
          expect(printed.lines,).toEqual([],);
          await expect(readFile(
            join(
              report.path,
              'census.json',
            ),
            'utf8',
          ),).rejects.toThrow('ENOENT',);
          return;
        }
        throw new Error('the report went on past a placement that refused',);
      },
    },),
  ],
},);
