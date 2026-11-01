/**
 Structure-aware mutators that corrupt valid TOML into near-miss inputs.
 
 Pure random bytes rarely reach the parser's interesting error paths; they are
 rejected in the first few tokens. Starting from a valid document and applying
 one local corruption (truncation, a deleted scalar, an injected delimiter, a
 duplicated line, a swapped pair) produces inputs that look almost-valid and
 exercise the recovery and rejection logic far deeper. The result is fed to the
 totality property, which asserts every such input either parses to a state or
 throws `TomlEditError`, never an unwrapped error.
 
 Every transform is a single linear pass over the string; none rescans or
 rebuilds the text quadratically.
 
 @module
 */

import {
  type Arbitrary,
  constantFrom,
  double,
  record,
} from 'fast-check';

import { documentArbitrary, } from './arb-documents.ts';

/**
 Disruptive tokens injected mid-document to unbalance structure or strings.
 */
const DISRUPTIVE_TOKENS: readonly string[] = [
  '[',
  ']',
  '{',
  '}',
  '"',
  "'",
  '=',
  '\n',
  '\u0000',
  '#',
  '\\',
  '.',
];

/**
 Map a `[0, 1)` fraction to an index into a string of `length`.
 
 @returns Clamped integer index in `[0, length]`.
 */
function indexFor({
  fraction,
  length,
}: {
  readonly fraction: number;
  readonly length: number
},): number {
  return Math.min(
    length,
    Math.max(
      0,
      Math.floor(fraction * (length + 1),),
    ),
  );
}

/**
 Truncate `source` at a fraction of its length.
 
 @returns Prefix of `source`, often leaving an unterminated construct.
 
 @example
 ```ts
 truncate({ source: 'a = 1\n', fraction: 0.5, },); // 'a =' (approximately)
 ```
 */
export function truncate({
  source,
  fraction,
}: {
  readonly source: string;
  readonly fraction: number
},): string {
  return source.slice(
    0,
    indexFor({
      fraction,
      length: source.length,
    },),
  );
}

/**
 Delete the scalar at a fraction of `source`.
 
 @returns `source` with one character removed (empty input unchanged).
 
 @example
 ```ts
 deleteAt({ source: 'a = 1\n', fraction: 0.5, },); // 'a  1\n' (approximately)
 ```
 */
export function deleteAt({
  source,
  fraction,
}: {
  readonly source: string;
  readonly fraction: number
},): string {
  if (source.length === 0) return source;
  /**
   Index of the character to drop, kept inside bounds.
   */
  const at = Math.min(
    source.length - 1,
    indexFor({
      fraction,
      length: source.length - 1,
    },),
  );
  return `${source.slice(
    0,
    at,
  )}${source.slice(at + 1,)}`;
}

/**
 Insert `token` at a fraction of `source`.
 
 @returns `source` with `token` spliced in.
 
 @example
 ```ts
 insertToken({ source: 'a = 1\n', fraction: 1, token: '[', },); // 'a = 1\n['
 ```
 */
export function insertToken(
  {
    source,
    fraction,
    token,
  }: {
    readonly source: string;
    readonly fraction: number;
    readonly token: string;
  },
): string {
  /**
   Splice index for the token.
   */
  const at = indexFor({
    fraction,
    length: source.length,
  },);
  return `${source.slice(
    0,
    at,
  )}${token}${source.slice(at,)}`;
}

/**
 Duplicate the line at a fraction of `source`, a common duplicate-key source.
 
 @returns `source` with one line repeated.
 
 @example
 ```ts
 duplicateLine({ source: 'a = 1\nb = 2\n', fraction: 0, },); // 'a = 1\na = 1\nb = 2\n'
 ```
 */
export function duplicateLine({
  source,
  fraction,
}: {
  readonly source: string;
  readonly fraction: number
},): string {
  /**
   Source split into lines so one can be repeated in place.
   */
  const lines = source.split('\n',);
  if (lines.length === 0) return source;
  /**
   Index of the line to duplicate.
   */
  const at = Math.min(
    lines.length - 1,
    indexFor({
      fraction,
      length: lines.length - 1,
    },),
  );
  return [
    ...lines.slice(
      0,
      at + 1,
    ),
    lines[at] ?? '',
    ...lines.slice(at + 1,),
  ].join('\n',);
}

/**
 Corruption strategy discriminant.
 */
type CorruptionKind = 'truncate' | 'delete' | 'insert' | 'duplicate';

/**
 Apply one corruption strategy to a base document.
 
 @returns The corrupted source text.
 */
function applyCorruption(
  {
    source,
    kind,
    fraction,
    token,
  }: {
    readonly source: string;
    readonly kind: CorruptionKind;
    readonly fraction: number;
    readonly token: string;
  },
): string {
  /**
   Corruption strategy dispatch table keyed by kind.
   */
  const strategies: Record<CorruptionKind, () => string> = {
    truncate: function run() { return truncate({
      source,
      fraction,
    },); },
    delete: function run() { return deleteAt({
      source,
      fraction,
    },); },
    insert: function run() { return insertToken({
      source,
      fraction,
      token,
    },); },
    duplicate: function run() { return duplicateLine({
      source,
      fraction,
    },); },
  };
  return strategies[kind]();
}

/**
 Arbitrary of near-miss documents: a valid base with one local corruption.
 */
export const corruptedDocumentArbitrary: Arbitrary<string> = record({
  source: documentArbitrary,
  kind: constantFrom(
    'truncate',
    'delete',
    'insert',
    'duplicate',
  ) as Arbitrary<CorruptionKind>,
  fraction: double({
    min: 0,
    max: 1,
    noNaN: true,
    noDefaultInfinity: true,
  },),
  token: constantFrom(...DISRUPTIVE_TOKENS,),
},)
  .map(function corrupt(plan: {
    readonly source: string;
    readonly kind: CorruptionKind;
    readonly fraction: number;
    readonly token: string;
  },) {
    return applyCorruption(plan,);
  },);
