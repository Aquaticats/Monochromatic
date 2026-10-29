/**
 Unit tests for the config-file read/write pipeline: parsed reads,
 validated reads with corrupt-file triage, custom serialization,
 encryption round-trips, and the directory and existence helpers.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import path from 'node:path';
import { writeFileSync, } from 'node:fs';
import type { Logger, } from '@monochromatic-dev/logger-core/ts';

import {
  bytesToString,
  createStoreFile,
  DecryptionFailedError,
  ensureDirectory,
  isMissingFileError,
  pathExists,
  readFileBytes,
  readFileText,
  SchemaViolationError,
  type Deserialize,
  type EncryptionAlgorithm,
  type Serialize,
  type StoreFile,
} from '../dist/final/neutral/index.mjs';

import {
  createTempDirectory,
} from './test-support.ts';

/**
 File name the fixtures place inside each temp directory.
 */
const CONFIG_FILE_NAME = 'config.json';

/**
 Marker prefix of the custom wire format used by the serialization tests.
 */
const CUSTOM_PREFIX = 'CONF1\n';

/**
 Malformed JSON sample the triage paths must classify as a syntax failure.
 */
const MALFORMED_JSON = '{ malformed';

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
 Builds a logger stub capturing warnings so corrupt-file triage diagnostics
 stay observable and out of the test output.
 
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

/**
 Builds the shared `createStoreFile` fields for a JSON store living in one
 fresh temp directory.
 
 @param directory - Temp directory hosting the config file.
 
 @returns Base pipeline options each test overrides as needed.
 */
function jsonStoreOptions(directory: string,): {
  readonly path: string;
  readonly encryptionAlgorithm: EncryptionAlgorithm;
  readonly serialize: Serialize<Record<string, unknown>>;
  readonly deserialize: Deserialize<Record<string, unknown>>;
  readonly clearInvalidConfig: boolean;
  readonly configFileMode: number;
  readonly validate: (data: unknown,) => void;
} {
  return {
    path: path.join(
      directory,
      CONFIG_FILE_NAME,
    ),
    encryptionAlgorithm: 'aes-256-cbc',
    serialize: function serialize(value: Record<string, unknown>,): string {
      return JSON.stringify(value,);
    },
    deserialize: function deserialize(text: string,): Record<string, unknown> {
      return JSON.parse(text,) as Record<string, unknown>;
    },
    clearInvalidConfig: false,
    configFileMode: 0o600,
    validate: function validate(): void {},
  };
}

await describe({
  name: createStoreFile.name,
  children: [
    it({
      name: 'readParsedFile returns a null-prototype object holding the parsed contents',
      fn: async () => {
        /**
         Temp directory hosting the pipeline under test.
         */
        const directory = createTempDirectory();
        /**
         Pipeline under test.
         */
        const storeFile: StoreFile = createStoreFile(jsonStoreOptions(directory,),);
        storeFile.writeStore({
          a: 1,
          greeting: 'héllo',
        },);
        /**
         Parsed file contents under inspection.
         */
        const parsed = storeFile.readParsedFile();
        expect(Object.getPrototypeOf(parsed,),).toBe(null,);
        expect(Object.entries(parsed,),).toEqual([
          [
            'a',
            1,
          ],
          [
            'greeting',
            'héllo',
          ],
        ],);
      },
    },),

    it({
      name: 'readParsedFile throws a missing-file error when the config file does not exist',
      fn: async () => {
        /**
         Temp directory hosting the never-written config file.
         */
        const directory = createTempDirectory();
        /**
         Pipeline whose config file was never written.
         */
        const storeFile = createStoreFile(jsonStoreOptions(directory,),);
        /**
         Failure captured from the missing-file read.
         */
        const failure = captureFailure(function readMissingFile(): void {
          storeFile.readParsedFile();
        },);
        expect(isMissingFileError(failure,),).toBe(true,);
      },
    },),

    it({
      name: 'readStore calls the validate hook with the parsed object it returns',
      fn: async () => {
        /**
         Objects handed to the validate hook.
         */
        const seen: unknown[] = [];
        /**
         Pipeline whose validate hook records its input.
         */
        const storeFile = createStoreFile({
          ...jsonStoreOptions(createTempDirectory(),),
          validate: function recordValidated(data: unknown,): void {
            seen.push(data,);
          },
        },);
        storeFile.writeStore({
          a: 1,
        },);
        /**
         Store returned alongside the recorded hook call.
         */
        const store = storeFile.readStore();
        expect(seen,).toHaveLength(1,);
        expect(seen[0],).toBe(store,);
      },
    },),

    it({
      name: 'readStore creates the directory and returns an empty null-prototype store when the file is missing',
      fn: async () => {
        /**
         Nested directory that does not exist yet.
         */
        const directory = path.join(
          createTempDirectory(),
          'missing',
          'nested',
        );
        /**
         Pipeline whose directory and file are both absent.
         */
        const storeFile = createStoreFile(jsonStoreOptions(directory,),);
        /**
         Store returned for the missing file.
         */
        const store = storeFile.readStore();
        expect(pathExists({
          path: directory,
        },),).toBe(true,);
        expect(Object.getPrototypeOf(store,),).toBe(null,);
        expect(Object.keys(store,),).toEqual([],);
      },
    },),

    it({
      name: 'readStore returns an empty store and warns for malformed JSON when clearInvalidConfig is true',
      fn: async () => {
        /**
         Temp directory hosting the corrupt file.
         */
        const directory = createTempDirectory();
        /**
         Recording logger plus its warning list.
         */
        const { logger, warnings, } = createRecordingLogger();
        /**
         Pipeline configured to clear invalid config.
         */
        const storeFile = createStoreFile({
          ...jsonStoreOptions(directory,),
          clearInvalidConfig: true,
          logger,
        },);
        storeFile.ensureDirectory();
        writeFileSync(path.join(
          directory,
          CONFIG_FILE_NAME,
        ), MALFORMED_JSON,);
        /**
         Store returned for the corrupt file.
         */
        const store = storeFile.readStore();
        expect(Object.keys(store,),).toEqual([],);
        expect(warnings.some(function mentionsClearing(message: string,): boolean {
          return message.includes('clearing invalid config',);
        },),).toBe(true,);
      },
    },),

    it({
      name: 'readStore returns an empty store when the validate hook throws SchemaViolationError and clearInvalidConfig is true',
      fn: async () => {
        /**
         Temp directory hosting the pipeline under test.
         */
        const directory = createTempDirectory();
        /**
         Recording logger keeping triage diagnostics out of the test output.
         */
        const { logger, } = createRecordingLogger();
        /**
         Pipeline whose validate hook rejects every store.
         */
        const storeFile = createStoreFile({
          ...jsonStoreOptions(directory,),
          clearInvalidConfig: true,
          logger,
          validate: function rejectStore(): void {
            throw new SchemaViolationError({
              violations: [
                '`a` must be string',
              ],
            },);
          },
        },);
        storeFile.writeStore({
          a: 1,
        },);
        /**
         Store returned after the hook failure.
         */
        const store = storeFile.readStore();
        expect(Object.keys(store,),).toEqual([],);
      },
    },),

    it({
      name: 'readStore returns an empty store after a decryption failure when clearInvalidConfig is true',
      fn: async () => {
        /**
         Temp directory hosting the undecryptable file.
         */
        const directory = createTempDirectory();
        /**
         Recording logger keeping triage diagnostics out of the test output.
         */
        const { logger, } = createRecordingLogger();
        /**
         Pipeline configured for authenticated encryption.
         */
        const storeFile = createStoreFile({
          ...jsonStoreOptions(directory,),
          encryptionKey: 'correct horse battery staple',
          encryptionAlgorithm: 'aes-256-gcm',
          clearInvalidConfig: true,
          logger,
        },);
        storeFile.ensureDirectory();
        writeFileSync(path.join(
          directory,
          CONFIG_FILE_NAME,
        ), new Uint8Array([
          1,
          2,
          3,
        ],),);
        /**
         Store returned for the undecryptable file.
         */
        const store = storeFile.readStore();
        expect(Object.keys(store,),).toEqual([],);
      },
    },),

    it({
      name: 'readStore rethrows malformed JSON as SyntaxError when clearInvalidConfig is false',
      fn: async () => {
        /**
         Temp directory hosting the corrupt file.
         */
        const directory = createTempDirectory();
        /**
         Pipeline with triage disabled.
         */
        const storeFile = createStoreFile(jsonStoreOptions(directory,),);
        storeFile.ensureDirectory();
        writeFileSync(path.join(
          directory,
          CONFIG_FILE_NAME,
        ), MALFORMED_JSON,);
        /**
         Failure captured from the corrupt-file read.
         */
        const failure = captureFailure(function readCorruptFile(): void {
          storeFile.readStore();
        },);
        expect(failure,).toBeInstanceOf(SyntaxError,);
      },
    },),

    it({
      name: 'readStore rethrows validation failures clearInvalidConfig does not cover even when it is true',
      fn: async () => {
        /**
         Marker error thrown by the validate hook.
         */
        const hookFailure = new Error('boom',);
        /**
         Pipeline whose hook throws outside the recoverable set.
         */
        const storeFile = createStoreFile({
          ...jsonStoreOptions(createTempDirectory(),),
          clearInvalidConfig: true,
          validate: function rejectStore(): void {
            throw hookFailure;
          },
        },);
        storeFile.writeStore({
          a: 1,
        },);
        /**
         Failure captured from the hook failure.
         */
        const failure = captureFailure(function readRejectedStore(): void {
          storeFile.readStore();
        },);
        expect(failure,).toBe(hookFailure,);
      },
    },),

    it({
      name: 'readStore rethrows read failures other than missing files even when clearInvalidConfig is true',
      fn: async () => {
        /**
         Directory standing in for an unreadable config file.
         */
        const blockedPath = path.join(
          createTempDirectory(),
          'a-directory',
        );
        /**
         Pipeline whose config path is a directory.
         */
        const storeFile = createStoreFile({
          ...jsonStoreOptions(createTempDirectory(),),
          path: blockedPath,
          clearInvalidConfig: true,
        },);
        ensureDirectory({
          path: blockedPath,
        },);
        /**
         Failure captured from the unreadable read.
         */
        const failure = captureFailure(function readBlockedFile(): void {
          storeFile.readStore();
        },);
        expect(isMissingFileError(failure,),).toBe(false,);
        expect(failure,).toBeInstanceOf(Error,);
      },
    },),

    it({
      name: 'writeStore and readStore round-trip through custom serialize and deserialize',
      fn: async () => {
        /**
         Temp directory hosting the custom-format file.
         */
        const directory = createTempDirectory();
        /**
         Pipeline speaking the custom prefixed wire format.
         */
        const storeFile = createStoreFile({
          ...jsonStoreOptions(directory,),
          serialize: function serializeWithPrefix(value: Record<string, unknown>,): string {
            return CUSTOM_PREFIX + JSON.stringify(value,);
          },
          deserialize: function deserializeWithPrefix(text: string,): Record<string, unknown> {
            return JSON.parse(text.slice(CUSTOM_PREFIX.length,),) as Record<string, unknown>;
          },
        },);
        storeFile.ensureDirectory();
        storeFile.writeStore({
          greeting: 'héllo',
        },);
        expect(readFileText({
          path: storeFile.path,
        },)
          .startsWith(CUSTOM_PREFIX,),).toBe(true,);
        /**
         Store entries read back through the custom deserializer.
         */
        const storedEntries = Object.entries(storeFile.readStore(),);
        expect(storedEntries,).toEqual([
          [
            'greeting',
            'héllo',
          ],
        ],);
      },
    },),

    it({
      name: 'writeStore encrypts the file when an encryptionKey is set and readStore decrypts it back',
      fn: async () => {
        /**
         Temp directory hosting the encrypted file.
         */
        const directory = createTempDirectory();
        /**
         Pipeline configured for authenticated encryption.
         */
        const storeFile = createStoreFile({
          ...jsonStoreOptions(directory,),
          encryptionKey: 'correct horse battery staple',
          encryptionAlgorithm: 'aes-256-gcm',
        },);
        storeFile.ensureDirectory();
        /**
         Store contents round-tripped through encryption.
         */
        const written = {
          token: 'abc-123',
        };
        storeFile.writeStore(written,);
        /**
         Plaintext the encrypting write must not leave on disk.
         */
        const serialized = JSON.stringify(written,);
        /**
         Raw bytes the encrypted write left on disk.
         */
        const rawBytes = readFileBytes({
          path: storeFile.path,
        },);
        expect(bytesToString(rawBytes,) === serialized,).toBe(false,);
        /**
         Store entries read back after decryption.
         */
        const decryptedEntries = Object.entries(storeFile.readStore(),);
        expect(decryptedEntries,).toEqual(Object.entries(written,),);
      },
    },),

    it({
      name: 'fileExists reports the config file only after it exists',
      fn: async () => {
        /**
         Temp directory hosting the pipeline under test.
         */
        const directory = createTempDirectory();
        /**
         Pipeline over a fresh temp directory.
         */
        const storeFile = createStoreFile(jsonStoreOptions(directory,),);
        expect(storeFile.path,).toBe(path.join(
          path.dirname(storeFile.path,),
          CONFIG_FILE_NAME,
        ),);
        expect(storeFile.fileExists(),).toBe(false,);
        storeFile.ensureDirectory();
        storeFile.writeStore({
          a: 1,
        },);
        expect(storeFile.fileExists(),).toBe(true,);
      },
    },),

    it({
      name: 'ensureDirectory creates the config file directory on demand',
      fn: async () => {
        /**
         Nested directory that does not exist yet.
         */
        const directory = path.join(
          createTempDirectory(),
          'deeply',
          'nested',
        );
        /**
         Pipeline whose directory is absent.
         */
        const storeFile = createStoreFile(jsonStoreOptions(directory,),);
        storeFile.ensureDirectory();
        expect(pathExists({
          path: directory,
        },),).toBe(true,);
      },
    },),
  ],
},);
