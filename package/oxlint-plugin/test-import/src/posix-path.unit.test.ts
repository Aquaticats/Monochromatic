import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  isUnderAnyDirectory,
  isUnderDirectory,
  resolvePosix,
  toPosixPath,
} from '../dist/final/node/index.mjs';

await describe({
  name: 'posix path normalization',
  children: [
    describe({
      name: toPosixPath.name,
      children: [
        it({
          name: 'leaves an already-normalized path untouched',
          fn: async () => {
            expect(toPosixPath({ path: '/repo/package/x', },),).toBe('/repo/package/x',);
          },
        },),
      ],
    },),

    describe({
      name: resolvePosix.name,
      children: [
        it({
          name: 'resolves a same-directory specifier',
          fn: async () => {
            expect(resolvePosix({
              base: '/repo/src',
              specifier: './parse.ts',
            },),).toBe('/repo/src/parse.ts',);
          },
        },),
        it({
          name: 'resolves a parent-directory specifier',
          fn: async () => {
            expect(resolvePosix({
              base: '/repo/src',
              specifier: '../dist/final/node/index.mjs',
            },),).toBe('/repo/dist/final/node/index.mjs',);
          },
        },),
        it({
          name: 'collapses redundant traversal',
          fn: async () => {
            expect(resolvePosix({
              base: '/repo/src',
              specifier: './nested/../parse.ts',
            },),).toBe('/repo/src/parse.ts',);
          },
        },),
        it({
          name: 'resolves without probing the file system, so missing targets still normalize',
          fn: async () => {
            expect(resolvePosix({
              base: '/repo/src',
              specifier: '../dist/final/node/never-built.mjs',
            },),).toBe('/repo/dist/final/node/never-built.mjs',);
          },
        },),
      ],
    },),

    describe({
      name: isUnderDirectory.name,
      children: [
        it({
          name: 'accepts the directory itself',
          fn: async () => {
            expect(isUnderDirectory({
              directory: '/repo/dist/final',
              path: '/repo/dist/final',
            },),).toBe(true,);
          },
        },),
        it({
          name: 'accepts a nested path',
          fn: async () => {
            expect(isUnderDirectory({
              directory: '/repo/dist/final',
              path: '/repo/dist/final/node/index.mjs',
            },),).toBe(true,);
          },
        },),
        it({
          name: 'rejects a sibling sharing the directory name as a prefix',
          fn: async () => {
            expect(isUnderDirectory({
              directory: '/repo/dist',
              path: '/repo/dist-extra/index.mjs',
            },),).toBe(false,);
          },
        },),
        it({
          name: 'rejects an unrelated path',
          fn: async () => {
            expect(isUnderDirectory({
              directory: '/repo/dist/final',
              path: '/repo/src/parse.ts',
            },),).toBe(false,);
          },
        },),
        it({
          name: 'rejects a path that only looks nested after traversal',
          fn: async () => {
            expect(isUnderDirectory({
              directory: '/repo/dist/final',
              path: '/repo/src/dist/final/fake.ts',
            },),).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: isUnderAnyDirectory.name,
      children: [
        it({
          name: 'rejects when no directory is configured',
          fn: async () => {
            expect(isUnderAnyDirectory({
              directories: [],
              path: '/repo/dist/final/index.mjs',
            },),).toBe(false,);
          },
        },),
        it({
          name: 'accepts when any directory contains the path',
          fn: async () => {
            expect(isUnderAnyDirectory({
              directories: [
                '/repo/dist/final',
                '/repo/dist/app',
              ],
              path: '/repo/dist/app/strip.js',
            },),).toBe(true,);
          },
        },),
        it({
          name: 'rejects when every directory misses',
          fn: async () => {
            expect(isUnderAnyDirectory({
              directories: [
                '/repo/dist/final',
                '/repo/dist/app',
              ],
              path: '/repo/dist/temp/strip.js',
            },),).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);
