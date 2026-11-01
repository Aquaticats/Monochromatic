/**
 Tests for upload authorization.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  authorized,
  createMemoryObjectStore,
  type WorkerEnv,
} from '@monochromatic-dev/config-lfs-r2-worker';

/**
 Logger shared by every case.
 */
const l = tagged({ tag: 'authorize.unit.test', },);

/**
 Upload secret configured on the env under test.
 */
const TOKEN = 'se:cret';

/**
 Env with the secret set.
 */
const env: WorkerEnv = {
  BUCKET: createMemoryObjectStore(),
  LFS_WRITE_TOKEN: TOKEN,
};

/**
 Build a PUT request carrying the given `Authorization` header, or none.

 @param authorization - header value; omit for no header

 @returns request against a fixed URL
 */
function requestWith(authorization?: string,): Request {
  return new Request('https://lfs.test/put', {
    method: 'PUT',
    ...authorization === undefined ? {} : { headers: { Authorization: authorization, }, },
  },);
}

await describe({
  name: authorized.name,
  children: [
    it({
      name: 'refuses every write while the secret is unset',
      fn: async () => {
        /**
         Env without a token, as on a fresh deploy.
         */
        const bare: WorkerEnv = { BUCKET: createMemoryObjectStore(), };
        expect(authorized({ request: requestWith(`Basic ${btoa(`lfs:${TOKEN}`,)}`,), env: bare, l, },),).toBe(false,);
      },
    },),
    it({
      name: 'refuses every write while the secret is blank',
      fn: async () => {
        /**
         Env whose secret was stored empty, as `wrangler secret put` does when its stdin yields nothing.
         Without a blank check the empty password below would compare equal to the empty secret.
         */
        const blank: WorkerEnv = { BUCKET: createMemoryObjectStore(), LFS_WRITE_TOKEN: '', };
        expect(authorized({ request: requestWith(`Basic ${btoa('lfs:',)}`,), env: blank, l, },),).toBe(false,);
        expect(authorized({ request: requestWith(`Basic ${btoa(`lfs:${TOKEN}`,)}`,), env: blank, l, },),).toBe(false,);
      },
    },),
    it({
      name: 'refuses every write while the secret holds only whitespace',
      fn: async () => {
        /**
         Env whose secret carries no credential characters.
         The request password matches the secret byte for byte, so only the blank check refuses it.
         */
        const whitespace: WorkerEnv = { BUCKET: createMemoryObjectStore(), LFS_WRITE_TOKEN: ' ', };
        expect(authorized({ request: requestWith(`Basic ${btoa('lfs: ',)}`,), env: whitespace, l, },),).toBe(false,);
      },
    },),
    it({
      name: 'refuses a request without an Authorization header',
      fn: async () => {
        expect(authorized({ request: requestWith(), env, l, },),).toBe(false,);
      },
    },),
    it({
      name: 'refuses a non-Basic scheme',
      fn: async () => {
        expect(authorized({ request: requestWith(`Bearer ${TOKEN}`,), env, l, },),).toBe(false,);
      },
    },),
    it({
      name: 'refuses a credential that is not base64',
      fn: async () => {
        expect(authorized({ request: requestWith('Basic %%%',), env, l, },),).toBe(false,);
      },
    },),
    it({
      name: 'refuses a wrong password',
      fn: async () => {
        expect(authorized({ request: requestWith(`Basic ${btoa('lfs:wrong',)}`,), env, l, },),).toBe(false,);
      },
    },),
    it({
      name: 'accepts the token as password with any username',
      fn: async () => {
        expect(authorized({ request: requestWith(`Basic ${btoa(`lfs:${TOKEN}`,)}`,), env, l, },),).toBe(true,);
        expect(authorized({ request: requestWith(`Basic ${btoa(`anyone:${TOKEN}`,)}`,), env, l, },),).toBe(true,);
      },
    },),
    it({
      name: 'splits on the first colon so a token containing colons still matches',
      fn: async () => {
        expect(authorized({ request: requestWith(`Basic ${btoa(`u:${TOKEN}`,)}`,), env, l, },),).toBe(true,);
        expect(authorized({ request: requestWith(`Basic ${btoa('u:se',)}`,), env, l, },),).toBe(false,);
      },
    },),
  ],
},);
