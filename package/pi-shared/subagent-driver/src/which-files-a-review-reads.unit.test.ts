/**
 Which files a review reads, by the extension of their name: the two lists, the extension a path carries, and the three answers a
 path gets.

 The lists are the ones this repository's files were measured against on 2026-11-14. The measurement itself is a check kept beside
 the change that added them, because it runs `git ls-files` and a suite reads no checkout: these tests hold the answers a path gets
 from the lists as they stand.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  CODE_FILE_EXTENSIONS,
  LEFT_OUT_FILE_EXTENSIONS,
  extensionOfAPath,
  whatAReviewDoesWithAFile,
} from '../dist/final/node/index.mjs';

/**
 The folder the paths of these tests sit under.

 @returns the folder, from the top of the checkout

 @example
 ```ts
 const folder = aSourceFolder();
 ```
 */
function aSourceFolder(): string {
  return 'package/a/src';
}

/**
 A path of a file whose extension is the one given.

 @param extension - the extension, with its dot

 @returns the path, from the top of the checkout

 @example
 ```ts
 const path = aPathOfTheExtension({ extension: '.rs', },);
 ```
 */
function aPathOfTheExtension({ extension, }: { readonly extension: string },): string {
  return `${aSourceFolder()}/a-file${extension}`;
}

await describe({
  name: '',
  children: [
    describe({
      name: extensionOfAPath.name,
      children: [
        it({
          name: 'reads the extension the file\'s own name ends with, not the folder it sits in, and takes the last dot of a name that carries several',
          fn: async () => {
            expect(extensionOfAPath({ path: `${aSourceFolder()}/index.d.cts`, },),).toBe('.cts',);
            expect(extensionOfAPath({ path: `${aSourceFolder()}/archive.tar.gz`, },),).toBe('.gz',);
            expect(extensionOfAPath({ path: 'package/a.b/index', },),).toBe('',);
          },
        },),
        it({
          name: 'reads no extension from a name that carries none, whether the dot leads it or the name holds no dot at all, because a hidden name is no extension',
          fn: async () => {
            expect(extensionOfAPath({ path: `${aSourceFolder()}/.gitignore`, },),).toBe('',);
            expect(extensionOfAPath({ path: 'LICENSE', },),).toBe('',);
            expect(extensionOfAPath({ path: `${aSourceFolder()}/a-file.`, },),).toBe('',);
          },
        },),
      ],
    },),
    describe({
      name: whatAReviewDoesWithAFile.name,
      children: [
        it({
          name: 'reads a file of every code extension this repository tracks as the code of a change, so no language its packages are written in is dropped',
          fn: async () => {
            expect(CODE_FILE_EXTENSIONS.size,).toBeGreaterThan(0,);
            /**
             The extensions this test found answered as something other than code.
             */
            const notReadAsCode = [...CODE_FILE_EXTENSIONS].filter(function isNotCode(extension,) {
              return whatAReviewDoesWithAFile({ path: aPathOfTheExtension({ extension, },), },) !== 'reads-it-as-code';
            },);
            expect(notReadAsCode,).toEqual([],);
          },
        },),
        it({
          name: 'leaves out a file of every extension that holds a note, data, generated output, a picture, a sound or a font, and leaves out a name that carries no extension at all',
          fn: async () => {
            expect(LEFT_OUT_FILE_EXTENSIONS.size,).toBeGreaterThan(0,);
            /**
             The extensions this test found read as something other than a file a review leaves out.
             */
            const readAsSomething = [...LEFT_OUT_FILE_EXTENSIONS].filter(function isRead(extension,) {
              return whatAReviewDoesWithAFile({ path: aPathOfTheExtension({ extension, },), },) !== 'leaves-it-out';
            },);
            expect(readAsSomething,).toEqual([],);
            expect(whatAReviewDoesWithAFile({ path: 'LICENSE', },),).toBe('leaves-it-out',);
          },
        },),
        it({
          name: 'lists no extension in both the code and the left-out list, so one file cannot be read and left out at once',
          fn: async () => {
            /**
             The extensions both lists name.
             */
            const inBoth = [...CODE_FILE_EXTENSIONS].filter(function isLeftOut(extension,) {
              return LEFT_OUT_FILE_EXTENSIONS.has(extension,);
            },);
            expect(inBoth,).toEqual([],);
          },
        },),
        it({
          name: 'reads the `.cjs` source and the `.d.cts` types of a shim as code, which is the change of `7ec2fd422d` that a range names',
          fn: async () => {
            expect(whatAReviewDoesWithAFile({ path: 'package/pnpm-override/shim-proper-lockfile/index.cjs', },),)
              .toBe('reads-it-as-code',);
            expect(whatAReviewDoesWithAFile({ path: 'package/pnpm-override/shim-proper-lockfile/index.d.cts', },),)
              .toBe('reads-it-as-code',);
          },
        },),
        it({
          name: 'reads a file of an extension in neither list as code of a kind no list names, so a language nobody added here yet is read instead of dropped',
          fn: async () => {
            expect(whatAReviewDoesWithAFile({ path: `${aSourceFolder()}/a-file.zig`, },),)
              .toBe('reads-it-as-code-of-an-unlisted-kind',);
          },
        },),
        it({
          name: 'leaves out a generated wrapper, the lockfile a tool writes and the note beside them, so build output and a document are never read as the code of a change',
          fn: async () => {
            expect(whatAReviewDoesWithAFile({ path: 'package/a/gradle/wrapper/gradle-wrapper.jar', },),).toBe('leaves-it-out',);
            expect(whatAReviewDoesWithAFile({ path: 'package/a/Cargo.lock', },),).toBe('leaves-it-out',);
            expect(whatAReviewDoesWithAFile({ path: 'package/a/README.md', },),).toBe('leaves-it-out',);
          },
        },),
      ],
    },),
  ],
},);
