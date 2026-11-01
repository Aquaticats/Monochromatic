//region Server-sent event data line
// One reading of a streamed body's `data:` lines, shared by every reader of a
// provider stream: the live delta scanners that watch for a runaway
// (`stream-delta-scan.ts`, `anthropic-delta-scan.ts`) and the readers that fold
// the drained body into the answer (`stream-completion.ts`,
// `anthropic-completion.ts`, `openrouter-chunk-scan.ts`).
//
// THEY USED TO READ ONE STREAM TWO WAYS (audit area six, 2026-09-28). The
// scanners followed the event-stream format: one trailing carriage return
// dropped, the field name at the start of the line, one optional space removed.
// The folding readers trimmed every surrounding space first, so a line the
// format does not count as an event (one indented before `data:`) reached the
// answer while the guards watching the stream never saw it. Now one rule, the
// format's, decides which lines carry events for all five.

/**
 Prefix marking a line that carries a payload.

 NO TRAILING SPACE, deliberately, and this is a conformance requirement rather
 than a guess about any one sender. The event-stream parsing algorithm says of
 a field's value: "If value starts with a U+0020 SPACE character, remove it
 from value." So `data: {...}` and `data:{...}` are THE SAME MESSAGE, and a
 reader that accepts only the spaced form is simply wrong, whatever this
 provider happens to emit today.

 Spelling the prefix with the space would skip the tight form as though it
 were a comment, and skip it SILENTLY, since only `data:` lines are ever
 counted as unreadable. This repository has already paid for that shape of
 trap once: `runner-closure.ts` carries four import spellings because the tight
 form produced a false null that looked like a self-contained bundle.
 */
const DATA_PREFIX = 'data:';

/**
 Single optional space a sender may put after the colon.
 */
const OPTIONAL_SPACE = ' ';

/**
 Carriage return a server may pair with each newline.
 */
const CARRIAGE_RETURN = '\r';

/**
 Payload one line of a streamed body carries, by the event-stream format.

 @param line - one line of the body, without its newline

 @returns Text after `data:` and its one optional space, empty for a line
 that carries no event payload (a comment, another field, a blank line, or a
 keep-alive)

 @example
 ```ts
 ssePayloadOf({ line: 'data: {"type":"ping"}', },); // '{"type":"ping"}'
 ssePayloadOf({ line: 'data:[DONE]\r', },); // '[DONE]'
 ssePayloadOf({ line: ': keep-alive', },); // ''
 ```
 */
export function ssePayloadOf({ line, }: { readonly line: string; },): string {
  /**
   Line without the carriage return a server may pair with its newline.
   */
  const clean = line.endsWith(CARRIAGE_RETURN,)
    ? line.slice(
      0,
      -1,
    )
    : line;
  if (!clean.startsWith(DATA_PREFIX,))
    return '';

  /**
   Everything after the colon, which may begin with one optional space.
   */
  const afterColon = clean.slice(DATA_PREFIX.length,);
  return afterColon.startsWith(OPTIONAL_SPACE,)
    ? afterColon.slice(OPTIONAL_SPACE.length,)
    : afterColon;
}

//endregion Server-sent event data line
