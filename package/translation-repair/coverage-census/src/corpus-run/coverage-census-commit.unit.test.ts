/**
 Tests what the coverage census asks git of the tree it reads (ledger T8),
 each against a throwaway repository with no global or system git
 configuration: the commit the package's files match, and which files of the
 work tree have changed since an earlier census's commit, named as the census
 names sources. Names are cat-themed invention.

 @module
 */

import {
  mkdir,
  writeFile,
} from 'node:fs/promises';
import { devNull, } from 'node:os';
import { join, } from 'node:path';

import { resolveRealGit, } from '@monochromatic-dev/git-executable-core/ts';
import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { spawnKeyless, } from '@monochromatic-dev/translation-repair-adjudication/ts/child-environment.test-fixture.ts';
import {
  packageCommit,
  sourcesEditedSince,
} from '../../dist/final/node/index.mjs';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

/**
 Environment that keeps the operator's git configuration out of a throwaway
 repository.
 */
const HERMETIC = {
  GIT_CONFIG_GLOBAL: devNull,
  GIT_CONFIG_SYSTEM: devNull,
};

/**
 Start of each case's throwaway repository name, so a leftover shows this
 file made it; each case binds its own `scratchDir` with `await using`.
 */
const REPOSITORY_PREFIX = 'translation-repair-census-commit-test-';

/**
 Runs git in a throwaway repository.

 @param directory - repository directory

 @param args - git arguments after `-C <directory>`

 @returns What git printed
 */
async function gitIn({
  directory,
  args,
}: {
  readonly directory: string;
  readonly args: readonly string[];
},) {
  return spawnKeyless({
    file: await resolveRealGit(),
    args: [
      '-C',
      directory,
      ...args,
    ],
    extra: HERMETIC,
  },);
}

/**
 Writes a file under a directory, making its parents.

 @param directory - directory it goes under

 @param path - its path under that directory

 @param text - its contents
 */
async function writeUnder({
  directory,
  path,
  text,
}: {
  readonly directory: string;
  readonly path: string;
  readonly text: string;
},) {
  /**
   Where the file goes.
   */
  const target = join(
    directory,
    path,
  );
  await mkdir(
    join(
      target,
      '..',
    ),
    { recursive: true, },
  );
  await writeFile(
    target,
    text,
  );
}

/**
 Stages everything and commits it.

 @param directory - repository directory

 @param message - commit message
 */
async function commitAll({
  directory,
  message,
}: {
  readonly directory: string;
  readonly message: string;
},) {
  await gitIn({
    directory,
    args: [
      'add',
      '--all',
    ],
  },);
  await gitIn({
    directory,
    args: [
      '-c',
      'user.name=cat',
      '-c',
      'user.email=cat@example.org',
      'commit',
      '--message',
      message,
      '--no-gpg-sign',
    ],
  },);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: packageCommit.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES THE COMMIT BY NINE CHARACTERS, clean until a file under the package changes',
          fn: async () => {
            await using directory = await scratchDir({ prefix: REPOSITORY_PREFIX, },);
            await gitIn({
              directory: directory.path,
              args: ['init',],
            },);
            await writeUnder({
              directory: directory.path,
              path: 'nap.txt',
              text: 'nap\n',
            },);
            await commitAll({
              directory: directory.path,
              message: 'nap',
            },);
            const { stdout: full, } = await gitIn({
              directory: directory.path,
              args: [
                'rev-parse',
                'HEAD',
              ],
            },);
            const committed = await packageCommit({ packageDirectory: directory.path, },);
            expect(committed.head,).toBe(full.slice(
              0,
              9,
            ),);
            expect(committed.clean,).toBe(true,);
            await writeUnder({
              directory: directory.path,
              path: 'nap.txt',
              text: 'purr\n',
            },);
            expect((await packageCommit({ packageDirectory: directory.path, },)).clean,).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: sourcesEditedSince.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name:
            'NAMES THE SOURCES THAT DIFFER FROM A COMMIT, relative to the package as a census names them: one edited '
              + 'and not committed, one added and committed since, and not one left as it was',
          fn: async () => {
            await using repository = await scratchDir({ prefix: REPOSITORY_PREFIX, },);
            await gitIn({
              directory: repository.path,
              args: ['init',],
            },);
            /**
             Package directory, a subdirectory of the repository as in this one.
             */
            const packageDirectory = join(
              repository.path,
              'package',
              'cat',
            );
            await writeUnder({
              directory: packageDirectory,
              path: 'src/nap.ts',
              text: 'export const nap = 1;\n',
            },);
            await writeUnder({
              directory: packageDirectory,
              path: 'src/purr.ts',
              text: 'export const purr = 1;\n',
            },);
            await commitAll({
              directory: repository.path,
              message: 'nap and purr',
            },);
            /**
             Commit an earlier census would record.
             */
            const { head, } = await packageCommit({ packageDirectory, },);
            await writeUnder({
              directory: packageDirectory,
              path: 'src/knead.ts',
              text: 'export const knead = 1;\n',
            },);
            await commitAll({
              directory: repository.path,
              message: 'knead',
            },);
            await writeUnder({
              directory: packageDirectory,
              path: 'src/nap.ts',
              text: '// one more line renumbers the rest of the file\nexport const nap = 1;\n',
            },);
            expect([...await sourcesEditedSince({
              packageDirectory,
              head,
            },),].toSorted(),).toEqual(['src/knead.ts', 'src/nap.ts',],);
          },
        },),
        it({
          name:
            'NAMES EVERY CHANGED FILE IN THE WORK TREE, relative to the package, since a census reads the sources of '
              + 'other packages the suite loads too: one asked about only inside the package left their stretches matched '
              + 'by line across an edit (ledger B62). Names git would quote, a space, a double quote, a newline and Han, '
              + 'come back as the census names them',
          fn: async () => {
            await using repository = await scratchDir({ prefix: REPOSITORY_PREFIX, },);
            await gitIn({
              directory: repository.path,
              args: ['init',],
            },);
            /**
             Package directory, a subdirectory of the repository.
             */
            const packageDirectory = join(
              repository.path,
              'package',
              'cat',
            );
            await writeUnder({
              directory: packageDirectory,
              path: 'src/nap.ts',
              text: 'export const nap = 1;\n',
            },);
            await writeUnder({
              directory: packageDirectory,
              path: 'doc/whiskers.md',
              text: '# Whiskers\n',
            },);
            await writeUnder({
              directory: repository.path,
              path: 'README.md',
              text: '# Litter\n',
            },);
            await writeUnder({
              directory: repository.path,
              path: 'package/whisker/src/index.ts',
              text: 'export const whisker = 1;\n',
            },);
            /**
             Sources whose names git quotes in its default output.
             */
            const unusual = ['src/猫 "nap".ts', 'src/nap\nkitten.ts',];
            await Promise.all(unusual.map(async function writeFirst(path,) {
              await writeUnder({
                directory: packageDirectory,
                path,
                text: 'export const nap = 1;\n',
              },);
            },),);
            await commitAll({
              directory: repository.path,
              message: 'nap, whiskers, litter and whisker',
            },);
            /**
             Commit an earlier census would record.
             */
            const { head, } = await packageCommit({ packageDirectory, },);
            await writeUnder({
              directory: packageDirectory,
              path: 'doc/whiskers.md',
              text: '# Whiskers, groomed\n',
            },);
            await writeUnder({
              directory: repository.path,
              path: 'README.md',
              text: '# Litter, fresh\n',
            },);
            await writeUnder({
              directory: repository.path,
              path: 'package/whisker/src/index.ts',
              text: '// one more line renumbers the rest of the file\nexport const whisker = 1;\n',
            },);
            await Promise.all(unusual.map(async function rewrite(path,) {
              await writeUnder({
                directory: packageDirectory,
                path,
                text: 'export const nap = 2;\n',
              },);
            },),);
            expect([...await sourcesEditedSince({
              packageDirectory,
              head,
            },),].toSorted(),).toEqual([
              '../../README.md',
              '../whisker/src/index.ts',
              'doc/whiskers.md',
              ...unusual,
            ].toSorted(),);
          },
        },),
      ],
    },),
  ],
},);
