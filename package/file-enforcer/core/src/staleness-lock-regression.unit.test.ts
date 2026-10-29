import {
  mkdir,
  readFile,
  utimes,
} from 'node:fs/promises';
import { join, } from 'node:path';

import spawn from 'nano-spawn';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  setupStalenessLockFixture,
  STALE_LOCK_DATE,
  teardownStalenessLockFixture,
  writeOverwriteConfig,
} from './staleness-lock-regression-fixture.ts';

await describe({
  name: 'file-enforcer stale-lock regressions',
  concurrency: 1,
  children: [
    it({
      name: 'recovers abandoned manifest lock directories before writing manifest',
      timeout: 10_000,
      fn: async function recoversAbandonedManifestLock(): Promise<void> {
        const tempDir = await setupStalenessLockFixture();
        await using _cleanup = {
          [Symbol.asyncDispose](): Promise<void> {
            return teardownStalenessLockFixture(tempDir,);
          },
        };
        const manifestPath = join(
          tempDir,
          'manifest.json',
        );
        const lockPath = `${manifestPath}.lock`;
        const configPath = join(
          tempDir,
          'write-config.ts',
        );
        const outputPath = join(
          tempDir,
          'output.txt',
        );
        await mkdir(
          lockPath,
          { recursive: true, },
        );
        await utimes(
          lockPath,
          STALE_LOCK_DATE,
          STALE_LOCK_DATE,
        );
        await writeOverwriteConfig({
          configPath,
          manifestPath,
          outputPath,
          content: 'alpha',
        },);

        await spawn(
          'node',
          [configPath,],
          { cwd: tempDir, },
        );

        const manifest = JSON.parse(await readFile(manifestPath, 'utf8',),) as {
          readonly entries?: Record<string, unknown>;
        };
        expect(Object.keys(manifest.entries ?? {},),).toEqual([
          `single:${outputPath}`,
        ],);
      },
    },),
  ],
},);
