/**
 Git LFS batch API: resolve requested objects into download or upload actions.

 Protocol reference: <https://github.com/git-lfs/git-lfs/blob/main/docs/api/batch.md>.

 @module
 */

import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';
import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import { authorized, } from './authorize.ts';
import { lfsJson, } from './lfs-json.ts';
import { isOid, } from './oid.ts';
import type { WorkerEnv, } from './store.ts';

/**
 HTTP status for a syntactically invalid batch request.
 */
const BAD_REQUEST = 400;

/**
 HTTP status for an upload attempted without the write token.
 */
const UNAUTHORIZED = 401;

/**
 Per-object error code for an object absent from the store.
 */
const NOT_FOUND = 404;

/**
 Batch operations the protocol defines.
 */
export type BatchOperation = 'download' | 'upload';

/**
 One object named in a batch request.
 */
export type BatchObject = {
  /**
   sha256 object id.
   */
  readonly oid: string;
  /**
   Byte length the client believes the object has.
   */
  readonly size: number;
};

/**
 Validated batch request body.
 */
export type BatchRequest = {
  /**
   Whether the client wants to download or upload the listed objects.
   */
  readonly operation: BatchOperation;
  /**
   Objects to resolve.
   */
  readonly objects: readonly BatchObject[];
};

/**
 Thrown when a batch request body fails validation; the message is returned
 to the client with status 400.
 */
export class BatchRequestError extends Error {
  /**
   @param message - reason the body was rejected

   @param options - optional cause, kept for the log line
   */
  constructor(
    message: string,
    options?: ErrorOptions,
  ) {
    super(
      message,
      options,
    );
    this.name = 'BatchRequestError';
  }
}

/**
 Whether `value` is a plain object rather than `null`, an array, or a primitive.

 @param value - parsed JSON value

 @returns `true` for object literals
 */
function isPlainObject(value: unknown,): value is Record<string, unknown> {
  return ((typeof value) === 'object') && (value !== null)
    && (!Array.isArray(value,));
}

/**
 Validate one entry of the `objects` array.

 @param value - array entry

 @returns the entry as a {@link BatchObject}

 @throws {@link BatchRequestError} when the entry lacks a well-formed oid or a non-negative integer size
 */
function parseBatchObject(value: unknown,): BatchObject {
  if (!isPlainObject(value,)) {
    throw new BatchRequestError('each object must be an object',);
  }
  /**
   Fields of the entry, still unvalidated.
   */
  const {
    oid,
    size,
  } = value;
  if (((typeof oid) !== 'string') || (!isOid(oid,))) {
    throw new BatchRequestError('each object needs a 64-character lowercase hex oid',);
  }
  if (((typeof size) !== 'number') || (!Number.isInteger(size,))
    || (size < 0)) {
    throw new BatchRequestError(`object ${oid} needs a non-negative integer size`,);
  }
  return {
    oid,
    size,
  };
}

/**
 Validate a parsed batch request body.

 @param payload - parsed JSON body

 @returns validated operation and objects

 @throws {@link BatchRequestError} when the body is not a batch request

 @example
 ```ts
 parseBatchRequest({ operation: 'download', objects: [{ oid: 'a'.repeat(64), size: 1 }] });
 ```
 */
export function parseBatchRequest(payload: unknown,): BatchRequest {
  if (!isPlainObject(payload,)) {
    throw new BatchRequestError('batch request body must be a JSON object',);
  }
  /**
   Fields of the body, still unvalidated.
   */
  const {
    operation,
    objects,
  } = payload;
  if ((operation !== 'download') && (operation !== 'upload')) {
    throw new BatchRequestError('operation must be "download" or "upload"',);
  }
  if (!Array.isArray(objects,)) {
    throw new BatchRequestError('objects must be an array',);
  }
  return {
    operation,
    objects: objects.map(parseBatchObject,),
  };
}

/**
 Parameters for {@link readBatchRequest}.
 */
type ReadBatchRequestParams = {
  /**
   Batch POST whose JSON body lists the requested objects.
   */
  readonly request: Request;
  /**
   Logger for the parse outcome.
   */
  readonly l: Logger;
};

/**
 Read and validate the batch request body.

 @param request - batch POST whose JSON body lists the requested objects

 @param l - logger for the parse outcome

 @returns validated batch request

 @throws {@link BatchRequestError} when the body is not valid JSON or not a batch request
 */
async function readBatchRequest({
  request,
  l,
}: ReadBatchRequestParams,): Promise<BatchRequest> {
  /**
   Logger tagged with this function's name.
   */
  const rl = tagged({
    tag: readBatchRequest.name,
    l,
  },);
  try {
    return parseBatchRequest(await request.json(),);
  }
  catch (error) {
    if (error instanceof BatchRequestError) {
      rl.warn(`rejecting batch request: ${error.message}`,);
      throw error;
    }
    rl.warn(`rejecting batch request with unparseable JSON: ${String(error,)}`,);
    throw new BatchRequestError(
      'batch request body must be valid JSON',
      { cause: error, },
    );
  }
}

/**
 Parameters for {@link resolveObject}.
 */
type ResolveObjectParams = {
  /**
   Object to resolve.
   */
  readonly object: BatchObject;
  /**
   Operation the client asked for.
   */
  readonly operation: BatchOperation;
  /**
   Worker env exposing the store.
   */
  readonly env: WorkerEnv;
  /**
   Origin of the request, used to build absolute action hrefs.
   */
  readonly origin: string;
  /**
   `Authorization` header to echo into upload actions so git-lfs repeats it
   on the PUT; present only for authorized uploads.
   */
  readonly authorization?: string;
  /**
   Logger for the per-object decision.
   */
  readonly l: Logger;
};

/**
 Resolve one object into its batch response entry.

 Downloads of absent objects return a per-object 404 error; uploads of present
 objects return no action so git-lfs skips re-uploading them.

 @param object - object to resolve

 @param operation - operation the client asked for

 @param env - Worker env exposing the store

 @param origin - origin of the request, used to build absolute action hrefs

 @param authorization - `Authorization` header echoed into upload actions

 @param l - logger for the per-object decision

 @returns batch response entry for the object
 */
async function resolveObject({
  object,
  operation,
  env,
  origin,
  authorization,
  l,
}: ResolveObjectParams,): Promise<Record<string, unknown>> {
  /**
   Logger tagged with this function's name.
   */
  const rl = tagged({
    tag: resolveObject.name,
    l,
  },);
  /**
   Object id and size echoed into the response entry.
   */
  const {
    oid,
    size,
  } = object;
  /**
   Absolute URL git-lfs will GET or PUT for this object.
   */
  const href = `${origin}/${oid}`;
  /**
   Store metadata; `null` when the object is absent.
   */
  const head = await env.BUCKET
    .head(oid,);
  if (operation === 'upload') {
    if (head !== null) {
      rl.debug(`${oid} already stored; no upload action`,);
      return {
        oid,
        size,
      };
    }
    rl.debug(`${oid} absent; issuing upload action`,);
    return {
      oid,
      size,
      actions: {
        upload: {
          href,
          header: { Authorization: authorization, },
        },
      },
    };
  }
  if (head === null) {
    rl.debug(`${oid} absent; reporting per-object 404`,);
    return {
      oid,
      size,
      error: {
        code: NOT_FOUND,
        message: 'Object not found',
      },
    };
  }
  rl.debug(`${oid} present; issuing download action`,);
  return {
    oid,
    size,
    actions: { download: { href, }, },
  };
}

/**
 Parameters for {@link handleBatch}.
 */
export type HandleBatchParams = {
  /**
   Batch POST whose JSON body lists the requested objects.
   */
  readonly request: Request;
  /**
   Worker env exposing the store and upload secret.
   */
  readonly env: WorkerEnv;
  /**
   Parsed request URL, used to derive absolute object hrefs.
   */
  readonly url: URL;
  /**
   Logger for the batch decision.
   */
  readonly l: Logger;
};

/**
 Resolve a git-lfs batch request into per-object download or upload actions.

 @param request - batch POST whose JSON body lists the requested objects

 @param env - Worker env exposing the store and upload secret

 @param url - parsed request URL, used to derive absolute object hrefs

 @param l - logger for the batch decision

 @returns git-lfs batch response, or a 400 for an invalid body, or a 401 for an
   unauthorized upload

 @example
 ```ts
 handleBatch({ request, env, url: new URL(request.url), l });
 ```
 */
export async function handleBatch({
  request,
  env,
  url,
  l,
}: HandleBatchParams,): Promise<Response> {
  /**
   Logger tagged with this function's name.
   */
  const hl = tagged({
    tag: handleBatch.name,
    l,
  },);
  try {
    /**
     Validated batch request.
     */
    const batch = await readBatchRequest({
      request,
      l: hl,
    },);
    hl.info(`${batch.operation} batch for ${batch.objects
      .length} object(s)`,);
    if ((batch.operation === 'upload') && (!authorized({
      request,
      env,
      l: hl,
    },))) {
      hl.warn('unauthorized upload batch',);
      return new Response(
        'Unauthorized',
        {
          status: UNAUTHORIZED,
          headers: { 'LFS-Authenticate': 'Basic realm="monochromatic-lfs"', },
        },
      );
    }
    /**
     Per-object entries resolved concurrently against the store.
     */
    const results = await Promise.all(
      batch.objects
        .map(function resolve(object: BatchObject,): Promise<Record<string, unknown>> {
          return resolveObject({
            object,
            operation: batch.operation,
            env,
            origin: url.origin,
            ...batch.operation === 'upload'
              ? { authorization: nonNullishOrThrow(request.headers
                .get('Authorization',),), }
              : {},
            l: hl,
          },);
        },),
    );
    return lfsJson({
      body: {
        transfer: 'basic',
        objects: results,
      },
      status: 200,
    },);
  }
  catch (error) {
    if (error instanceof BatchRequestError) {
      return lfsJson({
        body: { message: error.message, },
        status: BAD_REQUEST,
      },);
    }
    hl.error(`batch request failed unexpectedly: ${String(error,)}`,);
    throw error;
  }
}
