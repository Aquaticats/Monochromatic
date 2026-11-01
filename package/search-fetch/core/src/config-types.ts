/**
 Pi Search Fetch config types.
 
 @module
 */

/**
 Loaded Pi Search Fetch config.
 */
type LinkupConfig = {
  /**
   Optional DeepSeek API key after environment and store precedence are applied.
   */
  readonly deepseekApiKey?: string;
  /**
   Optional Exa API key after environment and store precedence are applied.
   */
  readonly exaApiKey?: string;
  /**
   Optional Linkup API key after environment and store precedence are applied.
   */
  readonly linkupApiKey?: string;
  /**
   Source of the effective DeepSeek key, when configured.
   */
  readonly deepseekApiKeyOrigin?: ApiKeyOrigin;
  /**
   Source of the effective Exa key, when configured.
   */
  readonly exaApiKeyOrigin?: ApiKeyOrigin;
  /**
   Source of the effective Linkup key, when configured.
   */
  readonly linkupApiKeyOrigin?: ApiKeyOrigin;
  /**
   Normalized global host suffix blocklist.
   */
  readonly blocklist: readonly string[];
  /**
   Source metadata useful for diagnostics.
   */
  readonly source: LinkupConfigSource;
};

/**
 Origin of an effective API key.
 */
type ApiKeyOrigin = 'env' | 'store';

/**
 Pi Search Fetch config source metadata.
 */
type LinkupConfigSource = {
  /**
   Absolute Pi config path checked by the loader.
   */
  readonly path: string;
  /**
   Whether the Pi config file existed and loaded successfully.
   */
  readonly loaded: boolean;
  /**
   Absolute encrypted store path, when discovery found one.
   */
  readonly storePath?: string;
  /**
   Whether the encrypted store was read and decrypted successfully.
   */
  readonly storeLoaded?: boolean;
  /**
   Value-free diagnostic explaining why the encrypted store was not read.
   */
  readonly storeError?: string;
};

/**
 Options for loading Pi Search Fetch config.
 */
type LoadLinkupConfigOptions = {
  /**
   Home directory used to resolve global Pi config and the identity.
   */
  readonly home?: string;
  /**
   Environment used for API key precedence and store discovery.
   */
  readonly env?: Readonly<NodeJS.ProcessEnv>;
  /**
   Directory used for store discovery; defaults to the process working
   directory.
   */
  readonly cwd?: string;
  /**
   Explicit encrypted store path; when set it is authoritative.
   */
  readonly envLocalJsonPath?: string;
  /**
   Optional cancellation signal forwarded to the store reader.
   */
  readonly signal?: AbortSignal;
};

/**
 Parsed config-file shape after schema validation.
 */
type ConfigFileShape = {
  /**
   Optional raw host suffix blocklist.
   */
  readonly blocklist?: readonly string[];
};

/**
 Optional parsed config JSON result.
 */
type ConfigJsonReadResult = {
  /**
   Whether config file loaded.
   */
  readonly loaded: false;
} | {
  /**
   Whether config file loaded.
   */
  readonly loaded: true;
  /**
   Parsed JSON value.
   */
  readonly value: unknown;
};

/**
 API key resolution result.
 */
type ApiKeyResolution = {
  /**
   Whether an API key was configured.
   */
  readonly configured: false;
} | {
  /**
   Whether an API key was configured.
   */
  readonly configured: true;
  /**
   Effective API key.
   */
  readonly value: string;
  /**
   Source that supplied the effective key.
   */
  readonly origin: ApiKeyOrigin;
};

/**
 Error object with a Node system code.
 */
type ErrorWithCode = Error & {
  /**
   Node system code.
   */
  readonly code: unknown;
};

export type {
  ApiKeyOrigin,
  ApiKeyResolution,
  ConfigFileShape,
  ConfigJsonReadResult,
  ErrorWithCode,
  LinkupConfig,
  LinkupConfigSource,
  LoadLinkupConfigOptions,
};
