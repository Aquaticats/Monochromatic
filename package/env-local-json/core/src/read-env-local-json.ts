/**
 Public reader for the repository's sops/age-encrypted `.env.local.json`.

 The reader returns requested keys only, never mutates `process.env`, and
 never places a secret value in a log or error. When every requested key is
 already non-blank in the supplied environment, the store is not touched.

 @module
 */

import { readFile, } from 'node:fs/promises';
import { homedir, } from 'node:os';

import {
  caughtValueText,
} from '@monochromatic-dev/caught-value-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import {
  buildSopsChildEnv,
  resolveSopsIdentity,
} from './identity.ts';
import { ENV_PATH_VARIABLE_NAME, } from './constants.ts';
import {
  NO_STORE_FOUND,
  resolveStorePath,
} from './discovery.ts';
import {
  hasEveryRequestedKey,
  isNonBlank,
  selectFillInValues,
} from './environment.ts';
import {
  EnvLocalJsonPathError,
  EnvLocalJsonReadError,
} from './errors.ts';
import { resolveSopsExecutable, } from './sops-executable.ts';
import { runSopsDecrypt, } from './sops.ts';
import {
  assertSopsEnvelope,
  parseDecryptedValues,
} from './store-shape.ts';
import type {
  EnvLocalJsonEnvironment,
  ReadEnvLocalJsonOptions,
  ReadEnvLocalJsonResult,
} from './types.ts';

//region Constants

/**
 Logger for reader lifecycle decisions; names keys and paths only.
 */
const readLogger = tagged({ tag: 'envLocalJsonRead', },);

//endregion Constants

//region Result helpers

/**
 Builds the absent result: no store was read.

 @returns result with empty values and `loaded: false`

 @example
 ```ts
 absentResult(); // { values: {}, loaded: false }
 ```
 */
function absentResult(): ReadEnvLocalJsonResult {
  return {
    loaded: false,
    values: {},
  };
}

//endregion Result helpers

//region Ciphertext reading

/**
 Reads ciphertext, distinguishing explicit-path failures from discovered
 read failures.

 @param path - canonical store path

 @param explicit - whether the caller supplied the path

 @returns raw ciphertext

 @throws EnvLocalJsonPathError when an explicit path cannot be read

 @throws EnvLocalJsonReadError when a discovered store cannot be read

 @example
 ```ts
 const ciphertext = await readCiphertext({ path, explicit: false });
 ```
 */
async function readCiphertext({
  path,
  explicit,
}: {
  readonly path: string;
  readonly explicit: boolean;
},): Promise<string> {
  try {
    return await readFile(
      path,
      'utf8',
    );
  }
  catch (error) {
    readLogger.error(
      `env store ${path} could not be read: ${caughtValueText(error,)}`,
    );
    if (explicit) {
      throw new EnvLocalJsonPathError({
        path,
        reason: 'unreadable',
      },);
    }
    throw new EnvLocalJsonReadError({ path, },);
  }
}

//endregion Ciphertext reading

//region Store loading

/**
 Loads and decrypts one discovered store, returning validated top-level
 values.

 @param storePath - canonical store path

 @param explicit - whether the caller supplied the path

 @param env - environment supplying identity candidates

 @param home - home directory for the default mise identity

 @param signal - abort signal forwarded to sops

 @returns validated decrypted store values

 @throws EnvLocalJsonPathError when an explicit path cannot be read

 @throws EnvLocalJsonReadError when a discovered store cannot be read

 @throws EnvLocalJsonPlaintextError when the store is not sops-encrypted

 @throws EnvLocalJsonMissingIdentityError when no identity candidate exists

 @throws EnvLocalJsonSopsMissingError when sops is not on the curated PATH

 @throws EnvLocalJsonDecryptError when sops cannot decrypt the store

 @throws EnvLocalJsonShapeError when decrypted content has the wrong shape

 @example
 ```ts
 const store = await loadStore({ storePath, explicit: false, env, home });
 ```
 */
async function loadStore({
  storePath,
  explicit,
  env,
  home,
  signal,
}: {
  readonly storePath: string;
  readonly explicit: boolean;
  readonly env: EnvLocalJsonEnvironment;
  readonly home: string;
  readonly signal?: AbortSignal;
},): Promise<Readonly<Record<string, string>>> {
  /**
   Store ciphertext read from disk.
   */
  const ciphertext = await readCiphertext({
    explicit,
    path: storePath,
  },);
  assertSopsEnvelope({
    content: ciphertext,
    path: storePath,
  },);

  /**
   Decryption identity resolved after the envelope check.
   */
  const identity = await resolveSopsIdentity({
    env,
    home,
  },);
  /**
   Resolved sops executable.
   */
  const sopsPath = await resolveSopsExecutable({ env, },);
  /**
   Curated environment for the sops child process.
   */
  const childEnv = buildSopsChildEnv({
    env,
    home,
    identity,
  },);
  /**
   Decrypted JSON emitted by sops.
   */
  const decrypted = await runSopsDecrypt({
    childEnv,
    ...(signal === undefined ? {} : { signal, }),
    sopsPath,
    storePath,
  },);
  return parseDecryptedValues({
    content: decrypted,
    path: storePath,
  },);
}

//endregion Store loading

//region Public reader

/**
 Reads requested keys from the sops/age-encrypted environment store.

 Discovery is an explicit path, then the nearest repository root containing
 the store, then a bounded walk from the canonical working directory. A
 missing store, or an environment that already supplies every requested key,
 returns `{ values: {}, loaded: false }` without reading or decrypting
 anything. Values already present in `env` are never returned, so callers
 merge with `{ ...env, ...values }` to keep environment precedence.

 @param keys - environment variable names to return

 @param path - authoritative store path; a missing or unreadable explicit
 path is a hard failure

 @param cwd - discovery start directory; defaults to `process.cwd()`

 @param env - environment to inspect; defaults to `process.env` and is never
 mutated

 @param home - home directory bounding discovery and locating the default
 mise identity; defaults to the runtime home directory

 @param signal - abort signal checked during discovery and forwarded to sops

 @returns fill-in values, the canonical store path when loaded, and whether a
 store was read

 @throws EnvLocalJsonPathError when an explicit path is missing or unreadable,
 or the working directory cannot be canonicalized

 @throws EnvLocalJsonReadError when a discovered store cannot be read

 @throws EnvLocalJsonPlaintextError when the store is not sops-encrypted

 @throws EnvLocalJsonMissingIdentityError when no identity candidate exists

 @throws EnvLocalJsonSopsMissingError when sops is not on the curated PATH

 @throws EnvLocalJsonDecryptError when sops cannot decrypt the store

 @throws EnvLocalJsonShapeError when decrypted content has the wrong shape

 @example
 ```ts
 const { values, loaded } = await readEnvLocalJson({
   keys: ['EXA_API_KEY', 'LINKUP_API_KEY'],
   env: process.env,
 });
 ```
 */
export async function readEnvLocalJson({
  keys,
  path,
  cwd,
  env = process.env,
  home = homedir(),
  signal,
}: ReadEnvLocalJsonOptions,): Promise<ReadEnvLocalJsonResult> {
  signal?.throwIfAborted();

  if (hasEveryRequestedKey({
    env,
    keys,
  },)) {
    readLogger.debug('every requested key is present in the environment',);
    return absentResult();
  }

  /**
   Environment override consulted when no explicit path option was given.
   */
  const environmentPath = env[ENV_PATH_VARIABLE_NAME];
  /**
   Non-blank environment override, or absence.
   */
  const overridePath = isNonBlank(environmentPath,) ? environmentPath : undefined;
  /**
   Authoritative store path: the option first, then the environment override.
   */
  const explicitPath = path ?? overridePath;
  /**
   Canonical store path chosen by explicit-path handling or discovery.
   */
  const storePath = await resolveStorePath({
    ...(cwd === undefined ? {} : { cwd, }),
    home,
    ...(explicitPath === undefined ? {} : { path: explicitPath, }),
    ...(signal === undefined ? {} : { signal, }),
  },);
  if ((typeof storePath) === 'symbol') {
    readLogger.debug('no encrypted env store found',);
    return absentResult();
  }

  /**
   Validated decrypted store values.
   */
  const store = await loadStore({
    env,
    explicit: explicitPath !== undefined,
    home,
    ...(signal === undefined ? {} : { signal, }),
    storePath,
  },);
  /**
   Fill-in values for requested keys absent or blank in the environment.
   */
  const values = selectFillInValues({
    env,
    keys,
    store,
  },);
  readLogger.info(
    `loaded ${String(Object.keys(values,)
      .length,)} requested key(s) from ${storePath}`,
  );
  return {
    loaded: true,
    path: storePath,
    values,
  };
}

//endregion Public reader
