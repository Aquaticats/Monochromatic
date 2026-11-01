/**
 Shared names and bounds for the encrypted environment store reader.

 @module
 */

//region Store and identity names

/**
 File name of the sops/age-encrypted environment store.
 */
export const STORE_FILE_NAME = '.env.local.json';

/**
 Environment variable that overrides store discovery with an authoritative
 explicit path.
 */
export const ENV_PATH_VARIABLE_NAME = 'ENV_LOCAL_JSON_PATH';

/**
 File name of the age identity inside the mise configuration directory.
 */
export const MISE_AGE_IDENTITY_FILE_NAME = 'age.txt';

/**
 Key groups that prove a parsed store carries sops metadata for a supported
 backend.
 */
export const SOPS_KEY_GROUPS: readonly string[] = [
  'age',
  'pgp',
  'kms',
  'gcp_kms',
  'azure_kv',
  'hc_vault',
];

/**
 Prefix sops leaves on values it could not decrypt. A decrypted store must
 never contain one.
 */
export const SOPS_ENCRYPTED_VALUE_PREFIX = 'ENC[';

//endregion Store and identity names

//region Bounds

/**
 Timeout for one sops child process.
 */
export const SOPS_TIMEOUT_MS = 15_000;

/**
 Maximum bytes captured from one sops child process.
 */
export const MAX_SOPS_OUTPUT_BYTES: number = (2 ** 10) * (2 ** 10)
  * (2 ** 2);

/**
 Environment names copied into the curated child environment. Anything else,
 including credentials for other backends, stays out of the child.
 */
export const CURATED_CHILD_ENV_NAMES: readonly string[] = [
  'PATH',
  'HOME',
  'XDG_CONFIG_HOME',
  'TMPDIR',
  'LANG',
  'LANGUAGE',
  'LC_ALL',
  'LC_CTYPE',
  'MISE_CONFIG_DIR',
  'MISE_DATA_DIR',
  'MISE_STATE_DIR',
  'MISE_CACHE_DIR',
  'MISE_SOPS_STRICT',
  'MISE_TRUSTED_CONFIG_PATHS',
];

/**
 Interactive-only environment names copied for the `edit` CLI so the user's
 chosen editor is preserved.
 */
export const CURATED_EDITOR_ENV_NAMES: readonly string[] = [
  'EDITOR',
  'VISUAL',
] as const;

//endregion Bounds
