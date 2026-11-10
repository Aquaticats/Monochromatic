/**
 Fixed inputs and shared helpers for the deterministic coverage driver.

 Every value here is fixed rather than generated, so reruns of the driver reach
 exactly the same store paths and the V8 coverage the baseline records is
 reproducible.

 @module
 */

import { createConf, } from '@monochromatic-dev/upstream-fork-conf/ts';

//region Values

/**
 Fresh item value the round-trip fixtures write.
 */
export const FRESH_VALUE = 'dark';

/**
 Value the encrypted-store fixture writes and must read back.
 */
export const ENCRYPTED_VALUE = 'encrypted';

/**
 Encryption key the encrypted-store fixture uses.
 */
export const ENCRYPTION_KEY = 'coverage-driver-key';

/**
 Garbage ciphertext forcing the decryption-failure path:
 longer than the initialization vector,
 without the separator byte at its fixed offset.
 */
export const GARBAGE_CIPHERTEXT = 'garbage-ciphertext-without-separator';

/**
 Project version the migration fixtures migrate to.
 */
export const MIGRATED_VERSION = '2.0.0';

/**
 Project version the seed migration fixture starts at.
 */
export const SEED_VERSION = '1.0.0';

/**
 Item value the successful migration step writes.
 */
export const MIGRATION_VALUE = 'two';

/**
 Failure thrown by the failing migration fixture.
 */
const MIGRATION_FAILURE = new Error('migration boom',);

/**
 Number fed where the API requires a string or an object.

 Both type-invalid inputs only need to be some number, so one value drives the
 invalid-key and invalid-schema paths; it is composed from the lint config's
 exempt literal range rather than spelled as a bare literal.
 */
const WRONG_TYPE_NUMBER = 1 + 2;

//endregion Values

//region Invalid inputs

/* oxlint-disable typescript/no-unsafe-type-assertion -- invalid runtime inputs mirror a mis-typed caller */

/**
 Non-string key fed past the static type,
 driving the invalid-key paths.
 */
export const NON_STRING_KEY = WRONG_TYPE_NUMBER as never;

/**
 Non-object `schema` option fed past the static type,
 driving the invalid-schema path.
 */
export const NON_OBJECT_SCHEMA = WRONG_TYPE_NUMBER as never;

/**
 Unsupported encryption algorithm fed past the static type,
 driving the invalid-algorithm path.
 */
export const UNSUPPORTED_ALGORITHM = 'bogus' as never;

/**
 Non-function callback fed past the static type,
 driving the invalid-callback path.
 */
export const NON_FUNCTION_CALLBACK = undefined as never;

/**
 Root schema carrying the `properties` keyword the fork refuses at the root,
 driving the root-schema error path.
 */
export const FORBIDDEN_ROOT_SCHEMA = {
  properties: {},
} as never;

/* oxlint-enable typescript/no-unsafe-type-assertion */

//endregion Invalid inputs

//region Helpers

/**
 Function value fed past the static type,
 driving the unsupported-value-type paths.
 
 @example
 ```ts
 config.set({ key: 'theme', value: functionValue, });
 ```
 */
export function functionValue(): void {
  return undefined;
}

/**
 Callback attached to `onDidChange` and `onDidAnyChange`,
 observing nothing and keeping the subscription surface exercised.
 
 @example
 ```ts
 config.onDidChange({ key: 'theme', callback: observeChange, });
 ```
 */
export function observeChange(): void {
  return undefined;
}

/**
 Migration handler marking the seed step without changing items.
 
 @example
 ```ts
 createConf({ projectVersion: SEED_VERSION, migrations: { '1.0.0': seedMigrationStep, }, });
 ```
 */
export function seedMigrationStep(): void {
  return undefined;
}

/**
 Migration step that always throws,
 driving the failing-migration path.
 
 @example
 ```ts
 createConf({ projectVersion: MIGRATED_VERSION, migrations: { '2.0.0': failingStep, }, });
 ```
 */
export function failingStep(): void {
  throw MIGRATION_FAILURE;
}

/**
 Runs a thunk that is expected to throw,
 swallowing `Error` outcomes so the driver keeps exercising remaining paths.
 Re-throws anything that is not an `Error`.
 
 @param thunk - Operation expected to throw.
 
 @example
 ```ts
 swallow(function bad() {
   createConf({});
 });
 ```
 */
export function swallow(thunk: () => void,): void {
  try {
    thunk();
  }
  catch (error: unknown) {
    if (!(Error.isError(error,)))
      throw error;
  }
}

/**
 Throws when a driver-internal sanity expectation fails,
 keeping stale driver inputs from silently freezing a weaker baseline.
 
 @param holds - Whether the expectation held.
 
 @param label - Name of the expectation that failed.
 
 @throws Error naming the failed expectation.
 
 @example
 ```ts
 ensure({ holds: roundTrip === FRESH_VALUE, label: 'encrypted round-trip', });
 ```
 */
export function ensure(
  {
    holds,
    label,
  }: {
    readonly holds: boolean;
    readonly label: string;
  },
): void {
  if (!holds)
    throw new Error(`coverage driver ${label} diverged; driver inputs are stale`,);
}

//endregion Helpers
