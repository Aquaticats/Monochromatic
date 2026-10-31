/**
 Pi Search Fetch config schema and value normalization.
 
 @module
 */

import { basename, } from 'node:path';

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';

import {
  CONFIG_KEY_SET,
  DEPRECATED_CREDENTIAL_KEYS,
} from './config-constants.ts';
import { normalizeBlocklist, } from './domain-policy.ts';
import type {
  ApiKeyResolution,
  ConfigFileShape,
} from './config-types.ts';

//region Errors

/**
 Pi config still carries a credential that now belongs in the encrypted store.
 */
class SearchFetchMigrationRequiredError extends Error {
  /**
   Creates the migration error for one config path.
 
   @param configPath - config path holding the old credential shape
 
   @param keys - old top-level keys that require migration
   */
  public constructor({
    configPath,
    keys,
  }: {
    readonly configPath: string;
    readonly keys: readonly string[];
  },) {
    super(
      `Pi config at ${configPath} still contains ${keys.join(', ',)}. `
        + 'Run `mise run secrets:migrate` to move credentials into .env.local.json, '
        + 'then remove the old fields.',
    );
    this.name = 'SearchFetchMigrationRequiredError';
  }
}

//endregion Errors

//region Schema

/**
 Validate parsed config-file shape.
 
 @param value - parsed JSON value
 
 @param configPath - config path used in diagnostics
 
 @returns config-file shape
 
 @throws when value is not the expected flat object or still has credential fields
 
 @example
 ```ts
 validateConfigShape({ value: {}, configPath: '/tmp/pi-search-fetch.json' });
 ```
 */
function validateConfigShape(
  {
    value,
    configPath,
  }: {
    readonly value: unknown;
    readonly configPath: string;
  },
): ConfigFileShape {
  /**
   Local value for record.
   */
  const record = configRecord({
    value,
    configPath,
  },);
  rejectDeprecatedCredentialKeys({
    value: record,
    configPath,
  },);
  rejectExtraKeys({
    value: record,
    configPath,
    keySet: CONFIG_KEY_SET,
  },);

  /**
   Local destructured value.
   */
  const { blocklist, } = record;
  rejectOptionalNonStringArray({
    value: blocklist,
    configPath,
    key: 'blocklist',
  },);

  return (isStringArray(blocklist,) ? { blocklist, } : {});
}

//endregion Schema

//region Resolution

/**
 Resolve API key precedence: environment, then encrypted store.
 
 @param env - environment values
 
 @param envKey - environment key to check
 
 @param storeValue - optional value from the encrypted store
 
 @returns effective API key resolution
 
 @example
 ```ts
 resolveApiKey({ env: {}, envKey: 'EXA_API_KEY' });
 ```
 */
function resolveApiKey(
  {
    env,
    envKey,
    storeValue,
  }: {
    readonly env: Readonly<NodeJS.ProcessEnv>;
    readonly envKey: string;
    readonly storeValue?: string;
  },
): ApiKeyResolution {
  /**
   Local value for envApiKey.
   */
  const envApiKey = env[envKey]
    ?.trim();
  if ((envApiKey !== undefined) && (envApiKey !== ''))
    return {
      configured: true,
      value: envApiKey,
      origin: 'env',
    };

  /**
   Local value for storeApiKey.
   */
  const storeApiKey = storeValue
    ?.trim();
  if ((storeApiKey === undefined) || (storeApiKey === ''))
    return { configured: false, };
  return {
    configured: true,
    value: storeApiKey,
    origin: 'store',
  };
}

//endregion Resolution

//region Normalization

/**
 Normalize blocklist from config and wrap failures with config context.
 
 @param entries - raw blocklist entries
 
 @param configPath - config path used in diagnostics
 
 @returns normalized host suffix blocklist
 
 @throws when blocklist normalization rejects an entry
 
 @example
 ```ts
 normalizeConfigBlocklist({ entries: [], configPath: '/tmp/pi-search-fetch.json' });
 ```
 */
function normalizeConfigBlocklist(
  {
    entries,
    configPath,
  }: {
    readonly entries: readonly string[];
    readonly configPath: string;
  },
): readonly string[] {
  try {
    return normalizeBlocklist(entries,);
  }
  catch (error: unknown) {
    /**
     Local value for detail.
     */
    const detail = caughtValueText(error,);
    throw new Error(
      `${basename(configPath,)} blocklist normalization failed at ${configPath}: ${detail}`,
      { cause: error, },
    );
  }
}

//endregion Normalization

//region Validation helpers

/**
 Return parsed config root as record or throw schema error.
 
 @param value - parsed JSON value
 
 @param configPath - config path used in diagnostics
 
 @returns config root record
 */
function configRecord(
  {
    value,
    configPath,
  }: {
    readonly value: unknown;
    readonly configPath: string;
  },
): Readonly<Record<string, unknown>> {
  if (!isRecord(value,))
    throw schemaError({
      configPath,
      reason: 'root value must be an object',
    },);
  return value;
}

/**
 Reject credentials that now belong in the encrypted store.
 
 @param value - config root record
 
 @param configPath - config path used in diagnostics
 
 @throws SearchFetchMigrationRequiredError when an old credential field is present
 */
function rejectDeprecatedCredentialKeys(
  {
    value,
    configPath,
  }: {
    readonly value: Readonly<Record<string, unknown>>;
    readonly configPath: string;
  },
): void {
  /**
   Local value for presentKeys.
   */
  const presentKeys = DEPRECATED_CREDENTIAL_KEYS
    .filter(function isPresent(key,) {
      return Object.hasOwn(
        value,
        key,
      );
    },);
  if (presentKeys.length > 0)
    throw new SearchFetchMigrationRequiredError({
      configPath,
      keys: presentKeys,
    },);
}

/**
 Reject unsupported config keys.
 
 @param value - config root record
 
 @param configPath - config path used in diagnostics
 
 @param keySet - supported key lookup
 */
function rejectExtraKeys(
  {
    value,
    configPath,
    keySet,
  }: {
    readonly value: Readonly<Record<string, unknown>>;
    readonly configPath: string;
    readonly keySet: ReadonlySet<string>;
  },
): void {
  /**
   Local value for extraKeys.
   */
  const extraKeys = Object
    .keys(value,)
    .filter(function isExtraKey(key,) {
      return !keySet.has(key,);
    },);
  if (extraKeys.length > 0)
    throw schemaError({
      configPath,
      reason: `unsupported keys: ${extraKeys.join(', ',)}`,
    },);
}

/**
 Require optional value to be string array when present.
 
 @param value - candidate value
 
 @param configPath - config path used in diagnostics
 
 @param key - config key name
 */
function rejectOptionalNonStringArray(
  {
    value,
    configPath,
    key,
  }: {
    readonly value: unknown;
    readonly configPath: string;
    readonly key: string;
  },
): void {
  if ((value !== undefined) && (!isStringArray(value,)))
    throw schemaError({
      configPath,
      reason: `${key} must be an array of strings when present`,
    },);
}

/**
 Build a schema-validation error with config context.
 
 @param configPath - config path used in diagnostics
 
 @param reason - validation failure summary
 
 @returns schema-validation error
 */
function schemaError(
  {
    configPath,
    reason,
  }: {
    readonly configPath: string;
    readonly reason: string;
  },
): Error {
  return new Error(`${basename(configPath,)} schema validation failed at ${configPath}: ${reason}`,);
}

/**
 Return whether value is a non-null object record.
 
 @param value - unknown value
 
 @returns whether value can be read by string keys
 */
function isRecord(value: unknown,): value is Record<string, unknown> {
  return (value !== null)
    && ((typeof value) === 'object')
    && (!Array.isArray(value,));
}

/**
 Return whether value is an array of strings.
 
 @param value - unknown value
 
 @returns whether value is a readonly string array
 */
function isStringArray(value: unknown,): value is readonly string[] {
  return Array.isArray(value,)
    && value.every(function isString(item,) {
      return (typeof item) === 'string';
    },);
}

//endregion Validation helpers

export {
  normalizeConfigBlocklist,
  resolveApiKey,
  SearchFetchMigrationRequiredError,
  validateConfigShape,
};
