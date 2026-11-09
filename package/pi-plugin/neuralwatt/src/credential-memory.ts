/**
 One process's memory of the encrypted store read.
 The read that answered is reused instead of decrypted per request,
 a failed read keeps the last good one,
 and a stale read is revalidated behind the request that asked for it,
 so no request waits on sops.
 The retry bound for a first read also lives here,
 so a stalled sops window delays one request instead of failing a session.

 @module
 */

import {
  caughtValueText,
} from '@monochromatic-dev/caught-value-core/ts';
import {
  tagged,
  type Logger,
} from '@monochromatic-dev/logger-core/ts';
import {
  setTimeout as sleep,
} from 'node:timers/promises';
import {
  CREDENTIAL_REVALIDATION_INTERVAL_MS,
  STORE_READ_ATTEMPTS,
  STORE_READ_DEADLINE_MS,
  STORE_READ_RETRY_DELAY_MS,
} from './constants.ts';

//region Types

/**
 One store read's outcome:
 the stored value under the credential name and the store path when loaded.
 */
export type NeuralWattStoreRead = {
  /**
   Stored value, absent when the store carried none.
   */
  readonly value?: string;
  /**
   Store path, absent when no store was read.
   */
  readonly path?: string;
};

/**
 One read of the encrypted store.

 @param signal - cancellation the caller owns
 */
export type NeuralWattStoreReadSource = (input: {
  /**
   Abort signal the read and its retries obey.
   */
  readonly signal?: AbortSignal;
}) => Promise<NeuralWattStoreRead>;

/**
 What one process remembers between credential resolutions.
 */
type NeuralWattStoreMemory = {
  /**
   Read that answered with a key, absent until one does.
   */
  readonly stored?: NeuralWattStoreRead;
  /**
   Earliest time the remembered read is revalidated,
   `now()` when nothing is remembered.
   */
  readonly revalidateAt: number;
  /**
   Read in flight, shared so concurrent requests decrypt once.
   */
  readonly reading?: Promise<NeuralWattStoreRead>;
};

//endregion Types

//region Attempt signal

/**
 Combine the caller's abort signal with the time one attempt may spend,
 so an attempt never outlives the resolution's deadline.

 @param signal - caller abort signal, absent when nobody can cancel

 @param budgetMs - time this attempt may take before its own signal aborts

 @returns signal covering both cancellation sources

 @example
 ```typescript
 const signal = budgetSignal({ signal: input.signal, budgetMs: 15_000 });
 ```
 */
function budgetSignal({
  signal,
  budgetMs,
}: {
  readonly signal?: AbortSignal;
  readonly budgetMs: number;
},): AbortSignal {
  /**
   Budget signal owned by this one attempt.
   */
  const budget = AbortSignal.timeout(budgetMs,);
  if (signal === undefined)
    return budget;
  return AbortSignal.any([
    signal,
    budget,
  ],);
}

//endregion Attempt signal

//region Remembering store read

/**
 Build a store read that remembers what it answered.
 `read` serves the remembered read inside its revalidation window,
 starts a revalidation behind the request once the window passes,
 and blocks only when nothing is remembered yet,
 where it retries inside the attempt bound.

 @param readStoreValue - reads the store once, injected so no test decrypts

 @param logger - package logger, re-tagged for this memory

 @param now - clock seam, injected in tests so a revalidation window passes
 without waiting for one

 @returns the store read a credential resolution calls

 @example
 ```typescript
 const read = createRememberingStoreRead({ readStoreValue, logger });
 const store = await read({ signal });
 ```
 */
export function createRememberingStoreRead({
  readStoreValue,
  logger,
  now = Date.now,
}: {
  readonly readStoreValue: NeuralWattStoreReadSource;
  readonly logger: Logger;
  readonly now?: () => number;
},): NeuralWattStoreReadSource {
  /**
   Logger tagged for this memory, so records name the credential seam.
   */
  const l = tagged({
    tag: createRememberingStoreRead.name,
    l: logger,
  },);
  /**
   Everything this process remembers about the store between resolutions,
   replaced wholesale on every change so no field is mutated in place.
   */
  const remembered: { current: NeuralWattStoreMemory; } = {
    current: { revalidateAt: 0, },
  };

  /**
   Remember one store read, keeping the in-flight slot untouched, or drop the
   remembered key when one was not named.

   @param key - read that answered with a key, absent when nothing answered

   @param revalidateAt - earliest time the remembered read is revalidated

   @example
   ```typescript
   remember({ key: read, revalidateAt: now() + 60_000 });
   ```
   */
  function remember({
    key,
    revalidateAt,
  }: {
    readonly key?: NeuralWattStoreRead;
    readonly revalidateAt: number;
  },): void {
    /**
     Read still in flight, carried across this change.
     */
    const { reading, } = remembered.current;
    remembered.current = {
      ...(reading === undefined ? {} : { reading, }),
      revalidateAt,
      ...(key === undefined ? {} : { stored: key, }),
    };
  }

  /**
   Forget the in-flight read, so the next resolution starts a fresh one.
   */
  function forgetReading(): void {
    /**
     Remembered read, kept while the in-flight slot is cleared.
     */
    const {
      revalidateAt,
      stored,
    } = remembered.current;
    remembered.current = (stored === undefined)
      ? { revalidateAt, }
      : {
        revalidateAt,
        stored,
      };
  }

  /**
   Read the store once under the attempt budget of this resolution.

   @param signal - caller abort signal, which ends the retries too

   @param attempt - attempt number this call spends

   @param deadline - wall clock time the attempts stop at

   @returns the store read that answered

   @throws the failure this attempt reported, once no attempt is left

   @example
   ```typescript
   const read = await readStoreAttempt({ attempt: 1, deadline });
   ```
   */
  async function readStoreAttempt({
    signal,
    attempt,
    deadline,
  }: {
    readonly signal?: AbortSignal;
    readonly attempt: number;
    readonly deadline: number;
  },): Promise<NeuralWattStoreRead> {
    /**
     Logger tagged for this attempt, so records name the entry point.
     */
    const rl = tagged({
      tag: readStoreAttempt.name,
      l,
    },);
    /**
     Time this attempt may spend: what is left of the deadline, never zero,
     so the first attempt always runs.
     */
    const budgetMs = Math.max(
      deadline - now(),
      1,
    );
    try {
      return await readStoreValue({
        signal: budgetSignal({
          ...(signal === undefined ? {} : { signal, }),
          budgetMs,
        },),
      },);
    } catch (error) {
      rl.warn(
        `encrypted store read attempt ${String(attempt,)} of ${String(STORE_READ_ATTEMPTS,)} failed: ${caughtValueText(error,)}`,
      );
      /**
       Whether this attempt is the last one this resolution spends.
       */
      const spent = (signal?.aborted === true)
        || (attempt >= STORE_READ_ATTEMPTS)
        || ((deadline - now()) <= STORE_READ_RETRY_DELAY_MS);
      if (spent)
        throw error;
      await sleep(STORE_READ_RETRY_DELAY_MS,);
      return await readStoreAttempt({
        ...(signal === undefined ? {} : { signal, }),
        attempt: attempt + 1,
        deadline,
      },);
    }
  }

  /**
   Read the store and remember what it answered, keeping the key resolved
   earlier when the read fails.

   @param signal - caller abort signal, forwarded to the read

   @returns the store read that answered, the remembered read when the read
   fails, or an empty result when nothing was ever remembered

   @example
   ```typescript
   const read = await refresh({ signal: input.signal });
   ```
   */
  async function refresh({
    signal,
  }: {
    readonly signal?: AbortSignal;
  },): Promise<NeuralWattStoreRead> {
    /**
     Logger tagged for this refresh, so records name the entry point.
     */
    const rl = tagged({
      tag: refresh.name,
      l,
    },);
    /**
     Read this process remembered before this refresh.
     */
    const { stored: previous, } = remembered.current;
    try {
      /**
       Store read this refresh earned.
       */
      const read = await readStoreAttempt({
        ...(signal === undefined ? {} : { signal, }),
        attempt: 1,
        deadline: now() + STORE_READ_DEADLINE_MS,
      },);
      if (read.value === undefined) {
        remember({ revalidateAt: 0, },);
        return read;
      }
      remember({
        key: read,
        revalidateAt: now() + CREDENTIAL_REVALIDATION_INTERVAL_MS,
      },);
      rl.debug(
        `the encrypted store answered, so its key stands until the next revalidation in ${String(CREDENTIAL_REVALIDATION_INTERVAL_MS,)} ms`,
      );
      return read;
    } catch (error) {
      rl.warn(
        `encrypted store read failed, falling through to the environment: ${caughtValueText(error,)}`,
      );
      if (previous === undefined) {
        remember({ revalidateAt: 0, },);
        return {};
      }
      rl.warn(
        'keeping the NeuralWatt credential resolved earlier, so requests keep working until a store read answers',
      );
      remember({
        key: previous,
        revalidateAt: now() + CREDENTIAL_REVALIDATION_INTERVAL_MS,
      },);
      return previous;
    }
  }

  /**
   Read the store once for every caller waiting on it, so concurrent requests
   decrypt once.

   @param signal - caller abort signal; the first caller's signal governs the
   shared read

   @returns the store read the shared refresh produced

   @example
   ```typescript
   const read = await sharedRead({ signal: input.signal });
   ```
   */
  async function sharedRead({
    signal,
  }: {
    readonly signal?: AbortSignal;
  },): Promise<NeuralWattStoreRead> {
    /**
     Read another caller already started, shared instead of repeated.
     */
    const { reading, } = remembered.current;
    if (reading !== undefined)
      return await reading;
    /**
     Read this caller starts and hands to any concurrent caller.
     */
    const started = refresh(signal === undefined ? {} : { signal, },);
    remembered.current = {
      ...remembered.current,
      reading: started,
    };
    try {
      /**
       Read the shared refresh produced.
       */
      const read = await started;
      forgetReading();
      return read;
    } catch (error) {
      forgetReading();
      throw error;
    }
  }

  /**
   Revalidate the remembered read behind the request that asked for it, so a
   rotated key lands without any request waiting for the store.

   @param rl - logger tagged for the requesting resolution

   @example
   ```typescript
   void revalidateBehindRequest({ rl });
   ```
   */
  async function revalidateBehindRequest({
    rl,
  }: {
    readonly rl: Logger;
  },): Promise<void> {
    try {
      await sharedRead({},);
    } catch (error) {
      rl.error(
        `the encrypted store re-read behind a request failed: ${caughtValueText(error,)}`,
      );
    }
  }

  /**
   Store read one resolution resolves against: the remembered read while its
   window stands, a revalidation started behind the request once the window
   passes, and one shared blocking read when nothing is remembered.

   @param signal - caller abort signal, used only by a blocking read

   @returns the store read to resolve against

   @example
   ```typescript
   const read = await rememberedRead({ signal: input.signal });
   ```
   */
  return async function readRememberedStore({
    signal,
  }: {
    readonly signal?: AbortSignal;
  },): Promise<NeuralWattStoreRead> {
    /**
     Logger tagged for this read, so records name the entry point.
     */
    const rl = tagged({
      tag: readRememberedStore.name,
      l,
    },);
    /**
     Everything remembered before this resolution.
     */
    const {
      revalidateAt,
      stored,
    } = remembered.current;
    if (stored !== undefined) {
      if (now() < revalidateAt) {
        rl.debug(
          'reusing the encrypted store read that already answered, so this resolution decrypts nothing',
        );
        return stored;
      }
      rl.debug(
        'serving the remembered credential while the encrypted store is re-read behind this request',
      );
      void revalidateBehindRequest({ rl, },);
      return stored;
    }
    return await sharedRead(signal === undefined ? {} : { signal, },);
  };
}

//endregion Remembering store read
