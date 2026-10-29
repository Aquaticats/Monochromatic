/**
 Unit tests for the synchronous filesystem primitives backing the config
 store: reads, existence probes, directory creation, atomic writes, and
 cleanup.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import path from 'node:path';
import {
  readdirSync,
  statSync,
} from 'node:fs';
import type { Logger, } from '@monochromatic-dev/logger-core/ts';

import {
  ensureDirectory,
  isMissingFileError,
  pathExists,
  readFileBytes,
  readFileText,
  removeFileIfExists,
  stringToBytes,
  writeFileAtomic,
} from '../dist/final/neutral/index.mjs';

import {
  createTempDirectory,
} from './test-support.ts';

/**
 Multi-byte UTF-8 sample mixing accents, a composed character, and an
 astral-plane emoji so byte length differs from string length.
 */
const MULTI_BYTE_TEXT = 'Grüße 🌏 café';

/**
 Captures whatever a call throws so assertions can inspect the error.
 
 @param call - Invocation expected to throw.
 
 @returns Captured failure, or `undefined` when the call returns normally.
 */
function captureFailure(call: () => void,): unknown {
  try {
    call();
  }
  catch (error) {
    return error;
  }
  return undefined;
}

/**
 No-op body for logger levels the tests do not observe.
 
 @param message - Ignored log message.
 */
function ignoreMessage(message: string,): void {
  void message;
}

/**
 Builds a logger stub capturing warnings so cleanup diagnostics stay
 observable and out of the test output.
 
 @returns The stub logger and the warning list it records.
 */
function createRecordingLogger(): {
  readonly logger: Logger;
  readonly warnings: string[];
} {
  /**
   Warning messages recorded so far.
   */
  const warnings: string[] = [];
  return {
    logger: {
      debug: ignoreMessage,
      error: ignoreMessage,
      fatal: ignoreMessage,
      flush: async function flush(): Promise<void> {
        // Nothing is buffered in tests, so there is nothing to flush.
      },
      info: ignoreMessage,
      trace: ignoreMessage,
      warn: function warn(message: string,): void {
        warnings.push(message,);
      },
    },
    warnings,
  };
}

await describe({
  name: 'file-io primitives',
  children: [
    it({
      name: 'readFileText round-trips UTF-8 text including multi-byte characters',
      fn: async () => {
        /**
         Temp directory hosting the round-trip file.
         */
        const directory = createTempDirectory();
        /**
         Absolute path of the round-trip file.
         */
        const filePath = path.join(
          directory,
          'text.json',
        );
        writeFileAtomic({
          path: filePath,
          data: MULTI_BYTE_TEXT,
          mode: 0o600,
        },);
        expect(readFileText({
          path: filePath,
        },),).toBe(MULTI_BYTE_TEXT,);
      },
    },),

    it({
      name: 'readFileBytes returns exactly the bytes behind string and Uint8Array data',
      fn: async () => {
        /**
         Temp directory hosting both round-trip files.
         */
        const directory = createTempDirectory();
        /**
         Absolute path written with string data.
         */
        const textPath = path.join(
          directory,
          'from-text.bin',
        );
        writeFileAtomic({
          path: textPath,
          data: MULTI_BYTE_TEXT,
          mode: 0o600,
        },);
        expect(readFileBytes({
          path: textPath,
        },),).toEqual(stringToBytes(MULTI_BYTE_TEXT,),);

        /**
         Byte payload written directly as an array.
         */
        const bytes = new Uint8Array([
          0,
          127,
          128,
          255,
        ],);
        /**
         Absolute path written with byte data.
         */
        const bytesPath = path.join(
          directory,
          'from-bytes.bin',
        );
        writeFileAtomic({
          path: bytesPath,
          data: bytes,
          mode: 0o600,
        },);
        expect(readFileBytes({
          path: bytesPath,
        },),).toEqual(bytes,);
      },
    },),

    it({
      name: 'readFileText and readFileBytes report a missing file through isMissingFileError',
      fn: async () => {
        /**
         Absolute path that was never created.
         */
        const missingPath = path.join(
          createTempDirectory(),
          'absent.json',
        );
        /**
         Failure captured from the text read.
         */
        const textFailure = captureFailure(function readMissingText(): void {
          readFileText({
            path: missingPath,
          },);
        },);
        expect(isMissingFileError(textFailure,),).toBe(true,);

        /**
         Failure captured from the byte read.
         */
        const bytesFailure = captureFailure(function readMissingBytes(): void {
          readFileBytes({
            path: missingPath,
          },);
        },);
        expect(isMissingFileError(bytesFailure,),).toBe(true,);
      },
    },),

    it({
      name: 'pathExists reports true for an existing file and false for a missing path',
      fn: async () => {
        /**
         Temp directory hosting one existing file.
         */
        const directory = createTempDirectory();
        /**
         Absolute path created before the probe.
         */
        const filePath = path.join(
          directory,
          'present.json',
        );
        writeFileAtomic({
          path: filePath,
          data: 'x',
          mode: 0o600,
        },);
        expect(pathExists({
          path: filePath,
        },),).toBe(true,);
        expect(pathExists({
          path: path.join(
            directory,
            'absent.json',
          ),
        },),).toBe(false,);
      },
    },),

    it({
      name: 'ensureDirectory creates nested directories and stays idempotent on repeat calls',
      fn: async () => {
        /**
         Nested directory that does not exist yet.
         */
        const nestedPath = path.join(
          createTempDirectory(),
          'one',
          'two',
          'three',
        );
        ensureDirectory({
          path: nestedPath,
        },);
        expect(pathExists({
          path: nestedPath,
        },),).toBe(true,);
        ensureDirectory({
          path: nestedPath,
        },);
        expect(pathExists({
          path: nestedPath,
        },),).toBe(true,);
      },
    },),

    it({
      name: 'writeFileAtomic creates the file with at most the requested mode after umask',
      fn: async () => {
        /**
         Requested creation mode kept restrictive so umask can only reduce
         it further.
         */
        const requestedMode = 0o600;
        /**
         Absolute path created by the write.
         */
        const filePath = path.join(
          createTempDirectory(),
          'mode.json',
        );
        writeFileAtomic({
          path: filePath,
          data: 'x',
          mode: requestedMode,
        },);
        /**
         Effective permission bits on the created file.
         */
        const effectiveMode = statSync(filePath,)
          .mode & 0o777;
        expect(effectiveMode,).toBeLessThanOrEqual(requestedMode,);
      },
    },),

    it({
      name: 'writeFileAtomic replaces existing content when the target already exists',
      fn: async () => {
        /**
         Temp directory hosting the overwritten file.
         */
        const directory = createTempDirectory();
        /**
         Absolute path written twice.
         */
        const filePath = path.join(
          directory,
          'overwrite.json',
        );
        writeFileAtomic({
          path: filePath,
          data: 'first content',
          mode: 0o600,
        },);
        writeFileAtomic({
          path: filePath,
          data: 'second content',
          mode: 0o600,
        },);
        expect(readFileText({
          path: filePath,
        },),).toBe('second content',);
      },
    },),

    it({
      name: 'writeFileAtomic leaves no temporary sibling files behind',
      fn: async () => {
        /**
         Temp directory inspected for leftover staging files.
         */
        const directory = createTempDirectory();
        writeFileAtomic({
          path: path.join(
            directory,
            'target.json',
          ),
          data: 'x',
          mode: 0o600,
        },);
        /**
         Directory entries still carrying the temporary-file marker.
         */
        const leftovers = readdirSync(directory,)
          .filter(function hasTemporaryMarker(entry: string,): boolean {
            return entry.includes('.tmp',);
          },);
        expect(leftovers,).toHaveLength(0,);
      },
    },),

    it({
      name: 'removeFileIfExists removes an existing file',
      fn: async () => {
        /**
         Absolute path created just to be removed.
         */
        const filePath = path.join(
          createTempDirectory(),
          'remove-me.json',
        );
        writeFileAtomic({
          path: filePath,
          data: 'x',
          mode: 0o600,
        },);
        removeFileIfExists({
          path: filePath,
        },);
        expect(pathExists({
          path: filePath,
        },),).toBe(false,);
      },
    },),

    it({
      name: 'removeFileIfExists swallows the failure when the file is missing',
      fn: async () => {
        /**
         Absolute path that was never created.
         */
        const missingPath = path.join(
          createTempDirectory(),
          'absent.json',
        );
        /**
         Recording logger plus its warning list.
         */
        const { logger, warnings, } = createRecordingLogger();
        expect(removeFileIfExists({
          path: missingPath,
          logger,
        },),).toBeUndefined();
        expect(warnings,).toHaveLength(1,);
      },
    },),

    it({
      name: 'removeFileIfExists swallows the failure when the target cannot be unlinked',
      fn: async () => {
        /**
         Directory path standing in for an unlinkable target.
         */
        const directoryPath = path.join(
          createTempDirectory(),
          'a-directory',
        );
        ensureDirectory({
          path: directoryPath,
        },);
        /**
         Recording logger plus its warning list.
         */
        const { logger, warnings, } = createRecordingLogger();
        expect(removeFileIfExists({
          path: directoryPath,
          logger,
        },),).toBeUndefined();
        expect(pathExists({
          path: directoryPath,
        },),).toBe(true,);
        expect(warnings,).toHaveLength(1,);
      },
    },),

    it({
      name: 'writeFileAtomic writes directly without a temporary sibling when SNAP is set',
      fn: async () => {
        /**
         Prior SNAP value restored after the probe.
         */
        const priorSnap = process.env.SNAP;
        /**
         Temp directory hosting the directly-written file.
         */
        const directory = createTempDirectory();
        /**
         Absolute path written under the Snap code path.
         */
        const filePath = path.join(
          directory,
          'snap.json',
        );
        process.env.SNAP = '/snap/conf-fork';
        writeFileAtomic({
          path: filePath,
          data: 'direct write',
          mode: 0o600,
        },);
        delete process.env.SNAP;
        if (priorSnap !== undefined)
          process.env.SNAP = priorSnap;
        expect(readFileText({
          path: filePath,
        },),).toBe('direct write',);
        /**
         Directory entries still carrying the temporary-file marker.
         */
        const leftovers = readdirSync(directory,)
          .filter(function hasTemporaryMarker(entry: string,): boolean {
            return entry.includes('.tmp',);
          },);
        expect(leftovers,).toHaveLength(0,);
      },
    },),
  ],
},);
