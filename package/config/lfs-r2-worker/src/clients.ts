/**
 The `clients` table: one row per machine allowed to upload objects.

 A row's `id` is the Basic user name and its `hash` is the sha256 of the password,
 so the database never holds a usable token. `scope` grants what the row may do,
 and `revoked` and `expires` take one machine out of service without touching any
 other: the columns are epoch milliseconds, and SQL NULL means "never expires" or
 "still standing", which this module reads as an absent property.

 The table lives in the same D1 database as the presence index, because a Worker
 binds that database already and a second one would need its own provisioning for
 no benefit at this size. Rows are written and revoked by hand with
 `wrangler d1 execute`, so there is no admin endpoint; the procedure is
 `doc/runbook/lfs-r2-worker.md`.

 @module
 */

import type {
  D1DatabaseLike,
  DatabaseRow,
  WorkerEnv,
} from './store.ts';

/**
 Scope a row needs before it may store objects.

 Every other scope is refused, because an upload is the only operation this Worker
 gates; a Worker that grows another scope needs a check next to the operation it
 protects.
 */
export const UPLOAD_SCOPE = 'write';

/**
 SQL reading the one row of a client id; `?1` is the id.
 */
export const SELECT_CLIENT_SQL = 'SELECT hash, scope, expires, revoked FROM clients WHERE id = ?1';

/**
 SQL asking whether the table holds any row at all, which is the switch that
 retires the shared upload secret.
 */
export const SELECT_ANY_CLIENT_SQL = 'SELECT id FROM clients LIMIT 1';

/**
 The columns {@link findClient} reads, narrowed.
 */
export type ClientRecord = {
  /**
   sha256 of the machine's token, lowercase hex.
   */
  readonly hash: string;
  /**
   Scope the row grants; {@link UPLOAD_SCOPE} is the only one that authorizes a write.
   */
  readonly scope: string;
  /**
   Epoch milliseconds after which the row is refused, absent when it never expires.
   */
  readonly expires?: number;
  /**
   Epoch milliseconds the row was revoked, absent while it stands.
   */
  readonly revoked?: number;
};

/**
 Answer of {@link findClient}.
 */
export type ClientLookup =
  | { readonly kind: 'absent'; }
    | {
      readonly kind: 'found';
      readonly client: ClientRecord
    }
    | {
      readonly kind: 'failed';
      readonly cause: unknown
    }
    | {
      readonly kind: 'malformed';
      readonly detail: string
    };

/**
 Answer of {@link anyClientRow}.
 */
export type ClientRowPresence =
  | {
    readonly cause: unknown;
    readonly kind: 'failed'
  }
    | { readonly kind: 'empty'; }
    | { readonly kind: 'occupied'; };

/**
 One timestamp column of a row, as the row answers it.
 */
type EpochColumn =
  | { readonly kind: 'absent'; }
    | { readonly kind: 'malformed'; }
    | {
      readonly at: number;
      readonly kind: 'value'
    };

/**
 A row read from the table, or the reason it cannot be used.
 */
type ParsedClientRow = {
  /**
   Fields the row holds.
   */
  readonly client: ClientRecord;
} | {
  /**
   Which column could not be read.
   */
  readonly detail: string;
};

/**
 Read one timestamp column, where SQL NULL and a missing column both mean absent.

 @param value - column value, as the database handed it back

 @returns the epoch milliseconds, `absent`, or `malformed`
 */
function parseEpochColumn(value: unknown,): EpochColumn {
  if ((value === null) || (value === undefined)) {
    return { kind: 'absent', };
  }
  if (((typeof value) !== 'number') || (!Number.isInteger(value,))) {
    return { kind: 'malformed', };
  }
  return {
    kind: 'value',
    at: value,
  };
}

/**
 Narrow one database row into a {@link ClientRecord}.

 Columns arrive untyped because D1 hands back whatever the table holds, so every
 column the row's meaning depends on is checked here instead of trusted; a row
 written by hand through `wrangler d1 execute` is exactly where a wrong type comes
 from.

 @param row - row as the database handed it back

 @returns the narrowed record, or which column could not be read
 */
function parseClientRow(row: DatabaseRow,): ParsedClientRow {
  /**
   Columns the row answers.
   */
  const {
    expires,
    hash,
    revoked,
    scope,
  } = row;
  if ((typeof hash) !== 'string') {
    return { detail: 'hash column is not text', };
  }
  if ((typeof scope) !== 'string') {
    return { detail: 'scope column is not text', };
  }
  /**
   Expiry column, where absent means the row never expires.
   */
  const expiry = parseEpochColumn(expires,);
  if (expiry.kind === 'malformed') {
    return { detail: 'expires column is not an integer or null', };
  }
  /**
   Revocation column, where absent means the row still stands.
   */
  const revocation = parseEpochColumn(revoked,);
  if (revocation.kind === 'malformed') {
    return { detail: 'revoked column is not an integer or null', };
  }
  return {
    client: {
      hash,
      scope,
      ...expiry.kind === 'value' ? { expires: expiry.at, } : {},
      ...revocation.kind === 'value' ? { revoked: revocation.at, } : {},
    },
  };
}

/**
 A query's rows, or the error that ended the query.
 */
type RowsOrFailure = {
  /**
   Error the query ended with.
   */
  readonly cause: unknown;
} | {
  /**
   Rows the query answered with.
   */
  readonly results: readonly DatabaseRow[];
};

/**
 Parameters for {@link queryRows}.
 */
type QueryRowsParams = {
  /**
   Database to query.
   */
  readonly database: D1DatabaseLike;
  /**
   SQL text of the statement.
   */
  readonly sql: string;
  /**
   Text to bind as the statement's one parameter; omit for a statement that takes none.
   */
  readonly value?: string;
};

/**
 Run one statement, turning a failing query into a result instead of a rejection.

 An authorization check that cannot read the table must refuse the write, so the
 caller needs the failure as a value it can branch on rather than an exception it
 must catch at every call site.

 @param database - database to query

 @param sql - SQL text of the statement

 @param value - text to bind as the statement's one parameter

 @returns the rows, or the error the query ended with
 */
async function queryRows({
  database,
  sql,
  value,
}: QueryRowsParams,): Promise<RowsOrFailure> {
  try {
    /**
     Statement with its parameter already bound, when the statement takes one.
     */
    const statement = value === undefined ? database.prepare(sql,) : database
      .prepare(sql,)
      .bind(value,);
    /**
     Answer to the query.
     */
    const answer = await statement.all();
    return { results: answer.results, };
  }
  catch (error) {
    return { cause: error, };
  }
}

/**
 Parameters for {@link findClient}.
 */
export type FindClientParams = {
  /**
   Worker env whose `INDEX` binding holds the table.
   */
  readonly env: WorkerEnv;
  /**
   Client id from the Basic user name.
   */
  readonly id: string;
};

/**
 Read the row of one client id.

 @param env - Worker env whose `INDEX` binding holds the table

 @param id - client id from the Basic user name

 @returns the row, `absent` when no row carries that id, `malformed` when a column cannot be read,
   or `failed` when the query itself threw

 @example
 ```ts
 const lookup = await findClient({ env, id: 'machine-one' });
 ```
 */
export async function findClient({
  env,
  id,
}: FindClientParams,): Promise<ClientLookup> {
  /**
   Database binding, absent in tests and when the binding is removed.
   */
  const { INDEX: database, } = env;
  if (database === undefined) {
    return { kind: 'absent', };
  }
  /**
   Answer to the one-row query.
   */
  const answer = await queryRows({
    database,
    sql: SELECT_CLIENT_SQL,
    value: id,
  },);
  if ('cause' in answer) {
    return {
      kind: 'failed',
      cause: answer.cause,
    };
  }
  /**
   Row of that id, when the table holds one.
   */
  const [row,] = answer.results;
  if (row === undefined) {
    return { kind: 'absent', };
  }
  /**
   Row with its columns narrowed, or the reason it cannot be used.
   */
  const parsed = parseClientRow(row,);
  if ('detail' in parsed) {
    return {
      kind: 'malformed',
      detail: parsed.detail,
    };
  }
  return {
    kind: 'found',
    client: parsed.client,
  };
}

/**
 Parameters for {@link anyClientRow}.
 */
export type AnyClientRowParams = {
  /**
   Worker env whose `INDEX` binding holds the table.
   */
  readonly env: WorkerEnv;
};

/**
 Whether the clients table holds any row, revoked ones included.

 This is the switch that retires the shared upload secret: once a coordinator has
 written the first per-machine row the shared secret stops being accepted, with no
 second deploy and no flag. A revoked row still counts, so cancelling every machine
 never reopens the shared secret.

 @param env - Worker env whose `INDEX` binding holds the table

 @returns `occupied` when the table holds at least one row, `empty` when it holds none,
   and `failed` when the query itself threw

 @example
 ```ts
 const presence = await anyClientRow({ env });
 if (presence.kind === 'occupied') {
   // the shared secret is retired
 }
 ```
 */
export async function anyClientRow({
  env,
}: AnyClientRowParams,): Promise<ClientRowPresence> {
  /**
   Database binding, absent in tests and when the binding is removed.
   */
  const { INDEX: database, } = env;
  if (database === undefined) {
    return { kind: 'empty', };
  }
  /**
   Answer to the existence query.
   */
  const answer = await queryRows({
    database,
    sql: SELECT_ANY_CLIENT_SQL,
  },);
  if ('cause' in answer) {
    return {
      kind: 'failed',
      cause: answer.cause,
    };
  }
  if (answer.results
    .length
    === 0) {
    return { kind: 'empty', };
  }
  return { kind: 'occupied', };
}
