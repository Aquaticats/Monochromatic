/**
 What arms the guard, and what leaves it off: the folders a run's worktree is read from, the word the driver writes into Pi's
 environment, the answers a word that is not one of this package's gets, and the Pi a person runs themselves, which registers
 nothing at all.

 Every folder here is a test's own under the harness's scratch root.

 @module
 */

import { readFile, } from 'node:fs/promises';
import { join, } from 'node:path';
import type { ExtensionAPI, } from '@earendil-works/pi-coding-agent';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  armTheMainCheckoutGuard,
  guardArmedDescription,
  MAIN_CHECKOUT_GUARD_COMMAND,
  MAIN_CHECKOUT_GUARD_OFF,
  MAIN_CHECKOUT_GUARD_VARIABLE,
  type MainCheckoutGuard,
  mainCheckoutGuardArguments,
  mainCheckoutGuardOfARun,
  mainCheckoutGuardOfPiEnvironment,
  mainCheckoutGuardWordOf,
  MainCheckoutGuardUnreadableError,
  isRecord,
} from '../dist/final/node/index.mjs';
import { makeThrowawayRepository, } from './throwaway-repository-fixture.ts';

/**
 One Pi's extension API, recording what a guard arms it with.

 @returns the API, and what the extension registered through it

 @example
 ```ts
 const { pi, armed } = recordingExtensionApi();
 ```
 */
function recordingExtensionApi(): {
  readonly pi: ExtensionAPI;
  readonly handlers: readonly string[];
  readonly commands: readonly { readonly name: string; readonly description: string }[];
} {
  /**
   Every event a handler was registered for.
   */
  const handlers: string[] = [];
  /**
   Every command registered.
   */
  const commands: { readonly name: string; readonly description: string }[] = [];
  const pi = {
    on: function on(event: string,) {
      handlers.push(event,);
      return function off(): void {
        // Nothing to release: the test reads what was registered and nothing else.
      };
    },
    registerCommand: function registerCommand(
      name: string,
      options: { readonly description?: string },
    ) {
      commands.push({
        name,
        description: options.description ?? '',
      },);
    },
  } as unknown as ExtensionAPI;
  return {
    pi,
    handlers,
    commands,
  };
}

/**
 The error one call threw, or nothing at all when it threw none.

 @param work - the call to make

 @returns what it threw, or the sentinel that says it threw nothing
 */
function errorOf(work: () => void,): unknown {
  try {
    work();
  }
  catch (error) {
    return error;
  }
  return NO_ERROR;
}

/**
 The answer of a call that threw nothing, which no refusal may be.
 */
const NO_ERROR: unique symbol = Symbol('pi-shared-subagent-driver/no-error-thrown',);

/**
 Reads the word a driver would write for the guard of a run started in a repository's worktree.

 @param repository - the throwaway repository

 @returns the word, and the guard it was written from

 @example
 ```ts
 const { word, guard } = await wordOfTheRun({ repository });
 ```
 */
async function wordOfTheRun({ repository, }: {
  readonly repository: Awaited<ReturnType<typeof makeThrowawayRepository>>;
},): Promise<{ readonly word: string; readonly guard: MainCheckoutGuard }> {
  /**
   The guard, read from the worktree the run was started in.
   */
  const guard = await mainCheckoutGuardOfARun({ workingFolder: repository.worktree, },);
  return {
    word: mainCheckoutGuardWordOf({ guard, },),
    guard,
  };
}

await describe({
  name: '',
  children: [
    describe({
      name: mainCheckoutGuardOfARun.name,
      children: [
        it({
          name: 'arms with the repository, the worktree and the folder every worktree lies in',
          fn: async () => {
            await using repository = await makeThrowawayRepository();
            /**
             The guard a run started in the worktree is armed with.
             */
            const guard = await mainCheckoutGuardOfARun({ workingFolder: repository.worktree, },);

            expect(guard,).toStrictEqual({
              armed: true,
              mainCheckout: repository.repository,
              worktree: repository.worktree,
              worktreesFolder: repository.worktreesFolder,
            },);
          },
        }),
        it({
          name: 'arms with nothing when the run was started in a folder that is in no repository',
          fn: async () => {
            await using repository = await makeThrowawayRepository();
            /**
             The guard a run started outside every checkout is armed with.
             */
            const guard = await mainCheckoutGuardOfARun({ workingFolder: join(repository.home, 'nowhere',), },);

            expect(guard.armed,).toBe(false,);
            expect(guard.armed ? '' : guard.why,).toContain('no repository',);
          },
        }),
        it({
          name: 'arms with nothing when the run was started in the main checkout itself',
          fn: async () => {
            await using repository = await makeThrowawayRepository();
            /**
             The guard a run started in the main checkout is armed with.
             */
            const guard = await mainCheckoutGuardOfARun({ workingFolder: repository.repository, },);

            expect(guard.armed,).toBe(false,);
            expect(guard.armed ? '' : guard.why,).toContain(repository.repository,);
          },
        }),
      ],
    }),
    describe({
      name: mainCheckoutGuardWordOf.name,
      children: [
        it({
          name: 'writes the folders, and reads them back out of the environment',
          fn: async () => {
            await using repository = await makeThrowawayRepository();
            /**
             The word the driver writes, and the guard it was written from.
             */
            const { word, guard, } = await wordOfTheRun({ repository, },);

            expect(
              mainCheckoutGuardOfPiEnvironment({ environment: { [MAIN_CHECKOUT_GUARD_VARIABLE]: word, }, },),
            ).toStrictEqual(guard,);
          },
        }),
        it({
          name: 'writes the word off for a run with no worktree, which reads back as a guard of nothing',
          fn: async () => {
            await using repository = await makeThrowawayRepository();
            /**
             The guard a run started in the main checkout is armed with.
             */
            const guard = await mainCheckoutGuardOfARun({ workingFolder: repository.repository, },);

            expect(mainCheckoutGuardWordOf({ guard, },),).toBe(MAIN_CHECKOUT_GUARD_OFF,);
            expect(
              mainCheckoutGuardOfPiEnvironment({
                environment: { [MAIN_CHECKOUT_GUARD_VARIABLE]: MAIN_CHECKOUT_GUARD_OFF, },
              },).armed,
            ).toBe(false,);
          },
        }),
      ],
    }),
    describe({
      name: mainCheckoutGuardOfPiEnvironment.name,
      children: [
        it({
          name: 'refuses to arm when the driver wrote nothing',
          fn: async () => {
            /**
             What the guard answered a Pi whose environment holds no word.
             */
            const caught = errorOf(function armWithNoWord() {
              mainCheckoutGuardOfPiEnvironment({ environment: {}, },);
            },);

            expect(caught,).toBeInstanceOf(MainCheckoutGuardUnreadableError,);
            expect(isRecord(caught,) ? caught.message : '',).toContain(MAIN_CHECKOUT_GUARD_VARIABLE,);
          },
        }),
        ...[
          { word: 'not json at all', why: 'words that are no JSON' },
          { word: '{}', why: 'a JSON object holding no folder' },
          {
            word: JSON.stringify({ mainCheckout: '/repo', },),
            why: 'a JSON object naming one folder of the three',
          },
          {
            word: JSON.stringify({
              mainCheckout: '/repo',
              worktree: '',
              worktreesFolder: '/repo/.claude/worktrees',
            },),
            why: 'a JSON object naming an empty folder',
          },
        ].map(function mapsOneWord(one,) {
          return it({
            name: `refuses to arm with ${one.why}`,
            fn: async () => {
              /**
               What the guard answered the word.
               */
              const caught = errorOf(function armWithTheWord() {
                mainCheckoutGuardOfPiEnvironment({
                  environment: { [MAIN_CHECKOUT_GUARD_VARIABLE]: one.word, },
                },);
              },);

              expect(caught,).toBeInstanceOf(MainCheckoutGuardUnreadableError,);
            },
          },);
        },),
      ],
    }),
    describe({
      name: armTheMainCheckoutGuard.name,
      children: [
        it({
          name: 'registers the hook and the command naming the folders this run may not change',
          fn: async () => {
            await using repository = await makeThrowawayRepository();
            /**
             The word and the guard a run started in the worktree is given.
             */
            const { word, guard, } = await wordOfTheRun({ repository, },);
            /**
             The Pi this guard is armed in.
             */
            const armed = recordingExtensionApi();

            armTheMainCheckoutGuard({
              pi: armed.pi,
              guard: mainCheckoutGuardOfPiEnvironment({ environment: { [MAIN_CHECKOUT_GUARD_VARIABLE]: word, }, },),
              home: repository.home,
            },);

            expect(armed.handlers,).toStrictEqual(['tool_call',],);
            expect(armed.commands,).toStrictEqual([{
              name: MAIN_CHECKOUT_GUARD_COMMAND,
              description: guardArmedDescription({ guard, },),
            },],);
          },
        }),
        it({
          name: 'registers nothing at all in a Pi with no worktree to guard',
          fn: async () => {
            await using repository = await makeThrowawayRepository();
            /**
             The Pi a person's own session is, which this guard is armed in with nothing to protect.
             */
            const armed = recordingExtensionApi();

            armTheMainCheckoutGuard({
              pi: armed.pi,
              guard: await mainCheckoutGuardOfARun({ workingFolder: repository.repository, },),
              home: repository.home,
            },);

            expect(armed.handlers,).toStrictEqual([],);
            expect(armed.commands,).toStrictEqual([],);
          },
        }),
      ],
    }),
    describe({
      name: mainCheckoutGuardArguments.name,
      children: [
        it({
          name: 'names the extension entry beside the driver entry, which Pi loads with --extension',
          fn: async () => {
            /**
             The words the driver adds to Pi's command line.
             */
            const words = await mainCheckoutGuardArguments({ folder: join(import.meta.dirname,), },);

            expect(words[0],).toBe('--extension',);
            expect(words[1],).toContain('main-checkout-guard-extension',);
            // The entry is a file that exists: the built one, or the source one when this package runs from a checkout.
            expect((await readFile(words[1] ?? '', 'utf8',)).length,).toBeGreaterThan(0,);
          },
        }),
      ],
    }),
  ],
},);
