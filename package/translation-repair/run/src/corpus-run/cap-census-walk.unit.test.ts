/**
 Tests for where the cap census looks for logs: which files it takes, which
 directories it enters, how deep it goes and what it says of a path it cannot
 read.

 Every tree is a scratch directory of invented, cat-themed names.

 @module
 */

import {
  chmod,
  mkdir,
  symlink,
  writeFile,
} from 'node:fs/promises';
import { dirname, join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { textsInCodePointOrder, } from '@monochromatic-dev/translation-repair-markup';
import { ARTIFACTS_DIR, } from '@monochromatic-dev/translation-repair-artifact-pool';
import {
  capCensusLogsUnder,
  PROMPT_PAYLOADS_DIR,
  SLICE_CACHE_DIR,
} from '../../dist/final/node/index.mjs';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

/**
 Writes an empty file, with the directories above it.

 @param path - file to write

 @example
 ```ts
 await touch({ path: join(root, 'tabby', 'nap.log.jsonl',), },);
 ```
 */
async function touch({ path, }: { readonly path: string; },): Promise<void> {
  await mkdir(
    dirname(path,),
    { recursive: true, },
  );
  await writeFile(
    path,
    '',
    'utf8',
  );
}

/**
 Directories nobody may list, opened again when their scope ends so the
 scratch directory holding them can be removed.

 @example
 ```ts
 await using sealed = await sealDirectories({ parent, names: ['tabby',], },);
 ```
 */
type SealedDirectories = {
  /**
   Their paths.
   */
  readonly paths: readonly string[];

  /**
   Opens them again.
   */
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Makes directories under a parent and takes every permission from them.

 @param parent - directory to make them in

 @param names - their names

 @returns Their paths, and a disposer that opens them again

 @example
 ```ts
 await using sealed = await sealDirectories({ parent, names: ['tabby',], },);
 ```
 */
async function sealDirectories(
  {
    parent,
    names,
  }: {
    readonly parent: string;
    readonly names: readonly string[];
  },
): Promise<SealedDirectories> {
  /**
   Paths of the directories.
   */
  const paths = names.map(function pathOf(name,): string {
    return join(
      parent,
      name,
    );
  },);
  for (const path of paths) {
    /* oxlint-disable no-await-in-loop -- each directory is made, then sealed, before the next */
    await mkdir(path,);
    await chmod(
      path,
      0,
    );
    /* oxlint-enable no-await-in-loop */
  }
  return {
    paths,
    [Symbol.asyncDispose]: async function reopen(): Promise<void> {
      await Promise.all(paths.map(async function open(path,): Promise<void> {
        await chmod(
          path,
          0o700,
        );
      },),);
    },
  };
}

await describe({
  name: capCensusLogsUnder.name,
  concurrency: 1,
  children: [
    it({
      name: 'TAKES EVERY LOG UNDER A DIRECTORY, nested ones included, and leaves a file that is no log',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'cap-census-walk-', },);
        await touch({
          path: join(
            scratch.path,
            'nap.log.jsonl',
          ),
        },);
        await touch({
          path: join(
            scratch.path,
            'tabby',
            'purr.log.jsonl',
          ),
        },);
        await touch({
          path: join(
            scratch.path,
            'tabby',
            'notes.txt',
          ),
        },);

        /**
         What the walk found under the directory.
         */
        const found = await capCensusLogsUnder({ roots: [scratch.path,], },);

        expect({
          logs: textsInCodePointOrder({ texts: found.logs, },),
          unreadable: found.unreadable,
          danglingLinks: found.danglingLinks,
        },).toEqual({
          logs: [
            join(
              scratch.path,
              'nap.log.jsonl',
            ),
            join(
              scratch.path,
              'tabby',
              'purr.log.jsonl',
            ),
          ],
          unreadable: 0,
          danglingLinks: 0,
        },);
      },
    },),
    it({
      name: 'TAKES A LOG NAMED DIRECTLY, and a file named directly that is no log is read as nothing',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'cap-census-walk-', },);

        /**
         A log and a file that is no log.
         */
        const log = join(
          scratch.path,
          'nap.log.jsonl',
        );
        const notes = join(
          scratch.path,
          'notes.txt',
        );
        await touch({ path: log, },);
        await touch({ path: notes, },);

        expect(await capCensusLogsUnder({
          roots: [
            log,
            notes,
          ],
        },),).toEqual({
          logs: [log,],
          unreadable: 0,
          danglingLinks: 0,
        },);
      },
    },),
    it({
      name: 'ENTERS NO DIRECTORY THAT HOLDS CACHES, ARTIFACTS, PAYLOADS OR DEPENDENCIES',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'cap-census-walk-', },);

        /**
         The one log outside every skipped directory.
         */
        const kept = join(
          scratch.path,
          'nap.log.jsonl',
        );
        await touch({ path: kept, },);
        for (const skipped of [SLICE_CACHE_DIR, ARTIFACTS_DIR, 'node_modules', PROMPT_PAYLOADS_DIR,]) {
          /* oxlint-disable no-await-in-loop -- the four trees are written one after another, so a failure names its own */
          await touch({
            path: join(
              scratch.path,
              skipped,
              'hidden.log.jsonl',
            ),
          },);
          /* oxlint-enable no-await-in-loop */
        }

        expect(await capCensusLogsUnder({ roots: [scratch.path,], },),).toEqual({
          logs: [kept,],
          unreadable: 0,
          danglingLinks: 0,
        },);
      },
    },),
    it({
      name: 'GOES SIX LEVELS DOWN A NAMED DIRECTORY and no further: a log in a directory at the sixth level is '
        + 'found, one a level below that is not',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'cap-census-walk-', },);

        /**
         A log in the deepest directory the walk lists.
         */
        const reached = join(
          scratch.path,
          'a',
          'b',
          'c',
          'd',
          'e',
          'reached.log.jsonl',
        );
        await touch({ path: reached, },);
        await touch({
          path: join(
            scratch.path,
            'a',
            'b',
            'c',
            'd',
            'e',
            'f',
            'beyond.log.jsonl',
          ),
        },);

        expect(await capCensusLogsUnder({ roots: [scratch.path,], },),).toEqual({
          logs: [reached,],
          unreadable: 0,
          danglingLinks: 0,
        },);
      },
    },),
    it({
      name: 'TAKES A LOG ONCE where the paths named reach it twice: a directory and a log inside it, or one log '
        + 'spelled two ways',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'cap-census-walk-', },);

        /**
         One log, reached through its directory and by name.
         */
        const log = join(
          scratch.path,
          'nap.log.jsonl',
        );
        await touch({ path: log, },);

        expect(await capCensusLogsUnder({
          roots: [
            scratch.path,
            log,
            join(
              scratch.path,
              '.',
              'nap.log.jsonl',
            ),
          ],
        },),).toEqual({
          logs: [log,],
          unreadable: 0,
          danglingLinks: 0,
        },);
      },
    },),
    it({
      name: 'FINDS NOTHING for no named path',
      fn: async () => {
        expect(await capCensusLogsUnder({ roots: [], },),).toEqual({
          logs: [],
          unreadable: 0,
          danglingLinks: 0,
        },);
      },
    },),
    it({
      name: 'COUNTS A PATH THAT IS NOT THERE as unreadable and says so on stderr with the error it met, without '
        + 'stopping the other paths',
      fn: async (ctx) => {
        await using scratch = await scratchDir({ prefix: 'cap-census-walk-', },);
        const stderr = ctx.sinon.stub(
          console,
          'error',
        );

        /**
         A log beside a path that does not exist.
         */
        const log = join(
          scratch.path,
          'nap.log.jsonl',
        );
        await touch({ path: log, },);
        const missing = join(
          scratch.path,
          'no-such-nest',
        );

        expect(await capCensusLogsUnder({
          roots: [
            missing,
            log,
          ],
        },),).toEqual({
          logs: [log,],
          unreadable: 1,
          danglingLinks: 0,
        },);
        expect(stderr.args,).toEqual([[
          `cap-census: cannot read ${missing}: Error: ENOENT: no such file or directory, stat '${missing}'`,
        ],],);
      },
    },),
    it({
      name: 'COUNTS A DIRECTORY IT CANNOT LIST as unreadable, one for each, and still takes the logs beside it',
      fn: async (ctx) => {
        await using scratch = await scratchDir({ prefix: 'cap-census-walk-', },);
        const stderr = ctx.sinon.stub(
          console,
          'error',
        );

        /**
         A log, and two directories nobody may list.
         */
        const log = join(
          scratch.path,
          'nap.log.jsonl',
        );
        await touch({ path: log, },);
        await using sealed = await sealDirectories({
          parent: scratch.path,
          names: ['sealed-one', 'sealed-two',],
        },);

        /**
         What the walk found.
         */
        const found = await capCensusLogsUnder({ roots: [scratch.path,], },);

        expect(found,).toEqual({
          logs: [log,],
          unreadable: 2,
          danglingLinks: 0,
        },);
        expect(textsInCodePointOrder({
          texts: stderr.args.map(function lineOf([line,],): string {
            return String(line,);
          },),
        },),).toEqual(textsInCodePointOrder({
          texts: sealed.paths.map(function lineFor(dir,): string {
            return `cap-census: cannot read ${dir}: Error: EACCES: permission denied, scandir '${dir}'`;
          },),
        },),);
      },
    },),
    it({
      name: 'PASSES OVER A DANGLING LINK WHOSE NAME IS NO LOG\'S, one for each, counting it apart from the unreadable '
        + 'paths and saying nothing of it on stderr, since a link to nothing holds no log (ledger B991)',
      fn: async (ctx) => {
        await using scratch = await scratchDir({ prefix: 'cap-census-walk-', },);
        const stderr = ctx.sinon.stub(
          console,
          'error',
        );

        /**
         A log, and two links in a nested directory whose targets are not there.
         */
        const log = join(
          scratch.path,
          'nap.log.jsonl',
        );
        await touch({ path: log, },);
        await mkdir(join(
          scratch.path,
          'profile',
        ),);
        await symlink(
          join(
            scratch.path,
            'no-such-nest',
          ),
          join(
            scratch.path,
            'profile',
            'SingletonLock',
          ),
        );
        await symlink(
          join(
            scratch.path,
            'no-such-hearth',
          ),
          join(
            scratch.path,
            'profile',
            'SingletonCookie',
          ),
        );

        expect(await capCensusLogsUnder({ roots: [scratch.path,], },),).toEqual({
          logs: [log,],
          unreadable: 0,
          danglingLinks: 2,
        },);
        expect(stderr.args,).toEqual([],);
      },
    },),
    it({
      name: 'STILL REPORTS A DANGLING LINK NAMED LIKE A LOG, as unreadable with the error it met, since the log it '
        + 'stands for could not be read (ledger B991)',
      fn: async (ctx) => {
        await using scratch = await scratchDir({ prefix: 'cap-census-walk-', },);
        const stderr = ctx.sinon.stub(
          console,
          'error',
        );

        /**
         A link named like a log whose target is not there.
         */
        const broken = join(
          scratch.path,
          'broken.log.jsonl',
        );
        await symlink(
          join(
            scratch.path,
            'no-such-nest',
          ),
          broken,
        );

        expect(await capCensusLogsUnder({ roots: [scratch.path,], },),).toEqual({
          logs: [],
          unreadable: 1,
          danglingLinks: 0,
        },);
        expect(stderr.args,).toEqual([[
          `cap-census: cannot read ${broken}: Error: ENOENT: no such file or directory, stat '${broken}'`,
        ],],);
      },
    },),
    it({
      name: 'STILL REPORTS A DIRECTORY IT CANNOT LIST beside a dangling link, since the directory may hold a log '
        + 'and the link cannot (ledger B991)',
      fn: async (ctx) => {
        await using scratch = await scratchDir({ prefix: 'cap-census-walk-', },);
        const stderr = ctx.sinon.stub(
          console,
          'error',
        );
        await symlink(
          join(
            scratch.path,
            'no-such-nest',
          ),
          join(
            scratch.path,
            'SingletonLock',
          ),
        );
        await using sealed = await sealDirectories({
          parent: scratch.path,
          names: ['sealed-one',],
        },);

        expect(await capCensusLogsUnder({ roots: [scratch.path,], },),).toEqual({
          logs: [],
          unreadable: 1,
          danglingLinks: 1,
        },);
        expect(stderr.args,).toEqual([[
          `cap-census: cannot read ${sealed.paths[0]}: Error: EACCES: permission denied, scandir '${sealed.paths[0]}'`,
        ],],);
      },
    },),
    it({
      name: 'STILL ENTERS A LINK TO A DIRECTORY and takes the log inside it, so the pass over dangling links costs '
        + 'no log a link leads to (ledger B991)',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'cap-census-walk-', },);
        await using elsewhere = await scratchDir({ prefix: 'cap-census-walk-target-', },);
        await touch({
          path: join(
            elsewhere.path,
            'purr.log.jsonl',
          ),
        },);
        await symlink(
          elsewhere.path,
          join(
            scratch.path,
            'tabby',
          ),
        );

        expect(await capCensusLogsUnder({ roots: [scratch.path,], },),).toEqual({
          logs: [
            join(
              scratch.path,
              'tabby',
              'purr.log.jsonl',
            ),
          ],
          unreadable: 0,
          danglingLinks: 0,
        },);
      },
    },),
  ],
},);
