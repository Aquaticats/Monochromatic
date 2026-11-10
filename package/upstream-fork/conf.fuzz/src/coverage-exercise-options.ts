/**
 Coverage driver exercise blocks for the store options:
 construction guards,
 schema validation,
 encryption round-trips and their failure path,
 and migrations.

 Each assertion is a fixed input, so the store paths it reaches and the V8
 coverage they produce are reproducible.

 @module
 */

import { writeFile, } from 'node:fs/promises';

import {
  type Conf,
  createConf,
} from '@monochromatic-dev/upstream-fork-conf/ts';

import {
  ENCRYPTED_VALUE,
  ENCRYPTION_KEY,
  FORBIDDEN_ROOT_SCHEMA,
  FRESH_VALUE,
  GARBAGE_CIPHERTEXT,
  MIGRATED_VERSION,
  MIGRATION_VALUE,
  NON_OBJECT_SCHEMA,
  SEED_VERSION,
  UNSUPPORTED_ALGORITHM,
  ensure,
  failingStep,
  seedMigrationStep,
  swallow,
} from './coverage-fixtures.ts';
import { createTempStoreDir, } from './store-fixture.ts';

/**
 Exercises the construction guards:
 a missing project name,
 a non-object schema,
 a root schema carrying `properties`,
 an unsupported encryption algorithm,
 and migrations without a project version.
 
 @example
 ```ts
 await exerciseConstruction();
 ```
 */
export async function exerciseConstruction(): Promise<void> {
  swallow(function missingProjectName() {
    createConf({});
  },);
  /**
   Disposable directory holding the rejected configurations' paths.
   */
  await using dir = await createTempStoreDir();
  swallow(function invalidSchema() {
    createConf({
      cwd: dir.path,
      schema: NON_OBJECT_SCHEMA,
    },);
  },);
  swallow(function rootSchemaProperties() {
    createConf({
      cwd: dir.path,
      rootSchema: FORBIDDEN_ROOT_SCHEMA,
    },);
  },);
  swallow(function invalidEncryptionAlgorithm() {
    createConf({
      cwd: dir.path,
      encryptionKey: ENCRYPTION_KEY,
      encryptionAlgorithm: UNSUPPORTED_ALGORITHM,
    },);
  },);
  swallow(function missingProjectVersion() {
    createConf({
      cwd: dir.path,
      migrations: {
        '1.0.0': seedMigrationStep,
      },
    },);
  },);
}

/**
 Exercises schema validation:
 a conforming write and one the schema rejects.
 
 @example
 ```ts
 await exerciseSchema();
 ```
 */
export async function exerciseSchema(): Promise<void> {
  /**
   Disposable directory holding this store's config file.
   */
  await using dir = await createTempStoreDir();
  /**
   Store validating writes against a small schema.
   */
  const schemaStore = createConf({
    cwd: dir.path,
    schema: {
      theme: {
        type: 'string',
        default: 'light',
      },
    },
  });
  schemaStore.set({
    key: 'theme',
    value: FRESH_VALUE,
  },);
  swallow(function schemaViolation() {
    schemaStore.set({
      key: 'theme',
      value: 5,
    },);
  },);
}

/**
 Exercises authenticated encryption:
 a round-trip,
 the decryption-failure path on unreadable contents,
 and a store configured to clear rather than throw.
 
 @example
 ```ts
 await exerciseEncryption();
 ```
 */
export async function exerciseEncryption(): Promise<void> {
  /**
   Disposable directory holding this store's config file.
   */
  await using dir = await createTempStoreDir();
  /**
   Store persisted through the authenticated encryption wire format.
   */
  const encrypted = createConf({
    cwd: dir.path,
    encryptionKey: ENCRYPTION_KEY,
    encryptionAlgorithm: 'aes-256-gcm',
  });
  encrypted.set({
    key: 'theme',
    value: ENCRYPTED_VALUE,
  },);
  ensure({
    holds: encrypted.get('theme',) === ENCRYPTED_VALUE,
    label: 'encrypted round-trip',
  },);
  await writeFile(
    encrypted.path,
    GARBAGE_CIPHERTEXT,
  );
  swallow(function decryptionFailure() {
    encrypted.get('theme',);
  },);
  /**
   Store configured to clear rather than throw on unreadable contents.
   */
  const tolerant = createConf({
    cwd: dir.path,
    encryptionKey: ENCRYPTION_KEY,
    encryptionAlgorithm: 'aes-256-gcm',
    clearInvalidConfig: true,
  });
  ensure({
    holds: tolerant.get({
      key: 'theme',
      defaultValue: 'cleared',
    },) === 'cleared',
    label: 'cleared invalid config',
  },);
}

/**
 Migration step recording the migrated version in the seeded item.
 
 @param store - Store the migration run hands the step.
 
 @example
 ```ts
 migratedStep(createConf({ cwd }));
 ```
 */
function migratedStep(store: Conf,): void {
  store.set({
    key: 'phase',
    value: MIGRATION_VALUE,
  },);
}

/**
 Exercises migrations:
 one successful step over a seeded file, and a failing step.
 
 @example
 ```ts
 await exerciseMigrations();
 ```
 */
export async function exerciseMigrations(): Promise<void> {
  /**
   Disposable directory holding the seeded config file.
   */
  await using dir = await createTempStoreDir();
  /**
   Seed store whose file exists without recorded bookkeeping,
   so the migration run below covers the unversioned-file path.
   */
  const seed = createConf({
    cwd: dir.path,
    projectVersion: SEED_VERSION,
  });
  seed.set({
    key: 'phase',
    value: 'old',
  },);

  /**
   Store migrating the seeded file up to {@link MIGRATED_VERSION}.
   */
  const migrated = createConf({
    cwd: dir.path,
    projectVersion: MIGRATED_VERSION,
    migrations: {
      '1.0.0': seedMigrationStep,
      '2.0.0': migratedStep,
    },
  },);
  ensure({
    holds: migrated.get('phase',) === MIGRATION_VALUE,
    label: 'migration step',
  },);
}

/**
 Exercises the failing-migration path over a seeded file.
 
 @example
 ```ts
 await exerciseFailingMigration();
 ```
 */
export async function exerciseFailingMigration(): Promise<void> {
  /**
   Disposable directory holding the seeded config file.
   */
  await using dir = await createTempStoreDir();
  /**
   Seed store for the failing-migration path.
   */
  const seed = createConf({
    cwd: dir.path,
    projectVersion: SEED_VERSION,
  });
  seed.set({
    key: 'phase',
    value: 'old',
  },);
  swallow(function failingMigration() {
    createConf({
      cwd: dir.path,
      projectVersion: MIGRATED_VERSION,
      migrations: {
        '2.0.0': failingStep,
      },
    },);
  },);
}
