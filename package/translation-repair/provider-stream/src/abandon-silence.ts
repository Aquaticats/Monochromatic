import { StreamCutShortError, } from './stream-cut.ts';

//region Abandon silence
// WHETHER A CUT STREAM HAD DELIVERED ANYTHING THE MODEL WROTE, read off the error that ended it, so the seat tally can
// tell a dead stream from a slow one.
//
// THE WIRE LENGTH IS NOT THE ANSWER. `inception/mercury-2.5` left 33 streams in CuspariaKLSY22 that read `0 content
// chars, 0 reasoning chars` beside 7,300 to 10,650 raw characters that grew with the wait: framing, no model text. A
// slow reasoning model cut at the same grace window reads `0 answer chars` too, with 25,400 to 69,336 reasoning
// characters behind it (zheermao22, 12 cuts), and that is a model still working. So this counts the decoded characters
// of both channels the drain's watch counted, and reads the wire length for nothing.

/**
 Threshold below which a first byte never arrived at all, as `abandon-kind.ts` reads it.
 */
const NO_FIRST_BYTE_MS = 0;

/**
 Whether a call that did not finish had delivered no answer and no reasoning.

 TRUE ONLY WHERE THE ERROR SAYS SO. A cut stream carrying the watch's count is silent when both channels read zero. A cut
 stream with no count is silent only when no byte ever arrived, since the wire length alone cannot say whether any of it
 was text. Any other error, an abort among them, says nothing of what was delivered, so it is not silent: unknown is not
 empty.

 @param error - whatever the call threw

 @returns True where the call is known to have delivered nothing the model wrote

 @example
 ```ts
 const silent = deliveredNoContent({ error, },);
 ```
 */
export function deliveredNoContent({ error, }: { readonly error: unknown; },): boolean {
  if (!(error instanceof StreamCutShortError))
    return false;
  /**
   The watch's count of decoded characters, absent where the caller had none.
   */
  const { generatedChars, } = error;
  if (generatedChars !== undefined)
    return (generatedChars.content === 0) && (generatedChars.reasoning === 0);
  /**
   When the first byte arrived, negative when none ever did.
   */
  const { firstByteMs, } = error.progress;
  return firstByteMs < NO_FIRST_BYTE_MS;
}

//endregion Abandon silence
