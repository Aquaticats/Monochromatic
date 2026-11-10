/**
 Tests for upload authorization: one row of per-machine client tokens in the D1
 database, and the shared secret that stops being accepted once those rows exist.

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
  createMemoryDatabase,
  createMemoryObjectStore,
  type MemoryClientFixture,
  sha256Hex,
  UPLOAD_SCOPE,
  type WorkerEnv,
} from '@monochromatic-dev/config-lfs-r2-worker';

/**
 Logger shared by every case.
 */
const l = tagged({ tag: 'authorize.unit.test', },);

/**
 Shared upload secret from before the per-machine rows existed.

 It is accepted only while the clients table holds no row, which is what these
 cases pin.
 */
const LEGACY_TOKEN = 'se:cret';

/**
 Client id of the machine these cases simulate, used as the Basic user name.
 */
const CLIENT_ID = 'machine-one';

/**
 Token that machine holds, used as the Basic password.
 */
const CLIENT_TOKEN = 'token:with:colons';

/**
 Clock every case authorizes against, so an expiry boundary is exact.
 */
const NOW = 1_800_000_000_000;

/**
 A client row that authorizes `CLIENT_TOKEN` for writes, with no expiry and standing.
 */
const VALID_CLIENT: MemoryClientFixture = {
  id: CLIENT_ID,
  hash: await sha256Hex(CLIENT_TOKEN,),
  scope: UPLOAD_SCOPE,
};

/**
 What an env is built from.
 */
type EnvOptions = {
  /**
   Client rows the D1 database holds; omit for an env without a database binding.
   */
  readonly clients?: readonly MemoryClientFixture[];
  /**
   Shared secret on the env; omit for an env without one.
   */
  readonly legacyToken?: string;
  /**
   Every client statement the database runs throws this error.
   */
  readonly clientFailure?: Error;
};

/**
 Build an env around the object store the cases need.

 @param options - client rows, shared secret and database failure for this env

 @returns env with a memory bucket, and a memory database only when the case names one
 */
function envWith(options: EnvOptions = {},): WorkerEnv {
  /**
   Client rows the env starts with.
   */
  const clients = options.clients ?? [];
  /**
   Whether the case needs a database binding at all.
   */
  const needsDatabase = (options.clients !== undefined) || (options.clientFailure !== undefined);
  return {
    BUCKET: createMemoryObjectStore(),
    ...options.legacyToken === undefined ? {} : { LFS_WRITE_TOKEN: options.legacyToken, },
    ...needsDatabase
      ? {
        INDEX: createMemoryDatabase({
          clients,
          ...options.clientFailure === undefined ? {} : { clientFailure: options.clientFailure, },
        },),
      }
      : {},
  };
}

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

/**
 Basic header carrying a user name and password.

 @param username - Basic user name

 @param password - Basic password

 @returns header value
 */
function basic({
  username,
  password,
}: { readonly username: string; readonly password: string; },): string {
  return `Basic ${btoa(`${username}:${password}`,)}`;
}

/**
 Authorize one request against an env, at the fixed clock.

 @param authorization - header value the request carries

 @param env - env under test

 @returns whether the write is allowed
 */
async function allows(authorization: string, env: WorkerEnv,): Promise<boolean> {
  return await authorized({
    request: requestWith(authorization,),
    env,
    now: NOW,
    l,
  },);
}

/**
 Authorize a request that carries no `Authorization` header at all.

 @param env - env under test

 @returns whether the write is allowed
 */
async function allowsWithoutHeader(env: WorkerEnv,): Promise<boolean> {
  return await authorized({
    request: requestWith(),
    env,
    now: NOW,
    l,
  },);
}

await describe({
  name: authorized.name,
  children: [
    //region No credential can be accepted

    it({
      name: 'refuses every write while neither a client row nor a shared secret exists',
      fn: async () => {
        expect(await allows(basic({ username: 'lfs', password: LEGACY_TOKEN, },), envWith(),),).toBe(false,);
      },
    },),
    it({
      name: 'refuses every write while the shared secret is blank and no client row exists',
      fn: async () => {
        /**
         Env whose secret was stored empty, as `wrangler secret put` does when its stdin yields nothing.
         Without a blank check the empty password below would compare equal to the empty secret.
         */
        const blank = envWith({ legacyToken: '', },);
        expect(await allows(basic({ username: 'lfs', password: '', },), blank,),).toBe(false,);
        expect(await allows(basic({ username: 'lfs', password: LEGACY_TOKEN, },), blank,),).toBe(false,);
      },
    },),
    it({
      name: 'refuses every write while the shared secret holds only whitespace',
      fn: async () => {
        /**
         Env whose secret carries no credential characters.
         The request password matches the secret byte for byte, so only the blank check refuses it.
         */
        const whitespace = envWith({ legacyToken: ' ', },);
        expect(await allows(basic({ username: 'lfs', password: ' ', },), whitespace,),).toBe(false,);
      },
    },),
    it({
      name: 'refuses a request without an Authorization header',
      fn: async () => {
        expect(
          await allowsWithoutHeader(envWith({ legacyToken: LEGACY_TOKEN, },),),
        ).toBe(false,);
      },
    },),
    it({
      name: 'refuses a non-Basic scheme',
      fn: async () => {
        expect(await allows(`Bearer ${LEGACY_TOKEN}`, envWith({ legacyToken: LEGACY_TOKEN, },),),).toBe(false,);
      },
    },),
    it({
      name: 'refuses a credential that is not base64',
      fn: async () => {
        expect(await allows('Basic %%%', envWith({ legacyToken: LEGACY_TOKEN, },),),).toBe(false,);
      },
    },),

    //endregion No credential can be accepted

    //region The shared secret is accepted only until the first client row exists

    it({
      name: 'accepts the shared secret as the password while the clients table is empty',
      fn: async () => {
        /**
         Env of a Worker deployed before the first per-machine row was written.
         */
        const beforeMigration = envWith({
          clients: [],
          legacyToken: LEGACY_TOKEN,
        },);
        expect(await allows(basic({ username: 'lfs', password: LEGACY_TOKEN, },), beforeMigration,),).toBe(true,);
        expect(await allows(basic({ username: 'anyone', password: LEGACY_TOKEN, },), beforeMigration,),).toBe(true,);
      },
    },),
    it({
      name: 'accepts the shared secret with no user name, as the old URL form sent it',
      fn: async () => {
        expect(await allows(
          basic({ username: '', password: LEGACY_TOKEN, },),
          envWith({ clients: [], legacyToken: LEGACY_TOKEN, },),
        ),).toBe(true,);
      },
    },),
    it({
      name: 'refuses the shared secret once one client row exists',
      fn: async () => {
        /**
         Env a coordinator reaches by writing the first per-machine row, with no Worker deploy in between.
         */
        const afterFirstRow = envWith({
          clients: [VALID_CLIENT,],
          legacyToken: LEGACY_TOKEN,
        },);
        expect(await allows(basic({ username: 'lfs', password: LEGACY_TOKEN, },), afterFirstRow,),).toBe(false,);
      },
    },),
    it({
      name: 'refuses the shared secret while every client row is revoked, because the switch is the row itself',
      fn: async () => {
        /**
         Env whose only row was revoked, which must not re-open the shared secret.
         */
        const allRevoked = envWith({
          clients: [{ ...VALID_CLIENT, revoked: NOW, },],
          legacyToken: LEGACY_TOKEN,
        },);
        expect(await allows(basic({ username: 'lfs', password: LEGACY_TOKEN, },), allRevoked,),).toBe(false,);
      },
    },),
    it({
      name: 'refuses the shared secret when the client row belongs to another token',
      fn: async () => {
        /**
         Env whose only row cannot authorize the presented password, so only the switch refuses it.
         */
        const otherMachine = envWith({
          clients: [{ ...VALID_CLIENT, id: 'machine-two', hash: await sha256Hex('another-token',), },],
          legacyToken: LEGACY_TOKEN,
        },);
        expect(await allows(basic({ username: 'lfs', password: LEGACY_TOKEN, },), otherMachine,),).toBe(false,);
      },
    },),

    //endregion The shared secret is accepted only until the first client row exists

    //region A per-machine token is what authorizes a write

    it({
      name: 'accepts the token of the client the user name names',
      fn: async () => {
        expect(await allows(
          basic({ username: CLIENT_ID, password: CLIENT_TOKEN, },),
          envWith({ clients: [VALID_CLIENT,], },),
        ),).toBe(true,);
      },
    },),
    it({
      name: 'refuses a wrong password for a known client',
      fn: async () => {
        expect(await allows(
          basic({ username: CLIENT_ID, password: 'wrong', },),
          envWith({ clients: [VALID_CLIENT,], },),
        ),).toBe(false,);
      },
    },),
    it({
      name: 'refuses an unknown client id',
      fn: async () => {
        expect(await allows(
          basic({ username: 'machine-nine', password: CLIENT_TOKEN, },),
          envWith({ clients: [VALID_CLIENT,], },),
        ),).toBe(false,);
      },
    },),
    it({
      name: 'never falls back to the shared secret for the id of a known client',
      fn: async () => {
        /**
         Env that still holds the shared secret next to the client rows.
         A known id with the shared secret as its password must be refused: the id selects the row, and only its
         hash decides.
         */
        const mixed = envWith({
          clients: [VALID_CLIENT,],
          legacyToken: LEGACY_TOKEN,
        },);
        expect(await allows(basic({ username: CLIENT_ID, password: LEGACY_TOKEN, },), mixed,),).toBe(false,);
      },
    },),
    it({
      name: 'refuses a revoked client',
      fn: async () => {
        expect(await allows(
          basic({ username: CLIENT_ID, password: CLIENT_TOKEN, },),
          envWith({ clients: [{ ...VALID_CLIENT, revoked: NOW - 1, },], },),
        ),).toBe(false,);
      },
    },),
    it({
      name: 'refuses a client whose expiry is the current instant',
      fn: async () => {
        expect(await allows(
          basic({ username: CLIENT_ID, password: CLIENT_TOKEN, },),
          envWith({ clients: [{ ...VALID_CLIENT, expires: NOW, },], },),
        ),).toBe(false,);
      },
    },),
    it({
      name: 'refuses an expired client and accepts one that expires later',
      fn: async () => {
        expect(await allows(
          basic({ username: CLIENT_ID, password: CLIENT_TOKEN, },),
          envWith({ clients: [{ ...VALID_CLIENT, expires: NOW - 1, },], },),
        ),).toBe(false,);
        expect(await allows(
          basic({ username: CLIENT_ID, password: CLIENT_TOKEN, },),
          envWith({ clients: [{ ...VALID_CLIENT, expires: NOW + 1, },], },),
        ),).toBe(true,);
      },
    },),
    it({
      name: 'refuses a client whose scope does not grant writes',
      fn: async () => {
        expect(await allows(
          basic({ username: CLIENT_ID, password: CLIENT_TOKEN, },),
          envWith({ clients: [{ ...VALID_CLIENT, scope: 'read', },], },),
        ),).toBe(false,);
      },
    },),
    it({
      name: 'refuses a client row whose hash is not a sha256 digest',
      fn: async () => {
        expect(await allows(
          basic({ username: CLIENT_ID, password: CLIENT_TOKEN, },),
          envWith({ clients: [{ ...VALID_CLIENT, hash: 'not-a-digest', },], },),
        ),).toBe(false,);
      },
    },),

    //endregion A per-machine token is what authorizes a write

    //region The database decides, and a database that cannot answer refuses

    it({
      name: 'refuses the write when the client lookup fails, and leaves the shared secret unused',
      fn: async () => {
        /**
         Env whose database refuses every client statement, as a missing table or an outage would.
         */
        const failing = envWith({
          clients: [VALID_CLIENT,],
          legacyToken: LEGACY_TOKEN,
          clientFailure: new Error('d1 unavailable',),
        },);
        expect(await allows(basic({ username: CLIENT_ID, password: CLIENT_TOKEN, },), failing,),).toBe(false,);
        expect(await allows(basic({ username: 'lfs', password: LEGACY_TOKEN, },), failing,),).toBe(false,);
      },
    },),

    //endregion The database decides, and a database that cannot answer refuses
  ],
},);
