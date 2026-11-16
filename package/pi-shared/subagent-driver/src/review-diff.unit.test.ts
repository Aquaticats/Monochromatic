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
          name: 'reads a path holding a space and a separator as the file git names, which the header alone cannot settle (F8)',
          fn: async () => {
            /**
             The diff of a file whose own name holds ` b/`.
             */
            const awkward = 'diff --git a/foo b/bar.ts b/foo b/bar.ts\n'
              + '--- a/foo b/bar.ts\n'
              + '+++ b/foo b/bar.ts\n'
              + '@@ -1 +1 @@\n'
              + '-old\n'
              + '+new\n';
            expect(filesInDiff({ diff: awkward, },).files,).toEqual(['foo b/bar.ts',],);
            expect(diffOfOnlyFiles({
              diff: awkward,
              files: ['foo b/bar.ts',],
            },),).toContain('+++ b/foo b/bar.ts',);
          },
        },),
        it({
          name: 'reads a renamed file as the path its post-image line names, and a deleted one as the path its pre-image line names',
          fn: async () => {
            /**
             A rename, whose two header paths differ.
             */
            const renamed = 'diff --git a/old name.ts b/new name.ts\n'
              + '--- a/old name.ts\n'
              + '+++ b/new name.ts\n'
              + '@@ -1 +1 @@\n'
              + '-old\n'
              + '+new\n';
            expect(filesInDiff({ diff: renamed, },).files,).toEqual(['new name.ts',],);
            /**
             A deletion, whose post-image line names no file.
             */
            const deleted = 'diff --git a/gone.ts b/gone.ts\n'
              + '--- a/gone.ts\n'
              + '+++ /dev/null\n'
              + '@@ -1 +0,0 @@\n'
              + '-old\n';
            expect(filesInDiff({ diff: deleted, },).files,).toEqual(['gone.ts',],);
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
