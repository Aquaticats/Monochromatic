import { isJsonRecord, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  parseModelJson,
  ssePayloadOf,
} from '@monochromatic-dev/translation-repair-provider-stream/ts';


//region OpenRouter chunk scan
// EVERY PARSED CHUNK OF A DRAINED CHAT COMPLETIONS STREAM, for the readers
// that want one field off the gateway's envelope rather than the generated
// text: the USD cost on the final chunk (`openrouter-cost.ts`) and the
// upstream endpoint named on every chunk (`openrouter-endpoint.ts`).
//
// SCANNED SEPARATELY FROM THE COMPLETION READER, which is shared with
// Synthetic and reports only text and token counts. Adding gateway-specific
// fields to `ExtractedCompletion` would make every other reader carry values
// it cannot fill.

/**
 Every chunk of one drained stream that parses as a JSON object, in arrival
 order.

 NOTHING HERE THROWS: a chunk that does not parse was already refused or
 accepted by the completion reader, and this scan reports nothing about it.
 The `[DONE]` sentinel, blank keep-alives and comment lines carry no JSON, so
 the parse fails on each and it is skipped. A chunk whose payload opens with
 spaces before its brace parses and is read, as the completion reader reads
 it.

 @param bodyText - whole drained `text/event-stream` body

 @returns Parsed chunks that are objects

 @example
 ```ts
 const chunks = openRouterChunksOf({ bodyText: reply.bodyText, },);
 ```
 */
export function openRouterChunksOf(
  { bodyText, }: { readonly bodyText: string; },
): readonly Readonly<Record<string, unknown>>[] {
  return bodyText
    .split('\n',)
    .flatMap(function chunkOf(rawLine,): readonly Readonly<Record<string, unknown>>[] {
      /**
       Payload of this line. NO `{`-START GATE: the parse runs on every
       payload, so the record guard's skip arm is what `data: 5` hits
       rather than code no input runs (ledger T8, 2026-10-04).
       */
      const payload = ssePayloadOf({ line: rawLine, },);
      /**
       Parse attempt, whose failure is data rather than a caught error this
       scan would have to drop (ledger B29): a chunk that does not parse was
       already refused or accepted by the completion reader, which reports
       it, and this scan only wants the envelope.
       */
      const attempt = parseModelJson({ text: payload, },);
      if (!attempt.parsed)
        return [];
      /**
       Parsed chunk.
       */
      const { value: chunk, } = attempt;
      if (!isJsonRecord(chunk,))
        return [];
      return [chunk,];
    },);
}

//endregion OpenRouter chunk scan
