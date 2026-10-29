/**
 Coverage-driver scenarios over the web storage sinks and their shared
 parts: the sessionStorage and localStorage sinks over Node's real
 backends (localStorage only when the process runs with
 `--localstorage-file`), quota overflow, mid-session failure, and refused
 access over stand-ins, and the record buffer's every flush trigger including
 the page-lifecycle hooks. The shared key, quota, and store helpers live
 in `coverage-keys-quota.ts`.

 @module
 */

import { wait, } from '@monochromatic-dev/async-time-core/ts';
import {
  _buildLogKey as buildLogKey,
  _createRecordBuffer as createRecordBuffer,
  type LogRecord,
  type Sink,
  sinks,
} from '@monochromatic-dev/logger-core/ts';

import {
  BOUNDARY_TOKENS,
  LEVELS,
} from './boundary-corpus.ts';
import {
  createFlakyStorage,
  createQuotaStorage,
  createThrowingStorage,
  installFakeLocalStorage,
  installFakePage,
  installFakeSessionStorage,
} from './coverage-storage-fakes.ts';

//region Fixtures

/**
 Message length past the buffer's 32 KiB cap, so one record flushes alone.
 */
const OVERSIZE_CHARS = 40_000;

/**
 Message length for the footprint burst that crosses half the runtime
 quota (2.5 MiB of code units on Node) and starts proactive eviction.
 */
const BURST_CHARS = 30_000;

/**
 Records in the footprint burst; together past the half-quota cap.
 */
const BURST_RECORDS = 100;

/**
 Quiet period after which the buffer's deadline timer flushes (250 ms),
 plus margin.
 */
const DEADLINE_WAIT_MS = 320;

/**
 Quota budget small enough that a few records overflow a stand-in.
 */
const SMALL_QUOTA_CHARS = 2_000;

/**
 Records written against the small quota.
 */
const SMALL_QUOTA_RECORDS = 6;

/**
 Message length that overflows the small quota within a few records.
 */
const SMALL_QUOTA_MESSAGE_CHARS = 600;

/**
 Node flag that enables a real localStorage.
 */
const LOCALSTORAGE_FLAG = '--localstorage-file';

/**
 Reports whether one Node argument is the localStorage flag.

 @param argument - Execution argument.

 @returns Whether it starts the flag.
 */
function startsWithLocalStorageFlag(argument: string,): boolean {
  return argument.startsWith(LOCALSTORAGE_FLAG,);
}

/**
 Whether this process runs with a real localStorage, the only case in
 which the localStorage sink probes the global.

 @returns Whether `--localstorage-file` was passed.
 */
function hasLocalStorageFile(): boolean {
  return process.execArgv
    .some(startsWithLocalStorageFlag,);
}

/**
 Builds one record at a level.

 @param level - Severity.

 @param message - Message body.

 @returns Record at a fixed timestamp.
 */
function record({
  level,
  message,
}: {
  readonly level: LogRecord['level'];
  readonly message: string;
},): LogRecord {
  return {
    level,
    message,
    timestamp: 0,
  };
}

/**
 One record per boundary token, cycling through every level.

 @returns Records in corpus order.
 */
function corpusRecords(): readonly LogRecord[] {
  return BOUNDARY_TOKENS.map(function toRecord(
    message,
    index,
  ): LogRecord {
    return {
      level: LEVELS[index % LEVELS.length] ?? 'info',
      message,
      timestamp: index,
    };
  },);
}

/**
 One large record of the footprint burst, flushed alone by the cap.

 @param index - Position in the burst.

 @returns Record carrying `BURST_CHARS` of filler.
 */
function burstRecord(index: number,): LogRecord {
  return record({
    level: 'info',
    message: `${String(index,)}:${'y'.repeat(BURST_CHARS,)}`,
  },);
}

/**
 One warn record sized to overflow the small quota within a few writes.

 @param index - Position in the sequence.

 @returns Record carrying `SMALL_QUOTA_MESSAGE_CHARS` of filler.
 */
function smallQuotaRecord(index: number,): LogRecord {
  return record({
    level: 'warn',
    message: `${String(index,)}:${'q'.repeat(SMALL_QUOTA_MESSAGE_CHARS,)}`,
  },);
}

/**
 Builds `count` records through `build`, indexed from zero.

 @param count - Records to build.

 @param build - Record builder by index.

 @returns Records in index order.
 */
function indexedRecords({
  count,
  build,
}: {
  readonly count: number;
  readonly build: (index: number,) => LogRecord;
},): readonly LogRecord[] {
  /**
   Records built so far.
   */
  const records: LogRecord[] = [];
  for (let index = 0; index < count; index += 1)
    records.push(build(index,),);
  return records;
}

/**
 Writes every record through a sink in one synchronous frame, then
 flushes when the sink has a hook.

 @param sink - Sink to write through.

 @param records - Records to write.
 */
async function writeAll({
  sink,
  records,
}: {
  readonly sink: Sink;
  readonly records: readonly LogRecord[];
},): Promise<void> {
  await Promise.all(records.map(function writeOne(entry,): Promise<void> {
    return sink.write(entry,);
  },),);
  if (sink.flush !== undefined)
    await sink.flush();
}

/**
 Verifies a buffered sink, writes the corpus (severity flushes for warn and
 worse), an oversize record after a small one (cap flush with isolation), a
 routine record left to the deadline timer, then a burst past half the
 runtime quota so proactive eviction runs.

 @param sink - Buffered sink to walk.
 */
async function walkBufferedSink({ sink, }: { readonly sink: Sink; },): Promise<void> {
  await sink.write(record({
    level: 'info',
    message: 'before verify',
  },),);
  await sink.verify();
  await writeAll({
    records: corpusRecords(),
    sink,
  },);
  await sink.write(record({
    level: 'info',
    message: 'small',
  },),);
  await sink.write(record({
    level: 'debug',
    message: 'x'.repeat(OVERSIZE_CHARS,),
  },),);
  await sink.write(record({
    level: 'trace',
    message: 'left to the timer',
  },),);
  await wait(DEADLINE_WAIT_MS,);
  await writeAll({
    records: [
      record({
        level: 'info',
        message: 'drained by flush',
      },),
    ],
    sink,
  },);
  await writeAll({
    records: indexedRecords({
      build: burstRecord,
      count: BURST_RECORDS,
    },),
    sink,
  },);
}

/**
 Walks a sink over a quota-limited, a flaky, and a refusing stand-in.

 @param createSink - Factory for the sink under test.

 @param install - Installer for the matching global.
 */
async function walkStandIns({
  createSink,
  install,
}: {
  readonly createSink: () => Sink;
  readonly install: (options: { readonly fake: Storage; },) => Disposable;
},): Promise<void> {
  {
    /**
     Overflowing store installed for the scope.
     */
    using _quota = install({ fake: createQuotaStorage({ budget: SMALL_QUOTA_CHARS, },), },);
    /**
     Sink over the overflowing store.
     */
    const sink = createSink();
    await sink.verify();
    await writeAll({
      records: indexedRecords({
        build: smallQuotaRecord,
        count: SMALL_QUOTA_RECORDS,
      },),
      sink,
    },);
  }
  {
    /**
     Store failing after its first write, installed for the scope.
     */
    using _flaky = install({ fake: createFlakyStorage(), },);
    /**
     Sink over the flaky store.
     */
    const sink = createSink();
    await sink.verify();
    await sink.write(record({
      level: 'warn',
      message: 'lands',
    },),);
    await sink.write(record({
      level: 'warn',
      message: 'fails and reports',
    },),);
    await sink.write(record({
      level: 'warn',
      message: 'fails silently',
    },),);
  }
  {
    /**
     Refusing store installed for the scope.
     */
    using _refused = install({ fake: createThrowingStorage(), },);
    await createSink()
      .verify();
  }
}

//endregion Fixtures

//region Scenarios

/**
 The sessionStorage sink over Node's real backend, then over stand-ins.
 */
async function exerciseSessionStorage(): Promise<void> {
  globalThis.sessionStorage
    .clear();
  await walkBufferedSink({ sink: sinks.createSessionStorageSink(), },);
  await walkStandIns({
    createSink: sinks.createSessionStorageSink,
    install: installFakeSessionStorage,
  },);
}

/**
 The localStorage sink: the flagless short-circuit when the process has no
 `--localstorage-file`; otherwise the real backend seeded with a prior
 runs' entries (two, so the oldest-first sort compares), a foreign flat key, and a host key, then stand-ins.
 */
async function exerciseLocalStorage(): Promise<void> {
  if (!hasLocalStorageFile()) {
    await sinks.createLocalStorageSink()
      .verify();
    return;
  }
  globalThis.localStorage
    .clear();
  globalThis.localStorage
    .setItem(
    buildLogKey({
      index: 0,
      nonce: 'seed',
      stamp: 1,
    },),
    '{"level":"info","message":"prior run","timestamp":0}',
    );
  globalThis.localStorage
    .setItem(
    buildLogKey({
      index: 0,
      nonce: 'later',
      stamp: 2,
    },),
    '{"level":"info","message":"second prior run","timestamp":0}',
    );
  globalThis.localStorage
    .setItem(
    'monochromatic.log.7',
    'foreign flat key',
    );
  globalThis.localStorage
    .setItem(
    'host.setting',
    'left alone',
    );
  await walkBufferedSink({ sink: sinks.createLocalStorageSink(), },);
  await walkStandIns({
    createSink: sinks.createLocalStorageSink,
    install: installFakeLocalStorage,
  },);
}

/**
 The record buffer through every trigger: an empty drain, a routine
 record, a severity flush, a cap flush isolating an oversize record, the
 deadline timer, the page hiding, the page leaving, and an explicit drain.
 */
async function exerciseRecordBuffer(): Promise<void> {
  /**
   Page stand-in so the buffer registers its lifecycle hooks.
   */
  using page = installFakePage();
  /**
   Batches the buffer handed over.
   */
  const batches: string[] = [];
  /**
   Buffer under test.
   */
  const buffer = createRecordBuffer({
    onFlush: function keep(batch,): void {
      batches.push(batch,);
    },
  },);
  buffer.drain();
  buffer.add({
    level: 'info',
    serialized: '{"a":1}',
  },);
  buffer.add({
    level: 'error',
    serialized: '{"b":2}',
  },);
  buffer.add({
    level: 'info',
    serialized: '{"c":3}',
  },);
  buffer.add({
    level: 'debug',
    serialized: 'x'.repeat(OVERSIZE_CHARS,),
  },);
  buffer.add({
    level: 'trace',
    serialized: '{"d":4}',
  },);
  await wait(DEADLINE_WAIT_MS,);
  buffer.add({
    level: 'info',
    serialized: '{"e":5}',
  },);
  page.fireVisibility({ state: 'visible', },);
  page.fireVisibility({ state: 'hidden', },);
  buffer.add({
    level: 'info',
    serialized: '{"f":6}',
  },);
  page.firePageHide();
  buffer.add({
    level: 'info',
    serialized: '{"g":7}',
  },);
  buffer.drain();
}

//endregion Scenarios

/**
 Runs every web storage scenario in order.

 @example
 ```ts
 await exerciseWebStorageSinks();
 ```
 */
export async function exerciseWebStorageSinks(): Promise<void> {
  await exerciseSessionStorage();
  await exerciseLocalStorage();
  await exerciseRecordBuffer();
}
