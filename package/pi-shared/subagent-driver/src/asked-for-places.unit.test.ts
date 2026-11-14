/**
 The places asked for: every entry of a batch list whose unit is still active and whose run has not appeared, counted against the
 level from the moment the batch was asked for. The record it is written from is a filling at a level of 24 that passed it by four,
 because a review it had asked for waited in a launcher with no unit yet.

 @module
 */

import {
  mkdir,
  mkdtemp,
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  batchListOfAUnit,
  type CommandOutput,
  placesAskedFor,
} from '../dist/final/node/index.mjs';

/**
 A folder a test may write in, with the runs it lists, removed when the test ends.
 */
type AThrowawayFolder = {
  /**
   The folder the runs live in.
   */
  readonly runs: string;

  /**
   Removes the folder and everything under it.
   */
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 A folder a test may write in, with the runs it lists, removed when the test ends.

 @returns the folder
 */
async function makeFolder(): Promise<AThrowawayFolder> {
  /**
   The folder's path.
   */
  const path = await mkdtemp(join(scratchRoot(), 'pi-asked-for-',),);
  /**
   The folder the runs live in.
   */
  const runs = join(path, 'runs',);
  await mkdir(runs, { recursive: true, },);
  return {
    runs,
    async [Symbol.asyncDispose]() {
      await rm(path, { recursive: true, force: true, },);
    },
  };
}

/**
 Answers the manager's own listing, which is the only question this asks.

 @param units - the unit names the listing prints

 @param refuses - whether the manager refuses to answer

 @returns the answer
 */
function systemdListing({
  units,
  refuses,
}: {
  readonly units: readonly string[];
  readonly refuses?: boolean;
},): () => Promise<CommandOutput> {
  return function run(): Promise<CommandOutput> {
    return Promise.resolve((refuses === true)
      ? {
        status: 1,
        stdout: '',
        stderr: 'the manager is not there',
      }
      : {
        status: 0,
        stdout: units.map(function asALine(unit,) {
          return `${unit} loaded active running a batch`;
        },).join('\n',),
        stderr: '',
      },);
  };
}

/**
 A run entry of a batch list, named after the word given.

 @param name - the run's name

 @returns the entry
 */
function runNamed(name: string,): Record<string, unknown> {
  return {
    name,
    branch: `feat/${name}`,
    prompt: `/runs/${name}-prompt.md`,
    wall: 28_800,
  };
}

await describe({
  name: '',
  children: [
    describe({
      name: batchListOfAUnit.name,
      children: [
        it({
          name: 'names the list beside the runs that a launch unit starts',
          fn: async () => {
            expect(batchListOfAUnit({
              unitName: 'launch-batch-20261114T092114-1-queue-entry-a.service',
              runsFolder: '/runs',
            },),).toBe('/runs/batch-20261114T092114-1-queue-entry-a.json',);
          },
        },),
      ],
    },),
    describe({
      name: placesAskedFor.name,
      children: [
        it({
          name: 'counts every entry of an active batch whose run has not appeared, holding every pool it could fill',
          fn: async () => {
            await using folder = await makeFolder();
            await writeFile(
              join(folder.runs, 'batch-1-queue-entry-a.json',),
              JSON.stringify([runNamed('a',)],),
            );
            /**
             What the look found.
             */
            const places = await placesAskedFor({
              runsFolder: folder.runs,
              run: systemdListing({ units: ['launch-batch-1-queue-entry-a.service',], },),
              goingNames: new Set(),
              onProblem: function onProblem() {
                throw new Error('nothing should be a problem here',);
              },
            },);
            expect(places,).toHaveLength(1,);
            expect(places[0]?.name,).toBe('a',);
            expect(places[0]?.membership.holds,).toBe('some-pools',);
          },
        },),
        it({
          name: 'leaves out an entry whose own run has appeared, because its unit is what the count reads',
          fn: async () => {
            await using folder = await makeFolder();
            await writeFile(
              join(folder.runs, 'batch-1-queue-entry-a.json',),
              JSON.stringify([runNamed('a',), runNamed('b',)],),
            );
            /**
             What the look found while a's run is going.
             */
            const places = await placesAskedFor({
              runsFolder: folder.runs,
              run: systemdListing({ units: ['launch-batch-1-queue-entry-a.service',], },),
              goingNames: new Set(['a',],),
              onProblem: function onProblem() {
                throw new Error('nothing should be a problem here',);
              },
            },);
            expect(places.map(function nameOf(place,) {
              return place.name;
            },),).toEqual(['b',],);
          },
        },),
        it({
          name: 'names no place for a launch unit that starts no batch list, and reports a list that is there and cannot be read',
          fn: async () => {
            await using folder = await makeFolder();
            await writeFile(
              join(folder.runs, 'batch-2-queue-entry-c.json',),
              'not json\n',
            );
            /**
             What the look reported.
             */
            const problems: string[] = [];
            /**
             What the look found.
             */
            const places = await placesAskedFor({
              runsFolder: folder.runs,
              run: systemdListing({
                units: [
                  'launch-review-20261113T160734-a.service',
                  'launch-batch-2-queue-entry-c.service',
                ],
              },),
              goingNames: new Set(),
              onProblem: function onProblem(problem,) {
                problems.push(problem,);
              },
            },);
            expect(places,).toEqual([],);
            expect(problems,).toHaveLength(1,);
            expect(problems[0],).toContain('launch-batch-2-queue-entry-c',);
          },
        },),
        it({
          name: 'counts no place and says so when the manager cannot list its units',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             What the look reported.
             */
            const problems: string[] = [];
            expect(await placesAskedFor({
              runsFolder: folder.runs,
              run: systemdListing({
                units: [],
                refuses: true,
              },),
              goingNames: new Set(),
              onProblem: function onProblem(problem,) {
                problems.push(problem,);
              },
            },),).toEqual([],);
            expect(problems[0],).toContain('could not be listed',);
          },
        },),
      ],
    },),
  ],
},);
