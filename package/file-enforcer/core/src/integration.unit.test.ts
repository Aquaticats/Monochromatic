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
import { join, } from 'node:path';
import spawn from 'nano-spawn';
import {
  reads,
  reset,
  resetWriteTimestamps,
  writes,
} from '../dist/final/node/index.mjs';

//region integration tests with real config files

await describe({
  name: 'integration: config execution',
  children: [
    it({
      name: 'config file that copies one file to another',
      fn: async () => {
        const tempDir = await mkdtemp(join(scratchRoot(), 'file-enforcer-integ-',),);
        reset();
        resetWriteTimestamps();
        await using _cleanup = {
          [Symbol.asyncDispose](): Promise<void> {
            return rm(tempDir, { recursive: true, force: true, },);
          },
        };
        /** Source file with known content */
        await writeFile(join(tempDir, 'source.md',), '# Source Content',);

        /** Config that imports from the package and runs a simple copy */
        const configContent = `
            import { cat, overwrite } from '${join(import.meta.dirname, '..', 'dist', 'final', 'node', 'index.mjs',)}';
            await overwrite({
              dest: '${join(tempDir, 'dest.md',)}',
              content: await cat(['${join(tempDir, 'source.md',)}']),
            });
          `;
        /** Write config to temp dir so it can be imported */
        const configPath = join(tempDir, 'test.config.ts',);
        await writeFile(configPath, configContent,);

        // Execute the config by importing it
        await import(configPath);

        /** Destination should have the source content */
        expect(await readFile(join(tempDir, 'dest.md',), 'utf8',),).toBe(
          '# Source Content',
        );
        /** Tracker should have recorded the read */
        expect(reads.size,).toBeGreaterThan(0,);
      },
    },),
    it({
      name: 'config file that concatenates multiple sources',
      fn: async () => {
        const tempDir = await mkdtemp(join(scratchRoot(), 'file-enforcer-integ-',),);
        reset();
        resetWriteTimestamps();
        await using _cleanup = {
          [Symbol.asyncDispose](): Promise<void> {
            return rm(tempDir, { recursive: true, force: true, },);
          },
        };
        await writeFile(join(tempDir, 'a.txt',), 'aaa',);
        await writeFile(join(tempDir, 'b.txt',), 'bbb',);

        /** Config concatenating two files */
        const configContent = `
            import { cat, overwrite } from '${join(import.meta.dirname, '..', 'dist', 'final', 'node', 'index.mjs',)}';
            await overwrite({
              dest: '${join(tempDir, 'combined.txt',)}',
              content: await cat(['${join(tempDir, 'a.txt',)}', '${
          join(tempDir, 'b.txt',)
        }']),
            });
          `;
        const configPath = join(tempDir, 'concat.config.ts',);
        await writeFile(configPath, configContent,);

        await import(configPath);

        expect(await readFile(join(tempDir, 'combined.txt',), 'utf8',),).toBe(
          'aaa\nbbb',
        );
      },
    },),
    it({
      name: 'config file using dedup transform',
      fn: async () => {
        const tempDir = await mkdtemp(join(scratchRoot(), 'file-enforcer-integ-',),);
        reset();
        resetWriteTimestamps();
        await using _cleanup = {
          [Symbol.asyncDispose](): Promise<void> {
            return rm(tempDir, { recursive: true, force: true, },);
          },
        };
        await writeFile(join(tempDir, 'dupes.txt',), 'line1\nline2\nline1\nline3',);

        /** Config that reads, deduplicates, and writes */
        const configContent = `
            import { cat, dedup, overwrite } from '${
          join(import.meta.dirname, '..', 'dist', 'final', 'node', 'index.mjs',)
        }';
            await overwrite({
              dest: '${join(tempDir, 'unique.txt',)}',
              content: dedup(await cat(['${join(tempDir, 'dupes.txt',)}'])),
            });
          `;
        const configPath = join(tempDir, 'dedup.config.ts',);
        await writeFile(configPath, configContent,);

        await import(configPath);

        expect(await readFile(join(tempDir, 'unique.txt',), 'utf8',),).toBe(
          'line1\nline2\nline3',
        );
      },
    },),
    it({
      name: 'config file using getJsonProperty transform',
      fn: async () => {
        const tempDir = await mkdtemp(join(scratchRoot(), 'file-enforcer-integ-',),);
        reset();
        resetWriteTimestamps();
        await using _cleanup = {
          [Symbol.asyncDispose](): Promise<void> {
            return rm(tempDir, { recursive: true, force: true, },);
          },
        };
        /** JSON source file */
        const jsonContent = JSON.stringify({ config: { name: 'test-project', }, },);
        await writeFile(join(tempDir, 'data.json',), jsonContent,);

        /** Config that extracts a property from JSON */
        const configContent = `
            import { cat, getJsonProperty, overwrite } from '${
          join(import.meta.dirname, '..', 'dist', 'final', 'node', 'index.mjs',)
        }';
            await overwrite({
              dest: '${join(tempDir, 'name.txt',)}',
              content: getJsonProperty({ path: ['config', 'name'], content: await cat(['${
          join(tempDir, 'data.json',)
        }']) }),
            });
          `;
        const configPath = join(tempDir, 'prop.config.ts',);
        await writeFile(configPath, configContent,);

        await import(configPath);

        expect(await readFile(join(tempDir, 'name.txt',), 'utf8',),).toBe(
          'test-project',
        );
      },
    },),
    it({
      name: 'config file tracks both reads and writes',
      fn: async () => {
        const tempDir = await mkdtemp(join(scratchRoot(), 'file-enforcer-integ-',),);
        reset();
        resetWriteTimestamps();
        await using _cleanup = {
          [Symbol.asyncDispose](): Promise<void> {
            return rm(tempDir, { recursive: true, force: true, },);
          },
        };
        await writeFile(join(tempDir, 'input.md',), 'tracked',);

        /** Config with a simple copy to verify tracker state */
        const configContent = `
            import { cat, overwrite } from '${join(import.meta.dirname, '..', 'dist', 'final', 'node', 'index.mjs',)}';
            await overwrite({
              dest: '${join(tempDir, 'output.md',)}',
              content: await cat(['${join(tempDir, 'input.md',)}']),
            });
          `;
        const configPath = join(tempDir, 'tracked.config.ts',);
        await writeFile(configPath, configContent,);

        await import(configPath);

        /** Both sets should be populated */
        expect(reads.size,).toBeGreaterThan(0,);
        expect(writes.size,).toBeGreaterThan(0,);
      },
    },),
    it({
      name: 'config using addWatchedPaths for exec dependencies',
      fn: async () => {
        const tempDir = await mkdtemp(join(scratchRoot(), 'file-enforcer-integ-',),);
        reset();
        resetWriteTimestamps();
        await using _cleanup = {
          [Symbol.asyncDispose](): Promise<void> {
            return rm(tempDir, { recursive: true, force: true, },);
          },
        };
        await writeFile(join(tempDir, 'dep.json',), '{"version":"1.0"}',);

        /** Config that registers a manual dependency via addWatchedPaths */
        const configContent = `
            import { addWatchedPaths, exec, overwrite } from '${
          join(import.meta.dirname, '..', 'dist', 'final', 'node', 'index.mjs',)
        }';
            addWatchedPaths(['${join(tempDir, 'dep.json',)}']);
            const result = await exec({ cmd: 'echo', args: ['from-exec'] });
            await overwrite({
              dest: '${join(tempDir, 'exec-out.txt',)}',
              content: result.trim(),
            });
          `;
        const configPath = join(tempDir, 'escape.config.ts',);
        await writeFile(configPath, configContent,);

        await import(configPath);

        /** The manually added dependency should appear in reads */
        /** resolve is applied inside addWatchedPaths */
        const hasDepTracked = [...reads,].some(readPath =>
          readPath.endsWith('dep.json',)
        );
        expect(hasDepTracked,).toBe(true,);
      },
    },),
    it({
      name: 'config with overwriteIfNotExists skips existing files',
      fn: async () => {
        const tempDir = await mkdtemp(join(scratchRoot(), 'file-enforcer-integ-',),);
        reset();
        resetWriteTimestamps();
        await using _cleanup = {
          [Symbol.asyncDispose](): Promise<void> {
            return rm(tempDir, { recursive: true, force: true, },);
          },
        };
        /** Pre-existing file that should not be overwritten */
        await writeFile(join(tempDir, 'keep.txt',), 'original',);
        await writeFile(join(tempDir, 'src.txt',), 'replacement',);

        /** Config using overwriteIfNotExists */
        const configContent = `
            import { cat, overwriteIfNotExists } from '${
          join(import.meta.dirname, '..', 'dist', 'final', 'node', 'index.mjs',)
        }';
            await overwriteIfNotExists({
              dest: '${join(tempDir, 'keep.txt',)}',
              content: await cat(['${join(tempDir, 'src.txt',)}']),
            });
          `;
        const configPath = join(tempDir, 'skip.config.ts',);
        await writeFile(configPath, configContent,);

        await import(configPath);

        /** Original content should be preserved */
        expect(await readFile(join(tempDir, 'keep.txt',), 'utf8',),).toBe('original',);
      },
    },),
    it({
      name: 'direct file-enforcer config reruns builders when manifest is fresh',
      fn: async () => {
        const tempDir = await mkdtemp(join(scratchRoot(), 'file-enforcer-integ-',),);
        await using _cleanup = {
          [Symbol.asyncDispose](): Promise<void> {
            return rm(tempDir, { recursive: true, force: true, },);
          },
        };
        const markerPath = join(tempDir, 'runs.txt',);
        const destPath = join(tempDir, 'output.txt',);
        const configPath = join(tempDir, 'file-enforcer.config.ts',);
        const configContent = `
            import { appendFile } from 'node:fs/promises';
            import { overwrite } from '${join(import.meta.dirname, '..', 'dist', 'final', 'node', 'index.mjs',)}';
            await appendFile('${markerPath}', 'ran\\n');
            await overwrite({
              dest: '${destPath}',
              content: 'stable',
            });
          `;
        await writeFile(configPath, configContent,);

        await spawn('node', [configPath,], { cwd: tempDir, },);
        await spawn('node', [configPath,], { cwd: tempDir, },);

        expect(await readFile(markerPath, 'utf8',),).toBe('ran\nran\n',);
        expect(await readFile(destPath, 'utf8',),).toBe('stable',);

        await writeFile(destPath, 'external edit',);
        await spawn('node', [configPath,], { cwd: tempDir, },);

        expect(await readFile(markerPath, 'utf8',),).toBe('ran\nran\nran\n',);
        expect(await readFile(destPath, 'utf8',),).toBe('stable',);
      },
    },),
    it({
      name: 'config with glob-based overwriteEach mirrors files',
      fn: async () => {
        const tempDir = await mkdtemp(join(scratchRoot(), 'file-enforcer-integ-',),);
        reset();
        resetWriteTimestamps();
        await using _cleanup = {
          [Symbol.asyncDispose](): Promise<void> {
            return rm(tempDir, { recursive: true, force: true, },);
          },
        };
        /** Source directory with files to mirror */
        const srcDir = join(tempDir, 'src',);
        await mkdir(srcDir, { recursive: true, },);
        await writeFile(join(srcDir, 'a.ts',), 'alpha',);
        await writeFile(join(srcDir, 'b.ts',), 'beta',);

        /** Config using cat(glob) -> overwriteEach */
        const configContent = `
            import { cat, overwriteEach } from '${
          join(import.meta.dirname, '..', 'dist', 'final', 'node', 'index.mjs',)
        }';
            const files = await cat('${join(srcDir, '*.ts',)}');
            await overwriteEach({
              destGlob: '${join(tempDir, 'out', '*.ts',)}',
              files,
            });
          `;
        const configPath = join(tempDir, 'mirror.config.ts',);
        await writeFile(configPath, configContent,);

        await import(configPath);

        expect(await readFile(join(tempDir, 'out', 'a.ts',), 'utf8',),).toBe('alpha',);
        expect(await readFile(join(tempDir, 'out', 'b.ts',), 'utf8',),).toBe('beta',);
      },
    },),
  ],
},);

//endregion integration tests with real config files
