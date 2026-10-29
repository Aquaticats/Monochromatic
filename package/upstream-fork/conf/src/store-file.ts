/**
 Config-file reading and writing pipeline.
 
 Owns the read path (bytes -> decrypt -> deserialize -> validate) and the
 write path (serialize -> encrypt -> atomic replace) that upstream `conf`
 keeps as private methods,
 so the store object above it only orchestrates.
 
 @module
 */

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import {
  tagged,
  type Logger,
} from '@monochromatic-dev/logger-core/ts';
import path from 'node:path';

import {
  DecryptionFailedError,
  SchemaViolationError,
} from './errors.ts';
import {
  decryptConfigData,
  encryptSerializedStore,
  type EncryptionAlgorithm,
  type EncryptionKey,
} from './encryption.ts';
import { createPlainObject, } from './store-access.ts';
import { bytesToString, } from './bytes.ts';
import {
  ensureDirectory,
  pathExists,
  readFileBytes,
  readFileText,
  writeFileAtomic,
} from './file-io.ts';
import type {
  Deserialize,
  Serialize,
} from './types.ts';

//region Types

/**
 Read/write surface over one config file.
 
 @example
 ```ts
 const storeFile = createStoreFile(options);
 const store = storeFile.readStore();
 storeFile.writeStore(store);
 ```
 */
export type StoreFile = {
  /**
   Absolute path of the config file.
   */
  readonly path: string;
  /**
   Whether the config file exists.
   */
  readonly fileExists: () => boolean;
  /**
   Creates the config directory when missing.
   */
  readonly ensureDirectory: () => void;
  /**
   Reads and parses the file without validation or corrupt-file triage.
   */
  readonly readParsedFile: () => Record<string, unknown>;
  /**
   Reads the file the way the store exposes it:
   validated,
   with corrupt files cleared or rethrown per `clearInvalidConfig`.
   */
  readonly readStore: () => Record<string, unknown>;
  /**
   Serializes and writes the store,
   atomically replacing the file.
   */
  readonly writeStore: (store: Record<string, unknown>,) => void;
};

//endregion Types

//region Factory

/**
 Builds the read/write pipeline for one config file.
 
 @param path - Absolute config file path.
 
 @param encryptionKey - Key material enabling encryption when present.
 
 @param encryptionAlgorithm - Algorithm applied when encrypting.
 
 @param serialize - Store-to-text serializer.
 
 @param deserialize - Text-to-store deserializer.
 
 @param clearInvalidConfig - Whether corrupt files read as empty stores.
 
 @param configFileMode - File mode for created files.
 
 @param validate - Validation hook run on every parsed store.
 
 @param logger - Logger for corrupt-file triage diagnostics.
 
 @returns StoreFile pipeline bound to these settings.
 
 @example
 ```ts
 const storeFile = createStoreFile({
   path: '/tmp/app/config.json',
   encryptionKey: undefined,
   encryptionAlgorithm: 'aes-256-cbc',
   serialize: function serialize(value): string { return JSON.stringify(value); },
   deserialize: function deserialize(text): Record<string, unknown> { return JSON.parse(text) as Record<string, unknown>; },
   clearInvalidConfig: false,
   configFileMode: 0o666,
   validate: function validate(): void {},
   logger,
 });
 ```
 */
export function createStoreFile<T extends Record<string, unknown>>({
  path: filePath,
  encryptionKey,
  encryptionAlgorithm,
  serialize,
  deserialize,
  clearInvalidConfig,
  configFileMode,
  validate,
  logger,
}: {
  readonly path: string;
  readonly encryptionKey?: EncryptionKey;
  readonly encryptionAlgorithm: EncryptionAlgorithm;
  readonly serialize: Serialize<T>;
  readonly deserialize: Deserialize<T>;
  readonly clearInvalidConfig: boolean;
  readonly configFileMode: number;
  readonly validate: (data: unknown,) => void;
  readonly logger?: Logger;
},): StoreFile {
  /**
   Logger wrapped with this factory's name so triage diagnostics name
   their origin.
   */
  const log = logger ?? tagged({
    tag: createStoreFile.name,
  },);

  /**
   Reads and parses the file without validation or triage.
   
   @returns File contents as a store-shaped object.
   */
  function readParsedFile(): Record<string, unknown> {
    /**
     Serialized store text after optional decryption.
     */
    const decrypted = encryptionKey === undefined
      ? readFileText({
        path: filePath,
      },)
      : decryptConfigData({
        data: readFileBytes({
          path: filePath,
        },),
        encryptionKey,
        encryptionAlgorithm,
        logger: log,
      },);
    return Object.assign(
      createPlainObject(),
      deserialize(decrypted,),
    );
  }

  /**
   Reads the file the way the store exposes it.
   
   @returns Validated store contents,
   or an empty store when `clearInvalidConfig` accepts the failure.
   */
  function readStore(): Record<string, unknown> {
    try {
      /**
       Parsed file contents awaiting validation.
       */
      const parsed = readParsedFile();
      validate(parsed,);
      return parsed;
    }
    catch (error) {
      if (isMissingFileError(error,)) {
        ensureDirectory({
          path: path.dirname(filePath,),
        },);
        return createPlainObject();
      }
      if (clearInvalidConfig && isRecoverableReadFailure(error,)) {
        log.warn(`clearing invalid config after: ${caughtValueText(error,)}`,);
        return createPlainObject();
      }
      throw error;
    }
  }

  return {
    path: filePath,
    fileExists: function fileExists(): boolean {
      return pathExists({
        path: filePath,
      },);
    },
    ensureDirectory: function ensureDirectoryPath(): void {
      ensureDirectory({
        path: path.dirname(filePath,),
      },);
    },
    readParsedFile,
    readStore,
    writeStore: function writeStore(store: Record<string, unknown>,): void {
      /**
       Serialized store text.
       */
      // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- file contents are a generic dictionary serialized through the caller's store type; the shape proof lives with the schema validator.
      const serialized = serialize(store as T,);
      /**
       File content: serialized text,
       or its encryption when a key is configured.
       */
      const data = encryptionKey === undefined
        ? serialized
        : encryptSerializedStore({
          serialized,
          encryptionKey,
          encryptionAlgorithm,
        },);
      writeFileAtomic({
        path: filePath,
        data,
        mode: configFileMode,
      },);
    },
  };
}

//endregion Factory

//region Failure triage

/**
 Reports whether a read failure just means the file is not there yet.
 
 @param error - Caught failure from the read path.
 
 @returns `true` for a missing-file failure.
 
 @example
 ```ts
 isMissingFileError(new Error('nope')); // => false
 ```
 */
export function isMissingFileError(error: unknown,): boolean {
  return Error.isError(error,)
    && ('code' in error)
    && (error.code === 'ENOENT');
}

/**
 Reports whether `clearInvalidConfig` should swallow a read failure.
 
 Upstream `conf` clears on malformed JSON,
 schema violations,
 and decryption failures;
 everything else surfaces.
 
 @param error - Caught failure from the read path.
 
 @returns `true` when the store should read as empty.
 
 @example
 ```ts
 isRecoverableReadFailure(new DecryptionFailedError()); // => true
 ```
 */
export function isRecoverableReadFailure(error: unknown,): boolean {
  return (error instanceof SyntaxError)
    || (error instanceof SchemaViolationError)
    || (error instanceof DecryptionFailedError);
}

//endregion Failure triage
