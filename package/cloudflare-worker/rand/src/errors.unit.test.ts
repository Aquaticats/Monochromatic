/**
 Tests for the two error classes the router maps onto HTTP statuses.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  BadRequestError,
  NotFoundError,
} from '@monochromatic-dev/cloudflare-worker-rand';

await describe({
  name: 'errors',
  children: [
    it({
      name: 'carries its message and names itself, so logs and stack traces distinguish the two refusals',
      fn: async () => {
        /**
         Refusal for input the Worker will not guess at.
         */
        const bad = new BadRequestError('length must be between 1 and 64',);
        expect(bad.message,).toBe('length must be between 1 and 64',);
        expect(bad.name,).toBe('BadRequestError',);
        expect(bad,).toBeInstanceOf(Error,);
        /**
         Refusal for a path that matches no route.
         */
        const missing = new NotFoundError('no route matches "/abc"',);
        expect(missing.message,).toBe('no route matches "/abc"',);
        expect(missing.name,).toBe('NotFoundError',);
        expect(missing,).toBeInstanceOf(Error,);
      },
    },),
    it({
      name: 'keeps the two classes distinct, so the router cannot map one onto the other status',
      fn: async () => {
        expect(new BadRequestError('x',),).not.toBeInstanceOf(NotFoundError,);
        expect(new NotFoundError('x',),).not.toBeInstanceOf(BadRequestError,);
      },
    },),
  ],
},);
