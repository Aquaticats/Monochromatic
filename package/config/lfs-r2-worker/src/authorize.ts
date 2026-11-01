/**
 Upload authorization: git-lfs sends the write token as HTTP Basic auth.

 @module
 */

import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';

import type { WorkerEnv, } from './store.ts';

/**
 Scheme prefix of an HTTP Basic `Authorization` header.
 */
const BASIC_PREFIX = 'Basic ';

/**
 Parameters for {@link basicPasswordMatches}.
 */
type BasicPasswordMatchesParams = {
  /**
   Base64 credential following the `Basic ` prefix.
   */
  readonly encoded: string;
  /**
   Configured upload secret.
   */
  readonly token: string;
  /**
   Logger for the decode outcome.
   */
  readonly l: Logger;
};

/**
 Whether the password half of a Basic credential equals the token.

 @param encoded - base64 credential following the `Basic ` prefix

 @param token - configured upload secret

 @param l - logger for the decode outcome

 @returns `true` when the decoded password equals `token`; `false` for a
   mismatch or a credential that is not valid base64
 */
function basicPasswordMatches({
  encoded,
  token,
  l,
}: BasicPasswordMatchesParams,): boolean {
  /**
   Logger tagged with this function's name.
   */
  const bl = tagged({
    tag: basicPasswordMatches.name,
    l,
  },);
  try {
    /**
     Decoded `user:password` pair.
     */
    const decoded = atob(encoded,);
    /**
     Password after the first colon; the username is free-form.
     */
    const password = decoded.slice(decoded.indexOf(':',) + 1,);
    return password === token;
  }
  catch (error) {
    bl.warn(`malformed Basic credentials rejected: ${String(error,)}`,);
    return false;
  }
}

/**
 Parameters for {@link authorized}.
 */
export type AuthorizedParams = {
  /**
   Inbound request whose `Authorization` header is inspected.
   */
  readonly request: Request;
  /**
   Worker env carrying the `LFS_WRITE_TOKEN` upload secret.
   */
  readonly env: WorkerEnv;
  /**
   Logger for the authorization decision.
   */
  readonly l: Logger;
};

/**
 Whether the request carries the upload token.

 Only the password half of the Basic credential is compared against the
 secret, so the username is free-form; git-lfs sends whatever userinfo the
 configured `lfs.url` carries.

 A secret that is absent or blank refuses every write, because comparing
 against a blank value would accept a blank password.

 @param request - inbound request whose `Authorization` header is inspected

 @param env - Worker env carrying the `LFS_WRITE_TOKEN` upload secret

 @param l - logger for the authorization decision

 @returns `true` when the caller may write objects

 @example
 ```ts
 authorized({ request, env: { BUCKET, LFS_WRITE_TOKEN: 's3cr3t' }, l });
 ```
 */
export function authorized({
  request,
  env,
  l,
}: AuthorizedParams,): boolean {
  /**
   Logger tagged with this function's name.
   */
  const al = tagged({
    tag: authorized.name,
    l,
  },);
  /**
   Configured upload secret; absent on a fresh deploy until `wrangler secret put` runs.
   `wrangler secret put` also stores a blank value without complaint when its stdin
   yields nothing, so a blank secret is refused here rather than compared against:
   an equality check would accept a request whose Basic password is blank too.
   See `doc/troubleshooting/wrangler-secret-put-empty-stdin.md`.
   */
  const token = env.LFS_WRITE_TOKEN;
  if ((token === undefined) || (token.trim() === '')) {
    al.warn('LFS_WRITE_TOKEN is unset or blank; refusing the write',);
    return false;
  }
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
   Outcome of the comparison, logged so refused writes are traceable.
   */
  const matches = basicPasswordMatches({
    encoded: header.slice(BASIC_PREFIX.length,),
    token,
    l: al,
  },);
  if (!matches) {
    al.warn('upload token mismatch; refusing the write',);
  }
  return matches;
}
