import { StreamCutShortError, } from '../dist/final/node/index.mjs';

//region Cut stream test support
// A CUT STREAM AS THE DRAIN RAISES IT, for the cases that read what a cut had delivered: the abandonment predicate here and
// the seat tally in the stage package. One builder, so the progress reading is written once.

/**
 Raw characters on the wire, framing included, as the mercury streams of CuspariaKLSY22 read (7,300 to 10,650).
 */
const FRAMING_CHARS = 7_450;

/**
 When the first byte arrived on those streams, near 800 ms.
 */
const FIRST_BYTE_MS = 703;

/**
 Decoded characters of the answer and of the reasoning a watch counted.
 */
type GeneratedChars = {
  readonly content: number;
  readonly reasoning: number;
};

/**
 Builds a cut stream as the drain raises it, on a model that streamed framing and nothing else.

 @param firstByte - whether any body byte arrived, `never` for a stream that stayed silent from the start

 @param generatedChars - decoded characters of the answer and of the reasoning the watch counted, absent where the drain had none

 @returns The error a cut stream throws

 @example
 ```ts
 const error = cutStreamError({ firstByte: 'arrived', generatedChars: { content: 0, reasoning: 0, }, },);
 ```
 */
export function cutStreamError(
  {
    firstByte,
    generatedChars,
  }: {
    readonly firstByte: 'arrived' | 'never';
    readonly generatedChars?: GeneratedChars;
  },
): StreamCutShortError {
  /**
   Raw characters on the wire, none where nothing ever arrived.
   */
  const rawChars = (firstByte === 'never') ? 0 : FRAMING_CHARS;
  /**
   Whether any generated text arrived, which is what gives the stream a first-content time.
   */
  const generatedAny = (generatedChars !== undefined) && ((generatedChars.content + generatedChars.reasoning) > 0);
  return new StreamCutShortError({
    label: 'inception/mercury-2.5',
    partialText: 'x'.repeat(rawChars,),
    progress: {
      firstByteMs: (firstByte === 'never') ? -1 : FIRST_BYTE_MS,
      firstContentMs: generatedAny ? FIRST_BYTE_MS : -1,
      maxGapMs: 627,
      chars: rawChars,
      elapsedMs: 125_589,
    },
    ...((generatedChars === undefined) ? {} : { generatedChars, }),
    cause: new Error('exchange torn down by abort',),
  },);
}

//endregion Cut stream test support
