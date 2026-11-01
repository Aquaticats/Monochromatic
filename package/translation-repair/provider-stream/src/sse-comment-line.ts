//region Server-sent event comment line
// One reading of a streamed body's NAMED COMMENT lines, the half of the
// event-stream format that `sse-data-line.ts` skips on purpose.
//
// A COMMENT IS A LINE WHOSE FIRST CHARACTER IS A COLON, and the format says a
// reader ignores it. NeuralWatt uses that to carry a request's measured energy
// and its cost without breaking clients that validate every `data:` chunk
// against a strict schema: `: energy {...}` and `: cost {...}`, sent before
// `data: [DONE]` (`https://docs.neuralwatt.com/energy-methodology.md`, fetched
// 2026-10-20). `drainBody` hands the whole decoded body back, comments
// included, so nothing is lost on the way here; only the data-line reader is
// blind to them, which is why every answer reader stays unchanged.
//
// THE SAME TWO SMALL RULES AS A DATA LINE, from the same format: one trailing
// carriage return is dropped, and one optional space after the colon is
// removed, so `: energy {...}` and `:energy {...}` are one comment.

/**
 Prefix marking a comment line.
 */
const COMMENT_PREFIX = ':';

/**
 Single optional space a sender may put after the colon.
 */
const OPTIONAL_SPACE = ' ';

/**
 Carriage return a server may pair with each newline.
 */
const CARRIAGE_RETURN = '\r';

/**
 Text a named comment line carries.

 @param line - one line of the body, without its newline

 @param name - word the comment opens with, as the sender spells it

 @returns Text after the name and the one space that follows it, empty for a
 line that is not a comment, a comment of another name (one that only begins
 with this name included), or a comment that carries nothing after its name

 @example
 ```ts
 sseCommentOf({ line: ': energy {"energy_kwh": 0.000015}', name: 'energy', },); // '{"energy_kwh": 0.000015}'
 sseCommentOf({ line: ': keep-alive', name: 'energy', },); // ''
 ```
 */
export function sseCommentOf(
  {
    line,
    name,
  }: {
    readonly line: string;
    readonly name: string;
  },
): string {
  /**
   Line without the carriage return a server may pair with its newline.
   */
  const clean = line.endsWith(CARRIAGE_RETURN,)
    ? line.slice(
      0,
      -1,
    )
    : line;
  if (!clean.startsWith(COMMENT_PREFIX,))
    return '';

  /**
   Everything after the colon, which may begin with one optional space.
   */
  const afterColon = clean.slice(COMMENT_PREFIX.length,);

  /**
   The comment's own text: the name, one space, then what it carries.
   */
  const comment = afterColon.startsWith(OPTIONAL_SPACE,)
    ? afterColon.slice(OPTIONAL_SPACE.length,)
    : afterColon;

  /**
   What a comment of this name opens with: the name and the space after it.
   */
  const opening = `${name}${OPTIONAL_SPACE}`;
  if (!comment.startsWith(opening,))
    return '';
  return comment.slice(opening.length,);
}

//endregion Server-sent event comment line
