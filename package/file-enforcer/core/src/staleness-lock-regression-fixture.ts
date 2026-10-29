import {
  mkdtemp,
  rm,
  writeFile,
} from 'node:fs/promises';
import { tmpdir, } from 'node:os';
import { join, } from 'node:path';
import { pathToFileURL, } from 'node:url';

import spawn from 'nano-spawn';

//region Stale lock fixture constants

/**
 Built package entry imported by spawned config fixtures.
 */
const DIST_INDEX_URL = pathToFileURL(join(
  import.meta.dirname,
  '../dist/final/node/index.mjs',
),)
  .href;

/**
 Date old enough to be considered abandoned stale-lock metadata.
 */
export const STALE_LOCK_DATE: Date = new Date(0,);

/**
 Process id used to represent definitely-dead lock owner in fixtures.
 */
export const DEAD_OWNER_PROCESS_ID: number = 999_999_999;

//endregion Stale lock fixture constants

//region Temp directory helpers

/**
 Creates an isolated temp directory for stale-lock regression tests.
 
 @returns Temp directory path.
 
 @example
 ```ts
 const tempDir = await setupStalenessLockFixture();
 ```
 */
export async function setupStalenessLockFixture(): Promise<string> {
  return await mkdtemp(join(
    tmpdir(),
    'file-enforcer-staleness-lock-regression-',
  ),);
}

/**
 Removes an isolated temp directory.
 
 @param tempDir - Directory returned by {@link setupStalenessLockFixture}.
 
 @example
 ```ts
 await teardownStalenessLockFixture(tempDir);
 ```
 */
export async function teardownStalenessLockFixture(tempDir: string,): Promise<void> {
  await rm(
    tempDir,
    {
      recursive: true,
      force: true,
    },
  );
}

//endregion Temp directory helpers

//region Generated config helpers

/**
 Returns JSON text for a generated TypeScript string literal.
 
 @param value - Value to quote.
 
 @returns JavaScript string literal source.
 
 @example
 ```ts
 const literal = jsString('/tmp/path');
 ```
 */
function jsString(value: string,): string {
  return JSON.stringify(value,);
}

/**
 Writes lock owner fixture metadata.
 
 @param lockPath - Lock directory path.
 
 @param pid - Owner process id to record.
 
 @example
 ```ts
 await writeLockOwnerFixture({ lockPath, pid: process.pid });
 ```
 */
export async function writeLockOwnerFixture(
  {
    lockPath,
    pid,
  }: {
    readonly lockPath: string;
    readonly pid: number;
  },
): Promise<void> {
  /**
   Serialized owner metadata fixture.
   */
  const ownerMetadata = `${JSON.stringify(
    {
      pid,
      createdAt: new Date().toISOString(),
    },
    null,
    2,
  )}\n`;
  await writeFile(
    join(
      lockPath,
      'owner.json',
    ),
    ownerMetadata,
  );
}

/**
 Writes generated config that performs one managed overwrite.
 
 @param configPath - Config file path to create.
 
 @param manifestPath - Manifest path passed to {@link overwrite}.
 
 @param outputPath - Destination file path passed to {@link overwrite}.
 
 @param content - Content written by {@link overwrite}.
 
 @example
 ```ts
 await writeOverwriteConfig({ configPath, manifestPath, outputPath, content: 'alpha' });
 ```
 */
export async function writeOverwriteConfig(
  {
    configPath,
    manifestPath,
    outputPath,
    content,
  }: {
    readonly configPath: string;
    readonly content: string;
    readonly manifestPath: string;
    readonly outputPath: string;
  },
): Promise<void> {
  await writeFile(
    configPath,
    `
import { overwrite } from ${jsString(DIST_INDEX_URL,)};

await overwrite({
  dest: ${jsString(outputPath,)},
  content: ${jsString(content,)},
  manifestPath: ${jsString(manifestPath,)},
});
`,
  );
}

//endregion Generated config helpers

//region Spawn helpers

/**
 Runs a generated config expected to fail while flushing manifest.
 
 @param configPath - Config file path to execute.
 
 @param cwd - Current working directory for spawned config.
 
 @returns Error thrown by spawned process.
 
 @throws When spawned process unexpectedly succeeds.
 
 @example
 ```ts
 const error = await runConfigExpectingError({ configPath, cwd: tempDir });
 ```
 */
export async function runConfigExpectingError(
  {
    configPath,
    cwd,
  }: {
    readonly configPath: string;
    readonly cwd: string;
  },
): Promise<unknown> {
  try {
    await spawn(
      'node',
      [configPath,],
      { cwd, },
    );
  }
  catch (spawnError: unknown) {
    return spawnError;
  }

  throw new Error('Config unexpectedly succeeded with a live-owner manifest lock',);
}

//endregion Spawn helpers
