/**
 Tests for the POSIX path operations through the built root entry.

 Under Node the `node` export condition serves `dist/final/node/index.mjs`,
 whose `#posix-path` backend delegates to `node:path/posix`; every case
 here therefore pins the delegating build. The pure-JS backend that the
 neutral artifact inlines is covered by `posix-path-neutral.unit.test.ts`.

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
  isAbsolute,
  join,
  normalize,
  resolve,
  sep,
  trimLeadingSlash,
  trimTrailingSlash,
} from '@monochromatic-dev/fs-path-core';

await describe({
  name: 'path-ops',
  children: [
    it({
      name: 'sep is POSIX `/`',
      fn: async () => {
        expect(sep,).toBe('/',);
      },
    },),
    it({
      name: 'dirname returns the parent directory',
      fn: async () => {
        expect(dirname('/foo/bar/baz.css',),).toBe('/foo/bar',);
        expect(dirname('/foo',),).toBe('/',);
        expect(dirname('foo',),).toBe('.',);
      },
    },),
    it({
      name: 'isAbsolute reflects leading slash',
      fn: async () => {
        expect(isAbsolute('/foo/bar',),).toBe(true,);
        expect(isAbsolute('foo/bar',),).toBe(false,);
        expect(isAbsolute('',),).toBe(false,);
      },
    },),
    it({
      name: 'join concatenates and normalizes segments',
      fn: async () => {
        expect(join(['/foo', 'bar', 'baz',],),).toBe('/foo/bar/baz',);
        expect(join(['foo', '../bar',],),).toBe('bar',);
        expect(join([],),).toBe('.',);
      },
    },),
    it({
      name: 'resolve produces an absolute path',
      fn: async () => {
        expect(resolve(['/foo', 'bar', 'baz',],),).toBe('/foo/bar/baz',);
        expect(resolve(['foo', '/bar', 'baz',],),).toBe('/bar/baz',);
        expect(resolve(['foo', 'bar',],),).toBe(posix.resolve(
          'foo',
          'bar',
        ),);
      },
    },),
    it({
      name: 'normalize collapses redundant segments and delegates to node:path/posix',
      fn: async () => {
        expect(normalize('/foo/bar//baz/./qux/../quux',),).toBe('/foo/bar/baz/quux',);
        expect(normalize('',),).toBe('.',);
        expect(normalize('a/../../b/',),).toBe(posix.normalize('a/../../b/',),);
      },
    },),
    it({
      name: 'trimTrailingSlash strips trailing `/` except on root',
      fn: async () => {
        expect(trimTrailingSlash('/foo/bar/',),).toBe('/foo/bar',);
        expect(trimTrailingSlash('/',),).toBe('/',);
        expect(trimTrailingSlash('foo',),).toBe('foo',);
      },
    },),
    it({
      name: 'trimLeadingSlash strips leading `/` except on root',
      fn: async () => {
        expect(trimLeadingSlash('/foo/bar',),).toBe('foo/bar',);
        expect(trimLeadingSlash('/',),).toBe('/',);
        expect(trimLeadingSlash('foo',),).toBe('foo',);
      },
    },),
  ],
},);
