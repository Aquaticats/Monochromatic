/**
 Small rendering helpers shared across locale renderers.
 
 Keep functions in this module locale-agnostic; anything tied to one
 locale's grammar belongs in that locale's module.
 
 @module
 */

import type { Capitalization, } from './grammar-primitives.ts';

/**
 Identifiers that must never have their first character recased by
 {@link applyCapitalization}.
 
 English `I` is the canonical example: capitalizing it is correct
 regardless of position, and lowercasing it would surface as `i`. The
 set is populated by the renderer (English adds `I`); locales that do
 not need the protection pass an empty set.
 */
export type CaseInvariantSet = ReadonlySet<string>;

/**
 Options accepted by {@link applyCapitalization}.
 */
type ApplyCapitalizationOptions = Readonly<{
  readonly text: string;
  readonly mode: Capitalization;
  readonly caseInvariants: ReadonlySet<string>;
}>;

/**
 Inclusive Unicode code-point range used for CJK boundary detection.
 */
type CodePointRange = Readonly<{
  /**
   First code point included in the range.
   */
  readonly start: number;
  /**
   Last code point included in the range.
   */
  readonly end: number;
}>;

/**
 CJK ranges whose adjacent token boundaries should not receive ASCII spaces.
 */
const CJK_CODE_POINT_RANGES: readonly CodePointRange[] = [
  {
    start: 0x4E_00,
    end: 0x9F_FF,
  },
  {
    start: 0x34_00,
    end: 0x4D_BF,
  },
  {
    start: 0xF9_00,
    end: 0xFA_FF,
  },
  {
    start: 0x02_00_00,
    end: 0x02_FA_1F,
  },
  {
    start: 0x30_00,
    end: 0x30_3F,
  },
  {
    start: 0x30_40,
    end: 0x30_FF,
  },
];

/**
 Checks whether a code point belongs to the CJK ranges that suppress boundary spaces.
 
 @param options - code point wrapped for named-parameter calls
 
 @returns whether code point is treated as CJK at token boundaries
 */
function isCjkCodePoint(
  options: {
    readonly codePoint: number;
  },
): boolean {
  /**
   Code point tested against every configured CJK range.
   */
  const { codePoint, } = options;
  return CJK_CODE_POINT_RANGES.some(function rangeContainsCodePoint(
    range,
  ): boolean {
    return (codePoint >= range.start) && (codePoint <= range.end);
  },);
}

/**
 Reads first code point from a non-empty token.
 
 @param options - token wrapped for named-parameter calls
 
 @returns first Unicode code point
 
 @throws when called with empty token
 */
function firstCodePointOf(
  options: {
    readonly token: string;
  },
): number {
  /**
   Token whose first code point is read.
   */
  const { token, } = options;
  /**
   First code point from token.
   */
  const codePoint = token.codePointAt(0,);
  if (codePoint === undefined)
    throw new Error('Cannot read first code point from empty token.',);
  return codePoint;
}

/**
 Reads last code point from a non-empty token.
 
 @param options - token wrapped for named-parameter calls
 
 @returns last Unicode code point
 
 @throws when called with empty token
 */
function lastCodePointOf(
  options: {
    readonly token: string;
  },
): number {
  /**
   Token whose last code point is read.
   */
  const { token, } = options;
  /**
   Token split into Unicode code-point strings so astral CJK ranges stay intact.
   */
  const characters = Array.from(token,);
  /**
   Final character string from token.
   */
  const finalCharacter = characters.at(-1,);
  if (finalCharacter === undefined)
    throw new Error('Cannot read last code point from empty token.',);
  /**
   Code point for final character.
   */
  const codePoint = finalCharacter.codePointAt(0,);
  if (codePoint === undefined)
    throw new Error('Cannot read code point from final token character.',);
  return codePoint;
}

/**
 Chooses boundary separator between two already-present render tokens.
 
 @param options - left and right tokens wrapped for named-parameter calls
 
 @returns empty separator for adjacent CJK boundaries, otherwise ASCII space
 */
function separatorForBoundary(
  options: {
    readonly leftToken: string;
    readonly rightToken: string;
  },
): '' | ' ' {
  /**
   Code point at end of left token.
   */
  const leftCodePoint = lastCodePointOf({ token: options.leftToken, },);
  /**
   Code point at start of right token.
   */
  const rightCodePoint = firstCodePointOf({ token: options.rightToken, },);
  if (isCjkCodePoint({ codePoint: leftCodePoint, },)
    && isCjkCodePoint({ codePoint: rightCodePoint, },))
    return '';
  return ' ';
}

/**
 Applies a {@link Capitalization} mode to a rendered string.
 
 Only the first character of the first token is touched. Tokens whose
 exact surface matches a member of `caseInvariants` (e.g. English `I`)
 are preserved as-is.
 
 @param options - text, capitalization mode, and case-invariant token set
 
 @returns capitalized string
 
 @example
 ```ts
 applyCapitalization({ text: 'the cat', mode: 'firstLetter', caseInvariants: new Set() }); // 'The cat'
 applyCapitalization({ text: 'I run',   mode: 'firstLetter', caseInvariants: new Set(['I']) }); // 'I run'
 ```
 */
export function applyCapitalization(
  options: ApplyCapitalizationOptions,
): string {
  /**
   Options destructured once so the branch logic reads like the rendered operation.
   */
  const {
    text,
    mode,
    caseInvariants,
  } = options;
  if (mode === 'preserve')
    return text;
  if (text.length
    === 0)
    return text;
  /**
   First whitespace-delimited token, used to consult `caseInvariants`.
   */
  const firstSpace = text.indexOf(' ',);
  /**
   Substring covering the first token only.
   */
  const firstToken = firstSpace === (-1) ? text : text.slice(
    0,
    firstSpace,
  );
  if (caseInvariants.has(firstToken,))
    return text;
  /**
   First Unicode code point of the rendered text.
   */
  const firstChar = text.charAt(0,);
  return firstChar.toUpperCase()
    + text
      .slice(1,);
}

/**
 Concatenates tokens with pangu-style boundary spacing, dropping empty entries.
 
 Renderers typically emit a token list per sentence component and then
 join them with this helper so a missing optional slot (no adverbials,
 no object) does not surface as a doubled space. Adjacent CJK boundaries
 join directly; other token boundaries use one ASCII space, preserving
 digit/classifier spacing that already exists inside a token.
 
 @param tokens - ordered token list; empty-string entries are skipped
 
 @returns boundary-joined string
 
 @example
 ```ts
 joinTokens(['Do', 'I', 'have', '1 cat']); // 'Do I have 1 cat'
 joinTokens(['Save', '', 'now']);           // 'Save now'
 joinTokens(['我', '有', '1 只猫']);          // '我有 1 只猫'
 ```
 */
export function joinTokens(tokens: readonly string[],): string {
  /**
   Tokens that render visible content and participate in boundary decisions.
   */
  const presentTokens = tokens.filter(function isPresent(token: string,): boolean {
    return token !== '';
  },);
  /**
   Per-token segments prefixed with boundary separators after the first token.
   */
  const segments = presentTokens.map(function tokenSegment(
    token: string,
    tokenIndex: number,
  ): string {
    if (tokenIndex === 0)
      return token;
    /**
     Previous non-empty token, required because the current token is not first.
     */
    const previousToken = presentTokens.at(tokenIndex - 1,);
    if (previousToken === undefined)
      throw new Error('Cannot join token without previous boundary token.',);
    return `${separatorForBoundary({
      leftToken: previousToken,
      rightToken: token,
    },)}${token}`;
  },);
  return segments.join('',);
}
