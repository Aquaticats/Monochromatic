import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
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
  overwriteTomlKey,
  reset,
  resetWriteTimestamps,
  writes,
  writeTimestamps,
} from '../../dist/final/node/index.mjs';

/** Creates a fresh temp directory and resets tracker state */
async function setup(prefix: string,): Promise<string> {
  const tempDir = await mkdtemp(join(scratchRoot(), prefix,),);
  reset();
  resetWriteTimestamps();
  return tempDir;
}

/**
 Builds a disposable staleness manifest path inside a test's own fixture directory,
 so writes never contend for the workspace's shared manifest and lock.

 @param tempDir - Fixture directory returned by {@link setup}.

 @returns Manifest path under the fixture directory, removed with it by {@link teardown}.

 @example
 ```ts
 await overwriteTomlKey({ dest, path, value, manifestPath: manifestPathFor(tempDir,), },);
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
  name: overwriteTomlKey.name,
  children: [
    it({
      name: 'updates an existing scalar and preserves comments',
      fn: async () => {
        const tempDir = await setup('file-enforcer-toml-',);
        const dest = join(tempDir, 'pkg.toml',);
        /** Original TOML with comments and a value to update */
        const original = '# Package metadata\nname = "old"\n# Tail comment\n';
        await writeFile(dest, original,);
        await overwriteTomlKey({
          dest,
          path: ['name',],
          value: 'new',
          manifestPath: manifestPathFor(tempDir,),
        },);
        const after = await readFile(dest, 'utf8',);
        expect(after.includes('name = "new"',),).toBe(true,);
        expect(after.includes('# Package metadata',),).toBe(true,);
        expect(after.includes('# Tail comment',),).toBe(true,);
        await teardown(tempDir,);
      },
    },),
    it({
      name: 'updates a value nested under a section header',
      fn: async () => {
        const tempDir = await setup('file-enforcer-toml-',);
        const dest = join(tempDir, 'config.toml',);
        await writeFile(dest, '[settings]\ntheme = "dark"\n',);
        await overwriteTomlKey({
          dest,
          path: ['settings', 'theme',],
          value: 'light',
          manifestPath: manifestPathFor(tempDir,),
        },);
        const after = await readFile(dest, 'utf8',);
        expect(after.includes('theme = "light"',),).toBe(true,);
        expect(after.includes('[settings]',),).toBe(true,);
        await teardown(tempDir,);
      },
    },),
    it({
      name: 'creates a missing top-level key',
      fn: async () => {
        const tempDir = await setup('file-enforcer-toml-',);
        const dest = join(tempDir, 'add.toml',);
        await writeFile(dest, 'existing = "yes"\n',);
        await overwriteTomlKey({
          dest,
          path: ['added',],
          value: 'value',
          manifestPath: manifestPathFor(tempDir,),
        },);
        const after = await readFile(dest, 'utf8',);
        expect(after.includes('added = "value"',),).toBe(true,);
        expect(after.includes('existing = "yes"',),).toBe(true,);
        await teardown(tempDir,);
      },
    },),
    it({
      name: 'updates an array-of-tables element',
      fn: async () => {
        const tempDir = await setup('file-enforcer-toml-',);
        const dest = join(tempDir, 'aot.toml',);
        await writeFile(
          dest,
          '[[fruits]]\nname = "apple"\n\n[[fruits]]\nname = "orange"\n',
        );
        await overwriteTomlKey({
          dest,
          path: ['fruits', 0, 'name',],
          value: 'banana',
          manifestPath: manifestPathFor(tempDir,),
        },);
        const after = await readFile(dest, 'utf8',);
        expect(after.includes('name = "banana"',),).toBe(true,);
        expect(after.includes('name = "orange"',),).toBe(true,);
        await teardown(tempDir,);
      },
    },),
    it({
      name: 'skips writeTimestamp when serialised text matches existing content',
      fn: async () => {
        const tempDir = await setup('file-enforcer-toml-',);
        const dest = join(tempDir, 'unchanged.toml',);
        /** Splice mode: writing the same value back yields identical bytes */
        const source = 'name = "Alice"\n';
        await writeFile(dest, source,);
        await overwriteTomlKey({
          dest,
          path: ['name',],
          value: 'Alice',
          manifestPath: manifestPathFor(tempDir,),
        },);
        /** writeIfChanged should detect no diff and skip the write */
        expect(
          writeTimestamps.has(resolve(dest,),),
        ).toBe(false,);
        await teardown(tempDir,);
      },
    },),
    it({
      name: 'registers dest in writes set even when the actual write is skipped',
      fn: async () => {
        const tempDir = await setup('file-enforcer-toml-',);
        const dest = join(tempDir, 'tracked.toml',);
        await writeFile(dest, 'name = "Alice"\n',);
        await overwriteTomlKey({
          dest,
          path: ['name',],
          value: 'Alice',
          manifestPath: manifestPathFor(tempDir,),
        },);
        expect(
          writes.has(resolve(dest,),),
        ).toBe(true,);
        await teardown(tempDir,);
      },
    },),
    it({
      name: 'throws when the destination does not exist',
      fn: async () => {
        const tempDir = await setup('file-enforcer-toml-',);
        const dest = join(tempDir, 'missing.toml',);
        await expect(overwriteTomlKey({
          dest,
          path: ['name',],
          value: 'value',
          manifestPath: manifestPathFor(tempDir,),
        },),)
          .rejects
          .toThrow();
        await teardown(tempDir,);
      },
    },),
  ],
},);
