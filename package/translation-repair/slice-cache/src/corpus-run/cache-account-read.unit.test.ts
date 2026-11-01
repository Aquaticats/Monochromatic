/**
 Tests the declaration reading behind the pre-launch cache version check
 (ledger M28): every declaration ending in `CACHE_VERSION`, exported or not,
 read with its name whole; a refusal for one that does not read as
 `NAME = digits;`; a hash cited by nine characters; and a diff's added and
 removed declarations of one value,
 where a longer name or value holding it, a move, a historical line that
 does not read and a line a merge only brought in are each told apart.

 @module
 */

import {
  caught,
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  CacheAccountReadError,
  cacheVersionsIn,
  citedHash,
  declarationLineCounts,
} from '../../dist/final/node/index.mjs';

/**
 A diff that moves one version from 3 to 34 and adds a longer name holding
 the shorter one at 34.
 */
const NESTED_DIFF = `diff --git a/src/nap-key.ts b/src/nap-key.ts
--- a/src/nap-key.ts
+++ b/src/nap-key.ts
@@ -1 +1,2 @@
-export const SLICE_CACHE_VERSION = 3;
+export const SLICE_CACHE_VERSION = 34;
+export const TRANSLATE_SLICE_CACHE_VERSION = 34;`;

/**
 A diff moving one declaration between files unchanged.
 */
const MOVE_DIFF = `diff --git a/src/purr-key.ts b/src/purr-key.ts
--- a/src/purr-key.ts
+++ b/src/purr-key.ts
@@ -1 +0,0 @@
-export const PURR_CACHE_VERSION = 3;
diff --git a/src/purr-version.ts b/src/purr-version.ts
--- /dev/null
+++ b/src/purr-version.ts
@@ -0,0 +1 @@
+export const PURR_CACHE_VERSION = 3;`;

/**
 A merge's combined diff, one mark column per parent: the resolution writes
 3 in place of each side's 1 and 2, and a second constant stands in the
 second parent already, so the merge only brings it in.
 */
const MERGE_DIFF = `diff --cc src/nap-key.ts
index 1111111,2222222..3333333
--- a/src/nap-key.ts
+++ b/src/nap-key.ts
@@@ -1,1 -1,2 +1,2 @@@
- export const NAP_CACHE_VERSION = 1;
 -export const NAP_CACHE_VERSION = 2;
++export const NAP_CACHE_VERSION = 3;
+ export const PURR_CACHE_VERSION = 4;`;

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: cacheVersionsIn.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS EVERY DECLARATION ENDING IN CACHE_VERSION, exported or not, and nothing in prose or under another name',
          fn: async () => {
            expect(cacheVersionsIn({
              path: 'src/nap-key.ts',
              text: [
                '// NAP_CACHE_VERSION = 9 was the old value.',
                'export const NAP_CACHE_VERSION = 3;',
                '  const WHISKER_CACHE_VERSION = 12;',
                'export const NAP_WINDOW = 5;',
                'export const CACHE_VERSIONS = 7;',
              ].join('\n',),
            },),).toEqual([
              {
                name: 'NAP_CACHE_VERSION',
                value: 3,
                path: 'src/nap-key.ts',
                declaration: 'NAP_CACHE_VERSION = 3',
              },
              {
                name: 'WHISKER_CACHE_VERSION',
                value: 12,
                path: 'src/nap-key.ts',
                declaration: 'WHISKER_CACHE_VERSION = 12',
              },
            ],);
          },
        },),
        it({
          name: 'READS A NAME WHOLE, so a longer name ending the same way is its own version',
          fn: async () => {
            expect(cacheVersionsIn({
              path: 'src/nap-key.ts',
              text: 'export const TRANSLATE_SLICE_CACHE_VERSION = 15;',
            },).map(function nameOf(version,): string {
              return version.name;
            },),).toEqual(['TRANSLATE_SLICE_CACHE_VERSION',],);
          },
        },),
        it({
          name: 'REFUSES A DECLARATION NAMING A VERSION IT CANNOT READ, which the check would otherwise skip',
          fn: async () => {
            for (const text of [
              'export const NAP_CACHE_VERSION: number = 3;',
              'export const NAP_CACHE_VERSION = 3',
              'export const NAP_CACHE_VERSION = 0x3;',
              'export const NAP_CACHE_VERSION',
            ]) {
              expect(function read(): void {
                cacheVersionsIn({
                  path: 'src/nap-key.ts',
                  text,
                },);
              },).toThrow(CacheAccountReadError,);
            }
          },
        },),
        it({
          name: 'REFUSES A VERSION PAST THE LARGEST WHOLE NUMBER A DOUBLE HOLDS EXACTLY, which `Number` reads as a '
            + 'neighbouring version, so two different declarations would compare equal (ledger B73)',
          fn: async () => {
            /**
             Declaration whose value lies two past the exact range.
             */
            const line = `export const NAP_CACHE_VERSION = ${String(BigInt(Number.MAX_SAFE_INTEGER,) + 2n,)};`;
            /**
             What the reader threw.
             */
            const refusal = caught(function read(): void {
              cacheVersionsIn({
                path: 'src/nap-key.ts',
                text: line,
              },);
            },);
            expect(refusal,).toBeInstanceOf(CacheAccountReadError,);
            expect((refusal as Error).message,).toBe(`src/nap-key.ts declares a cache version the audit cannot read: "${
          line
        }". Write it as NAME = digits; so every constant is checked.`,);
          },
        },),
        it({
          name: 'SKIPS A NAME RUNNING TO ITS LINE END that does not end in the marker, since it is no cache version',
          fn: async () => {
            expect(cacheVersionsIn({
              path: 'src/nap-key.ts',
              text: 'export const NAP_CACHE_VERSION_x',
            },),).toEqual([],);
          },
        },),
      ],
    },),

    describe({
      name: citedHash.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'CITES A HASH BY ITS FIRST NINE CHARACTERS, as the version accounts do',
          fn: async () => {
            expect(citedHash({ hash: 'f6e93ed5f123', },),).toBe('f6e93ed5f',);
          },
        },),
      ],
    },),

    describe({
      name: declarationLineCounts.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'COUNTS ONLY DECLARATIONS OF THIS NAME AT THIS VALUE, never a longer name or value holding them',
          fn: async () => {
            expect(declarationLineCounts({
              diff: NESTED_DIFF,
              version: {
                name: 'SLICE_CACHE_VERSION',
                value: 34,
              },
            },),).toEqual({
              added: 1,
              removed: 0,
            },);
            expect(declarationLineCounts({
              diff: NESTED_DIFF,
              version: {
                name: 'SLICE_CACHE_VERSION',
                value: 3,
              },
            },),).toEqual({
              added: 0,
              removed: 1,
            },);
          },
        },),
        it({
          name: 'COUNTS A MOVE BETWEEN FILES AS ONE ADDED AND ONE REMOVED, which sets nothing',
          fn: async () => {
            expect(declarationLineCounts({
              diff: MOVE_DIFF,
              version: {
                name: 'PURR_CACHE_VERSION',
                value: 3,
              },
            },),).toEqual({
              added: 1,
              removed: 1,
            },);
          },
        },),
        it({
          name: 'COUNTS WHAT A MERGE RESOLUTION WROTE, marked added against every parent, and not what one parent '
            + 'already held, since a merge that only brings a line in sets nothing (ledger B814)',
          fn: async () => {
            expect(declarationLineCounts({
              diff: MERGE_DIFF,
              version: {
                name: 'NAP_CACHE_VERSION',
                value: 3,
              },
            },),).toEqual({
              added: 1,
              removed: 0,
            },);
            expect(declarationLineCounts({
              diff: MERGE_DIFF,
              version: {
                name: 'NAP_CACHE_VERSION',
                value: 2,
              },
            },),).toEqual({
              added: 0,
              removed: 0,
            },);
            expect(declarationLineCounts({
              diff: MERGE_DIFF,
              version: {
                name: 'PURR_CACHE_VERSION',
                value: 4,
              },
            },),).toEqual({
              added: 0,
              removed: 0,
            },);
          },
        },),
        it({
          name: 'COUNTS A LINE A MERGE REMOVED FROM EVERY PARENT AS REMOVED, so a value the resolution dropped is '
            + 'not taken as standing',
          fn: async () => {
            expect(declarationLineCounts({
              diff: '@@@ -1 -1 +0,0 @@@\n--export const NAP_CACHE_VERSION = 5;',
              version: {
                name: 'NAP_CACHE_VERSION',
                value: 5,
              },
            },),).toEqual({
              added: 0,
              removed: 1,
            },);
          },
        },),
        it({
          name: 'READS AN OCTOPUS MERGE\'S THREE MARK COLUMNS, so a line marked `+++` or `---` in every column '
            + 'counts and is not taken for a file header',
          fn: async () => {
            /**
             Combined diff of a merge of three parents that writes 6 over 5.
             */
            const diff = [
              '@@@@ -1,1 -1,1 -1,1 +1,1 @@@@',
              '---export const NAP_CACHE_VERSION = 5;',
              '+++export const NAP_CACHE_VERSION = 6;',
            ].join('\n',);
            expect(declarationLineCounts({
              diff,
              version: {
                name: 'NAP_CACHE_VERSION',
                value: 5,
              },
            },),).toEqual({
              added: 0,
              removed: 1,
            },);
            expect(declarationLineCounts({
              diff,
              version: {
                name: 'NAP_CACHE_VERSION',
                value: 6,
              },
            },),).toEqual({
              added: 1,
              removed: 0,
            },);
          },
        },),
        it({
          name: 'LEAVES A HISTORICAL LINE THAT DOES NOT READ UNCOUNTED rather than refusing the whole history',
          fn: async () => {
            expect(declarationLineCounts({
              diff: '+export const PURR_CACHE_VERSION: number = 3;',
              version: {
                name: 'PURR_CACHE_VERSION',
                value: 3,
              },
            },),).toEqual({
              added: 0,
              removed: 0,
            },);
          },
        },),
      ],
    },),
  ],
},);
