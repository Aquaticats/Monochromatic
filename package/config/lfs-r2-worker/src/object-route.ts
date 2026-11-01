/**
 Object routes: stream, head, or store one object addressed by its oid.

 Objects are content-addressed, so every successful response is immutable and
 carries a strong `ETag` equal to the quoted oid; a matching `If-None-Match`
 short-circuits to 304 without touching the store body.

 @module
 */

import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';

import { authorized, } from './authorize.ts';
import type { WorkerEnv, } from './store.ts';

/**
 Cache policy for content-addressed objects: a year, and never revalidate.
 */
export const IMMUTABLE_CACHE_CONTROL = 'public, max-age=31536000, immutable';

/**
 HTTP status for a conditional request whose validator still matches.
 */
const NOT_MODIFIED = 304;

/**
 HTTP status for a write whose body is missing.
 */
const BAD_REQUEST = 400;

/**
 HTTP status for a write without the upload token.
 */
const UNAUTHORIZED = 401;

/**
 HTTP status for an absent object.
 */
const NOT_FOUND = 404;

/**
 Strong entity tag for an object: its oid in double quotes.

 @param oid - sha256 object id

 @returns quoted oid
 */
function etagFor(oid: string,): string {
  return `"${oid}"`;
}

/**
 Parameters for {@link clientHoldsObject}.
 */
type ClientHoldsObjectParams = {
  /**
   Inbound request whose `If-None-Match` header is inspected.
   */
  readonly request: Request;
  /**
   sha256 object id the response would carry.
   */
  readonly oid: string;
};

/**
 Whether the request's `If-None-Match` header names the object's entity tag.

 Accepts a comma-separated list, weak validators, and `*`, per RFC 9110.

 @param request - inbound request whose `If-None-Match` header is inspected

 @param oid - sha256 object id the response would carry

 @returns `true` when the client already holds the object
 */
function clientHoldsObject({
  request,
  oid,
}: ClientHoldsObjectParams,): boolean {
  if (!request.headers
    .has('If-None-Match',)) {
    return false;
  }
  /**
   Raw header value; presence was established just above.
   */
  const header = String(request.headers
    .get('If-None-Match',),);
  if (header.trim() === '*') {
    return true;
  }
  /**
   Expected strong tag.
   */
  const etag = etagFor(oid,);
  return header
    .split(',',)
    .map(function trim(candidate: string,): string {
      return candidate.trim();
    },)
    .some(function matches(candidate: string,): boolean {
      return (candidate === etag) || (candidate === `W/${etag}`);
    },);
}

/**
 Parameters for {@link objectHeaders}.
 */
export type ObjectHeadersParams = {
  /**
   sha256 object id, used as the entity tag.
   */
  readonly oid: string;
  /**
   Byte length of the object.
   */
  readonly size: number;
  /**
   Media type decided from the requested path suffix.
   */
  readonly mediaType: string;
};

/**
 Response headers shared by GET, HEAD, and 304 responses for an object.

 @param oid - sha256 object id, used as the entity tag

 @param size - byte length of the object

 @param mediaType - media type decided from the requested path suffix

 @returns headers with content type, length, cache policy, and entity tag

 @example
 ```ts
 objectHeaders({ oid: 'a'.repeat(64), size: 12, mediaType: 'image/png' });
 ```
 */
export function objectHeaders({
  oid,
  size,
  mediaType,
}: ObjectHeadersParams,): Headers {
  return new Headers({
    'Content-Type': mediaType,
    'Content-Length': String(size,),
    'Cache-Control': IMMUTABLE_CACHE_CONTROL,
    ETag: etagFor(oid,),
  },);
}

/**
 Parameters for {@link serveObject}.
 */
export type ServeObjectParams = {
  /**
   sha256 object id used as the store key.
   */
  readonly oid: string;
  /**
   Media type decided from the requested path suffix.
   */
  readonly mediaType: string;
  /**
   Inbound GET or HEAD request, inspected for `If-None-Match`.
   */
  readonly request: Request;
  /**
   Worker env exposing the store.
   */
  readonly env: WorkerEnv;
  /**
   `true` for GET, `false` for HEAD.
   */
  readonly includeBody: boolean;
  /**
   Logger for the serve decision.
   */
  readonly l: Logger;
};

/**
 Stream a stored object back to the caller, answer a HEAD with its headers
 only, or 404 when absent.

 @param oid - sha256 object id used as the store key

 @param mediaType - media type decided from the requested path suffix

 @param request - inbound GET or HEAD request, inspected for `If-None-Match`

 @param env - Worker env exposing the store

 @param includeBody - `true` for GET, `false` for HEAD

 @param l - logger for the serve decision

 @returns 200 with body (GET), 200 without body (HEAD), 304, or 404

 @example
 ```ts
 serveObject({ oid, mediaType: 'image/png', request, env, includeBody: request.method === 'GET', l });
 ```
 */
export async function serveObject({
  oid,
  mediaType,
  request,
  env,
  includeBody,
  l,
}: ServeObjectParams,): Promise<Response> {
  /**
   Logger tagged with this function's name.
   */
  const sl = tagged({
    tag: serveObject.name,
    l,
  },);
  if (clientHoldsObject({
    request,
    oid,
  },)) {
    /**
     Size for the 304 headers; absent objects still 404 so a stale validator
     cannot claim a deleted object exists.
     */
    const head = await env.BUCKET
      .head(oid,);
    if (head === null) {
      sl.info(`${oid} absent on conditional request`,);
      return new Response(
        'Not found',
        { status: NOT_FOUND, },
      );
    }
    sl.debug(`${oid} not modified`,);
    return new Response(
      null,
      {
        status: NOT_MODIFIED,
        headers: objectHeaders({
          oid,
          size: head.size,
          mediaType,
        },),
      },
    );
  }
  if (!includeBody) {
    /**
     HEAD needs only metadata, so skip reading the body from the store.
     */
    const head = await env.BUCKET
      .head(oid,);
    if (head === null) {
      sl.info(`${oid} absent on HEAD`,);
      return new Response(
        'Not found',
        { status: NOT_FOUND, },
      );
    }
    sl.debug(`${oid} HEAD as ${mediaType}, ${head.size} bytes`,);
    return new Response(
      null,
      {
        status: 200,
        headers: objectHeaders({
          oid,
          size: head.size,
          mediaType,
        },),
      },
    );
  }
  /**
   Object with body; `null` when absent.
   */
  const object = await env.BUCKET
    .get(oid,);
  if (object === null) {
    sl.info(`${oid} absent on GET`,);
    return new Response(
      'Not found',
      { status: NOT_FOUND, },
    );
  }
  sl.debug(`${oid} GET as ${mediaType}, ${object.size} bytes`,);
  return new Response(
    object.body,
    {
      status: 200,
      headers: objectHeaders({
        oid,
        size: object.size,
        mediaType,
      },),
    },
  );
}

/**
 Parameters for {@link storeObject}.
 */
export type StoreObjectParams = {
  /**
   sha256 object id used as the store key.
   */
  readonly oid: string;
  /**
   Upload PUT whose body is the object bytes.
   */
  readonly request: Request;
  /**
   Worker env exposing the store and upload secret.
   */
  readonly env: WorkerEnv;
  /**
   Logger for the store decision.
   */
  readonly l: Logger;
};

/**
 Store an uploaded object under its oid after checking the upload token.

 @param oid - sha256 object id used as the store key

 @param request - upload PUT whose body is the object bytes

 @param env - Worker env exposing the store and upload secret

 @param l - logger for the store decision

 @returns empty 200 on success, 401 when unauthorized, or 400 when the PUT
   carries no body

 @example
 ```ts
 storeObject({ oid, request, env, l });
 ```
 */
export async function storeObject({
  oid,
  request,
  env,
  l,
}: StoreObjectParams,): Promise<Response> {
  /**
   Logger tagged with this function's name.
   */
  const sl = tagged({
    tag: storeObject.name,
    l,
  },);
  if (!authorized({
    request,
    env,
    l: sl,
  },)) {
    sl.warn(`unauthorized PUT for ${oid}`,);
    return new Response(
      'Unauthorized',
      { status: UNAUTHORIZED, },
    );
  }
  /**
   Upload body; a PUT without one has nothing to store.
   */
  const {body} = request;
  if (body === null) {
    sl.warn(`PUT for ${oid} carried no body`,);
    return new Response(
      'Missing body',
      { status: BAD_REQUEST, },
    );
  }
  await env.BUCKET
    .put(
    oid,
    body,
    );
  sl.info(`stored ${oid}`,);
  return new Response(
    null,
    { status: 200, },
  );
}
