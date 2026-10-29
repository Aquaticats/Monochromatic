/**
 Serialize file mutations that target one path while leaving different paths concurrent.

 Ported from Pi's `withFileMutationQueue` so output formatting needs no pi dependency.
 One documented deviation: the queue key is `path.resolve` output rather than a `realpath`
 canonicalization, so two symlink spellings of one file queue separately while mutation
 registration stays strictly call-ordered.

 @module
 */

import { resolve, } from 'node:path';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

//region Logger

/**
 Logger root for search-fetch shared modules.
 */
const searchFetchLogger = tagged({ tag: 'search-fetch', },);

/**
 Module logger.
 */
const l = tagged({
  tag: 'file-mutation-queue',
  l: searchFetchLogger,
},);

//endregion Logger

//region Types

/**
 Input accepted by {@link withFileMutationQueue}.
 */
export type FileMutationQueueInput<MutationResult> = {
  /**
   File the mutation writes; relative and absolute spellings share one queue.
   */
  readonly filePath: string;
  /**
   Mutation to run once earlier same-path mutations settled.
   */
  readonly mutate: () => Promise<MutationResult>;
};

/**
 Scope that removes a settled tail from the queue map.
 */
type TailCleanupInput = {
  /**
   Resolved path key owning the tail.
   */
  readonly key: string;
  /**
   Tail published for this mutation.
   */
  readonly tail: Promise<void>;
};

//endregion Types

//region State

/**
 Tail promise per resolved path; settles when the queued mutation settles.
 */
const mutationTails = new Map<string, Promise<void>>();

/**
 Tail used when no earlier mutation is queued for a path.
 */
const UNQUEUED_TAIL: Promise<void> = Promise.resolve();

//endregion State

//region Public API

/**
 Run a file mutation serialized against earlier mutations of the same path.

 @param input - file path and mutation to queue

 @returns mutation result, forwarded unchanged

 @throws whatever the mutation throws, after the queue tail is released

 @example
 ```ts
 await withFileMutationQueue({
   filePath: '/tmp/response.json',
   mutate: async function writeResponse() { await writeFile(path, text); },
 });
 ```
 */
export async function withFileMutationQueue<MutationResult>(
  input: FileMutationQueueInput<MutationResult>,
): Promise<MutationResult> {
  /**
   Resolved path key shared by relative and absolute spellings of one file.
   */
  const key = resolve(input.filePath,);
  /**
   Tail of the mutation queued before this one on the same path.
   */
  const previousTail = mutationTails.get(key,) ?? UNQUEUED_TAIL;
  /**
   This mutation's outcome, awaited by both the caller and the published tail.
   */
  const outcome = (async function runQueuedMutation(): Promise<MutationResult> {
    await previousTail;
    return input.mutate();
  })();
  /**
   Completion signal for the next queued mutation, settling even when this one fails.
   */
  const tail = (async function publishTail(): Promise<void> {
    try {
      await outcome;
    }
    catch (error: unknown) {
      l.debug(
        `queued mutation on ${key} failed before tail release: ${caughtValueText(error,)}`,
      );
    }
  })();
  mutationTails.set(
    key,
    tail,
  );
  /**
   Scope guard dropping this tail once the mutation settles.
   */
  using cleanup = createTailCleanup({
    key,
    tail,
  });
  return await outcome;
}

//endregion Public API

//region Helpers

/**
 Build the scope that drops a settled tail when no later mutation replaced it.

 @param input - resolved path key and published tail

 @returns disposable removing the tail entry on scope exit

 @example
 ```ts
 using cleanup = createTailCleanup({ key, tail });
 ```
 */
function createTailCleanup(input: TailCleanupInput,): Disposable {
  return {
    [Symbol.dispose](): void {
      if (mutationTails.get(input.key,) === input.tail)
        mutationTails.delete(input.key,);
    },
  };
}

//endregion Helpers
