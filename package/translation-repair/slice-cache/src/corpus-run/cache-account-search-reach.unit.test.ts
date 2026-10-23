/**
 Tests for whether a search for runs directories from a root would find the runs of another directory: the same
 directory under two spellings, the depth the walk stops at and the names it never enters.

 Every directory is a scratch directory or a path that does not exist; no record is read.

 Fixtures are cat-themed invention. No corpus content appears here.

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
} from '@monochromatic-dev/module-test/ts';

import { searchReaches, } from '../../dist/final/node/index.mjs';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

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
  ],
},);
