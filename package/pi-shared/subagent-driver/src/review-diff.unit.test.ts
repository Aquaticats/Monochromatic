/**
 Reading the diff a review was given: the files it touches, the lines it holds, and the part of it that touches only some files.

 Each case is a diff text the test writes, so no checkout, worktree or process is involved.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  ReviewDiffError,
  diffOfOnlyFiles,
  filesInDiff,
} from '../dist/final/node/index.mjs';

/**
 The diff every test starts from.

 @returns the diff's text, touching two files

 @example
 ```ts
 const diff = aDiff();
 ```
 */
function aDiff(): string {
  return [
    'diff --git a/package/a/src/one.rs b/package/a/src/one.rs',
    '--- a/package/a/src/one.rs',
    '+++ b/package/a/src/one.rs',
    '@@ -1 +1 @@',
    '-old',
    '+new',
    'diff --git a/package/a/src/two.rs b/package/a/src/two.rs',
    '--- a/package/a/src/two.rs',
    '+++ b/package/a/src/two.rs',
    '@@ -1 +1 @@',
    '-old',
    '+new',
    '',
  ].join('\n',);
}

await describe({
  name: '',
  children: [
    describe({
      name: filesInDiff.name,
      children: [
        it({
          name: 'reads each file the diff touches once, in the order git named them, and the lines the file holds',
          fn: async () => {
            expect(filesInDiff({ diff: aDiff(), },),).toEqual({
              lines: 12,
              files: ['package/a/src/one.rs', 'package/a/src/two.rs',],
            },);
          },
        },),
        it({
          name: 'names a file once although a diff repeats its header, which a merge of two ranges writes',
          fn: async () => {
            expect(filesInDiff({ diff: [aDiff(), aDiff(),].join('',), },).files,).toEqual([
              'package/a/src/one.rs',
              'package/a/src/two.rs',
            ],);
          },
        },),
        it({
          name: 'counts no line for a diff that touches nothing, and names no file',
          fn: async () => {
            expect(filesInDiff({ diff: '', },),).toEqual({
              lines: 0,
              files: [],
            },);
          },
        },),
        it({
          name: 'refuses a header git wrote with a quoted path, because the quoted text is not the path a reviewer copies',
          fn: async () => {
            try {
              filesInDiff({ diff: 'diff --git "a/a b.rs" "b/a b.rs"\n', },);
              expect('it read',).toBe('it refused',);
            }
            catch (error) {
              expect(error,).toBeInstanceOf(ReviewDiffError,);
              expect((error as Error).message,).toContain('cannot compare',);
            }
          },
        },),
      ],
    },),
    describe({
      name: diffOfOnlyFiles.name,
      children: [
        it({
          name: 'keeps only the sections of the files it is given, and the lines before the first section',
          fn: async () => {
            const part = diffOfOnlyFiles({
              diff: aDiff(),
              files: ['package/a/src/two.rs',],
            },);
            expect(part,).toContain('package/a/src/two.rs',);
            expect(part,).not.toContain('package/a/src/one.rs',);
          },
        },),
        it({
          name: 'refuses a diff that touches none of the files, because a brief with no file reads as a clean review',
          fn: async () => {
            try {
              diffOfOnlyFiles({
                diff: aDiff(),
                files: ['package/a/src/three.rs',],
              },);
              expect('it read',).toBe('it refused',);
            }
            catch (error) {
              expect(error,).toBeInstanceOf(ReviewDiffError,);
              expect((error as Error).message,).toContain('touches none of the files');
            }
          },
        },),
      ],
    },),
  ],
},);
