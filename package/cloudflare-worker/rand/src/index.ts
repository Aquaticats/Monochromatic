/**
 Random-value HTTP service, served as a Cloudflare Worker.

 Routes, all matched case-insensitively on the path:
 - `GET` and `HEAD` on `/uuidv4`: a version 4 UUID.
 - `GET` and `HEAD` on `/int?min=&max=`: an integer drawn uniformly from the
   closed interval, both bounds included.
 - `GET` and `HEAD` on `/1` to `/64`: that many characters from `0-9A-Za-z`.
 - `GET` and `HEAD` on `/`: 64 characters from `0-9A-Za-z`.
 - `OPTIONS` on any path: a CORS preflight answer.

 Every other method answers 405, every unmatched path answers 404, and every
 malformed input answers 400 naming the offending value.

 Replaces the Caddy site block at `rand.c.aquati.cat`, committed as
 `Caddyfile.origin` in this package. That origin is soft-deprecated: the owner
 keeps it running and will neither delete nor change it, even if it breaks.

 @module
 */

import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';

import {
  BadRequestError,
  NotFoundError,
} from './errors.ts';
import {
  MAX_RANDOM_LENGTH,
  randomAlphanumeric,
  randomIntInclusive,
  randomUuidV4,
} from './random.ts';
import {
  BAD_REQUEST,
  errorResponse,
  methodNotAllowedResponse,
  NOT_FOUND,
  preflightResponse,
  valueResponse,
} from './respond.ts';
import {
  parseIntParams,
  parseLengthRoute,
} from './validate.ts';

export {
  BadRequestError,
  NotFoundError,
} from './errors.ts';
export type {
  ExecutionContextLike,
  WorkerEnv,
} from './env.ts';
export {
  ALPHANUMERIC_CHARSET,
  bitLength,
  drawBits,
  MAX_RANDOM_LENGTH,
  randomAlphanumeric,
  type RandomAlphanumericParams,
  randomIntInclusive,
  type RandomIntInclusiveParams,
  randomUuidV4,
  UNBIASED_BYTE_LIMIT,
} from './random.ts';
export {
  ALLOWED_METHODS,
  ALLOW_ORIGIN,
  BAD_REQUEST,
  CACHE_CONTROL,
  errorResponse,
  type ErrorResponseParams,
  methodNotAllowedResponse,
  type MethodNotAllowedResponseParams,
  METHOD_NOT_ALLOWED,
  NO_CONTENT,
  NOT_FOUND,
  OK,
  PLAIN_TEXT,
  preflightResponse,
  valueResponse,
  type ValueResponseParams,
} from './respond.ts';
export {
  type IntBounds,
  looksNumeric,
  parseCanonicalInteger,
  type ParseCanonicalIntegerParams,
  parseIntParams,
  type ParseIntParamsParams,
  parseLengthRoute,
  type ParseLengthRouteParams,
} from './validate.ts';

//region Constants

/**
 Path serving a version 4 UUID.
 */
const UUID_ROUTE = '/uuidv4';

/**
 Path serving an inclusive random integer.
 */
const INT_ROUTE = '/int';

/**
 Path serving the default-length random string.
 */
const ROOT_ROUTE = '/';

//endregion Constants

//region Request handling

/**
 Parameters for {@link handleRequest}.
 */
export type HandleRequestParams = {
  /**
   Inbound HTTP request from a browser, a shell one-liner, or a script.
   */
  readonly request: Request;
  /**
   Logger for the routing decision.
   */
  readonly l: Logger;
};

/**
 Route one request to the value it asks for, or to the refusal it deserves.

 @param request - inbound HTTP request from a browser, a shell one-liner, or a
   script

 @param l - logger for the routing decision

 @returns 200 carrying the value, 204 for a preflight, or the 400, 404 or 405
   that explains the refusal

 @throws Error only for a failure that is not the caller's fault, which the
   platform then reports as a 500

 @example
 ```ts
 handleRequest({ request: new Request('https://rand.test/8'), l });
 ```
 */
export function handleRequest({
  request,
  l: parentLogger,
}: HandleRequestParams,): Response {
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
  /**
   Method the caller used.
   */
  const {method} = request;
  /**
   Whether the caller asked for headers without a body.
   */
  const omitBody = method === 'HEAD';
  hl.debug(`${method} ${url.pathname}`,);
  // The preflight precedes method enforcement: a browser sends OPTIONS to
  // discover the allowed methods, so refusing it here would make every
  // cross-origin read impossible.
  if (method === 'OPTIONS') {
    return preflightResponse();
  }
  if ((method !== 'GET') && (method !== 'HEAD')) {
    hl.debug(`refusing method ${method}`,);
    return methodNotAllowedResponse({ method, },);
  }
  // Lowercased for parity with the Caddy original, measured to serve /UUIDV4
  // and /INT. Otherwise the path is used exactly as URL normalized it, so
  // percent-encoded and doubled-slash forms fall through to 404 instead of
  // being decoded or collapsed the way the origin did.
  /**
   Path to match, case-folded.
   */
  const pathname = url.pathname
    .toLowerCase();
  try {
    if (pathname === UUID_ROUTE) {
      return valueResponse({
        body: randomUuidV4(),
        omitBody,
      },);
    }
    if (pathname === INT_ROUTE) {
      /**
       Validated inclusive bounds.
       */
      const bounds = parseIntParams({ searchParams: url.searchParams, },);
      return valueResponse({
        body: String(randomIntInclusive(bounds,),),
        omitBody,
      },);
    }
    if (pathname === ROOT_ROUTE) {
      return valueResponse({
        body: randomAlphanumeric({ length: MAX_RANDOM_LENGTH, },),
        omitBody,
      },);
    }
    return valueResponse({
      body: randomAlphanumeric({
        length: parseLengthRoute({ pathname, },),
      },),
      omitBody,
    },);
  }
  catch (error) {
    if (error instanceof BadRequestError) {
      hl.debug(`refusing input: ${error.message}`,);
      return errorResponse({
        status: BAD_REQUEST,
        message: error.message,
        omitBody,
      },);
    }
    if (error instanceof NotFoundError) {
      hl.debug(`no route: ${error.message}`,);
      return errorResponse({
        status: NOT_FOUND,
        message: error.message,
        omitBody,
      },);
    }
    throw error;
  }
}

//endregion Request handling
