/**
 Tests for whether a search for runs directories from a root would find the runs of another directory: the same
 directory under two spellings, the depth the walk stops at and the names it never enters.

 Every directory is a scratch directory or a path that does not exist; no record is read.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  chmod,
  mkdir,
  symlink,
  writeFile,
} from 'node:fs/promises';
import {
  join,
  sep,
} from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { failureName, } from '@monochromatic-dev/translation-repair-markup/ts';
import { searchReaches, } from '../../dist/final/node/index.mjs';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

/**
 The code a failed attempt raised, or that it returned an answer.

 @param attempt - call expected to fail

 @returns Filesystem code of the failure, the failure's class name where it has no code, or `returned`

 @example
 ```ts
 const code = await codeOfFailure({ attempt: async () => await searchReaches({ root, directory, runsLevels: 0, },), },);
 ```
 */
async function codeOfFailure({ attempt, }: { readonly attempt: () => Promise<boolean>; },): Promise<string> {
  try {
    await attempt();
    return 'returned';
  }
  catch (error) {
    return failureName({ error, },);
  }
}

await describe({
  name: searchReaches.name,
  concurrency: 1,
  children: [
    it({
      name: 'REACHES THE DIRECTORY ITSELF AND THE ONES BELOW IT within the depth the walk stops at: runs two levels '
        + 'below a directory two levels under the root sit four levels down, the last the search reads',
      fn: async () => {
        await using root = await scratchDir({ prefix: 'cache-account-reach-root-', },);

        expect({
          itself: await searchReaches({ root: root.path, directory: root.path, runsLevels: 2, },),
          oneBelow: await searchReaches({ root: root.path, directory: join(root.path, 'cats',), runsLevels: 2, },),
          twoBelow: await searchReaches({
            root: root.path,
            directory: join(root.path, 'cats', 'temp',),
            runsLevels: 2,
          },),
        },).toEqual({
          itself: true,
          oneBelow: true,
          twoBelow: true,
        },);
      },
    },),
    it({
      name: 'DOES NOT REACH RUNS THE WALK STOPS BEFORE: a directory three levels below the root has runs up to five '
        + 'levels down, one more than the walk reads, while runs one level below it are within the walk',
      fn: async () => {
        await using root = await scratchDir({ prefix: 'cache-account-reach-root-', },);
        /**
         A directory three levels below the root.
         */
        const deep = join(
          root.path,
          'cats',
          'temp',
          'agent',
        );

        expect({
          deepRunsTwoLevels: await searchReaches({ root: root.path, directory: deep, runsLevels: 2, },),
          deepRunsOneLevel: await searchReaches({ root: root.path, directory: deep, runsLevels: 1, },),
        },).toEqual({
          deepRunsTwoLevels: false,
          deepRunsOneLevel: true,
        },);
      },
    },),
    it({
      name: 'DOES NOT REACH A DIRECTORY ABOVE THE ROOT, BESIDE IT, OR BEHIND A NAME THE SEARCH NEVER ENTERS: a root '
        + 'below the directory, a sibling, a hidden directory and a dependency tree each read as not reached',
      fn: async () => {
        await using root = await scratchDir({ prefix: 'cache-account-reach-root-', },);
        await using sibling = await scratchDir({ prefix: 'cache-account-reach-sibling-', },);
        const inRoot = join(
          root.path,
          'cats',
        );

        expect({
          above: await searchReaches({ root: inRoot, directory: root.path, runsLevels: 0, },),
          beside: await searchReaches({ root: root.path, directory: sibling.path, runsLevels: 0, },),
          hidden: await searchReaches({ root: root.path, directory: join(root.path, '.cats',), runsLevels: 0, },),
          dependencyTree: await searchReaches({
            root: root.path,
            directory: join(root.path, 'node_modules',),
            runsLevels: 0,
          },),
          visible: await searchReaches({ root: root.path, directory: join(root.path, 'cats',), runsLevels: 0, },),
        },).toEqual({
          above: false,
          beside: false,
          hidden: false,
          dependencyTree: false,
          visible: true,
        },);
      },
    },),
    it({
      name: 'COMPARES BY REAL PATH: a link to the root and the root are one directory whichever is given, and a '
        + 'directory that does not exist yet is placed by the nearest ancestor that does',
      fn: async () => {
        await using real = await scratchDir({ prefix: 'cache-account-reach-real-', },);
        await using links = await scratchDir({ prefix: 'cache-account-reach-links-', },);
        await mkdir(join(real.path, 'cats',),);
        /**
         A link to the real directory, in a directory of its own.
         */
        const link = join(
          links.path,
          'real-link',
        );
        await symlink(
          real.path,
          link,
          'dir',
        );
        /**
         A directory that exists nowhere, below the real one.
         */
        const absent = join(
          real.path,
          'cats',
          'temp',
          'agent',
        );

        expect({
          linkRootRealDirectory: await searchReaches({
            root: link,
            directory: join(real.path, 'cats',),
            runsLevels: 2,
          },),
          realRootLinkDirectory: await searchReaches({
            root: real.path,
            directory: join(link, 'cats',),
            runsLevels: 2,
          },),
          absentThroughLink: await searchReaches({
            root: link,
            directory: join(link, 'cats', 'temp', 'agent',),
            runsLevels: 0,
          },),
          absentThroughReal: await searchReaches({ root: link, directory: absent, runsLevels: 0, },),
          neitherExists: await searchReaches({
            root: '/cats/home',
            directory: '/cats/home/temp/agent',
            runsLevels: 2,
          },),
        },).toEqual({
          linkRootRealDirectory: true,
          realRootLinkDirectory: true,
          absentThroughLink: true,
          absentThroughReal: true,
          neitherExists: true,
        },);
      },
    },),
    it({
      name: 'STILL RAISES THE FILESYSTEM\'S REFUSAL FOR A PATH IT CANNOT RESOLVE: ENOTDIR for a path through a file '
        + 'and ELOOP for a loop of links, as root and as directory, so the caller decides what an unreadable root '
        + 'means: the report catches it per root',
      fn: async () => {
        await using base = await scratchDir({ prefix: 'cache-account-reach-refused-', },);
        /**
         A file, and a path that goes on below it.
         */
        const file = join(
          base.path,
          'file',
        );
        await writeFile(
          file,
          'meow',
          'utf8',
        );
        const belowFile = join(
          file,
          'below',
        );
        /**
         Two links that point at each other.
         */
        const loopA = join(
          base.path,
          'loop-a',
        );
        const loopB = join(
          base.path,
          'loop-b',
        );
        await symlink(
          loopB,
          loopA,
        );
        await symlink(
          loopA,
          loopB,
        );

        expect({
          fileRoot: await codeOfFailure({
            attempt: async () => await searchReaches({ root: belowFile, directory: base.path, runsLevels: 0, },),
          },),
          fileDirectory: await codeOfFailure({
            attempt: async () => await searchReaches({ root: base.path, directory: belowFile, runsLevels: 0, },),
          },),
          loopRoot: await codeOfFailure({
            attempt: async () => await searchReaches({ root: loopA, directory: base.path, runsLevels: 0, },),
          },),
          loopDirectory: await codeOfFailure({
            attempt: async () => await searchReaches({ root: base.path, directory: loopA, runsLevels: 0, },),
          },),
        },).toEqual({
          fileRoot: 'ENOTDIR',
          fileDirectory: 'ENOTDIR',
          loopRoot: 'ELOOP',
          loopDirectory: 'ELOOP',
        },);
      },
    },),
    it({
      name: 'STILL RAISES THE FILESYSTEM\'S REFUSAL FOR A PATH BEHIND A DIRECTORY NOBODY MAY ENTER (EACCES)',
      skip: (process.getuid?.() === 0)
        ? 'the process is root, and no mode refuses root a directory'
        : false,
      fn: async () => {
        await using base = await scratchDir({ prefix: 'cache-account-reach-sealed-', },);
        /**
         A directory with no permission bits, and a path below it.
         */
        const sealed = join(
          base.path,
          'sealed',
        );
        await mkdir(sealed,);
        await chmod(
          sealed,
          0,
        );
        await using _reopened = {
          [Symbol.asyncDispose]: async function reopen(): Promise<void> {
            await chmod(
              sealed,
              0o700,
            );
          },
        };

        expect(await codeOfFailure({
          attempt: async () =>
            await searchReaches({
              root: join(
                sealed,
                'below',
              ),
              directory: base.path,
              runsLevels: 0,
            },),
        },),).toEqual('EACCES',);
      },
    },),
    it({
      name: 'RESOLVES .. AFTER THE LINKS BEFORE IT, as the kernel does: base/lnk/../d is the d beside the link\'s '
        + 'target, not the d beside the link',
      fn: async () => {
        await using base = await scratchDir({ prefix: 'cache-account-reach-dotdot-', },);
        /**
         A directory the link will point into, two levels below the base.
         */
        const home = join(
          base.path,
          'h',
        );
        const inner = join(
          home,
          'sub',
        );
        await mkdir(
          inner,
          { recursive: true, },
        );
        await mkdir(join(
          base.path,
          'd',
        ),);
        await mkdir(join(
          home,
          'd',
        ),);
        await symlink(
          inner,
          join(
            base.path,
            'lnk',
          ),
          'dir',
        );
        // The string is built whole: join() would fold the dot-dot into the base before any link was read.
        const through = `${base.path}${sep}lnk${sep}..${sep}d`;

        expect({
          rootIsLinkTargetsParent: await searchReaches({ root: home, directory: through, runsLevels: 0, },),
          rootIsLinksParent: await searchReaches({
            root: join(
              base.path,
              'd',
            ),
            directory: through,
            runsLevels: 0,
          },),
        },).toEqual({
          rootIsLinkTargetsParent: true,
          rootIsLinksParent: false,
        },);
      },
    },),
    it({
      name: 'APPLIES THE DEPTH RULE WHERE THE ROOT IS THE DIRECTORY: runs four levels below it are the last the walk '
        + 'reads, runs five levels below are not',
      fn: async () => {
        await using root = await scratchDir({ prefix: 'cache-account-reach-equal-', },);

        expect({
          fourLevels: await searchReaches({ root: root.path, directory: root.path, runsLevels: 4, },),
          fiveLevels: await searchReaches({ root: root.path, directory: root.path, runsLevels: 5, },),
        },).toEqual({
          fourLevels: true,
          fiveLevels: false,
        },);
      },
    },),
  ],
},);
