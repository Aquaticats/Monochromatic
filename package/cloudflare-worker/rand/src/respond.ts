/**
 Response construction and the headers every response carries.

 Kept in one module so the cache, CORS and media-type policy cannot drift
 between the success and error paths.

 @module
 */

//region Constants

/**
 Cache policy on every response, including errors.

 A random value that a shared or browser cache may replay is a correctness
 bug, so nothing here is storable.
 */
export const CACHE_CONTROL = 'no-store, no-cache, must-revalidate';

/**
 CORS policy on every response.

 This is a public service with no rate limits, so any origin may read it.
 */
export const ALLOW_ORIGIN = '*';

/**
 Media type for served values and for error bodies.

 Matches the Caddy original, measured as `text/plain; charset=utf-8`.
 */
export const PLAIN_TEXT = 'text/plain; charset=utf-8';

/**
 Methods this Worker serves, advertised in `Allow` and in the preflight.
 */
export const ALLOWED_METHODS = 'GET, HEAD, OPTIONS';

/**
 HTTP status for a served value.
 */
export const OK = 200;

/**
 HTTP status for input the Worker refuses to guess at.
 */
export const BAD_REQUEST = 400;

/**
 HTTP status for a path that matches no route.
 */
export const NOT_FOUND = 404;

/**
 HTTP status for a method this Worker does not serve.
 */
export const METHOD_NOT_ALLOWED = 405;

/**
 HTTP status for an answered preflight, which carries no body.
 */
export const NO_CONTENT = 204;

//endregion Constants

//region Helpers

/**
 Headers every response carries, including errors.

 @returns fresh header set, so callers may add route-specific entries
 */
function sharedHeaders(): Headers {
  /**
   Header set to extend.
   */
  const headers = new Headers();
  headers.set(
    'Cache-Control',
    CACHE_CONTROL,
  );
  headers.set(
    'Access-Control-Allow-Origin',
    ALLOW_ORIGIN,
  );
  return headers;
}

/**
 Build a plain-text response whose length a HEAD request still reports.

 The length is set explicitly for both methods, because the runtime cannot
 derive it from an absent body and the Caddy original reported the real length
 on HEAD, measured as `Content-Length: 36` for `/uuidv4`.

 @param body - plain-text body for a GET, and the length a HEAD reports

 @param omitBody - whether to send headers without the body, set for HEAD

 @param status - HTTP status

 @returns response carrying the shared headers plus the text media type
 */
function plainTextResponse({
  body,
  omitBody,
  status,
}: PlainTextResponseParams,): Response {
  /**
   Encoded body, used for its byte length.
   */
  const bytes = new TextEncoder().encode(body,);
  /**
   Headers for this response.
   */
  const headers = sharedHeaders();
  headers.set(
    'Content-Type',
    PLAIN_TEXT,
  );
  headers.set(
    'Content-Length',
    String(bytes.byteLength,),
  );
  return new Response(
    omitBody ? null : body,
    {
      status,
      headers,
    },
  );
}

/**
 Parameters for {@link plainTextResponse}.
 */
type PlainTextResponseParams = {
  /**
   Plain-text body for a GET, and the length a HEAD reports.
   */
  readonly body: string;
  /**
   Whether to send headers without the body, set for HEAD.
   */
  readonly omitBody: boolean;
  /**
   HTTP status.
   */
  readonly status: number;
};

//endregion Helpers

//region Responses

/**
 Parameters for {@link valueResponse}.
 */
export type ValueResponseParams = {
  /**
   Served value, with no trailing newline, so the length equals the value
   length exactly as the Caddy original did.
   */
  readonly body: string;
  /**
   Whether to send headers without the body, set for HEAD.
   */
  readonly omitBody: boolean;
};

/**
 Build the 200 response for a served value.

 @param body - served value, with no trailing newline

 @param omitBody - whether to send headers without the body, set for HEAD

 @returns 200 response carrying the value

 @example
 ```ts
 valueResponse({ body: 'aZ3Q', omitBody: false });
 ```
 */
export function valueResponse({
  body,
  omitBody,
}: ValueResponseParams,): Response {
  return plainTextResponse({
    body,
    omitBody,
    status: OK,
  },);
}

/**
 Parameters for {@link errorResponse}.
 */
export type ErrorResponseParams = {
  /**
   HTTP status for the refusal.
   */
  readonly status: number;
  /**
   Plain-text explanation naming the offending input and the accepted domain.
   */
  readonly message: string;
  /**
   Whether to send headers without the body, set for HEAD.
   */
  readonly omitBody: boolean;
};

/**
 Build a refusal response whose body explains the refusal.

 Terminated with a newline, unlike a served value, because a human reads it in
 a terminal while a served value is consumed by command substitution.

 @param status - HTTP status for the refusal

 @param message - explanation naming the offending input and the accepted
   domain

 @param omitBody - whether to send headers without the body, set for HEAD

 @returns refusal response carrying the explanation

 @example
 ```ts
 errorResponse({
   status: 400,
   message: 'length must be between 1 and 64, got "65"',
   omitBody: false,
 });
 ```
 */
export function errorResponse({
  status,
  message,
  omitBody,
}: ErrorResponseParams,): Response {
  return plainTextResponse({
    body: `${message}\n`,
    omitBody,
    status,
  },);
}

/**
 Build the preflight response a cross-origin caller needs.

 Answered before method enforcement, because a browser sends OPTIONS to
 discover the allowed methods and a 405 here would make every cross-origin
 read impossible.

 @returns 204 response advertising the allowed methods

 @example
 ```ts
 preflightResponse();
 ```
 */
export function preflightResponse(): Response {
  /**
   Headers for the preflight.
   */
  const headers = sharedHeaders();
  headers.set(
    'Allow',
    ALLOWED_METHODS,
  );
  headers.set(
    'Access-Control-Allow-Methods',
    ALLOWED_METHODS,
  );
  return new Response(
    null,
    {
      status: NO_CONTENT,
      headers,
    },
  );
}

/**
 Parameters for {@link methodNotAllowedResponse}.
 */
export type MethodNotAllowedResponseParams = {
  /**
   Method the caller used.
   */
  readonly method: string;
};

/**
 Build the 405 for a method this Worker does not serve.

 Carries `Allow`, since a random-value endpoint has no write semantics and the
 caller needs to know what it may send instead.

 @param method - method the caller used

 @returns 405 response explaining the refusal and listing allowed methods

 @example
 ```ts
 methodNotAllowedResponse({ method: 'POST', });
 ```
 */
export function methodNotAllowedResponse({ method, }: MethodNotAllowedResponseParams,): Response {
  /**
   Explanation naming the refused method.
   */
  const message = `${method} is not served; allowed methods are ${ALLOWED_METHODS}\n`;
  /**
   Encoded body, used for its byte length.
   */
  const bytes = new TextEncoder().encode(message,);
  /**
   Headers for this response.
   */
  const headers = sharedHeaders();
  headers.set(
    'Allow',
    ALLOWED_METHODS,
  );
  headers.set(
    'Content-Type',
    PLAIN_TEXT,
  );
  headers.set(
    'Content-Length',
    String(bytes.byteLength,),
  );
  return new Response(
    message,
    {
      status: METHOD_NOT_ALLOWED,
      headers,
    },
  );
}

//endregion Responses
