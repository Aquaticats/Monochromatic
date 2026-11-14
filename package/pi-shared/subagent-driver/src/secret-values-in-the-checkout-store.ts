/**
 The checkout's own secret environment: the sops store this repository keeps its values in, read through the reader that owns the
 store's discovery, its identity rules and its sops call.

 The names are read from the ciphertext, which holds them in clear because sops encrypts values rather than keys, and they are
 what the reader is asked for one by one. The names are not secret; the values are, and they are returned to a caller that masks
 with them and keeps them nowhere.

 @module
 */

import { readFile, } from 'node:fs/promises';
import { ENV_PATH_VARIABLE_NAME, } from '@monochromatic-dev/env-local-json-core/ts/constants.ts';
import {
  NO_STORE_FOUND,
  resolveStorePath,
} from '@monochromatic-dev/env-local-json-core/ts/discovery.ts';
import { readEnvLocalJson, } from '@monochromatic-dev/env-local-json-core/ts';
import type { SecretValue, } from './secret-value-masking.ts';
import {
  type EnvironmentOfThisMachine,
  isMissingPathFailure,
  plainCauseOf,
} from './secret-values-in-the-files-of-this-machine.ts';

//region Constants

/**
 The store's own file name, which the reader's discovery looks for.
 */
const STORE_FILE_NAME = '.env.local.json';

/**
 The metadata key a sops store holds beside its values. It is not a variable, and what it holds encrypts nothing.
 */
const SOPS_METADATA_KEY = 'sops';

//endregion Constants

//region Errors

/**
 The checkout's secret environment is there and cannot be read, so this code does not know which values to mask.
 */
export class SecretEnvironmentUnreadableError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'SecretEnvironmentUnreadableError';
}

//endregion Errors

//region Where the store is and what it declares

/**
 The store a checkout declares, or the reason this code cannot learn what it declares.
 */
export type StoreOfTheCheckout =
  | {
    readonly storePath: string;
    readonly names: readonly string[]
  }
    | { readonly absent: true }
    | { readonly cause: string };

/**
 Reads the names a sops store declares, which its ciphertext holds in clear.

 @param checkoutFolder - the checkout whose store is read

 @param environment - the environment naming a store path of its own, when it does

 @param home - home folder bounding discovery

 @returns the store's path and its names, that the checkout has no store, or why the store cannot be read

 @example
 ```ts
 const found = await storeOfTheCheckout({ checkoutFolder: '/repo', environment: process.env, home: '/home/a', },);
 ```
 */
export async function storeOfTheCheckout({
  checkoutFolder,
  environment,
  home,
}: {
  readonly checkoutFolder: string;
  readonly environment: EnvironmentOfThisMachine;
  readonly home: string;
},): Promise<StoreOfTheCheckout> {
  /**
   The store path the environment names, which wins over discovery, as the reader's own rules say.
   */
  const named = environment[ENV_PATH_VARIABLE_NAME];
  /**
   Where discovery looks, which is the checkout this run works in rather than the driver's own folder.
   */
  const storePath = await resolveStorePath({
    cwd: checkoutFolder,
    home,
    ...(((typeof named) === 'string') && (named !== '') ? { path: named, } : {}),
  },);
  if (storePath === NO_STORE_FOUND) {
    return { absent: true, };
  }
  /**
   The store's ciphertext, or why it could not be read.
   */
  const ciphertext = await textOfTheStore({ storePath, },);
  if ('cause' in ciphertext) {
    return { cause: ciphertext.cause, };
  }
  /**
   The store parsed no further than its names, which is all this read wants of it.
   */
  const parsed = parseStore({ ciphertext: ciphertext.text, },);
  if ('cause' in parsed) {
    return { cause: parsed.cause, };
  }
  if (((typeof parsed.parsed) !== 'object') || (parsed.parsed === null)
    || (Array.isArray(parsed.parsed,))) {
    return { cause: `${STORE_FILE_NAME} is not the object a sops store is`, };
  }
  return {
    storePath,
    names: Object.keys(parsed.parsed,)
      .filter(function isNotMetadata(name,) {
        return name !== SOPS_METADATA_KEY;
      },),
  };
}

/**
 Reads a store's ciphertext.

 @param storePath - the store

 @returns its text, or why it could not be read
 */
async function textOfTheStore({ storePath, }: { readonly storePath: string },): Promise<
  | { readonly text: string }
    | { readonly cause: string }
  > {
  try {
    return { text: await readFile(
      storePath,
      'utf8',
    ), };
  }
  catch (error) {
    if (isMissingPathFailure({ error, },)) {
      return { cause: `${storePath} is not there to read`, };
    }
    return { cause: plainCauseOf(error,), };
  }
}

/**
 Parses a store's ciphertext, which is read for its names rather than decrypted here.

 @param ciphertext - the store's text

 @returns what it parsed to, or why it is not the JSON a store is
 */
function parseStore({ ciphertext, }: { readonly ciphertext: string },):
  | { readonly parsed: unknown }
    | { readonly cause: string } {
  try {
    return { parsed: JSON.parse(ciphertext,), };
  }
  catch (error) {
    return { cause: `${STORE_FILE_NAME} is not the JSON a sops store is (${plainCauseOf(error,)})`, };
  }
}

//endregion Where the store is and what it declares

//region Reading its values

/**
 The values a checkout's store declares.
 */
export type ValuesInTheCheckoutStore =
  | {
    readonly values: readonly SecretValue[];
    readonly storePath: string
  }
    | { readonly absent: true };

/**
 Reads the values of a checkout's secret environment.

 @param checkoutFolder - the checkout whose store is read

 @param environment - the environment naming an identity

 @param home - the home folder the default identity file is in

 @returns the store's values, or that the checkout has no store

 @throws SecretEnvironmentUnreadableError when the store is there and one of the steps it needs failed, because a run would
 otherwise start with the values of the repository's secret environment unmasked

 @example
 ```ts
 const read = await valuesInTheCheckoutStore({ checkoutFolder: '/repo', environment: process.env, home: '/home/a', },);
 ```
 */
export async function valuesInTheCheckoutStore({
  checkoutFolder,
  environment,
  home,
}: {
  readonly checkoutFolder: string;
  readonly environment: EnvironmentOfThisMachine;
  readonly home: string;
},): Promise<ValuesInTheCheckoutStore> {
  /**
   What the store's ciphertext names, and where it is.
   */
  const found = await storeOfTheCheckout({
    checkoutFolder,
    environment,
    home,
  },);
  if ('absent' in found) {
    return { absent: true, };
  }
  if ('cause' in found) {
    throw new SecretEnvironmentUnreadableError(found.cause,);
  }
  /**
   The store's values, read by the reader that owns the store rather than by a second copy of its rules.
   */
  const read = await valuesOfTheStore({
    environment,
    home,
    storePath: found.storePath,
    names: found.names,
    checkoutFolder,
  },);
  if ('cause' in read) {
    throw new SecretEnvironmentUnreadableError(
      read.cause,
      { cause: read.failure, },
    );
  }
  return {
    storePath: found.storePath,
    values: read.values,
  };
}

/**
 Reads a store's values with the reader that owns it and names why it could not be read.

 @param environment - the environment naming an identity

 @param home - the home folder the default identity file is in

 @param storePath - the store

 @param names - what the store declares

 @param checkoutFolder - the checkout, which is where the reader reads from

 @returns the values, or why the store could not be read
 */
async function valuesOfTheStore({
  environment,
  home,
  storePath,
  names,
  checkoutFolder,
}: {
  readonly environment: EnvironmentOfThisMachine;
  readonly home: string;
  readonly storePath: string;
  readonly names: readonly string[];
  readonly checkoutFolder: string;
},): Promise<
  | { readonly values: readonly SecretValue[] }
    | {
      readonly cause: string;
      readonly failure: unknown
    }
  > {
  try {
    /**
     What the reader answered for the names the store declares.
     */
    const read = await readEnvLocalJson({
      keys: names,
      path: storePath,
      cwd: checkoutFolder,
      env: environment,
      home,
    },);
    /**
     The values as a list, named as the store names them.
     */
    const values: SecretValue[] = [];
    for (const name of Object.keys(read.values,)) {
      values.push({
        name,
        value: read.values[name] ?? '',
      },);
    }
    return { values, };
  }
  catch (error) {
    return {
      cause: plainCauseOf(error,),
      failure: error,
    };
  }
}

//endregion Reading its values
