import { format, } from 'node:util';

import type { DisposableSandbox, } from '@monochromatic-dev/test-harness-core/ts';

//region Entry log lines
// THE LINES AN ENTRY'S LOGGERS WRITE WHILE WORK RUNS, read off every console method the tagged logger writes to:
// `console.info`, `console.warn`, `console.error` (where a fatal line goes too) and `console.trace`, through stubs on
// the calling case's own sandbox (`ctx.sinon`), which module-test restores when the case ends. The caller runs the work
// inside the log context it wants (this fixture imports no production code, so it cannot set one), and the lines that
// name the entry in a bracket come back for the case to read, one line each (the logger hands a run of lines of one
// level to the console as one call). `console.debug` is not read, because the logger writes debug records to the
// process's stderr, and only those a verbose run lets through (`LEVEL_TO_CONSOLE_METHOD` in the logger's console sink
// maps the levels).
//
// SEQUENTIAL CASES ONLY. The console methods are globals, and a case running beside this one would be captured.

/**
 Runs work and collects the lines the loggers wrote that name an entry.

 @param sinon - calling case's own sandbox (`ctx.sinon`)

 @param entryId - entry whose bracket (`[entry]`) a collected line holds

 @param run - work, run inside whatever log context the caller sets

 @returns Lines from `console.info`, `console.warn`, `console.error` and `console.trace`, each holding `[entryId]`

 @example
 ```ts
 const lines = await entryLinesWritten({ sinon: ctx.sinon, entryId: 'Tabby', run: async () => { await settle(); }, },);
 ```
 */
export async function entryLinesWritten(
  {
    sinon,
    entryId,
    run,
  }: {
    readonly sinon: DisposableSandbox;
    readonly entryId: string;
    readonly run: () => Promise<void>;
  },
): Promise<readonly string[]> {
  /**
   Stub on `console.info`, recording each call's arguments.
   */
  const informed = sinon.stub(
    console,
    'info',
  );
  /**
   Stub on `console.warn`, recording each call's arguments.
   */
  const warned = sinon.stub(
    console,
    'warn',
  );
  /**
   Stub on `console.error`, recording each call's arguments.
   */
  const errored = sinon.stub(
    console,
    'error',
  );
  /**
   Stub on `console.trace`, recording each call's arguments.
   */
  const traced = sinon.stub(
    console,
    'trace',
  );
  await run();
  /**
   Arguments of every call the work made to any of the four methods, `console.info` first.
   */
  const calls: readonly (readonly unknown[])[] = [
    ...informed.args,
    ...warned.args,
    ...errored.args,
    ...traced.args,
  ];
  return calls
    .flatMap(function linesOf(parts,): readonly string[] {
      // The logger hands a run of lines of one level to the console as one call, so a call can hold several lines.
      return format(...parts,)
        .split('\n',);
    },)
    .filter(function namesEntry(line,): boolean {
      return line.includes(`[${entryId}]`,);
    },);
}

/**
 The lines that do not name an attempt.

 @param lines - lines the loggers wrote

 @param attempt - number of the attempt the work ran inside

 @returns Those without the `[attempt n]` bracket

 @example
 ```ts
 expect(linesWithoutAttempt({ lines, attempt: 2, },),).toStrictEqual([],);
 ```
 */
export function linesWithoutAttempt(
  {
    lines,
    attempt,
  }: {
    readonly lines: readonly string[];
    readonly attempt: number;
  },
): readonly string[] {
  return lines.filter(function lacksAttempt(line,): boolean {
    return !line.includes(`[attempt ${String(attempt,)}]`,);
  },);
}

/**
 The lines that name an attempt more than once, which a lane slice-tagging a logger that is slice-tagged already
 would write if the tags were not written once per line (ledger B963).

 @param lines - lines the loggers wrote

 @param attempt - number of the attempt the work ran inside

 @returns Those holding the `[attempt n]` bracket twice or more

 @example
 ```ts
 expect(linesRepeatingAttempt({ lines, attempt: 2, },),).toStrictEqual([],);
 ```
 */
export function linesRepeatingAttempt(
  {
    lines,
    attempt,
  }: {
    readonly lines: readonly string[];
    readonly attempt: number;
  },
): readonly string[] {
  return lines.filter(function repeatsAttempt(line,): boolean {
    /**
     The pieces of the line around each bracket, one more than the brackets.
     */
    const pieces = line.split(`[attempt ${String(attempt,)}]`,);
    return pieces.length > 2;
  },);
}

//endregion Entry log lines
