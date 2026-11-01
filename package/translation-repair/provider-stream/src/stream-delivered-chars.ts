import { StreamCutShortError, } from './stream-cut.ts';
import { StreamOverrunError, } from './stream-overrun.ts';
import { StreamDegenerateError, } from './stream-runaway-watch.ts';

//region Stream delivered characters
// WHAT A STREAM THAT DID NOT FINISH HAD DELIVERED, read off the error that
// ended it. Moved out of `openrouter-abandoned-spend.ts` when the retry
// ladder began reporting every abandoned attempt (ledger P1): the ladder sits
// under every provider, and a provider's reckoning module is not its home.

/**
 Reads what an abandoned stream had delivered off the error that ended it.

 RAW WIRE CHARACTERS FROM EVERY ERROR (ledger P7, 2026-09-28): the ratio this
 is divided by is raw characters per token, and an overrun or a degenerate
 ending was read by one channel's decoded count, about a hundredth of the raw
 figure (`completion=5` for 1,633 content characters on shihai4h2).

 @param error - whatever the exchange threw

 @returns Raw characters delivered, or that the error says nothing about it

 @example
 ```ts
 const delivered = deliveredCharsOf({ error, },);
 ```
 */
export function deliveredCharsOf(
  { error, }: { readonly error: unknown; },
): number | 'nothing-known' {
  if (error instanceof StreamCutShortError) {
    /**
     Text the cut stream had delivered.
     */
    const { partialText, } = error;
    return partialText.length;
  }
  if (error instanceof StreamOverrunError)
    return error.rawChars;
  if (error instanceof StreamDegenerateError)
    return error.rawChars;
  return 'nothing-known';
}

//endregion Stream delivered characters
