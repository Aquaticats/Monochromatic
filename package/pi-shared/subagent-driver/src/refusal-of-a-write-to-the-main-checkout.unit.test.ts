/**
 What an `edit` or `write` call is refused for, and every call it is not: the worktree's own copy, the run's scratch folder, a path
 outside the repository, a sibling folder whose name begins with the main checkout's, a read of the main checkout, and every call
 of a session that has no worktree.

 Every path here is a folder of the test's own under the harness's scratch root, so nothing reads or changes this repository.

 @module
 */

import {
  mkdir,
  symlink,
} from 'node:fs/promises';
import { join, } from 'node:path';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  type MainCheckoutGuard,
  mainCheckoutGuardOfARun,
  type MainCheckoutRefusal,
  NOT_REFUSED,
  refusalOfAnEditOrWrite,
  refusalWords,
} from '../dist/final/node/index.mjs';
import {
  A_FILE_NAME,
  makeThrowawayRepository,
  type ThrowawayRepository,
} from './throwaway-repository-fixture.ts';

/**
 Reads the guard a run started in this repository's worktree is armed with.

 @param repository - the throwaway repository

 @returns the folders the guard holds

 @example
 ```ts
 const guard = await guardOfTheRun({ repository });
 ```
 */
async function guardOfTheRun({ repository, }: {
  readonly repository: ThrowawayRepository;
},): Promise<MainCheckoutGuard> {
  return await mainCheckoutGuardOfARun({ workingFolder: repository.worktree, },);
}

/**
 Asks the guard about one tool call, the way Pi's own `tool_call` hook does.

 @param repository - the throwaway repository the call is made in

 @param toolName - the tool the model asked for

 @param path - the path the call names

 @param cwd - the folder the session works in, the worktree unless a test says otherwise

 @param guard - the guard to ask, which is the run's own unless a test says otherwise

 @returns the refusal, or {@link NOT_REFUSED}

 @example
 ```ts
 const refusal = await refusalOf({ repository, toolName: 'edit', path: '/repo/a.ts' });
 ```
 */
async function refusalOf({
  repository,
  toolName,
  path,
  cwd = repository.worktree,
  guard,
}: {
  readonly repository: ThrowawayRepository;
  readonly toolName: string;
  readonly path: string;
  readonly cwd?: string;
  readonly guard?: MainCheckoutGuard;
},): Promise<MainCheckoutRefusal | typeof NOT_REFUSED> {
  return await refusalOfAnEditOrWrite({
    toolName,
    input: { path, },
    cwd,
    guard: guard ?? await guardOfTheRun({ repository, },),
    home: repository.home,
  },);
}

/**
 Reads the words of a refusal this test expects.

 @param refusal - what the guard answered

 @returns the refusal's words

 @throws Error when the call was allowed, which is a test that expected the wrong answer
 */
function refusedWords(refusal: MainCheckoutRefusal | typeof NOT_REFUSED,): string {
  if (refusal === NOT_REFUSED) {
    throw new Error('the call was allowed, and this test needs it refused',);
  }
  return refusal.reason;
}

await describe({
  name: '',
  children: [
    describe({
      name: refusalOfAnEditOrWrite.name,
      children: [
        //region Refused

        it({
          name: 'refuses an edit of the main checkout and names the worktree copy of the same path',
          fn: async () => {
            await using repository = await makeThrowawayRepository();
            /**
             The main checkout's copy of the file.
             */
            const mainCopy = join(
              repository.repository,
              'package',
              A_FILE_NAME,
            );
            /**
             The worktree's copy of it.
             */
            const worktreeCopy = join(
              repository.worktree,
              'package',
              A_FILE_NAME,
            );
            /**
             What the guard answered.
             */
            const refusal = await refusalOf({
              repository,
              toolName: 'edit',
              path: mainCopy,
            },);

            expect(refusedWords(refusal,),).toContain(mainCopy,);
            expect(refusedWords(refusal,),).toContain('the main checkout',);
            expect(refusedWords(refusal,),).toContain(worktreeCopy,);
            expect(refusedWords(refusal,),).toContain(repository.worktree,);
          },
        }),
        it({
          name: 'refuses a write of a main-checkout path that is not there yet',
          fn: async () => {
            await using repository = await makeThrowawayRepository();
            /**
             A path under the main checkout that nothing holds, which a write would create.
             */
            const madeByTheWrite = join(
              repository.repository,
              'package',
              'a-new-file.ts',
            );
            /**
             What the guard answered.
             */
            const refusal = await refusalOf({
              repository,
              toolName: 'write',
              path: madeByTheWrite,
            },);

            expect(refusedWords(refusal,),).toContain(madeByTheWrite,);
            expect(refusedWords(refusal,),).toContain(
              join(
                repository.worktree,
                'package',
                'a-new-file.ts',
              ),
            );
          },
        }),
        it({
          name: 'refuses a relative path that climbs out of the worktree into the main checkout',
          fn: async () => {
            await using repository = await makeThrowawayRepository();
            /**
             The way out of the worktree: three folders up from `<repository>/.claude/worktrees/pi-a`.
             */
            const climbing = join(
              '..',
              '..',
              '..',
              'package',
              A_FILE_NAME,
            );
            /**
             What the guard answered.
             */
            const refusal = await refusalOf({
              repository,
              toolName: 'edit',
              path: climbing,
            },);

            expect(refusedWords(refusal,),).toContain('the main checkout',);
          },
        }),
        it({
          name: 'refuses a symbolic link in the worktree that points into the main checkout',
          fn: async () => {
            await using repository = await makeThrowawayRepository();
            /**
             The link inside the worktree, which names the main checkout's copy.
             */
            await symlink(
              join(
                repository.repository,
                'package',
                A_FILE_NAME,
              ),
              join(
                repository.worktree,
                'link.ts',
              ),
            );
            /**
             What the guard answered.
             */
            const refusal = await refusalOf({
              repository,
              toolName: 'edit',
              path: 'link.ts',
            },);

            expect(refusedWords(refusal,),).toContain('link.ts',);
            expect(refusedWords(refusal,),).toContain(
              join(
                repository.worktree,
                'package',
                A_FILE_NAME,
              ),
            );
          },
        }),
        ...[
          {
            spelling: 'a doubled slash',
            path: ({ repository, }: { readonly repository: ThrowawayRepository },) => `${repository.repository}//package/${
              A_FILE_NAME
            }`,
          },
          {
            spelling: 'a dot segment',
            path: ({ repository, }: { readonly repository: ThrowawayRepository },) => `${repository.repository}/package/./${
              A_FILE_NAME
            }`,
          },
          {
            spelling: 'a dot-dot segment that comes back to the same folder',
            path: ({ repository, }: { readonly repository: ThrowawayRepository },) => `${repository.repository}/package/../package/${
              A_FILE_NAME
            }`,
          },
          {
            spelling: 'a home-folder tilde',
            // The repository lies under the home folder this test names, so a tilde path reaches the same file.
            path: () => `~/Monochromatic/package/${A_FILE_NAME}`,
          },
          {
            spelling: 'a file-reference at sign',
            path: ({ repository, }: { readonly repository: ThrowawayRepository },) =>
              `@${repository.repository}/package/${A_FILE_NAME}`,
          },
          {
            spelling: 'a file URL',
            path: ({ repository, }: { readonly repository: ThrowawayRepository },) =>
              `file://${repository.repository}/package/${A_FILE_NAME}`,
          },
        ].map(function mapsOneSpelling(one,) {
          return it({
            name: `refuses the main checkout's copy spelled with ${one.spelling}`,
            fn: async () => {
              await using repository = await makeThrowawayRepository();
              /**
               What the guard answered.
               */
              const refusal = await refusalOf({
                repository,
                toolName: 'edit',
                path: one.path({ repository, },),
              },);

              expect(refusedWords(refusal,),).toContain('the main checkout',);
            },
          },);
        },),

        //endregion Refused

        //region Allowed

        it({
          name: 'allows the worktree copy of the same path',
          fn: async () => {
            await using repository = await makeThrowawayRepository();
            /**
             What the guard answered.
             */
            const refusal = await refusalOf({
              repository,
              toolName: 'edit',
              path: join(
                repository.worktree,
                'package',
                A_FILE_NAME,
              ),
            },);

            expect(refusal,).toBe(NOT_REFUSED,);
          },
        }),
        it({
          name: 'allows a relative path inside the worktree',
          fn: async () => {
            await using repository = await makeThrowawayRepository();
            /**
             What the guard answered.
             */
            const refusal = await refusalOf({
              repository,
              toolName: 'edit',
              path: `package/${A_FILE_NAME}`,
            },);

            expect(refusal,).toBe(NOT_REFUSED,);
          },
        }),
        it({
          name: 'allows a symbolic link in the worktree that points inside the worktree',
          fn: async () => {
            await using repository = await makeThrowawayRepository();
            /**
             The link inside the worktree, which names the worktree's own copy.
             */
            await symlink(
              join(
                repository.worktree,
                'package',
                A_FILE_NAME,
              ),
              join(
                repository.worktree,
                'own-link.ts',
              ),
            );
            /**
             What the guard answered.
             */
            const refusal = await refusalOf({
              repository,
              toolName: 'edit',
              path: 'own-link.ts',
            },);

            expect(refusal,).toBe(NOT_REFUSED,);
          },
        }),
        it({
          name: "allows a write to the run's scratch folder",
          fn: async () => {
            await using repository = await makeThrowawayRepository();
            /**
             What the guard answered.
             */
            const refusal = await refusalOf({
              repository,
              toolName: 'write',
              path: join(
                repository.scratchFolder,
                'notes.md',
              ),
            },);

            expect(refusal,).toBe(NOT_REFUSED,);
          },
        }),
        it({
          name: 'allows a path outside the repository',
          fn: async () => {
            await using repository = await makeThrowawayRepository();
            /**
             The folder outside the repository, which a test writes into.
             */
            const elsewhere = join(
              repository.home,
              'elsewhere',
            );
            await mkdir(
              elsewhere,
              { recursive: true, },
            );
            /**
             What the guard answered.
             */
            const refusal = await refusalOf({
              repository,
              toolName: 'write',
              path: join(
                elsewhere,
                'a-file.ts',
              ),
            },);

            expect(refusal,).toBe(NOT_REFUSED,);
          },
        }),
        it({
          name: "allows the sibling folder whose name begins with the main checkout's own",
          fn: async () => {
            await using repository = await makeThrowawayRepository();
            /**
             What the guard answered.
             */
            const refusal = await refusalOf({
              repository,
              toolName: 'edit',
              path: join(
                repository.sibling,
                'package',
                A_FILE_NAME,
              ),
            },);

            expect(refusal,).toBe(NOT_REFUSED,);
          },
        }),
        it({
          name: 'allows a read of the main checkout',
          fn: async () => {
            await using repository = await makeThrowawayRepository();
            /**
             What the guard answered.
             */
            const refusal = await refusalOf({
              repository,
              toolName: 'read',
              path: join(
                repository.repository,
                'package',
                A_FILE_NAME,
              ),
            },);

            expect(refusal,).toBe(NOT_REFUSED,);
          },
        }),
        it({
          name: 'allows an edit of the main checkout in a session with no worktree',
          fn: async () => {
            await using repository = await makeThrowawayRepository();
            /**
             What the guard answered, armed the way a run started in the checkout itself is.
             */
            const refusal = await refusalOf({
              repository,
              toolName: 'edit',
              path: join(
                repository.repository,
                'package',
                A_FILE_NAME,
              ),
              cwd: repository.repository,
              guard: await mainCheckoutGuardOfARun({ workingFolder: repository.repository, },),
            },);

            expect(refusal,).toBe(NOT_REFUSED,);
          },
        }),
        it({
          name: 'allows an edit call that names no path at all',
          fn: async () => {
            await using repository = await makeThrowawayRepository();
            /**
             What the guard answered a call whose arguments hold no path.
             */
            const refusal = await refusalOfAnEditOrWrite({
              toolName: 'edit',
              input: { edits: [], },
              cwd: repository.worktree,
              guard: await guardOfTheRun({ repository, },),
              home: repository.home,
            },);

            expect(refusal,).toBe(NOT_REFUSED,);
          },
        }),

        //endregion Allowed
      ],
    }),
    describe({
      name: refusalWords.name,
      children: [
        it({
          name: 'names the path as the call gave it, the main checkout, the worktree and the worktree copy',
          fn: async () => {
            /**
             The path as a call gave it, which is not the path it resolves to.
             */
            const given = '../Monochromatic/package/a-file.ts';
            /**
             The refusal's words.
             */
            const words = refusalWords({
              toolName: 'write',
              given,
              guard: {
                armed: true,
                mainCheckout: '/home/a/Monochromatic',
                worktree: '/home/a/Monochromatic/.claude/worktrees/pi-a',
                worktreesFolder: '/home/a/Monochromatic/.claude/worktrees',
              },
              copy: '/home/a/Monochromatic/.claude/worktrees/pi-a/package/a-file.ts',
            },);

            expect(words,).toContain(`"${given}"`,);
            expect(words,).toContain('the main checkout',);
            expect(words,).toContain('/home/a/Monochromatic/.claude/worktrees/pi-a/package/a-file.ts',);
            expect(words,).toContain('Reading the main checkout',);
            expect(words,).toContain('Nothing was written',);
          },
        }),
      ],
    }),
  ],
},);
