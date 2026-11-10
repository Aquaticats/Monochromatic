/**
 Tests for the cost of a batch: R2 lookups and log records per object, and the presence index that keeps both flat.

 The Workers Free plan allows 10 ms of CPU per request, and an R2 `head` measured 0.25 to 0.45 ms of CPU on the
 live account, so the number of `head` calls a batch makes is the quantity these tests pin.

 @module
 */

import type { Logger, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  createMemoryObjectStore,
  createMemoryPresenceIndex,
  DOWNLOAD_MISS_CHECK_LIMIT,
  handleBatch,
  INDEX_CHUNK_SIZE,
  LFS_JSON,
  type MemoryPresenceIndex,
  SELECT_PRESENT_SQL,
  storeObject,
  type WorkerEnv,
} from '@monochromatic-dev/config-lfs-r2-worker';

/**
 Upload secret configured on the env under test.
 */
const TOKEN = 'secret';

/**
 Basic credential carrying the token.
 */
const AUTHORIZATION = `Basic ${btoa(`lfs:${TOKEN}`,)}`;

/**
 Batch endpoint URL git-lfs posts to.
 */
const BATCH_URL = 'https://lfs.test/objects/batch';

/**
 Batch size git-lfs sends by default.
 */
const GIT_LFS_DEFAULT_BATCH = 100;

/**
 Distinct oid for position `position`.

 @param position - index of the object

 @returns 64-character lowercase hex oid
 */
function oidAt(position: number,): string {
  return `${'0'.repeat(60,)}${position.toString(16,)
    .padStart(
      4,
      '0',
    )}`;
}

/**
 Distinct oids `0` to `count - 1`.

 @param count - how many oids

 @returns oids in order
 */
function oids(count: number,): readonly string[] {
  return Array.from({ length: count, }, function toOid(_unused: unknown, position: number,): string {
    return oidAt(position,);
  },);
}

/**
 Everything a batch costs the platform that a test can observe from outside.
 */
type BatchCost = {
  /**
   Messages passed to any logger level, in call order.
   */
  readonly logged: readonly string[];
  /**
   Keys passed to the bucket's `head`, in call order.
   */
  readonly heads: readonly string[];
  /**
   Parsed `objects` of the response.
   */
  readonly entries: readonly Record<string, unknown>[];
  /**
   Index used by the run, when there was one.
   */
  readonly index?: MemoryPresenceIndex;
};

/**
 Parameters for {@link runBatch}.
 */
type RunBatchParams = {
  /**
   Operation the client asks for.
   */
  readonly operation: 'download' | 'upload';
  /**
   Oids the request names.
   */
  readonly requested: readonly string[];
  /**
   Oids the bucket holds.
   */
  readonly stored?: readonly string[];
  /**
   Oids the index holds, or `undefined` for an env with no index.
   */
  readonly indexed?: readonly string[];
  /**
   Error every index statement throws.
   */
  readonly indexFailure?: Error;
};

/**
 Run one batch against a recording logger and a recording store.

 @param operation - operation the client asks for

 @param requested - oids the request names

 @param stored - oids the bucket holds

 @param indexed - oids the index holds; omit for an env with no index

 @param indexFailure - error every index statement throws

 @returns what the batch cost and answered
 */
async function runBatch({
  operation,
  requested,
  stored = [],
  indexed,
  indexFailure,
}: RunBatchParams,): Promise<BatchCost> {
  /**
   Messages captured from every level.
   */
  const logged: string[] = [];
  /**
   Keys the bucket was asked about.
   */
  const heads: string[] = [];
  /**
   Capture one message.

   @param message - log message
   */
  function capture(message: string,): void {
    logged.push(message,);
  }
  /**
   Logger recording every call instead of writing it.
   */
  const recording: Logger = {
    debug: capture,
    error: capture,
    fatal: capture,
    flush: async function flush(): Promise<void> {},
    info: capture,
    trace: capture,
    warn: capture,
  };
  /**
   Bucket seeded with the stored objects.
   */
  const seeded = stored.map(function toEntry(oid: string,) {
    return [oid, new TextEncoder().encode('bytes',),] as const;
  },);
  /**
   Bucket seeded with the stored objects.
   */
  const store = createMemoryObjectStore(Object.fromEntries(seeded,),);
  /**
   Index for this run, when the env has one.
   */
  const index = (indexed === undefined) && (indexFailure === undefined)
    ? undefined
    : createMemoryPresenceIndex({
      initial: indexed ?? [],
      ...indexFailure === undefined ? {} : { failure: indexFailure, },
    },);
  /**
   Env whose bucket records each lookup.
   */
  const env: WorkerEnv = {
    BUCKET: {
      ...store,
      head: function head(key: string,) {
        heads.push(key,);
        return store.head(key,);
      },
    },
    LFS_WRITE_TOKEN: TOKEN,
    ...index === undefined ? {} : { INDEX: index, },
  };
  /**
   Response of the batch handler.
   */
  const response = await handleBatch({
    request: new Request(BATCH_URL, {
      method: 'POST',
      headers: { 'Content-Type': LFS_JSON, Authorization: AUTHORIZATION, },
      body: JSON.stringify({
        operation,
        objects: requested.map(function toObject(oid: string,) {
          return { oid, size: 5, };
        },),
      },),
    },),
    env,
    url: new URL(BATCH_URL,),
    l: recording,
  },);
  /**
   Parsed response body.
   */
  const body = await response.json() as { readonly objects: readonly Record<string, unknown>[]; };
  return {
    logged,
    heads,
    entries: body.objects,
    ...index === undefined ? {} : { index, },
  };
}

await describe({
  name: 'batch cost',
  children: [
    describe({
      name: 'upload with a presence index',
      children: [
        it({
          name: 'asks the bucket nothing when the index covers a default-size batch, and issues no action',
          fn: async () => {
            /**
             Oids of one default-size batch.
             */
            const requested = oids(GIT_LFS_DEFAULT_BATCH,);
            /**
             Cost of a batch whose objects the index holds.
             */
            const cost = await runBatch({
              operation: 'upload',
              requested,
              stored: requested,
              indexed: requested,
            },);
            expect(cost.heads,).toHaveLength(0,);
            expect(cost.entries,).toHaveLength(requested.length,);
            expect(cost.entries.every(function hasNoAction(entry,): boolean {
              return !('actions' in entry);
            },),).toBe(true,);
          },
        },),
        it({
          name: 'asks the bucket nothing for a batch twice the default size, in one query per chunk',
          fn: async () => {
            /**
             Oids of a batch larger than one index chunk.
             */
            const requested = oids(INDEX_CHUNK_SIZE + 1,);
            /**
             Cost of a batch the index covers.
             */
            const cost = await runBatch({
              operation: 'upload',
              requested,
              stored: requested,
              indexed: requested,
            },);
            expect(cost.heads,).toHaveLength(0,);
            expect(cost.index?.statements.filter(function isLookup(sql: string,): boolean {
              return sql === SELECT_PRESENT_SQL;
            },),).toHaveLength(2,);
          },
        },),
        it({
          name: 'issues an upload action without asking the bucket for an object the index lacks',
          fn: async () => {
            /**
             Oids of one default-size batch.
             */
            const requested = oids(GIT_LFS_DEFAULT_BATCH,);
            /**
             Cost of a batch whose objects are all new.
             */
            const cost = await runBatch({
              operation: 'upload',
              requested,
              indexed: [],
            },);
            expect(cost.heads,).toHaveLength(0,);
            expect(cost.entries.every(function hasUploadAction(entry,): boolean {
              return 'actions' in entry;
            },),).toBe(true,);
          },
        },),
        it({
          name: 'writes the same number of log records for one object as for a default-size batch',
          fn: async () => {
            /**
             Cost of the smallest batch.
             */
            const one = await runBatch({
              operation: 'upload',
              requested: oids(1,),
              indexed: [],
            },);
            /**
             Cost of a default-size batch.
             */
            const many = await runBatch({
              operation: 'upload',
              requested: oids(GIT_LFS_DEFAULT_BATCH,),
              indexed: [],
            },);
            expect(many.logged,).toHaveLength(one.logged.length,);
          },
        },),
        it({
          name: 'summarizes the batch outcome in one record',
          fn: async () => {
            /**
             Cost of a batch whose objects are all new.
             */
            const cost = await runBatch({
              operation: 'upload',
              requested: oids(3,),
              indexed: [],
            },);
            expect(cost.logged.some(function mentionsOutcome(message: string,): boolean {
              return message.includes('resolved 3 object(s): 3 with an action, 0 without an action, 0 reported absent',);
            },),).toBe(true,);
          },
        },),
      ],
    },),
    describe({
      name: 'without a usable index',
      children: [
        it({
          name: 'asks the bucket once per object when the env has no index, and answers correctly',
          fn: async () => {
            /**
             Oids of one default-size batch.
             */
            const requested = oids(GIT_LFS_DEFAULT_BATCH,);
            /**
             Cost of a batch the bucket holds.
             */
            const cost = await runBatch({
              operation: 'upload',
              requested,
              stored: requested,
            },);
            expect(cost.heads,).toEqual(requested,);
            expect(cost.entries.every(function hasNoAction(entry,): boolean {
              return !('actions' in entry);
            },),).toBe(true,);
          },
        },),
        it({
          name: 'falls back to the bucket when the index query fails, and logs the failure',
          fn: async () => {
            /**
             Oids of a small batch.
             */
            const requested = oids(4,);
            /**
             Cost of a batch whose index query throws.
             */
            const cost = await runBatch({
              operation: 'upload',
              requested,
              stored: requested,
              indexFailure: new Error('d1 unavailable',),
            },);
            expect(cost.heads,).toEqual(requested,);
            expect(cost.logged.some(function namesFailure(message: string,): boolean {
              return message.includes('d1 unavailable',);
            },),).toBe(true,);
          },
        },),
      ],
    },),
    describe({
      name: 'download with a presence index',
      children: [
        it({
          name: 'issues download actions without asking the bucket when the index holds every object',
          fn: async () => {
            /**
             Oids of one default-size batch.
             */
            const requested = oids(GIT_LFS_DEFAULT_BATCH,);
            /**
             Cost of a download the index covers.
             */
            const cost = await runBatch({
              operation: 'download',
              requested,
              stored: requested,
              indexed: requested,
            },);
            expect(cost.heads,).toHaveLength(0,);
            expect(cost.entries.every(function hasDownloadAction(entry,): boolean {
              return 'actions' in entry;
            },),).toBe(true,);
          },
        },),
        it({
          name: 'confirms an index miss against the bucket, serves the stored object, and records it in the index',
          fn: async () => {
            /**
             The one object the bucket holds but the index lacks.
             */
            const [unindexed,] = oids(1,);
            /**
             Cost of a download of an unindexed stored object.
             */
            const cost = await runBatch({
              operation: 'download',
              requested: [unindexed ?? '',],
              stored: [unindexed ?? '',],
              indexed: [],
            },);
            expect(cost.heads,).toEqual([unindexed,],);
            expect(cost.entries[0],).toHaveProperty('actions',);
            expect(cost.index?.oids.has(unindexed ?? '',),).toBe(true,);
          },
        },),
        it({
          name: 'reports an index miss the bucket also lacks as a per-object 404 and records nothing',
          fn: async () => {
            /**
             Cost of a download of an object nobody holds.
             */
            const cost = await runBatch({
              operation: 'download',
              requested: oids(1,),
              indexed: [],
            },);
            expect(cost.entries[0],).toHaveProperty('error.code', 404,);
            expect(cost.index?.oids.size,).toBe(0,);
          },
        },),
        it({
          name: 'confirms at most the miss limit against the bucket and reports the rest absent',
          fn: async () => {
            /**
             More misses than the limit allows to check.
             */
            const requested = oids(DOWNLOAD_MISS_CHECK_LIMIT + 5,);
            /**
             Cost of a download whose objects the index lacks.
             */
            const cost = await runBatch({
              operation: 'download',
              requested,
              stored: requested,
              indexed: [],
            },);
            expect(cost.heads,).toHaveLength(DOWNLOAD_MISS_CHECK_LIMIT,);
            expect(cost.entries,).toHaveLength(requested.length,);
            expect(cost.entries.filter(function hasDownloadAction(entry,): boolean {
              return 'actions' in entry;
            },),).toHaveLength(DOWNLOAD_MISS_CHECK_LIMIT,);
            expect(cost.entries.filter(function hasError(entry,): boolean {
              return 'error' in entry;
            },),).toHaveLength(requested.length - DOWNLOAD_MISS_CHECK_LIMIT,);
          },
        },),
      ],
    },),
    describe({
      name: 'PUT',
      children: [
        it({
          name: 'records the stored oid in the index after the bucket accepted it',
          fn: async () => {
            /**
             Oid being uploaded.
             */
            const [oid,] = oids(1,);
            /**
             Index starting empty.
             */
            const index = createMemoryPresenceIndex();
            /**
             Bucket that accepts the object.
             */
            const store = createMemoryObjectStore();
            /**
             Response of the PUT.
             */
            const response = await storeObject({
              oid: oid ?? '',
              request: new Request(`https://lfs.test/${oid}`, {
                method: 'PUT',
                headers: { Authorization: AUTHORIZATION, },
                body: 'bytes',
              },),
              env: { BUCKET: store, INDEX: index, LFS_WRITE_TOKEN: TOKEN, },
              l: { debug() {}, error() {}, fatal() {}, flush: async () => {}, info() {}, trace() {}, warn() {}, },
            },);
            expect(response.status,).toBe(200,);
            expect(store.objects.has(oid ?? '',),).toBe(true,);
            expect(index.oids.has(oid ?? '',),).toBe(true,);
          },
        },),
        it({
          name: 'still answers 200 and keeps the object when the index write fails',
          fn: async () => {
            /**
             Oid being uploaded.
             */
            const [oid,] = oids(1,);
            /**
             Bucket that accepts the object.
             */
            const store = createMemoryObjectStore();
            /**
             Response of the PUT with a failing index.
             */
            const response = await storeObject({
              oid: oid ?? '',
              request: new Request(`https://lfs.test/${oid}`, {
                method: 'PUT',
                headers: { Authorization: AUTHORIZATION, },
                body: 'bytes',
              },),
              env: {
                BUCKET: store,
                INDEX: createMemoryPresenceIndex({ failure: new Error('d1 unavailable',), },),
                LFS_WRITE_TOKEN: TOKEN,
              },
              l: { debug() {}, error() {}, fatal() {}, flush: async () => {}, info() {}, trace() {}, warn() {}, },
            },);
            expect(response.status,).toBe(200,);
            expect(store.objects.has(oid ?? '',),).toBe(true,);
          },
        },),
        it({
          name: 'leaves the index untouched when the bucket rejects the object',
          fn: async () => {
            /**
             Oid being uploaded.
             */
            const [oid,] = oids(1,);
            /**
             Index starting empty.
             */
            const index = createMemoryPresenceIndex();
            /**
             Failure surfaced by the handler.
             */
            let caught: unknown;
            try {
              await storeObject({
                oid: oid ?? '',
                request: new Request(`https://lfs.test/${oid}`, {
                  method: 'PUT',
                  headers: { Authorization: AUTHORIZATION, },
                  body: 'bytes',
                },),
                env: {
                  BUCKET: {
                    ...createMemoryObjectStore(),
                    put: async function put(): Promise<never> {
                      throw new Error('bucket unavailable',);
                    },
                  },
                  INDEX: index,
                  LFS_WRITE_TOKEN: TOKEN,
                },
                l: { debug() {}, error() {}, fatal() {}, flush: async () => {}, info() {}, trace() {}, warn() {}, },
              },);
            }
            catch (error) {
              caught = error;
            }
            expect((caught as Error).message,).toBe('bucket unavailable',);
            expect(index.oids.size,).toBe(0,);
          },
        },),
      ],
    },),
  ],
},);
