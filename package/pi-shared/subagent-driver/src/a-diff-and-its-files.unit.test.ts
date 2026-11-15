/**
 One answer of `git diff --raw -z --patch`, read as the diff a review is given and the paths it touches.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { diffAndFilesIn, } from '../dist/final/node/index.mjs';

/**
 The byte Git ends every field with when `-z` is given.

 @returns the byte

 @example
 ```ts
 const fieldEnd = fieldEnd();
 ```
 */
function fieldEnd(): string {
  return '\0';
}

/**
 Writes one raw entry, as the part of one answer before its patch.

 @param status - the status Git gave the file

 @param path - the file's path

 @param secondPath - the destination a rename or a copy adds

 @returns the entry, as the answer holds it

 @example
 ```ts
 const entry = entryOf({ status: 'A', path: 'src/a.ts', },);
 ```
 */
function entryOf({
  status,
  path,
  secondPath,
}: {
  readonly status: string;
  readonly path: string;
  readonly secondPath?: string;
},): string {
  /**
   The paths this entry keeps, which a rename or a copy gives two of.
   */
  const paths = (secondPath === undefined) ? [path,] : [path, secondPath,];
  return `:100644 100644 0000000 1234567 ${status}${fieldEnd()}${paths.map(function ended(one,) {
      return `${one}${fieldEnd()}`;
    },)
      .join('',)}`;
}

await describe({
  name: '',
  children: [
    describe({
      name: diffAndFilesIn.name,
      children: [
        it({
          name: 'reads the patch and the paths a range added and modified',
          fn: async () => {
            /**
             The answer one question gave: both entries, then the field before the patch, then the patch.
             */
            const answer = `${entryOf({
              status: 'A',
              path: 'package/a/src/added.ts',
            },)
              + entryOf({
                status: 'M',
                path: 'package/a/src/changed.ts',
              },)
              + fieldEnd()
               }diff --git a/package/a/src/added.ts b/package/a/src/added.ts\n+added\n`
              + `diff --git a/package/a/src/changed.ts b/package/a/src/changed.ts\n+changed\n`;
            expect(diffAndFilesIn({ answer, },),).toEqual({
              diff: 'diff --git a/package/a/src/added.ts b/package/a/src/added.ts\n+added\n'
                + 'diff --git a/package/a/src/changed.ts b/package/a/src/changed.ts\n+changed\n',
              paths: ['package/a/src/added.ts', 'package/a/src/changed.ts',],
            },);
          },
        },),
        it({
          name: 'leaves out a deleted, a renamed and a copied file, which --diff-filter=AM leaves out too',
          fn: async () => {
            /**
             The answer one question gave, with one file of each status and a patch of its own.
             */
            const answer = `${entryOf({ status: 'D', path: 'package/a/src/deleted.ts', },)
              + entryOf({
                status: 'R100',
                path: 'package/a/src/before.ts',
                secondPath: 'package/a/src/after.ts',
              },)
              + entryOf({
                status: 'C75',
                path: 'package/a/src/source.ts',
                secondPath: 'package/a/src/copy.ts',
              },)
              + entryOf({ status: 'T', path: 'package/a/src/link.ts', },)
              + fieldEnd()
               }diff --git a/package/a/src/before.ts b/package/a/src/after.ts\nsimilarity index 100%\n`;
            expect(diffAndFilesIn({ answer, },).paths,).toEqual([],);
            // The patch of every entry is what the reviewer reads, whatever the status of the file it is about.
            expect(diffAndFilesIn({ answer, },).diff,).toContain('similarity index 100%',);
          },
        },),
        it({
          name: 'reads a path holding a tab, a space and a newline whole, because -z ends every field',
          fn: async () => {
            /**
             The path, which holds the three bytes a path may hold and a line-based answer could not carry.
             */
            const path = 'package/a/src/a\tb c\nd.ts';
            expect(diffAndFilesIn({
              answer: `${entryOf({ status: 'A', path, },) + fieldEnd()  }diff --git a/the-path b/the-path\n+it\n`,
            },).paths,).toEqual([path,],);
          },
        },),
        it({
          name: 'answers no diff and no path for a range that changed nothing',
          fn: async () => {
            expect(diffAndFilesIn({ answer: '', },),).toEqual({
              diff: '',
              paths: [],
            },);
          },
        },),
        it({
          name: 'answers no diff for an answer that is neither a raw entry nor the field before its patch',
          fn: async () => {
            expect(diffAndFilesIn({ answer: 'not a raw entry\n', },),).toEqual({
              diff: '',
              paths: [],
            },);
          },
        },),
      ],
    },),
  ],
},);
