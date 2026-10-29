/**
 Generic oxlint output augmentation for configured diagnostic guidance.
 
 This module provides pure functions that scan oxlint text output, detect
 diagnostic boundaries, and inject guidance resolved by the guidance
 configuration module.
 
 @module
 */

import {
  NO_DIAGNOSTIC_GUIDANCE,
  resolveDiagnosticGuidance,
} from './oxlint-guidance.ts';

//region Character predicates

/**
 ASCII code 27 (0x1B) for the ESC byte that opens every ANSI control sequence; named to satisfy `no-magic-numbers`.
 */
const ESC_CODE_POINT = 0x1B;
/**
 ESC byte string; located via `indexOf(ESC_CHAR, ...)` instead of a regex.
 */
const ESC_CHAR = String.fromCodePoint(ESC_CODE_POINT,);

/**
 Tests whether `c` is an ASCII digit (`0`-`9`).
 
 @param c - single character
 
 @returns whether `c` is an ASCII digit
 
 @example
 ```ts
 isAsciiDigit('7'); // true
 isAsciiDigit('a'); // false
 ```
 */
function isAsciiDigit(c: string,): boolean {
  return (c >= '0') && (c <= '9');
}

/**
 Tests whether `c` is an ASCII letter (`a`-`z` or `A`-`Z`).
 
 @param c - single character
 
 @returns whether `c` is an ASCII letter
 
 @example
 ```ts
 isAsciiLetter('a'); // true
 isAsciiLetter('1'); // false
 ```
 */
function isAsciiLetter(c: string,): boolean {
  return ((c >= 'a') && (c <= 'z'))
    || ((c >= 'A') && (c <= 'Z'));
}

/**
 Tests whether `c` qualifies as a `\w` word char (alphanumeric or `_`).
 
 @param c - single character
 
 @returns whether `c` is a word char
 
 @example
 ```ts
 isWordChar('_'); // true
 isWordChar('-'); // false
 ```
 */
function isWordChar(c: string,): boolean {
  return isAsciiLetter(c,)
    || isAsciiDigit(c,)
    || (c === '_');
}

/**
 Tests whether `c` is permitted inside an oxlint rule name (`[\w-]`).
 
 @param c - single character
 
 @returns whether `c` may appear inside a rule name
 
 @example
 ```ts
 isRuleNameChar('-'); // true
 isRuleNameChar('('); // false
 ```
 */
function isRuleNameChar(c: string,): boolean {
  return isWordChar(c,)
    || (c === '-');
}

/**
 Tests whether `c` is `\s` whitespace (space, tab, newline, return,
 form feed, vertical tab).
 
 @param c - single character
 
 @returns whether `c` is whitespace
 
 @example
 ```ts
 isWhitespace(' '); // true
 isWhitespace('x'); // false
 ```
 */
function isWhitespace(c: string,): boolean {
  return (c === ' ')
    || (c === '\t')
    || (c === '\n')
    || (c === '\r')
    || (c === '\f')
    || (c === '\v');
}

//endregion Character predicates

//region ANSI handling

/**
 Returns the length of a valid ANSI CSI sequence starting at
 `text[start]`, or `-1` when no valid sequence begins there.
 
 Mirrors the shape of `/\x1B\[\d+(?:;\d+)*m/` with a linear scan:
 after `ESC[`, accept one or more digits, optionally followed by
 `;<digits>` runs, ending at `m`.
 
 @param text - input string
 
 @param start - cursor at the candidate ESC byte
 
 @returns matched length, or `-1` for no match
 
 @example
 ```ts
 ansiEscapeLength({ text: '\x1B[31m', start: 0 }); // 5
 ansiEscapeLength({ text: 'plain', start: 0 });    // -1
 ```
 */
function ansiEscapeLength({
  text,
  start,
}: {
  readonly text: string;
  readonly start: number;
},): number {
  if (text.charAt(start,)
    !== ESC_CHAR)
    return -1;
  if (text.charAt(start + 1,)
    !== '[')
    return -1;

  return (function scan(): number {
    /**
     Cursor over the parameter bytes; the sequence requires at least one digit before any terminator.
     */
    let idx = start + 2;
    if (!isAsciiDigit(text.charAt(idx,),))
      return -1;
    idx += 1;
    while (idx < text
      .length) {
      /**
       Current char; drives the digit / terminator / separator transition.
       */
      const c = text.charAt(idx,);
      if (isAsciiDigit(c,)) {
        idx += 1;
        continue;
      }
      if (c === 'm')
        return (idx - start) + 1;
      if (c === ';') {
        // `;` opens another parameter run, which must begin with a digit.
        if (!isAsciiDigit(text.charAt(idx + 1,),))
          return -1;
        idx += 2;
        continue;
      }
      return -1;
    }
    return -1;
  })();
}

/**
 Strips ANSI escape codes from a string for reliable text matching.
 
 Invalid ESC bytes (e.g. non-CSI escapes) are preserved so the caller
 can still see them; only sequences matched by {@link ansiEscapeLength}
 (`\x1B[<digits>(;<digits>)*m`) are stripped.
 
 @param text - string potentially containing ANSI escape sequences
 
 @returns plain text with all ANSI codes removed
 
 @example
 ```ts
 stripAnsi('\x1B[31merror\x1B[0m');
 // 'error'
 ```
 
 @internal
 */
export function stripAnsi(text: string,): string {
  /**
   Output segments in order; joined once at the end so no intermediate string is recopied per chunk.
   */
  const parts: string[] = [];
  // Single forward pass; `idx` jumps by whole ANSI sequences, so the stride is variable and updated in the body.
  for (let cursorIndex = 0; cursorIndex < text
    .length;) {
    /**
     Position of the next ESC byte; `-1` means no further ANSI sequences.
     */
    const escIdx = text.indexOf(
      ESC_CHAR,
      cursorIndex,
    );
    if (escIdx === (-1)) {
      parts.push(text.slice(cursorIndex,),);
      break;
    }
    /**
     Length of the ANSI sequence at `escIdx`, or `-1` when invalid.
     */
    const escLen = ansiEscapeLength({
      text,
      start: escIdx,
    },);
    if (escLen === (-1)) {
      // Invalid escape: keep the ESC byte verbatim and resume just past it.
      parts.push(text.slice(
        cursorIndex,
        escIdx + 1,
      ),);
      cursorIndex = escIdx + 1;
    }
    else {
      // Valid ANSI sequence: keep the text before it and drop the sequence itself.
      parts.push(text.slice(
        cursorIndex,
        escIdx,
      ),);
      cursorIndex = escIdx + escLen;
    }
  }
  return parts.join('',);
}

//endregion ANSI handling

//region Diagnostic detection

/**
 Sentinel returned by {@link extractRuleName} and {@link matchHeaderAt} when a
 line is not a diagnostic header.
 
 A unique `Symbol` keeps the absent case out of the `string` domain (a rule
 name is always a non-empty string), so callers distinguish "no rule" by
 identity instead of a nullish union.
 
 @example
 ```ts
 extractRuleName('context line') === NO_RULE // true
 ```
 
 @internal
 */
export const NO_RULE: unique symbol = Symbol('diagnostic_header_absent',);

/**
 Walks consecutive whitespace chars starting at `idx`, returning the
 first non-whitespace position (or `text.length`).
 
 @param text - input string
 
 @param idx - cursor at the candidate whitespace
 
 @returns first non-whitespace position
 
 @example
 ```ts
 skipWhitespace({ text: '   x', idx: 0 }); // 3
 skipWhitespace({ text: 'x', idx: 0 });    // 0
 ```
 */
function skipWhitespace({
  text,
  idx,
}: {
  readonly text: string;
  readonly idx: number;
},): number {
  return (function walk(): number {
    /**
     Cursor advanced across the whitespace run; stops at the first non-whitespace char or the end of `text`.
     */
    let cursor = idx;
    while ((cursor < text
      .length) && isWhitespace(text.charAt(cursor,),))
      cursor += 1;
    return cursor;
  })();
}

/**
 Walks consecutive rule-name chars starting at `idx`, returning the
 exclusive end of the run.
 
 @param text - input string
 
 @param idx - cursor at the candidate rule-name char
 
 @returns exclusive end of the rule-name run
 
 @example
 ```ts
 skipRuleNameChars({ text: 'no-magic-numbers)', idx: 0 }); // 16
 skipRuleNameChars({ text: ')', idx: 0 });                 // 0
 ```
 */
function skipRuleNameChars({
  text,
  idx,
}: {
  readonly text: string;
  readonly idx: number;
},): number {
  return (function walk(): number {
    /**
     Cursor advanced across the rule-name char run; stops at the first non-rule-name char or the end of `text`.
     */
    let cursor = idx;
    while ((cursor < text
      .length) && isRuleNameChar(text.charAt(cursor,),))
      cursor += 1;
    return cursor;
  })();
}

/**
 Tests whether every char in `[start, end)` is non-whitespace.
 
 @param text - input string
 
 @param start - inclusive start
 
 @param end - exclusive end
 
 @returns whether the range is non-empty and contains no whitespace
 
 @example
 ```ts
 allNonWhitespaceBetween({ text: 'abc def', start: 0, end: 3 }); // true
 allNonWhitespaceBetween({ text: 'a c', start: 0, end: 3 });     // false
 ```
 */
function allNonWhitespaceBetween({
  text,
  start,
  end,
}: {
  readonly text: string;
  readonly start: number;
  readonly end: number;
},): boolean {
  return (function check(): boolean {
    /**
     Cursor scanned across `[start, end)`; any whitespace short-circuits to false.
     */
    let cursor = start;
    while (cursor < end) {
      if (isWhitespace(text.charAt(cursor,),))
        return false;
      cursor += 1;
    }
    return true;
  })();
}

/**
 Attempts to parse an oxlint diagnostic header anchored at the `x` or
 `!` punctuation char.
 
 Mirrors `/[x!]\s+\S+\(([\w-]+)\)\s*:/` (anchored at the punctuation):
 whitespace (via {@link skipWhitespace}), plugin name (validated by
 {@link allNonWhitespaceBetween}), `(`, rule name (via {@link skipRuleNameChars}),
 `)`, optional whitespace, `:`.
 
 @param text - input string (typically ANSI-stripped)
 
 @param punctIdx - cursor at the candidate `x` or `!`
 
 @returns captured rule name, or {@link NO_RULE} when no match
 
 @example
 ```ts
 matchHeaderAt({ text: 'x foo(bar): baz', punctIdx: 0 }); // 'bar'
 matchHeaderAt({ text: 'x foo: baz', punctIdx: 0 });      // NO_RULE
 ```
 */
function matchHeaderAt({
  text,
  punctIdx,
}: {
  readonly text: string;
  readonly punctIdx: number;
},): string | typeof NO_RULE {
  if (!isWhitespace(text.charAt(punctIdx + 1,),))
    return NO_RULE;

  /**
   Cursor past the run of whitespace following the punctuation.
   */
  const afterWs = skipWhitespace({
    text,
    idx: punctIdx + 2,
  },);

  /**
   Position of the `(` that opens the rule name; bounds the plugin-name run.
   */
  const openParen = text.indexOf(
    '(',
    afterWs,
  );
  if ((openParen === (-1)) || (openParen === afterWs))
    return NO_RULE;
  if (
    !allNonWhitespaceBetween({
      text,
      start: afterWs,
      end: openParen,
    },)
  ) {
    return NO_RULE;
  }

  /**
   Cursor at the first char inside the parens; rule name starts here.
   */
  const ruleStart = openParen + 1;
  /**
   Exclusive end of the rule-name char run.
   */
  const ruleEnd = skipRuleNameChars({
    text,
    idx: ruleStart,
  },);
  if (ruleEnd === ruleStart)
    return NO_RULE;

  if (text.charAt(ruleEnd,)
    !== ')')
    return NO_RULE;

  /**
   Cursor past optional whitespace after `)`; the next char must be `:`.
   */
  const afterRule = skipWhitespace({
    text,
    idx: ruleEnd + 1,
  },);
  if (text.charAt(afterRule,)
    !== ':')
    return NO_RULE;

  return text.slice(
    ruleStart,
    ruleEnd,
  );
}

/**
 Extracts the rule name from an oxlint diagnostic header line.
 
 Handles ANSI color codes by stripping them before matching each
 candidate position with {@link matchHeaderAt}.
 
 @param line - single line of oxlint output, possibly with ANSI codes
 
 @returns rule name (e.g. `no-misused-promises`), or {@link NO_RULE} if not a diagnostic header
 
 @example
 ```ts
 extractRuleName('  x typescript-eslint(no-misused-promises): Promise-returning ...');
 // 'no-misused-promises'
 
 extractRuleName('  92 |   const form = ...');
 // NO_RULE
 ```
 
 @internal
 */
export function extractRuleName(line: string,): string | typeof NO_RULE {
  /**
   ANSI-stripped working copy; the matcher operates on plain text.
   */
  const stripped = stripAnsi(line,);

  // Single linear pass over the stripped line; attempt a header match at each `x`/`!` candidate.
  for (let cursorIndex = 0; cursorIndex < stripped
    .length; cursorIndex += 1) {
    /**
     Char at the cursor; only `x` or `!` can open a diagnostic header.
     */
    const c = stripped.charAt(cursorIndex,);
    if ((c === 'x') || (c === '!')) {
      /**
       Header-match attempt anchored at the candidate; a rule name on the first valid header, else `NO_RULE`.
       */
      const result = matchHeaderAt({
        text: stripped,
        punctIdx: cursorIndex,
      },);
      if (result !== NO_RULE)
        return result;
    }
  }
  return NO_RULE;
}

/**
 Tests whether a line is a `help:` line from an oxlint diagnostic.
 
 @param line - single line of oxlint output, possibly with ANSI codes
 
 @returns true when the line contains a `help:` prefix
 
 @example
 ```ts
 isHelpLine('  help: Expected void return type.');
 // true
 ```
 
 @internal
 */
export function isHelpLine(line: string,): boolean {
  return stripAnsi(line,)
    .trimStart()
    .startsWith('help:',);
}

//endregion Diagnostic detection

//region Output augmentation

/**
 Indentation prefix for injected guidance lines, matching oxlint's `help:` alignment.
 */
const GUIDANCE_PREFIX = '  ';

/**
 Formats a guidance entry into an indented `note:` line for terminal output.
 
 @param guidance - guidance text
 
 @returns formatted note line ready to inject into oxlint output
 
 @example
 ```ts
 formatGuidanceLine('Wrap async logic in try/catch.');
 // '  note: Wrap async logic in try/catch.'
 ```
 
 @internal
 */
export function formatGuidanceLine(guidance: string,): string {
  return `${GUIDANCE_PREFIX}note: ${guidance}`;
}

/**
 Formats a guidance entry into an indented `help:` line for terminal output.
 
 @param guidance - guidance text
 
 @returns formatted help line ready to inject into oxlint output
 */
function formatHelpLine(guidance: string,): string {
  return `${GUIDANCE_PREFIX}help: ${guidance}`;
}

/**
 Augments oxlint text output with configured diagnostic guidance.
 
 Scans diagnostic headers with {@link extractRuleName}, resolves configured
 guidance via {@link resolveDiagnosticGuidance}, appends it to an existing
 `help:` line when present, and otherwise injects a `help:` line before the
 diagnostic boundary.
 
 Preserves all original output including ANSI codes and formatting.
 
 @param output - raw oxlint stdout or stderr text
 
 @returns augmented text with guidance lines injected
 
 @example
 ```ts
 const augmented = augmentOxlintOutput([
   '  x typescript-eslint(no-misused-promises): Promise-returning function ...',
   '   ,-[src/client.ts:93:30]',
   '   `----',
   '  help: Expected void return type.',
   '',
 ].join('\n'));
 // Contains '  help: Expected void return type. Async callbacks silently drop ...'
 ```
 
 @internal
 */
export function augmentOxlintOutput(output: string,): string {
  if (output.length
    === 0)
    return '';

  /**
   Source output split per line so each diagnostic's header, body, and trailing blank can be inspected individually.
   */
  const lines = output.split('\n',);
  /**
   Output buffer assembled in order; guidance lines are spliced in alongside the originals.
   */
  const result: string[] = [];

  /* oxlint-disable no-restricted-syntax/no-function-root-let -- multi-statement state machine: activeGuidance and injected are mutated by four branches across loop iterations, with side effects on `result`. */
  /**
   Guidance for the current diagnostic block, `NO_RULE` when unmatched.
   */
  let activeGuidance: string | typeof NO_RULE = NO_RULE;
  /**
   Whether guidance has been injected for the current diagnostic block.
   */
  let injected = false;
  /* oxlint-enable no-restricted-syntax/no-function-root-let */

  for (const line of lines) {
    /**
     Rule name extracted from the current line when it matches a diagnostic header; otherwise `NO_RULE`.
     */
    const ruleName = extractRuleName(line,);
    if (ruleName !== NO_RULE) {
      /**
       ANSI-stripped header line supplied to the guidance resolver.
       */
      const strippedLine = stripAnsi(line,);
      /**
       Guidance resolved for this diagnostic header, or a guidance sentinel.
       */
      const guidance = resolveDiagnosticGuidance({
        ruleName,
        strippedHeaderLine: strippedLine,
      },);
      activeGuidance = (guidance === NO_DIAGNOSTIC_GUIDANCE)
        ? NO_RULE
        : guidance;
      injected = false;
    }

    // Append to help line when guidance is pending
    if ((activeGuidance !== NO_RULE) && (!injected)
      && isHelpLine(line,)) {
      result.push(`${line} ${activeGuidance}`,);
      injected = true;
      continue;
    }

    // Inject before blank line (end of diagnostic) if no help line was found
    if ((activeGuidance !== NO_RULE) && (!injected)
      && (stripAnsi(line,)
        .trim()
        === '')) {
      result.push(formatHelpLine(activeGuidance,),);
      injected = true;
      activeGuidance = NO_RULE;
    }

    result.push(line,);
  }

  // Handle trailing diagnostic with no blank line at end
  if ((activeGuidance !== NO_RULE) && (!injected))
    result.push(formatHelpLine(activeGuidance,),);

  return result.join('\n',);
}

//endregion Output augmentation
