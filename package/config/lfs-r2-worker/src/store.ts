/**
 Structural view of the R2 bucket binding the Worker depends on.

 Declared here instead of importing Cloudflare's runtime types so unit tests can
 substitute an in-memory store and the handlers stay typed against web
 standards only. Cloudflare's `R2Bucket` satisfies the shape unchanged.

 @module
 */

/**
 Metadata a head lookup returns: the byte length the object was stored with.
 */
export type StoredObjectHead = {
  /**
   Byte length of the stored object.
   */
  readonly size: number;
};

/**
 Stored object together with its streaming body.
 */
export type StoredObject = StoredObjectHead & {
  /**
   Object bytes as a readable stream.
   */
  readonly body: ReadableStream<Uint8Array>;
};

/**
 Minimal key-value object store the handlers need.

 The method signatures mirror `R2Bucket` so the binding is assignable without
 an adapter; they keep positional parameters for that reason instead of the
 single-object-parameter convention, and `head` and `get` mirror R2's `null`
 for an absent key because the binding cannot be reshaped before it reaches
 the Worker.
 */
export type ObjectStore = {
  /**
   Look up size metadata without reading the body; `null` when absent.
   */
  // oxlint-disable-next-line no-restricted-syntax/no-nullish-union -- mirrors Cloudflare R2Bucket.head, which resolves to null for an absent key and is bound by the platform, not constructed here
  readonly head: (key: string,) => Promise<StoredObjectHead | null>;
  /**
   Read the object and its body; `null` when absent.
   */
  // oxlint-disable-next-line no-restricted-syntax/no-nullish-union -- mirrors Cloudflare R2Bucket.get, which resolves to null for an absent key and is bound by the platform, not constructed here
  readonly get: (key: string,) => Promise<StoredObject | null>;
  /**
   Store `body` under `key`, replacing any previous object.
   */
  readonly put: (
    key: string,
    body: ReadableStream<Uint8Array>,
  ) => Promise<unknown>;
};

/**
 One row of a query result.

 Columns arrive untyped because that is what D1 hands back for a table whose schema
 lives in SQL, and each caller narrows the columns it reads.
 */
export type DatabaseRow = Readonly<Record<string, unknown>>;

/**
 One prepared statement of the D1 database the Worker binds.

 Mirrors the slice of Cloudflare's `D1PreparedStatement` this package uses, so the
 D1 binding is assignable without an adapter and unit tests can substitute an
 in-memory database.
 */
export type D1StatementLike = {
  /**
   Attach the statement's one text parameter: a JSON array of oids, or a client id.
   */
  readonly bind: (value: string,) => D1StatementLike;
  /**
   Run a query and collect its rows.
   */
  readonly all: () => Promise<{ readonly results: readonly DatabaseRow[]; }>;
  /**
   Run a write.
   */
  readonly run: () => Promise<unknown>;
};

/**
 The D1 database the Worker binds: Cloudflare D1 in production.

 One database holds both tables, the presence index and the per-machine client
 rows, because a Worker binds it already and a second database would need its own
 provisioning for no benefit at this size.
 */
export type D1DatabaseLike = {
  /**
   Compile one SQL statement.
   */
  readonly prepare: (sql: string,) => D1StatementLike;
};

/**
 Bindings and secrets wrangler hands the Worker on every request.
 */
export type WorkerEnv = {
  /**
   R2 bucket binding declared in `wrangler.toml`.
   */
  readonly BUCKET: ObjectStore;
  /**
   D1 database declared in `wrangler.toml`; absent in tests and when the binding
   is removed. It holds the presence index, which lets a batch learn which oids
   exist in one query, and the `clients` rows that authorize uploads.
   */
  readonly INDEX?: D1DatabaseLike;
  /**
   Shared upload secret set with `wrangler secret put`; accepted only while the
   clients table holds no row, and every write is refused while it is absent.
   */
  readonly LFS_WRITE_TOKEN?: string;
};

/**
 Structural view of the execution context wrangler passes as the third
 handler argument; only `waitUntil` is needed, to keep the log flush alive
 after the response is returned.
 */
export type ExecutionContextLike = {
  /**
   Extend the request lifetime until `promise` settles.
   */
  readonly waitUntil: (promise: Promise<unknown>,) => void;
};
