/**
 Citation predicates for the categorical-dismissal allowlist.
 
 A dismissal accompanied by any of these signals is treated as grounded:
 a file path with a recognised extension, a `:N` line-number suffix, or
 the literal `AGENTS.md`.
 
 @module
 */

import {
  containsWordBoundedPhrase,
  isAlphaNum,
  isDigit,
  isWordChar,
} from '@monochromatic-dev/agent-harness-shared-text-scan/ts';

//region File-path predicate

/**
 Recognised source-file extensions for the citation check.
 */
const FILE_EXTENSIONS: readonly string[] = [
  'ts',
  'tsx',
  'js',
  'jsx',
  'json',
  'md',
  'yaml',
  'yml',
  'toml',
  'rs',
  'py',
];

/**
 Whether `c` may appear in a file-path token (alphanumeric, underscore,
 dot, forward slash, `@`, or `-`). Mirrors the original regex's
 `[\w./@-]` class.
 
 @param c - one-character string to inspect
 
 @returns whether `c` is a valid path-token character
 
 @example
 ```ts
 isPathChar('_'); // true
 isPathChar(':'); // false
 ```
 */
function isPathChar(c: string,): boolean {
  return isAlphaNum(c,)
    || (c === '_')
    || (c === '.')
    || (c === '/')
    || (c === '@')
    || (c === '-');
}

/**
 Whether `line` contains a `.<ext>` token preceded by at least one
 path character. Mirrors the original regex
 `/[\w./@-]+\.(?:ts|tsx|...)\b/i`.
 
 @param line - candidate line of prose
 
 @returns whether a file path with a recognised extension is present
 
 @example
 ```ts
 hasFilePathWithExtension('see package/x/y.ts:42'); // true
 hasFilePathWithExtension('plain prose');            // false
 ```
 */
function hasFilePathWithExtension(line: string,): boolean {
  /**
   Lower-cased line used for case-insensitive extension matches.
   */
  const lower = line.toLowerCase();
  /**
   Checks whether `line` contains `.<ext>` matching the regex's
   boundary semantics.
   
   @param ext - extension without leading dot (e.g. `ts`)
   
   @returns whether the extension appears in a valid path context
   
   @example
   ```ts
   checkExt('ts'); // true when prose contains `<path>.ts<boundary>`
   ```
   */
  function checkExt(ext: string,): boolean {
    /**
     Search token: leading dot plus the extension.
     */
    const token = `.${ext}`;
    // Walk every `.<ext>` occurrence in order (monotonic `indexOf`). A valid
    // path context (path-char prefix, word boundary after the extension)
    // confirms a file path; other occurrences are skipped.
    for (
      let idx = lower.indexOf(
        token,
        0,
      );
      idx !== (-1);
      idx = lower.indexOf(
        token,
        idx + 1,
      )
    ) {
      if (idx === 0)
        continue;
      /**
       Char immediately before the dot; must be a path char per the original regex.
       */
      const before = line.charAt(idx - 1,);
      if (!isPathChar(before,))
        continue;
      /**
       Position one past the extension; checked below for a word boundary.
       */
      const endIdx = idx + token
        .length;
      if ((endIdx < line
        .length) && isWordChar(line.charAt(endIdx,),))
        continue;
      return true;
    }
    return false;
  }
  return FILE_EXTENSIONS.some(function checkExtension(ext,): boolean {
    return checkExt(ext,);
  },);
}

//endregion

//region Line-number predicate

/**
 Minimum number of digits accepted for the line-number suffix (matches `\d{1,5}`'s lower bound).
 */
const LINE_NUMBER_MIN_DIGITS = 1;

/**
 Maximum number of digits accepted for the line-number suffix (matches `\d{1,5}`'s upper bound).
 */
const LINE_NUMBER_MAX_DIGITS = 5;

/**
 Whether `line` contains a `:N` line-number suffix (1-5 digits) at a word
 boundary. Mirrors the original regex `/:\d{1,5}\b/`.
 
 @param line - candidate line of prose
 
 @returns whether a line-number suffix is present
 
 @example
 ```ts
 hasLineNumberSuffix('see file.ts:42 for details'); // true
 hasLineNumberSuffix('the time is 10:30');          // true (matches by shape; cheap false positive)
 hasLineNumberSuffix('no numbers here');            // false
 ```
 */
function hasLineNumberSuffix(line: string,): boolean {
  /**
   Counts consecutive ASCII digits starting at `at`, capped at
   {@link LINE_NUMBER_MAX_DIGITS} to mirror the original regex. The cap makes
   this a constant-bounded scan; it never recurses on input length.
   
   @param at - first offset to inspect
   
   @returns digits matched, in `[0, LINE_NUMBER_MAX_DIGITS]`
   
   @example
   ```ts
   countDigits(colonIdx + 1); // 2 for ':42 trailing'
   ```
   */
  function countDigits(at: number,): number {
    /**
     Running digit tally; returned as the helper-shape binding.
     */
    let count = 0;
    while (
      (count < LINE_NUMBER_MAX_DIGITS)
        && ((at + count) < line
          .length)
        && isDigit(line.charAt(at + count,),)
    ) {
      count += 1;
    }
    return count;
  }
  // Walk every `:` in order (monotonic `indexOf`). A run of 1-5 digits ending at
  // a word boundary marks a line-number suffix; other colons are skipped.
  for (
    let colonIdx = line.indexOf(
      ':',
      0,
    );
    colonIdx !== (-1);
    colonIdx = line.indexOf(
      ':',
      colonIdx + 1,
    )
  ) {
    /**
     Number of digits found right after the colon.
     */
    const digitCount = countDigits(colonIdx + 1,);
    if ((digitCount >= LINE_NUMBER_MIN_DIGITS)
      && (digitCount <= LINE_NUMBER_MAX_DIGITS))
      {
      /**
       Position one past the digit run; checked for a word boundary below.
       */
      const afterIdx = colonIdx + 1
        + digitCount;
      if ((afterIdx >= line
        .length) || (!isWordChar(line.charAt(afterIdx,),)))
        return true;
    }
  }
  return false;
}

//endregion

//region AGENTS.md predicate

/**
 Whether `line` mentions `AGENTS.md` as a word-bounded token. Mirrors
 the original regex `/\bAGENTS\.md\b/`. The check is case-insensitive
 because {@link containsWordBoundedPhrase} lower-cases both sides; the legacy
 regex was case-sensitive, but accepting `agents.md` / `Agents.md` is
 a stricter version of the citation allowlist (more lines allowed, no
 false negatives lost).
 
 @param line - candidate line of prose
 
 @returns whether `AGENTS.md` appears word-bounded
 
 @example
 ```ts
 hasAgentsMdMention('see AGENTS.md rule 5'); // true
 ```
 */
function hasAgentsMdMention(line: string,): boolean {
  return containsWordBoundedPhrase({
    haystack: line,
    phrase: 'AGENTS.md',
  },);
}

//endregion

//region Aggregate

/**
 Whether `line` carries any citation signal that allows a categorical
 dismissal to pass.
 
 @param line - one prose line to inspect
 
 @returns whether the line carries at least one citation signal
 
 @example
 ```ts
 lineHasCitation('skip; see package/x/y.ts:42');  // true
 lineHasCitation("the project doesn't use JSX");    // false
 ```
 */
function lineHasCitation(line: string,): boolean {
  return hasFilePathWithExtension(line,)
    || hasLineNumberSuffix(line,)
    || hasAgentsMdMention(line,);
}

//endregion

export {
  hasAgentsMdMention,
  hasFilePathWithExtension,
  hasLineNumberSuffix,
  lineHasCitation,
};
