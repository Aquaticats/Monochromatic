/**
 Pure text-scanning primitives for TSDoc comment processing.
 
 Leaf module with no dependency on the parser side, so the block scanner,
 the structural-message scanner, and the rule modules can all share these
 helpers without forming an import cycle through `tsdoc-comments.ts`.
 
 @module
 */

import {
  isWhitespaceChar as sharedIsWhitespaceChar,
  isWordChar as sharedIsWordChar,
} from '@monochromatic-dev/oxlint-plugin-shared/ts';
import type { Comment, } from '@oxlint/plugins';
import type { ReadonlyDeep, } from 'type-fest';

export {
  isWhitespaceChar,
  isWordChar,
} from '@monochromatic-dev/oxlint-plugin-shared/ts';

/**
 Triple-backtick delimiter that opens or closes a fenced code block.
 */
const FENCE_DELIMITER = '```';

/**
 Single-backtick delimiter that wraps inline-code segments.
 */
const INLINE_CODE_DELIMITER = '`';

/**
 Backslash-escape sequence stripped before tag scanning so an escaped
 `\@` reference is not mistaken for a real tag.
 */
const ESCAPED_AT: string = String.raw`\@`;

/**
 One normalized comment line plus whether it sits inside a fenced code block.
 
 `inFence` lets later passes skip block-tag detection on example bodies
 without re-deriving the running fence state.
 */
export type NormalizedLine = {
  /**
   Line text after indentation, leading `*`, and the following gap are removed.
   */
  readonly text: string;
  /**
   True for fence-delimiter lines and any line between them.
   */
  readonly inFence: boolean;
};

/**
 Checks whether `line` is a fenced-code-block delimiter (allows leading
 whitespace before the triple backticks).
 
 @param line - raw TSDoc comment line
 
 @returns whether line opens or closes a fenced code block
 
 @example
 ```ts
 isFenceLine('  ```ts'); // true
 ```
 */
export function isFenceLine(line: string,): boolean {
  return line.trimStart()
    .startsWith(FENCE_DELIMITER,);
}

/**
 Strips the leading `*` marker from a TSDoc block-comment line whose
 indentation has already been removed by `String.prototype.trimStart`.
 
 Returns the input verbatim when the line does not begin with `*`.
 
 @param s - line text after `trimStart`
 
 @returns line text with leading `*` removed when present
 
 @example
 ```ts
 stripCommentLineMarker('* @param x foo'); // ' @param x foo'
 stripCommentLineMarker('@param x foo'); // '@param x foo'
 ```
 */
export function stripCommentLineMarker(s: string,): string {
  return s.startsWith('*',) ? s.slice(1,) : s;
}

/**
 Splits a block comment value into its constituent lines.
 
 @param comment - block comment AST node
 
 @returns array of lines (without opening `/*` and closing `*\/`)
 
 @example
 ```ts
 const lines = getCommentLines(commentNode);
 ```
 */
export function getCommentLines(comment: ReadonlyDeep<Comment>,): readonly string[] {
  return comment.value
    .split('\n',);
}

/**
 Removes inline-code spans bounded by single backticks via a linear scan.
 
 Each opening backtick is paired with the next backtick via `indexOf`; the
 span between them (inclusive of both delimiters) is dropped before the
 cursor resumes past the closing delimiter, so each character is visited a
 bounded number of times and the stack stays flat.
 
 @param s - line content
 
 @returns `s` with every backtick-delimited span removed
 
 @example
 ```ts
 stripInlineCodeSpans('a `code` b'); // 'a  b'
 ```
 */
export function stripInlineCodeSpans(s: string,): string {
  /**
   Plain-text segments collected between code spans; joined into the result.
   */
  const parts: string[] = [];
  for (let from = 0; from <= s
    .length;) {
    /**
     Position of the next opening backtick; -1 means the rest is plain text.
     */
    const open = s.indexOf(
      INLINE_CODE_DELIMITER,
      from,
    );
    if (open === (-1)) {
      parts.push(s.slice(from,),);
      break;
    }
    /**
     Position of the closing backtick; -1 means the line ends inside inline code.
     */
    const close = s.indexOf(
      INLINE_CODE_DELIMITER,
      open + 1,
    );
    if (close === (-1)) {
      parts.push(s.slice(from,),);
      break;
    }
    parts.push(s.slice(
      from,
      open,
    ),);
    from = close + 1;
  }
  return parts.join('',);
}

/**
 Strips inline code spans (via {@link stripInlineCodeSpans}) and
 backslash-escaped at signs from a line so tag scanning does not produce
 false positives on package names or escaped tag references.
 
 @param line - raw TSDoc comment line
 
 @returns line with inline code and escaped at signs removed
 
 @example
 ```ts
 stripInlineCodeAndEscapes('see `@x` and \\@y'); // 'see  and y'
 ```
 */
export function stripInlineCodeAndEscapes(line: string,): string {
  return stripInlineCodeSpans(line,)
    .replaceAll(
    ESCAPED_AT,
    '',
    );
}

/**
 Returns the exclusive end index of the ASCII word run starting at `start`.
 
 Uses a bounded `for` cursor so the scan is linear and the stack stays flat.
 
 @param text - string scanned for the word run
 
 @param start - index where the word run is expected to begin
 
 @returns first index at or after `start` that is not a word character
 
 @example
 ```ts
 wordRunEnd({ text: '@param x', start: 1 }); // 6
 ```
 */
export function wordRunEnd({
  text,
  start,
}: {
  /**
   String scanned for the word run.
   */
  readonly text: string;
  /**
   Index where the word run is expected to begin.
   */
  readonly start: number;
},): number {
  for (let cursor = start; cursor < text
    .length; cursor += 1) {
    if (!sharedIsWordChar(text.charAt(cursor,),))
      return cursor;
  }
  return text.length;
}

/**
 Returns the exclusive end index of the first non-whitespace token starting
 at `start`.
 
 @param text - string scanned for the token
 
 @param start - index where the token is expected to begin
 
 @returns first index at or after `start` that is whitespace, or the length
 
 @example
 ```ts
 tokenEnd({ text: 'name - desc', start: 0 }); // 4
 ```
 */
export function tokenEnd({
  text,
  start,
}: {
  /**
   String scanned for the token.
   */
  readonly text: string;
  /**
   Index where the token is expected to begin.
   */
  readonly start: number;
},): number {
  for (let cursor = start; cursor < text
    .length; cursor += 1) {
    if (sharedIsWhitespaceChar(text.charAt(cursor,),))
      return cursor;
  }
  return text.length;
}

/**
 Returns the exclusive end index of the whitespace run starting at `start`.
 
 @param text - string scanned for whitespace
 
 @param start - index where the whitespace run is expected to begin
 
 @returns first index at or after `start` that is not whitespace
 
 @example
 ```ts
 whitespaceRunEnd({ text: '@param name', start: 6 }); // 7
 ```
 */
export function whitespaceRunEnd({
  text,
  start,
}: {
  /**
   String scanned for whitespace.
   */
  readonly text: string;
  /**
   Index where the whitespace run is expected to begin.
   */
  readonly start: number;
},): number {
  for (let cursor = start; cursor < text
    .length; cursor += 1) {
    if (!sharedIsWhitespaceChar(text.charAt(cursor,),))
      return cursor;
  }
  return text.length;
}

/**
 Sentinel meaning "line has no leading tag"; never a real tag string.
 
 A unique symbol (like `NO_TSDOC`) rather than `''`/`undefined`, so the
 `no-nullish-union` rule is satisfied and callers narrow with `typeof` or an
 identity check.
 */
export const NO_TAG: unique symbol = Symbol('tsdoc line has no leading tag',);

/**
 Returns the leading block/inline tag of a normalized line, including the
 `@`, or {@link NO_TAG} when the line does not begin with an `@word`.
 
 Strips inline code and escapes first (via {@link stripInlineCodeAndEscapes})
 so a tag wrapped in backticks does not register as the line's leading tag.
 
 @param normalizedText - line text after marker stripping
 
 @returns leading tag such as `'@param'`, or {@link NO_TAG}
 
 @example
 ```ts
 leadingTag('@param x - desc'); // '@param'
 leadingTag('plain text'); // NO_TAG
 ```
 */
export function leadingTag(normalizedText: string,): string | typeof NO_TAG {
  /**
   Line with inline code and escaped at signs removed before tag detection.
   */
  const stripped = stripInlineCodeAndEscapes(normalizedText,);
  if (!stripped.startsWith('@',))
    return NO_TAG;
  /**
   Exclusive end of the tag-name run; equals 1 when no word follows the `@`.
   */
  const end = wordRunEnd({
    text: stripped,
    start: 1,
  },);
  if (end === 1)
    return NO_TAG;
  return stripped.slice(
    0,
    end,
  );
}

/**
 Iterates every literal `@word` occurrence in `stripped`, yielding the
 captured word (without the leading `@`).
 
 Each `@` is located via `indexOf`, the trailing word run is collected by
 `wordRunEnd`, and the cursor jumps past the run so each character is
 inspected at most twice.
 
 @param stripped - line content with inline code/escapes already removed
 
 @returns ordered tag-name list (each without `@`)
 
 @example
 ```ts
 collectTags('see @param and @returns'); // ['param', 'returns']
 ```
 */
export function collectTags(stripped: string,): readonly string[] {
  /**
   Tag words collected in source order; each entry omits its leading at-sign.
   */
  const out: string[] = [];
  for (let from = 0; from < stripped
    .length;) {
    /**
     Position of the next at-sign; -1 ends the scan.
     */
    const atIdx = stripped.indexOf(
      '@',
      from,
    );
    if (atIdx === (-1))
      break;
    /**
     Exclusive end of the word run; equals `atIdx + 1` when no word follows.
     */
    const end = wordRunEnd({
      text: stripped,
      start: atIdx + 1,
    },);
    if (end === (atIdx + 1)) {
      from = atIdx + 1;
      continue;
    }
    out.push(stripped.slice(
      atIdx + 1,
      end,
    ),);
    from = end;
  }
  return out;
}

/**
 Normalizes a comment into per-line text plus running fenced-code state.
 
 Each line is reduced to its content (indentation, leading `*`, and the gap
 after it removed); a fence-delimiter line toggles the running state and is
 itself marked as fenced so block detection skips example bodies.
 
 @param comment - block comment AST node
 
 @returns ordered normalized lines with their fence flags
 
 @example
 ```ts
 const lines = normalizeLines({ comment });
 ```
 */
export function normalizeLines({
  comment,
}: {
  /**
   Block comment whose body is split and normalized.
   */
  readonly comment: ReadonlyDeep<Comment>;
},): readonly NormalizedLine[] {
  /**
   Raw comment body split into lines before normalization.
   */
  const rawLines = getCommentLines(comment,);
  /**
   Fold seed carrying the running fence state and accumulated rows.
   */
  const seed: {
    inside: boolean;
    rows: NormalizedLine[];
  } = {
    inside: false,
    rows: [],
  };
  /**
   Folded result whose `rows` hold every normalized line in source order.
   */
  const folded = rawLines.reduce(
    /**
     Handles effectful plugin callback.
     
     @param acc - Mutable fold state accumulated across comment lines.
     
     @param rawLine - Current unnormalized comment line.
     
     @returns same fold state after current line normalization.
     
     @mutates acc - Updates fence state and appends normalized comment rows.
     
     @example
     ```ts
     fold(acc);
     ```
     */
    function fold(
      acc,
      rawLine,
    ): {
      inside: boolean;
      rows: NormalizedLine[];
    } {
      /**
       Line content after indentation, leading `*`, and following gap removed.
       */
      const normalized = stripCommentLineMarker(rawLine.trimStart(),)
        .trimStart();
      // Detect the fence on the normalized text: a TSDoc comment line carries a
      // ` * ` prefix, so the triple backticks only sit at the start once the
      // marker is stripped.
      if (isFenceLine(normalized,)) {
        acc.rows
          .push({
            text: normalized,
            inFence: true,
          },);
        return {
          inside: !acc.inside,
          rows: acc.rows,
        };
      }
      acc.rows
        .push({
          text: normalized,
          inFence: acc.inside,
        },);
      return acc;
    },
    seed,
  );
  return folded.rows;
}
