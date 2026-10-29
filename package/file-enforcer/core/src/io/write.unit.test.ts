import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  mkdir,
  mkdtemp,
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  join,
  resolve,
} from 'node:path';
import {
  globResults,
  overwrite,
  overwriteEach,
  overwriteIfNotExists,
  writes,
  writeTimestamps,
} from '../../dist/final/node/index.mjs';

// No reset()/resetWriteTimestamps() in setup: each test uses a unique mkdtemp
// path, so tracker keys never collide. Clearing the globals under concurrent
// test execution wipes sibling tests' entries mid-flight, causing false
// failures. Mirrors the convention documented in ./cache.unit.test.ts.
async function setup(prefix: string,): Promise<string> {
  return mkdtemp(join(scratchRoot(), prefix,),);
}

/**
 Builds a disposable staleness manifest path inside a test's own fixture directory,
 so writes never contend for the workspace's shared manifest and lock.

 @param tempDir - Fixture directory returned by {@link setup}.

 @returns Manifest path under the fixture directory, removed with it by {@link teardown}.

 @example
 ```ts
 await overwrite({ dest, content, manifestPath: manifestPathFor(tempDir,), },);
 ```
 */
function manifestPathFor(tempDir: string,): string {
  return join(tempDir, 'staleness-manifest.json',);
}

/** Removes the temp directory */
async function teardown(tempDir: string,): Promise<void> {
  await rm(tempDir, { recursive: true, force: true, },);
}

await describe({
  name: '',
  children: [
    //region overwrite

    describe({
      name: overwrite.name,
      children: [
        it({
          name: 'creates a new file with the given content',
          fn: async () => {
            const tempDir = await setup('file-enforcer-write-',);
            const dest = join(tempDir, 'new.txt',);
            await overwrite({ dest, content: 'fresh content', manifestPath: manifestPathFor(tempDir,), },);
            expect(await readFile(dest, 'utf8',),).toBe('fresh content',);
            await teardown(tempDir,);
          },
        },),
        it({
          name: 'overwrites an existing file when content differs',
          fn: async () => {
            const tempDir = await setup('file-enforcer-write-',);
            const dest = join(tempDir, 'existing.txt',);
            await writeFile(dest, 'old',);
            await overwrite({ dest, content: 'new', manifestPath: manifestPathFor(tempDir,), },);
            expect(await readFile(dest, 'utf8',),).toBe('new',);
            await teardown(tempDir,);
          },
        },),
        it({
          name: 'skips write when content is identical',
          fn: async () => {
            const tempDir = await setup('file-enforcer-write-',);
            const dest = join(tempDir, 'same.txt',);
            await writeFile(dest, 'unchanged',);
            await overwrite({ dest, content: 'unchanged', manifestPath: manifestPathFor(tempDir,), },);
            /** No writeTimestamp recorded because the actual write was skipped */
            expect(
              writeTimestamps.has(resolve(dest,),),
            ).toBe(false,);
            await teardown(tempDir,);
          },
        },),
        it({
          name: 'still registers dest in writes set even when skipping',
          fn: async () => {
            const tempDir = await setup('file-enforcer-write-',);
            const dest = join(tempDir, 'tracked-skip.txt',);
            await writeFile(dest, 'same',);
            await overwrite({ dest, content: 'same', manifestPath: manifestPathFor(tempDir,), },);
            /** Path should be managed regardless of skip */
            expect(
              writes.has(resolve(dest,),),
            ).toBe(true,);
            await teardown(tempDir,);
          },
        },),
        it({
          name: 'records writeTimestamp only when content actually changes',
          fn: async () => {
            const tempDir = await setup('file-enforcer-write-',);
            const dest = join(tempDir, 'changed.txt',);
            await writeFile(dest, 'old content',);

            await overwrite({ dest, content: 'new content', manifestPath: manifestPathFor(tempDir,), },);
            /** Captured value proves no new write occurred without touching sibling state */
            const firstTimestamp = writeTimestamps.get(resolve(dest,),);
            expect(firstTimestamp,).toBeDefined();

            await overwrite({ dest, content: 'new content', manifestPath: manifestPathFor(tempDir,), },);
            /** Same content now: timestamp must be identical to the first write */
            expect(
              writeTimestamps.get(resolve(dest,),),
            ).toBe(firstTimestamp,);
            await teardown(tempDir,);
          },
        },),
        it({
          name: 'creates parent directories if they do not exist',
          fn: async () => {
            const tempDir = await setup('file-enforcer-write-',);
            const dest = join(tempDir, 'a', 'b', 'c', 'deep.txt',);
            await overwrite({ dest, content: 'deep', manifestPath: manifestPathFor(tempDir,), },);
            expect(await readFile(dest, 'utf8',),).toBe('deep',);
            await teardown(tempDir,);
          },
        },),
        it({
          name: 'handles empty content',
          fn: async () => {
            const tempDir = await setup('file-enforcer-write-',);
            const dest = join(tempDir, 'empty.txt',);
            await overwrite({ dest, content: '', manifestPath: manifestPathFor(tempDir,), },);
            expect(await readFile(dest, 'utf8',),).toBe('',);
            await teardown(tempDir,);
          },
        },),
        it({
          name: 'handles content with special characters',
          fn: async () => {
            const tempDir = await setup('file-enforcer-write-',);
            const content = 'line1\nline2\ttab\r\nwindows\n\u{1F600}emoji';
            const dest = join(tempDir, 'special.txt',);
            await overwrite({ dest, content, manifestPath: manifestPathFor(tempDir,), },);
            expect(await readFile(dest, 'utf8',),).toBe(content,);
            await teardown(tempDir,);
          },
        },),
      ],
    },),

    //endregion overwrite

    //region overwriteIfNotExists

    describe({
      name: overwriteIfNotExists.name,
      children: [
        it({
          name: 'creates file when it does not exist',
          fn: async () => {
            const tempDir = await setup('file-enforcer-wne-',);
            const dest = join(tempDir, 'new.txt',);
            await overwriteIfNotExists({ dest, content: 'created', manifestPath: manifestPathFor(tempDir,), },);
            expect(await readFile(dest, 'utf8',),).toBe('created',);
            await teardown(tempDir,);
          },
        },),
        it({
          name: 'skips writing when file already exists',
          fn: async () => {
            const tempDir = await setup('file-enforcer-wne-',);
            const dest = join(tempDir, 'keep.txt',);
            await writeFile(dest, 'original',);
            await overwriteIfNotExists({ dest, content: 'should-not-appear', manifestPath: manifestPathFor(tempDir,), },);
            expect(await readFile(dest, 'utf8',),).toBe('original',);
            await teardown(tempDir,);
          },
        },),
        it({
          name: 'still registers dest as managed when skipped',
          fn: async () => {
            const tempDir = await setup('file-enforcer-wne-',);
            const dest = join(tempDir, 'skipme.txt',);
            await writeFile(dest, 'existing',);
            await overwriteIfNotExists({ dest, content: 'ignored', manifestPath: manifestPathFor(tempDir,), },);
            expect(
              writes.has(resolve(dest,),),
            ).toBe(true,);
            await teardown(tempDir,);
          },
        },),
        it({
          name: 'creates parent directories for new files',
          fn: async () => {
            const tempDir = await setup('file-enforcer-wne-',);
            const dest = join(tempDir, 'sub', 'dir', 'new.txt',);
            await overwriteIfNotExists({ dest, content: 'nested', manifestPath: manifestPathFor(tempDir,), },);
            expect(await readFile(dest, 'utf8',),).toBe('nested',);
            await teardown(tempDir,);
          },
        },),
      ],
    },),

    //endregion overwriteIfNotExists

    //region overwriteEach

    describe({
      name: overwriteEach.name,
      children: [
        it({
          name: 'writes each file to its mirrored destination',
          fn: async () => {
            const tempDir = await setup('file-enforcer-each-',);
            const srcDir = join(tempDir, 'src',);
            await mkdir(srcDir, { recursive: true, },);

            const files = globResults({
              sourceGlob: join(srcDir, '*.ts',),
              results: [
                { path: join(srcDir, 'a.ts',), content: 'alpha', },
                { path: join(srcDir, 'b.ts',), content: 'beta', },
              ],
            },);

            await overwriteEach({
              destGlob: join(tempDir, 'dest', '*.ts',),
              files,
              manifestPath: manifestPathFor(tempDir,),
            },);

            expect(await readFile(join(tempDir, 'dest', 'a.ts',), 'utf8',),).toBe(
              'alpha',
            );
            expect(await readFile(join(tempDir, 'dest', 'b.ts',), 'utf8',),).toBe(
              'beta',
            );
            await teardown(tempDir,);
          },
        },),
        it({
          name: 'skips files whose destination content is already identical',
          fn: async () => {
            const tempDir = await setup('file-enforcer-each-',);
            const srcDir = join(tempDir, 'src',);
            const destDir = join(tempDir, 'dest',);
            await mkdir(srcDir, { recursive: true, },);
            await mkdir(destDir, { recursive: true, },);

            /** Pre-populate destination with identical content */
            await writeFile(join(destDir, 'same.ts',), 'unchanged',);

            const files = globResults({
              sourceGlob: join(srcDir, '*.ts',),
              results: [
                { path: join(srcDir, 'same.ts',), content: 'unchanged', },
              ],
            },);

            await overwriteEach({
              destGlob: join(destDir, '*.ts',),
              files,
              manifestPath: manifestPathFor(tempDir,),
            },);

            /** No writeTimestamp for this dest because content was identical (size check unsafe under concurrent execution) */
            expect(
              writeTimestamps.has(
                resolve(join(destDir, 'same.ts',),),
              ),
            ).toBe(
              false,
            );
            await teardown(tempDir,);
          },
        },),
        it({
          name: 'handles empty file array without error',
          fn: async () => {
            const tempDir = await setup('file-enforcer-each-',);
            await overwriteEach({
              destGlob: join(tempDir, 'dest', '*.ts',),
              files: globResults({
                sourceGlob: join(tempDir, 'src', '*.ts',),
                results: [],
              },),
              manifestPath: manifestPathFor(tempDir,),
            },);
            /** No tracker entry should reference this test's tempDir (size check unsafe under concurrent execution) */
            const tempPrefix = resolve(tempDir,);
            expect([...writes,].some(function isUnderTemp(p,): boolean {
              return p.startsWith(tempPrefix,);
            },),)
              .toBe(false,);
            await teardown(tempDir,);
          },
        },),
        it({
          name: 'tracks each destination in writes set',
          fn: async () => {
            const tempDir = await setup('file-enforcer-each-',);
            const srcDir = join(tempDir, 'src',);
            await mkdir(srcDir, { recursive: true, },);

            const files = globResults({
              sourceGlob: join(srcDir, '*.ts',),
              results: [
                { path: join(srcDir, 'x.ts',), content: '1', },
                { path: join(srcDir, 'y.ts',), content: '2', },
              ],
            },);

            await overwriteEach({
              destGlob: join(tempDir, 'out', '*.ts',),
              files,
              manifestPath: manifestPathFor(tempDir,),
            },);
            /** Each dest path must be tracked individually (size check unsafe under concurrent execution) */
            expect(
              writes.has(
                resolve(join(tempDir, 'out', 'x.ts',),),
              ),
            ).toBe(true,);
            expect(
              writes.has(
                resolve(join(tempDir, 'out', 'y.ts',),),
              ),
            ).toBe(true,);
            await teardown(tempDir,);
          },
        },),
      ],
    },),
    //endregion overwriteEach
  ],
},);
