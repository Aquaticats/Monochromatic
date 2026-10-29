/**
 Readers that reparse what the browser sinks persisted, plus record
 normalization and breadcrumb capture, for the browser property bundle.
 Browser APIs only: this module is bundled into a page.

 @module
 */

import {
  _compareLogKeys as compareLogKeys,
  _parseLogKey as parseLogKey,
  type LogRecord,
} from '@monochromatic-dev/logger-core';

//region Records

/**
 Canonical text of one record, so two record lists compare by value
 whatever object shape they arrived in.

 @param record - Parsed or original record.

 @returns Stable JSON of level, message, and timestamp.

 @example
 ```ts
 canonical({ level: 'info', message: 'a', timestamp: 0 });
 ```
 */
export function canonical(record: LogRecord,): string {
  return JSON.stringify([
    record.level,
    record.message,
    record.timestamp,
  ],);
}

/**
 Whether a parsed JSONL line has the record shape.

 @param value - Parsed line.

 @returns Whether it carries a level, a message, and a timestamp.

 @example
 ```ts
 isRecord(JSON.parse(line));
 ```
 */
export function isRecord(value: unknown,): value is LogRecord {
  return ((typeof value) === 'object')
    && (value !== null)
    && ('level' in value)
    && ('message' in value)
    && ('timestamp' in value);
}

/**
 Parses newline-separated JSONL into records, dropping anything that is not
 a record.

 @param text - Batch text.

 @returns Records in line order.

 @example
 ```ts
 recordsInBatch({ text: '{"level":"info","message":"a","timestamp":0}' });
 ```
 */
export function recordsInBatch({ text, }: { readonly text: string; },): readonly LogRecord[] {
  return text.split('\n',)
    .filter(function nonEmpty(line,): boolean {
      return line.length > 0;
    },)
    .map(function parseLine(line,): unknown {
      return JSON.parse(line,);
    },)
    .filter(isRecord,);
}

//endregion Records

//region localStorage

/**
 Key prefix shared by both web storage sinks.
 */
const KEY_PREFIX = 'monochromatic.log.';

/**
 One run-scoped batch found in localStorage.
 */
type StoredBatch = {
  readonly parsed: NonNullable<ReturnType<typeof parseLogKey>['parsed']>;
  readonly text: string;
};

/**
 Reads every run-scoped batch the localStorage sink persisted, oldest run
 and lowest index first, and reparses the records.

 @returns Records in persistence order.

 @example
 ```ts
 readLocalStorageRecords();
 ```
 */
export function readLocalStorageRecords(): readonly LogRecord[] {
  /**
   Batches keyed by their parsed identity.
   */
  const batches: StoredBatch[] = [];
  for (let slot = 0; slot < localStorage.length; slot += 1) {
    /**
     Key at this slot.
     */
    const key = localStorage.key(slot,);
    if ((key === null) || (!key.startsWith(KEY_PREFIX,)))
      continue;
    /**
     Run identity, absent for keys the sink does not own.
     */
    const { parsed, } = parseLogKey(key,);
    /**
     Batch text under the key.
     */
    const text = localStorage.getItem(key,);
    if ((parsed === undefined) || (text === null))
      continue;
    batches.push({
      parsed,
      text,
    },);
  }
  return batches
    .toSorted(function oldestFirst(
      left,
      right,
    ): number {
      return compareLogKeys({
        first: left.parsed,
        second: right.parsed,
      },);
    },)
    .flatMap(function parseBatch(batch,): readonly LogRecord[] {
      return recordsInBatch({ text: batch.text, },);
    },);
}

//endregion localStorage

//region IndexedDB

/**
 Database the IndexedDB sink opens.
 */
const DATABASE_NAME = 'monochromatic.log';

/**
 Schema version the IndexedDB sink opens.
 */
const DATABASE_VERSION = 1;

/**
 Object store holding one JSONL batch per entry.
 */
const BATCH_STORE = 'batch';

/**
 Settles a request's promise from its `success` and `error` events.

 @param request - IndexedDB request to await.

 @returns Request result.

 @example
 ```ts
 await settle({ request: store.getAll() });
 ```
 */
async function settle<Result,>({ request, }: { readonly request: IDBRequest<Result>; },): Promise<Result> {
  /**
   Resolvers the event listeners settle.
   */
  const pending = Promise.withResolvers<Result>();
  request.addEventListener(
    'success',
    function onSuccess(): void {
      pending.resolve(request.result,);
    },
  );
  request.addEventListener(
    'error',
    function onError(): void {
      pending.reject(request.error ?? new Error('IndexedDB request failed',),);
    },
  );
  return await pending.promise;
}

/**
 Reads every batch the IndexedDB sink persisted, in key order, and
 reparses the records.

 @returns Records in persistence order.

 @example
 ```ts
 await readIndexedDbRecords();
 ```
 */
export async function readIndexedDbRecords(): Promise<readonly LogRecord[]> {
  /**
   Open connection to the sink's database.
   */
  const database = await settle({
    request: indexedDB.open(
      DATABASE_NAME,
      DATABASE_VERSION,
    ),
  },);
  /**
   Every stored batch text, in key order.
   */
  const batches = await settle({
    request: database
      .transaction(
      BATCH_STORE,
      'readonly',
      )
      .objectStore(BATCH_STORE,)
      .getAll(),
  },);
  database.close();
  return batches.flatMap(function parseBatch(batch: unknown,): readonly LogRecord[] {
    return ((typeof batch) === 'string') ? recordsInBatch({ text: batch, },) : [];
  },);
}

//endregion IndexedDB

//region Breadcrumbs

/**
 Captured `console.warn` lines; disposing restores the method.
 */
export type BreadcrumbCapture = Disposable & {
  readonly lines: readonly string[];
};

/**
 Replaces `console.warn` with a recorder for a scope, so a property can
 assert that a sink wrote no breadcrumb.

 @returns Capture exposing the recorded lines.

 @example
 ```ts
 using breadcrumbs = captureBreadcrumbs();
 ```
 */
export function captureBreadcrumbs(): BreadcrumbCapture {
  /**
   Original method to restore.
   */
  const original = console.warn;
  /**
   Recorded first arguments.
   */
  const lines: string[] = [];
  console.warn = function record(first: unknown,): void {
    lines.push(String(first,),);
  };
  return {
    lines,
    [Symbol.dispose](): void {
      console.warn = original;
    },
  };
}

//endregion Breadcrumbs
