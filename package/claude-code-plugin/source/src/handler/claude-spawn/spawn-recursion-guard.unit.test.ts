/**
 Tests for the recursion guard the `spawn-claude` CLI applies to a spawn chain.
 
 A session at the limit, or one whose depth marker cannot be read, must be
 refused before anything is launched: A guard that only reports the problem
 after opening a terminal window has not stopped the runaway it exists for.
 
 @module
 */

import { readFileSync, } from 'node:fs';
import { resolve, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  DEFAULT_MAX_SPAWN_DEPTH,
  MAX_SPAWN_DEPTH_ENV,
  maxSpawnDepth,
  SPAWN_DEPTH_ENV,
  spawnChainDecision,
  spawnDepth,
} from './spawn-recursion-guard.ts';

/**
 Repository root, resolved from this test file's directory so the document read
 follows the checkout rather than the test runner's working directory.
 */
const REPO_ROOT = resolve(import.meta.dirname, '../../../../../..',);

/**
 Plugin documentation that must state the limit and the variable that changes it.
 */
const README_PATH = resolve(
  REPO_ROOT,
  'package/claude-code-plugin/claude-spawn/README.md',
);

//region Fixtures

/**
 Environment a session at the top of a chain runs in.
 */
const ROOT_ENV = {} satisfies NodeJS.ProcessEnv;

/**
 Settings that must leave the default limit in place.
 */
const SETTINGS_THAT_MUST_NOT_REMOVE_THE_GUARD: readonly string[] = [
  '',
  '   ',
  '0',
  '-2',
  '1.5',
  'five',
  'Infinity',
];

/**
 Markers that are not a count of spawn generations.
 */
const UNREADABLE_MARKERS: readonly string[] = [
  '-1',
  '1.5',
  'Infinity',
  'NaN',
  'two',
];

/**
 Builds the environment of a session a given number of spawn generations deep.
 
 @param generations - depth the marker carries
 
 @param limit - limit override, left unset when omitted
 
 @returns environment object the guard reads
 */
function envAtDepth({
  generations,
  limit,
}: {
  readonly generations: string;
  readonly limit?: string;
}): NodeJS.ProcessEnv {
  return {
    [SPAWN_DEPTH_ENV]: generations,
    ...(limit === undefined
      ? {}
      : { [MAX_SPAWN_DEPTH_ENV]: limit }),
  };
}

/**
 Reads the refusal text of a decision that must refuse.
 
 @param decision - decision under test
 
 @returns the diagnostic text, failing the case when the attempt was allowed
 */
function refusalOf(decision: ReturnType<typeof spawnChainDecision>,): string {
  if (!('diagnostic' in decision))
    throw new Error(`expected a refusal, got ${JSON.stringify(decision,)}`,);
  return decision.diagnostic;
}

//endregion Fixtures

await describe({
  name: '',
  children: [
    describe({
      name: spawnDepth.name,
      children: [
        //region Values that name a generation

        it({
          name: 'reads an unset or blank marker as the top of a chain',
          fn: async () => {
            expect(spawnDepth('',),).toBe(0,);
            expect(spawnDepth('  ',),).toBe(0,);
          },
        },),
        it({
          name: 'counts the generations a marker names',
          fn: async () => {
            expect(spawnDepth('0',),).toBe(0,);
            expect(spawnDepth('2',),).toBe(2,);
          },
        },),
        it({
          name: 'ignores the whitespace of a shell assignment',
          fn: async () => {
            expect(spawnDepth(' 2 ',),).toBe(2,);
          },
        },),

        //endregion Values that name a generation
      ],
    },),
    describe({
      name: maxSpawnDepth.name,
      children: [
        //region Settings that name a limit

        it({
          name: 'reads a whole count as the limit',
          fn: async () => {
            expect(maxSpawnDepth('5',),).toBe(5,);
            expect(maxSpawnDepth(' 5 ',),).toBe(5,);
          },
        },),

        //endregion Settings that name a limit

        //region Settings that leave the default in place

        ...SETTINGS_THAT_MUST_NOT_REMOVE_THE_GUARD.map(function mapSetting(rawSetting,) {
          return it({
            name: `falls back on the default for ${JSON.stringify(rawSetting,)}`,
            fn: async () => {
              expect(maxSpawnDepth(rawSetting,),).toBe(DEFAULT_MAX_SPAWN_DEPTH,);
            },
          },);
        },),

        //endregion Settings that leave the default in place
      ],
    },),
    describe({
      name: spawnChainDecision.name,
      children: [
        //region Attempts the guard allows

        it({
          name: 'allows the first child of a session at the top of a chain',
          fn: async () => {
            expect(spawnChainDecision(ROOT_ENV,),).toEqual({ childDepth: 1, },);
          },
        },),
        it({
          name: 'counts one generation below the parent that spawns',
          fn: async () => {
            expect(
              spawnChainDecision(envAtDepth({ generations: '0', },),),
            ).toEqual({ childDepth: 1, },);
            expect(
              spawnChainDecision(envAtDepth({ generations: '2', },),),
            ).toEqual({ childDepth: DEFAULT_MAX_SPAWN_DEPTH, },);
          },
        },),
        it({
          name: 'allows the last generation under the default limit',
          fn: async () => {
            expect(
              spawnChainDecision(
                envAtDepth({ generations: String(DEFAULT_MAX_SPAWN_DEPTH - 1,), },),
              ),
            ).toEqual({ childDepth: DEFAULT_MAX_SPAWN_DEPTH, },);
          },
        },),
        it({
          name: 'allows a chain the override raises the limit for',
          fn: async () => {
            expect(
              spawnChainDecision(
                envAtDepth({
                  generations: '4',
                  limit: '5',
                },),
              ),
            ).toEqual({ childDepth: 5, },);
          },
        },),

        //endregion Attempts the guard allows

        //region Attempts the guard refuses

        it({
          name: 'refuses at the default limit and names the limit and the override',
          fn: async () => {
            /**
             Refusal text for a session sitting exactly at the default limit.
             */
            const diagnostic = refusalOf(
              spawnChainDecision(
                envAtDepth({ generations: String(DEFAULT_MAX_SPAWN_DEPTH,), },),
              ),
            );
            expect(diagnostic,)
              .toContain(`${String(DEFAULT_MAX_SPAWN_DEPTH,)} spawn generations deep`,);
            expect(diagnostic,)
              .toContain(MAX_SPAWN_DEPTH_ENV,);
            expect(diagnostic,)
              .toContain('No terminal was opened',);
          },
        },),
        it({
          name: 'refuses past the limit rather than only at it',
          fn: async () => {
            expect(
              'diagnostic'
                in spawnChainDecision(
                  envAtDepth({ generations: String(DEFAULT_MAX_SPAWN_DEPTH + 4,), },),
                ),
            ).toBe(true,);
          },
        },),
        it({
          name: 'refuses when the override lowers the limit to one generation',
          fn: async () => {
            expect(
              'diagnostic'
                in spawnChainDecision(
                  envAtDepth({
                    generations: '1',
                    limit: '1',
                  },),
                ),
            ).toBe(true,);
          },
        },),
        it({
          name: 'refuses a marker that is not a generation count and says how to clear it',
          fn: async () => {
            /**
             Refusal text for a corrupt marker.
             */
            const diagnostic = refusalOf(
              spawnChainDecision(envAtDepth({ generations: 'two', },),),
            );
            expect(diagnostic,)
              .toContain(SPAWN_DEPTH_ENV,);
            expect(diagnostic,)
              .toContain(JSON.stringify('two',),);
            expect(diagnostic,)
              .toContain(`Unset ${SPAWN_DEPTH_ENV}`,);
          },
        },),
        ...UNREADABLE_MARKERS.map(function mapUnreadableMarker(generations,) {
          return it({
            name: `refuses the marker ${JSON.stringify(generations,)}`,
            fn: async () => {
              expect(
                'diagnostic'
                  in spawnChainDecision(envAtDepth({ generations, },),),
              ).toBe(true,);
            },
          },);
        },),
        it({
          name: 'refuses even when a corrupt marker sits beside a raised limit',
          fn: async () => {
            expect(
              'diagnostic'
                in spawnChainDecision(
                  envAtDepth({
                    generations: 'many',
                    limit: '10',
                  },),
                ),
            ).toBe(true,);
          },
        },),

        //endregion Attempts the guard refuses
      ],
    },),
    describe({
      name: 'plugin documentation',
      children: [
        it({
          name: 'states the limit, the marker, the variable that raises it, and no unbounded promise',
          fn: async () => {
            /**
             Plugin README text, read once for every documented claim below.
             */
            const readme = readFileSync(
              README_PATH,
              'utf8',
            );
            expect(readme,)
              .toContain(`nest up to ${String(DEFAULT_MAX_SPAWN_DEPTH,)} spawn generations deep`,);
            expect(readme,)
              .toContain(`\`${SPAWN_DEPTH_ENV}\``,);
            expect(readme,)
              .toContain(`\`${MAX_SPAWN_DEPTH_ENV}\``,);
            expect(readme,)
              .not
              .toContain('Arbitrary nesting depth is supported',);
          },
        },),
      ],
    },),
  ],
},);
