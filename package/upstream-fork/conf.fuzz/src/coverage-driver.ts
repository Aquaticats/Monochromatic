/**
 Deterministic coverage driver:
 exercises the conf-fork public store surface and its error paths with
 fixed inputs,
 so the V8 coverage it produces is reproducible.
 Run under `NODE_V8_COVERAGE` by the `fuzz:coverage` task,
 then summarized by `coverage-report.ts`.
 
 @module
 */

import { writeFileSync, } from 'node:fs';

import {
  type Conf,
  createConf,
} from '@monochromatic-dev/upstream-fork-conf/ts';

import { STORE_DEFAULTS, } from './store-operations.ts';
import { createTempStoreDir, } from './store-adapters.ts';

//region Fixtures

/**
 Fresh item value the round-trip fixtures write.
 */
const FRESH_VALUE = 'dark';

/**
 Value the encrypted-store fixture writes and must read back.
 */
const ENCRYPTED_VALUE = 'encrypted';

/**
 Encryption key the encrypted-store fixture uses.
 */
const ENCRYPTION_KEY = 'coverage-driver-key';

/**
 Garbage ciphertext forcing the decryption-failure path:
 longer than the initialization vector,
 without the separator byte at its fixed offset.
 */
const GARBAGE_CIPHERTEXT = 'garbage-ciphertext-without-separator';

/**
 Project version the migration fixtures migrate to.
 */
const MIGRATED_VERSION = '2.0.0';

/**
 Project version the seed migration fixture starts at.
 */
const SEED_VERSION = '1.0.0';

/**
 Item value the successful migration step writes.
 */
const MIGRATION_VALUE = 'two';

/**
 Failure thrown by the failing migration fixture.
 */
const MIGRATION_FAILURE = new Error('migration boom',);

/* oxlint-disable typescript/no-unsafe-type-assertion -- invalid runtime inputs mirror a mis-typed caller */

/**
 Non-string key fed past the static type,
 driving the invalid-key paths.
 */
const NON_STRING_KEY = 5 as never;

/**
 Non-object `schema` option fed past the static type,
 driving the invalid-schema path.
 */
const NON_OBJECT_SCHEMA = 5 as never;

/**
 Unsupported encryption algorithm fed past the static type,
 driving the invalid-algorithm path.
 */
const UNSUPPORTED_ALGORITHM = 'bogus' as never;

/**
 Non-function callback fed past the static type,
 driving the invalid-callback path.
 */
const NON_FUNCTION_CALLBACK = undefined as never;

/* oxlint-enable typescript/no-unsafe-type-assertion */

/**
 Function value fed past the static type,
 driving the unsupported-value-type paths.
 */
function functionValue(): void {
  return;
}

/**
 Callback attached to `onDidChange` and `onDidAnyChange`,
 observing nothing and keeping the subscription surface exercised.
 */
function observeChange(): void {
  return;
}

/**
 Migration handler marking the seed step without changing items.
 */
function seedMigrationStep(): void {
  return;
}

//endregion Fixtures

//region Helpers

/**
 Runs a thunk that is expected to throw,
 swallowing `Error` outcomes so the driver keeps exercising remaining paths.
 Re-throws anything that is not an `Error`.
 
 @param thunk - Operation expected to throw.
 
 @example
 ```ts
 swallow(function bad() {
   createConf({});
 });
 ```
 */
function swallow(thunk: () => void,): void {
  try {
    thunk();
  }
  catch (error: unknown) {
    if (!(Error.isError(error,)))
      throw error;
  }
}

/**
 Throws when a driver-internal sanity expectation fails,
 keeping stale driver inputs from silently freezing a weaker baseline.
 
 @param holds - Whether the expectation held.
 
 @param label - Name of the expectation that failed.
 
 @throws Error naming the failed expectation.
 
 @example
 ```ts
 ensure(roundTrip === FRESH_VALUE, 'encrypted round-trip');
 ```
 */
function ensure(
  holds: boolean,
  label: string,
): void {
  if (!holds)
    throw new Error(`coverage driver ${label} diverged; driver inputs are stale`,);
}

//endregion Helpers

//region Exercise

/**
 Exercises construction,
 defaults,
 the item surface with its error paths,
 store replacement with bookkeeping preservation,
 item counts and iteration,
 change subscriptions,
 schema validation,
 encryption round-trips,
 and migrations.
 
 @example
 ```ts
 exercise();
 ```
 */
function exercise(): void {
  //region Construction

  swallow(function missingProjectName() {
    createConf({});
  },);
  {
    using dir = createTempStoreDir();
    swallow(function invalidSchema() {
      createConf({
        cwd: dir.path,
        schema: NON_OBJECT_SCHEMA,
      },);
    },);
    swallow(function rootSchemaProperties() {
      createConf({
        cwd: dir.path,
        rootSchema: {
          properties: {},
        },
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

  //endregion Construction

  //region Items

  {
    using dir = createTempStoreDir();
    /**
     Store exercising the item surface.
     */
    const config = createConf({
      cwd: dir.path,
      defaults: STORE_DEFAULTS,
    });

    config.set({
      key: 'theme',
      value: FRESH_VALUE,
    },);
    config.set({
      values: {
        retries: 2,
        'nested.flag': false,
      },
    },);
    ensure(
      config.get('theme',) === FRESH_VALUE,
      'single-set readback',
    );
    ensure(
      config.get({
        key: 'missing',
        defaultValue: 'fallback',
      },) === 'fallback',
      'default readback',
    );
    ensure(
      config.get({
        key: 'missing',
        defaultValue: 'fallback2',
      },) === 'fallback2',
      'object default readback',
    );
    ensure(
      config.has('theme',),
      'probe hit',
    );
    ensure(
      !config.has('missing',),
      'probe miss',
    );

    config.appendToArray({
      key: 'items',
      value: FRESH_VALUE,
    },);
    config.appendToArray({
      key: 'fresh-array',
      value: FRESH_VALUE,
    },);
    config.reset({
      keys: [
        'theme',
        'no-default',
      ],
    },);
    config.reset({
      keys: [],
    },);
    config.delete('retries',);
    config.delete('missing',);

    //region Item error paths

    /**
     Multi-item payload carrying one `undefined` value,
     driving the unsupported-value-type path of multi-`set`.
     */
    const undefinedEntry: Record<string, unknown> = Object.fromEntries([
      [
        'theme',
        undefined,
      ],
    ],);
    /**
     Multi-item payload carrying one function value.
     */
    const functionEntry: Record<string, unknown> = Object.fromEntries([
      [
        'theme',
        functionValue,
      ],
    ],);

    swallow(function missingSingleValue() {
      config.set({
        key: 'theme',
        value: undefined,
      },);
    },);
    swallow(function unsupportedFunctionValue() {
      config.set({
        key: 'theme',
        value: functionValue,
      },);
    },);
    swallow(function unsupportedSymbolValue() {
      config.set({
        key: 'theme',
        value: Symbol('bad',),
      },);
    },);
    swallow(function unsupportedMultiValue() {
      config.set({
        // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- deliberately type-invalid payload exercising the unsupported-value-type path of multi-item set.
        values: undefinedEntry as never,
      },);
    },);
    swallow(function unsupportedFunctionMultiValue() {
      config.set({
        // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- deliberately type-invalid payload exercising the unsupported-value-type path of multi-item set.
        values: functionEntry as never,
      },);
    },);
    swallow(function unsupportedAppendValue() {
      config.appendToArray({
        key: 'items',
        value: undefined,
      },);
    },);
    swallow(function nonArrayAppend() {
      config.appendToArray({
        key: 'theme',
        value: FRESH_VALUE,
      },);
    },);
    swallow(function reservedSingleKey() {
      config.set({
        key: '__internal__',
        value: FRESH_VALUE,
      },);
    },);
    swallow(function reservedPathKey() {
      config.set({
        key: '__internal__.x',
        value: FRESH_VALUE,
      },);
    },);
    swallow(function reservedMultiKey() {
      config.set({
        values: {
          __internal__: FRESH_VALUE,
        },
      },);
    },);
    swallow(function reservedNestedKey() {
      config.set({
        values: {
          nested: {
            __internal__: FRESH_VALUE,
          },
        },
      },);
    },);
    swallow(function invalidSetKey() {
      config.set(NON_STRING_KEY,);
    },);
    swallow(function invalidGetKey() {
      config.get(NON_STRING_KEY,);
    },);

    //endregion Item error paths

    //region Store surface

    config.store = {
      theme: FRESH_VALUE,
    };
    ensure(
      config.size
        === Object.keys(config.store,)
          .length,
      'store size',
    );
    ensure(
      [...config,].length === config.size,
      'store iteration',
    );

    /**
     Raw file contents carrying bookkeeping the replacement must preserve.
     */
    const fileWithBookkeeping = JSON.stringify({
      __internal__: {
        migrations: {
          version: SEED_VERSION,
        },
      },
      theme: FRESH_VALUE,
    },);
    writeFileSync(
      config.path,
      fileWithBookkeeping,
    );
    config.store = {
      theme: FRESH_VALUE,
    };
    ensure(
      config.store
        .theme
        === FRESH_VALUE,
      'store replacement',
    );

    //endregion Store surface

    //region Change subscriptions

    /**
     Unsubscribe handle of a key subscription.
     */
    const stopKeyWatch = config.onDidChange({
      key: 'theme',
      callback: observeChange,
    },);
    /**
     Unsubscribe handle of a whole-store subscription.
     */
    const stopStoreWatch = config.onDidAnyChange(observeChange,);
    config.set({
      key: 'theme',
      value: FRESH_VALUE,
    },);
    stopKeyWatch();
    stopStoreWatch();
    swallow(function invalidCallback() {
      config.onDidAnyChange(NON_FUNCTION_CALLBACK,);
    },);
    config.closeWatcher();

    //endregion Change subscriptions

    config.clear();
  }

  //endregion Items

  //region Schema

  {
    using dir = createTempStoreDir();
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

  //endregion Schema

  //region Encryption

  {
    using dir = createTempStoreDir();
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
    ensure(
      encrypted.get('theme',) === ENCRYPTED_VALUE,
      'encrypted round-trip',
    );
    writeFileSync(
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
    ensure(
      tolerant.get({
        key: 'theme',
        defaultValue: 'cleared',
      },) === 'cleared',
      'cleared invalid config',
    );
  }

  //endregion Encryption

  //region Migrations

  {
    using dir = createTempStoreDir();
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
        '2.0.0': function stepTwo(store: Conf): void {
          store.set({
            key: 'phase',
            value: MIGRATION_VALUE,
          },);
        },
      },
    },);
    ensure(
      migrated.get('phase',) === MIGRATION_VALUE,
      'migration step',
    );
  }

  {
    using dir = createTempStoreDir();
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
          '2.0.0': function failingStep(): void {
            throw MIGRATION_FAILURE;
          },
        },
      },);
    },);
  }

  //endregion Migrations
}

//endregion Exercise

exercise();
