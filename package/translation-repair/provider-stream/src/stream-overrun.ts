import { StreamDegenerateError, } from './stream-runaway-watch.ts';
import type { StreamChannel, } from './stream-delta-scan.ts';

//region Stream overrun
// THE SECOND WAY A CALL RUNS AWAY, and the one repetition cannot see. A model
// that writes ten times more answer than any legitimate call ever wrote is not
// repeating itself: every window is distinct, so both detectors in
// `stream-runaway-watch.ts` report a healthy stream while it runs to the wall
// clock.
//
// The bound is a volume, measured rather than chosen:
// `doc/decision/translation-repair-runaway-call-termination.md` records the
// largest legitimate content emission anywhere at 4,278 characters, against
// the two content-producing cuts at 25,482 and 28,026.
//
// WHY A SHARED PREDICATE LIVES HERE. It was one defect repeated at several
// classification sites: the retry ladder re-bought a runaway because its check
// named one error class and a second had been added beside it. Every site that
// asks "did we end this call ourselves" now asks it through
// `isSelfEndedStream`, so a third guard added later updates one place instead
// of three.

/**
 Raised when a call is ended because it produced far more text than any
 legitimate call.

 ITS OWN CLASS RATHER THAN A FLAG on `StreamDegenerateError`, because the
 evidence differs: that one reports how little of the text was distinct, and
 this one reports how much text there was. Reporting a volume overrun as a
 distinct-window share would put a number in that field which was never
 computed.

 @example
 ```ts
 throw new StreamOverrunError({ label, channel: 'content', charsSeen: 40_000, cap: 32_000, rawChars: 3_600_000, },);
 ```
 */
export class StreamOverrunError extends Error {
  /**
   Declares this message safe to forward: it names the model, the channel and two bounds.
   */
  readonly messageNamesOnly: true = true;

  /**
   Model or endpoint whose stream overran.

   CARRIED AS A PROPERTY for the reason `StreamDegenerateError` carries it:
   every chat-completions call shares one endpoint across the roster, so
   attributing a stream to the endpoint makes a per-model figure unreadable.
   */
  readonly label: string;

  /**
   Channel that exceeded its bound.
   */
  readonly channel: StreamChannel;

  /**
   Characters that channel produced before the call was ended.
   */
  readonly charsSeen: number;

  /**
   Bound that was exceeded, carried so a log line says what was expected
   rather than only what happened.
   */
  readonly cap: number;

  /**
   Raw wire characters the stream delivered before the call was ended,
   envelope included: the unit the abandoned-spend reckoning divides by its
   measured raw-characters-per-token ratio (ledger P7). `charsSeen` counts
   one channel's decoded characters, about a hundredth of it.
   */
  readonly rawChars: number;

  /**
   @param label - what was being called, for the message

   @param channel - channel that exceeded its bound

   @param charsSeen - characters produced on that channel

   @param cap - bound that was exceeded

   @param rawChars - raw wire characters delivered before the end

   @example
   ```ts
   const error = new StreamOverrunError({
     label: 'editor',
     channel: 'content',
     charsSeen: 26_000,
     cap: 32_000,
     rawChars: 2_340_000,
   },);
   ```
   */
  constructor(
    {
      label,
      channel,
      charsSeen,
      cap,
      rawChars,
    }: {
      readonly label: string;
      readonly channel: StreamChannel;
      readonly charsSeen: number;
      readonly cap: number;
      readonly rawChars: number;
    },
  ) {
    super(
      `${label}: ended a call that exceeded its content bound, ${String(charsSeen,)} `
        + `characters on the ${channel} channel against a bound of ${String(cap,)}`,
    );
    this.name = 'StreamOverrunError';
    this.label = label;
    this.channel = channel;
    this.charsSeen = charsSeen;
    this.cap = cap;
    this.rawChars = rawChars;
  }
}

/**
 Says whether a failure is one this system chose rather than one it suffered.

 THE ONE PLACE THAT LIST LIVES. A caller abort is steering and a stall is
 weather; both of those are somebody else's decision. These two are ours, and
 every one of them is a decision not to spend more on this call, so retrying
 any of them buys back exactly what the guard just refused.

 @param error - whatever the call threw

 @returns True when a stream guard ended the call deliberately

 @example
 ```ts
 if (isSelfEndedStream({ error, },))
   throw error;
 ```
 */
export function isSelfEndedStream({ error, }: { readonly error: unknown; },): boolean {
  return (error instanceof StreamDegenerateError)
    || (error instanceof StreamOverrunError);
}

//endregion Stream overrun
