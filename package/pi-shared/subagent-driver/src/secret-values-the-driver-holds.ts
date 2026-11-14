/**
 The values the driver's own process holds: its credential-named environment, and the age identity that decrypts everything else.

 Pi's environment has the credential-named variables taken from it (`environment-without-credentials.ts`), so a command cannot
 inherit one; an ancestor process still holds them, which is exactly what a read of `/proc/<pid>/environ` from inside a run gets
 back. The identity is read for the same reason one step further: it is the one key that opens every value of the store.

 @module
 */

import { readFile, } from 'node:fs/promises';
import {
  resolveSopsIdentity,
  type SopsIdentity,
} from '@monochromatic-dev/env-local-json-core/ts/identity.ts';
import { carriesCredentialName, } from './environment-without-credentials.ts';
import type { SecretValue, } from './secret-value-masking.ts';
import {
  type EnvironmentOfThisMachine,
  isKeptDespiteItsName,
  isMaskableValue,
  plainCauseOf,
} from './secret-values-in-the-files-of-this-machine.ts';

//region Constants

/**
 The shape an age secret key has, which is what an identity file holds.

 A regular expression is what finds a key in an identity file, whose only other lines are comments and blank ones. The pattern is
 one run of characters with no alternative and no nested quantifier, so matching a file is linear in its length.
 */
// oxlint-disable-next-line no-restricted-syntax/no-regex -- finding a key in an identity file needs a pattern; see the comment above.
const AGE_SECRET_KEY_PATTERN = /AGE-SECRET-KEY-1[A-Z0-9]+/gu;

//endregion Constants

//region The driver's environment

/**
 Reads the credential-named variables of the driver's own environment.

 A store variable's name is read here too when the environment already supplies it: the store reader answers no value for a name
 the environment holds, so that value is the one a command could read back from an ancestor's environment.

 @param environment - the driver's environment

 @param alsoTheseNames - names to include whatever they are spelled like, which is what a store declares

 @returns the values, named by their variable

 @example
 ```ts
 const values = valuesInTheEnvironment({ environment: process.env, alsoTheseNames: new Set(['DEEPSEEK_API_KEY']), },);
 ```
 */
export function valuesInTheEnvironment({
  environment,
  alsoTheseNames,
}: {
  readonly environment: EnvironmentOfThisMachine;
  readonly alsoTheseNames: ReadonlySet<string>;
},): readonly SecretValue[] {
  /**
   The values found.
   */
  const values: SecretValue[] = [];
  for (const [name, value,] of Object.entries(environment,)) {
    if ((typeof value) !== 'string') {
      continue;
    }
    if ((!carriesCredentialName({ name, },)) && (!alsoTheseNames.has(name,))) {
      continue;
    }
    if ((isKeptDespiteItsName(name,)) || (!isMaskableValue(value,))) {
      continue;
    }
    values.push({
      name,
      value,
    },);
  }
  return values;
}

//endregion The driver's environment

//region The age identity

/**
 What reading the decrypting key found.
 */
export type ValuesInTheAgeIdentity =
  | {
    readonly values: readonly SecretValue[];
    readonly source: string
  }
    | { readonly cause: string };

/**
 Reads the key that decrypts the store, which is the one value that opens every other one.

 @param environment - the environment naming an identity

 @param home - the home folder the default identity file is in

 @returns the identity's keys, what the source is called, and why it could not be read when it could not

 @example
 ```ts
 const read = await valuesInTheAgeIdentity({ environment: process.env, home: '/home/a', },);
 ```
 */
export async function valuesInTheAgeIdentity({
  environment,
  home,
}: {
  readonly environment: EnvironmentOfThisMachine;
  readonly home: string;
},): Promise<ValuesInTheAgeIdentity> {
  /**
   The identity mise and sops agree on, resolved by the reader that owns those rules.
   */
  const identity = await identityOfThisMachine({
    environment,
    home,
  },);
  if ('cause' in identity) {
    return { cause: identity.cause, };
  }
  return valuesOfTheIdentity({ identity: identity.identity, },);
}

/**
 Resolves the identity mise and sops agree on.

 @param environment - the environment naming an identity

 @param home - the home folder the default identity file is in

 @returns the identity, or why no candidate could be used
 */
async function identityOfThisMachine({
  environment,
  home,
}: {
  readonly environment: EnvironmentOfThisMachine;
  readonly home: string;
},): Promise<
  | { readonly identity: SopsIdentity }
    | { readonly cause: string }
  > {
  try {
    return { identity: await resolveSopsIdentity({
      env: environment,
      home,
    },), };
  }
  catch (error) {
    return { cause: plainCauseOf(error,), };
  }
}

/**
 Reads the keys an identity holds, which are the key material itself for an inline identity and the lines of an identity file for
 one that names a file.

 @param identity - the resolved identity

 @returns the keys, and what a marker names the source as
 */
async function valuesOfTheIdentity({ identity, }: { readonly identity: SopsIdentity },): Promise<
  | {
    readonly values: readonly SecretValue[];
    readonly source: string
  }
    | { readonly cause: string }
  > {
  /**
   What a marker names this identity as: the variable that named it, or the file it is kept in.
   */
  const source = (identity.kind === 'inline')
    ? `the age identity in ${identity.source}`
    : `the age identity in ${identity.value}`;
  if (identity.kind === 'inline') {
    return {
      source,
      values: isMaskableValue(identity.value,)
        ? [{
          name: source,
          value: identity.value,
        },]
        : [],
    };
  }
  try {
    /**
     The identity file's text.
     */
    const text = await readFile(
      identity.value,
      'utf8',
    );
    return {
      source,
      values: [...text.matchAll(AGE_SECRET_KEY_PATTERN),].map(function toValue([key,],) {
        return {
          name: source,
          value: key,
        };
      },),
    };
  }
  catch (error) {
    return { cause: plainCauseOf(error,), };
  }
}

//endregion The age identity
