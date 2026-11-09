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
   Working copy threaded through the three rewrite passes below.
   */
  let out = pattern;
  /* oxlint-disable no-restricted-syntax/no-regex -- both passes are one anchored sweep of a single upstream rule body: the first matches the literal `(?P<` prefix plus one identifier class, the second one quantified character followed by `?`. Neither nests a quantifier over a group, both run once over a body bounded by the vendored config, so each scan is linear and cannot backtrack. Issues 823 and 824 replace them with cursor walks. */
  // Drop named-capture syntax: `(?P<name>...)` -> `(?:...)`. Resharp
  // parses the named form but has no captures; making them non-capturing
  // is the cleanest signal of intent.
  out = out.replaceAll(
    /\(\?P<[A-Za-z_][A-Za-z0-9_]*>/gu,
    '(?:',
  );
  // Strip the trailing `?` in lazy quantifier suffixes: `*?` `+?` `??`
  // and `}?` (the latter closes `{n}?`/`{n,m}?`/`{n,}?`). Resharp uses
  // leftmost-longest semantics and errors on explicit lazy markers; the
  // greedy form is the closest available behavior.
  // Group-prefix forms `(?:`, `(?=`, `(?!`, `(?<=`, `(?<!` do not
  // match this pattern: the `?` there sits directly after `(`, never
  // after `* + ? }`.
  out = out.replaceAll(
    /(?<quantifier>[*+?}])\?/gu,
    '$<quantifier>',
  );
  /* oxlint-enable no-restricted-syntax/no-regex */
  // Escape resharp-only meta characters (`~`, `_`, `&`) that PCRE treats
  // as literal.
  out = escapeResharpOnlyMeta({ pattern: out, },);
  return out;
}
