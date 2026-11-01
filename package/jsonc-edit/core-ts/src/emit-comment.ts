import type { JsoncComment, } from './comment.ts';

//region Constants

/**
 One level of canonical indentation.
 */
const INDENT_UNIT = '  ';

/**
 Block-comment close delimiter; a comment body containing it cannot be emitted
 as a block comment, since C-family block comments do not nest.
 */
const BLOCK_CLOSE = '*/';

/**
 Line terminators. A `//` comment ends at any of them, so a body containing one
 cannot be emitted in trailing form, and a body line containing one cannot be
 emitted as a `//` line at all.
 */
const LINE_TERMINATORS = [
  '\r\n',
  '\n',
  '\r'
] as const;

//endregion Constants

//region Comment emit

/**
 Tests whether a comment body fits on a single line, deciding whether it can be
 emitted as a trailing comment.
 
 @param comment - Comment to test.
 
 @returns `true` when the body contains no line terminator.
 
 @example
 ```ts
 isSingleLineComment({ type: 'inline', text: ' note' }); // => true
 ```
 */
export function isSingleLineComment(comment: JsoncComment,): boolean {
  // A bare CR counts: `//` comments end at CR, LF or CRLF, so emitting a body that
  // contains one as a trailing comment would terminate it early and leave the rest of
  // the body as code. Fuzzing found this as an emission that the parser then rejected.
  return !LINE_TERMINATORS.some(function containsTerminator(terminator: string,): boolean {
    return comment.text
      .includes(terminator,);
  },);
}

/**
 Emits a single-line comment to sit after a value on the same line, always as a
 `//` line comment so no `*\/` termination hazard arises.
 
 @param comment - Single-line comment to emit.
 
 @returns Comment text, without indentation.
 
 @example
 ```ts
 trailingComment({ comment: { type: 'inline', text: ' note' } }); // => '// note'
 ```
 */
export function trailingComment({
  comment,
}: {
  readonly comment: JsoncComment;
},): string {
  return `//${comment.text}`;
}

/**
 Emits a comment as leading lines before a node, indented to its level and
 terminated with a newline. A block comment whose body has no `*\/` is kept as a
 block; everything else becomes one `//` line per body line, which is always
 safe to re-parse.
 
 @param comment - Comment to emit.
 
 @param indent - Indentation depth in levels.
 
 @returns Indented comment text ending in a newline.
 
 @example
 ```ts
 leadingComment({ comment: { type: 'inline', text: ' a' }, indent: 1 });
 // => '  // a\n'
 ```
 */
export function leadingComment({
  comment,
  indent,
}: {
  readonly comment: JsoncComment;
  readonly indent: number;
},): string {
  /**
   Indentation prefix for this depth.
   */
  const pad = INDENT_UNIT.repeat(indent,);
  // A body carrying a bare CR is emitted as a block whenever it can be, because a block comment
  // may contain CR verbatim while a `//` line ends there. A body carrying only LF keeps the
  // established one `//` line per body line layout. Bodies that contain the block close
  // delimiter cannot be blocks, so they fall through and their lines are split on every
  // terminator, which is lossy for CR but never corrupts the document.
  /**
   Whether the body can be wrapped in block delimiters at all.
   */
  const blockSafe = !comment.text
    .includes(BLOCK_CLOSE,);
  if (blockSafe && ((comment.type === 'block')
    || comment.text
      .includes('\r',)))
    return `${pad}/*${comment.text}*/\n`;
  /**
   Body lines after splitting on every terminator, so no emitted `//` line can carry one.
   */
  const lines = LINE_TERMINATORS.reduce< readonly string[]>(
    function splitOn(
      accumulator: readonly string[],
      terminator: string,
    ): readonly string[] {
      return accumulator.flatMap(function splitPart(part: string,): readonly string[] {
        return part
          .split(terminator,);
      },);
    },
    [comment.text,],
  );
  return `${lines
    .map(function lineToComment(line: string,): string {
      return `${pad}//${line}`;
    },)
    .join('\n',)}\n`;
}

//endregion Comment emit
