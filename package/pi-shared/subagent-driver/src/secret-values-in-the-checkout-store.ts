/**
 The checkout's own secret environment: the sops store this repository keeps its values in, read through the reader that owns the
 store's discovery, its identity rules and its sops call.

 The names are read from the ciphertext, which holds them in clear because sops encrypts values rather than keys, and they are
 what the reader is asked for one by one. The names are not secret; the values are, and they are returned to a caller that masks
 with them and keeps them nowhere.

 @module
 */

import { join, } from 'node:path';
import { readOrdinaryFile, } from './read-ordinary-file.ts';
import { openAnOrdinaryFile, } from './ordinary-file-opened-without-waiting.ts';
import { SecretEnvironmentUnreadableError, } from './secret-environment-unreadable-error.ts';
import { ENV_PATH_VARIABLE_NAME, } from '@monochromatic-dev/env-local-json-core/ts/constants.ts';
import {
  NO_STORE_FOUND,
  resolveStorePath,
} from '@monochromatic-dev/env-local-json-core/ts/discovery.ts';
import { readEnvLocalJson, } from '@monochromatic-dev/env-local-json-core/ts';
import { isNonBlank, } from '@monochromatic-dev/env-local-json-core/ts/environment.ts';
import {
  isMaskableValue,
  type SecretValue,
} from './secret-value-masking.ts';
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
   Whether the environment named a store of its own.
   */
  const explicitlyNamed = ((typeof named) === 'string') && (named !== '');
  /**
   Where discovery looks, which is the checkout this run works in rather than the driver's own folder.
   */
  const discovered = await discoveredStorePath({
    checkoutFolder,
    home,
    ...(explicitlyNamed ? { path: named, } : {}),
    ...(explicitlyNamed ? { namedPath: named, } : {}),
  },);
  if ('cause' in discovered) {
    return { cause: discovered.cause, };
  }
  if ('absent' in discovered) {
    return { absent: true, };
  }
  /**
   The store discovery found.
   */
  const {storePath} = discovered;
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
 Asks this package's discovery where the store is, keeping a failure rather than letting it become "no store".

 @param checkoutFolder - the checkout whose store is looked for

 @param home - home folder bounding discovery

 @param path - the store path the environment named, when it named one

 @returns the store's path, that discovery found none, or why discovery failed
 */
async function askDiscovery({
  checkoutFolder,
  home,
  path,
}: {
  readonly checkoutFolder: string;
  readonly home: string;
  readonly path?: string;
}): Promise<
  | { readonly storePath: string }
    | { readonly absent: true }
    | { readonly cause: string }
  > {
  try {
    /**
     What discovery answered.
     */
    const discovered = await resolveStorePath({
      cwd: checkoutFolder,
      home,
      ...(path === undefined ? {} : { path, }),
    },);
    return (discovered === NO_STORE_FOUND) ? { absent: true, } : { storePath: discovered, };
  }
  catch (error) {
    if (!Error.isError(error,)) {
      throw error;
    }
    return { cause: plainCauseOf(error,), };
  }
}

/**
 Finds the store a checkout declares, telling "the checkout has none" from "a store is there and cannot be read".

 The discovery this package uses answers one symbol for both a checkout with no store and a store it could not stat, so the checkout's
 own store file is asked about here, through the one place that decides absent from could-not.

 @param checkoutFolder - the checkout whose store is looked for

 @param home - home folder bounding discovery

 @param path - the store path the environment named, when it named one

 @param namedPath - that same path, for the message a failure says

 @returns the store's path, that the checkout has none, or why a store that is there cannot be read
 */
async function discoveredStorePath({
  checkoutFolder,
  home,
  path: namedPathToRead,
  namedPath,
}: {
  readonly checkoutFolder: string;
  readonly home: string;
  readonly path?: string;
  readonly namedPath?: string;
}): Promise<
  | { readonly storePath: string }
    | { readonly absent: true }
    | { readonly cause: string }
  > {
  /**
   What discovery answered, including the failure it turns into "no store".
   */
  const discovery = await askDiscovery({
    checkoutFolder,
    home,
    ...(namedPathToRead === undefined ? {} : { path: namedPathToRead, }),
  },);
  if ('cause' in discovery) {
    if (namedPathToRead === undefined) {
      return { cause: discovery.cause, };
    }
    // An explicit path that cannot be used is a store somebody named and this code cannot read, unless the name is simply not there.
    /**
     Whether the named path can be opened at all.
     */
    const opened = await openAnOrdinaryFile({ path: namedPathToRead, },);
    if ('handle' in opened) {
      await opened.handle
        .close();
      return { storePath: namedPathToRead, };
    }
    return ('absent' in opened)
      ? { absent: true, }
      : { cause: discovery.cause, };
  }
  if ('storePath' in discovery) {
    return { storePath: discovery.storePath, };
  }
  // Discovery found nothing. The checkout's own store file is what decides whether that is "none" or "one that cannot be read": a
  // symlink whose target lies behind a folder that denies the way is a store this code must not treat as absent, and a store the
  // discovery missed is still a store.
  /**
   The checkout's own store file.
   */
  const ownStore = join(
    checkoutFolder,
    STORE_FILE_NAME,
  );
  /**
   Whether that file can be opened, which is the question "is a store there" really asks.
   */
  const opened = await openAnOrdinaryFile({ path: ownStore, },);
  if ('handle' in opened) {
    await opened.handle
      .close();
    return { storePath: ownStore, };
  }
  if ('absent' in opened) {
    return { absent: true, };
  }
  return {
    cause: (namedPath === undefined)
      ? opened.cause
      : `${opened.cause}, so the store ${namedPath} cannot be read`,
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
    return { text: await readOrdinaryFile(storePath,), };
  }
  catch (error) {
    if (isMissingPathFailure({ error, },)) {
      return { cause: `${storePath} is not there to read`, };
    }
    // A named pipe, a device or a folder is there and cannot be read as a store: the read refuses it instead of waiting on it.
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
    // The names the store declares, whatever the environment happens to supply, which is the set every place a value for the name
    // can come from is asked with.
    readonly names: readonly string[];
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
    throw new SecretEnvironmentUnreadableError({
      source: `the ${STORE_FILE_NAME} of ${checkoutFolder}`,
      cause: found.cause,
    },);
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
    throw new SecretEnvironmentUnreadableError({
      source: `the ${STORE_FILE_NAME} of ${checkoutFolder}`,
      cause: read.cause,
    },);
  }
  return {
    storePath: found.storePath,
    names: found.names,
    values: read.values,
  };
}

/**
 The environment a store is read with: the caller's own, less the names the store declares and the environment already supplies.

 The reader this code asks keeps the environment's precedence: a name the environment holds is never returned from the store, so the
 store's own value for that name is reachable only by asking with the name absent. Every other variable is passed as it stands, so the
 identity and the sops executable are still found. The pruning is kept to the names the store itself declares, which is what makes that
 ask answer for all of them at once.

 @param environment - the driver's own environment

 @param names - the names the store declares

 @returns the environment to read the store with
 */
function environmentForTheStore({
  environment,
  names,
}: {
  readonly environment: EnvironmentOfThisMachine;
  readonly names: readonly string[];
},): EnvironmentOfThisMachine {
  /**
   The declared names the environment supplies, which the reader would answer the store's value for only if they were absent.
   */
  const shadowed = names.filter(function isHeld(name,) {
    // The reader this code asks keeps the environment's precedence for any value that is not blank, so a short value shadows the
    // store's long one just as a long value does: pruning only the long ones left the long store value unreadable, and unarmed.
    return isNonBlank(environment[name],);
  },);
  if (shadowed.length === 0) {
    return environment;
  }
  /**
   The environment with those names taken out.
   */
  const pruned: Record<string, string> = {};
  for (const [name, value,] of Object.entries(environment,)) {
    if (((typeof value) === 'string') && (!shadowed.includes(name,))) {
      pruned[name] = value;
    }
  }
  return pruned;
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
      env: environmentForTheStore({
        environment,
        names,
      },),
      home,
    },);
    /**
     The values as a list, named as the store names them.
     */
    const values: SecretValue[] = [];
    for (const name of names) {
      /**
       The store's own value for that name, which the reader answers only when the environment does not shadow it.
       */
      const value = read.values[name];
      if ((value === undefined) || (!isMaskableValue(value,))) {
        continue;
      }
      values.push({
        name,
        value,
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
