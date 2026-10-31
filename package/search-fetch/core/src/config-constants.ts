/**
 Pi Search Fetch config constants.
 
 @module
 */

import { join, } from 'node:path';

/**
 Environment variable that wins over the encrypted store Exa API key.
 */
const EXA_API_KEY_ENV = 'EXA_API_KEY';

/**
 Environment variable that wins over the encrypted store Linkup API key.
 */
const LINKUP_API_KEY_ENV = 'LINKUP_API_KEY';

/**
 Environment variable that wins over the encrypted store DeepSeek API key.
 */
const DEEPSEEK_API_KEY_ENV = 'DEEPSEEK_API_KEY';

/**
 Directory below home that stores global Pi extension config.
 */
const PI_EXTENSION_CONFIG_DIR: string = join(
  '.pi',
  'agent',
  'extensions',
);

/**
 Pi Search Fetch config filename.
 */
const PI_SEARCH_FETCH_CONFIG_FILE = 'pi-search-fetch.json';

/**
 Legacy Pi Linkup config filename detected by the migration error.
 */
const LEGACY_PI_LINKUP_CONFIG_FILE = 'pi-linkup.json';

/**
 JSON indentation used when rewriting the Pi config during migration.
 */
const CONFIG_JSON_INDENT_SPACES = 2;

/**
 Node error code for missing files.
 */
const FILE_NOT_FOUND_CODE = 'ENOENT';

/**
 Valid top-level config keys; credentials moved to the encrypted store.
 */
const CONFIG_KEYS = [
  'blocklist',
] as const;

/**
 Top-level key fields that require the one-time credential migration.
 */
const DEPRECATED_CREDENTIAL_KEYS = [
  'exaApiKey',
  'linkupApiKey',
] as const;

/**
 Valid top-level config key lookup.
 */
const CONFIG_KEY_SET: ReadonlySet<string> = new Set<string>(CONFIG_KEYS,);

export {
  CONFIG_JSON_INDENT_SPACES,
  CONFIG_KEY_SET,
  DEEPSEEK_API_KEY_ENV,
  DEPRECATED_CREDENTIAL_KEYS,
  EXA_API_KEY_ENV,
  FILE_NOT_FOUND_CODE,
  LEGACY_PI_LINKUP_CONFIG_FILE,
  LINKUP_API_KEY_ENV,
  PI_EXTENSION_CONFIG_DIR,
  PI_SEARCH_FETCH_CONFIG_FILE,
};
