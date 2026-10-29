/**
 Equivalence tests for `stripTrailingSlashes`.
 
 Capture the pre-refactor behavior of the trailing-slash trimmer so the
 single-slice rewrite stays behavior-identical: no trailing slash, one,
 many, all-slash collapse to empty, leading slashes preserved, only `/`
 (not `\`) stripped, and a long repeated run.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { stripTrailingSlashes, } from '../dist/final/node/launch.mjs';

/** Trailing-slash count for the long-run case; large enough to exercise the linear scan. */
const LONG_RUN = 100_000;

await describe({
  name: '',
  children: [
    describe({
      name: stripTrailingSlashes.name,
      children: [
        it({
          name: 'returns empty string for empty input',
          fn: async () => {
            expect(stripTrailingSlashes('',),).toBe('',);
          },
        },),

        it({
          name: 'leaves all-whitespace input unchanged',
          fn: async () => {
            expect(stripTrailingSlashes('   ',),).toBe('   ',);
          },
        },),

        it({
          name: 'leaves a path without a trailing slash unchanged',
          fn: async () => {
            expect(stripTrailingSlashes('/usr/share',),).toBe('/usr/share',);
          },
        },),

        it({
          name: 'strips a single trailing slash',
          fn: async () => {
            expect(stripTrailingSlashes('/usr/share/',),).toBe('/usr/share',);
          },
        },),

        it({
          name: 'strips multiple trailing slashes',
          fn: async () => {
            expect(stripTrailingSlashes('/usr/share///',),).toBe('/usr/share',);
          },
        },),

        it({
          name: 'collapses an all-slash string to empty',
          fn: async () => {
            expect(stripTrailingSlashes('///',),).toBe('',);
          },
        },),

        it({
          name: 'collapses a single slash to empty',
          fn: async () => {
            expect(stripTrailingSlashes('/',),).toBe('',);
          },
        },),

        it({
          name: 'preserves leading slashes',
          fn: async () => {
            expect(stripTrailingSlashes('///foo',),).toBe('///foo',);
          },
        },),

        it({
          name: 'strips trailing while preserving leading slashes',
          fn: async () => {
            expect(stripTrailingSlashes('///foo///',),).toBe('///foo',);
          },
        },),

        it({
          name: 'does not strip a trailing backslash',
          fn: async () => {
            expect(stripTrailingSlashes('/foo\\',),).toBe('/foo\\',);
          },
        },),

        it({
          name: 'strips a long trailing run in one linear pass',
          fn: async () => {
            expect(stripTrailingSlashes(`/x${'/'.repeat(LONG_RUN,)}`,),).toBe('/x',);
          },
        },),
      ],
    },),
  ],
},);
