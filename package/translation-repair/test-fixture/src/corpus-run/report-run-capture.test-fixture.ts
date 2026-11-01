import type { DisposableSandbox, } from '@monochromatic-dev/test-harness-core/ts';

import { divertingConsoleLog, } from './console-log-capture.test-fixture.ts';

//region Report run capture
// WHAT A REPORT PRINTED AND THE EXIT CODE IT LEFT, read in process.
//
// A report prints with `console.log` and may set `process.exitCode`, and both
// are process-wide. This diverts the first through the calling case's own
// sandbox and puts the second back to what it was however the report ends, so
// a case that drives a report never decides the suite's own exit. A case that
// calls it sits in a suite that runs one case at a time: the exit code is shared
// by every case of the file.

/**
 What one run of a report printed and left behind.

 @example
 ```ts
 const captured: ReportCapture = { lines: ['meter-report: logs=1 readings=0 unread=0',], exitCode: 1, };
 ```
 */
export type ReportCapture = {
  /**
   Lines printed with `console.log`, in order, each as `console.log` prints it.
   */
  readonly lines: readonly string[];

  /**
   Exit code the report set, `unset` where it left none.
   */
  readonly exitCode: number | 'unset';
};

/**
 Runs a report with its printing diverted and its exit code held.

 @param sinon - calling case's own sandbox (`ctx.sinon`)

 @param run - report to run

 @returns Lines it printed and the exit code it left

 @throws Whatever the report throws, after the exit code is put back

 @example
 ```ts
 const captured = await captureReport({ sinon: ctx.sinon, run: async () => reportMeters({ line, },), },);
 ```
 */
export async function captureReport(
  {
    sinon,
    run,
  }: {
    readonly sinon: DisposableSandbox;
    readonly run: () => Promise<void>;
  },
): Promise<ReportCapture> {
  /**
   Exit code standing before the report ran.
   */
  const before = process.exitCode;

  /**
   Puts the exit code back when this scope ends, however it ends.
   */
  using restoring = {
    [Symbol.dispose](): void {
      process.exitCode = before;
    },
  };

  /**
   What the report prints, diverted into the case.
   */
  using printed = divertingConsoleLog({ sinon, },);
  process.exitCode = undefined;
  await run();

  /**
   Exit code the report left.
   */
  const left = process.exitCode;
  if (left === undefined)
    return {
      lines: printed.lines,
      exitCode: 'unset',
    };
  if ((typeof left) !== 'number')
    throw new Error(`unreachable: a report left an exit code that is not a number, ${String(left,)}`,);
  return {
    lines: printed.lines,
    exitCode: left,
  };
}

/**
 What one run of a report that returns its exit code printed, returned and
 left on the process.

 @example
 ```ts
 const captured: CodedReportCapture = { lines: ['meter-report: logs=1 readings=0 unread=0',], returned: 1, left: 'unset', };
 ```
 */
export type CodedReportCapture = {
  /**
   Lines printed with `console.log`, in order, each as `console.log` prints it.
   */
  readonly lines: readonly string[];

  /**
   What the report returned, read unchecked so a case compares it whole with
   the code it expects.
   */
  readonly returned: unknown;

  /**
   Exit code the report set on the process itself, `unset` where it left
   none, which is what a report that returns its code leaves.
   */
  readonly left: number | 'unset';
};

/**
 Runs a report that returns its exit code, with its printing diverted and the
 process's exit code held, so a case reads both the code it returned and
 whether it set one on the process as well.

 @param sinon - calling case's own sandbox (`ctx.sinon`)

 @param run - report to run, answering with the exit code it returns

 @returns Lines it printed, the code it returned and the code it left

 @throws Whatever the report throws, after the exit code is put back

 @example
 ```ts
 const captured = await captureCodedReport({ sinon: ctx.sinon, run: async () => reportMeters({ line, },), },);
 ```
 */
export async function captureCodedReport(
  {
    sinon,
    run,
  }: {
    readonly sinon: DisposableSandbox;
    readonly run: () => Promise<number>;
  },
): Promise<CodedReportCapture> {
  /**
   The code the report returned, kept by the run it is handed to.
   */
  const returned: unknown[] = [];

  /**
   What it printed and what it left on the process.
   */
  const captured = await captureReport({
    sinon,
    run: async function keepingCode(): Promise<void> {
      returned.push(await run(),);
    },
  },);
  return {
    lines: captured.lines,
    returned: returned.at(0,),
    left: captured.exitCode,
  };
}

//endregion Report run capture
