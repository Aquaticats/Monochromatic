/**
 Tests for HTTP status code constants.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  HTTP_BAD_REQUEST,
  HTTP_CONFLICT,
  HTTP_CREATED,
  HTTP_FORBIDDEN,
  HTTP_INTERNAL_SERVER_ERROR,
  HTTP_NO_CONTENT,
  HTTP_NOT_FOUND,
  HTTP_NOT_MODIFIED,
  HTTP_OK,
  HTTP_UNAUTHORIZED,
} from '../dist/final/neutral/index.mjs';

await describe({
  name: 'http-status',
  children: [
    it({
      name: '2xx success codes match RFC 9110',
      fn: async () => {
        expect(HTTP_OK,).toBe(200,);
        expect(HTTP_CREATED,).toBe(201,);
        expect(HTTP_NO_CONTENT,).toBe(204,);
      },
    },),
    it({
      name: '3xx redirection codes match RFC 9110',
      fn: async () => {
        expect(HTTP_NOT_MODIFIED,).toBe(304,);
      },
    },),
    it({
      name: '4xx client error codes match RFC 9110',
      fn: async () => {
        expect(HTTP_BAD_REQUEST,).toBe(400,);
        expect(HTTP_UNAUTHORIZED,).toBe(401,);
        expect(HTTP_FORBIDDEN,).toBe(403,);
        expect(HTTP_NOT_FOUND,).toBe(404,);
        expect(HTTP_CONFLICT,).toBe(409,);
      },
    },),
    it({
      name: '5xx server error codes match RFC 9110',
      fn: async () => {
        expect(HTTP_INTERNAL_SERVER_ERROR,).toBe(500,);
      },
    },),
  ],
},);
