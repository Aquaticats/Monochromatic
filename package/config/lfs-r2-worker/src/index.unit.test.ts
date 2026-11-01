/**
 Tests for request routing and the object routes through the library entry.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  createMemoryObjectStore,
  handleRequest,
  type MemoryObjectStore,
  IMMUTABLE_CACHE_CONTROL,
  LFS_JSON,
  OCTET_STREAM,
  splitPath,
  type WorkerEnv,
} from '@monochromatic-dev/config-lfs-r2-worker';

/**
 Logger shared by every case.
 */
const l = tagged({ tag: 'index.unit.test', },);

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
 Origin every request targets.
 */
const ORIGIN = 'https://lfs.test';

/**
 Repo-relative path suffix a README link would carry.
 */
const PNG_SUFFIX = 'package/music-player/asset/readme/desktop-wide-empty.png';

/**
 Fresh store and env holding only the present object.

 @returns store and env for one case
 */
function fresh(): { readonly store: MemoryObjectStore; readonly env: WorkerEnv; } {
  /**
   Store seeded with the present object.
   */
  const store = createMemoryObjectStore({ [PRESENT_OID]: PRESENT_BYTES, },);
  return { store, env: { BUCKET: store, LFS_WRITE_TOKEN: TOKEN, }, };
}

/**
 Send one request through the router.

 @param path - request path including the leading slash

 @param init - request options

 @param env - env override; defaults to a fresh one

 @returns router response
 */
async function send({
  path,
  init = {},
  env = fresh().env,
}: {
  readonly path: string;
  readonly init?: RequestInit;
  readonly env?: WorkerEnv;
},): Promise<Response> {
  return await handleRequest({ request: new Request(`${ORIGIN}${path}`, init,), env, l, },);
}

/**
 Assert the immutable object headers for the present object.

 @param response - response under test

 @param mediaType - expected content type
 */
function expectObjectHeaders({
  response,
  mediaType,
}: {
  readonly response: Response;
  readonly mediaType: string;
},): void {
  expect(response.headers.get('Content-Type',),).toBe(mediaType,);
  expect(response.headers.get('Content-Length',),).toBe(String(PRESENT_BYTES.byteLength,),);
  expect(response.headers.get('Cache-Control',),).toBe(IMMUTABLE_CACHE_CONTROL,);
  expect(response.headers.get('ETag',),).toBe(`"${PRESENT_OID}"`,);
}

await describe({
  name: '',
  children: [
    describe({
      name: splitPath.name,
      children: [
        it({
          name: 'splits a bare segment with an empty rest',
          fn: async () => {
            expect(splitPath(`/${PRESENT_OID}`,),).toEqual({ first: PRESENT_OID, rest: '', },);
          },
        },),
        it({
          name: 'splits the first segment from a nested suffix',
          fn: async () => {
            expect(splitPath(`/${PRESENT_OID}/${PNG_SUFFIX}`,),).toEqual({ first: PRESENT_OID, rest: PNG_SUFFIX, },);
          },
        },),
        it({
          name: 'keeps an empty rest for a trailing slash',
          fn: async () => {
            expect(splitPath(`/${PRESENT_OID}/`,),).toEqual({ first: PRESENT_OID, rest: '', },);
          },
        },),
        it({
          name: 'returns empty parts for the root path',
          fn: async () => {
            expect(splitPath('/',),).toEqual({ first: '', rest: '', },);
          },
        },),
        it({
          name: 'tolerates a missing leading slash',
          fn: async () => {
            expect(splitPath('objects/batch',),).toEqual({ first: 'objects', rest: 'batch', },);
          },
        },),
      ],
    },),
    describe({
      name: `${handleRequest.name} routing`,
      children: [
        it({
          name: 'dispatches POST /objects/batch to the batch handler',
          fn: async () => {
            /**
             Response for a download batch of the present object.
             */
            const response = await send({
              path: '/objects/batch',
              init: {
                method: 'POST',
                body: JSON.stringify({ operation: 'download', objects: [{ oid: PRESENT_OID, size: 14, },], },),
              },
            },);
            expect(response.status,).toBe(200,);
            expect(response.headers.get('Content-Type',),).toBe(LFS_JSON,);
          },
        },),
        it({
          name: 'answers 404 for a path whose first segment is not an oid',
          fn: async () => {
            expect((await send({ path: '/objects/batch', },)).status,).toBe(404,);
            expect((await send({ path: '/', },)).status,).toBe(404,);
            expect((await send({ path: `/${PRESENT_OID.toUpperCase()}`, },)).status,).toBe(404,);
          },
        },),
        it({
          name: 'answers 405 with Allow for an unsupported method on a bare oid',
          fn: async () => {
            /**
             Response for a DELETE.
             */
            const response = await send({ path: `/${PRESENT_OID}`, init: { method: 'DELETE', }, },);
            expect(response.status,).toBe(405,);
            expect(response.headers.get('Allow',),).toBe('GET, HEAD, PUT',);
          },
        },),
        it({
          name: 'answers 405 for a PUT on a path-suffixed oid',
          fn: async () => {
            /**
             Response for a PUT with a suffix.
             */
            const response = await send({
              path: `/${ABSENT_OID}/${PNG_SUFFIX}`,
              init: { method: 'PUT', body: 'bytes', headers: { Authorization: AUTHORIZATION, }, },
            },);
            expect(response.status,).toBe(405,);
            expect(response.headers.get('Allow',),).toBe('GET, HEAD',);
          },
        },),
      ],
    },),
    describe({
      name: 'GET',
      children: [
        it({
          name: 'streams a present object as octet-stream on the bare oid',
          fn: async () => {
            /**
             Response for the bare oid.
             */
            const response = await send({ path: `/${PRESENT_OID}`, },);
            expect(response.status,).toBe(200,);
            expectObjectHeaders({ response, mediaType: OCTET_STREAM, },);
            expect(
              new Uint8Array(await response.arrayBuffer(),),
            ).toEqual(PRESENT_BYTES,);
          },
        },),
        it({
          name: 'serves the image media type when the path suffix has a known extension',
          fn: async () => {
            /**
             Response for the README-style URL.
             */
            const response = await send({ path: `/${PRESENT_OID}/${PNG_SUFFIX}`, },);
            expect(response.status,).toBe(200,);
            expectObjectHeaders({ response, mediaType: 'image/png', },);
            expect(
              new Uint8Array(await response.arrayBuffer(),),
            ).toEqual(PRESENT_BYTES,);
          },
        },),
        it({
          name: 'answers 404 for an absent object',
          fn: async () => {
            expect((await send({ path: `/${ABSENT_OID}`, },)).status,).toBe(404,);
            expect((await send({ path: `/${ABSENT_OID}/${PNG_SUFFIX}`, },)).status,).toBe(404,);
          },
        },),
      ],
    },),
    describe({
      name: 'HEAD',
      children: [
        it({
          name: 'answers with the object headers and no body',
          fn: async () => {
            /**
             Response for a HEAD of the README-style URL.
             */
            const response = await send({ path: `/${PRESENT_OID}/${PNG_SUFFIX}`, init: { method: 'HEAD', }, },);
            expect(response.status,).toBe(200,);
            expectObjectHeaders({ response, mediaType: 'image/png', },);
            expect(response.body,).toBeNull();
          },
        },),
        it({
          name: 'answers 404 for an absent object',
          fn: async () => {
            expect((await send({ path: `/${ABSENT_OID}`, init: { method: 'HEAD', }, },)).status,).toBe(404,);
          },
        },),
      ],
    },),
    describe({
      name: 'If-None-Match',
      children: [
        ...[
          ['the strong tag', `"${PRESENT_OID}"`,],
          ['a weak tag', `W/"${PRESENT_OID}"`,],
          ['a list containing the tag', `"other", "${PRESENT_OID}"`,],
          ['a wildcard', '*',],
        ].map(function mapCase([label, header,],) {
          return it({
            name: `answers 304 with the object headers for ${label}`,
            fn: async () => {
              /**
               Conditional GET response.
               */
              const response = await send({
                path: `/${PRESENT_OID}/${PNG_SUFFIX}`,
                init: { headers: { 'If-None-Match': String(header,), }, },
              },);
              expect(response.status,).toBe(304,);
              expectObjectHeaders({ response, mediaType: 'image/png', },);
              expect(response.body,).toBeNull();
            },
          },);
        },),
        it({
          name: 'serves the body when the tag does not match',
          fn: async () => {
            /**
             Conditional GET response with a stale tag.
             */
            const response = await send({
              path: `/${PRESENT_OID}`,
              init: { headers: { 'If-None-Match': '"stale"', }, },
            },);
            expect(response.status,).toBe(200,);
            expect(
              new Uint8Array(await response.arrayBuffer(),),
            ).toEqual(PRESENT_BYTES,);
          },
        },),
        it({
          name: 'answers 404 for an absent object even when the tag matches',
          fn: async () => {
            /**
             Conditional GET response for an absent object.
             */
            const response = await send({
              path: `/${ABSENT_OID}`,
              init: { headers: { 'If-None-Match': `"${ABSENT_OID}"`, }, },
            },);
            expect(response.status,).toBe(404,);
          },
        },),
      ],
    },),
    describe({
      name: 'PUT',
      children: [
        it({
          name: 'rejects an unauthorized write with 401 and stores nothing',
          fn: async () => {
            const { store, env, } = fresh();
            /**
             Response for an anonymous PUT.
             */
            const response = await send({
              path: `/${ABSENT_OID}`,
              init: { method: 'PUT', body: 'new bytes', },
              env,
            },);
            expect(response.status,).toBe(401,);
            expect(store.objects.has(ABSENT_OID,),).toBe(false,);
          },
        },),
        it({
          name: 'stores the body under the oid for an authorized write',
          fn: async () => {
            const { store, env, } = fresh();
            /**
             Bytes uploaded.
             */
            const uploaded = new TextEncoder().encode('new bytes',);
            /**
             Response for the authorized PUT.
             */
            const response = await send({
              path: `/${ABSENT_OID}`,
              init: { method: 'PUT', body: uploaded, headers: { Authorization: AUTHORIZATION, }, },
              env,
            },);
            expect(response.status,).toBe(200,);
            expect(store.objects.get(ABSENT_OID,),).toEqual(uploaded,);
          },
        },),
        it({
          name: 'rejects an authorized write without a body with 400',
          fn: async () => {
            const { store, env, } = fresh();
            /**
             Response for a body-less PUT.
             */
            const response = await send({
              path: `/${ABSENT_OID}`,
              init: { method: 'PUT', headers: { Authorization: AUTHORIZATION, }, },
              env,
            },);
            expect(response.status,).toBe(400,);
            expect(store.objects.has(ABSENT_OID,),).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);
