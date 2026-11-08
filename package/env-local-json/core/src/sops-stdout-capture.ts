/**
 Stdout capture for one spawned child: the whole document it printed, read
 only after its stdio streams end.

 The seam exists because the completion event a caller awaits decides whether
 the join sees the whole pipe: `exit` fires while stdout can still hold
 unemitted data, and joining there returned empty or truncated documents under
 concurrent startup load, which surfaced upstream as a store that "is not
 valid JSON".

 @module
 */

import type { ChildProcess, } from 'node:child_process';
import { once, } from 'node:events';

//region Types

/**
 Capture state accumulated while one child runs. The counters stay mutable so
 every binding stays `const`.
 */
type StdoutCaptureState = {
  /**
   Text chunks received so far.
   */
  readonly chunks: string[];

  /**
   Characters received so far.
   */
  size: number;

  /**
   Whether the output bound was exceeded and the overflow callback ran.
   */
  overflow: boolean;
};

/**
 Result of watching one child to completion.
 */
export type ChildStdoutCapture = {
  /**
   Whole stdout text the child produced.
   */
  readonly output: string;

  /**
   Whether the output bound was exceeded and the overflow callback ran.
   */
  readonly overflow: boolean;

  /**
   Raw completion event payload of the child: the event tuple for a
   `ChildProcess`, the event object for an `EventTarget`.
   */
  readonly endEvent: unknown;
};

//endregion Types

//region Capture

/**
 Awaits one completion event from either emitter flavor node's `once` accepts.

 node's `once` types take `EventEmitter` and `EventTarget` as separate
 overloads, so the union parameter narrows before the call.

 @param source - emitter carrying the completion event

 @param eventName - completion event name

 @returns the completion event arguments

 @example
 ```ts
 const args = await awaitCompletion(child, 'close');
 ```
 */
function awaitCompletion({
  source,
  eventName,
}: {
  readonly source: NodeJS.EventEmitter | EventTarget;
  readonly eventName: string;
},): Promise<unknown[]> {
  if (source instanceof EventTarget) {
    return once(
      source,
      eventName,
    );
  }
  return once(
    source,
    eventName,
  );
}

/**
 Watches one child to completion and returns its whole stdout.

 Resolves at the child's `close` event, which fires after every stdio stream
 has ended. The `exit` event resolves earlier, and a join there loses the tail
 of the stdout pipe.

 @param completionSource - emitter carrying the child's completion events

 @param stdoutSource - child surface carrying the stdout stream

 @param onOverflow - terminates the child once the output bound is exceeded

 @param maxBytes - stdout bound in characters

 @returns the whole stdout, the overflow flag, and the completion event payload

 @throws Error when the child fails to spawn

 @example
 ```ts
 const capture = await collectChildStdout({
   completionSource: child,
   stdoutSource: child,
   onOverflow: () => { child.kill(); },
   maxBytes: MAX_SOPS_OUTPUT_BYTES,
 });
 ```
 */
export async function collectChildStdout({
  completionSource,
  stdoutSource,
  onOverflow,
  maxBytes,
}: {
  readonly completionSource: NodeJS.EventEmitter | EventTarget;
  readonly stdoutSource: Pick<ChildProcess, 'stdout'>;
  readonly onOverflow: () => void;
  readonly maxBytes: number;
},): Promise<ChildStdoutCapture> {
  /**
   Capture state filled in by the stdout listener.
   */
  const captured: StdoutCaptureState = {
    chunks: [],
    size: 0,
    overflow: false,
  };
  stdoutSource.stdout
    ?.setEncoding('utf8',);
  stdoutSource.stdout
    ?.on(
    'data',
    function captureChunk(chunk: string,): void {
      captured.chunks
        .push(chunk,);
      captured.size += chunk.length;
      if ((captured.size > maxBytes) && (!captured.overflow)) {
        captured.overflow = true;
        onOverflow();
      }
    },
    );
  /**
   Raw completion payload, taken at `close` so every stdout chunk has been
   emitted before the capture joins.
   */
  const endEvent: unknown = await awaitCompletion({
    source: completionSource,
    eventName: 'close',
  },);
  return {
    output: captured.chunks
      .join('',),
    overflow: captured.overflow,
    endEvent,
  };
}

//endregion Capture
