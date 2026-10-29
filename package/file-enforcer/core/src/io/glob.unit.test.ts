import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  mkdir,
  mkdtemp,
  rm,
  writeFile,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';
import {
  expandGlob,
  firstGlobMetaIndex,
  mirrorGlobPath,
} from '../../dist/final/node/index.mjs';

/** Creates a fresh temp directory */
async function makeTmpDir(): Promise<string> {
  return mkdtemp(join(scratchRoot(), 'file-enforcer-glob-',),);
}

/** Removes the temp directory */
async function teardown(tempDir: string,): Promise<void> {
  await rm(tempDir, { recursive: true, force: true, },);
}

await describe({
  name: '',
  children: [
    //region mirrorGlobPath

    describe({
      name: mirrorGlobPath.name,
      children: [
        it({
          name: 'substitutes a single wildcard',
          fn: async () => {
            expect(
              mirrorGlobPath({
                sourcePattern: 'src/*.ts',
                destPattern: 'dist/*.ts',
                sourcePath: 'src/index.ts',
              },),
            )
              .toBe('dist/index.ts',);
          },
        },),
        it({
          name: 'substitutes multiple wildcards positionally',
          fn: async () => {
            expect(
              mirrorGlobPath({
                sourcePattern: 'package/*/src/*.ts',
                destPattern: 'output/*/lib/*.ts',
                sourcePath: 'package/foo/src/bar.ts',
              },),
            )
              .toBe('output/foo/lib/bar.ts',);
          },
        },),
        it({
          name: 'handles wildcards capturing multi-character segments',
          fn: async () => {
            expect(
              mirrorGlobPath({
                sourcePattern: 'a/*/b',
                destPattern: 'x/*/y',
                sourcePath: 'a/long-segment-name/b',
              },),
            )
              .toBe('x/long-segment-name/y',);
          },
        },),
        it({
          name: 'handles pattern with no wildcards (literal copy)',
          fn: async () => {
            expect(
              mirrorGlobPath({
                sourcePattern: 'exact/path.ts',
                destPattern: 'other/path.ts',
                sourcePath: 'exact/path.ts',
              },),
            )
              .toBe('other/path.ts',);
          },
        },),
        it({
          name: 'throws when wildcard counts differ between source and dest',
          fn: async () => {
            expect(() =>
              mirrorGlobPath({
                sourcePattern: 'src/*.ts',
                destPattern: 'dist/*/*.ts',
                sourcePath: 'src/a.ts',
              },)
            )
              .toThrow(
              'Wildcard count mismatch',
              );
          },
        },),
        it({
          name: 'throws when source path does not match source pattern prefix',
          fn: async () => {
            expect(() =>
              mirrorGlobPath({
                sourcePattern: 'src/*.ts',
                destPattern: 'dist/*.ts',
                sourcePath: 'lib/index.ts',
              },)
            )
              .toThrow(
              'does not match pattern',
              );
          },
        },),
        it({
          name: 'throws when source path does not match source pattern suffix',
          fn: async () => {
            expect(() =>
              mirrorGlobPath({
                sourcePattern: 'src/*.ts',
                destPattern: 'dist/*.ts',
                sourcePath: 'src/index.js',
              },)
            )
              .toThrow(
              'does not match pattern',
              );
          },
        },),
        it({
          name: 'handles wildcard capturing empty string',
          fn: async () => {
            // Wildcard captures empty segment when path has nothing between fixed parts
            expect(
              mirrorGlobPath({
                sourcePattern: 'src/*-suffix.ts',
                destPattern: 'out/*-suffix.ts',
                sourcePath: 'src/-suffix.ts',
              },),
            )
              .toBe('out/-suffix.ts',);
          },
        },),
        it({
          name: 'handles wildcard at the very start of pattern',
          fn: async () => {
            expect(
              mirrorGlobPath({
                sourcePattern: '*.txt',
                destPattern: '*.md',
                sourcePath: 'readme.txt',
              },),
            )
              .toBe('readme.md',);
          },
        },),
        it({
          name: 'handles wildcard at the very end of pattern',
          fn: async () => {
            expect(
              mirrorGlobPath({
                sourcePattern: 'prefix-*',
                destPattern: 'output-*',
                sourcePath: 'prefix-data',
              },),
            )
              .toBe('output-data',);
          },
        },),
        it({
          name: 'handles adjacent wildcards in source and dest',
          fn: async () => {
            expect(
              mirrorGlobPath({
                sourcePattern: 'a/*/*.ext',
                destPattern: 'b/*/*.ext',
                sourcePath: 'a/dir/file.ext',
              },),
            )
              .toBe('b/dir/file.ext',);
          },
        },),
      ],
    },),

    //endregion mirrorGlobPath

    //region expandGlob

    describe({
      name: expandGlob.name,
      children: [
        it({
          name: 'matches files with a wildcard extension pattern',
          fn: async () => {
            const tempDir = await makeTmpDir();
            await writeFile(join(tempDir, 'a.ts',), 'a',);
            await writeFile(join(tempDir, 'b.ts',), 'b',);
            await writeFile(join(tempDir, 'c.js',), 'c',);

            /** Matched paths should only include .ts files */
            const matches = await expandGlob(join(tempDir, '*.ts',),);
            expect([...matches,].toSorted(),).toEqual(
              [join(tempDir, 'a.ts',), join(tempDir, 'b.ts',),].toSorted(),
            );
            await teardown(tempDir,);
          },
        },),
        it({
          name: 'returns empty array when nothing matches',
          fn: async () => {
            const tempDir = await makeTmpDir();
            /** Pattern that matches no files in the empty temp dir */
            const matches = await expandGlob(join(tempDir, '*.nonexistent',),);
            expect(matches,).toEqual([],);
            await teardown(tempDir,);
          },
        },),
        it({
          name: 'includes dot-files when dot option is enabled',
          fn: async () => {
            const tempDir = await makeTmpDir();
            await writeFile(join(tempDir, '.hidden',), 'secret',);
            await writeFile(join(tempDir, 'visible',), 'public',);

            /** Pattern matching everything in the temp dir */
            const matches = await expandGlob(join(tempDir, '*',),);
            /** Should include the dot-file */
            const filenames = matches.map(match => match.split('/',).at(-1,));
            expect(filenames,).toContain('.hidden',);
            await teardown(tempDir,);
          },
        },),
        it({
          name: 'matches files in nested directories with double-star',
          fn: async () => {
            const tempDir = await makeTmpDir();
            /** Nested directory structure */
            const nested = join(tempDir, 'sub', 'deep',);
            await mkdir(nested, { recursive: true, },);
            await writeFile(join(nested, 'found.ts',), 'deep',);
            await writeFile(join(tempDir, 'top.ts',), 'top',);

            /** Double-star should find files at all depths */
            const matches = await expandGlob(join(tempDir, '**/*.ts',),);
            expect(matches.length,).toBe(2,);
            await teardown(tempDir,);
          },
        },),
      ],
    },),
    //endregion expandGlob

    //region firstGlobMetaIndex

    describe({
      name: firstGlobMetaIndex.name,
      children: [
        it({
          name: 'returns -1 for an empty string',
          fn: async function emptyString() {
            expect(firstGlobMetaIndex('',),).toBe(-1,);
          },
        },),
        it({
          name: 'returns -1 when no metacharacter is present',
          fn: async function noMetacharacter() {
            expect(firstGlobMetaIndex('src/index.ts',),).toBe(-1,);
          },
        },),
        it({
          name: 'returns 0 when the first character is a metacharacter',
          fn: async function metacharacterAtStart() {
            expect(firstGlobMetaIndex('*foo',),).toBe(0,);
          },
        },),
        it({
          name: 'returns the index of a metacharacter in the middle',
          fn: async function metacharacterInMiddle() {
            expect(firstGlobMetaIndex('src/*.ts',),).toBe(4,);
          },
        },),
        it({
          name: 'detects each metacharacter kind',
          fn: async function eachMetacharacterKind() {
            expect(firstGlobMetaIndex('a*b',),).toBe(1,);
            expect(firstGlobMetaIndex('a?b',),).toBe(1,);
            expect(firstGlobMetaIndex('a{b',),).toBe(1,);
            expect(firstGlobMetaIndex('a[b',),).toBe(1,);
          },
        },),
        it({
          name: 'returns the earliest index when several metacharacters appear',
          fn: async function earliestOfSeveral() {
            expect(firstGlobMetaIndex('a*b?c',),).toBe(1,);
            expect(firstGlobMetaIndex('ab[c]*',),).toBe(2,);
          },
        },),
        it({
          name: 'returns the index of a trailing metacharacter',
          fn: async function trailingMetacharacter() {
            expect(firstGlobMetaIndex('abc*',),).toBe(3,);
          },
        },),
        it({
          name: 'treats path separators as ordinary characters, not metacharacters',
          fn: async function pathSeparatorsAreNotMeta() {
            expect(firstGlobMetaIndex(String.raw`a/b\c`,),).toBe(-1,);
          },
        },),
        it({
          name: 'measures the metacharacter index in UTF-16 code units, not code points',
          fn: async function codeUnitIndexing() {
            // The astral emoji is two UTF-16 code units, so the metacharacter
            // after it sits at code-unit index 2. splitGlob slices on this
            // index, so code-unit positions must be preserved.
            expect(firstGlobMetaIndex('\u{1F600}*',),).toBe(2,);
          },
        },),
        it({
          name: 'returns -1 over a long run with no metacharacter, linear and stack-safe',
          fn: async function longRunNoMatch() {
            const runLength = 100_000;
            expect(
              firstGlobMetaIndex('a'.repeat(runLength,),),
            ).toBe(-1,);
          },
        },),
        it({
          name: 'finds a metacharacter at the end of a long run, linear and stack-safe',
          fn: async function longRunTrailingMatch() {
            const runLength = 100_000;
            expect(
              firstGlobMetaIndex(`${'a'.repeat(runLength,)}*`,),
            ).toBe(runLength,);
          },
        },),
      ],
    },),

    //endregion firstGlobMetaIndex
  ],
},);
