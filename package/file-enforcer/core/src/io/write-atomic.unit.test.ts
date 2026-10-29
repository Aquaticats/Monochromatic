import {
  mkdtemp,
  readFile,
  readdir,
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

import {
  type AtomicTempFileWriter,
  overwrite,
  writeFileAtomically,
} from '../../dist/final/node/index.mjs';

//region Atomic write fixture helpers

/**
 Creates isolated temp directory for atomic write tests.
 
 @returns Temp directory path.
 
 @example
 ```ts
 const tempDir = await setup();
 ```
 */
async function setup(): Promise<string> {
  return await mkdtemp(join(
    scratchRoot(),
    'file-enforcer-write-atomic-',
  ),);
}

/**
 Removes isolated temp directory.
 
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
 Captures error thrown by atomic write fault injection.
 
 @param filePath - Destination path passed to atomic writer.
 
 @param content - Replacement content passed to atomic writer.
 
 @param tempFileWriter - Temp-file writer expected to throw.
 
 @returns Error thrown by atomic write.
 
 @throws When atomic write unexpectedly succeeds.
 
 @example
 ```ts
 const error = await captureAtomicWriteError({ filePath, content: 'new', tempFileWriter });
 ```
 */
async function captureAtomicWriteError(
  {
    filePath,
    content,
    tempFileWriter,
  }: {
    readonly content: string;
    readonly filePath: string;
    readonly tempFileWriter: AtomicTempFileWriter;
  },
): Promise<unknown> {
  try {
    await writeFileAtomically({
      filePath,
      content,
      tempFileWriter,
    },);
  }
  catch (atomicWriteError: unknown) {
    return atomicWriteError;
  }

  throw new Error('Atomic write unexpectedly succeeded',);
}

/**
 Returns temp files left beside destination outputs.
 
 @param tempDir - Directory to inspect.
 
 @returns Directory entries ending with atomic temp suffix.
 
 @example
 ```ts
 const leftovers = await atomicTempEntries(tempDir);
 ```
 */
async function atomicTempEntries(tempDir: string,): Promise<readonly string[]> {
  /**
   Directory entries after attempted write.
   */
  const entries = await readdir(tempDir,);
  return entries.filter(function isAtomicTempEntry(entry,): boolean {
    return entry.endsWith('.tmp',);
  },);
}

//endregion Atomic write fixture helpers

await describe({
  name: 'atomic destination writes',
  concurrency: 1,
  children: [
    it({
      name: 'preserves existing destination and removes temp file when temp write fails',
      fn: async function preservesExistingDestinationOnTempWriteFailure(): Promise<void> {
        const tempDir = await setup();
        await using _cleanup = {
          [Symbol.asyncDispose](): Promise<void> {
            return teardown(tempDir,);
          },
        };
        const dest = join(
          tempDir,
          'existing.txt',
        );
        await writeFile(
          dest,
          'old content',
        );
        /**
         Fault-injection writer that leaves partial temp content before throwing.
         
         @param tempPath - Temp path provided by atomic writer.
         
         @example
         ```ts
         failAfterPartialTempWrite({ tempPath, content: 'new' });
         ```
         */
        async function failAfterPartialTempWrite(
          {
            tempPath,
          }: Parameters<AtomicTempFileWriter>[0],
        ): Promise<void> {
          await writeFile(
            tempPath,
            'partial new content',
          );
          throw new Error('intentional atomic temp write failure',);
        }

        const atomicWriteError = await captureAtomicWriteError({
          filePath: dest,
          content: 'new content',
          tempFileWriter: failAfterPartialTempWrite,
        },);

        expect(atomicWriteError,).toBeInstanceOf(Error,);
        expect(await readFile(dest, 'utf8',),).toBe('old content',);
        expect(await atomicTempEntries(tempDir,),).toEqual([],);
      },
    },),

    it({
      name: 'public overwrite replaces content without leaving temp files',
      fn: async function publicOverwriteUsesAtomicDestinationReplacement(): Promise<void> {
        const tempDir = await setup();
        await using _cleanup = {
          [Symbol.asyncDispose](): Promise<void> {
            return teardown(tempDir,);
          },
        };
        const dest = join(
          tempDir,
          'public.txt',
        );
        await writeFile(
          dest,
          'old public content',
        );

        await overwrite({
          dest,
          content: 'new public content',
          manifestPath: join(
            tempDir,
            'manifest.json',
          ),
        },);

        expect(await readFile(dest, 'utf8',),).toBe('new public content',);
        expect(await atomicTempEntries(tempDir,),).toEqual([],);
      },
    },),
  ],
},);
