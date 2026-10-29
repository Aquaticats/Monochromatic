/**
 Atomic value writer for the encrypted environment store.

 The writer requires an existing sops-encrypted store, copies it to a
 same-directory temporary file at mode 0600, applies each value through
 `sops set --value-stdin` (the JSON-encoded value never appears on argv), and
 renames the temporary file over the store. A failure removes the temporary
 file and leaves the store untouched.

 @module
 */

import {
  chmod,
  readFile,
  realpath,
  rename,
  rm,
  stat,
  writeFile,
} from 'node:fs/promises';
import { randomUUID, } from 'node:crypto';
import {
  basename,
  dirname,
  isAbsolute,
  join,
  resolve,
} from 'node:path';
import { homedir, } from 'node:os';

import {
  caughtValueText,
} from '@monochromatic-dev/caught-value-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import {
  buildSopsChildEnv,
  resolveSopsIdentity,
} from './identity.ts';
import {
  EnvLocalJsonPathError,
  EnvLocalJsonPlaintextError,
  EnvLocalJsonWriteError,
} from './errors.ts';
import { isNonBlank, } from './environment.ts';
import { resolveSopsExecutable, } from './sops-executable.ts';
import { runSopsSet, } from './sops.ts';
import { assertSopsEnvelope, } from './store-shape.ts';
import type { EnvLocalJsonEnvironment, } from './types.ts';

//region Constants

/**
 Logger for write lifecycle decisions; names paths and keys only.
 */
const writeLogger = tagged({ tag: 'envLocalJsonWrite', },);

/**
 Mode applied to the temporary ciphertext copy and the published store.
 */
const STORE_FILE_MODE = 0o600;

/**
 Temp file infix inserted between the store name and a unique suffix.
 */
const TEMP_FILE_INFIX = '.tmp-';

//endregion Constants

//region Types

/**
 Options for {@link writeEnvLocalJsonValues}.
 */
export type WriteEnvLocalJsonValuesOptions = {
  /**
   Existing encrypted store to rewrite.
   */
  readonly path: string;

  /**
   Values to set, keyed by environment variable name.
   */
  readonly values: Readonly<Record<string, string>>;

  /**
   Environment supplying identity candidates and the curated sops child
   environment. Defaults to `process.env` and is never mutated.
   */
  readonly env?: EnvLocalJsonEnvironment;

  /**
   Home directory used for the default mise identity path. Defaults to the
   runtime home directory.
   */
  readonly home?: string;
};

//endregion Types

//region Validation

/**
 Characters allowed after the first position of an environment variable
 name.
 */
const NAME_CHARACTERS =
  'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_';

/**
 Characters allowed at the first position of an environment variable name.
 */
const NAME_INITIAL_CHARACTERS =
  'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz_';

/**
 Whether a key is a portable environment variable name: an ASCII letter or
 underscore followed by ASCII letters, digits, or underscores.

 The character scans keep quotes and JSON-path delimiters out of the sops
 path expression built for each key.

 @param key - candidate environment variable name

 @returns `true` when the name is safe for a sops JSON-path expression

 @example
 ```ts
 isEnvironmentVariableName('EXA_API_KEY'); // true
 ```
 */
export function isEnvironmentVariableName(key: string,): boolean {
  if (key === '')
    return false;
  /**
   First character of the candidate name.
   */
  const first = key.charAt(0,);
  if (!NAME_INITIAL_CHARACTERS.includes(first,))
    return false;
  for (const character of key.slice(1,)) {
    if (!NAME_CHARACTERS.includes(character,))
      return false;
  }
  return true;
}

/**
 Validates every key and value before the store is copied.

 @param values - values about to be written

 @param path - store path named in failures

 @throws EnvLocalJsonWriteError when a key is not an environment variable
 name or a value is not a non-blank string

 @example
 ```ts
 assertWritableValues({ values: { A: 'x' }, path });
 ```
 */
function assertWritableValues({
  values,
  path,
}: {
  readonly values: Readonly<Record<string, string>>;
  readonly path: string;
},): void {
  for (const [key, value,] of Object.entries(values,)) {
    if (!isEnvironmentVariableName(key,)) {
      throw new EnvLocalJsonWriteError({
        key,
        path,
        reason: 'invalid-key',
      },);
    }
    if (!isNonBlank(value,)) {
      throw new EnvLocalJsonWriteError({
        key,
        path,
        reason: 'blank-value',
      },);
    }
  }
}

/**
 Applies values one at a time because every `sops set` rewrites the same
 temporary file.

 @param entries - key and value pairs in caller order

 @param index - next entry to apply

 @param sopsPath - resolved sops executable

 @param storePath - temporary encrypted store

 @param childEnv - curated sops child environment

 @throws EnvLocalJsonWriteError when a sops invocation fails

 @example
 ```ts
 await applyEntriesSequentially({ entries, index: 0, sopsPath, storePath, childEnv });
 ```
 */
async function applyEntriesSequentially({
  entries,
  index,
  sopsPath,
  storePath,
  childEnv,
}: {
  readonly entries: readonly (readonly [
    string,
    string
  ])[];
  readonly index: number;
  readonly sopsPath: string;
  readonly storePath: string;
  readonly childEnv: Readonly<Record<string, string>>;
},): Promise<void> {
  /**
   Entry applied by this level, when one remains.
   */
  const entry = entries[index];
  if (entry === undefined)
    return;
  /**
   Key and value applied at this level.
   */
  const [key, value,] = entry;
  await runSopsSet({
    childEnv,
    key,
    sopsPath,
    storePath,
    value,
  },);
  return applyEntriesSequentially({
    childEnv,
    entries,
    index: index + 1,
    sopsPath,
    storePath,
  },);
}

//endregion Validation

//region Path preparation

/**
 Canonicalizes the existing store path and confirms it is a regular file.

 @param path - caller-supplied store path

 @param cwd - directory a relative path resolves against

 @returns canonical store path

 @throws EnvLocalJsonPathError when the store is missing, not a file, or
 cannot be resolved

 @example
 ```ts
 const store = await canonicalExistingStore({ path: '.env.local.json', cwd });
 ```
 */
async function canonicalExistingStore({
  path,
  cwd,
}: {
  readonly path: string;
  readonly cwd: string;
},): Promise<string> {
  /**
   Absolute store path resolved against the caller's directory.
   */
  const absolute = isAbsolute(path,)
    ? path
    : resolve(
      cwd,
      path,
    );
  try {
    /**
     Canonical form of the store path.
     */
    const canonical = await realpath(absolute,);
    /**
     Metadata of the canonical store path.
     */
    const metadata = await stat(canonical,);
    if (!metadata.isFile()) {
      throw new EnvLocalJsonPathError({
        path,
        reason: 'not-a-file',
      },);
    }
    return canonical;
  }
  catch (error) {
    if (error instanceof EnvLocalJsonPathError)
      throw error;
    throw new EnvLocalJsonPathError({
      path,
      reason: 'missing',
    },);
  }
}

/**
 Reads and envelope-checks the existing store ciphertext.

 @param path - canonical store path

 @returns ciphertext content

 @throws EnvLocalJsonPathError when the store cannot be read

 @throws EnvLocalJsonPlaintextError when the store is not sops-encrypted

 @example
 ```ts
 const ciphertext = await readCheckedCiphertext({ path });
 ```
 */
async function readCheckedCiphertext({
  path,
}: {
  readonly path: string;
},): Promise<string> {
  try {
    /**
     Raw ciphertext read from the store.
     */
    const ciphertext = await readFile(
      path,
      'utf8',
    );
    assertSopsEnvelope({
      content: ciphertext,
      path,
    },);
    return ciphertext;
  }
  catch (error) {
    if ((error instanceof EnvLocalJsonPlaintextError)
      || (error instanceof EnvLocalJsonPathError)) {
      throw error;
    }
    throw new EnvLocalJsonPathError({
      path,
      reason: 'unreadable',
    },);
  }
}

//endregion Path preparation

//region Atomic write

/**
 Writes requested values to an existing encrypted store atomically.

 @param path - existing encrypted store path

 @param values - values to set; other store keys are preserved

 @param env - identity and curated child environment source

 @param home - home directory for the default mise identity path

 @throws EnvLocalJsonPathError when the store is missing or unreadable

 @throws EnvLocalJsonPlaintextError when the store is not sops-encrypted

 @throws EnvLocalJsonMissingIdentityError when no identity candidate exists

 @throws EnvLocalJsonSopsMissingError when sops is not on the curated PATH

 @throws EnvLocalJsonWriteError when validation, sops set, or the atomic
 rename fails; the temporary file is removed and the store is unchanged

 @example
 ```ts
 await writeEnvLocalJsonValues({
   path: '/repo/.env.local.json',
   values: { EXA_API_KEY: 'replacement' },
   env: process.env,
   home: os.homedir(),
 });
 ```
 */
export async function writeEnvLocalJsonValues({
  path,
  values,
  env = process.env,
  home = homedir(),
}: WriteEnvLocalJsonValuesOptions,): Promise<void> {
  assertWritableValues({
    path,
    values,
  },);

  /**
   Canonical store path being replaced.
   */
  const storePath = await canonicalExistingStore({
    cwd: process.cwd(),
    path,
  },);
  /**
   Existing store ciphertext, validated as a sops envelope.
   */
  const ciphertext = await readCheckedCiphertext({ path: storePath, },);
  /**
   Decryption identity resolved before creating the temporary file.
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
   Curated environment for every sops child process.
   */
  const childEnv = buildSopsChildEnv({
    env,
    home,
    identity,
  },);

  /**
   Same-directory temporary ciphertext path that disappears on both success
   and failure.
   */
  const tempPath = join(
    dirname(storePath,),
    `${basename(storePath,)}${TEMP_FILE_INFIX}${randomUUID()}`,
  );

  /**
   Disposable guard that removes the temporary ciphertext on every exit path.
   */
  await using _temporary = {
    [Symbol.asyncDispose]: async function removeTemporary(): Promise<void> {
      await rm(
        tempPath,
        { force: true, },
      );
    },
  };

  try {
    await writeFile(
      tempPath,
      ciphertext,
      {
        encoding: 'utf8',
        flag: 'wx',
        mode: STORE_FILE_MODE,
      },
    );
    await chmod(
      tempPath,
      STORE_FILE_MODE,
    );
  }
  catch (error) {
    writeLogger.error(
      `could not create temporary store beside ${storePath}: ${caughtValueText(error,)}`,
    );
    throw new EnvLocalJsonWriteError({
      path: storePath,
      reason: 'permissions',
    },);
  }

  await applyEntriesSequentially({
    childEnv,
    entries: Object.entries(values,),
    index: 0,
    sopsPath,
    storePath: tempPath,
  },);

  try {
    await chmod(
      tempPath,
      STORE_FILE_MODE,
    );
    await rename(
      tempPath,
      storePath,
    );
  }
  catch (error) {
    writeLogger.error(
      `could not atomically replace ${storePath}: ${caughtValueText(error,)}`,
    );
    throw new EnvLocalJsonWriteError({
      path: storePath,
      reason: 'rename',
    },);
  }

  writeLogger.info(
    `updated ${String(Object.keys(values,)
      .length,)} key(s) in ${storePath}`,
  );
}

//endregion Atomic write
