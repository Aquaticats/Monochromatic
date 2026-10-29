/**
 Tests the slice-cache account behind the pre-launch cache version check
 (ledger M57, M81): which runs directories it reads, which files it counts as
 records, the count it reports as the control, and the newest record, in
 throwaway trees of cat-themed entries.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import fsPromises, {
  chmod,
  mkdir,
  stat,
  symlink,
  utimes,
  writeFile,
} from 'node:fs/promises';
import { syncBuiltinESMExports, } from 'node:module';
import { join, } from 'node:path';

import {
  runsDirsIn,
  runsDirsUnder,
  sliceCacheAccount,
} from '../../dist/final/node/index.mjs';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

//region Fixture tree

/**
 Writes one file and sets its modification time.

 @param path - file to write

 @param seconds - modification time, in seconds since the epoch

 @example
 ```ts
 await writeAt({ path, seconds: 2_000, },);
 ```
 */
async function writeAt(
  {
    path,
    seconds,
  }: {
    readonly path: string;
    readonly seconds: number;
  },
): Promise<void> {
  await writeFile(
    path,
    '{}',
  );
  await utimes(
    path,
    seconds,
    seconds,
  );
}

/**
 Builds runs directories under a parent: the default one, with two records, a
 namespace marker, a link and a directory named like a record; a hand-set one
 with one older record; one with no slice cache; and one not named for runs,
 holding the newest record of all.

 @param parent - directory to build under

 @example
 ```ts
 await buildRunsTree({ parent, },);
 ```
 */
async function buildRunsTree({ parent, }: { readonly parent: string; },): Promise<void> {
  /**
   The default runs directory's cache for one entry.
   */
  const tabby = join(
    parent,
    'translation-repair-runs',
    'slice-cache',
    'Tabby',
  );
  /**
   A hand-set runs directory's cache for another entry.
   */
  const mittens = join(
    parent,
    'translation-repair-runs-pass1',
    'slice-cache',
    'Mittens',
  );
  /**
   A directory not named for runs.
   */
  const ginger = join(
    parent,
    'catnip-runs',
    'slice-cache',
    'Ginger',
  );
  await mkdir(
    join(
      tabby,
      'nap.json',
    ),
    { recursive: true, },
  );
  await mkdir(
    mittens,
    { recursive: true, },
  );
  await mkdir(
    ginger,
    { recursive: true, },
  );
  await mkdir(
    join(
      parent,
      'translation-repair-runs-empty',
    ),
    { recursive: true, },
  );
  await writeAt({
    path: join(
      tabby,
      'translate.0-1-tabby.json',
    ),
    seconds: 1_000,
  },);
  await writeAt({
    path: join(
      tabby,
      '0-2-tabby.json',
    ),
    seconds: 2_000,
  },);
  await writeAt({
    path: join(
      tabby,
      'translate-generation.txt',
    ),
    seconds: 3_000,
  },);
  await symlink(
    join(
      tabby,
      '0-2-tabby.json',
    ),
    join(
      tabby,
      'link.json',
    ),
  );
  await writeAt({
    path: join(
      mittens,
      '0-0-mittens.json',
    ),
    seconds: 1_500,
  },);
  await writeAt({
    path: join(
      ginger,
      '0-0-ginger.json',
    ),
    seconds: 9_000,
  },);
}

//endregion Fixture tree

//region Cases

await describe({
  name: sliceCacheAccount.name,
  children: [
    it({
      name: 'READS THE RUNS DIRECTORIES NAMED FOR RUNS, the default and those carrying its name as a prefix, in '
        + 'code-point order, and none from an absent parent',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'whiskers-cache-account-', },);
        await buildRunsTree({ parent: scratch.path, },);
        expect(await runsDirsIn({ parent: scratch.path, },),).toEqual([
          join(
            scratch.path,
            'translation-repair-runs',
          ),
          join(
            scratch.path,
            'translation-repair-runs-empty',
          ),
          join(
            scratch.path,
            'translation-repair-runs-pass1',
          ),
        ],);
        expect(await runsDirsIn({
          parent: join(
            scratch.path,
            'no-such-parent',
          ),
        },),).toEqual([],);
      },
    },),

    it({
      name: 'COUNTS EVERY RECORD AND NAMES THE NEWEST, leaving out a namespace marker, a link, a directory named '
        + 'like a record and a runs directory with no slice cache',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'whiskers-cache-account-', },);
        await buildRunsTree({ parent: scratch.path, },);
        expect(await sliceCacheAccount({ runsDirs: await runsDirsIn({ parent: scratch.path, },), },),).toEqual({
          runsDirs: [
            join(
              scratch.path,
              'translation-repair-runs',
            ),
            join(
              scratch.path,
              'translation-repair-runs-empty',
            ),
            join(
              scratch.path,
              'translation-repair-runs-pass1',
            ),
          ],
          count: 3,
          newest: {
            kind: 'found',
            record: {
              path: join(
                scratch.path,
                'translation-repair-runs',
                'slice-cache',
                'Tabby',
                '0-2-tabby.json',
              ),
              modifiedMs: 2_000_000,
            },
          },
        },);
      },
    },),

    it({
      name: 'LEAVES OUT A RECORD A DISCARD REMOVED BETWEEN THE LISTING AND THE READ, counting and naming the rest',
      fn: async (ctx,) => {
        await using scratch = await scratchDir({ prefix: 'whiskers-cache-account-', },);
        /**
         The slice-cache directory of one entry.
         */
        const entryDir = join(
          scratch.path,
          'slice-cache',
          'Tabby',
        );
        await mkdir(
          entryDir,
          { recursive: true, },
        );
        /**
         The record the discard removes, and the one it leaves.
         */
        const [removed, kept,] = [
          join(
            entryDir,
            '0-1-tabby.json',
          ),
          join(
            entryDir,
            '1-1-tabby.json',
          ),
        ];
        await writeAt({
          path: removed,
          seconds: 1_000,
        },);
        await writeAt({
          path: kept,
          seconds: 2_000,
        },);
        /**
         The real `stat`, which every other path still reaches.
         */
        const realStat = stat;
        /**
         `stat` as it answers once the discard has run: no such file for the
         record it removed. A removal between a listing and a read cannot be
         timed from a case, so the case stands in for its outcome.
         */
        const afterDiscard = ctx.sinon.stub(
          fsPromises,
          'stat',
        ).callsFake(async function statAfterDiscard(path,) {
          if (path === removed) {
            throw Object.assign(
              new Error('no such file',),
              { code: 'ENOENT', },
            );
          }
          return await realStat(path,);
        },);
        syncBuiltinESMExports();
        // THE ESM BINDING OF `stat` FOLLOWS THE STUB ONLY WHILE IT IS SYNCED, so
        // putting it back is a step of its own that the case's end runs.
        using putBack = {
          [Symbol.dispose]: function restoreStat(): void {
            afterDiscard.restore();
            syncBuiltinESMExports();
          },
        };
        expect(await sliceCacheAccount({ runsDirs: [scratch.path,], },),).toEqual({
          runsDirs: [scratch.path,],
          count: 1,
          newest: {
            kind: 'found',
            record: {
              path: kept,
              modifiedMs: 2_000_000,
            },
          },
        },);
      },
    },),

    it({
      name: 'REPORTS NO NEWEST RECORD AND A ZERO COUNT where the runs directories hold no slice cache, the control '
        + 'a reader checks before trusting any newest record',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'whiskers-cache-account-', },);
        await buildRunsTree({ parent: scratch.path, },);
        expect(await sliceCacheAccount({
          runsDirs: [
            join(
              scratch.path,
              'translation-repair-runs-empty',
            ),
          ],
        },),).toEqual({
          runsDirs: [
            join(
              scratch.path,
              'translation-repair-runs-empty',
            ),
          ],
          count: 0,
          newest: { kind: 'none', },
        },);
      },
    },),

    it({
      name: 'BREAKS A TIE IN TIME BY PATH IN CODE-POINT ORDER, so one tree always names one record',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'whiskers-cache-account-', },);
        /**
         Two entries whose records were written in the same second.
         */
        const entries = [
          'Socks',
          'Biscuit',
        ];
        await Promise.all(entries.map(async function writeEntry(entry,): Promise<void> {
          /**
           The entry's cache directory.
           */
          const entryDir = join(
            scratch.path,
            'translation-repair-runs',
            'slice-cache',
            entry,
          );
          await mkdir(
            entryDir,
            { recursive: true, },
          );
          await writeAt({
            path: join(
              entryDir,
              '0-0-nap.json',
            ),
            seconds: 4_000,
          },);
        },),);
        /**
         Their account.
         */
        const account = await sliceCacheAccount({
          runsDirs: [
            join(
              scratch.path,
              'translation-repair-runs',
            ),
          ],
        },);
        expect(account.newest,).toEqual({
          kind: 'found',
          record: {
            path: join(
              scratch.path,
              'translation-repair-runs',
              'slice-cache',
              'Biscuit',
              '0-0-nap.json',
            ),
            modifiedMs: 4_000_000,
          },
        },);
      },
    },),

    it({
      name: 'FINDS RUNS DIRECTORIES UNDER A SEARCHED TREE, four levels down at most, stopping at each one found '
        + 'and never entering a hidden directory or a dependency tree',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'whiskers-cache-account-', },);
        /**
         Slice caches by where they sit under the searched root: one and two
         levels down, at the depth bound and past it, inside a runs directory
         already found, under a hidden directory and under a dependency tree.
         */
        const caches = [
          ['purr-run',],
          [
            'nap-room',
            'runs',
          ],
          [
            'edge',
            'one',
            'two',
            'run',
          ],
          [
            'deep',
            'one',
            'two',
            'three',
            'run',
          ],
          [
            'purr-run',
            'nested',
          ],
          [
            '.copy-payload',
            'run',
          ],
          [
            'node_modules',
            'run',
          ],
        ];
        await Promise.all(caches.map(async function makeCache(levels,): Promise<void> {
          await mkdir(
            join(
              scratch.path,
              ...levels,
              'slice-cache',
            ),
            { recursive: true, },
          );
        },),);
        expect(await runsDirsUnder({ root: scratch.path, },),).toEqual({
          found: [
            join(
              scratch.path,
              'edge',
              'one',
              'two',
              'run',
            ),
            join(
              scratch.path,
              'nap-room',
              'runs',
            ),
            join(
              scratch.path,
              'purr-run',
            ),
          ],
          unlisted: [],
        },);
      },
    },),

    it({
      name: 'REPORTS A DIRECTORY IT CANNOT LIST, with the reason, and keeps searching beside it; an absent root is '
        + 'reported the same way',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'whiskers-cache-account-', },);
        /**
         A directory the search cannot list, holding a runs directory.
         */
        const locked = join(
          scratch.path,
          'locked-basket',
        );
        await mkdir(
          join(
            locked,
            'run',
            'slice-cache',
          ),
          { recursive: true, },
        );
        await mkdir(
          join(
            scratch.path,
            'open-basket',
            'slice-cache',
          ),
          { recursive: true, },
        );
        // Permissions do not constrain a superuser, so run as root the locked
        // directory is listed and the case fails rather than passing quietly.
        await chmod(
          locked,
          0o000,
        );
        /**
         The search, read before the directory is put back.
         */
        const search = await runsDirsUnder({ root: scratch.path, },);
        // Put the directory back before asserting, so a failing assertion still
        // leaves a removable tree behind for the disposal.
        await chmod(
          locked,
          0o700,
        );
        expect(search,).toEqual({
          found: [
            join(
              scratch.path,
              'open-basket',
            ),
          ],
          unlisted: [{
            dir: locked,
            reason: 'EACCES',
          },],
        },);
        /**
         A root that does not exist.
         */
        const absent = join(
          scratch.path,
          'no-such-basket',
        );
        expect(await runsDirsUnder({ root: absent, },),).toEqual({
          found: [],
          unlisted: [{
            dir: absent,
            reason: 'ENOENT',
          },],
        },);
      },
    },),

    it({
      name: 'NAMES THE DIRECTORIES IT CANNOT LIST IN CODE-POINT ORDER, whatever order the filesystem listed them in',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'whiskers-cache-account-', },);
        /**
         Directories the search cannot list, created in ascending order.
         */
        const baskets = [
          'basket-1',
          'basket-2',
          'basket-3',
          'basket-4',
          'basket-5',
        ].map(function pathOf(name,): string {
          return join(
            scratch.path,
            name,
          );
        },);
        await Promise.all(baskets.map(async function lock(basket,): Promise<void> {
          await mkdir(basket,);
        },),);
        // Permissions do not constrain a superuser, so run as root the baskets
        // are listed and the case fails rather than passing quietly.
        await Promise.all(baskets.map(async function lock(basket,): Promise<void> {
          await chmod(
            basket,
            0o000,
          );
        },),);
        /**
         The search, read before the directories are put back.
         */
        const search = await runsDirsUnder({ root: scratch.path, },);
        // Put the directories back before asserting, so a failing assertion
        // still leaves a removable tree behind for the disposal.
        await Promise.all(baskets.map(async function unlock(basket,): Promise<void> {
          await chmod(
            basket,
            0o700,
          );
        },),);
        expect(search,).toEqual({
          found: [],
          unlisted: baskets.map(function unlisted(basket,): { readonly dir: string; readonly reason: string; } {
            return {
              dir: basket,
              reason: 'EACCES',
            };
          },),
        },);
      },
    },),
  ],
},);

//endregion Cases
