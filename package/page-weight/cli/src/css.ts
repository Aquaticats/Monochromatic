/**
 CSS asset reference extraction.
 
 Scans CSS source for `url(...)` references using the CSS Syntax Level 3
 tokenizer from `@csstools/css-tokenizer`. The tokenizer is used directly
 (rather than a full AST parser) because the only structure we need is
 the flat list of URLs the stylesheet references.
 */
import {
  type CSSToken,
  isTokenAtKeyword,
  isTokenFunction,
  isTokenString,
  isTokenURL,
  isTokenWhiteSpaceOrComment,
  tokenize,
} from '@csstools/css-tokenizer';
import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import { startsWithUriScheme, } from './url-detect.ts';

/**
 Tuple index of the "extra data" field in a CSS token.
 
 The tokenizer uses a tuple shape `[type, raw, start, end, data]`; index `4`
 is the structured payload (e.g. `{ value: string }` for URL/String/Function
 tokens). Exposing it as a named constant avoids repeated magic-number use.
 */
const TOKEN_DATA_INDEX = 4;

/**
 Minimal readonly view of a value-carrying CSS token.
 
 The four token kinds this module reads (`Url`, `String`, `Function`,
 `AtKeyword`) all expose their unescaped payload at tuple index
 {@link TOKEN_DATA_INDEX}. Upstream token types extend mutable `Array`,
 while this observer needs only an immutable indexed view of that payload.
 Every concrete token type remains structurally assignable to the narrower view.
 */
type ValueToken =
  & readonly unknown[]
    & Readonly<Record<typeof TOKEN_DATA_INDEX, { readonly value: string; }>>;

/**
 Reads the structured `value` payload from a URL / String / Function /
 AtKeyword token at the {@link TOKEN_DATA_INDEX} tuple slot.
 
 @param token - tokenizer output tuple
 
 @returns token's unescaped string payload
 */
function tokenValue(token: ValueToken,): string {
  return token[TOKEN_DATA_INDEX]
    .value;
}

/**
 Sentinel returned by {@link localUrlOrAbsent} when a CSS URL value is not a
 local reference: external schemes, protocol-relative, fragment-only, or
 empty. A `unique symbol`; callers narrow with `=== NON_LOCAL_REF`.
 */
const NON_LOCAL_REF: unique symbol = Symbol('page-weight/non-local-ref',);

/**
 Returns the local reference carried by a raw CSS URL value, or
 {@link NON_LOCAL_REF} when the value is external and contributes no local
 weight.
 
 External forms filtered:
 - absolute URLs with scheme (`http:`, `https:`, `ftp:`, `data:`, ...),
   detected via {@link startsWithUriScheme}
 - protocol-relative (`//cdn.example.com/...`)
 - fragment-only (`#id`)
 - empty string
 
 @param raw - URL string as it appeared in CSS
 
 @returns trimmed local reference, or {@link NON_LOCAL_REF}
 */
function localUrlOrAbsent(raw: string,): string | typeof NON_LOCAL_REF {
  /**
   URL with surrounding whitespace removed; raw CSS values often carry stray padding.
   */
  const trimmed = raw.trim();
  if ((trimmed === '') || trimmed
    .startsWith('#',))
    return NON_LOCAL_REF;
  if (trimmed.startsWith('//',)
    || startsWithUriScheme(trimmed,))
    return NON_LOCAL_REF;
  return trimmed;
}

/**
 Sentinel returned by the `nextSemanticToken` scanner inside
 {@link extractCssUrls} when the token stream ends before a non-whitespace,
 non-comment token is found. A `unique symbol`; callers narrow with
 `=== NO_MORE_TOKENS`.
 */
const NO_MORE_TOKENS: unique symbol = Symbol('page-weight/no-more-tokens',);

/**
 Extracts every `url(...)` reference from a CSS source string.
 
 Handles both forms the tokenizer recognises:
 - `url(path)` / `url("path")` / `url('path')`: tokenised as a single `Url` token
 - `url("path")` with whitespace or escapes; tokenised as `Function(url)` followed
   by a `String` token and then a `CloseParen`
 
 `@import` URLs emit a bare `String` token preceded by `@import`; those are
 also captured so imported stylesheets get walked.
 
 External references (`http://`, `https://`, `//`, `data:`) and fragment-only
 references (`#anchor`) are filtered out since they don't contribute to
 local transfer weight.
 
 @param source - raw CSS text
 
 @returns unique asset reference strings as they appeared in the source
 
 @example
 ```ts
 extractCssUrls(`@font-face { src: url('../inter.woff2'); }`);
 // ['../inter.woff2']
 ```
 */
export function extractCssUrls(source: string,): string[] {
  /**
   Full token stream produced by the CSS tokenizer; walked once below.
   */
  const tokens: CSSToken[] = tokenize({ css: source, },);
  /**
   Output set; deduplicates references seen multiple times across the stylesheet.
   */
  const refs = new Set<string>();

  /**
   Returns the next token at or after `startIndex` that is not whitespace or
   a comment, or {@link NO_MORE_TOKENS} when the stream ends first. Closes
   over `tokens` so the shared stream is not threaded through a parameter.
   
   @param startIndex - index to begin scanning from (inclusive)
   
   @returns next semantic token, or {@link NO_MORE_TOKENS} past the end
   */
  function nextSemanticToken(startIndex: number,): CSSToken | typeof NO_MORE_TOKENS {
    for (let loopIndex = startIndex; loopIndex < tokens
      .length; loopIndex += 1) {
      /**
       Current token under inspection; skipped if it carries no semantic content.
       */
      const token = nonNullishOrThrow(tokens[loopIndex],);
      if (!isTokenWhiteSpaceOrComment(token,))
        return token;
    }
    return NO_MORE_TOKENS;
  }

  /**
   Filters `raw` through {@link localUrlOrAbsent} and records the reference
   when it is local. Closes over `refs`.
   
   @param raw - URL string as it appeared in CSS
   */
  function addLocalRef(raw: string,): void {
    /**
     Local reference carried by `raw`, or `NON_LOCAL_REF` when external.
     */
    const local = localUrlOrAbsent(raw,);
    if (local !== NON_LOCAL_REF)
      refs.add(local,);
  }

  for (let loopIndex = 0; loopIndex < tokens
    .length; loopIndex += 1) {
    /**
     Current token in the linear scan; dispatch below depends on its kind.
     */
    const token = nonNullishOrThrow(tokens[loopIndex],);
    if (isTokenURL(token,)) {
      addLocalRef(tokenValue(token,),);
      continue;
    }
    if (isTokenFunction(token,)) {
      if (tokenValue(token,)
        .toLowerCase()
        !== 'url')
        continue;
      /**
       First semantic token after `url(`; expected to be the quoted URL string.
       */
      const next = nextSemanticToken(loopIndex + 1,);
      if ((next !== NO_MORE_TOKENS) && isTokenString(next,))
        addLocalRef(tokenValue(next,),);
      continue;
    }
    if (isTokenAtKeyword(token,)) {
      if (tokenValue(token,)
        .toLowerCase()
        !== 'import')
        continue;
      /**
       First semantic token after `@import`; expected to be the imported stylesheet URL.
       */
      const next = nextSemanticToken(loopIndex + 1,);
      if ((next !== NO_MORE_TOKENS) && isTokenString(next,))
        addLocalRef(tokenValue(next,),);
    }
  }

  return [...refs,];
}
