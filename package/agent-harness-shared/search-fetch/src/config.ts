/**
 Global Pi Search Fetch configuration loading.
 
 @module
 */

import { homedir, } from 'node:os';
import {
  EnvLocalJsonDecryptError,
  EnvLocalJsonMissingIdentityError,
  EnvLocalJsonPathError,
  EnvLocalJsonPlaintextError,
  EnvLocalJsonReadError,
  EnvLocalJsonShapeError,
  EnvLocalJsonSopsMissingError,
  readEnvLocalJson,
} from '@monochromatic-dev/env-local-json-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import {
  DEEPSEEK_API_KEY_ENV,
  EXA_API_KEY_ENV,
  LINKUP_API_KEY_ENV,
} from './config-constants.ts';
import {
  fileExists,
  readOptionalConfigJson,
} from './config-file.ts';
import {
  configPathForHome,
  legacyConfigPathForHome,
} from './config-paths.ts';
import {
  normalizeConfigBlocklist,
  resolveApiKey,
  SearchFetchMigrationRequiredError,
  validateConfigShape,
} from './config-schema.ts';
import type {
  ConfigJsonReadResult,
  LinkupConfig,
  LinkupConfigSource,
  LoadLinkupConfigOptions,
} from './config-types.ts';

/**
 Logger root for pi-search-fetch after removing the package log shim.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l: linkupLogger, },);
 ```
 */
const linkupLogger = tagged({ tag: 'pi-search-fetch', },);

/**
 Module logger.
 */
const l = tagged({
  tag: 'config',
  l: linkupLogger,
},);

//region Public API

/**
 Load Pi Search Fetch global config.
 
 @param options - optional environment, home, cwd, and store overrides for tests
 
 @returns loaded config with normalized blocklist and API key precedence
 
 @throws when the Pi config has invalid JSON or schema, a legacy credential shape,
 or the requested blocklist cannot be normalized
 
 @example
 ```ts
 loadLinkupConfig();
 ```
 */
async function loadLinkupConfig(options: LoadLinkupConfigOptions = {},): Promise<LinkupConfig> {
  /**
   Logger tagged for config loading.
   */
  const innerL = tagged({
    tag: loadLinkupConfig.name,
    l,
  },);
  /**
   Local value for processHome.
   */
  const processHome = process.env
    .HOME;
  /**
   Local value for home.
   */
  const home = options.home
    ?? processHome
    ?? homedir();
  /**
   Local value for env.
   */
  const env = options.env
    ?? process.env;
  /**
   Local value for cwd.
   */
  const cwd = options.cwd
    ?? process.cwd();
  /**
   Local value for configPath.
   */
  const configPath = configPathForHome({ home, },);
  /**
   Local value for readResult.
   */
  const readResult: ConfigJsonReadResult = await readOptionalConfigJson({ configPath, },);
  /**
   Local value for legacyPath.
   */
  const legacyPath = legacyConfigPathForHome({ home, },);
  if ((!readResult.loaded) && await fileExists({ filePath: legacyPath, },))
    throw new SearchFetchMigrationRequiredError({
      configPath: legacyPath,
      keys: ['apiKey',],
    },);
  /**
   Local value for configFile.
   */
  const configFile = readResult.loaded
    ? validateConfigShape({
      value: readResult.value,
      configPath,
    },)
    : {};
  /**
   Local value for blocklist.
   */
  const blocklist = normalizeConfigBlocklist({
    entries: configFile.blocklist ?? [],
    configPath,
  },);
  /**
   Requested provider keys that the environment did not supply.
   */
  const missingKeys = [
    EXA_API_KEY_ENV,
    LINKUP_API_KEY_ENV,
    DEEPSEEK_API_KEY_ENV,
  ].filter(function isMissingFromEnv(envKey,) {
    /**
     Local value for value.
     */
    const value = env[envKey]
      ?.trim();
    return (value === undefined) || (value === '');
  },);
  /**
   Store values and facts, either read now or absent without discovery.
   */
  const store = missingKeys.length === 0
    ? EMPTY_STORE
    : await resolveStoreValues({
      missingKeys,
      cwd,
      env,
      home,
      ...(options.envLocalJsonPath === undefined
        ? {}
        : { envLocalJsonPath: options.envLocalJsonPath, }),
      ...(options.signal === undefined ? {} : { signal: options.signal, }),
    },);
  if (store.error !== undefined)
    innerL.warn(`encrypted store unavailable: ${store.error}`,);
  /**
   Local value for exaApiKey.
   */
  const exaApiKey = resolveApiKey({
    env,
    envKey: EXA_API_KEY_ENV,
    ...(store.values[EXA_API_KEY_ENV] === undefined
      ? {}
      : { storeValue: store.values[EXA_API_KEY_ENV], }),
  },);
  /**
   Local value for linkupApiKey.
   */
  const linkupApiKey = resolveApiKey({
    env,
    envKey: LINKUP_API_KEY_ENV,
    ...(store.values[LINKUP_API_KEY_ENV] === undefined
      ? {}
      : { storeValue: store.values[LINKUP_API_KEY_ENV], }),
  },);
  /**
   Local value for deepseekApiKey.
   */
  const deepseekApiKey = resolveApiKey({
    env,
    envKey: DEEPSEEK_API_KEY_ENV,
    ...(store.values[DEEPSEEK_API_KEY_ENV] === undefined
      ? {}
      : { storeValue: store.values[DEEPSEEK_API_KEY_ENV], }),
  },);
  /**
   Source metadata, with optional store facts only when discovery ran.
   */
  const source: LinkupConfigSource = {
    path: configPath,
    loaded: readResult.loaded,
    ...(store.path === undefined ? {} : { storePath: store.path, }),
    ...(store.loaded ? { storeLoaded: true, } : {}),
    ...(store.error === undefined ? {} : { storeError: store.error, }),
  };

  innerL.debug(
    `loaded pi-search-fetch config from ${configPath}; present=${String(readResult.loaded,)}${store.path === undefined ? '' : `; store=${store.path}`}${store.error === undefined ? '' : '; store unavailable'}`,
  );
  return {
    ...(deepseekApiKey.configured ? { deepseekApiKey: deepseekApiKey.value, } : {}),
    ...(exaApiKey.configured ? { exaApiKey: exaApiKey.value, } : {}),
    ...(linkupApiKey.configured ? { linkupApiKey: linkupApiKey.value, } : {}),
    ...(deepseekApiKey.configured ? { deepseekApiKeyOrigin: deepseekApiKey.origin, } : {}),
    ...(exaApiKey.configured ? { exaApiKeyOrigin: exaApiKey.origin, } : {}),
    ...(linkupApiKey.configured ? { linkupApiKeyOrigin: linkupApiKey.origin, } : {}),
    blocklist,
    source,
  };
}

//endregion Public API

//region Helpers

/**
 Store facts returned by a read attempt.
 */
type StoreResolution = {
  /**
   Fill-in values keyed by environment name.
   */
  readonly values: Readonly<Record<string, string>>;
  /**
   Canonical store path when the store loaded.
   */
  readonly path?: string;
  /**
   Whether the store was discovered and decrypted.
   */
  readonly loaded: boolean;
  /**
   Value-free diagnostic when the store was expected but unusable.
   */
  readonly error?: string;
};

/**
 Store result when the environment already supplied every requested key.
 */
const EMPTY_STORE: StoreResolution = {
  values: {},
  loaded: false,
};

/**
 Resolve store values, degrading only for the reader's expected failures.
 
 @param missingKeys - provider keys the environment did not supply
 
 @param cwd - directory used for store discovery
 
 @param env - environment used for identity and precedence
 
 @param home - home directory used by the identity resolver
 
 @param envLocalJsonPath - optional explicit store path
 
 @param signal - optional cancellation signal
 
 @returns fill-in values and store facts
 
 @throws when the reader raises an unexpected error
 */
async function resolveStoreValues(
  {
    missingKeys,
    cwd,
    env,
    home,
    envLocalJsonPath,
    signal,
  }: {
    readonly missingKeys: readonly string[];
    readonly cwd: string;
    readonly env: Readonly<NodeJS.ProcessEnv>;
    readonly home: string;
    readonly envLocalJsonPath?: string;
    readonly signal?: AbortSignal;
  },
): Promise<StoreResolution> {
  try {
    /**
     Local value for result.
     */
    const result = await readEnvLocalJson({
      keys: missingKeys,
      cwd,
      env,
      home,
      ...(envLocalJsonPath === undefined ? {} : { path: envLocalJsonPath, }),
      ...(signal === undefined ? {} : { signal, }),
    },);
    return {
      values: result.values,
      loaded: result.loaded,
      ...(result.path === undefined ? {} : { path: result.path, }),
    };
  }
  catch (error: unknown) {
    if (!isExpectedStoreError(error,))
      throw error;
    return {
      values: {},
      loaded: false,
      error: error.message,
    };
  }
}

/**
 Report whether an error is one of the store reader's expected failures.
 
 @param error - caught value
 
 @returns whether the loader may degrade to an unconfigured store
 
 @example
 ```ts
 isExpectedStoreError(new EnvLocalJsonPlaintextError({ path: '/tmp/store' }));
 ```
 */
function isExpectedStoreError(error: unknown,): error is Error {
  return (error instanceof EnvLocalJsonPathError)
    || (error instanceof EnvLocalJsonReadError)
    || (error instanceof EnvLocalJsonPlaintextError)
    || (error instanceof EnvLocalJsonSopsMissingError)
    || (error instanceof EnvLocalJsonMissingIdentityError)
    || (error instanceof EnvLocalJsonDecryptError)
    || (error instanceof EnvLocalJsonShapeError);
}

//endregion Helpers

export {
  loadLinkupConfig,
};
export type {
  LinkupConfig,
  LinkupConfigSource,
  LoadLinkupConfigOptions,
} from './config-types.ts';
