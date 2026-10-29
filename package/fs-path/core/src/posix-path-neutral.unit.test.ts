/**
 Tests for the platform-neutral artifact under Node.

 The neutral build inlines the pure-JS `#posix-path` backend and the
 origin-private-file-system `#root-filesystem` backend, so importing
 `dist/final/neutral/index.mjs` directly (bypassing the `node` export
 condition) exercises the code browsers run. The path operations are
 checked against `node:path/posix` as the oracle over a corpus of edge
 cases; root discovery must report that no filesystem exists rather
 than quietly reach `node:fs`.

 @module
 */

import { posix, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  dirname,
  findRoot,
  GIT_REPOSITORY,
  isAbsolute,
  join,
  MISE_MONOREPO,
  normalize,
  PNPM_WORKSPACE,
  resolve,
  RootNotFoundError,
  sep,
} from '../dist/final/neutral/index.mjs';

//region Corpus

/**
 Single paths covering roots, trailing slashes, doubled separators, dot
 segments, and the empty string.
 */
const SINGLE_PATHS: readonly string[] = [
  '',
  '/',
  '//',
  '.',
  '..',
  '/a',
  '/a/',
  'a',
  'a/',
  'a/b',
  'a/b/',
  'a//b',
  '/a/b/c.txt',
  '///a///b///',
  './a',
  '../a/../b',
  'a/../..',
  '/a/../..',
  'a/./b/./',
  '/a/b/../../../c',
];

/**
 Segment lists covering empty lists, empty segments, and absolute resets.
 */
const SEGMENT_LISTS: readonly (readonly string[])[] = [
  [],
  ['',],
  ['a',],
  ['a', 'b',],
  ['/a', 'b', 'c',],
  ['a', '/b', 'c',],
  ['/a', '', 'b',],
  ['/root', '../sibling',],
  ['/foo', 'bar', './baz',],
  ['/foo', '/bar',],
  ['..', 'a',],
  ['/', '..',],
  ['a/', '/b/', 'c/',],
];

//endregion Corpus

await describe({
  name: 'neutral artifact under Node',
  children: [
    it({
      name: 'sep is POSIX `/`',
      fn: async () => {
        expect(sep,).toBe('/',);
      },
    },),

    ...SINGLE_PATHS.map(function toDirnameCase(path,) {
      return it({
        name: `dirname(${JSON.stringify(path,)}) matches node:path/posix`,
        fn: async () => {
          expect(dirname(path,),).toBe(posix.dirname(path,),);
        },
      },);
    },),

    ...SINGLE_PATHS.map(function toNormalizeCase(path,) {
      return it({
        name: `normalize(${JSON.stringify(path,)}) matches node:path/posix`,
        fn: async () => {
          expect(normalize(path,),).toBe(posix.normalize(path,),);
        },
      },);
    },),

    ...SINGLE_PATHS.map(function toIsAbsoluteCase(path,) {
      return it({
        name: `isAbsolute(${JSON.stringify(path,)}) matches node:path/posix`,
        fn: async () => {
          expect(isAbsolute(path,),).toBe(posix.isAbsolute(path,),);
        },
      },);
    },),

    ...SEGMENT_LISTS.map(function toJoinCase(segments,) {
      return it({
        name: `join(${JSON.stringify(segments,)}) matches node:path/posix`,
        fn: async () => {
          expect(join(segments,),).toBe(posix.join(...segments,),);
        },
      },);
    },),

    ...SEGMENT_LISTS.map(function toResolveCase(segments,) {
      return it({
        name: `resolve(${JSON.stringify(segments,)}) matches node:path/posix`,
        fn: async () => {
          expect(resolve(segments,),).toBe(posix.resolve(...segments,),);
        },
      },);
    },),

    it({
      name: 'root discovery sees no filesystem and fails instead of reaching node:fs',
      fn: async () => {
        /**
         Rejections for the three preset markers, started from this checkout
         where every marker exists on the real filesystem.
         */
        const outcomes = await Promise.allSettled([
          findRoot({ cwd: import.meta.dirname, marker: MISE_MONOREPO, },),
          findRoot({ cwd: import.meta.dirname, marker: GIT_REPOSITORY, },),
          findRoot({ cwd: import.meta.dirname, marker: PNPM_WORKSPACE, },),
        ],);
        expect(outcomes.map(function toStatus(outcome,): string {
          return outcome.status;
        },),).toEqual([
          'rejected',
          'rejected',
          'rejected',
        ],);
        for (const outcome of outcomes)
          expect((outcome as PromiseRejectedResult).reason,).toBeInstanceOf(RootNotFoundError,);
      },
    },),
  ],
},);
