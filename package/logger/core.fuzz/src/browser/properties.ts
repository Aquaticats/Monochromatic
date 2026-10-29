/**
 Browser property layer, bundled for the Playwright harness page. Each
 property feeds adversarial records through a browser-only backend of the
 neutral logger artifact and reparses what it persisted:

 - localStorage: run-scoped batches reparse to the exact records.
 - IndexedDB: batches in the `monochromatic.log` database reparse to the
   exact records, identified by a per-run nonce because the store outlives
   the run.
 - OPFS: every record is accepted and the flush settles with no breadcrumb.
   The sink keeps its writable open for the session and the platform shows
   a file's committed bytes only after close, so the content is not
   readable from the page; a backend without `createWritable` (WebKit)
   reports the property skipped.

 The bundle installs `globalThis.loggerFuzz`; the Playwright test calls its
 `run` and asserts on the returned report.

 @module
 */

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import {
  type LogRecord,
  sinks,
} from '@monochromatic-dev/logger-core';
import {
  createIndexedDbSink,
  createOpfsSink,
} from '@monochromatic-dev/logger-core/browser';
import {
  asyncProperty,
  check,
  defaultReportMessage,
} from 'fast-check';

import { logRecords, } from '../adversarial-message.ts';
import {
  canonical,
  captureBreadcrumbs,
  readIndexedDbRecords,
  readLocalStorageRecords,
} from './readback.ts';
import type {
  BrowserPropertyRunner,
  BrowserReport,
  BrowserRunOptions,
  PropertyResult,
} from './report.ts';

declare global {
  /**
   Runner the bundle installs on the page for the Playwright test to call.
   */
  var loggerFuzz: BrowserPropertyRunner;
}

//region Helpers

/**
 Canonical text of every record, in order.

 @param records - Records to render.

 @returns One canonical line per record.
 */
function canonicalLines(records: readonly LogRecord[],): readonly string[] {
  return records.map(function toLine(record,): string {
    return canonical(record,);
  },);
}

/**
 Thrown by a property body when the persisted records differ from the
 written ones.
 */
class RecordMismatchError extends Error {
  /**
   Builds the mismatch with both sides rendered canonically.

   @param expected - Records written.

   @param actual - Records read back.
   */
  constructor({
    expected,
    actual,
  }: {
    readonly expected: readonly LogRecord[];
    readonly actual: readonly LogRecord[];
  },) {
    super(`persisted records differ\nexpected: ${canonicalLines(expected,)
      .join('\n',)}\nactual: ${canonicalLines(actual,)
        .join('\n',)}`,);
    this.name = 'RecordMismatchError';
  }
}

/**
 Thrown by the OPFS property when a breadcrumb was written during the run.
 */
class BreadcrumbError extends Error {
  /**
   Builds the failure naming every breadcrumb.

   @param lines - Captured `console.warn` lines.
   */
  constructor({ lines, }: { readonly lines: readonly string[]; },) {
    super(`breadcrumbs written:\n${lines.join('\n',)}`,);
    this.name = 'BreadcrumbError';
  }
}

/**
 Throws unless both record lists match by value in order.

 @param expected - Records written.

 @param actual - Records read back.

 @throws RecordMismatchError on any difference.
 */
function expectSameRecords({
  expected,
  actual,
}: {
  readonly expected: readonly LogRecord[];
  readonly actual: readonly LogRecord[];
},): void {
  /**
   Canonical lines of each side.
   */
  const left = canonicalLines(expected,);
  /**
   Canonical lines of the other side.
   */
  const right = canonicalLines(actual,);
  if ((left.length !== right.length) || left.some(function differs(
    line,
    index,
  ): boolean {
    return line !== right[index];
  },))
    throw new RecordMismatchError({
      actual,
      expected,
    },);
}

/**
 Writes every record through a sink in one synchronous frame, then flushes
 when the sink has a hook.

 @param sink - Sink to write through.

 @param records - Records to write.
 */
async function writeAll({
  sink,
  records,
}: {
  readonly sink: {
    readonly write: (record: LogRecord,) => Promise<void>;
    readonly flush?: () => Promise<void>
  };
  readonly records: readonly LogRecord[];
},): Promise<void> {
  await Promise.all(records.map(function writeOne(record,): Promise<void> {
    return sink.write(record,);
  },),);
  if (sink.flush !== undefined)
    await sink.flush();
}

/**
 Runs one fast-check property and folds its outcome into a result.

 @param name - Result name.

 @param body - Property body over one record list.

 @param numRuns - Runs to attempt.

 @returns Passed or failed result with fast-check's report on failure.
 */
async function runProperty({
  name,
  body,
  numRuns,
}: {
  readonly name: string;
  readonly body: (records: readonly LogRecord[],) => Promise<void>;
  readonly numRuns: number;
},): Promise<PropertyResult> {
  /**
   fast-check's verdict.
   */
  const details = await check(
    asyncProperty(
      logRecords(),
      body,
    ),
    { numRuns, },
  );
  if (details.failed)
    return {
      detail: defaultReportMessage(details,) ?? 'fast-check reported a failure without a message',
      name,
      status: 'failed',
    };
  return {
    name,
    status: 'passed',
  };
}

//endregion Helpers

//region Properties

/**
 localStorage sink: after a clear, the run-scoped batches reparse to the
 exact records.

 @param records - Records to write.
 */
async function localStorageRoundTrips(records: readonly LogRecord[],): Promise<void> {
  localStorage.clear();
  /**
   Fresh sink over the page's localStorage.
   */
  const sink = sinks.createLocalStorageSink();
  if (!(await sink.verify()))
    throw new Error('localStorage sink declined at verify',);
  await writeAll({
    records,
    sink,
  },);
  expectSameRecords({
    actual: readLocalStorageRecords(),
    expected: records,
  },);
}

/**
 IndexedDB sink: the batches carrying this run's nonce reparse to the
 exact records.

 @param records - Records to write; every message is prefixed with the
 nonce so the run's lines are found among earlier runs' batches.
 */
async function indexedDbRoundTrips(records: readonly LogRecord[],): Promise<void> {
  /**
   Marker separating this run's records from every other run's in the
   shared store.
   */
  const nonce = `${crypto.randomUUID()}|`;
  /**
   Records as written, nonce first.
   */
  const marked = records.map(function mark(record,): LogRecord {
    return {
      ...record,
      message: `${nonce}${record.message}`,
    };
  },);
  /**
   Fresh sink over the page's IndexedDB.
   */
  const sink = createIndexedDbSink();
  if (!(await sink.verify()))
    throw new Error('IndexedDB sink declined at verify',);
  await writeAll({
    records: marked,
    sink,
  },);
  /**
   Persisted records belonging to this run.
   */
  const mine = (await readIndexedDbRecords())
    .filter(function ownRecord(record,): boolean {
      return record.message
        .startsWith(nonce,);
    },);
  expectSameRecords({
    actual: mine,
    expected: marked,
  },);
}

/**
 OPFS sink: every record is accepted and the flush settles with no
 breadcrumb.

 @param records - Records to write.
 */
async function opfsAcceptsEverything(records: readonly LogRecord[],): Promise<void> {
  /**
   Breadcrumbs recorded during the run.
   */
  using breadcrumbs = captureBreadcrumbs();
  /**
   Fresh sink over the page's origin-private file system.
   */
  const sink = createOpfsSink();
  if (!(await sink.verify()))
    throw new Error('OPFS sink declined at verify',);
  await writeAll({
    records,
    sink,
  },);
  if (breadcrumbs.lines
    .length
    > 0)
    throw new BreadcrumbError({ lines: breadcrumbs.lines, },);
}

/**
 Whether this page can open an OPFS writable, the capability the OPFS
 property needs; WebKit exposes the directory but not `createWritable`.

 @returns Skip reason, or an empty string when the property can run.
 */
async function opfsSkipReason(): Promise<string> {
  /**
   Storage manager, absent on pages without the Storage API.
   */
  const { storage, } = navigator;
  if ((typeof storage.getDirectory) !== 'function')
    return 'navigator.storage.getDirectory is unavailable';
  try {
    /**
     Probe handle used only to inspect the writable capability.
     */
    const handle = await (await storage.getDirectory())
      .getFileHandle(
      'monochromatic-fuzz-probe.jsonl',
      { create: true, },
      );
    return ('createWritable' in handle) ? '' : 'FileSystemFileHandle.createWritable is unavailable';
  }
  catch (error: unknown) {
    // Headless WebKit refuses the origin-private file system outright
    // (`UnknownError`); the property cannot run where the platform itself
    // declines, so the refusal becomes the skip reason.
    return `origin-private file system refused: ${caughtValueText(error,)}`;
  }
}

//endregion Properties

/**
 Runs every browser property in order and reports.

 @param numRuns - Runs per property.

 @returns Report for the Playwright test.

 @example
 ```ts
 const report = await run({ numRuns: 25 });
 ```
 */
async function run({ numRuns, }: BrowserRunOptions,): Promise<BrowserReport> {
  /**
   Why OPFS cannot run here, or empty.
   */
  const skip = await opfsSkipReason();
  /**
   Results in property order.
   */
  const results: readonly PropertyResult[] = [
    await runProperty({
      body: localStorageRoundTrips,
      name: 'localStorage sink batches reparse to the exact records',
      numRuns,
    },),
    await runProperty({
      body: indexedDbRoundTrips,
      name: 'IndexedDB sink batches reparse to the exact records',
      numRuns,
    },),
    skip === ''
      ? await runProperty({
        body: opfsAcceptsEverything,
        name: 'OPFS sink accepts every record with no breadcrumb',
        numRuns,
      },)
      : {
        detail: skip,
        name: 'OPFS sink accepts every record with no breadcrumb',
        status: 'skipped',
      },
  ];
  return {
    results,
    userAgent: navigator.userAgent,
  };
}

globalThis.loggerFuzz = { run, };
