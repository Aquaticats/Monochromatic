/**
 * Better Auth instance for the forge server.
 *
 * Sign-up + sign-in via email and password (Better Auth's default flow)
 * plus the `username` plugin so users have a normalised handle (`@alice`)
 * for URLs and rendered HTML. The handle replaces the legacy
 * `users.login` column once the data plane cuts over to the Better Auth
 * schema (a follow-up commit drops the legacy `users` table).
 *
 * Database wiring uses the Kysely libsql dialect against the same SQLite
 * file as `data/db.ts`. Both libraries open separate connections; SQLite
 * WAL mode (set by `data/db.ts`) handles concurrent access.
 *
 * Caveat for tests using `DB_PATH=:memory:`: each connection gets its
 * own in-memory database. `@tursodatabase/database` and Better Auth's
 * libsql client cannot see each other's data in that mode. Tests that
 * exercise auth flows must use a file-backed DB (e.g. `mkdtemp` +
 * `--db=/tmp/forge-test-${id}.db`) so both libraries share a SQLite
 * file.
 *
 * Required env: `BETTER_AUTH_SECRET`, `BETTER_AUTH_URL`. A deterministic
 * dev fallback exists for local development; production must override.
 */

import { LibsqlDialect, } from '@libsql/kysely-libsql';
import { logger, } from '@monochromatic-dev/logger-core/logger';
import { tagged, } from '@monochromatic-dev/logger-core/tagged';
import {
  type Auth,
  betterAuth,
} from 'better-auth';
import { username, } from 'better-auth/plugins';

import { getArgumentValue, } from './args.ts';

/**
 * Tagged logger scoped to the auth instance.
 */
const l = tagged({
  tag: 'auth',
  l: logger,
},);

/**
 * Default database path when neither `--db=` nor `DB_PATH` env is set.
 *
 * Mirrors `data/db.ts` so both libraries point at the same file.
 */
const DEFAULT_DATABASE_PATH = './data/forge.db';

/**
 * Default Better Auth base URL when env override is absent.
 */
const DEFAULT_BETTER_AUTH_URL = 'http://localhost:3000';

/**
 * Deterministic development fallback for `BETTER_AUTH_SECRET`. Not safe
 * for production; the env var must be set there. Documented inline so
 * grep finds it during the production-readiness audit.
 */
const DEV_BETTER_AUTH_SECRET =
  'dev-only-please-set-BETTER_AUTH_SECRET-in-production-12345';

/**
 * Resolves the libsql connection URL for Better Auth's Kysely dialect.
 *
 * Mirrors `data/db.ts`'s priority chain so both libraries point at
 * the same SQLite file: `--db=` argv beats `DB_PATH` env beats default.
 *
 * @returns libsql URL with `file:` scheme, or `:memory:` for in-memory
 *
 * @example
 * ```ts
 * const url = resolveLibsqlUrl();
 * ```
 */
function resolveLibsqlUrl(): string {
  /**
   * `--db=PATH` CLI argument when supplied; highest priority source.
   */
  const argumentPath = getArgumentValue('db',);
  /**
   * `DB_PATH` environment variable; second priority source.
   */
  const environmentPath = process.env
    .DB_PATH;
  /**
   * Selected raw path; falls back to the compile-time default.
   */
  const rawPath = argumentPath ?? environmentPath
    ?? DEFAULT_DATABASE_PATH;
  if (rawPath === ':memory:')
    return ':memory:';
  if (rawPath.startsWith('file:',))
    return rawPath;
  return `file:${rawPath}`;
}

/**
 * Resolved Better Auth secret; production must set `BETTER_AUTH_SECRET`.
 */
const betterAuthSecret = process.env
  .BETTER_AUTH_SECRET
  ?? DEV_BETTER_AUTH_SECRET;

/**
 * Resolved app base URL for Better Auth's session cookie scope.
 */
const betterAuthUrl = process.env
  .BETTER_AUTH_URL
  ?? DEFAULT_BETTER_AUTH_URL;

/**
 * Resolved libsql URL captured once so the log line and the dialect agree.
 */
const libsqlUrl = resolveLibsqlUrl();

l.info(`init libsql=${libsqlUrl} baseURL=${betterAuthUrl}`,);

/* oxlint-disable typescript/no-unsafe-type-assertion -- isolatedDeclarations forces an explicit type on this export, but Better Auth's `betterAuth(...)` returns a deeply-inferred `Auth<TConfig>` whose plugin tuple is invariant against the public `Auth` interface's `BetterAuthPlugin[]`; the cast through `unknown` is the documented escape hatch for this Better Auth pattern under isolatedDeclarations */

/**
 * Configured Better Auth instance for the forge server.
 *
 * Surfaces three things to consumers:
 *
 * - `auth.handler(req)` forwards a `Request` through the Better Auth
 *   route table at `/api/auth/...`
 * - `auth.api.*` exposes the typed server-side API for direct calls
 *   (sign-up, sign-in, session lookup) without going through HTTP
 * - `auth.$Infer` carries the inferred types for the user, session,
 *   and account shapes after plugins are merged
 *
 * @example
 * ```ts
 * import { auth } from './lib/auth.ts';
 * const session = await auth.api.getSession({ headers: req.headers });
 * ```
 */
export const auth = betterAuth({
  database: {
    dialect: new LibsqlDialect({ url: libsqlUrl, },),
    type: 'sqlite',
  },
  secret: betterAuthSecret,
  baseURL: betterAuthUrl,
  emailAndPassword: { enabled: true, },
  plugins: [username(),],
},) as unknown as Auth;
/* oxlint-enable typescript/no-unsafe-type-assertion */
