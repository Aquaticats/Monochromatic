/**
 Every value this machine holds for its secret environment, read once in the driver's own process so each can be masked before it
 reaches a model.

 Four sources, and why these: the checkout's `.env.local.json` is the repository's secret environment itself, and the names it
 declares are the ones the rules file bans printing; Pi's own `auth.json` and `models.json` hold provider keys in clear, and both
 are files the rules file names; the shell start-up files are where a token exported for a terminal lives, and the rules file
 names `~/.bashrc` for the same reason; and the driver's own environment holds what a command cannot inherit, because
 `environment-without-credentials.ts` takes it from Pi's environment, which is what a read of `/proc/<pid>/environ` gets back. The
 age identity that decrypts the store is read as well, because it is the one key that opens every other value.

 None of these is a place this code writes: a value lives in the returned list, and the caller masks with it and lets it go.

 @module
 */

import type { SecretValue, } from './secret-value-masking.ts';
import { valuesInTheCheckoutStore, } from './secret-values-in-the-checkout-store.ts';
import {
  type EnvironmentOfThisMachine,
  type SecretValueSourceReport,
  type UnreadSecretValueSource,
  valuesInPiCredentialFiles,
  valuesInShellStartFiles,
  withoutRepeatedValues,
} from './secret-values-in-the-files-of-this-machine.ts';
import {
  valuesInTheAgeIdentity,
  valuesInTheEnvironment,
} from './secret-values-the-driver-holds.ts';

//region Types

/**
 Everything this machine holds for its secret environment.
 */
export type SecretValuesOfThisMachine = {
  /**
   The values, each with what a marker names in its place. A value appears once, under the first source that gave it.
   */
  readonly values: readonly SecretValue[];

  /**
   The sources that gave values, in the order they were read.
   */
  readonly sources: readonly SecretValueSourceReport[];

  /**
   The sources that hold values and could not be read. The store is never one of these: a store that is there and cannot be read
   stops the run, because the values the rules file bans printing are exactly the ones this code would then not know.
   */
  readonly unread: readonly UnreadSecretValueSource[];
};

//endregion Types

//region Reading them all

/**
 Reads every value this machine holds for its secret environment, once, in this process.

 A source that holds values and cannot be read is named in `unread`, except the checkout's store: a store that is there and
 cannot be read stops the run, because the values the rules file bans printing are exactly the ones this code would then not
 know, and a run that started would show them to a model.

 @param environment - the driver's own environment

 @param home - the home folder the identity, Pi's files and the shell start-up files are read from

 @param checkoutFolder - the checkout this run works in, whose secret environment is the one the rules file is about

 @returns the values, the sources that gave them and the sources that could not be read

 @throws SecretEnvironmentUnreadableError when the checkout's store is there and cannot be read

 @example
 ```ts
 const read = await readSecretValuesOfThisMachine({ environment: process.env, home: '/home/a', checkoutFolder: '/repo', },);
 ```
 */
export async function readSecretValuesOfThisMachine({
  environment,
  home,
  checkoutFolder,
}: {
  readonly environment: EnvironmentOfThisMachine;
  readonly home: string;
  readonly checkoutFolder: string;
},): Promise<SecretValuesOfThisMachine> {
  /**
   The checkout's own store, which is the only source that can stop the run.
   */
  const store = await valuesInTheCheckoutStore({
    checkoutFolder,
    environment,
    home,
  },);
  /**
   What the store declared, which is also what the environment is asked for by name.
   */
  const storeValues: readonly SecretValue[] = ('absent' in store) ? [] : store.values;
  /**
   Pi's own files.
   */
  const pi = await valuesInPiCredentialFiles({ home, },);
  /**
   The shell start-up files.
   */
  const shell = await valuesInShellStartFiles({ home, },);
  /**
   The key that decrypts the store.
   */
  const identity = await valuesInTheAgeIdentity({
    environment,
    home,
  },);
  /**
   The driver's own environment.
   */
  const fromEnvironment = valuesInTheEnvironment({
    environment,
    alsoTheseNames: new Set(storeValues.map(function nameOf(value,) {
      return value.name;
    },),),
  },);
  /**
   The identity's keys, or none when the identity could not be read.
   */
  const identityValues: readonly SecretValue[] = ('cause' in identity) ? [] : identity.values;
  return {
    values: withoutRepeatedValues([
      ...storeValues,
      ...pi.values,
      ...shell.values,
      ...identityValues,
      ...fromEnvironment,
    ],),
    sources: [
      ...(('absent' in store) ? [] : [{
        source: `the .env.local.json of ${checkoutFolder}`,
        count: storeValues.length,
      },]),
      ...pi.sources,
      ...shell.sources,
      ...(('cause' in identity) ? [] : [{
        source: identity.source,
        count: identityValues.length,
      },]),
      {
        source: 'the driver\'s own environment',
        count: fromEnvironment.length,
      },
    ],
    unread: [
      ...pi.unread,
      ...shell.unread,
      ...(('cause' in identity) ? [{
        source: 'the age identity',
        cause: identity.cause,
      },] : []),
    ],
  };
}

//endregion Reading them all
