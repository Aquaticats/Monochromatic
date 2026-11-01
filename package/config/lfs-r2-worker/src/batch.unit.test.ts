/**
 Tests for the git-lfs batch API handler.

 @module
 */

import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  BATCH_BAD_REQUEST,
  BATCH_UNAUTHORIZED,
  BatchRequestError,
  createMemoryObjectStore,
  handleBatch,
  LFS_JSON,
  parseBatchRequest,
  type WorkerEnv,
} from '@monochromatic-dev/config-lfs-r2-worker';

/**
 Logger shared by every case.
 */
const l = tagged({ tag: 'batch.unit.test', },);

/**
 Upload secret configured on the env under test.
 */
const TOKEN = 'secret';

/**
 Basic credential carrying the token.
 */
const AUTHORIZATION = `Basic ${btoa(`lfs:${TOKEN}`,)}`;

/**
 oid of the object seeded into the store.
 */
const PRESENT_OID = 'a'.repeat(64,);

/**
 oid of an object absent from the store.
 */
const ABSENT_OID = 'b'.repeat(64,);

/**
 Bytes seeded under {@link PRESENT_OID}.
 */
const PRESENT_BYTES = new TextEncoder().encode('present object',);

/**
 Batch endpoint URL git-lfs posts to.
 */
const BATCH_URL = 'https://lfs.test/objects/batch';

/**
 Fresh env whose store holds only the present object.

 @returns env for one case
 */
function freshEnv(): WorkerEnv {
  return {
    BUCKET: createMemoryObjectStore({ [PRESENT_OID]: PRESENT_BYTES, },),
    LFS_WRITE_TOKEN: TOKEN,
  };
}

/**
 Post a batch body and return the response.

 @param body - raw request body; objects are serialized, strings sent verbatim

 @param authorization - optional `Authorization` header

 @param env - env override; defaults to a fresh one

 @returns handler response
 */
async function postBatch({
  body,
  authorization,
  env = freshEnv(),
}: {
  readonly body: unknown;
  readonly authorization?: string;
  readonly env?: WorkerEnv;
},): Promise<Response> {
  /**
   Request against the batch endpoint.
   */
  const request = new Request(BATCH_URL, {
    method: 'POST',
    headers: {
      'Content-Type': LFS_JSON,
      ...authorization === undefined ? {} : { Authorization: authorization, },
    },
    body: (typeof body) === 'string' ? body : JSON.stringify(body,),
  },);
  return await handleBatch({ request, env, url: new URL(BATCH_URL,), l, },);
}

/**
 Everything a batch costs the platform, observable from outside: log records written and store lookups made.
 */
type BatchCost = {
  /**
   Messages passed to any logger level, in call order.
   */
  readonly logged: readonly string[];
  /**
   Keys passed to `head`, in call order.
   */
  readonly heads: readonly string[];
};

/**
 Run an upload batch of `count` distinct absent objects and record its cost.

 The Workers Free plan allows 10 ms of CPU per request, so the work a batch
 does per object (log records, store lookups) is the quantity to pin.

 @param count - number of distinct objects in the batch

 @returns log messages and `head` keys the batch produced
 */
async function measureBatchCost(count: number,): Promise<BatchCost> {
  /**
   Messages captured from every level.
   */
  const logged: string[] = [];
  /**
   Keys the store was asked about.
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
   Store that records each lookup.
   */
  const store = createMemoryObjectStore();
  /**
   Env whose store records each lookup.
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
  };
  /**
   Distinct oids, one per requested object.
   */
  const objects = Array.from({ length: count, }, function toObject(_, index,) {
    return {
      oid: `${'0'.repeat(60,)}${index.toString(16,).padStart(4, '0',)}`,
      size: 1,
    };
  },);
  await handleBatch({
    request: new Request(BATCH_URL, {
      method: 'POST',
      headers: { 'Content-Type': LFS_JSON, Authorization: AUTHORIZATION, },
      body: JSON.stringify({ operation: 'upload', objects, },),
    },),
    env,
    url: new URL(BATCH_URL,),
    l: recording,
  },);
  return { logged, heads, };
}

/**
 Parsed batch response entries.
 */
type BatchResponse = {
  readonly transfer: string;
  readonly objects: readonly Record<string, unknown>[];
};

await describe({
  name: '',
  children: [
    describe({
      name: parseBatchRequest.name,
      children: [
        it({
          name: 'returns the operation and objects of a well-formed body',
          fn: async () => {
            expect(parseBatchRequest({
              operation: 'download',
              transfers: ['basic',],
              objects: [{ oid: PRESENT_OID, size: 14, },],
            },),).toEqual({
              operation: 'download',
              objects: [{ oid: PRESENT_OID, size: 14, },],
            },);
          },
        },),
        it({
          name: 'throws a named BatchRequestError',
          fn: async () => {
            /**
             Error thrown for a non-object body.
             */
            let caught: unknown;
            try {
              parseBatchRequest([],);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(BatchRequestError,);
            expect((caught as Error).name,).toBe('BatchRequestError',);
          },
        },),
      ],
    },),
    describe({
      name: `${handleBatch.name} validation`,
      children: [
        it({
          name: 'rejects unparseable JSON as a bad request with an LFS JSON message',
          fn: async () => {
            /**
             Response for a body that is not JSON.
             */
            const response = await postBatch({ body: '{not json', },);
            expect(response.status,).toBe(BATCH_BAD_REQUEST,);
            expect(response.headers.get('Content-Type',),).toBe(LFS_JSON,);
            expect(await response.json(),).toEqual({ message: 'batch request body must be valid JSON', },);
          },
        },),
        ...([
          [[], 'batch request body must be a JSON object',],
          [{ operation: 'verify', objects: [], }, 'operation must be "download" or "upload"',],
          [{ operation: 'download', objects: {}, }, 'objects must be an array',],
          [{ operation: 'download', objects: ['x',], }, 'each object must be an object',],
          [{ operation: 'download', objects: [{ oid: 'short', size: 1, },], }, 'each object needs a 64-character lowercase hex oid',],
          [{ operation: 'download', objects: [{ oid: PRESENT_OID, size: -1, },], }, `object ${PRESENT_OID} needs a non-negative integer size`,],
          [{ operation: 'download', objects: [{ oid: PRESENT_OID, size: 1.5, },], }, `object ${PRESENT_OID} needs a non-negative integer size`,],
        ] as const).map(function mapCase([body, message,],) {
          return it({
            name: `rejects ${JSON.stringify(body,)} with "${message}"`,
            fn: async () => {
              /**
               Response for the invalid body.
               */
              const response = await postBatch({ body, },);
              expect(response.status,).toBe(BATCH_BAD_REQUEST,);
              expect(await response.json(),).toEqual({ message, },);
            },
          },);
        },),
      ],
    },),
    describe({
      name: `${handleBatch.name} download`,
      children: [
        it({
          name: 'issues a download action for a present object and a 404 error for an absent one',
          fn: async () => {
            /**
             Response for one present and one absent object.
             */
            const response = await postBatch({
              body: {
                operation: 'download',
                objects: [{ oid: PRESENT_OID, size: 14, }, { oid: ABSENT_OID, size: 3, },],
              },
            },);
            expect(response.status,).toBe(200,);
            expect(response.headers.get('Content-Type',),).toBe(LFS_JSON,);
            /**
             Parsed batch response.
             */
            const parsed = await response.json() as BatchResponse;
            expect(parsed.transfer,).toBe('basic',);
            expect(parsed.objects,).toEqual([
              {
                oid: PRESENT_OID,
                size: 14,
                actions: { download: { href: `https://lfs.test/${PRESENT_OID}`, }, },
              },
              {
                oid: ABSENT_OID,
                size: 3,
                error: { code: 404, message: 'Object not found', },
              },
            ],);
          },
        },),
        it({
          name: 'needs no credentials',
          fn: async () => {
            /**
             Response for an anonymous download of the present object.
             */
            const response = await postBatch({
              body: { operation: 'download', objects: [{ oid: PRESENT_OID, size: 14, },], },
            },);
            expect(response.status,).toBe(200,);
          },
        },),
        it({
          name: 'returns an empty object list for an empty request',
          fn: async () => {
            /**
             Response for no objects.
             */
            const response = await postBatch({ body: { operation: 'download', objects: [], }, },);
            expect(await response.json(),).toEqual({ transfer: 'basic', objects: [], },);
          },
        },),
      ],
    },),
    describe({
      name: `${handleBatch.name} upload`,
      children: [
        it({
          name: 'rejects an upload without the token as unauthorized and names LFS-Authenticate',
          fn: async () => {
            /**
             Response for an anonymous upload.
             */
            const response = await postBatch({
              body: { operation: 'upload', objects: [{ oid: ABSENT_OID, size: 3, },], },
            },);
            expect(response.status,).toBe(BATCH_UNAUTHORIZED,);
            expect(response.headers.get('LFS-Authenticate',),).toBe('Basic realm="monochromatic-lfs"',);
          },
        },),
        it({
          name: 'rejects an upload with a wrong token',
          fn: async () => {
            /**
             Response for a wrong password.
             */
            const response = await postBatch({
              body: { operation: 'upload', objects: [{ oid: ABSENT_OID, size: 3, },], },
              authorization: `Basic ${btoa('lfs:wrong',)}`,
            },);
            expect(response.status,).toBe(BATCH_UNAUTHORIZED,);
          },
        },),
        it({
          name: 'issues an upload action echoing the credential for an absent object and no action for a present one',
          fn: async () => {
            /**
             Response for one present and one absent object.
             */
            const response = await postBatch({
              body: {
                operation: 'upload',
                objects: [{ oid: PRESENT_OID, size: 14, }, { oid: ABSENT_OID, size: 3, },],
              },
              authorization: AUTHORIZATION,
            },);
            expect(response.status,).toBe(200,);
            /**
             Parsed batch response.
             */
            const parsed = await response.json() as BatchResponse;
            expect(parsed.objects,).toEqual([
              { oid: PRESENT_OID, size: 14, },
              {
                oid: ABSENT_OID,
                size: 3,
                actions: {
                  upload: {
                    href: `https://lfs.test/${ABSENT_OID}`,
                    header: { Authorization: AUTHORIZATION, },
                  },
                },
              },
            ],);
          },
        },),
      ],
    },),
    describe({
      name: `${handleBatch.name} cost per object`,
      children: [
        it({
          name: 'writes the same number of log records for one object as for two hundred',
          fn: async () => {
            /**
             Cost of the smallest batch.
             */
            const one = await measureBatchCost(1,);
            /**
             Cost of a batch twice git-lfs's default size.
             */
            const many = await measureBatchCost(200,);
            expect(many.logged,).toHaveLength(one.logged.length,);
          },
        },),
        it({
          name: 'looks each object up in the store exactly once',
          fn: async () => {
            /**
             Cost of a batch of distinct objects.
             */
            const cost = await measureBatchCost(200,);
            expect(cost.heads,).toHaveLength(200,);
            expect(new Set(cost.heads,).size,).toBe(200,);
          },
        },),
        it({
          name: 'summarizes the batch outcome in one record',
          fn: async () => {
            /**
             Cost of a batch whose objects are all absent.
             */
            const cost = await measureBatchCost(3,);
            expect(cost.logged.some(function mentionsOutcome(message,): boolean {
              return message.includes('resolved 3 object(s): 3 with an action, 0 without an action, 0 reported absent',);
            },),).toBe(true,);
          },
        },),
      ],
    },),
    describe({
      name: `${handleBatch.name} store failure`,
      children: [
        it({
          name: 'rethrows an unexpected store failure instead of answering',
          fn: async () => {
            /**
             Env whose store fails every head lookup.
             */
            const failing: WorkerEnv = {
              BUCKET: {
                ...createMemoryObjectStore(),
                head: async function head(): Promise<never> {
                  throw new Error('bucket unavailable',);
                },
              },
            };
            /**
             Failure surfaced by the handler.
             */
            let caught: unknown;
            try {
              await postBatch({
                body: { operation: 'download', objects: [{ oid: PRESENT_OID, size: 14, },], },
                env: failing,
              },);
            }
            catch (error) {
              caught = error;
            }
            expect((caught as Error).message,).toBe('bucket unavailable',);
          },
        },),
      ],
    },),
  ],
},);
