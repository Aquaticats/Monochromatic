/**
 Observed-state helpers for pure-Wayland Electron boundary tests.
 
 @example
 ```ts
 await waitForObservedState({ statePath: '/tmp/state.json', expected: { count: 1 } });
 ```
 */

import { readFile, } from 'node:fs/promises';
import { setTimeout as wait, } from 'node:timers/promises';

import type {
  JsonObject,
  JsonScalar,
} from './atomic-json.js';
import {
  pollIntervalMs,
  stateReadyDeadlineMs,
} from './wayland-constants.js';

/**
 Sentinel for an observed state file that has not been written yet.
 
 @example
 ```ts
 console.log(typeof OBSERVED_STATE_ABSENT);
 ```
 */
const OBSERVED_STATE_ABSENT: unique symbol = Symbol(
  'Boundary test state file has not been written yet',
);

/**
 Expected shallow state snapshot.
 
 @example
 ```ts
 const state: ExpectedObservedState = { count: 1 };
 ```
 */
export type ExpectedObservedState = JsonObject;

/**
 Checks whether an unknown value is a supported JSON scalar.
 
 @param value - Value to check.
 
 @returns Whether value can be compared by the state poller.
 
 @example
 ```ts
 isJsonScalar(1);
 ```
 */
function isJsonScalar(value: unknown,): value is JsonScalar {
  return ((typeof value) === 'string')
    || ((typeof value) === 'number')
    || ((typeof value) === 'boolean');
}

/**
 Checks whether a value can be read as a string-keyed unknown record.
 
 @param value - Value to check.
 
 @returns Whether value is a non-array object.
 
 @example
 ```ts
 isReadonlyUnknownRecord({ count: 1 });
 ```
 */
function isReadonlyUnknownRecord(value: unknown,): value is Readonly<Record<string, unknown>> {
  return ((typeof value) === 'object')
    && (value !== null)
    && (!Array.isArray(value,));
}

/**
 Parses observed JSON state written by an Electron main process.
 
 @param value - Parsed JSON value.
 
 @returns Observed shallow JSON state.
 
 @throws Error when JSON shape is unexpected.
 
 @example
 ```ts
 parseObservedState({ value: { count: 1 } });
 ```
 */
function parseObservedState({ value, }: { readonly value: unknown; },): JsonObject {
  if (!isReadonlyUnknownRecord(value,))
    throw new Error('Observed boundary-test state must be a shallow object.',);

  /**
   State reconstructed after scalar validation.
   */
  const state: Record<string, JsonScalar> = {};

  Object.entries(value,)
    .forEach(function assertScalarStateEntry([key, entryValue,],): void {
      if (!isJsonScalar(entryValue,))
        throw new Error(`Observed boundary-test state entry ${key} must be a string, number, or boolean.`,);

      state[key] = entryValue;
    },);

  return state;
}

/**
 Reads observed state file if it exists.
 
 @param statePath - State file path written by Electron main process.
 
 @returns Parsed observed state, or {@link OBSERVED_STATE_ABSENT} when file is not present yet.
 
 @example
 ```ts
 await readObservedState({ statePath: '/tmp/state.json' });
 ```
 */
async function readObservedState(
  { statePath, }: { readonly statePath: string; },
): Promise<JsonObject | typeof OBSERVED_STATE_ABSENT> {
  try {
    /**
     Raw state JSON emitted by Electron main process.
     */
    const stateText = await readFile(
      statePath,
      'utf8',
    );
    return parseObservedState({ value: JSON.parse(stateText,), },);
  }
  catch (error: unknown) {
    if (
      Error.isError(error,)
        && ('code' in error)
        && (error.code === 'ENOENT')
    )
      return OBSERVED_STATE_ABSENT;

    throw error;
  }
}

/**
 Checks whether observed state contains every expected entry.
 
 @param expected - Expected shallow state entries.
 
 @param observed - Observed shallow state entries.
 
 @returns Whether all expected entries match.
 
 @example
 ```ts
 stateMatches({ observed: { count: 1 }, expected: { count: 1 } });
 ```
 */
function stateMatches(
  {
    expected,
    observed,
  }: {
    readonly expected: ExpectedObservedState;
    readonly observed: JsonObject;
  },
): boolean {
  return Object.entries(expected,)
    .every(function expectedEntryMatches([key, value,],): boolean {
      return observed[key] === value;
    },);
}

/**
 Waits until observed state contains expected entries.
 
 @param expected - Expected shallow state entries.
 
 @param statePath - State file path written by Electron main process.
 
 @mutates expected - `JSON.stringify` may invoke record accessors or proxy traps when timeout is reported.
 
 @example
 ```ts
 await waitForObservedState({ statePath: '/tmp/state.json', expected: { count: 1 } });
 ```
 */
export async function waitForObservedState(
  {
    expected,
    statePath,
  }: {
    readonly expected: ExpectedObservedState;
    readonly statePath: string;
  },
): Promise<void> {
  /**
   Absolute timestamp when state waiting must fail.
   */
  const deadline = Date.now() + stateReadyDeadlineMs;

  while (Date.now() < deadline) {
    /**
     Current observed state, if Electron has written one.
     */
    // oxlint-disable-next-line eslint/no-await-in-loop -- sequential polling must read latest state before sleeping.
    const state = await readObservedState({ statePath, },);

    if ((state !== OBSERVED_STATE_ABSENT) && stateMatches({
      expected,
      observed: state,
    },))
      return;

    // oxlint-disable-next-line eslint/no-await-in-loop -- sequential polling must delay between state reads.
    await wait(pollIntervalMs,);
  }

  throw new Error(`Timed out waiting for observed state ${JSON.stringify(expected,)}`,
  );
}
