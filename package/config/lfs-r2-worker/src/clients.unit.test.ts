/**
 Tests for the client token store: the sha256 digest a token row holds, the
 constant-time comparison of digests, and the queries that read one row.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  anyClientRow,
  constantTimeEquals,
  createMemoryDatabase,
  type D1DatabaseLike,
  type D1StatementLike,
  type DatabaseRow,
  findClient,
  isSha256Hex,
  type MemoryClientFixture,
  SELECT_ANY_CLIENT_SQL,
  SELECT_CLIENT_SQL,
  sha256Hex,
  SHA256_HEX_LENGTH,
  UPLOAD_SCOPE,
  type WorkerEnv,
} from '@monochromatic-dev/config-lfs-r2-worker';

/**
 Input whose digest is a published test vector.
 */
const VECTOR_INPUT = 'abc';

/**
 Published sha256 digest of `VECTOR_INPUT`.
 */
const VECTOR_DIGEST = 'ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad';

/**
 A bucket that stores nothing, for envs whose case never reaches it.
 */
const EMPTY_BUCKET = {
  head: () => Promise.resolve(null,),
  get: () => Promise.resolve(null,),
  put: () => Promise.resolve(undefined,),
};

/**
 Build an env whose only D1 binding is a memory database with the given rows.

 @param clients - client rows the database holds

 @param failure - error every client statement throws

 @returns env with a memory database
 */
function envWith(
  clients: readonly MemoryClientFixture[] = [],
  failure?: Error,
): WorkerEnv & { readonly INDEX: ReturnType<typeof createMemoryDatabase>; } {
  return {
    BUCKET: EMPTY_BUCKET,
    INDEX: createMemoryDatabase({
      clients,
      ...failure === undefined ? {} : { clientFailure: failure, },
    },),
  };
}

/**
 Build a database that answers every query with the rows a test hands it, so a row
 whose columns have the wrong types can be exercised.

 @param rows - rows every query answers with

 @returns database answering those rows
 */
function databaseAnswering(rows: readonly DatabaseRow[],): D1DatabaseLike {
  /**
   Statement answering the rows this database was built with.

   @returns statement whose `bind` answers the same rows
   */
  function statement(): D1StatementLike {
    return {
      bind: statement,
      all: () => Promise.resolve({ results: rows, },),
      run: () => Promise.resolve(undefined,),
    };
  }
  return {
    prepare: () => statement(),
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
            expect(digest.length,).toBe(SHA256_HEX_LENGTH,);
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
            expect(constantTimeEquals({ left: VECTOR_DIGEST, right: VECTOR_DIGEST, },),).toBe(true,);
            expect(constantTimeEquals({ left: '', right: '', },),).toBe(true,);
          },
        },),
        it({
          name: 'refuses a value that differs in its first character',
          fn: async () => {
            expect(constantTimeEquals({ left: `0${VECTOR_DIGEST.slice(1,)}`, right: VECTOR_DIGEST, },),).toBe(false,);
          },
        },),
        it({
          name: 'refuses a value that differs in its last character',
          fn: async () => {
            expect(constantTimeEquals({ left: `${VECTOR_DIGEST.slice(0, -1,)}0`, right: VECTOR_DIGEST, },),).toBe(false,);
          },
        },),
        it({
          name: 'refuses a value that is a prefix of the other, in either order',
          fn: async () => {
            /**
             First half of the digest, which is a prefix of it.
             */
            const prefix = VECTOR_DIGEST.slice(0, 32,);
            expect(constantTimeEquals({ left: prefix, right: VECTOR_DIGEST, },),).toBe(false,);
            expect(constantTimeEquals({ left: VECTOR_DIGEST, right: prefix, },),).toBe(false,);
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
                },
              ],),
              id,
            },);
            expect(lookup.kind,).toBe('found',);
            expect(lookup.kind === 'found' ? lookup.client.hash : '',).toBe(hash,);
            expect(lookup.kind === 'found' ? lookup.client.scope : '',).toBe(UPLOAD_SCOPE,);
            expect(lookup.kind === 'found' ? lookup.client.expires : 0,).toBeUndefined();
            expect(lookup.kind === 'found' ? lookup.client.revoked : 0,).toBeUndefined();
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
            },);
            expect(lookup.kind,).toBe('absent',);
          },
        },),
        ...([
          ['hash', { hash: 5, scope: UPLOAD_SCOPE, expires: null, revoked: null, },],
          ['scope', { hash: VECTOR_DIGEST, scope: 5, expires: null, revoked: null, },],
          ['expires', { hash: VECTOR_DIGEST, scope: UPLOAD_SCOPE, expires: 'soon', revoked: null, },],
          ['revoked', { hash: VECTOR_DIGEST, scope: UPLOAD_SCOPE, expires: null, revoked: 'yesterday', },],
        ] as const).map(function mapMalformed([column, row,]: readonly [string, DatabaseRow,],) {
          return it({
            name: `answers malformed for a row whose ${column} column has the wrong type`,
            fn: async () => {
              /**
               Outcome of the lookup.
               */
              const lookup = await findClient({
                env: { BUCKET: EMPTY_BUCKET, INDEX: databaseAnswering([row,],), },
                id: 'machine-one',
              },);
              expect(lookup.kind,).toBe('malformed',);
              expect(lookup.kind === 'malformed' ? lookup.detail : '',).toContain(column,);
            },
          },);
        },),
        it({
          name: 'asks the database for the row of the client id',
          fn: async () => {
            /**
             Database whose prepared statements are recorded.
             */
            const database = createMemoryDatabase({ clients: [], },);
            await findClient({
              env: { BUCKET: EMPTY_BUCKET, INDEX: database, },
              id: 'machine-one',
            },);
            expect(database.statements,).toContain(SELECT_CLIENT_SQL,);
          },
        },),
        it({
          name: 'answers failed when the query throws, so the caller can refuse the write',
          fn: async () => {
            /**
             Outcome of the lookup against a database that refuses every statement.
             */
            const lookup = await findClient({
              env: envWith([], new Error('d1 unavailable',),),
              id: 'machine-one',
            },);
            expect(lookup.kind,).toBe('failed',);
            expect(lookup.kind === 'failed' ? String(lookup.cause,) : '',).toContain('d1 unavailable',);
          },
        },),
      ],
    },),
    describe({
      name: anyClientRow.name,
      children: [
        it({
          name: 'answers empty for an empty clients table',
          fn: async () => {
            expect((await anyClientRow({ env: envWith(), },)).kind,).toBe('empty',);
          },
        },),
        it({
          name: 'answers occupied once one row exists, even a revoked one',
          fn: async () => {
            expect((await anyClientRow({
              env: envWith([{ id: 'machine-one', hash: await sha256Hex('token',), revoked: 1, },],),
            },)).kind,).toBe('occupied',);
          },
        },),
        it({
          name: 'asks the database whether any row exists',
          fn: async () => {
            /**
             Database whose prepared statements are recorded.
             */
            const database = createMemoryDatabase({ clients: [], },);
            await anyClientRow({ env: { BUCKET: EMPTY_BUCKET, INDEX: database, }, },);
            expect(database.statements,).toContain(SELECT_ANY_CLIENT_SQL,);
          },
        },),
        it({
          name: 'answers failed when the query throws, so the caller can refuse the write',
          fn: async () => {
            /**
             Outcome of the query against a database that refuses every statement.
             */
            const lookup = await anyClientRow({ env: envWith([], new Error('d1 unavailable',),), },);
            expect(lookup.kind,).toBe('failed',);
            expect(lookup.kind === 'failed' ? String(lookup.cause,) : '',).toContain('d1 unavailable',);
          },
        },),
      ],
    },),
  ],
},);
