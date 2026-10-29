/**
 The config store factory.
 
 Rewrites upstream `conf` 15.1.0's `Conf` class as {@link createConf},
 a factory returning one frozen store object,
 because this repository bans classes for long-lived stateful objects.
 Behavior,
 error texts,
 and the file format stay upstream's:
 a JSON config file with defaults,
 schema validation,
 dot-notation access,
 optional encryption,
 migrations,
 watching,
 and change events.
 
 @module
 */

import path from 'node:path';

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import {
  tagged,
  type Logger,
} from '@monochromatic-dev/logger-core/ts';

import {
  createCrud,
  type CrudApi,
} from './conf-crud.ts';
import {
  createEventMethods,
  type EventApi,
} from './conf-events.ts';
import {
  createPlainObject,
  getStoreValue,
  hasStoreValue,
  isStoreContentEqual,
  withStoreValue,
} from './store-access.ts';
import {
  INTERNAL_KEY,
  MIGRATION_KEY,
  isReservedKeyPath,
} from './internal-key.ts';
import {
  createStoreFile,
  isMissingFileError,
  type StoreFile,
} from './store-file.ts';
import { applyMigrations, } from './migrate.ts';
import {
  prepareOptions,
  type PreparedOptions,
} from './prepare-options.ts';
import { createSchemaValidator, } from './schema.ts';
import {
  createConfigWatcher,
  type ConfigWatcher,
} from './watcher.ts';
import type { MigrationHost, } from './migration-host.ts';
import type { Conf, } from './conf-type.ts';
import type { Options, } from './options.ts';

//region Helpers

/**
 Resolves the config file path from prepared options,
 with upstream `conf`'s extension handling.
 
 @param options - Prepared store options.
 
 @returns Absolute config file path.
 
 @example
 ```ts
 resolveConfigPath(prepared); // => '/home/me/.config/foo-nodejs/config.json'
 ```
 */
function resolveConfigPath<T extends Record<string, unknown>>(options: PreparedOptions<T>,): string {
  /**
   Extension suffix including its leading dot;
   empty when the extension is empty.
   */
  const extensionSuffix = options.fileExtension === '' ? '' : `.${options.fileExtension}`;
  return path.resolve(
    options.cwd,
    `${options.configName}${extensionSuffix}`,
  );
}

/**
 Mutable per-store runtime state shared by the store's closures.
 
 @example
 ```ts
 const state: StoreState<Record<string, unknown>> = { isInMigration: false, };
 ```
 */
type StoreState<T extends Record<string, unknown>> = {
  /**
   While true,
   reads and writes skip schema validation,
   as upstream `conf` does during migrations.
   */
  isInMigration: boolean;
  /**
   Memoized user store when `cache` is enabled.
   */
  cachedStore?: T;
  /**
   Running file watcher when `watch` is enabled.
   */
  watcher?: ConfigWatcher;
};

/**
 Marks the migration window so validation is bypassed while steps run and
 restored when the scope ends.
 
 @param state - Store state whose flag is managed.
 
 @returns Disposable resetting the flag at scope exit.
 
 @example
 ```ts
 {
   using _scope = createMigrationScope(state);
   applyMigrations({ host, migrations, projectVersion: '1.0.0', });
 }
 ```
 */
function createMigrationScope<T extends Record<string, unknown>>(state: StoreState<T>,): {
  readonly [Symbol.dispose]: () => void;
} {
  state.isInMigration = true;
  return {
    [Symbol.dispose]: function leaveMigrationScope(): void {
      state.isInMigration = false;
    },
  };
}

/**
 Serializes the store like upstream `conf`'s default:
 indented JSON.
 
 @param value - Store object to serialize.
 
 @returns JSON text written to the config file.
 
 @example
 ```ts
 defaultSerialize({ theme: 'dark', });
 ```
 */
function defaultSerialize(value: unknown,): string {
  return JSON.stringify(
    value,
    undefined,
    '\t',
  );
}

//endregion Helpers

//region Factory

/**
 Creates one config store bound to a config file.
 
 @param options - Store configuration;
 pass `cwd` or `projectName` to locate the file.
 
 @returns Frozen store object persisting to the resolved config file.
 
 @throws InvalidEncryptionAlgorithmError when `encryptionAlgorithm` names an
 unsupported algorithm.
 
 @throws MissingProjectNameError when neither `cwd` nor `projectName`
 resolves a config directory.
 
 @throws MissingProjectVersionError when `migrations` is set without
 `projectVersion`.
 
 @example
 ```ts
 const config = createConf({
   projectName: 'foo',
   defaults: { theme: 'light', },
 });
 ```
 */
export function createConf<T extends Record<string, unknown> = Record<string, unknown>>(options: Options<T> = {},): Conf<T> {
  /**
   Options with defaults filled and paths resolved.
   */
  const prepared = prepareOptions(options,);
  /**
   Schema validator carrying the schema's declared defaults.
   */
  const validator = createSchemaValidator(prepared,);
  /**
   Logger wrapped with this factory's name so store diagnostics name their
   origin.
   */
  const log: Logger = prepared.logger ?? tagged({
    tag: createConf.name,
  },);
  /**
   Mutable runtime state shared by the closures below.
   */
  const state: StoreState<T> = {
    isInMigration: false,
  };
  /**
   Absolute config file path.
   */
  const configPath = resolveConfigPath(prepared,);
  /**
   Deserializes config text with JSON when no custom deserializer is set.
   
   @param text - Config file text to parse.
   
   @returns Parsed store contents.
   */
  function deserializeWithJson(text: string,): T {
    // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- JSON.parse's dynamic output becomes the caller's store type; no narrower static type exists for parsed JSON.
    return JSON.parse(text,) as T;
  }
  /**
   Validation hook honoring the migration bypass,
   mirroring upstream `conf`'s `#isInMigration` gate.
   
   @param data - Parsed store contents to validate.
   */
  function validateParsedStore(data: unknown,): void {
    if (state.isInMigration)
      return;
    validator.validate(data,);
  }
  /**
   Read/write pipeline over the config file.
   */
  const storeFile = createStoreFile<T>({
    path: configPath,
    ...(prepared.encryptionKey === undefined ? {} : { encryptionKey: prepared.encryptionKey, }),
    encryptionAlgorithm: prepared.encryptionAlgorithm,
    serialize: prepared.serialize ?? defaultSerialize,
    deserialize: prepared.deserialize ?? deserializeWithJson,
    clearInvalidConfig: prepared.clearInvalidConfig,
    configFileMode: prepared.configFileMode,
    validate: validateParsedStore,
    logger: log,
  },);
  /**
   Event target backing the change subscriptions.
   */
  const events = new EventTarget();
  /**
   Default values:
   schema defaults overwritten by `defaults`,
   exactly as upstream `conf` merges them.
   */
  const defaultValues: Record<string, unknown> = {
    ...validator.schemaDefaults,
    ...prepared.defaults,
  };

  /**
   Drops the memoized store so the next read hits the file.
   */
  function dropCache(): void {
    delete state.cachedStore;
  }

  /**
   Reads the user-visible store:
   file contents validated and stripped of reserved bookkeeping keys.
   
   @returns User-visible store contents.
   */
  function readUserStore(): T {
    /**
     File contents including bookkeeping.
     */
    const store = storeFile.readStore();
    for (const key of Object.keys(store)) {
      if (isReservedKeyPath(key,))
        Reflect.deleteProperty(
          store,
          key,
        );
    }
    // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- file contents are parsed into a generic dictionary and surfaced as the caller's store type; the shape proof lives with the schema validator.
    return store as T;
  }

  /**
   Reads the whole store,
   through the cache when caching is enabled.
   
   @returns Whole user-visible store.
   */
  function getStore(): T {
    if (prepared.cache !== true)
      return readUserStore();
    state.cachedStore ??= readUserStore();
    return state.cachedStore;
  }

  /**
   Writes raw store contents,
   dropping the cache first like upstream `conf`'s `_write`.
   
   @param store - Store contents to persist.
   */
  function writeStore(store: Record<string, unknown>,): void {
    dropCache();
    storeFile.writeStore(store,);
  }

  /**
   Merges the reserved bookkeeping key back into a replacement store,
   leaving the caller's object untouched.
   
   @param value - Replacement store from the caller.
   
   @returns Store to persist,
   carrying preserved bookkeeping.
   */
  function preserveInternalKey(value: Record<string, unknown>,): Record<string, unknown> {
    if (hasStoreValue({
      store: value,
      key: INTERNAL_KEY,
      accessPropertiesByDotNotation: true,
    },))
      return value;
    try {
      /**
       Current file contents carrying any bookkeeping to preserve.
       */
      const currentStore = storeFile.readParsedFile();
      if (!hasStoreValue({
        store: currentStore,
        key: INTERNAL_KEY,
        accessPropertiesByDotNotation: true,
      },))
        return value;
      /**
       Caller value copy carrying the preserved bookkeeping.
       */
      const merged = createPlainObject();
      Object.assign(
        merged,
        value,
      );
      return withStoreValue({
        store: merged,
        key: INTERNAL_KEY,
        value: getStoreValue({
          store: currentStore,
          key: INTERNAL_KEY,
          accessPropertiesByDotNotation: true,
        },),
        accessPropertiesByDotNotation: true,
      },);
    }
    catch (error) {
      if (!isMissingFileError(error,))
        log.debug(`internal-data preservation skipped: ${caughtValueText(error,)}`,);
      return value;
    }
  }

  /**
   Replaces the whole store the way the `store` setter does:
   preserve bookkeeping,
   validate,
   persist,
   dispatch `change`.
   
   @param value - Replacement store contents.
   */
  function assignStore(value: Record<string, unknown>,): void {
    storeFile.ensureDirectory();
    /**
     Value actually persisted.
     */
    const valueToWrite = preserveInternalKey(value,);
    if (!state.isInMigration)
      validateParsedStore(valueToWrite,);
    writeStore(valueToWrite,);
    events.dispatchEvent(new Event('change',),);
  }

  /**
   Initializes a non-migrating store:
   merge defaults,
   validate,
   and persist only when the merge changed something.
   */
  function initializeStore(): void {
    /**
     File contents before the defaults merge.
     */
    const fileStore = storeFile.readStore();
    /**
     File contents with defaults filled in for missing keys.
     */
    const storeWithDefaults = Object.assign(
      createPlainObject(),
      prepared.defaults ?? {},
      fileStore,
    );
    validateParsedStore(storeWithDefaults,);
    if (!isStoreContentEqual({
      left: fileStore,
      right: storeWithDefaults,
    },))
      assignStore(storeWithDefaults,);
  }

  /**
   CRUD methods bound to this store's context.
   */
  const crud: CrudApi = createCrud<T>({
    options: prepared,
    defaultValues,
    storeFile,
    getStore,
    assignStore,
  },);
  /**
   Change-subscription methods bound to this store's context.
   */
  const eventMethods: EventApi = createEventMethods<T>({
    events,
    getStore,
    readValue: function readValue(key: string,): unknown {
      return crud.get(key,);
    },
  },);

  /**
   The frozen store object handed to the caller.
   */
  const conf = {
    path: configPath,
    events,
    /**
     Item count of the user-visible store.
     */
    get size(): number {
      return Object.keys(getStore(),)
        .length;
    },
    /**
     The whole user-visible store;
     assigning replaces everything.
     */
    get store(): T {
      return getStore();
    },
    /**
     Replaces the whole store contents.
     */
    set store(value: T) {
      assignStore(value,);
    },
    get: crud.get,
    set: crud.set,
    has: crud.has,
    appendToArray: crud.appendToArray,
    reset: crud.reset,
    delete: crud.delete,
    clear: crud.clear,
    onDidChange: eventMethods.onDidChange,
    onDidAnyChange: eventMethods.onDidAnyChange,
    closeWatcher: function closeWatcher(): void {
      state.watcher
        ?.close();
      delete state.watcher;
    },
    * [Symbol.iterator](): IterableIterator<[
      keyof T,
      T[keyof T]
    ]> {
      for (const [key, value,] of Object.entries(getStore(),)) {
        /* oxlint-disable typescript/no-unsafe-type-assertion -- entry pairs come from a generic dictionary and are re-exposed as the caller's key and value types at this iteration boundary. */
        yield [
          key as keyof T,
          value as T[keyof T],
        ];
        /* oxlint-enable typescript/no-unsafe-type-assertion */
      }
    },
    // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- the implementation object satisfies Conf<T> member by member; the overload set is restated at this single factory edge because per-member overload assignability cannot be expressed on an object literal.
  } as unknown as Conf<T>;
  Object.freeze(conf,);

  if (prepared.migrations !== undefined) {
    {
      /**
       Migration window suppressing validation while steps run.
       */
      using _migrationScope = createMigrationScope(state,);
      applyMigrations<T>({
        host: {
          store: conf,
          configFileExists: storeFile.fileExists,
          readRawStore: storeFile.readStore,
          readUserStore: getStore,
          writeUserStore: function writeUserStore(store: T,): void {
            assignStore(store,);
          },
          writeStoreWithoutEvents: writeStore,
          recordVersion: function recordVersion(version: string,): void {
            /**
             File contents awaiting the recorded version.
             */
            const store = storeFile.readStore();
            assignStore(withStoreValue({
              store,
              key: MIGRATION_KEY,
              value: version,
              accessPropertiesByDotNotation: true,
            },),);
          },
        },
        migrations: prepared.migrations,
        projectVersion: prepared.projectVersion ?? '',
        ...(prepared.defaults === undefined ? {} : { defaults: prepared.defaults, }),
        ...(prepared.beforeEachMigration === undefined
          ? {}
          : { beforeEachMigration: prepared.beforeEachMigration, }),
      },);
    }
    validateParsedStore(getStore(),);
  }
  else {
    initializeStore();
  }

  if (prepared.watch === true) {
    storeFile.ensureDirectory();
    if (!storeFile.fileExists())
      writeStore(createPlainObject(),);
    state.watcher = createConfigWatcher({
      path: configPath,
      onChange: function onConfigFileChanged(): void {
        dropCache();
        events.dispatchEvent(new Event('change',),);
      },
      logger: log,
    },);
  }

  return conf;
}

//endregion Factory

export type { Conf, } from './conf-type.ts';
