/**
 Transformation functions for the Bash output filter.
 
 Each function handles one transformation pass: stripping noise lines,
 collapsing repeated characters, replacing absolute paths with relative
 equivalents, truncating long lines, and deduplicating consecutive identical lines.
 
 @module
 */

import {
  isWhitespace,
  isWordChar,
} from '@monochromatic-dev/agent-harness-shared-text-scan/ts';
import {
  ALT_CWD_PREFIX,
  CWD_PREFIX,
  DEDUP_THRESHOLD,
  GIT_TRANSPORT_PROGRESS_PREDICATES,
  HOME_DIR,
  isGitFileModeLine,
  MAX_LINE_LENGTH,
  MAX_REPEATED_CHARS,
  MISE_NOISE_PREDICATES,
  PATH_PREFIX_ABSENT,
  REAL_HOME_DIR,
  SANDBOX_NOISE_PREDICATES,
  type PathPrefix,
} from './filter-patterns.ts';

//region Line stripping

/**
 Every noise predicate consulted by {@link shouldStripLine}, flattened into a
 single scan. Adding a noise category is one array entry here rather than
 another `.some` branch inside the function.
 */
const STRIP_LINE_PREDICATES: readonly ((line: string,) => boolean)[] = [
  isGitFileModeLine,
  ...GIT_TRANSPORT_PROGRESS_PREDICATES,
  ...SANDBOX_NOISE_PREDICATES,
  ...MISE_NOISE_PREDICATES,
];

/**
 Whether a line should be removed entirely.
 
 @param line - trimmed line to check
 
 @returns `true` if the line is noise that should be stripped
 
 @example
 ```ts
 shouldStripLine('100644'); // true (git file mode noise)
 shouldStripLine('hello world'); // false
 ```
 */
function shouldStripLine(line: string,): boolean {
  return STRIP_LINE_PREDICATES.some(function predicateTest(predicate,): boolean {
    return predicate(line,);
  },);
}

//endregion

//region Character collapsing

/**
 Minimum consecutive identical non-word, non-whitespace characters before collapsing.
 */
const COLLAPSE_THRESHOLD = 10;

/**
 Whether `c` should participate in a collapsible run: any character that is
 neither a word character (`\w`) nor whitespace. Mirrors the original
 regex's `[^\w\s]` character class.
 
 @param c - one-character string to inspect
 
 @returns whether `c` is a punctuation/symbol candidate for collapse
 
 @example
 ```ts
 isCollapseCandidate('=');  // true
 isCollapseCandidate('a');  // false (word char)
 isCollapseCandidate(' ');  // false (whitespace)
 ```
 */
function isCollapseCandidate(c: string,): boolean {
  return (!isWordChar(c,)) && (!isWhitespace(c,));
}

/**
 Collapses runs of repeated characters beyond {@link MAX_REPEATED_CHARS}.
 Only collapses runs of {@link COLLAPSE_THRESHOLD}+ non-word, non-whitespace
 characters. Mirrors the original regex `([^\w\s])\1{9,}` without invoking
 the regex engine.
 
 @param line - line to process
 
 @returns line with long character runs collapsed
 
 @example
 ```ts
 collapseRepeatedChars('====================');
 // '==== (x20 repeated characters)'
 ```
 */
function collapseRepeatedChars(line: string,): string {
  /**
   Output segments, joined once at the end so no intermediate string is recopied per character.
   */
  const parts: string[] = [];
  // Single forward pass; `idx` jumps by whole runs, so the stride is variable
  // and the update happens in the body rather than a fixed `for` step.
  for (let cursorIndex = 0; cursorIndex < line
    .length;) {
    /**
     Character under the cursor; gates whether a run is even considered.
     */
    const c = line.charAt(cursorIndex,);
    if (!isCollapseCandidate(c,)) {
      parts.push(c,);
      cursorIndex += 1;
      continue;
    }
    /**
     Exclusive end of the current run of `c`, advanced by a linear scan.
     */
    let runEnd = cursorIndex + 1;
    while ((runEnd < line
      .length) && (line.charAt(runEnd,)
        === c)) {
      runEnd += 1;
    }
    /**
     Length of the current run; gates the collapse vs. emit-verbatim choice.
     */
    const runLength = runEnd - cursorIndex;
    if (runLength >= COLLAPSE_THRESHOLD) {
      parts.push(`${c.repeat(MAX_REPEATED_CHARS,)} (x${runLength} repeated characters)`,);
    }
    else {
      parts.push(line.slice(
        cursorIndex,
        runEnd,
      ),);
    }
    cursorIndex = runEnd;
  }
  return parts.join('',);
}

//endregion

//region Path collapsing

/**
 Narrows a path prefix from sentinel-bearing form to a usable string.
 
 @param prefix - candidate path prefix
 
 @returns whether the prefix is available
 */
function hasPathPrefix(prefix: PathPrefix,): prefix is string {
  return prefix !== PATH_PREFIX_ABSENT;
}

/**
 Replaces working-directory paths with relative equivalents, only when the
 path appears at the beginning of a line. Restricts replacement to line-initial
 position to avoid mangling paths embedded in error messages or JSON.
 
 @param line - line to process
 
 @returns line with CWD-prefixed start converted to a relative path
 
 @example
 ```ts
 // CWD = '/repo'
 collapseCwdPaths('/repo/src/index.ts'); // 'src/index.ts'
 collapseCwdPaths('error at /repo/x'); // unchanged (not at start)
 ```
 */
function collapseCwdPaths(line: string,): string {
  if (!hasPathPrefix(CWD_PREFIX,))
    return line;

  if (hasPathPrefix(ALT_CWD_PREFIX,)) {
    if (ALT_CWD_PREFIX.length
      >= CWD_PREFIX
        .length) {
      if (line.startsWith(ALT_CWD_PREFIX,))
        return line.slice(ALT_CWD_PREFIX.length,);
      if (line.startsWith(CWD_PREFIX,))
        return line.slice(CWD_PREFIX.length,);
    }
    else {
      if (line.startsWith(CWD_PREFIX,))
        return line.slice(CWD_PREFIX.length,);
      if (line.startsWith(ALT_CWD_PREFIX,))
        return line.slice(ALT_CWD_PREFIX.length,);
    }
  }
  else if (line.startsWith(CWD_PREFIX,)) {
    return line.slice(CWD_PREFIX.length,);
  }
  return line;
}

/**
 Replaces home directory paths with `~` shorthand, only at line start.
 Handles both `$HOME` path and its real (symlink-resolved) path.
 
 @param line - line to process
 
 @returns line with home directory path collapsed to `~` if at start
 
 @example
 ```ts
 // HOME = '/home/alice'
 collapseHomePaths('/home/alice/notes.md'); // '~/notes.md'
 ```
 */
function collapseHomePaths(line: string,): string {
  if (!hasPathPrefix(HOME_DIR,))
    return line;

  if (hasPathPrefix(REAL_HOME_DIR,)) {
    if (REAL_HOME_DIR.length
      >= HOME_DIR
        .length) {
      if (line.startsWith(REAL_HOME_DIR,))
        return `~${line.slice(REAL_HOME_DIR.length,)}`;
      if (line.startsWith(HOME_DIR,))
        return `~${line.slice(HOME_DIR.length,)}`;
    }
    else {
      if (line.startsWith(HOME_DIR,))
        return `~${line.slice(HOME_DIR.length,)}`;
      if (line.startsWith(REAL_HOME_DIR,))
        return `~${line.slice(REAL_HOME_DIR.length,)}`;
    }
  }
  else if (line.startsWith(HOME_DIR,)) {
    return `~${line.slice(HOME_DIR.length,)}`;
  }
  return line;
}

//endregion

//region Line truncation

/**
 Truncates a line that exceeds {@link MAX_LINE_LENGTH}, appending a length marker.
 
 @param line - line to potentially truncate
 
 @returns original line if short enough, or truncated with a length marker
 
 @example
 ```ts
 truncateLine('x'.repeat(800)); // truncates with `... [800 chars]`
 ```
 */
function truncateLine(line: string,): string {
  if (line.length
    <= MAX_LINE_LENGTH)
    return line;
  return `${
    line.slice(
      0,
      MAX_LINE_LENGTH,
    )
  }... [${line.length} chars]`;
}

//endregion

//region Deduplication

/**
 Builds the lines representing a run of repeated lines. Collapses runs of
 {@link DEDUP_THRESHOLD}+ identical lines to a single `line (xN)` marker.
 
 @param line - repeated line content
 
 @param count - how many consecutive times `line` appeared
 
 @returns lines to append: empty for a zero-count run, one collapsed marker
   once the run reaches the threshold, otherwise `count` copies of `line`
 
 @example
 ```ts
 flushRepeated({ line: 'foo', count: 5, },); // ['foo (x5 repeated lines)']
 flushRepeated({ line: 'bar', count: 2, },); // ['bar', 'bar']
 ```
 */
function flushRepeated(
  {
    line,
    count,
  }: {
    readonly line: string;
    readonly count: number;
  },
): readonly string[] {
  if (count === 0)
    return [];
  if (count >= DEDUP_THRESHOLD)
    return [
      `${line} (x${count} repeated lines)`,
    ];
  return Array.from(
    { length: count, },
    function repeatLine(): string {
      return line;
    },
  );
}

//endregion

export {
  collapseCwdPaths,
  collapseHomePaths,
  collapseRepeatedChars,
  flushRepeated,
  shouldStripLine,
  truncateLine,
};
