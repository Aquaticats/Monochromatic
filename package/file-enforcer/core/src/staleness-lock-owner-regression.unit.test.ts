import {
  access,
  mkdir,
  readFile,
  utimes,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import spawn from 'nano-spawn';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { lockOwnerState, } from './io/staleness-manifest-lock-owner.ts';
import {
  publishLockOwnerPublication,
  stageLockOwnerPublication,
} from './io/staleness-manifest-lock-owner-publish.ts';
import {
  DEAD_OWNER_PROCESS_ID,
  runConfigExpectingError,
  setupStalenessLockFixture,
  STALE_LOCK_DATE,
  teardownStalenessLockFixture,
  writeLockOwnerFixture,
  writeOverwriteConfig,
} from './staleness-lock-regression-fixture.ts';

await describe({
  name: 'file-enforcer stale-lock owner regressions',
  concurrency: 1,
  children: [
    it({
      name: 'keeps contenders from observing owner metadata before atomic publication',
      fn: async function hidesStagedOwnerMetadata(): Promise<void> {
        const tempDir = await setupStalenessLockFixture();
        await using _cleanup = {
          [Symbol.asyncDispose](): Promise<void> {
            return teardownStalenessLockFixture(tempDir,);
          },
        };
        const lockPath = join(
          tempDir,
          'atomic-publication.lock',
        );
        await mkdir(lockPath,);
        /**
         Complete owner metadata staged before publication.
         */
        const ownerText = `${JSON.stringify(
          {
            pid: process.pid,
            createdAt: new Date().toISOString(),
          },
          null,
          2,
        )}\n`;

        await stageLockOwnerPublication({
          lockPath,
          ownerText,
        },);

        /**
         Parsed complete owner expected at each publication stage.
         */
        const expectedOwner: unknown = JSON.parse(ownerText,);
        /**
         Private staged owner visible only by its private path.
         */
        const stagedOwner: unknown = JSON.parse(await readFile(
          join(lockPath, 'owner.pending.json',),
          'utf8',
        ),);
        expect(await lockOwnerState(lockPath,),).toBe('absent',);
        expect(stagedOwner,).toEqual(expectedOwner,);

        await publishLockOwnerPublication(lockPath,);

        /**
         Owner visible to contenders only after atomic rename.
         */
        const publishedOwner: unknown = JSON.parse(await readFile(
          join(lockPath, 'owner.json',),
          'utf8',
        ),);
        expect(await lockOwnerState(lockPath,),).toBe('live',);
        expect(publishedOwner,).toEqual(expectedOwner,);
      },
    },),

    it({
      name: 'recovers fresh manifest lock directories owned by dead process',
      timeout: 10_000,
      fn: async function recoversDeadOwnerManifestLock(): Promise<void> {
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
          'dead-owner-config.ts',
        );
        const outputPath = join(
          tempDir,
          'dead-owner-output.txt',
        );
        await mkdir(
          lockPath,
          { recursive: true, },
        );
        await writeLockOwnerFixture({
          lockPath,
          pid: DEAD_OWNER_PROCESS_ID,
        },);
        await writeOverwriteConfig({
          configPath,
          manifestPath,
          outputPath,
          content: 'bravo',
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

    it({
      name: 'does not reclaim old manifest lock directories owned by live process',
      timeout: 10_000,
      fn: async function doesNotReclaimLiveOwnerManifestLock(): Promise<void> {
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
          'live-owner-config.ts',
        );
        const outputPath = join(
          tempDir,
          'live-owner-output.txt',
        );
        await mkdir(
          lockPath,
          { recursive: true, },
        );
        await writeLockOwnerFixture({
          lockPath,
          pid: process.pid,
        },);
        await utimes(
          lockPath,
          STALE_LOCK_DATE,
          STALE_LOCK_DATE,
        );
        await writeOverwriteConfig({
          configPath,
          manifestPath,
          outputPath,
          content: 'charlie',
        },);

        const spawnError = await runConfigExpectingError({
          configPath,
          cwd: tempDir,
        },);

        expect(spawnError,).toBeInstanceOf(Error,);
        await access(lockPath,);
      },
    },),

    it({
      name: 'does not reclaim old manifest lock directories with malformed owner metadata',
      timeout: 10_000,
      fn: async function doesNotReclaimMalformedOwnerManifestLock(): Promise<void> {
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
          'malformed-owner-config.ts',
        );
        const outputPath = join(
          tempDir,
          'malformed-owner-output.txt',
        );
        await mkdir(
          lockPath,
          { recursive: true, },
        );
        await writeFile(
          join(
            lockPath,
            'owner.json',
          ),
          '{ malformed owner json',
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
          content: 'delta',
        },);

        const spawnError = await runConfigExpectingError({
          configPath,
          cwd: tempDir,
        },);

        expect(spawnError,).toBeInstanceOf(Error,);
        await access(lockPath,);
      },
    },),
  ],
},);
