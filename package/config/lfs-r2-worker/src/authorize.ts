/**
 Upload authorization: git-lfs sends a client id as the Basic user name and that
 machine's token as the password.

 The row the id names decides the answer, and a row that is revoked, expired, out
 of scope or carries another token refuses the write outright, so a known id never
 falls back to anything else.

 The shared upload secret (`LFS_WRITE_TOKEN`) is accepted only while the clients
 table holds no row: writing the first per-machine row retires it, which needs no
 second deploy and no flag. After every machine holds a row the secret can be
 deleted with `wrangler secret delete LFS_WRITE_TOKEN`, and then no code path can
 accept it at all.

 Reads never reach this module: downloads are anonymous by design.

 @module
 */

import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';

import {
  anyClientRow,
  type ClientLookup,
  type ClientRecord,
  findClient,
  UPLOAD_SCOPE,
} from './clients.ts';
import { constantTimeEquals, } from './constant-time-equal.ts';
import {
  isSha256Hex,
  sha256Hex,
} from './sha256-hex.ts';
import type { WorkerEnv, } from './store.ts';

/**
 Scheme prefix of an HTTP Basic `Authorization` header.
 */
const BASIC_PREFIX = 'Basic ';

/**
 A Basic credential split at its first colon.

 git-lfs sends the user name and password its Git credential helper returned, and
 only a colon splits the pair, so a token holding colons survives intact.
 */
type BasicCredential = {
  /**
   Part before the first colon; the client id the row is read by.
   */
  readonly username: string;
  /**
   Part after the first colon; the machine's token.
   */
  readonly password: string;
};

/**
 The credential half of an `Authorization` header, or why it cannot be read.
 */
type BasicCredentialLookup = {
  /**
   Split credential.
   */
  readonly credential: BasicCredential;
} | {
  /**
   Why the header could not be decoded.
   */
  readonly detail: string;
};

/**
 What one authorization stage concluded.
 */
type Decision = 'accept' | 'refuse' | 'unknown-client';

/**
 Parameters for {@link authorized}.
 */
export type AuthorizedParams = {
  /**
   Inbound request whose `Authorization` header is inspected.
   */
  readonly request: Request;
  /**
   Worker env carrying the database binding and, before the migration, the shared secret.
   */
  readonly env: WorkerEnv;
  /**
   Logger for the authorization decision.
   */
  readonly l: Logger;
  /**
   Epoch milliseconds to judge expiries against; defaults to the wall clock and is
   fixed by tests so a boundary is exact.
   */
  readonly now?: number;
};

/**
 Split the credential half of an `Authorization` header at its first colon.

 @param encoded - base64 credential following the `Basic ` prefix

 @returns the user name and password, or why the text is not base64

 @example
 ```ts
 parseBasicCredential(btoa('machine-one:token'));
 ```
 */
function parseBasicCredential(encoded: string,): BasicCredentialLookup {
  try {
    /**
     Decoded `username:password` pair.
     */
    const decoded = atob(encoded,);
    /**
     Index of the colon that ends the user name; a pair without one is all user name.
     */
    const separator = decoded.indexOf(':',);
    if (separator === (-1)) {
      return {
        credential: {
          username: decoded,
          password: '',
        },
      };
    }
    return {
      credential: {
        username: decoded.slice(
          0,
          separator,
        ),
        password: decoded.slice(separator + 1,),
      },
    };
  }
  catch (error) {
    return { detail: `the credential is not valid base64: ${String(error,)}`, };
  }
}

/**
 Parameters for {@link decideForClient}.
 */
type DecideForClientParams = {
  /**
   Worker env whose database holds the rows.
   */
  readonly env: WorkerEnv;
  /**
   Credential the request carried.
   */
  readonly credential: BasicCredential;
  /**
   Epoch milliseconds to judge expiries against.
   */
  readonly now: number;
  /**
   Logger for the decision.
   */
  readonly l: Logger;
};

/**
 Judge the row the presented user name names.

 @param env - Worker env whose database holds the rows

 @param credential - credential the request carried

 @param now - epoch milliseconds to judge expiries against

 @param l - logger for the decision

 @returns `accept`, `refuse`, or `unknown-client` when no row carries that id
 */
async function decideForClient({
  env,
  credential,
  now,
  l,
}: DecideForClientParams,): Promise<Decision> {
  /**
   Logger tagged with this function's name.
   */
  const dl = tagged({
    tag: decideForClient.name,
    l,
  },);
  if (env.INDEX === undefined) {
    return 'unknown-client';
  }
  /**
   Row of the presented id, when it can be read.
   */
  const lookup: ClientLookup = credential.username === ''
    ? { kind: 'absent', }
    : await findClient({
      env,
      id: credential.username,
    },);
  if (lookup.kind === 'absent') {
    return 'unknown-client';
  }
  if (lookup.kind === 'failed') {
    dl.error(`client lookup failed, refusing the write: ${String(lookup.cause,)}`,);
    return 'refuse';
  }
  if (lookup.kind === 'malformed') {
    dl.error(`client row for ${credential.username} is unreadable, refusing the write: ${lookup.detail}`,);
    return 'refuse';
  }
  return await judgeClientRow({
    client: lookup.client,
    credential,
    id: credential.username,
    now,
    l: dl,
  },);
}

/**
 Parameters for {@link judgeClientRow}.
 */
type JudgeClientRowParams = {
  /**
   Row of the presented client id.
   */
  readonly client: ClientRecord;
  /**
   Credential the request carried.
   */
  readonly credential: BasicCredential;
  /**
   Client id, for the log line.
   */
  readonly id: string;
  /**
   Epoch milliseconds to judge the expiry against.
   */
  readonly now: number;
  /**
   Logger for the decision.
   */
  readonly l: Logger;
};

/**
 Judge one readable row against the presented password and the clock.

 The token is digested before any check runs, so a refusal costs the same whether
 the row is revoked, expired or simply carries a different token.

 @param client - row of the presented client id

 @param credential - credential the request carried

 @param id - client id, for the log line

 @param now - epoch milliseconds to judge the expiry against

 @param l - logger for the decision

 @returns `accept` when the row grants writes and its digest matches; else `refuse`
 */
async function judgeClientRow({
  client,
  credential,
  id,
  now,
  l,
}: JudgeClientRowParams,): Promise<Decision> {
  /**
   Logger tagged with this function's name.
   */
  const jl = tagged({
    tag: judgeClientRow.name,
    l,
  },);
  /**
   Digest of the presented password, which a row stores instead of the token.
   */
  const presented = await sha256Hex(credential.password,);
  if (client.revoked !== undefined) {
    jl.warn(`client ${id} was revoked; refusing the write`,);
    return 'refuse';
  }
  if ((client.expires !== undefined) && (client.expires <= now)) {
    jl.warn(`client ${id} expired; refusing the write`,);
    return 'refuse';
  }
  if (client.scope !== UPLOAD_SCOPE) {
    jl.warn(`client ${id} holds scope ${client.scope}, which does not grant a write; refusing the write`,);
    return 'refuse';
  }
  if (!isSha256Hex(client.hash,)) {
    jl.error(`client ${id} holds a hash that is not a sha256 digest; refusing the write`,);
    return 'refuse';
  }
  if (!constantTimeEquals({
    left: client.hash,
    right: presented,
  },)) {
    jl.warn(`client ${id} presented a token that does not match; refusing the write`,);
    return 'refuse';
  }
  jl.info(`client ${id} authorized`,);
  return 'accept';
}

/**
 Parameters for {@link decideForSharedSecret}.
 */
type DecideForSharedSecretParams = {
  /**
   Worker env carrying the database binding and the shared secret.
   */
  readonly env: WorkerEnv;
  /**
   Credential the request carried.
   */
  readonly credential: BasicCredential;
  /**
   Logger for the decision.
   */
  readonly l: Logger;
};

/**
 Judge the shared upload secret, the credential this Worker shipped with.

 The secret answers only while the clients table holds no row: the first row is
 what retires it. A database that cannot answer refuses the secret too, because
 whether a row exists is exactly what decides if the secret still counts.

 @param env - Worker env carrying the database binding and the shared secret

 @param credential - credential the request carried

 @param l - logger for the decision

 @returns `accept` when the secret is configured, no client row exists and it matches; else `refuse`
 */
async function decideForSharedSecret({
  env,
  credential,
  l,
}: DecideForSharedSecretParams,): Promise<Decision> {
  /**
   Logger tagged with this function's name.
   */
  const sl = tagged({
    tag: decideForSharedSecret.name,
    l,
  },);
  /**
   Configured shared secret; absent on a fresh deploy and after the migration deletes it.
   `wrangler secret put` also stores a blank value without complaint when its stdin
   yields nothing, so a blank secret is refused here rather than compared against:
   an equality check would accept a request whose Basic password is blank too.
   See `doc/troubleshooting/wrangler-secret-put-empty-stdin.md`.
   */
  const { LFS_WRITE_TOKEN: token, } = env;
  if ((token === undefined) || (token.trim() === '')) {
    sl.warn('no shared upload secret is configured; refusing the write',);
    return 'refuse';
  }
  if (env.INDEX !== undefined) {
    /**
     Whether any per-machine row exists, which retires the shared secret.
     */
    const presence = await anyClientRow({ env, },);
    if (presence.kind === 'failed') {
      sl.error(`client table lookup failed, refusing the shared secret and the write: ${String(presence.cause,)}`,);
      return 'refuse';
    }
    if (presence.kind === 'occupied') {
      sl.warn('shared upload secret refused: per-machine client rows exist',);
      return 'refuse';
    }
  }
  if (!constantTimeEquals({
    left: token,
    right: credential.password,
  },)) {
    sl.warn('shared upload secret mismatch; refusing the write',);
    return 'refuse';
  }
  sl.info('shared upload secret accepted: no per-machine client row exists yet',);
  return 'accept';
}

/**
 Whether the request may store objects.

 The presented user name selects a `clients` row. That row alone decides, so a
 revoked, expired, out-of-scope or mismatched row refuses the write without trying
 anything else. Only an id no row names reaches the shared secret, which itself
 answers only while the table is empty.

 @param request - inbound request whose `Authorization` header is inspected

 @param env - Worker env carrying the database binding and, before the migration, the shared secret

 @param l - logger for the authorization decision

 @param now - epoch milliseconds to judge expiries against

 @returns `true` when the caller may write objects

 @example
 ```ts
 if (!(await authorized({ request, env, l }))) {
   return new Response('Unauthorized', { status: 401 });
 }
 ```
 */
export async function authorized({
  request,
  env,
  l,
  now = Date.now(),
}: AuthorizedParams,): Promise<boolean> {
  /**
   Logger tagged with this function's name.
   */
  const al = tagged({
    tag: authorized.name,
    l,
  },);
  /**
   Raw `Authorization` header; an absent header reads as empty, which fails the prefix check.
   */
  const header = request.headers
    .has('Authorization',)
    ? String(request.headers
      .get('Authorization',),)
    : '';
  if (!header.startsWith(BASIC_PREFIX,)) {
    al.warn('no Basic credentials on the request; refusing the write',);
    return false;
  }
  /**
   Credential the request carried, when it is valid base64.
   */
  const parsed = parseBasicCredential(header.slice(BASIC_PREFIX.length,),);
  if ('detail' in parsed) {
    al.warn(`malformed Basic credentials rejected: ${parsed.detail}`,);
    return false;
  }
  /**
   What the row of the presented client id concluded.
   */
  const clientDecision = await decideForClient({
    env,
    credential: parsed.credential,
    now,
    l: al,
  },);
  if (clientDecision !== 'unknown-client') {
    return clientDecision === 'accept';
  }
  return (await decideForSharedSecret({
    env,
    credential: parsed.credential,
    l: al,
  },)) === 'accept';
}
