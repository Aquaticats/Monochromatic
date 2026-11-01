/**
 Tests for the two git questions the pool asks, on throwaway repositories.

 WHY THROWAWAY REPOSITORIES. `resolveCommit` and `tipContains` ask git, and
 the only suite that reached them before asked about this repository's own
 root and HEAD, which can never produce the two failures worth a test: a
 shallow history and a commit git does not know. Both are built here from
 nothing, in `scratchDir` directories, as three empty commits and a
 `--depth 2` clone of them. Nothing here reads the pinned corpus clone or this worktree,
 and the identity every commit is written under is passed per call, so the
 fixtures never read this machine's git configuration.

 THE SHALLOW GUARD IS THE ONE WORTH A TEST. `git merge-base --is-ancestor`
 exits 1 both for "not an ancestor" and for "history stops before the answer",
 so in a shallow clone a clean negative would quietly drop every entry produced
 before the cut while every rate above the pool looked ordinary. Measured
 before these were written, on exactly this fixture: in the depth-2 clone,
 asking whether the second commit contains the third exits 1 and
 `rev-parse --is-shallow-repository` prints true; in the full clone the same
 question exits 1 and the flag prints false; an invented id exits 128 in both.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { spawnKeyless, } from '@monochromatic-dev/translation-repair-adjudication/ts/child-environment.test-fixture.ts';
import {
  resolveCommit,
  tipContains,
} from '../../dist/final/node/index.mjs';
import {
  scratchDirPrepared,
  scratchDirWith,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

/**
 Real git binary, the one the module under test prefers as well.
 */
const GIT = '/usr/bin/git';

/**
 Identity and signing settings every throwaway commit is written under,
 passed on each call so nothing here reads or writes this machine's own git
 configuration.
 */
const PER_CALL_CONFIG = [
  '-c',
  'user.name=Cat',
  '-c',
  'user.email=cat@example.invalid',
  '-c',
  'commit.gpgsign=false',
];

/**
 Characters in the object ids the fixture repository writes.
 */
const OBJECT_ID_LENGTH = 40;

/**
 Characters of an id git still resolves as an abbreviation.
 */
const ABBREVIATION_LENGTH = 7;

/**
 Commits the fixture history holds, which the depth of the shallow clone is
 one short of.
 */
const HISTORY_LENGTH = 3;

/**
 A commit no repository built here has ever seen.
 */
const UNKNOWN_COMMIT = '0'.repeat(OBJECT_ID_LENGTH,);

/**
 Runs one git command against a throwaway repository and returns its stdout.

 @param repository - checkout to run in

 @param args - subcommand and its arguments

 @returns Trimmed stdout

 @example
 ```ts
 const head = await git({ repository, args: ['rev-parse', 'HEAD',], },);
 ```
 */
async function git(
  {
    repository,
    args,
  }: {
    readonly repository: string;
    readonly args: readonly string[];
  },
): Promise<string> {
  return (await spawnKeyless({
    file: GIT,
    args: [
      '-C',
      repository,
      ...PER_CALL_CONFIG,
      ...args,
    ],
  },)).stdout
    .trim();
}

/**
 Writes one empty commit, so the history has shape and no content.

 @param repository - checkout to commit in

 @param subject - commit message

 @example
 ```ts
 await commitEmpty({ repository, subject: 'first', },);
 ```
 */
async function commitEmpty(
  {
    repository,
    subject,
  }: {
    readonly repository: string;
    readonly subject: string;
  },
): Promise<void> {
  await git({
    repository,
    args: [
      'commit',
      '--quiet',
      '--allow-empty',
      '--message',
      subject,
    ],
  },);
}

/**
 Two throwaway checkouts of one three-commit history.
 */
type ThrowawayHistory = Readonly<{
  /**
   Complete clone, which can answer every ancestry question.
   */
  full: string;

  /**
   `--depth 2` clone, which knows the second and third commits and has the
   first grafted away.
   */
  shallow: string;

  /**
   The three commits, oldest first.
   */
  commits: readonly [
    string,
    string,
    string,
  ];
}>;

/**
 Builds a three-commit history and a depth-2 clone of it, in temporary
 directories.

 @returns Both checkouts and the commits they share, removed when its
 `await using` scope ends

 @throws When git lists other than three commits, which means the fixture
 itself is broken and no case in this file can mean anything

 @example
 ```ts
 await using history = await throwawayHistory();
 ```
 */
async function throwawayHistory(): Promise<ThrowawayHistory & AsyncDisposable> {
  /**
   Complete repository, written from nothing; `scratchDirWith` removes it if
   `git`, a commit or the commit-count check throws.
   */
  const fullScratch = await scratchDirWith({
    prefix: 'artifact-generation-full-',
    setup: async function committed({ path: full, },): Promise<{
      readonly commits: readonly [string, string, string];
    }> {
      await git({
        repository: full,
        args: [
          'init',
          '--quiet',
          '--initial-branch=main',
        ],
      },);
      await commitEmpty({
        repository: full,
        subject: 'first',
      },);
      await commitEmpty({
        repository: full,
        subject: 'second',
      },);
      await commitEmpty({
        repository: full,
        subject: 'third',
      },);

      /**
       The three commits, oldest first, as git lists them.
       */
      const listed = (await git({
        repository: full,
        args: [
          'rev-list',
          '--reverse',
          'HEAD',
        ],
      },))
        .split('\n',);
      const [
        first,
        second,
        third,
      ] = listed;
      if ((first === undefined) || (second === undefined) || (third === undefined)) {
        throw new Error(
          `expected ${String(HISTORY_LENGTH,)} commits, git listed ${String(listed.length,)}`,
        );
      }
      return { commits: [first, second, third,], };
    },
  },);

  // THE OUTER OWNS THE INNER FROM HERE: `fullScratch` already disposes
  // itself on a throw inside its own setup, but nothing disposes it once it
  // has returned until the caller's `await using` fires, so a throw from
  // the shallow clone's own setup must still remove it here.
  try {
    /**
     Shallow clone, which sees the last two commits only;
     `scratchDirPrepared` removes it if the clone itself throws.
     */
    const shallowScratch = await scratchDirPrepared({
      prefix: 'artifact-generation-shallow-',
      prepare: async function cloned({ path: shallow, },): Promise<void> {
        await spawnKeyless({
          file: GIT,
          args: [
            'clone',
            '--quiet',
            '--depth',
            String(HISTORY_LENGTH - 1,),
            `file://${fullScratch.path}`,
            shallow,
          ],
        },);
      },
    },);
    return {
      full: fullScratch.path,
      shallow: shallowScratch.path,
      commits: fullScratch.commits,
      [Symbol.asyncDispose]: async function removeHistory(): Promise<void> {
        await shallowScratch[Symbol.asyncDispose]();
        await fullScratch[Symbol.asyncDispose]();
      },
    };
  }
  catch (error) {
    await fullScratch[Symbol.asyncDispose]();
    throw error;
  }
}

/**
 Runs a call expected to refuse, returning what it said.

 @param act - call expected to reject

 @returns Refusal text, or an empty string where the call resolved

 @example
 ```ts
 const said = await refusalOf({ act: () => resolveCommit({ revision: 'nope', repository, },), },);
 ```
 */
async function refusalOf(
  { act, }: { readonly act: () => Promise<unknown>; },
): Promise<string> {
  try {
    await act();
    return '';
  }
  catch (error) {
    return String(error,);
  }
}

/**
 One history for every case, built once because each case reads it and none
 writes to it, removed when this module's top-level `await using` scope ends.
 */
await using history = await throwawayHistory();

/**
 The three commits, named for the cases.
 */
const [
  first,
  second,
  third,
] = history.commits;

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: resolveCommit.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'RESOLVES a symbolic name to the full object id of the commit it '
            + 'names, which is the control the refusal departs from',
          fn: async () => {
            expect(await resolveCommit({
              revision: 'HEAD',
              repository: history.full,
            },),)
              .toBe(third,);
          },
        },),

        it({
          name: 'RESOLVES an abbreviation to the full id, since a requirement '
            + 'spelled short would otherwise be compared against full ids and '
            + 'match nothing',
          fn: async () => {
            expect(await resolveCommit({
              revision: first.slice(
                0,
                ABBREVIATION_LENGTH,
              ),
              repository: history.full,
            },),)
              .toBe(first,);
          },
        },),

        it({
          name: 'REFUSES a name the repository does not know, naming the '
            + 'resolution that failed rather than quietly filtering nothing',
          fn: async () => {
            /**
             What the refusal says.
             */
            const said = await refusalOf({
              act: () =>
                resolveCommit({
                  revision: 'no-such-thing',
                  repository: history.full,
                },),
            },);

            expect(said.includes('Cannot resolve "no-such-thing" to a commit',),).toBe(true,);
          },
        },),
      ],
    },),

    describe({
      name: tipContains.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ANSWERS TRUE where the required commit is an ancestor of the tip',
          fn: async () => {
            expect(await tipContains({
              tip: third,
              commit: first,
              repository: history.full,
            },),)
              .toBe(true,);
          },
        },),

        it({
          name: 'ANSWERS TRUE where the two are one commit, since a tip contains '
            + 'itself',
          fn: async () => {
            expect(await tipContains({
              tip: second,
              commit: second,
              repository: history.full,
            },),)
              .toBe(true,);
          },
        },),

        it({
          name: 'ANSWERS FALSE on a clean negative in a complete history, which is '
            + 'the one exit git defines for not an ancestor',
          fn: async () => {
            expect(await tipContains({
              tip: first,
              commit: third,
              repository: history.full,
            },),)
              .toBe(false,);
          },
        },),

        it({
          name: 'REFUSES TO ANSWER THE SAME NEGATIVE IN A SHALLOW CLONE, naming '
            + 'the clone, since git reports a cut history with the exit it uses '
            + 'for a real negative and the pool would otherwise lose every entry '
            + 'produced before the cut',
          fn: async () => {
            /**
             What the refusal says.
             */
            const said = await refusalOf({
              act: () =>
                tipContains({
                  tip: second,
                  commit: third,
                  repository: history.shallow,
                },),
            },);

            expect(said.includes('SHALLOW',),).toBe(true,);
            expect(said.includes('Unshallow the repository',),).toBe(true,);
          },
        },),

        it({
          name: 'REFUSES A COMMIT THE REPOSITORY DOES NOT KNOW rather than reading '
            + 'it as not eligible, since a pool that cannot be partitioned must '
            + 'not be silently narrowed',
          fn: async () => {
            /**
             What the refusal says.
             */
            const said = await refusalOf({
              act: () =>
                tipContains({
                  tip: third,
                  commit: UNKNOWN_COMMIT,
                  repository: history.full,
                },),
            },);

            expect(said.includes('Cannot place pipeline commit',),).toBe(true,);
            expect(said.includes('NOT treated as "not eligible"',),).toBe(true,);
          },
        },),
      ],
    },),
  ],
},);
