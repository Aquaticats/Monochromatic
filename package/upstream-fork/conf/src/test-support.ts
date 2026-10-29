/**
 Test-only fixtures shared by this package's unit tests.
 
 Tests exercise real config files,
 so fixtures live in disposable temp directories and,
 for path-resolution tests,
 under injected disposable homes. Nothing here touches a user's real config
 location.
 
 @module
 */

import path from 'node:path';
import process from 'node:process';
import { tmpdir, } from 'node:os';
import {
  createCipheriv,
  createDecipheriv,
  pbkdf2Sync,
  randomBytes,
  randomUUID,
} from 'node:crypto';
import {
  mkdirSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from 'node:fs';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';

import {
  bytesToString,
  concatBytes,
  stringToBytes,
  type EncryptionAlgorithm,
  type EncryptionKey,
} from '../dist/final/neutral/index.mjs';

//region Constants

/**
 Byte length of the encryption initialization vector written before the
 separator.
 
 @example
 ```ts
 INITIALIZATION_VECTOR_LENGTH; // 16
 ```
 */
const INITIALIZATION_VECTOR_LENGTH = 16;

/**
 Byte length of the `aes-256-gcm` authentication tag.
 
 @example
 ```ts
 AUTHENTICATION_TAG_LENGTH; // 16
 ```
 */
const AUTHENTICATION_TAG_LENGTH = 16;

/**
 PBKDF2 round count matching the store's key derivation.
 
 @example
 ```ts
 PBKDF2_ITERATIONS; // 10_000
 ```
 */
const PBKDF2_ITERATIONS = 10_000;

/**
 Byte length of the derived cipher key.
 
 @example
 ```ts
 DERIVED_KEY_LENGTH; // 32
 ```
 */
const DERIVED_KEY_LENGTH = 32;

/**
 PBKDF2 digest matching the store's key derivation.
 
 @example
 ```ts
 PBKDF2_DIGEST; // 'sha512'
 ```
 */
const PBKDF2_DIGEST = 'sha512';

/**
 Per-process fixture registry cleaned up at exit.
 
 @example
 ```ts
 fixtureRegistry.size; // number of live fixtures
 ```
 */
const fixtureRegistry = new Set<() => void>();

//endregion Constants

//region Temp directories

/* oxlint-disable no-restricted-syntax/no-sync -- Test-only fixture module; unit tests are exempt from no-sync but this shared helper is not, so the sync calls centralize here per package/upstream-fork/conf/DECISION.sync-api.md. */

/**
 Removes one directory tree,
 logging and swallowing failures so cleanup never masks a test's own error.
 
 @param directory - Absolute directory path to remove.
 
 @example
 ```ts
 removeDirectory({ directory: tempDirectory, });
 ```
 */
export function removeDirectory({ directory, }: { readonly directory: string; },): void {
  /**
   Logger wrapped with this function's name so cleanup diagnostics name
   their origin.
   */
  const log = tagged({
    tag: removeDirectory.name,
  },);
  try {
    rmSync(
      directory,
      {
        recursive: true,
        force: true,
      },
    );
  }
  catch (error) {
    log.warn(`fixture cleanup of ${directory} failed: ${caughtValueText(error,)}`,);
  }
}

/**
 Creates one disposable temp directory registered for cleanup.
 
 @returns Absolute path of an empty directory under the system temp dir.
 
 @example
 ```ts
 const directory = createTempDirectory();
 ```
 */
export function createTempDirectory(): string {
  /**
   Parent for this fixture's uniquely-named directory.
   */
  const parent = path.join(
    tmpdir(),
    'conf-fork-fixtures',
  );
  mkdirSync(
    parent,
    {
      recursive: true,
    },
  );
  /**
   Empty fixture directory created for one test.
   */
  const directory = path.join(
    parent,
    randomUUID(),
  );
  mkdirSync(directory,);
  fixtureRegistry.add(function cleanupFixtureDirectory(): void {
    removeDirectory({
      directory,
    },);
  },);
  return directory;
}

/**
 Writes one config file as pretty JSON,
 so tests can start from a file the store treats as pre-existing.
 
 @param directory - Fixture directory holding the config file.
 
 @param data - Store contents to serialize.
 
 @param fileName - Config file name inside the directory.
 
 @example
 ```ts
 writeConfigFile({ directory, data: { theme: 'dark', }, });
 ```
 */
export function writeConfigFile({
  directory,
  data,
  fileName = 'config.json',
}: {
  readonly directory: string;
  readonly data: Record<string, unknown>;
  readonly fileName?: string;
},): void {
  writeFileSync(
    path.join(
      directory,
      fileName,
    ),
    JSON.stringify(
      data,
      undefined,
      '\t',
    ),
  );
}

/* oxlint-disable no-restricted-syntax/no-nullish-union -- test fixture mirroring the file's optional bookkeeping, where absence of a recorded version is exactly what migration tests assert on. */
/**
 Reads the recorded migration version straight from the config file,
 since the store hides its `__internal__` bookkeeping.
 
 @param configPath - Absolute config file path.
 
 @returns Recorded version,
 or an empty string when none is recorded.
 
 @example
 ```ts
 readMigrationVersion({ configPath: config.path, }); // '1.0.0'
 ```
 */
export function readMigrationVersion({ configPath, }: { readonly configPath: string; },): string | undefined {
  /**
   Parsed file contents including bookkeeping.
   */
  /* oxlint-disable typescript/no-unsafe-type-assertion -- JSON.parse's dynamic output is narrowed here to the known bookkeeping shape for test assertions only. */
  const parsed = JSON.parse(readFileSync(
    configPath,
    'utf8',
  )) as {
    __internal__?: {
      migrations?: {
        version?: unknown;
      };
    };
  };
  /* oxlint-enable typescript/no-unsafe-type-assertion */
  return (typeof parsed.__internal__
    ?.migrations
    ?.version) === 'string'
    ? parsed.__internal__
      .migrations
      .version
    : undefined;
}

/* oxlint-enable no-restricted-syntax/no-nullish-union */

/* oxlint-enable no-restricted-syntax/no-sync */

//endregion Temp directories

//region Disposable homes

/**
 One injected disposable home with its restoration.
 
 @example
 ```ts
 const home = createDisposableHome();
 restoreHome(home);
 ```
 */
export type DisposableHome = {
  /**
   Directory standing in for the user's home.
   */
  readonly home: string;
  /**
   Environment variable values present before injection,
   keyed by name.
   */
  readonly previousValues: Readonly<Record<string, string>>;
  /**
   Names of environment variables that were absent before injection.
   */
  readonly previousAbsent: readonly string[];
};

/**
 Creates a disposable home directory and points the path-resolution
 environment at it,
 so `configDirectory` resolves inside the fixture.
 
 @returns Disposable home handle carrying the previous environment.
 
 @example
 ```ts
 const home = createDisposableHome();
 ```
 */
export function createDisposableHome(): DisposableHome {
  /**
   Directory standing in for the user's home.
   */
  const home = createTempDirectory();
  /**
   Environment variables path resolution reads.
   */
  const names = [
    'HOME',
    'XDG_CONFIG_HOME',
  ] as const;
  /**
   Values present before injection.
   */
  const previousValues: Record<string, string> = {};
  /**
   Names that were absent before injection.
   */
  const previousAbsent: string[] = [];
  for (const name of names) {
    /**
     Value present before injection for this name.
     */
    const previous = process.env[name];
    if (previous === undefined)
      previousAbsent.push(name,);
    else
      previousValues[name] = previous;
    Reflect.deleteProperty(
      process.env,
      name,
    );
  }
  process.env
    .HOME = home;
  process.env
    .XDG_CONFIG_HOME = path.join(
      home,
      '.config',
    );
  return {
    home,
    previousValues,
    previousAbsent,
  };
}

/**
 Restores the environment a {@link createDisposableHome} call replaced.
 
 @param previousValues - Environment values to put back by name.
 
 @param previousAbsent - Environment names to delete again.
 
 @example
 ```ts
 restoreHome(home);
 ```
 */
export function restoreHome({
  previousValues,
  previousAbsent,
}: DisposableHome,): void {
  for (const [name, value,] of Object.entries(previousValues))
    process.env[name] = value;
  for (const name of previousAbsent)
    Reflect.deleteProperty(
      process.env,
      name,
    );
}

//endregion Disposable homes

//region Encryption fixtures

/* oxlint-disable no-restricted-syntax/no-sync -- Test-only fixture module; unit tests are exempt from no-sync but this shared helper is not, so the sync calls centralize here per package/upstream-fork/conf/DECISION.sync-api.md. */

/**
 Decrypts a config file with the store's wire format,
 so encryption tests can assert exact on-disk layout.
 
 @param filePath - Absolute config file path.
 
 @param encryptionKey - Key the file was written with.
 
 @param encryptionAlgorithm - Algorithm the file was written with.
 
 @returns Decrypted serialized store text.
 
 @example
 ```ts
 decryptConfigFile({
   filePath: config.path,
   encryptionKey: 'k',
   encryptionAlgorithm: 'aes-256-cbc',
 });
 ```
 */
export function decryptConfigFile({
  filePath,
  encryptionKey,
  encryptionAlgorithm,
}: {
  readonly filePath: string;
  readonly encryptionKey: EncryptionKey;
  readonly encryptionAlgorithm: EncryptionAlgorithm;
},): string {
  /**
   Raw file bytes as written by the store.
   */
  const data = new Uint8Array(readFileSync(filePath,),);
  /**
   Initialization vector framing the ciphertext.
   */
  const initializationVector = data.slice(
    0,
    INITIALIZATION_VECTOR_LENGTH,
  );
  /**
   Ciphertext region following the separator byte.
   */
  const payload = data.slice(INITIALIZATION_VECTOR_LENGTH + 1,);
  /**
   Derived per-file cipher key.
   */
  const key = pbkdf2Sync(
    encryptionKey,
    initializationVector,
    PBKDF2_ITERATIONS,
    DERIVED_KEY_LENGTH,
    PBKDF2_DIGEST,
  );
  if (encryptionAlgorithm === 'aes-256-gcm') {
    /**
     GCM decipher authenticated against the trailing tag.
     */
    const decipher = createDecipheriv(
      'aes-256-gcm',
      key,
      initializationVector,
    );
    decipher.setAuthTag(payload.slice(payload.length - AUTHENTICATION_TAG_LENGTH,),);
    return bytesToString(
      concatBytes([
        decipher.update(payload.slice(
          0,
          payload.length - AUTHENTICATION_TAG_LENGTH,
        ),),
        decipher.final(),
      ],),
    );
  }
  /**
   Block or stream decipher for the unauthenticated algorithms.
   */
  const decipher = createDecipheriv(
    encryptionAlgorithm,
    key,
    initializationVector,
  );
  return bytesToString(
    concatBytes([
      decipher.update(payload,),
      decipher.final(),
    ],),
  );
}

/**
 Writes one config file in the legacy encrypted layout,
 where the salt is the initialization vector's text form.
 
 @param filePath - Absolute config file path.
 
 @param encryptionKey - Key to encrypt under.
 
 @param serialized - Serialized store text to protect.
 
 @example
 ```ts
 writeLegacyEncryptedConfigFile({
   filePath: config.path,
   encryptionKey: 'k',
   serialized: '{"theme":"dark"}',
 });
 ```
 */
export function writeLegacyEncryptedConfigFile({
  filePath,
  encryptionKey,
  serialized,
}: {
  readonly filePath: string;
  readonly encryptionKey: string;
  readonly serialized: string;
},): void {
  /**
   Random initialization vector doubling as the legacy salt.
   */
  const initializationVector = randomBytes(INITIALIZATION_VECTOR_LENGTH,);
  /**
   Cipher keyed by the legacy text-form salt.
   */
  const cipher = createCipheriv(
    'aes-256-cbc',
    pbkdf2Sync(
      encryptionKey,
      initializationVector.toString(),
      PBKDF2_ITERATIONS,
      DERIVED_KEY_LENGTH,
      PBKDF2_DIGEST,
    ),
    initializationVector,
  );
  writeFileSync(
    filePath,
    concatBytes([
      initializationVector,
      stringToBytes(':',),
      concatBytes([
        cipher.update(stringToBytes(serialized,),),
        cipher.final(),
      ],),
    ],),
  );
}

/* oxlint-enable no-restricted-syntax/no-sync */

//endregion Encryption fixtures

/**
 Runs every registered fixture cleanup.
 
 Called by tests that need a mid-file teardown;
 the registry also empties at process exit.
 
 @example
 ```ts
 runRegisteredCleanups();
 ```
 */
export function runRegisteredCleanups(): void {
  for (const cleanup of fixtureRegistry)
    cleanup();
  fixtureRegistry.clear();
}

// Keep the process-exit sweep after the registry so cleanup order reads top-down.
process.on(
  'exit',
  function sweepFixtureRegistry(): void {
    for (const cleanup of fixtureRegistry)
      cleanup();
  },
);
