/**
 Shared HTTP response utilities for API handlers.
 */

import { HTTP_OK, } from '@monochromatic-dev/const-core/ts';

export {
  HTTP_BAD_REQUEST,
  HTTP_CONFLICT,
  HTTP_CREATED,
  HTTP_INTERNAL_SERVER_ERROR,
  HTTP_NOT_FOUND,
  HTTP_OK,
} from '@monochromatic-dev/const-core/ts';

/**
 Wraps a payload in a JSON `Response` with the correct content type.
 
 @param options - Serializable payload and HTTP status.
 
 @returns JSON response with content-type header
 
 @mutates options - `Fetch commit 586cd2a4 Response.json serializes data and reads response initialization`
 may invoke serialization hooks reachable from `options.payload`.
 
 @example
 ```ts
 return jsonResponse({ payload: { ok: true }, status: 200 });
 ```
 */
export function jsonResponse(
  options: {
    readonly payload: unknown;
    readonly status?: number;
  },
): Response {
  /**
   Response fields separated after boundary contract attaches to their containing input.
   */
  const {
    payload,
    status = HTTP_OK,
  } = options;
  return Response.json(
    payload,
    { status, },
  );
}
