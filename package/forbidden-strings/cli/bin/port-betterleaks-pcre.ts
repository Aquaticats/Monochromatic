/**
 Stage one's pattern rewriting: the resharp-only meta escape walker plus the
 PCRE-to-resharp passes.

 Reads one upstream rule body and returns the body the scanner compiles.
 The TOML walker in `port-betterleaks-parse.ts` stores the body and the
 section renderer in `port-betterleaks.ts` converts it through
 {@link pcreToResharp}.
 */

import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

/** Escape resharp-only meta characters that PCRE treats as literal: `~`,
  `_`, `&`. `~` is the resharp complement operator, `_` is the universal
  wildcard, `&` is intersection. Outside a character class these need
  backslash escapes; inside `[...]` they're already literal (resharp's
  class-level operators are doubled, e.g. `[A&&B]`).
 
  The walker tracks whether the cursor is inside `[...]`, skips `\X`
  escape sequences verbatim (so existing escapes are preserved), and
  treats `]` as a class terminator unless it appears in the
  literal-`]` position immediately after `[` or `[^`.
 
  @param pattern - one upstream rule body, still in the PCRE-leaning dialect
 
  @returns the body with every resharp-only meta character escaped
 */
function escapeResharpOnlyMeta({ pattern, }: { readonly pattern: string; },): string {
  /**
   Accumulator for the rewritten pattern, built char-by-char.
   */
  let out = '';
  /**
   Cursor into `pattern`; advanced by one or two chars per iteration.
   */
  let i = 0;
  /**
   Tracks whether the cursor currently sits inside a `[...]` character class.
   */
  let inClass = false;
  /**
   Index where the current class body begins (right after `[` or `[^`).
   
   Used to recognise the literal-`]` position: a `]` at `classBodyStart` is
   a literal character, not the class terminator.
   */
  let classBodyStart = -1;
  /**
   Resharp-only meta characters that need backslash-escaping outside `[...]`.
   */
  const META: ReadonlySet<string> = new Set([
    '~',
    '_',
    '&',
  ],);
  while (i < pattern
    .length) {
    /**
     Current character under the cursor; shorthand to avoid repeating `pattern[i]`.
     */
    const c = nonNullishOrThrow(pattern[i],);
    // Pass an escape sequence through unmodified (consumes two chars).
    if ((c === '\\') && ((i + 1) < pattern
      .length)) {
      out += pattern.slice(
        i,
        i + 2,
      );
      i += 2;
      continue;
    }
    if (!inClass) {
      if (c === '[') {
        inClass = true;
        out += c;
        i += 1;
        // After `[`, an optional `^` belongs to the class header.
        if (pattern[i]
          === '^') {
          out += pattern[i];
          i += 1;
        }
        classBodyStart = i;
        continue;
      }
      if (META.has(c,)) {
        out += `\\${c}`;
        i += 1;
        continue;
      }
      out += c;
      i += 1;
      continue;
    }
    // Inside class.
    if ((c === ']') && (i !== classBodyStart)) {
      inClass = false;
      out += c;
      i += 1;
      continue;
    }
    out += c;
    i += 1;
  }
  return out;
}

//region Pattern rewrites, one walk over the body each
/**
 Characters whose later `?` marks a lazy quantifier, which resharp refuses.
 */
const QUANTIFIER_CHARACTERS: ReadonlySet<string> = new Set([
  '*',
  '+',
  '?',
  '}',
],);

/**
 Whether the character may open a PCRE capture name.
 
 @param character - one character of the body, or the empty string past its end
 
 @returns whether the character is an ASCII letter or an underscore
 */
function isNameStartCharacter({ character, }: { readonly character: string; },): boolean {
  return ((character >= 'a') && (character <= 'z'))
    || ((character >= 'A') && (character <= 'Z'))
    || (character === '_');
}

/**
 Whether the character may continue a PCRE capture name.
 
 @param character - one character of the body, or the empty string past its end
 
 @returns whether the character is an ASCII letter, a digit or an underscore
 */
function isNameCharacter({ character, }: { readonly character: string; },): boolean {
  return isNameStartCharacter({ character, },) || ((character >= '0') && (character <= '9'));
}

/**
 Replace PCRE's named-capture opening, `(?P<name>`, with resharp's non-capturing `(?:`. Resharp parses the named
 form but has no captures, so the non-capturing form is the cleanest signal of intent.
 
 @param pattern - one upstream rule body
 
 @returns the body with every named-capture opening made non-capturing
 
 @example
 ```ts
 namedCaptureGroupsMadeNonCapturing({ pattern: '(?P<key>a)' });
 ```
 */
export function namedCaptureGroupsMadeNonCapturing({ pattern, }: { readonly pattern: string; },): string {
  /**
   The opening the walk looks for.
   */
  const opening = '(?P<';
  /**
   The body with every completed opening already replaced.
   */
  let out = '';
  /**
   Position in `pattern` from which the text is still owed to `out`.
   */
  let from = 0;
  /**
   Last position a candidate opening can start at, since nothing shorter than the opening itself can hold one.
   */
  const lastOpeningStart = pattern.length - opening.length;
  for (let searchFrom = 0; searchFrom <= lastOpeningStart;) {
    /**
     Position of the next candidate opening.
     */
    const at = pattern.indexOf(
      opening,
      searchFrom,
    );
    if (at === (-1))
      break;
    /**
     Position just past the characters that continue the name the opening begins.
     */
    let nameEnd = at + opening.length;
    while (isNameCharacter({ character: pattern.charAt(nameEnd,), },)) nameEnd += 1;
    if (isNameStartCharacter({ character: pattern.charAt(at + opening.length,), },)
      && (pattern.charAt(nameEnd,) === '>')) {
      out += `${pattern.slice(
        from,
        at,
      )  }(?:`;
      from = nameEnd + 1;
      searchFrom = from;
      continue;
    }
    // No name here; a later opening may still be one.
    searchFrom = at + 1;
  }
  out += pattern.slice(from,);
  return out;
}

/**
 Drop the lazy marker of every quantifier suffix: `*?`, `+?`, `??` and `}?` keep the quantifier alone. Resharp uses
 leftmost-longest semantics and errors on an explicit lazy marker, so the greedy form is the closest available
 behavior. The group openings `(?:`, `(?=`, `(?!`, `(?<=` and `(?<!` keep their `?`, which follows `(` and never a
 quantifier character.
 
 @param pattern - one upstream rule body
 
 @returns the body with no lazy marker left
 
 @example
 ```ts
 lazyQuantifierMarkersDropped({ pattern: 'a*?' });
 ```
 */
export function lazyQuantifierMarkersDropped({ pattern, }: { readonly pattern: string; },): string {
  /**
   The body without the markers its quantifiers carried.
   */
  let out = '';
  for (let at = 0; at < pattern
    .length; at += 1) {
    /**
     The character the walk is on, which the body keeps.
     */
    const character = pattern.charAt(at,);
    out += character;
    if (QUANTIFIER_CHARACTERS.has(character,) && (pattern.charAt(at + 1,) === '?')) at += 1;
  }
  return out;
}
//endregion

/**
 Convert a betterleaks-style PCRE regex to resharp-compatible form, finishing
 with {@link escapeResharpOnlyMeta} for the resharp-only meta characters.
 
 @param pattern - one upstream rule body, still in the PCRE-leaning dialect
 
 @returns the same body in the resharp dialect the scanner compiles
 
 @example
 ```ts
 pcreToResharp({ pattern: '(?i)\\b(ghp_[A-Za-z0-9]{36})' });
 ```
 */
export function pcreToResharp({ pattern, }: { readonly pattern: string; },): string {
  /**
   The body with every pass applied, each one walking what the pass before it left.
   */
  let out = namedCaptureGroupsMadeNonCapturing({ pattern, },);
  out = lazyQuantifierMarkersDropped({ pattern: out, },);
  // Escape resharp-only meta characters (`~`, `_`, `&`) that PCRE treats
  // as literal.
  out = escapeResharpOnlyMeta({ pattern: out, },);
  return out;
}
