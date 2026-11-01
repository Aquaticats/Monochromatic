/**
 Decide, for each object of a batch, whether the presence index answers or the bucket must be asked.

 @module
 */

import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';

import type {
  BatchObject,
  BatchOperation,
} from './batch.ts';
import {
  lookupPresent,
  type PresenceLookup,
  recordPresent,
} from './presence-index.ts';
import type { WorkerEnv, } from './store.ts';

/**
 Most index misses of one download batch that are confirmed against the bucket.

 A miss means "unconfirmed". git-lfs downloads only objects it saw in a tree, so
 misses are rare, and each confirmation is an R2 `head` costing 0.25 to 0.45 ms
 of CPU. Past this many misses the index is trusted and the rest are reported absent.
 */
export const DOWNLOAD_MISS_CHECK_LIMIT = 20;

/**
 What is known about one object before the bucket is asked.
 - `present`: the index holds it, so the bucket holds it.
 - `absent`: the index lacks it and the answer does not need the bucket.
 - `check-store`: the bucket decides.
 */
export type Presence = 'absent' | 'check-store' | 'present';

/**
 Parameters for {@link classifyPresence}.
 */
export type ClassifyPresenceParams = {
  /**
   Objects of the batch, in request order.
   */
  readonly objects: readonly BatchObject[];
  /**
   Operation the client asked for.
   */
  readonly operation: BatchOperation;
  /**
   Worker env exposing the index.
   */
  readonly env: WorkerEnv;
  /**
   Logger for the lookup outcome.
   */
  readonly l: Logger;
};

/**
 Look the batch up in the presence index once.

 An upload treats an index miss as absent: git-lfs then PUTs the object, which
 is idempotent and records it. A download confirms the first few misses against
 the bucket, because reporting a stored object as missing would be wrong.
 Without an index, or when the index query fails, every object is checked
 against the bucket.

 @param objects - objects of the batch, in request order

 @param operation - operation the client asked for

 @param env - Worker env exposing the index

 @param l - logger for the lookup outcome

 @returns one entry per object, aligned with `objects`

 @example
 ```ts
 const presence = await classifyPresence({ objects, operation: 'upload', env, l });
 ```
 */
export async function classifyPresence({
  objects,
  operation,
  env,
  l,
}: ClassifyPresenceParams,): Promise<readonly Presence[]> {
  /**
   Logger tagged with this function's name.
   */
  const cl = tagged({
    tag: classifyPresence.name,
    l,
  },);
  /**
   Index answer, or the reason there is none.
   */
  const lookup = await lookupOrUnavailable({
    objects,
    env,
    l: cl,
  },);
  if (lookup.kind === 'unavailable') {
    return objects.map(function askBucket(): Presence {
      return 'check-store';
    },);
  }
  if (operation === 'upload') {
    return objects.map(function trustIndex(object: BatchObject,): Presence {
      return lookup.known
        .has(object.oid,)
        ? 'present'
        : 'absent';
    },);
  }
  /**
   Oids of this download the index does not hold.
   */
  const misses = objects
    .map(function toOid(object: BatchObject,): string {
      return object.oid;
    },)
    .filter(function isMiss(oid: string,): boolean {
      return !lookup.known
        .has(oid,);
    },);
  /**
   The misses confirmed against the bucket.
   */
  const checked = new Set(misses.slice(
    0,
    DOWNLOAD_MISS_CHECK_LIMIT,
  ),);
  if (misses.length > checked.size) {
    cl.warn(`${misses.length} object(s) missing from the index; confirming ${checked.size} against the bucket, reporting the rest absent`,);
  }
  return objects.map(function classifyDownload(object: BatchObject,): Presence {
    if (lookup.known
      .has(object.oid,)) {
      return 'present';
    }
    return checked.has(object.oid,) ? 'check-store' : 'absent';
  },);
}

/**
 Parameters for {@link lookupOrUnavailable}.
 */
type LookupOrUnavailableParams = {
  /**
   Objects of the batch.
   */
  readonly objects: readonly BatchObject[];
  /**
   Worker env exposing the index.
   */
  readonly env: WorkerEnv;
  /**
   Logger for the failure.
   */
  readonly l: Logger;
};

/**
 Query the index, treating a failing query as no index so the bucket still answers.

 @param objects - objects of the batch

 @param env - Worker env exposing the index

 @param l - logger for the failure

 @returns the index answer, or `unavailable` when there is no index or the query threw
 */
async function lookupOrUnavailable({
  objects,
  env,
  l,
}: LookupOrUnavailableParams,): Promise<PresenceLookup> {
  try {
    return await lookupPresent({
      env,
      oids: objects.map(function toOid(object: BatchObject,): string {
        return object.oid;
      },),
    },);
  }
  catch (error) {
    l.error(`presence index query failed, asking the bucket instead: ${String(error,)}`,);
    return { kind: 'unavailable', };
  }
}

/**
 Parameters for {@link healIndex}.
 */
export type HealIndexParams = {
  /**
   Worker env exposing the index.
   */
  readonly env: WorkerEnv;
  /**
   Oids the bucket confirmed but the index lacked.
   */
  readonly oids: readonly string[];
  /**
   Logger for the outcome.
   */
  readonly l: Logger;
};

/**
 Record oids the bucket confirmed so later batches skip the bucket; a failing write is logged and ignored.

 @param env - Worker env exposing the index

 @param oids - oids the bucket confirmed but the index lacked

 @param l - logger for the outcome

 @example
 ```ts
 await healIndex({ env, oids: [oid], l });
 ```
 */
export async function healIndex({
  env,
  oids,
  l,
}: HealIndexParams,): Promise<void> {
  if ((oids.length === 0) || (env.INDEX === undefined)) {
    return;
  }
  try {
    await recordPresent({
      env,
      oids,
    },);
    l.info(`recorded ${oids.length} confirmed object(s) in the presence index`,);
  }
  catch (error) {
    l.warn(`could not record ${oids.length} confirmed object(s) in the presence index: ${String(error,)}`,);
  }
}
