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
import {
  join,
  resolve,
} from 'node:path';
import {
  cat,
  type GlobResults,
  reads,
} from '../../dist/final/node/index.mjs';

// No reset() in setup: each test uses a unique mkdtemp path, so tracker keys
// never collide. Clearing the globals under concurrent test execution wipes
// sibling tests' entries mid-flight, causing false failures. Mirrors the
// convention documented in ./cache.unit.test.ts.
async function setup(prefix: string,): Promise<string> {
  return mkdtemp(join(scratchRoot(), prefix,),);
}

/** Removes the temp directory */
async function teardown(tempDir: string,): Promise<void> {
  await rm(tempDir, { recursive: true, force: true, },);
}

await describe({
  name: '',
  children: [
    //region cat(string[]): array mode

    describe({
      name: 'cat(string[])',
      children: [
        it({
          name: 'concatenates multiple files with newline separator',
          fn: async () => {
            const tempDir = await setup('file-enforcer-cat-',);
            await writeFile(join(tempDir, 'a.txt',), 'hello',);
            await writeFile(join(tempDir, 'b.txt',), 'world',);

            /** Concatenated result of two files */
            const result = await cat([join(tempDir, 'a.txt',),
              join(tempDir, 'b.txt',),],);
            expect(result,).toBe('hello\nworld',);
            await teardown(tempDir,);
          },
        },),
        it({
          name: 'returns single file content when array has one element',
          fn: async () => {
            const tempDir = await setup('file-enforcer-cat-',);
            await writeFile(join(tempDir, 'only.txt',), 'solo',);

            /** Should return the content without any separator artifacts */
            const result = await cat([join(tempDir, 'only.txt',),],);
            expect(result,).toBe('solo',);
            await teardown(tempDir,);
          },
        },),
        it({
          name: 'auto-expands glob patterns within the array',
          fn: async () => {
            const tempDir = await setup('file-enforcer-cat-',);
            await writeFile(join(tempDir, 'x.ts',), 'one',);
            await writeFile(join(tempDir, 'y.ts',), 'two',);

            /** Glob in the array should be expanded and all matches concatenated */
            const result = await cat([join(tempDir, '*.ts',),],);
            /** Order may vary; check both pieces are present */
            expect(result.split('\n',).toSorted(),).toEqual(['one', 'two',].toSorted(),);
            await teardown(tempDir,);
          },
        },),
        it({
          name: 'handles mix of literal paths and globs',
          fn: async () => {
            const tempDir = await setup('file-enforcer-cat-',);
            await writeFile(join(tempDir, 'literal.txt',), 'fixed',);
            await writeFile(join(tempDir, 'matched.ts',), 'globbed',);

            /** Mixed array with one literal and one glob */
            const result = await cat([
              join(tempDir, 'literal.txt',),
              join(tempDir, '*.ts',),
            ],);
            expect(result,).toContain('fixed',);
            await teardown(tempDir,);
          },
        },),
        it({
          name: 'preserves file content exactly (no trimming)',
          fn: async () => {
            const tempDir = await setup('file-enforcer-cat-',);
            /** Content with leading/trailing whitespace and newlines */
            const content = '  spaces  \n\n  tabs\t\n';
            await writeFile(join(tempDir, 'whitespace.txt',), content,);

            const result = await cat([join(tempDir, 'whitespace.txt',),],);
            expect(result,).toBe(content,);
            await teardown(tempDir,);
          },
        },),
        it({
          name: 'handles empty file',
          fn: async () => {
            const tempDir = await setup('file-enforcer-cat-',);
            await writeFile(join(tempDir, 'empty.txt',), '',);

            const result = await cat([join(tempDir, 'empty.txt',),],);
            expect(result,).toBe('',);
            await teardown(tempDir,);
          },
        },),
        it({
          name: 'tracks read paths in the tracker',
          fn: async () => {
            const tempDir = await setup('file-enforcer-cat-',);
            const tracked = join(tempDir, 'tracked.txt',);
            await writeFile(tracked, 'data',);

            await cat([tracked,],);
            /** Tracker should have recorded the absolute read path (size check unsafe under concurrent execution) */
            expect(
              reads.has(resolve(tracked,),),
            ).toBe(true,);
            await teardown(tempDir,);
          },
        },),
      ],
    },),

    //endregion cat(string[]): array mode

    //region cat(string): glob mode

    describe({
      name: 'cat(string)',
      children: [
        it({
          name: 'returns GlobResult array with path and content per match',
          fn: async () => {
            const tempDir = await setup('file-enforcer-cat-glob-',);
            await writeFile(join(tempDir, 'file1.md',), 'content1',);
            await writeFile(join(tempDir, 'file2.md',), 'content2',);

            /** Glob mode returns structured results, not a flat string */
            const results = await cat(join(tempDir, '*.md',),);
            expect(results.length,).toBe(2,);
            /** Each result should have both path and content */
            const contents = results.map(result => result.content).toSorted();
            expect(contents,).toEqual(['content1', 'content2',],);
            await teardown(tempDir,);
          },
        },),
        it({
          name: 'returns empty array when glob matches nothing',
          fn: async () => {
            const tempDir = await setup('file-enforcer-cat-glob-',);
            /** Pattern with no matches */
            const results = await cat(join(tempDir, '*.xyz',),);
            expect(results,).toHaveLength(0,);
            await teardown(tempDir,);
          },
        },),
        it({
          name: 'includes the matched file path in each result',
          fn: async () => {
            const tempDir = await setup('file-enforcer-cat-glob-',);
            await writeFile(join(tempDir, 'named.ts',), 'code',);

            const results = await cat(join(tempDir, '*.ts',),);
            /** Path should end with the filename */
            expect(results[0]?.path,).toContain('named.ts',);
            await teardown(tempDir,);
          },
        },),
        it({
          name: 'tracks each matched file in the tracker',
          fn: async () => {
            const tempDir = await setup('file-enforcer-cat-glob-',);
            const r1 = join(tempDir, 'r1.ts',);
            const r2 = join(tempDir, 'r2.ts',);
            await writeFile(r1, 'a',);
            await writeFile(r2, 'b',);

            await cat(join(tempDir, '*.ts',),);
            /** Both files should be tracked individually (size check unsafe under concurrent execution) */
            expect(
              reads.has(resolve(r1,),),
            ).toBe(true,);
            expect(
              reads.has(resolve(r2,),),
            ).toBe(true,);
            await teardown(tempDir,);
          },
        },),
        it({
          name: 'matches files in nested directories',
          fn: async () => {
            const tempDir = await setup('file-enforcer-cat-glob-',);
            /** Nested directory with a matching file */
            const subDir = join(tempDir, 'nested',);
            await mkdir(subDir, { recursive: true, },);
            await writeFile(join(subDir, 'deep.ts',), 'nested',);
            await writeFile(join(tempDir, 'top.ts',), 'top',);

            /** Double-star should find both */
            const results = await cat(join(tempDir, '**/*.ts',),);
            expect(results.length,).toBe(2,);
            await teardown(tempDir,);
          },
        },),
      ],
    },),
    //endregion cat(string): glob mode
  ],
},);
