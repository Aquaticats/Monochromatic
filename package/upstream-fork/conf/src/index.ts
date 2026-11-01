/**
 TypeScript fork of [`conf`](https://github.com/sindresorhus/conf) by
 Sindre Sorhus (MIT): a JSON config file store with defaults,
 schema validation,
 dot-notation access,
 optional encryption,
 migrations,
 watching,
 and change events.
 
 Rewritten to this repository's TypeScript standards with upstream `conf`
 15.1.0 behavior preserved. The only API-shape deviations are
 lint-mandated: {@link createConf} is a factory instead of a class,
 multi-argument members take one destructured object,
 and change callbacks receive one change object instead of positional
 pairs.
 
 @example
 ```ts
 import { createConf, } from '\@monochromatic-dev/upstream-fork-conf';
 
 const config = createConf<{ theme: string, }>({
   projectName: 'foo',
   defaults: { theme: 'light', },
 });
 config.set({ key: 'theme', value: 'dark', });
 config.get('theme'); // => 'dark'
 ```
 
 @packageDocumentation
 */

export {
  createConf,
  type Conf,
} from './conf.ts';

export type {
  AjvOptions,
  Options,
  Schema,
  ValueSchema,
} from './options.ts';

export type {
  BeforeEachMigrationCallback,
  BeforeEachMigrationContext,
  Deserialize,
  DotNotationKeyOf,
  DotNotationValueOf,
  Migrations,
  OnDidAnyChangeCallback,
  OnDidChangeCallback,
  PartialObjectDeep,
  Serialize,
  StoreChange,
  Unsubscribe,
  ValueChange,
} from './types.ts';

export {
  DecryptionFailedError,
  InvalidAuthenticationTagError,
  InvalidCallbackError,
  InvalidEncryptionAlgorithmError,
  InvalidKeyError,
  InvalidSchemaError,
  MigrationFailedError,
  MissingProjectNameError,
  MissingProjectVersionError,
  MissingValueError,
  NonArrayValueError,
  ReservedKeyError,
  RootSchemaPropertiesError,
  SchemaViolationError,
  UnsupportedValueTypeError,
} from './errors.ts';

export {
  decryptConfigData,
  encryptSerializedStore,
  type EncryptionAlgorithm,
  type EncryptionKey,
} from './encryption.ts';

export {
  isMissingFileError,
  isRecoverableReadFailure,
} from './store-file.ts';

/**
 @internal Test-fixture helpers for the built artifact;
 not part of the supported store API.
 */
export {
  bytesToString,
  concatBytes,
  stringToBytes,
} from './bytes.ts';

/**
 @internal Key-access primitives for built-artifact tests;
 not part of the supported store API.
 */
export {
  createPlainObject,
  isStoreContentEqual,
  getStoreValue,
  hasStoreValue,
  withStoreValue,
  withoutStoreValue,
} from './store-access.ts';

/**
 @internal Reserved-key primitives for built-artifact tests;
 not part of the supported store API.
 */
export {
  containsReservedKey,
  INTERNAL_KEY,
  isReservedKeyPath,
  MIGRATION_KEY,
} from './internal-key.ts';

/**
 @internal Value-type gate for built-artifact tests;
 not part of the supported store API.
 */
export {
  checkValueType,
  UNSUPPORTED_VALUE_TYPES,
  type UnsupportedValueType,
} from './value-type.ts';

/**
 @internal Path resolution for built-artifact tests;
 not part of the supported store API.
 */
export { configDirectory, } from './config-dir.ts';

/**
 @internal Debounce primitive for built-artifact tests;
 not part of the supported store API.
 */
export {
  debounce,
  type DebouncedCall,
} from './debounce.ts';

/**
 @internal Synchronous file primitives for built-artifact tests;
 not part of the supported store API.
 */
export {
  ensureDirectory,
  pathExists,
  readFileBytes,
  readFileText,
  removeFileIfExists,
  writeFileAtomic,
} from './file-io.ts';

/**
 @internal Options normalization for built-artifact tests;
 not part of the supported store API.
 */
export {
  prepareOptions,
  type PreparedOptions,
} from './prepare-options.ts';

/**
 @internal Encryption primitives for built-artifact tests;
 not part of the supported store API.
 */
export {
  decryptWithSalt,
  deriveKey,
  isSupportedEncryptionAlgorithm,
  SUPPORTED_ENCRYPTION_ALGORITHMS,
} from './encryption.ts';

/**
 @internal Schema validation for built-artifact tests;
 not part of the supported store API.
 */
export {
  captureSchemaDefaults,
  createSchemaValidator,
  type SchemaValidator,
} from './schema.ts';

/**
 @internal File pipeline for built-artifact tests;
 not part of the supported store API.
 */
export {
  createStoreFile,
  type StoreFile,
} from './store-file.ts';

/**
 @internal Migration predicates and runner for built-artifact tests;
 not part of the supported store API.
 */
export {
  applyMigrations,
  isVersionInRangeFormat,
  runMigrationSteps,
  shouldPerformMigration,
} from './migrate.ts';

/**
 @internal Migration host contract for built-artifact tests;
 not part of the supported store API.
 */
export type { MigrationHost, } from './migration-host.ts';

/**
 @internal File watching for built-artifact tests;
 not part of the supported store API.
 */
export {
  createConfigWatcher,
  type ConfigWatcher,
} from './watcher.ts';

/**
 @internal Change subscriptions for built-artifact tests;
 not part of the supported store API.
 */
export {
  subscribeStoreChange,
  subscribeValueChange,
} from './change-notifications.ts';

/**
 @internal Subscription methods for built-artifact tests;
 not part of the supported store API.
 */
export {
  createEventMethods,
  type EventApi,
  type EventContext,
} from './conf-events.ts';

/**
 @internal CRUD methods for built-artifact tests;
 not part of the supported store API.
 */
export {
  createCrud,
  type CrudApi,
  type CrudContext,
  type SetInput,
} from './conf-crud.ts';
