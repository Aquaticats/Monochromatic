/**
 Helpers behind the sink boundary properties: a throwaway package directory
 that points the file sink at a fresh `node_modules`, readers that reparse
 what the JSONL sinks persisted, an independent reference for the console
 sink's output, and run tracking so a campaign interrupted mid-run still
 restores the working directory before the next property starts.

 @module
 */

import {
  mkdir,
  mkdtemp,
  readdir,
  readFile,
  rm,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';
import { setImmediate, } from 'node:timers/promises';

import type {
  Level,
  LogRecord,
} from '@monochromatic-dev/logger-core';

//region Run tracking

/**
 Lets the event loop deliver I/O callbacks and timers before the next run.
 The boundary bodies settle through microtasks alone, so a campaign would
 otherwise hold the loop for its whole budget; the test harness's own
 logger, still verifying its file sink when the first property starts,
 then saw its verify timer fire before the filesystem answered and wrote a
 breadcrumb. One `setImmediate` per run costs microseconds and keeps every
 pending callback in the process serviced.

 @example
 ```ts
 await yieldToEventLoop();
 ```
 */
export async function yieldToEventLoop(): Promise<void> {
  await setImmediate();
}

/**
 Runs still executing. A fast-check campaign interrupted by its time limit
 abandons the run in flight; the file property changes the working
 directory per run, so the property awaits {@link settleTrackedRuns} after
 `assert` to be sure the abandoned run has restored it.
 */
const inFlight = new Set<Promise<void>>();

/**
 Waits for every tracked run to finish.

 @example
 ```ts
 await assert(asyncProperty(...), run.params);
 await settleTrackedRuns();
 ```
 */
export async function settleTrackedRuns(): Promise<void> {
  await Promise.allSettled(inFlight,);
}

/**
 Registers one run until it settles, however it settles.

 @param run - Property body in flight.

 @example
 ```ts
 asyncProperty(logRecords(), function body(records) {
   return trackRun(fileRoundTrip(records));
 });
 ```
 */
export async function trackRun(run: Promise<void>,): Promise<void> {
  inFlight.add(run,);
  /**
   Removes the run from the in-flight set once it settles, however it settles.
   */
  using _untrack = {
    [Symbol.dispose](): void {
      inFlight.delete(run,);
    },
  };
  await run;
}

//endregion Run tracking

//region Throwaway package

/**
 Directory name the file sink creates under the nearest `node_modules`.
 */
const LOG_DIR_NAME = '.monochromatic';

/**
 Suffix of the JSONL log files the sink creates, one per process under a UTC
 day directory.
 */
const LOG_FILE_SUFFIX = '.log.jsonl';

/**
 Throwaway package directory the process has entered; disposing leaves it
 and removes it.
 */
export type ThrowawayPackage = AsyncDisposable & {
  readonly dir: string;
};

/**
 Creates a fresh temporary directory holding an empty `node_modules`, makes
 it the working directory, and returns a disposable that restores the
 previous working directory and removes the tree. The file sink resolves
 its log path from `process.cwd()`, so each run gets its own log directory
 and never shares a file with another run in the same millisecond.

 @returns Entered package.

 @example
 ```ts
 await using pkg = await enterThrowawayPackage();
 ```
 */
export async function enterThrowawayPackage(): Promise<ThrowawayPackage> {
  /**
   Working directory to restore on dispose.
   */
  const original = process.cwd();
  /**
   Log-directory override to restore on dispose. It outranks the `process.cwd()`
   discovery this fixture relies on, so entering removes it for the duration.
   */
  const originalLogDirectory = process.env.MONOCHROMATIC_LOG_DIRECTORY;
  delete process.env.MONOCHROMATIC_LOG_DIRECTORY;
  /**
   Fresh temporary package root.
   */
  const dir = await mkdtemp(join(
    scratchRoot(),
    'logger-fuzz-file-',
  ),);
  await mkdir(join(
    dir,
    'node_modules',
  ),);
  process.chdir(dir,);
  return {
    dir,
    async [Symbol.asyncDispose](): Promise<void> {
      process.chdir(original,);
      if (originalLogDirectory === undefined)
        delete process.env.MONOCHROMATIC_LOG_DIRECTORY;
      else
        process.env.MONOCHROMATIC_LOG_DIRECTORY = originalLogDirectory;
      await rm(
        dir,
        {
          force: true,
          recursive: true,
        },
      );
    },
  };
}

//endregion Throwaway package

//region JSONL readers

/**
 Parses newline-terminated JSONL text into its values.

 @param text - File or batch contents.

 @returns Parsed value per line.

 @throws Error when the text does not end with a newline, the shape every
 JSONL sink writes.

 @example
 ```ts
 parseJsonlLines({ text: '{"a":1}\n{"a":2}\n' }); // => [{ a: 1 }, { a: 2 }]
 ```
 */
export function parseJsonlLines({ text, }: { readonly text: string; },): readonly unknown[] {
  /**
   Lines including the empty string after the final newline.
   */
  const lines = text.split('\n',);
  if (lines.at(-1,) !== '')
    throw new Error('JSONL text must end with a newline',);
  return lines
    .slice(
    0,
    -1,
    )
    .map(function parseLine(line,): unknown {
      return JSON.parse(line,);
    },);
}

/**
 Reads the one log file the file sink created under a throwaway package
 and reparses its lines. The first line is the sink's own verify probe.

 @param dir - Throwaway package root.

 @returns Parsed value per line, probe first.

 @throws Error when the log directory holds more than one log file.

 @example
 ```ts
 const records = await readFileSinkLines({ dir: pkg.dir });
 ```
 */
export async function readFileSinkLines({ dir, }: { readonly dir: string; },): Promise<readonly unknown[]> {
  /**
   Directory the sink appends into.
   */
  const logDir = join(
    dir,
    'node_modules',
    LOG_DIR_NAME,
  );
  /**
   Log files the sink created. Other entries (the retention sweep stamp, day
   directories) are residue its contract permits, so only `*.log.jsonl` files
   count.
   */
  const logFiles: string[] = [];
  for (const entry of await readdir(logDir, { withFileTypes: true, },)) {
    /**
     Directory holding this entry: the log directory itself for a loose file,
     the UTC day directory for the sink's own layout.
     */
    const container = join(
      logDir,
      entry.name,
    );
    for (const name of entry.isDirectory() ? await readdir(container,) : [entry.name,])
      if (name.endsWith(LOG_FILE_SUFFIX,)) logFiles.push(join(
        container,
        name,
      ),);
  }
  if (logFiles.length > 1)
    throw new Error(`expected one log file under ${logDir}, found ${String(logFiles.length,)}: ${logFiles.join(', ',)}`,);
  /**
   Sole log file, absent when the run wrote no records (the sink then leaves
   no file at all, so the round trip is empty).
   */
  const [logFile] = logFiles;
  if (logFile === undefined) { return []; }
  /**
   Whole file contents.
   */
  const text = await readFile(
    logFile,
    'utf8',
  );
  return parseJsonlLines({ text, },);
}

/**
 Key prefix the sessionStorage sink writes batches under; the suffix is the
 batch index.
 */
const SESSION_KEY_PREFIX = 'monochromatic.log.';

/**
 Reads every batch the sessionStorage sink persisted, in batch order, and
 reparses their JSONL lines.

 @returns Parsed record per line across all batches.

 @throws Error when a batch key has no value.

 @example
 ```ts
 readSessionStorageLines();
 ```
 */
export function readSessionStorageLines(): readonly unknown[] {
  /**
   Web storage the sink wrote into.
   */
  const storage = globalThis.sessionStorage;
  /**
   Batches keyed by their index, in storage iteration order.
   */
  const batches: {
    readonly index: number;
    readonly batch: string
  }[] = [];
  for (let slot = 0; slot < storage.length; slot += 1) {
    /**
     Key at this slot.
     */
    const key = storage.key(slot,);
    if ((key === null) || (!key.startsWith(SESSION_KEY_PREFIX,)))
      continue;
    /**
     Batch text under the key.
     */
    const batch = storage.getItem(key,);
    if (batch === null)
      throw new Error(`sessionStorage key ${key} has no value`,);
    batches.push({
      batch,
      index: Number(key.slice(SESSION_KEY_PREFIX.length,),),
    },);
  }
  return batches
    .toSorted(function byIndex(
      left,
      right,
    ): number {
      return left.index - right.index;
    },)
    .flatMap(function parseBatch(entry,): readonly unknown[] {
      return parseJsonlLines({ text: `${entry.batch}\n`, },);
    },);
}

//endregion JSONL readers

//region Console reference

/**
 Highest C0 control code unit.
 */
const C0_CONTROL_LAST = 0x1F;

/**
 Tab, which the console sink leaves literal.
 */
const TAB_CODE_UNIT = 0x09;

/**
 Newline, which the console sink leaves literal.
 */
const NEWLINE_CODE_UNIT = 0x0A;

/**
 DEL.
 */
const DELETE_CODE_UNIT = 0x7F;

/**
 First C1 control code unit.
 */
const C1_CONTROL_FIRST = 0x80;

/**
 Last C1 control code unit.
 */
const C1_CONTROL_LAST = 0x9F;

/**
 Radix of the hex digits in a `\uXXXX` escape.
 */
const HEX_RADIX = 16;

/**
 Digit count of a `\uXXXX` escape.
 */
const ESCAPE_WIDTH = 4;

/**
 Enumerates the code units a terminal must never receive from the console
 sink: every C0 control except tab and newline, DEL, and every C1 control.
 Built as a lookup set rather than range tests so the reference shares no
 logic with the classifier under test.

 @returns Forbidden code units.

 @example
 ```ts
 forbiddenConsoleUnits().has(0x1B); // true
 ```
 */
function forbiddenConsoleUnits(): ReadonlySet<number> {
  /**
   Forbidden code units collected so far.
   */
  const units = new Set<number>();
  for (let unit = 0; unit <= C0_CONTROL_LAST; unit += 1)
    units.add(unit,);
  units.delete(TAB_CODE_UNIT,);
  units.delete(NEWLINE_CODE_UNIT,);
  units.add(DELETE_CODE_UNIT,);
  for (let unit = C1_CONTROL_FIRST; unit <= C1_CONTROL_LAST; unit += 1)
    units.add(unit,);
  return units;
}

/**
 Code units the console sink must never emit.
 */
const FORBIDDEN_CONSOLE_UNITS = forbiddenConsoleUnits();

/**
 Reports whether console-bound text still carries a forbidden control.

 @param text - Text handed to `console.*` or `process.stderr`.

 @returns Whether any code unit is forbidden.

 @example
 ```ts
 hasForbiddenConsoleUnit('a\u001Bb'); // true
 ```
 */
export function hasForbiddenConsoleUnit(text: string,): boolean {
  for (let index = 0; index < text.length; index += 1) {
    // oxlint-disable-next-line unicorn/prefer-code-point -- The reference classifies UTF-16 code units on purpose: every forbidden control is below U+00A0 and never part of a surrogate pair.
    if (FORBIDDEN_CONSOLE_UNITS.has(text.charCodeAt(index,),))
      return true;
  }
  return false;
}

/**
 Independent reference for the console sink's neutralizer: walks code
 units by index and renders each forbidden one as an uppercase `\uXXXX`
 escape.

 @param text - Message text.

 @returns Neutralized text.

 @example
 ```ts
 referenceNeutralize('a\u001Bb'); // => 'a\\u001Bb'
 ```
 */
export function referenceNeutralize(text: string,): string {
  /**
   Output pieces in input order.
   */
  const pieces: string[] = [];
  for (let index = 0; index < text.length; index += 1) {
    /**
     Code unit at this index; surrogate halves pass through untouched and rejoin on join.
     */
    // oxlint-disable-next-line unicorn/prefer-code-point -- Same code-unit walk as the forbidden check.
    const unit = text.charCodeAt(index,);
    pieces.push(
      FORBIDDEN_CONSOLE_UNITS.has(unit,)
        ? `\\u${
          unit.toString(HEX_RADIX,)
            .toUpperCase()
            .padStart(
              ESCAPE_WIDTH,
              '0',
            )
        }`
        : text.charAt(index,),
    );
  }
  return pieces.join('',);
}

/**
 Contiguous same-level slice of records and their formatted lines.
 */
type ReferenceRun = {
  readonly level: Level;
  readonly lines: readonly string[];
};

/**
 Renders one run as the single text the sink emits for it: lines joined by
 newline, plus the trailing newline the sink adds when it writes a debug run
 to process stderr.

 @param run - Completed run.

 @returns Emitted text.

 @example
 ```ts
 formatRun({ level: 'debug', lines: ['a', 'b'] }); // => 'a\nb\n'
 ```
 */
function formatRun(run: ReferenceRun,): string {
  /**
   Joined run text.
   */
  const text = run.lines
    .join('\n',);
  return (run.level === 'debug') ? `${text}\n` : text;
}

/**
 Predicts the exact texts the console sink hands to its outputs for records
 written in one synchronous frame: one text per contiguous same-level run,
 lines joined by newline, each line `[level] [iso] message` with the
 message neutralized, and debug runs carrying the trailing newline the
 sink adds when it writes them to process stderr.

 @param records - Records in write order.

 @returns Expected output texts in emission order.

 @example
 ```ts
 referenceConsoleRuns([{ level: 'info', message: 'a', timestamp: 0 }]);
 // => ['[info] [1970-01-01T00:00:00.000Z] a']
 ```
 */
export function referenceConsoleRuns(records: readonly LogRecord[],): readonly string[] {
  /**
   Runs built so far; the open run's lines grow until the level changes.
   */
  const runs: {
    readonly level: Level;
    readonly lines: string[];
  }[] = [];
  for (const record of records) {
    /**
     Formatted line for this record.
     */
    const line = `[${record.level}] [${
      new Date(record.timestamp,)
        .toISOString()
    }] ${referenceNeutralize(record.message,)}`;
    /**
     Run currently open, if any.
     */
    const open = runs.at(-1,);
    if ((open !== undefined) && (open.level === record.level))
      open.lines
        .push(line,);
    else
      runs.push({
        level: record.level,
        lines: [line,],
      },);
  }
  return runs.map(formatRun,);
}

//endregion Console reference
