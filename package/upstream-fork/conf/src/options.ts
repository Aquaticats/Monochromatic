/**
 Construction options and schema types for the config store.
 
 Option texts and defaults mirror upstream `conf` 15.1.0 so a migrating
 caller's configuration keeps the same meaning; the call shape stays one
 destructured object per this repository's standards.
 
 @module
 */

import type { Options as AjvOptions_, } from 'ajv';
import type { JSONSchema as TypedJSONSchema, } from 'json-schema-typed';
import type { Logger, } from '@monochromatic-dev/logger-core/ts';

import type {
  BeforeEachMigrationCallback,
  Deserialize,
  Migrations,
  Serialize,
} from './types.ts';

//region Schema types

/**
 Typed [JSON Schema](https://json-schema.org) for one store property.
 
 @example
 ```ts
 const valueSchema: ValueSchema = {
   type: 'number',
   maximum: 100,
   minimum: 1,
   default: 50,
 };
 ```
 */
export type ValueSchema = TypedJSONSchema;

/**
 Maps store property names to their JSON Schemas,
 so `get()` infers each key's value type.
 
 @example
 ```ts
 type Store = {
   isEnabled: boolean;
   interval: number;
 };
 
 const schema: Schema<Store> = {
   isEnabled: { type: 'boolean', },
   interval: { type: 'number', },
 };
 ```
 */
export type Schema<T> = {[Property in keyof T]: ValueSchema};

/**
 Options object shape accepted by ajv.
 
 @example
 ```ts
 const ajvOptions: AjvOptions = {
   removeAdditional: true,
 };
 ```
 */
export type AjvOptions = AjvOptions_;

//endregion Schema types

//region Options

/**
 Configuration for one {@link Conf} store.
 
 Every property is optional except the path resolution pair: pass `cwd`, or
 `projectName` (plus `projectVersion` when `migrations` is set).
 
 @example
 ```ts
 const options: Options<Record<string, unknown>> = {
   projectName: 'foo',
   defaults: { theme: 'light', },
 };
 ```
 */
export type Options<T extends Record<string, unknown>> = {
  /**
   Default values for the config items.
   
   **Note:** Values in `defaults` overwrite the `default` key in `schema`.
   */
  defaults?: Readonly<T>;

  /**
   [JSON Schema](https://json-schema.org) to validate the config data.
   
   This becomes the [`properties`](https://json-schema.org/understanding-json-schema/reference/object.html#properties)
   object of the JSON schema: each key names a data property and each value
   validates that property.
   
   **Note:** The `default` value is overwritten by the `defaults` option.
   */
  schema?: Schema<T>;

  /**
   Root-level [JSON Schema](https://json-schema.org/understanding-json-schema/reference) keywords
   such as `additionalProperties` or `patternProperties`,
   for stores whose key names are not known up front.
   
   The `properties` keyword belongs to the `schema` option; `rootSchema`
   must not contain it.
   */
  rootSchema?: Omit<TypedJSONSchema, 'properties'>;

  /**
   [Options passed to ajv](https://ajv.js.org/options.html).
   
   Validation uses [JSON Schema draft-2020-12](https://json-schema.org/draft-2020-12/release-notes)
   with all validation keywords and formats.
   
   **Note:** `allErrors` and `useDefaults` default to `true` but can be
   overridden here.
   */
  ajvOptions?: AjvOptions;

  /**
   Name of the config file,
   without extension.
   
   Useful for multiple config files per app,
   for example one per major version.
   
   @defaultValue 'config'
   */
  configName?: string;

  /**
   **Required unless the `cwd` option is set.**
   
   Usually the `name` field of package.json.
   */
  projectName?: string;

  /**
   **Required when the `migrations` option is set.**
   
   Usually the `version` field of package.json.
   */
  projectVersion?: string;

  /**
   Operations to run on the store whenever the project version is upgraded.
   
   Keys are versions or [semver ranges](https://github.com/npm/node-semver#ranges);
   values are handlers. Bookkeeping lives in the config file under the
   reserved `__internal__` key and is never exposed through `store`, `get`,
   `has`, or iteration.
   
   Migrations do not run for a config file that does not exist yet: the
   store starts at the current project version. A file that exists without a
   recorded version is still migrated.
   */
  migrations?: Migrations<T>;

  /**
   Called before each migration step,
   for logging or preparation.
   */
  beforeEachMigration?: BeforeEachMigrationCallback<T>;

  /**
   **You most likely don't need this.**
   
   Overrides the system default user config directory and the `projectName`
   resolution. The default stores config under the [system user's config directory](https://github.com/sindresorhus/env-paths#pathsconfig);
   set `cwd` to share one store across users or to keep config beside the app.
   */
  cwd?: string;

  /**
   **Not intended for security purposes:**
    the key would be found inside a plain-text Node.js app.
   
   Its use is obscurity; encrypting deters casual edits of the config file.
   
   When set,
   the store is encrypted with the `encryptionAlgorithm` option.
   */
  encryptionKey?: string | Uint8Array | NodeJS.TypedArray | DataView;

  /**
   Encryption algorithm used when `encryptionKey` is set.
   
   Use `aes-256-gcm` for authenticated encryption;
   `aes-256-cbc` and `aes-256-ctr` leave tampering undetected.
   Changing the algorithm makes existing encrypted data unreadable.
   `aes-256-gcm` and `aes-256-ctr` cannot read existing plaintext config;
   `aes-256-cbc` still can.
   
   @defaultValue 'aes-256-cbc'
   */
  encryptionAlgorithm?: 'aes-256-cbc' | 'aes-256-gcm' | 'aes-256-ctr';

  /**
   Extension of the config file.
   
   Mostly unnecessary; useful for shareable save or export files with a
   custom extension.
   
   @defaultValue 'json'
   */
  fileExtension?: string;

  /**
   Clear the config when reading it fails on malformed JSON,
   a schema violation,
   or a decryption failure.
   
   Good for unimportant data that users never hand-edit;
   leave `false` when hand-edited mistakes should surface as errors.
   
   @defaultValue false
   */
  clearInvalidConfig?: boolean;

  /**
   Serializes the store object to the UTF-8 string written to the config
   file.
   
   Useful for formats other than JSON.
   */
  readonly serialize?: Serialize<T>;

  /**
   Deserializes config file text back into the store object.
   
   Useful for formats other than JSON.
   
   @defaultValue JSON.parse
   */
  readonly deserialize?: Deserialize<T>;

  /**
   **You most likely don't need this.**
   
   Suffix appended to `projectName` when resolving the config directory,
   to avoid name conflicts with native apps. Pass `''` to drop the suffix.
   
   @defaultValue 'nodejs'
   */
  readonly projectSuffix?: string;

  /**
   Access nested properties by dot notation.
   
   When `false`,
   the whole key string is treated as one literal key.
   
   @defaultValue true
   */
  readonly accessPropertiesByDotNotation?: boolean;

  /**
   Watch the config file for changes made by other processes and report them
   to `onDidChange` and `onDidAnyChange` subscribers.
   
   @defaultValue false
   */
  readonly watch?: boolean;

  /**
   Keep the store in memory so reads skip reading and parsing the config
   file each time.
   
   The cache is dropped on every write and whenever `watch` reports a change.
   Writes still read the file first so other processes' changes survive,
   but reads only see them with `watch` enabled or after a write. Objects
   handed out by `store` and `get` are the cache itself;
   do not mutate them directly.
   
   @defaultValue false
   */
  readonly cache?: boolean;

  /**
   [Mode](https://en.wikipedia.org/wiki/File-system_permissions#Numeric_notation) used when creating the config file,
   reduced by the process umask.
   
   @defaultValue 0o666
   */
  readonly configFileMode?: number;

  /**
   Logger for corruption clearing,
   decryption fallbacks,
   migration failures,
   and watch events.
   
   @defaultValue module-logger `logger` tagged per module
   */
  readonly logger?: Logger;
};

//endregion Options
