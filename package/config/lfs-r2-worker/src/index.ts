/**
 Git LFS server for this repository, backed by Cloudflare R2.

 Routes:
 - `POST /objects/batch`: the git-lfs batch API.
 - `GET` and `HEAD` on `/<oid>` and `/<oid>/<path>`: stream an object
   anonymously; a path suffix selects the image media type by extension so
   GitHub's README image proxy accepts the response.
 - `PUT /<oid>`: store an object; requires the `LFS_WRITE_TOKEN` secret.

 @module
 */

import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';

import { handleBatch, } from './batch.ts';
import { mediaTypeForPath, } from './media-type.ts';
import { isOid, } from './oid.ts';
import {
  serveObject,
  storeObject,
} from './object-route.ts';
import type { WorkerEnv, } from './store.ts';

export {
  authorized,
  type AuthorizedParams,
} from './authorize.ts';
export {
  type BatchObject,
  type BatchOperation,
  type BatchRequest,
  BatchRequestError,
  handleBatch,
  type HandleBatchParams,
  parseBatchRequest,
} from './batch.ts';
export {
  LFS_JSON,
  lfsJson,
  type LfsJsonParams,
} from './lfs-json.ts';
// In-memory store for unit tests, exported so tests import it through the
// built artifact instead of package source.
/**
 @internal
 */
export {
  createMemoryObjectStore,
  type MemoryObjectStore,
} from './memory-object-store.ts';
export {
  IMAGE_MEDIA_TYPES,
  mediaTypeForPath,
  OCTET_STREAM,
} from './media-type.ts';
export {
  IMMUTABLE_CACHE_CONTROL,
  objectHeaders,
  type ObjectHeadersParams,
  serveObject,
  type ServeObjectParams,
  storeObject,
  type StoreObjectParams,
} from './object-route.ts';
export {
  isOid,
  OID_LENGTH,
} from './oid.ts';
export type {
  ExecutionContextLike,
  ObjectStore,
  StoredObject,
  StoredObjectHead,
  WorkerEnv,
} from './store.ts';

/**
 HTTP status for a method the matched route does not support.
 */
const METHOD_NOT_ALLOWED = 405;

/**
 HTTP status for an unmatched path.
 */
const NOT_FOUND = 404;

/**
 A request path split into its first segment and the remainder.
 */
export type SplitPath = {
  /**
   First path segment, without the leading slash.
   */
  readonly first: string;
  /**
   Remaining path after the first segment, without its leading slash; empty
   when the path had one segment.
   */
  readonly rest: string;
};

/**
 Split a request path into its first segment and the remainder.

 @param pathname - request path including the leading slash

 @returns first segment and remainder

 @example
 ```ts
 splitPath(`/${'a'.repeat(64)}/asset/readme/desktop.png`);
 // { first: 'aaaa…', rest: 'asset/readme/desktop.png' }
 splitPath('/objects/batch'); // { first: 'objects', rest: 'batch' }
 splitPath('/'); // { first: '', rest: '' }
 ```
 */
export function splitPath(pathname: string,): SplitPath {
  /**
   Path without its leading slash.
   */
  const withoutLeadingSlash = pathname.startsWith('/',) ? pathname.slice(1,) : pathname;
  /**
   Boundary between the first segment and the remainder.
   */
  const separator = withoutLeadingSlash.indexOf('/',);
  if (separator === (-1)) {
    return {
      first: withoutLeadingSlash,
      rest: '',
    };
  }
  return {
    first: withoutLeadingSlash.slice(
      0,
      separator,
    ),
    rest: withoutLeadingSlash.slice(separator + 1,),
  };
}

/**
 Parameters for {@link handleRequest}.
 */
export type HandleRequestParams = {
  /**
   Inbound HTTP request from git-lfs, a browser, or GitHub's image proxy.
   */
  readonly request: Request;
  /**
   Worker env exposing the store and upload secret.
   */
  readonly env: WorkerEnv;
  /**
   Logger for the routing decision.
   */
  readonly l: Logger;
};

/**
 Route one request to the batch, serve, or store handler.

 @param request - inbound HTTP request from git-lfs, a browser, or GitHub's image proxy

 @param env - Worker env exposing the store and upload secret

 @param l - logger for the routing decision

 @returns response for the matched route; 404 for an unmatched path; 405 for a
   method the matched object route does not support

 @example
 ```ts
 handleRequest({ request, env, l });
 ```
 */
export async function handleRequest({
  request,
  env,
  l: parentLogger,
}: HandleRequestParams,): Promise<Response> {
  /**
   Logger tagged with this function's name.
   */
  const hl = tagged({
    tag: handleRequest.name,
    l: parentLogger,
  },);
  /**
   Parsed request URL.
   */
  const url = new URL(request.url,);
  hl.debug(`${request.method} ${url.pathname}`,);
  if ((request.method === 'POST')
    && url.pathname
      .endsWith('/objects/batch',)) {
    return await handleBatch({
      request,
      env,
      url,
      l: hl,
    },);
  }
  /**
   Candidate oid and any path suffix after it.
   */
  const {
    first: oid,
    rest: suffix,
  } = splitPath(url.pathname,);
  if (!isOid(oid,)) {
    hl.info(`no route for ${request.method} ${url.pathname}`,);
    return new Response(
      'Not found',
      { status: NOT_FOUND, },
    );
  }
  /**
   Media type from the suffix; bare oids serve as octet streams for git-lfs.
   */
  const mediaType = mediaTypeForPath(suffix,);
  if (request.method === 'GET') {
    return await serveObject({
      oid,
      mediaType,
      request,
      env,
      includeBody: true,
      l: hl,
    },);
  }
  if (request.method === 'HEAD') {
    return await serveObject({
      oid,
      mediaType,
      request,
      env,
      includeBody: false,
      l: hl,
    },);
  }
  if ((request.method === 'PUT') && (suffix === '')) {
    return await storeObject({
      oid,
      request,
      env,
      l: hl,
    },);
  }
  hl.info(`${request.method} not allowed on ${url.pathname}`,);
  return new Response(
    'Method not allowed',
    {
      status: METHOD_NOT_ALLOWED,
      headers: { Allow: suffix === '' ? 'GET, HEAD, PUT' : 'GET, HEAD', },
    },
  );
}
