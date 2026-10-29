/**
 {@link normalizeTableHeaderKeyQuoting}: rewrite literal-quoted table-header keys
 as basic-quoted strings.
 
 @module
 */

import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import type { AST, } from 'toml-eslint-parser';

import { escapeBasicSingleLine, } from './basic-escape.ts';
import { safeParse, } from './safe-parse.ts';

//region Replacement collection

/**
 One source span replaced by canonical key text: the exact range of a
 literal-quoted key segment and its basic-quoted encoding.
 */
type KeyQuotingReplacement = {
  readonly start: number;
  readonly end: number;
  readonly text: string;
};

/**
 Collects one replacement per literal-quoted key segment in the table headers of
 a parsed document, in source order.
 
 Bare and basic-quoted keys are left alone because they are already in canonical
 spelling. Every header is reachable from the program body: TOML table bodies
 hold key-values only, so nested tables appear as sibling top-level items.
 
 @param program - root parse result whose foreign ownership is known at the parse boundary
 
 @returns replacements in source order, empty when every header key is canonical
 */
function collectKeyQuotingReplacements(
  { program, }: { readonly program: ForeignBorrowed<AST.TOMLProgram>; },
): KeyQuotingReplacement[] {
  /**
   Single top-level body the parser always emits for a whole document.
   */
  const topLevel = nonNullishOrThrow(program.body[0],);
  /**
   Replacements from every table header in the body.
   */
  const replacements: KeyQuotingReplacement[] = [];
  for (const item of topLevel.body) {
    if (item.type !== 'TOMLTable') continue;
    /**
     Key segments of this table header; dotted keys stay a segment list.
     */
    const { keys, } = item.key;
    for (const key of keys) {
      if ((key.type !== 'TOMLQuoted') || (key.style !== 'literal')) continue;
      /**
       Same key text, escaped and re-quoted as a basic string.
       */
      const quoted = `"${escapeBasicSingleLine({ value: key.value, },)}"`;
      replacements.push({
        start: key.range[0],
        end: key.range[1],
        text: quoted,
      },);
    }
  }
  return replacements;
}

//endregion Replacement collection

//region Substitution

/**
 Applies ordered, non-overlapping replacements to `source` in one pass.
 
 @param source - source text the replacement spans index into
 
 @param replacements - spans ordered by increasing start offset
 
 @returns text with every span substituted and all other bytes preserved
 */
function applyKeyQuotingReplacements(
  {
    source,
    replacements,
  }: {
    readonly source: string;
    readonly replacements: readonly KeyQuotingReplacement[];
  },
): string {
  /**
   Emitted chunks: verbatim text between replacements, then each replacement text.
   */
  const parts: string[] = [];
  /**
   End offset of the last emitted verbatim chunk.
   */
  let cursor = 0;
  for (const replacement of replacements) {
    parts.push(
      source.slice(
        cursor,
        replacement.start,
      ),
      replacement.text,
    );
    cursor = replacement.end;
  }
  parts.push(source.slice(cursor,),);
  /**
   Source text with each replacement span substituted.
   */
  const text = parts.join('',);
  return text;
}

//endregion Substitution

/**
 Rewrites every literal-quoted (single-quoted) key segment in a table header as a
 basic-quoted (double-quoted) string, changing no other bytes.
 
 The transformation is a semantic no-op for TOML: both spellings denote the same
 key. Bare keys and keys already written as basic strings stay exactly as
 written, so the result is idempotent and the diff is limited to the segments
 whose quote style changed. Key-value keys, comments, indentation, line endings,
 and table order are preserved because only header key segments are spliced.
 
 @param source - TOML text to normalize
 
 @returns text with every single-quoted table-header key rewritten as a basic string
 
 @throws {@link TomlEditError} when `source` is not valid TOML
 
 @example
 ```ts
 normalizeTableHeaderKeyQuoting({ source: "[tasks.'lint:types']\nextends = 'lint:types'\n", },);
 // '[tasks."lint:types"]\nextends = 'lint:types'\n'
 ```
 */
export function normalizeTableHeaderKeyQuoting(
  { source, }: { readonly source: string; },
): string {
  /**
   Parsed program; its single top-level body holds every header and key-value.
   */
  const program = safeParse({
    source,
    tomlVersion: undefined,
  },);
  /**
   Replacements in source order; empty when every header key is already canonical.
   */
  const replacements = collectKeyQuotingReplacements({ program, },);
  if (replacements.length === 0) return source;
  return applyKeyQuotingReplacements({
    source,
    replacements,
  },);
}
