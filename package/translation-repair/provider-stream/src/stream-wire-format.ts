import { scanAnthropicDeltas, } from './anthropic-delta-scan.ts';
import {
  type DeltaScanner,
  scanStreamDeltas,
} from './stream-delta-scan.ts';

//region Stream wire format
// WHICH EVENT GRAMMAR A DRAINED STREAM SPEAKS, as a closed choice rather than
// an injected function.
//
// The runaway guards, the idle and straggler windows and the answer-volume
// bound all read a `DeltaScanner`, and every threshold in them was measured.
// Two providers speak two grammars, and letting a caller pass any scanner would
// make it possible to drain a stream with a reader that does not understand it,
// which fails silently: the guards would see an empty answer channel and a
// perfectly well-behaved call.
//
// A NAMED FORMAT INSTEAD, so the set of readable grammars is the set of
// grammars somebody wrote a reader for.

/**
 Event grammars a drained stream can speak.

 @example
 ```ts
 const wireFormat: StreamWireFormat = 'anthropic';
 ```
 */
export type StreamWireFormat = 'openai' | 'anthropic';

/**
 Reader for each grammar.
 */
const WIRE_SCANNERS: Readonly<Record<StreamWireFormat, () => DeltaScanner>> = {
  openai: scanStreamDeltas,
  anthropic: scanAnthropicDeltas,
};

/**
 Grammar assumed where a caller names none.

 THE OLDER ONE, so every existing call site keeps draining exactly the stream
 it drained before this choice existed.
 */
export const DEFAULT_WIRE_FORMAT: StreamWireFormat = 'openai';

/**
 Opens a reader for one grammar.

 @param wireFormat - grammar the stream speaks

 @returns Fresh scanner, which carries per-stream state and is never shared

 @example
 ```ts
 const scanner = scannerFor({ wireFormat: 'anthropic', },);
 ```
 */
export function scannerFor(
  { wireFormat, }: { readonly wireFormat: StreamWireFormat; },
): DeltaScanner {
  return WIRE_SCANNERS[wireFormat]();
}

//endregion Stream wire format
