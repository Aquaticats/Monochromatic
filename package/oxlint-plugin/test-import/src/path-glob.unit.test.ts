import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  matchesAnyGlob,
  matchesGlob,
} from '../dist/final/node/index.mjs';

/** Repetition count proving the scans stay linear on long input. */
const LONG_RUN = 2_000;

await describe({
  name: 'path glob matching',
  children: [
    describe({
      name: matchesGlob.name,
      children: [
        it({
          name: 'matches an identical literal path',
          fn: async () => {
            expect(matchesGlob({
              pattern: '/a/b/c.ts',
              path: '/a/b/c.ts',
            },),).toBe(true,);
          },
        },),
        it({
          name: 'rejects a different literal path',
          fn: async () => {
            expect(matchesGlob({
              pattern: '/a/b/c.ts',
              path: '/a/b/d.ts',
            },),).toBe(false,);
          },
        },),
        it({
          name: 'matches a suffix wildcard inside one segment',
          fn: async () => {
            expect(matchesGlob({
              pattern: '/a/fixture.*',
              path: '/a/fixture.json',
            },),).toBe(true,);
          },
        },),
        it({
          name: 'matches a prefix wildcard inside one segment',
          fn: async () => {
            expect(matchesGlob({
              pattern: '/a/*-helpers.ts',
              path: '/a/tree-helpers.ts',
            },),).toBe(true,);
          },
        },),
        it({
          name: 'matches two wildcards inside one segment',
          fn: async () => {
            expect(matchesGlob({
              pattern: '/a/*-fixture*.ts',
              path: '/a/toml-fixture-data.ts',
            },),).toBe(true,);
          },
        },),
        it({
          name: 'lets a segment wildcard match nothing',
          fn: async () => {
            expect(matchesGlob({
              pattern: '/a/*b.ts',
              path: '/a/b.ts',
            },),).toBe(true,);
          },
        },),
        it({
          name: 'stops a segment wildcard at the separator',
          fn: async () => {
            expect(matchesGlob({
              pattern: '/a/*.ts',
              path: '/a/b/c.ts',
            },),).toBe(false,);
          },
        },),
        it({
          name: 'lets a path wildcard match zero segments',
          fn: async () => {
            expect(matchesGlob({
              pattern: '**/c.ts',
              path: 'c.ts',
            },),).toBe(true,);
          },
        },),
        it({
          name: 'lets a path wildcard match many segments',
          fn: async () => {
            expect(matchesGlob({
              pattern: '**/c.ts',
              path: '/a/b/deep/c.ts',
            },),).toBe(true,);
          },
        },),
        it({
          name: 'lets a trailing path wildcard match nothing',
          fn: async () => {
            expect(matchesGlob({
              pattern: '/a/b/**',
              path: '/a/b',
            },),).toBe(true,);
          },
        },),
        it({
          name: 'backtracks when a later segment forces the wildcard to grow',
          fn: async () => {
            expect(matchesGlob({
              pattern: '**/src/*.test.ts',
              path: '/repo/src/nested/src/a.test.ts',
            },),).toBe(true,);
          },
        },),
        it({
          name: 'rejects when the pattern runs out before the path does',
          fn: async () => {
            expect(matchesGlob({
              pattern: '/a',
              path: '/a/b',
            },),).toBe(false,);
          },
        },),
        it({
          name: 'rejects when the path runs out before the pattern does',
          fn: async () => {
            expect(matchesGlob({
              pattern: '/a/b',
              path: '/a',
            },),).toBe(false,);
          },
        },),
        it({
          name: 'handles a long non-matching run without blowing up',
          fn: async () => {
            /** Path segment that forces the wildcard scan to backtrack repeatedly. */
            const text = 'a'.repeat(LONG_RUN,);
            expect(matchesGlob({
              pattern: `/${'*a'.repeat(20,)}b`,
              path: `/${text}`,
            },),).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: matchesAnyGlob.name,
      children: [
        it({
          name: 'rejects when no pattern is configured',
          fn: async () => {
            expect(matchesAnyGlob({
              patterns: [],
              path: '/a/b.ts',
            },),).toBe(false,);
          },
        },),
        it({
          name: 'accepts when any pattern covers the path',
          fn: async () => {
            expect(matchesAnyGlob({
              patterns: [
                '**/never.ts',
                '**/*-harness.ts',
              ],
              path: '/a/render-harness.ts',
            },),).toBe(true,);
          },
        },),
        it({
          name: 'rejects when every pattern misses',
          fn: async () => {
            expect(matchesAnyGlob({
              patterns: [
                '**/never.ts',
                '**/*-harness.ts',
              ],
              path: '/a/parse.ts',
            },),).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);
