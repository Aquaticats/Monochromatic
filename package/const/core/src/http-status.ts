/**
 HTTP response status codes from RFC 9110.
 
 Limited to codes that already appear duplicated across the workspace.
 Add additional spec-defined codes only when a second use site materialises;
 speculative additions belong in the consuming package until then.
 
 Reference: RFC 9110, Section 15.
 
 @example
 ```ts
 import {
   HTTP_NOT_FOUND,
   HTTP_OK,
 } from '@monochromatic-dev/const-core';
 ```
 
 @module
 */

//region 2xx Success

/**
 200 OK. The request succeeded; semantics depend on the HTTP method.
 
 @example
 ```ts
 return new Response(body, { status: HTTP_OK, },);
 ```
 */
export const HTTP_OK = 200;

/**
 201 Created. The request succeeded and a new resource was created.
 
 @example
 ```ts
 return new Response(JSON.stringify(newResource,), { status: HTTP_CREATED, },);
 ```
 */
export const HTTP_CREATED = 201;

/**
 204 No Content. The request succeeded; the response intentionally has no body.
 
 @example
 ```ts
 return new Response(null, { status: HTTP_NO_CONTENT, },);
 ```
 */
export const HTTP_NO_CONTENT = 204;

//endregion 2xx Success

//region 3xx Redirection

/**
 304 Not Modified. The cached representation is still valid; the client
 should reuse it. Sent in response to a conditional GET.
 
 @example
 ```ts
 if (matches(req, etag,)) return new Response(null, { status: HTTP_NOT_MODIFIED, },);
 ```
 */
export const HTTP_NOT_MODIFIED = 304;

//endregion 3xx Redirection

//region 4xx Client errors

/**
 400 Bad Request. The server cannot process the request because of a
 client-side error such as malformed syntax.
 
 @example
 ```ts
 return new Response('invalid input', { status: HTTP_BAD_REQUEST, },);
 ```
 */
export const HTTP_BAD_REQUEST = 400;

/**
 401 Unauthorized. The request lacks valid authentication credentials.
 
 @example
 ```ts
 return new Response('login required', { status: HTTP_UNAUTHORIZED, },);
 ```
 */
export const HTTP_UNAUTHORIZED = 401;

/**
 403 Forbidden. The server understood the request but refuses to authorise
 it. Distinct from 401: re-authenticating will not help.
 
 @example
 ```ts
 return new Response('not yours', { status: HTTP_FORBIDDEN, },);
 ```
 */
export const HTTP_FORBIDDEN = 403;

/**
 404 Not Found. The server cannot find the requested resource.
 
 @example
 ```ts
 return new Response('missing', { status: HTTP_NOT_FOUND, },);
 ```
 */
export const HTTP_NOT_FOUND = 404;

/**
 409 Conflict. The request conflicts with the current state of the resource,
 commonly used for unique-constraint violations on creation.
 
 @example
 ```ts
 return new Response('already exists', { status: HTTP_CONFLICT, },);
 ```
 */
export const HTTP_CONFLICT = 409;

//endregion 4xx Client errors

//region 5xx Server errors

/**
 500 Internal Server Error. The server encountered an unexpected condition
 that prevented fulfilling the request.
 
 @example
 ```ts
 return new Response('oops', { status: HTTP_INTERNAL_SERVER_ERROR, },);
 ```
 */
export const HTTP_INTERNAL_SERVER_ERROR = 500;

//endregion 5xx Server errors
