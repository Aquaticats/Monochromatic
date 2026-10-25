import { format, } from 'node:util';

import type { DisposableSandbox, } from '@monochromatic-dev/module-test/ts';

//region Entry log lines
// THE LINES AN ENTRY'S LOGGERS WRITE WHILE WORK RUNS, read off `console.info` and `console.warn`, where the tagged
// logger's `info` and `warn` sinks resolve, through stubs on the calling case's own sandbox (`ctx.sinon`), which
// module-test restores when the case ends. The caller runs the work inside the log context it wants (this fixture
// imports no production code, so it cannot set one), and the lines that name the entry in a bracket come back for the
// case to read. `console.debug` is not read, because the logger writes debug records to the process's stderr.
//
// SEQUENTIAL CASES ONLY. `console.info` is one global, and a case running beside this one would be captured.

/**
 Runs work and collects the lines the loggers wrote that name an entry.

 @param sinon - calling case's own sandbox (`ctx.sinon`)

 @param entryId - entry whose bracket (`[entry]`) a collected line holds

 @param run - the work, run inside whatever log context the caller sets

 @returns Lines from `console.info` then `console.warn`, each holding `[entryId]`

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
  await run();
  return [
    ...informed.args,
    ...warned.args,
  ]
    .map(function lineOf(parts,): string {
      return format(...parts,);
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

//endregion Entry log lines
