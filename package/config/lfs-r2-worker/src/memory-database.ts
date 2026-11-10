/**
 In-memory D1 database for unit tests: the presence index and the `clients` table
 in one object, because a Worker binds one database.

 Exported from the package entry as `\@internal` so tests import it through the
 built artifact. Every other statement is refused, so a test cannot pass against a
 query the production database would not answer.

 @module
 */

import {
  SELECT_ANY_CLIENT_SQL,
  SELECT_CLIENT_SQL,
  UPLOAD_SCOPE,
} from './clients.ts';
import {
  INSERT_PRESENT_SQL,
  SELECT_PRESENT_SQL,
} from './presence-index.ts';
import type {
  D1DatabaseLike,
  D1StatementLike,
  DatabaseRow,
} from './store.ts';

/**
 One client row as a test seeds it; the defaults are the ones a valid row has.
 */
export type MemoryClientFixture = {
  /**
   Client id, the Basic user name.
   */
  readonly id: string;
  /**
   sha256 of the machine's token, lowercase hex.
   */
  readonly hash: string;
  /**
   Scope the row grants; defaults to {@link UPLOAD_SCOPE}.
   */
  readonly scope?: string;
  /**
   Epoch milliseconds after which the row is refused; absent for no expiry.
   */
  readonly expires?: number;
  /**
   Epoch milliseconds the row was revoked; absent while it stands.
   */
  readonly revoked?: number;
};

/**
 A stored client row, in the shape the table answers.
 */
export type MemoryClientRow = {
  /**
   sha256 of the machine's token, lowercase hex.
   */
  readonly hash: string;
  /**
   Scope the row grants.
   */
  readonly scope: string;
  /**
   Epoch milliseconds after which the row is refused; absent for no expiry.
   */
  readonly expires?: number;
  /**
   Epoch milliseconds the row was revoked; absent while it stands.
   */
  readonly revoked?: number;
};

/**
 In-memory database plus a view of its contents and traffic for assertions.
 */
export type MemoryDatabase = D1DatabaseLike & {
  /**
   Oids the presence index holds.
   */
  readonly oids: Set<string>;
  /**
   Client rows, keyed by client id.
   */
  readonly clients: Map<string, MemoryClientRow>;
  /**
   SQL text of every prepared statement, in call order.
   */
  readonly statements: string[];
};

/**
 Parameters for {@link createMemoryDatabase}.
 */
export type CreateMemoryDatabaseParams = {
  /**
   Oids the presence index holds before the test starts.
   */
  readonly oids?: readonly string[];
  /**
   Client rows the table holds before the test starts.
   */
  readonly clients?: readonly MemoryClientFixture[];
  /**
   Error every presence-index statement throws, to exercise the fallback to the bucket.
   */
  readonly presenceFailure?: Error;
  /**
   Error every client statement throws, to exercise a write refused because the
   authorization lookup could not answer.
   */
  readonly clientFailure?: Error;
};

/**
 Parameters for {@link buildStatement}.
 */
type StatementParams = {
  /**
   SQL text of the statement.
   */
  readonly sql: string;
  /**
   Oids the presence index holds.
   */
  readonly oids: Set<string>;
  /**
   Client rows the table holds.
   */
  readonly clients: Map<string, MemoryClientRow>;
  /**
   Error every presence-index statement throws.
   */
  readonly presenceFailure?: Error;
  /**
   Error every client statement throws.
   */
  readonly clientFailure?: Error;
  /**
   Text the statement was bound to; absent until `bind` is called.
   */
  readonly value?: string;
};

/**
 Both operations of one statement, already tied to its table.
 */
type Operations = {
  /**
   Query operation.
   */
  readonly all: () => Promise<{ readonly results: readonly DatabaseRow[]; }>;
  /**
   Write operation.
   */
  readonly run: () => Promise<unknown>;
};

/**
 Build an operation that answers with rows, or rejects with the seeded failure.

 @param rows - rows this statement answers with

 @param failure - error the statement throws instead of answering

 @returns the query operation
 */
function answering({
  rows,
  failure,
}: {
  readonly rows: () => readonly DatabaseRow[];
  readonly failure?: Error
},): Operations['all'] {
  return function answer() {
    if (failure !== undefined) {
      return Promise.reject(failure,);
    }
    return Promise.resolve({ results: rows(), },);
  };
}

/**
 Build an operation that rejects, because this statement has nothing to answer there.

 @param reason - what the statement cannot do

 @returns the rejecting operation
 */
function unsupported(reason: string,): () => Promise<never> {
  return function refuse() {
    return Promise.reject(new Error(reason,),);
  };
}

/**
 Build a write that succeeds, or rejects with the seeded failure.

 @param write - work the write does when it succeeds

 @param failure - error the write throws instead

 @returns the write operation
 */
function writing({
  write,
  failure,
}: {
  readonly write: () => void;
  readonly failure?: Error
},): Operations['run'] {
  return function run() {
    if (failure !== undefined) {
      return Promise.reject(failure,);
    }
    write();
    return Promise.resolve(undefined,);
  };
}

/**
 Parse the JSON array of oids a statement was bound to.

 @param oidsJson - JSON text of an array of strings

 @returns the strings

 @throws when the text is not a JSON array of strings
 */
function parseOidArray(oidsJson: string,): readonly string[] {
  /**
   Parsed value, still unchecked.
   */
  const parsed: unknown = JSON.parse(oidsJson,);
  if (!Array.isArray(parsed,)) {
    throw new TypeError('bound value must be a JSON array',);
  }
  return parsed.map(function requireString(value: unknown,): string {
    if ((typeof value) !== 'string') {
      throw new TypeError('bound array must hold only strings',);
    }
    return value;
  },);
}

/**
 Seed the client rows of a fresh in-memory table.

 @param fixtures - rows a test seeds

 @returns stored rows keyed by client id
 */
function seedClients(fixtures: readonly MemoryClientFixture[],): Map<string, MemoryClientRow> {
  return new Map(fixtures.map(function toEntry(fixture: MemoryClientFixture,) {
    return [
      fixture.id,
      {
        hash: fixture.hash,
        scope: fixture.scope ?? UPLOAD_SCOPE,
        ...fixture.expires === undefined ? {} : { expires: fixture.expires, },
        ...fixture.revoked === undefined ? {} : { revoked: fixture.revoked, },
      },
    ] as const;
  },),);
}

/**
 Choose the two operations of one statement.

 @param sql - SQL text of the statement

 @param value - text the statement was bound to

 @param oids - oids the presence index holds

 @param clients - client rows the table holds

 @param presenceFailure - error every presence-index statement throws

 @param clientFailure - error every client statement throws

 @returns the query and write operations of that statement

 @throws Error when the statement is not one this database answers
 */
function operationsFor({
  sql,
  value,
  oids,
  clients,
  presenceFailure,
  clientFailure,
}: StatementParams,): Operations {
  if (sql === SELECT_PRESENT_SQL) {
    return {
      all: answering({
        rows: function heldOids() {
          return parseOidArray(value ?? '',)
            .filter(function isHeld(oid: string,): boolean {
              return oids.has(oid,);
            },)
            .map(function toRow(oid: string,): DatabaseRow {
              return { oid, };
            },);
        },
        ...presenceFailure === undefined ? {} : { failure: presenceFailure, },
      },),
      run: unsupported('this statement is a query',),
    };
  }
  if (sql === INSERT_PRESENT_SQL) {
    return {
      all: unsupported('this statement is a write',),
      run: writing({
        write: function recordOids() {
          for (const oid of parseOidArray(value ?? '',)) {
            oids.add(oid,);
          }
        },
        ...presenceFailure === undefined ? {} : { failure: presenceFailure, },
      },),
    };
  }
  if (sql === SELECT_CLIENT_SQL) {
    return {
      all: answering({
        rows: function clientRow() {
          /**
           Row of the bound client id, when the table holds one.
           */
          const row = clients.get(value ?? '',);
          return row === undefined ? [] : [row,];
        },
        ...clientFailure === undefined ? {} : { failure: clientFailure, },
      },),
      run: unsupported('this statement is a query',),
    };
  }
  if (sql === SELECT_ANY_CLIENT_SQL) {
    return {
      all: answering({
        rows: function firstClientId() {
          /**
           First client id of the table, when it holds one.
           */
          const [id,] = clients.keys();
          return id === undefined ? [] : [{ id, },];
        },
        ...clientFailure === undefined ? {} : { failure: clientFailure, },
      },),
      run: unsupported('this statement is a query',),
    };
  }
  throw new Error(`unexpected statement: ${sql}`,);
}

/**
 Build one statement of the in-memory database.

 `bind` returns a new statement carrying the value instead of mutating this one, so
 the double keeps the shape production code chains through and no mutable binding
 has to live at a function root.

 @param sql - SQL text of the statement

 @param oids - oids the presence index holds

 @param clients - client rows the table holds

 @param presenceFailure - error every presence-index statement throws

 @param clientFailure - error every client statement throws

 @param value - text the statement was bound to

 @returns statement whose `bind` attaches the value it is given

 @throws Error when the statement is not one this database answers
 */
function buildStatement({
  sql,
  oids,
  clients,
  presenceFailure,
  clientFailure,
  value,
}: StatementParams,): D1StatementLike {
  /**
   Operations of this statement, tied to its table.
   */
  const operations = operationsFor({
    sql,
    oids,
    clients,
    ...value === undefined ? {} : { value, },
    ...presenceFailure === undefined ? {} : { presenceFailure, },
    ...clientFailure === undefined ? {} : { clientFailure, },
  },);
  return {
    bind: function bind(bound: string,): D1StatementLike {
      return buildStatement({
        sql,
        oids,
        clients,
        value: bound,
        ...presenceFailure === undefined ? {} : { presenceFailure, },
        ...clientFailure === undefined ? {} : { clientFailure, },
      },);
    },
    all: operations.all,
    run: operations.run,
  };
}

/**
 Build an in-memory database that understands the presence-index statements and the
 client statements.

 @param oids - oids held before the test starts

 @param clients - client rows held before the test starts

 @param presenceFailure - error every presence-index statement throws

 @param clientFailure - error every client statement throws

 @returns database whose contents reflect every recorded write

 @throws when a statement other than a known one is prepared

 @example
 ```ts
 const database = createMemoryDatabase({ oids: [oid], clients: [{ id, hash }] });
 ```
 */
export function createMemoryDatabase({
  oids: initialOids = [],
  clients: initialClients = [],
  presenceFailure,
  clientFailure,
}: CreateMemoryDatabaseParams = {},): MemoryDatabase {
  /**
   Oids held, seeded from the fixtures.
   */
  const oids = new Set<string>(initialOids,);
  /**
   Client rows held, seeded from the fixtures.
   */
  const clients = seedClients(initialClients,);
  /**
   Statements prepared so far.
   */
  const statements: string[] = [];
  return {
    oids,
    clients,
    statements,
    prepare(sql: string,): D1StatementLike {
      statements.push(sql,);
      return buildStatement({
        sql,
        oids,
        clients,
        ...presenceFailure === undefined ? {} : { presenceFailure, },
        ...clientFailure === undefined ? {} : { clientFailure, },
      },);
    },
  };
}
