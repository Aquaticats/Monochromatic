/**
 Terminal failures of the native TypeScript synchronous channel.

 @module
 */

/**
 Markers naming a channel whose child process is gone and can never answer again.

 Node system errors carry one of these as their `code`; the native sync channel
 raises plain errors whose message carries the same vocabulary.
 */
const TERMINAL_CHANNEL_MARKERS: readonly string[] = [
  'EPIPE',
  'EBADF',
  'EIO',
  'ERR_STREAM_DESTROYED',
  'SyncRpcChannel is closed',
  'Unexpected EOF while reading from child process',
];

/**
 Error-shaped fields read while walking a cause chain.
 */
type ErrorLink = {
  readonly code?: unknown;
  readonly message?: unknown;
  readonly cause?: unknown;
};

/**
 Reports whether one cause chain link carries error-shaped fields.

 @param value - Candidate link from the error cause chain.

 @returns whether the walk should read value as an error.
 */
function isErrorLink(value: unknown,): value is ErrorLink {
  return ((typeof value) === 'object') && (value !== null);
}

/**
 Reports whether one chain link's text names a dead channel.

 @param text - Rendered code or message from the link.

 @returns whether text carries terminal channel vocabulary.
 */
function textNamesChannelDeath(text: string,): boolean {
  return TERMINAL_CHANNEL_MARKERS.some(function markerNamed(marker,): boolean {
    return text.includes(marker,);
  },);
}

/**
 Reports whether an error came from writing to a native channel whose child is gone.

 A dead channel surfaces as a Node system error (`EPIPE`, `EBADF`, `EIO`,
 `ERR_STREAM_DESTROYED`) or as one of the native sync channel's own shutdown
 errors. Any match means every later request fails the same way, so the only
 sound reaction is to retire the bridge instead of writing again.

 @param error - Caught value from a failed bridge request or teardown.

 @returns whether the failure is a terminal channel failure.

 @example
 ```ts
 isTerminalSemanticChannelFailure({ error: new Error('boom'), }); // false
 isTerminalSemanticChannelFailure({ error: new Error('write EPIPE'), }); // true
 ```
 */
export function isTerminalSemanticChannelFailure({
  error,
}: {
  readonly error: unknown;
},): boolean {
  /**
   Chain links left to inspect, seeded with the caught value; the work stack keeps
   the linear cause walk iterative instead of recursive.
   */
  const work: unknown[] = [error,];
  /**
   Links already inspected, so a cause referencing an ancestor cannot loop forever.
   */
  const seen = new Set<unknown>();
  while (work.length > 0) {
    /**
     Next chain link to classify, present because the work stack is non-empty.
     */
    const current = work.pop();
    if (seen.has(current,))
      continue;
    seen.add(current,);
    if (isErrorLink(current,)) {
      /**
       Error code field from the current link, untyped until its shape is checked.
       */
      const { code, } = current;
      if (((typeof code) === 'string') && textNamesChannelDeath(code,))
        return true;
      /**
       Error message field from the current link, untyped until its shape is checked.
       */
      const { message, } = current;
      if (((typeof message) === 'string') && textNamesChannelDeath(message,))
        return true;
      if (current.cause !== undefined)
        work.push(current.cause,);
      continue;
    }
    if (((typeof current) === 'string') && textNamesChannelDeath(current,))
      return true;
  }
  return false;
}
