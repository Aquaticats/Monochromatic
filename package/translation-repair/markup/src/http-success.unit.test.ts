/**
 Tests for the shared success-status reading both clients ask before they
 treat a body as an answer.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { isSuccessStatus, } from '../dist/final/node/index.mjs';

await describe({
  name: isSuccessStatus.name,
  children: [
    it({
      name: 'ACCEPTS the success family at both its boundaries',
      fn: async () => {
        expect(isSuccessStatus({ status: 200, },),).toBe(true,);
        expect(isSuccessStatus({ status: 204, },),).toBe(true,);
        expect(isSuccessStatus({ status: 299, },),).toBe(true,);
      },
    },),

    it({
      name: 'REFUSES a redirect, which carries no completion',
      fn: async () => {
        // Admitting one would send an HTML or empty body into a parser that
        // reports it as a provider contract violation rather than as the
        // misrouted request it is.
        expect(isSuccessStatus({ status: 300, },),).toBe(false,);
        expect(isSuccessStatus({ status: 302, },),).toBe(false,);
      },
    },),

    it({
      name: 'REFUSES the statuses this pipeline actually meets',
      fn: async () => {
        expect(isSuccessStatus({ status: 199, },),).toBe(false,);
        expect(isSuccessStatus({ status: 400, },),).toBe(false,);
        expect(isSuccessStatus({ status: 401, },),).toBe(false,);
        expect(isSuccessStatus({ status: 429, },),).toBe(false,);
        expect(isSuccessStatus({ status: 502, },),).toBe(false,);
      },
    },),
  ],
},);
