/**
 {@link parseTomlEdit}: parse a source string into an immutable edit state.
 
 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { buildBlocks, } from './build-document.ts';
import { TomlEditError, } from './errors.ts';
import { safeParse, } from './safe-parse.ts';
import {
  DEFAULT_CANONICAL_OPTIONS,
  type TomlEditOptions,
  type TomlEditState,
} from './types.ts';

/**
 Carriage-return code unit (U+000D).
 */
const CARRIAGE_RETURN = 0x0D;

/**
 Line-feed code unit (U+000A).
 */
const LINE_FEED = 0x0A;

/**
 Reject a bare carriage return: a `CR` not immediately followed by `LF`.
 
 TOML permits `CR` only as part of a `CRLF` newline; a lone `CR` (anywhere,
 including inside a multiline string) is invalid, but `toml-eslint-parser`
 accepts it. This pre-parse scan closes that gap so {@link parseTomlEdit} rejects the
 input the way the spec requires. A trailing `CR` is rejected too, since
 `codePointAt` past the end yields `undefined`, which is not `LF`.
 
 @param source - Raw TOML source.
 
 @throws {@link TomlEditError} when a bare carriage return is present.
 
 @example
 ```ts
 assertNoBareCarriageReturn({ source: 'a = 1\r\n', }); // ok (CRLF)
 ```
 */
function assertNoBareCarriageReturn({ source, }: { readonly source: string; },): void {
  for (let index = 0; index < source.length; index += 1) {
    if ((source.codePointAt(index,) === CARRIAGE_RETURN) && (source.codePointAt(index + 1,) !== LINE_FEED))
      throw new TomlEditError('Failed to parse TOML: a bare carriage return is not allowed; CR must be part of CRLF',);
  }
}

/**
 Normalize newlines to LF, warning when a CRLF document is converted.
 
 A bare carriage return is rejected first, via {@link assertNoBareCarriageReturn}.
 Any surviving CR is therefore part
 of a CRLF, which is converted to LF so the splice, comment-range, and emit
 paths only ever reason about single-byte LF newlines. The returned source,
 not the caller's original, is what the edit state holds, so a CRLF input
 round-trips as LF by design; the warning makes that conversion visible.
 
 @param source - Raw TOML source.
 
 @returns Source with every CRLF converted to LF.
 
 @throws {@link TomlEditError} when a bare carriage return is present.
 
 @example
 ```ts
 normalizeNewlines({ source: crlfText, }); // LF text, warns once
 ```
 */
function normalizeNewlines({ source, }: { readonly source: string; },): string {
  assertNoBareCarriageReturn({ source, },);
  /**
   Carriage-return character, built from its code unit to avoid a raw byte.
   */
  const carriageReturn = String.fromCodePoint(CARRIAGE_RETURN,);
  if (!source.includes(carriageReturn,)) return source;
  /**
   Line-feed character, the normalization target.
   */
  const lineFeed = String.fromCodePoint(LINE_FEED,);
  tagged({ tag: normalizeNewlines.name, },)
    .warn('CRLF line endings detected; the document is normalized to LF',);
  return source.replaceAll(
    `${carriageReturn}${lineFeed}`,
    lineFeed,
  );
}

/**
 Parse a TOML source string and produce a fresh {@link TomlEditState}.
 
 The returned state holds the newline-normalized source and the parse-time
 AST. CRLF input is converted to LF (with a warning) before parsing, so the
 state, splice output, and emission use LF newlines; a bare carriage return is
 rejected. Source and AST are shared by reference with every state derived from
 this one via mutating functions; the type system marks every other field as
 immutable.
 
 @param source - TOML text to parse; CRLF is normalized to LF, a bare CR rejected
 
 @param mode - `'splice'` (default) preserves unmutated regions byte-for-byte
               at {@link tomlStringify}; `'canonical'` rebuilds text from the AST
               on every emit.
 
 @param canonical - Partial override of {@link CanonicalOptions}; defaults apply
                    for omitted fields. Used for canonical emission and for
                    canonical re-emission of mutated nodes in splice mode.
 
 @param tomlVersion - Forwarded to `toml-eslint-parser`. Defaults to the
                      parser's default (currently `'1.1'` in 1.0.3).
 
 @returns Fresh {@link TomlEditState} with empty deltas.
 
 @throws {@link TomlEditError} when the parser rejects the input. The original
         `ParseError` is exposed via `cause`.
 
 @example
 ```ts
 import { parseTomlEdit, tomlStringify } from '\@monochromatic-dev/toml-edit-core';
 
 const edit = parseTomlEdit({ source: 'foo = "bar"\\n', },);
 tomlStringify({ edit, },); // 'foo = "bar"\\n'
 ```
 */
export function parseTomlEdit(
  {
    source,
    mode = 'splice',
    canonical,
    tomlVersion,
  }: TomlEditOptions,
): TomlEditState {
  /**
   Source with CRLF normalized to LF; held by the state so every byte range,
   splice, and re-emission reasons about LF newlines only.
   */
  const normalizedSource = normalizeNewlines({ source, },);
  /**
   Single parse so the block tree is built once from the parse-time AST.
   */
  const program = safeParse({
    source: normalizedSource,
    tomlVersion,
  },);

  return {
    source: normalizedSource,
    blocks: buildBlocks({
      source: normalizedSource,
      program,
    },),
    comments: program.comments,
    mode,
    canonical: Object.freeze({
      indent: canonical?.indent ?? DEFAULT_CANONICAL_OPTIONS.indent,
      arrayInlineThreshold: canonical?.arrayInlineThreshold
        ?? DEFAULT_CANONICAL_OPTIONS.arrayInlineThreshold,
      arrayInlineMaxColumns: canonical?.arrayInlineMaxColumns
        ?? DEFAULT_CANONICAL_OPTIONS.arrayInlineMaxColumns,
      preferDottedKeysForCreate: canonical?.preferDottedKeysForCreate
        ?? DEFAULT_CANONICAL_OPTIONS.preferDottedKeysForCreate,
      trailingNewline: canonical?.trailingNewline
        ?? DEFAULT_CANONICAL_OPTIONS.trailingNewline,
    },),
  };
}
