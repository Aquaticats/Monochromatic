import { format, } from 'node:util';

import type { DisposableSandbox, } from '@monochromatic-dev/test-harness-core/ts';

//region Console log capture
// READS WHAT console.log PRINTED WHILE A CASE RAN, through a stub on the
// calling case's own sandbox, `ctx.sinon`, which module-test answers only to
// that case (`sandbox-slot.ts`) and restores when the case ends; disposing
// the capture restores it sooner. The stub records each call's arguments, and
// a line is read off them the way console.log itself prints them
// (`util.format`), so no fake stands in for console.log and no part of a
// call is dropped. `divertingConsoleError` reads console.error the same way.
//
// TEST SUPPORT, NOT PACKAGE SOURCE. artifact-placement,
// rendering-audit-settled and rendering-audit-settled-report kept their own
// copy of the relaying capture, and bench-report-groups and
// probe-telemetry-report of the diverting one, each assigning console.log
// directly; all five now import these.

/**
 What a capture hands its case: the lines printed so far, and a disposer
 restoring console.log.
 */
type ConsoleLogCapture = {
  /**
   Lines printed since the capture began, in order, read afresh each time.
   */
  readonly lines: readonly string[];
} & Disposable;

/**
 The parts of an installed stub a capture reads: its recorded calls and its
 own restore.
 */
type InstalledStub = {
  /**
   Recorded calls, one argument list each, read at the moment of asking.
   */
  readonly args: readonly (readonly unknown[])[];

  /**
   Puts back the console.log the stub replaced.
   */
  readonly restore: () => void;
};

/**
 Builds the capture over one installed stub.

 @param stub - stub on console.log, whose recorded calls are the lines

 @returns Capture reading its lines off the recorded calls

 @example
 ```ts
 const capture = captureOver({ stub, },);
 ```
 */
function captureOver({ stub, }: { readonly stub: InstalledStub; },): ConsoleLogCapture {
  return {
    /**
     Each recorded call as console.log prints it.
     */
    get lines(): readonly string[] {
      return stub.args
        .map(function lineOf(parts,): string {
          return format(...parts,);
        },);
    },
    [Symbol.dispose]: function restoreLog(): void {
      stub.restore();
    },
  };
}

/**
 Captures what `console.log` prints while still printing it, so the
 runner's own output keeps flowing.

 @param sinon - calling case's own sandbox (`ctx.sinon`)

 @returns Capture of the printed lines, restoring console.log on disposal

 @example
 ```ts
 using printed = relayingConsoleLog({ sinon: ctx.sinon, },);
 ```
 */
export function relayingConsoleLog({ sinon, }: { readonly sinon: DisposableSandbox; },): ConsoleLogCapture {
  /**
   Stub forwarding every call to the console.log it replaced.
   */
  const stub = sinon
    .stub(
    console,
    'log',
    )
    .callThrough();
  return captureOver({ stub, },);
}

/**
 Captures what `console.log` would print, printing nothing meanwhile.

 @param sinon - calling case's own sandbox (`ctx.sinon`)

 @returns Capture of the diverted lines, restoring console.log on disposal

 @example
 ```ts
 using capture = divertingConsoleLog({ sinon: ctx.sinon, },);
 ```
 */
export function divertingConsoleLog({ sinon, }: { readonly sinon: DisposableSandbox; },): ConsoleLogCapture {
  /**
   Stub swallowing every call, recording its arguments.
   */
  const stub = sinon.stub(
    console,
    'log',
  );
  return captureOver({ stub, },);
}

/**
 Captures what `console.error` would print, printing nothing meanwhile, for a
 runner whose refusal lines go to stderr.

 @param sinon - calling case's own sandbox (`ctx.sinon`)

 @returns Capture of the diverted lines, restoring console.error on disposal

 @example
 ```ts
 using said = divertingConsoleError({ sinon: ctx.sinon, },);
 ```
 */
export function divertingConsoleError({ sinon, }: { readonly sinon: DisposableSandbox; },): ConsoleLogCapture {
  /**
   Stub swallowing every call to console.error, recording its arguments.
   */
  const stub = sinon.stub(
    console,
    'error',
  );
  return captureOver({ stub, },);
}

//endregion Console log capture
