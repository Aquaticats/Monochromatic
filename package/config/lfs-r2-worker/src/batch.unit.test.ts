/**
 Tests for the git-lfs batch API handler.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
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
          name: 'rejects unparseable JSON with 400 and an LFS JSON message',
          fn: async () => {
            /**
             Response for a body that is not JSON.
             */
            const response = await postBatch({ body: '{not json', },);
            expect(response.status,).toBe(400,);
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
              expect(response.status,).toBe(400,);
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
          name: 'rejects an upload without the token with 401 and LFS-Authenticate',
          fn: async () => {
            /**
             Response for an anonymous upload.
             */
            const response = await postBatch({
              body: { operation: 'upload', objects: [{ oid: ABSENT_OID, size: 3, },], },
            },);
            expect(response.status,).toBe(401,);
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
            expect(response.status,).toBe(401,);
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
