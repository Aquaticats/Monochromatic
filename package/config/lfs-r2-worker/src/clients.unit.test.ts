/**
 Tests for the client token store: the sha256 digest a token row holds, the
 constant-time comparison of digests, and the queries that read one row.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  anyClientRow,
  constantTimeEquals,
  createMemoryDatabase,
  findClient,
  isSha256Hex,
  SELECT_ANY_CLIENT_SQL,
  SELECT_CLIENT_SQL,
  sha256Hex,
  SHA256_HEX_LENGTH,
  UPLOAD_SCOPE,
  type WorkerEnv,
} from '@monochromatic-dev/config-lfs-r2-worker';

/**
 Logger shared by every case.
 */
const l = tagged({ tag: 'clients.unit.test', },);

/**
 Input whose digest is a published test vector.
 */
const VECTOR_INPUT = 'abc';

/**
 Published sha256 digest of `VECTOR_INPUT`.
 */
const VECTOR_DIGEST = 'ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad';

/**
 Build an env whose only D1 binding is the given memory database.

 @param clients - client rows the database holds

 @param failure - error every statement throws

 @returns env with a memory database
 */
function envWith(
  clients: readonly {
    readonly id: string;
    readonly hash: string;
    readonly scope?: string;
    readonly expires?: number | null;
    readonly revoked?: number | null;
  }[] = [],
  failure?: Error,
): WorkerEnv & { readonly INDEX: ReturnType<typeof createMemoryDatabase>; } {
  return {
    BUCKET: {
      head: async () => null,
      get: async () => null,
      put: async () => undefined,
    },
    INDEX: createMemoryDatabase({
      clients,
      ...failure === undefined ? {} : { failure, },
    },),
  };
}

await describe({
  name: '',
  children: [
    describe({
      name: sha256Hex.name,
      children: [
        it({
          name: 'answers the published digest of a known input',
          fn: async () => {
            expect(await sha256Hex(VECTOR_INPUT,),).toBe(VECTOR_DIGEST,);
          },
        },),
        it({
          name: 'answers lowercase hex of the fixed sha256 length for a non-ASCII input',
          fn: async () => {
            /**
             Digest of text that has more bytes than characters.
             */
            const digest = await sha256Hex('ünïcöde',);
            expect(digest,).toHaveLength(SHA256_HEX_LENGTH,);
            expect(isSha256Hex(digest,),).toBe(true,);
          },
        },),
      ],
    },),
    describe({
      name: isSha256Hex.name,
      children: [
        it({
          name: 'accepts a lowercase digest of the sha256 length',
          fn: async () => {
            expect(isSha256Hex(VECTOR_DIGEST,),).toBe(true,);
          },
        },),
        ...([
          ['an uppercase digest', VECTOR_DIGEST.toUpperCase(),],
          ['one character short', VECTOR_DIGEST.slice(1,),],
          ['one character too long', `${VECTOR_DIGEST}0`,],
          ['a non-hex character', `${VECTOR_DIGEST.slice(0, -1,)}z`,],
          ['an empty value', '',],
        ] as const).map(function mapRejected([description, value,]: readonly [string, string,],) {
          return it({
            name: `refuses ${description}`,
            fn: async () => {
              expect(isSha256Hex(value,),).toBe(false,);
            },
          },);
        },),
      ],
    },),
    describe({
      name: constantTimeEquals.name,
      children: [
        it({
          name: 'accepts equal values',
          fn: async () => {
            expect(constantTimeEquals(VECTOR_DIGEST, VECTOR_DIGEST,),).toBe(true,);
            expect(constantTimeEquals('', '',),).toBe(true,);
          },
        },),
        it({
          name: 'refuses a value that differs in its first character',
          fn: async () => {
            expect(constantTimeEquals(`0${VECTOR_DIGEST.slice(1,)}`, VECTOR_DIGEST,),).toBe(false,);
          },
        },),
        it({
          name: 'refuses a value that differs in its last character',
          fn: async () => {
            expect(constantTimeEquals(`${VECTOR_DIGEST.slice(0, -1,)}0`, VECTOR_DIGEST,),).toBe(false,);
          },
        },),
        it({
          name: 'refuses a value that is a prefix of the other, in either order',
          fn: async () => {
            /**
             First half of the digest, which is a prefix of it.
             */
            const prefix = VECTOR_DIGEST.slice(0, 32,);
            expect(constantTimeEquals(prefix, VECTOR_DIGEST,),).toBe(false,);
            expect(constantTimeEquals(VECTOR_DIGEST, prefix,),).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: findClient.name,
      children: [
        it({
          name: 'answers the row of a known client id',
          fn: async () => {
            /**
             Client id the database holds a row for.
             */
            const id = 'machine-one';
            /**
             Digest the row holds.
             */
            const hash = await sha256Hex('token',);
            /**
             Outcome of the lookup.
             */
            const lookup = await findClient({
              env: envWith([
                {
                  id,
                  hash,
                  scope: UPLOAD_SCOPE,
                  expires: null,
                  revoked: null,
                },
              ],),
              id,
              l,
            },);
            expect(lookup.kind,).toBe('found',);
            expect(lookup.kind === 'found' ? lookup.client.hash : '',).toBe(hash,);
            expect(lookup.kind === 'found' ? lookup.client.scope : '',).toBe(UPLOAD_SCOPE,);
            expect(lookup.kind === 'found' ? lookup.client.expires : 0,).toBeNull();
            expect(lookup.kind === 'found' ? lookup.client.revoked : 0,).toBeNull();
          },
        },),
        it({
          name: 'answers absent for a client id the database does not hold',
          fn: async () => {
            /**
             Outcome of the lookup.
             */
            const lookup = await findClient({
              env: envWith(),
              id: 'machine-nine',
              l,
            },);
            expect(lookup.kind,).toBe('absent',);
          },
        },),
        it({
          name: 'answers malformed when the row holds a hash that is not text',
          fn: async () => {
            /**
             Outcome of the lookup.
             */
            const lookup = await findClient({
              env: {
                BUCKET: {
                  head: async () => null,
                  get: async () => null,
                  put: async () => undefined,
                },
                INDEX: {
                  prepare: function prepare() {
                    return {
                      bind: function bind() {
                        return this;
                      },
                      all: async () => ({ results: [{ expires: null, hash: 5, revoked: null, scope: UPLOAD_SCOPE, },], }),
                      run: async () => undefined,
                    };
                  },
                },
              },
              id: 'machine-one',
              l,
            },);
            expect(lookup.kind,).toBe('malformed',);
          },
        },),
        it({
          name: 'asks the database for the row of the client id',
          fn: async () => {
            /**
             Database whose prepared statements are recorded.
             */
            const database = createMemoryDatabase({ clients: [], },);
            await findClient({
              env: { BUCKET: envWith().BUCKET, INDEX: database, },
              id: 'machine-one',
              l,
            },);
            expect(database.statements,).toContain(SELECT_CLIENT_SQL,);
          },
        },),
        it({
          name: 'surfaces a database failure so the caller can refuse the write',
          fn: async () => {
            /**
             Failure the database raises.
             */
            const failure = new Error('d1 unavailable',);
            /**
             Rejection the lookup surfaced.
             */
            let caught: unknown;
            try {
              await findClient({
                env: envWith([], failure,),
                id: 'machine-one',
                l,
              },);
            }
            catch (error) {
              caught = error;
            }
            expect((caught as Error).message,).toBe('d1 unavailable',);
          },
        },),
      ],
    },),
    describe({
      name: anyClientRow.name,
      children: [
        it({
          name: 'answers false for an empty clients table',
          fn: async () => {
            expect(await anyClientRow({ env: envWith(), l, },),).toBe(false,);
          },
        },),
        it({
          name: 'answers true once one row exists, even a revoked one',
          fn: async () => {
            expect(await anyClientRow({
              env: envWith([{ id: 'machine-one', hash: await sha256Hex('token',), revoked: 1, },],),
              l,
            },),).toBe(true,);
          },
        },),
        it({
          name: 'asks the database whether any row exists',
          fn: async () => {
            /**
             Database whose prepared statements are recorded.
             */
            const database = createMemoryDatabase({ clients: [], },);
            await anyClientRow({ env: { BUCKET: envWith().BUCKET, INDEX: database, }, l, },);
            expect(database.statements,).toContain(SELECT_ANY_CLIENT_SQL,);
          },
        },),
        it({
          name: 'surfaces a database failure so the caller can refuse the write',
          fn: async () => {
            /**
             Failure the database raises.
             */
            const failure = new Error('d1 unavailable',);
            /**
             Rejection the query surfaced.
             */
            let caught: unknown;
            try {
              await anyClientRow({ env: envWith([], failure,), l, },);
            }
            catch (error) {
              caught = error;
            }
            expect((caught as Error).message,).toBe('d1 unavailable',);
          },
        },),
      ],
    },),
  ],
},);
