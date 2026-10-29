/**
 Tests for matrix test file discovery.
 
 Creates a temporary directory tree and verifies that
 {@link discoverTestFiles} finds the right files and
 skips excluded directories.
 
 @module
 */

import {
  mkdir,
  mkdtemp,
  rm,
  writeFile,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { discoverTestFiles, } from '../dist/final/node/index.mjs';

/** Unique prefix avoids collisions with parallel test runs. */
const TEST_DIR_PREFIX = 'matrix-discover-test';

/**
 Disposable temporary directory that cleans itself up via `Symbol.asyncDispose`.
 */
type TempDir = {
  /** Absolute path to the temp directory. */
  readonly path: string;
  [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Creates a disposable temporary directory for discovery tests.
 Automatically removed when the `await using` binding goes out of scope.
 
 @param id - unique suffix for this test's directory
 
 @returns disposable temp directory handle
 */
async function createTempDir(id: string,): Promise<TempDir> {
  const path = await mkdtemp(
    join(scratchRoot(), `${TEST_DIR_PREFIX}-${id}-`,),
  );

  return {
    path,
    [Symbol.asyncDispose]: async function cleanup() {
      await rm(path, { recursive: true, force: true, },);
    },
  };
}

await describe({
  name: 'discover',
  children: [
    it({
      name: 'finds .unit.matrix.test.ts files',
      fn: async () => {
        await using dir = await createTempDir('finds',);

        await writeFile(
          join(dir.path, 'foo.unit.matrix.test.ts',),
          '// test',
        );
        await writeFile(
          join(dir.path, 'bar.unit.matrix.test.ts',),
          '// test',
        );

        const files = await discoverTestFiles(dir.path,);
        expect(files.length,).toBe(2,);
        expect(files[0],).toContain('bar.unit.matrix.test.ts',);
        expect(files[1],).toContain('foo.unit.matrix.test.ts',);
      },
    },),

    it({
      name: 'skips node_modules directories',
      fn: async () => {
        await using dir = await createTempDir('skip-nm',);

        await mkdir(join(dir.path, 'node_modules', 'pkg',), { recursive: true, },);
        await writeFile(
          join(dir.path, 'node_modules', 'pkg', 'hidden.unit.matrix.test.ts',),
          '// test',
        );
        await writeFile(
          join(dir.path, 'visible.unit.matrix.test.ts',),
          '// test',
        );

        const files = await discoverTestFiles(dir.path,);
        expect(files.length,).toBe(1,);
        expect(files[0],).toContain('visible',);
      },
    },),

    it({
      name: 'skips dist directories',
      fn: async () => {
        await using dir = await createTempDir('skip-dist',);

        await mkdir(join(dir.path, 'dist', 'final',), { recursive: true, },);
        await writeFile(
          join(dir.path, 'dist', 'final', 'hidden.unit.matrix.test.ts',),
          '// test',
        );
        await writeFile(
          join(dir.path, 'visible.unit.matrix.test.ts',),
          '// test',
        );

        const files = await discoverTestFiles(dir.path,);
        expect(files.length,).toBe(1,);
        expect(files[0],).toContain('visible',);
      },
    },),

    it({
      name: 'finds files in subdirectories recursively',
      fn: async () => {
        await using dir = await createTempDir('recursive',);

        await mkdir(join(dir.path, 'src', 'nested',), { recursive: true, },);
        await writeFile(
          join(dir.path, 'src', 'nested', 'deep.unit.matrix.test.ts',),
          '// test',
        );

        const files = await discoverTestFiles(dir.path,);
        expect(files.length,).toBe(1,);
        expect(files[0],).toContain('deep.unit.matrix.test.ts',);
      },
    },),

    it({
      name: 'throws when no test files found',
      fn: async () => {
        await using dir = await createTempDir('empty',);

        /** Write a non-matching file so the directory is not empty. */
        await writeFile(
          join(dir.path, 'foo.test.ts',),
          '// not matrix pattern',
        );

        let caught: unknown;
        try {
          await discoverTestFiles(dir.path,);
        }
        catch (error: unknown) {
          caught = error;
        }

        expect(caught,).toBeInstanceOf(Error,);
        expect((caught as Error).message,).toContain('No matrix test files',);
      },
    },),

    it({
      name: 'returns sorted absolute paths',
      fn: async () => {
        await using dir = await createTempDir('sorted',);

        await writeFile(
          join(dir.path, 'z.unit.matrix.test.ts',),
          '// test',
        );
        await writeFile(
          join(dir.path, 'a.unit.matrix.test.ts',),
          '// test',
        );
        await writeFile(
          join(dir.path, 'm.unit.matrix.test.ts',),
          '// test',
        );

        const files = await discoverTestFiles(dir.path,);
        expect(files.length,).toBe(3,);

        /** Paths should be absolute. */
        for (const file of files)
          expect(file.startsWith('/',),).toBe(true,);

        /** Should be sorted lexicographically. */
        expect(files[0],).toContain('a.unit.matrix.test.ts',);
        expect(files[1],).toContain('m.unit.matrix.test.ts',);
        expect(files[2],).toContain('z.unit.matrix.test.ts',);
      },
    },),

    it({
      name: 'ignores non-.ts files with similar names',
      fn: async () => {
        await using dir = await createTempDir('non-ts',);

        await writeFile(
          join(dir.path, 'foo.unit.matrix.test.js',),
          '// wrong extension',
        );
        await writeFile(
          join(dir.path, 'bar.unit.test.ts',),
          '// not matrix pattern',
        );
        await writeFile(
          join(dir.path, 'baz.unit.matrix.test.ts',),
          '// match',
        );

        const files = await discoverTestFiles(dir.path,);
        expect(files.length,).toBe(1,);
        expect(files[0],).toContain('baz',);
      },
    },),
  ],
},);
