/**
 Tests for response construction: the headers every response carries, the
 length a HEAD request still reports, and the preflight and 405 shapes.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  ALLOWED_METHODS,
  ALLOW_ORIGIN,
  BAD_REQUEST,
  CACHE_CONTROL,
  errorResponse,
  methodNotAllowedResponse,
  METHOD_NOT_ALLOWED,
  NO_CONTENT,
  NOT_FOUND,
  OK,
  PLAIN_TEXT,
  preflightResponse,
  valueResponse,
} from '@monochromatic-dev/cloudflare-worker-rand';

/**
 Assert the headers every response must carry, so the cache and CORS policy
 cannot drift between the success and error paths.

 @param response - response to inspect
 */
async function expectSharedHeaders(response: Response,): Promise<void> {
  expect(response.headers.get('Cache-Control',),).toBe(CACHE_CONTROL,);
  expect(CACHE_CONTROL,).toBe('no-store, no-cache, must-revalidate',);
  expect(response.headers.get('Access-Control-Allow-Origin',),).toBe(ALLOW_ORIGIN,);
}

await describe({
  name: 'respond',
  children: [
    describe({
      name: valueResponse.name,
      children: [
        it({
          name: 'serves the value verbatim with no trailing newline and its exact length',
          fn: async () => {
            /**
             Response for a four-character value.
             */
            const response = valueResponse({
              body: 'aZ3Q',
              omitBody: false,
            },);
            expect(response.status,).toBe(OK,);
            expect(response.headers.get('Content-Type',),).toBe(PLAIN_TEXT,);
            expect(response.headers.get('Content-Length',),).toBe('4',);
            expect(await response.text(),).toBe('aZ3Q',);
            await expectSharedHeaders(response,);
          },
        },),
        it({
          name: 'reports the real length on a HEAD with no body, matching the origin',
          fn: async () => {
            /**
             HEAD response for the same value.
             */
            const response = valueResponse({
              body: 'aZ3Q',
              omitBody: true,
            },);
            expect(response.status,).toBe(OK,);
            expect(response.headers.get('Content-Length',),).toBe('4',);
            expect(response.headers.get('Content-Type',),).toBe(PLAIN_TEXT,);
            expect(await response.text(),).toBe('',);
            await expectSharedHeaders(response,);
          },
        },),
        it({
          name: 'counts bytes rather than characters for the reported length',
          fn: async () => {
            /**
             Response for a body whose byte length exceeds its character count.
             */
            const response = valueResponse({
              body: 'é',
              omitBody: false,
            },);
            expect(response.headers.get('Content-Length',),).toBe('2',);
          },
        },),
      ],
    },),
    describe({
      name: errorResponse.name,
      children: [
        it({
          name: 'carries the status, explains the refusal and terminates the body',
          fn: async () => {
            /**
             Refusal for a bad length.
             */
            const response = errorResponse({
              status: BAD_REQUEST,
              message: 'length must be between 1 and 64, got "65"',
              omitBody: false,
            },);
            expect(response.status,).toBe(BAD_REQUEST,);
            expect(response.headers.get('Content-Type',),).toBe(PLAIN_TEXT,);
            /**
             Body text.
             */
            const body = await response.text();
            expect(body.endsWith('\n',),).toBe(true,);
            expect(body,).toMatch('length must be between 1 and 64',);
            await expectSharedHeaders(response,);
          },
        },),
        it({
          name: 'passes the 404 status through and omits the body on HEAD',
          fn: async () => {
            /**
             Refusal text, held so the expected length is derived rather than
             counted by hand.
             */
            const message = 'no route matches "/abc"';
            /**
             HEAD refusal for an unknown route.
             */
            const response = errorResponse({
              status: NOT_FOUND,
              message,
              omitBody: true,
            },);
            expect(response.status,).toBe(NOT_FOUND,);
            expect(response.headers.get('Content-Length',),).toBe(String(new TextEncoder().encode(`${message}\n`,).byteLength,),);
            expect(await response.text(),).toBe('',);
          },
        },),
      ],
    },),
    describe({
      name: preflightResponse.name,
      children: [
        it({
          name: 'answers 204 with the allowed methods and no body',
          fn: async () => {
            /**
             Preflight answer.
             */
            const response = preflightResponse();
            expect(response.status,).toBe(NO_CONTENT,);
            expect(response.headers.get('Allow',),).toBe(ALLOWED_METHODS,);
            expect(response.headers.get('Access-Control-Allow-Methods',),).toBe(ALLOWED_METHODS,);
            expect(await response.text(),).toBe('',);
            await expectSharedHeaders(response,);
          },
        },),
      ],
    },),
    describe({
      name: methodNotAllowedResponse.name,
      children: [
        it({
          name: 'answers 405 naming the refused method and advertising Allow',
          fn: async () => {
            /**
             Refusal for a write method this endpoint has no semantics for.
             */
            const response = methodNotAllowedResponse({ method: 'POST', },);
            expect(response.status,).toBe(METHOD_NOT_ALLOWED,);
            expect(response.headers.get('Allow',),).toBe(ALLOWED_METHODS,);
            expect(response.headers.get('Content-Type',),).toBe(PLAIN_TEXT,);
            /**
             Body text.
             */
            const body = await response.text();
            expect(body,).toMatch('POST is not served',);
            expect(response.headers.get('Content-Length',),).toBe(String(new TextEncoder().encode(body,).byteLength,),);
            await expectSharedHeaders(response,);
          },
        },),
      ],
    },),
  ],
},);
