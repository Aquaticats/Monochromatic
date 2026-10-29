import { randomUUID, } from 'node:crypto';
import {
  mkdtemp,
  readFile,
  readdir,
  rm,
  writeFile,
} from 'node:fs/promises';
import { tmpdir, } from 'node:os';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type AtomicTempFileWriter,
  writeMergedManifest,
} from '../dist/final/node/index.mjs';
import { DEAD_OWNER_PROCESS_ID, } from './staleness-lock-regression-fixture.ts';

//region Orphaned manifest temp file fixture helpers

/**
 File name of the manifest inside each throwaway fixture directory.
 */
const MANIFEST_FILE_NAME = 'staleness-manifest.json';

/**
 Empty manifest persisted by every write in these tests.
 */
const EMPTY_MANIFEST = {
  version: 1,
  entries: {},
} as const;

/**
 Creates an isolated temp directory for orphaned temp file tests.

 @returns Temp directory path.

 @example
 ```ts
 const tempDir = await setup();
 ```
 */
async function setup(): Promise<string> {
  return await mkdtemp(join(
    tmpdir(),
    'file-enforcer-manifest-orphan-temp-',
  ),);
}

/**
 Removes an isolated temp directory.

 @param tempDir - Directory returned by {@link setup}.

 @example
 ```ts
 await teardown(tempDir);
 ```
 */
async function teardown(tempDir: string,): Promise<void> {
  await rm(
    tempDir,
    {
      recursive: true,
      force: true,
    },
  );
}

/**
 Returns the entry name a manifest writer of the given process would leave behind.

 @param manifestFileName - File name of the manifest.

 @param processIdText - Writer process id text, kept as text to allow malformed ids.

 @returns Temp file name of the form `<manifest>.<pid>.<uuid>.tmp`.

 @example
 ```ts
 const name = tempFileName({ manifestFileName: 'm.json', processIdText: '42' });
 ```
 */
function tempFileName(
  {
    manifestFileName,
    processIdText,
  }: {
    readonly manifestFileName: string;
    readonly processIdText: string;
  },
): string {
  return `${manifestFileName}.${processIdText}.${randomUUID()}.tmp`;
}

/**
 Writes a partial temp file the way a writer killed before its rename leaves one.

 @param directory - Directory holding the manifest.

 @param entryName - Temp file name to create.

 @returns Absolute path of the created file.

 @example
 ```ts
 const orphanPath = await plantTempFile({ directory: tempDir, entryName });
 ```
 */
async function plantTempFile(
  {
    directory,
    entryName,
  }: {
    readonly directory: string;
    readonly entryName: string;
  },
): Promise<string> {
  /**
   Path of the planted file.
   */
  const plantedPath = join(
    directory,
    entryName,
  );
  await writeFile(
    plantedPath,
    '{"version":1,"entries":{"truncated',
  );
  return plantedPath;
}

/**
 Lists directory entries in a stable order.

 @param directory - Directory to list.

 @returns Sorted entry names.

 @example
 ```ts
 const names = await sortedEntries(tempDir);
 ```
 */
async function sortedEntries(directory: string,): Promise<readonly string[]> {
  return (await readdir(directory,)).toSorted();
}

/**
 Captures the error thrown by a manifest write that is expected to fail.

 @param manifestPath - Manifest path to write.

 @param tempFileWriter - Fault-injection writer expected to throw.

 @returns Error thrown by the write.

 @throws When the write unexpectedly succeeds.

 @example
 ```ts
 const error = await captureManifestWriteError({ manifestPath, tempFileWriter });
 ```
 */
async function captureManifestWriteError(
  {
    manifestPath,
    tempFileWriter,
  }: {
    readonly manifestPath: string;
    readonly tempFileWriter: AtomicTempFileWriter;
  },
): Promise<unknown> {
  try {
    await writeMergedManifest({
      manifestPath,
      manifest: EMPTY_MANIFEST,
      tempFileWriter,
    },);
  }
  catch (manifestWriteError: unknown) {
    return manifestWriteError;
  }

  throw new Error('Manifest write unexpectedly succeeded',);
}

//endregion Orphaned manifest temp file fixture helpers

await describe({
  name: 'file-enforcer orphaned manifest temp files',
  concurrency: 1,
  children: [
    it({
      name: 'removes a temp file left by a dead process on the next manifest write',
      fn: async function removesDeadProcessOrphan(): Promise<void> {
        const tempDir = await setup();
        await using _cleanup = {
          [Symbol.asyncDispose](): Promise<void> {
            return teardown(tempDir,);
          },
        };
        const manifestPath = join(
          tempDir,
          MANIFEST_FILE_NAME,
        );
        const orphanName = tempFileName({
          manifestFileName: MANIFEST_FILE_NAME,
          processIdText: String(DEAD_OWNER_PROCESS_ID,),
        },);
        await plantTempFile({
          directory: tempDir,
          entryName: orphanName,
        },);

        await writeMergedManifest({
          manifestPath,
          manifest: EMPTY_MANIFEST,
        },);

        expect(await sortedEntries(tempDir,),).toEqual([MANIFEST_FILE_NAME,],);
        expect(JSON.parse(await readFile(manifestPath, 'utf8',),),).toEqual(EMPTY_MANIFEST,);
      },
    },),

    it({
      name: 'keeps a temp file whose writer process is alive',
      fn: async function keepsLiveProcessTempFile(): Promise<void> {
        const tempDir = await setup();
        await using _cleanup = {
          [Symbol.asyncDispose](): Promise<void> {
            return teardown(tempDir,);
          },
        };
        const manifestPath = join(
          tempDir,
          MANIFEST_FILE_NAME,
        );
        const liveName = tempFileName({
          manifestFileName: MANIFEST_FILE_NAME,
          processIdText: String(process.pid,),
        },);
        await plantTempFile({
          directory: tempDir,
          entryName: liveName,
        },);

        await writeMergedManifest({
          manifestPath,
          manifest: EMPTY_MANIFEST,
        },);

        expect(await sortedEntries(tempDir,),).toEqual([
          MANIFEST_FILE_NAME,
          liveName,
        ].toSorted(),);
      },
    },),

    it({
      name: 'keeps entries that are not temp files of this manifest even when their pid is dead',
      fn: async function keepsForeignEntries(): Promise<void> {
        const tempDir = await setup();
        await using _cleanup = {
          [Symbol.asyncDispose](): Promise<void> {
            return teardown(tempDir,);
          },
        };
        const manifestPath = join(
          tempDir,
          MANIFEST_FILE_NAME,
        );
        const deadPidText = String(DEAD_OWNER_PROCESS_ID,);
        /**
         Entries that resemble temp files but must never be claimed by the manifest writer.
         */
        const foreignNames = [
          // Temp file of another destination whose name starts with the manifest file name.
          `${MANIFEST_FILE_NAME}.extra.${deadPidText}.${randomUUID()}.tmp`,
          // Temp file of a different manifest.
          tempFileName({
            manifestFileName: 'other-manifest.json',
            processIdText: deadPidText,
          },),
          // Non-numeric, non-canonical, and zero process ids.
          tempFileName({
            manifestFileName: MANIFEST_FILE_NAME,
            processIdText: 'abc',
          },),
          tempFileName({
            manifestFileName: MANIFEST_FILE_NAME,
            processIdText: '0',
          },),
          tempFileName({
            manifestFileName: MANIFEST_FILE_NAME,
            processIdText: `0${deadPidText}`,
          },),
          // Missing unique token and missing suffix.
          `${MANIFEST_FILE_NAME}.${deadPidText}.tmp`,
          `${MANIFEST_FILE_NAME}.${deadPidText}.${randomUUID()}.bak`,
        ];
        await Promise.all(foreignNames.map(async function plantForeign(entryName,): Promise<void> {
          await plantTempFile({
            directory: tempDir,
            entryName,
          },);
        },),);

        await writeMergedManifest({
          manifestPath,
          manifest: EMPTY_MANIFEST,
        },);

        expect(await sortedEntries(tempDir,),).toEqual([
          MANIFEST_FILE_NAME,
          ...foreignNames,
        ].toSorted(),);
      },
    },),

    it({
      name: 'removes its own temp file when the write fails before the rename',
      fn: async function removesOwnTempFileOnFailedWrite(): Promise<void> {
        const tempDir = await setup();
        await using _cleanup = {
          [Symbol.asyncDispose](): Promise<void> {
            return teardown(tempDir,);
          },
        };
        const manifestPath = join(
          tempDir,
          MANIFEST_FILE_NAME,
        );
        /**
         Fault-injection writer that leaves partial temp content before throwing.

         @param tempPath - Temp path provided by the manifest writer.

         @example
         ```ts
         failAfterPartialTempWrite({ tempPath, content: '{}' });
         ```
         */
        async function failAfterPartialTempWrite(
          {
            tempPath,
          }: Parameters<AtomicTempFileWriter>[0],
        ): Promise<void> {
          await writeFile(
            tempPath,
            'partial manifest',
          );
          throw new Error('intentional manifest temp write failure',);
        }

        const manifestWriteError = await captureManifestWriteError({
          manifestPath,
          tempFileWriter: failAfterPartialTempWrite,
        },);

        expect(manifestWriteError,).toBeInstanceOf(Error,);
        expect(await sortedEntries(tempDir,),).toEqual([],);
      },
    },),
  ],
},);
