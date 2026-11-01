/**
 Path glob matching for the fixture allowlist.
 
 Supports the subset of glob syntax the allowlist needs: `**` matching zero or
 more whole path segments, and `*` matching zero or more characters inside a
 single segment. Both levels run the same two-pointer backtracking scan, so
 matching stays bounded by pattern length times subject length with no
 recursion and no regex engine.
 
 @module
 */

/**
 Wildcard matching a whole run of path segments.
 */
const SEGMENT_WILDCARD = '**';

/**
 Wildcard matching a run of characters inside one path segment.
 */
const CHARACTER_WILDCARD = '*';

/**
 Sentinel meaning no wildcard has been passed yet, so no backtrack point exists.
 */
const NO_BACKTRACK = -1;

/**
 Separator that both patterns and normalized paths use.
 */
const SEPARATOR = '/';

/**
 Mutable scan position shared by both matching levels.
 
 Held as one object rather than separate bindings so the scan needs no
 function-root `let`.
 */
type ScanCursor = {
  /**
   Cursor into the pattern.
   */
  pattern: number;
  /**
   Cursor into the subject being matched.
   */
  subject: number;
  /**
   Pattern position of the most recent wildcard, or the no-backtrack sentinel.
   */
  wildcardPattern: number;
  /**
   Subject position the most recent wildcard was asked to cover from.
   */
  wildcardSubject: number;
};

/**
 Builds a scan cursor positioned at the start of both inputs.
 
 @returns fresh cursor with no recorded backtrack point
 
 @example
 ```ts
 const cursor = startCursor();
 ```
 */
function startCursor(): ScanCursor {
  return {
    pattern: 0,
    subject: 0,
    wildcardPattern: NO_BACKTRACK,
    wildcardSubject: NO_BACKTRACK,
  };
}

/**
 Rewinds a cursor to just past its recorded wildcard, consuming one more subject unit.
 
 @param cursor - scan position to rewind
 
 @returns false when no wildcard was recorded, so the match cannot continue
 
 @mutates cursor - advances the wildcard's coverage and resets both cursors
 
 @example
 ```ts
 if (!backtrack({ cursor })) return cursor.pattern;
 ```
 */
function backtrack({ cursor, }: {
  /**
   Scan position to rewind.
   */
  readonly cursor: ScanCursor;
},): boolean {
  if (cursor.wildcardPattern === NO_BACKTRACK)
    return false;
  cursor.wildcardSubject += 1;
  cursor.subject = cursor.wildcardSubject;
  cursor.pattern = cursor.wildcardPattern + 1;
  return true;
}

/**
 Scans one path segment against one pattern segment containing `*` wildcards.
 
 On mismatch the scan rewinds to just after the most recent `*` and lets it
 cover one more character. Without a wildcard to rewind to the scan stops, so
 no input drives it beyond pattern length times text length.
 
 @param pattern - single pattern segment, `*` wildcards allowed
 
 @param text - single path segment to test
 
 @returns pattern position reached, equal to pattern length only on a full match
 
 @example
 ```ts
 scanSegment({ pattern: 'fixture.*', text: 'fixture.json' });
 ```
 */
function scanSegment({
  pattern,
  text,
}: {
  /**
   Single pattern segment, `*` wildcards allowed.
   */
  readonly pattern: string;
  /**
   Single path segment to test.
   */
  readonly text: string;
},): number {
  /**
   Scan position across pattern and text, carrying the wildcard backtrack point.
   */
  const cursor = startCursor();

  while (cursor.subject < text.length) {
    if (pattern.charAt(cursor.pattern,) === CHARACTER_WILDCARD) {
      cursor.wildcardPattern = cursor.pattern;
      cursor.wildcardSubject = cursor.subject;
      cursor.pattern += 1;
      continue;
    }
    if ((cursor.pattern < pattern.length)
      && (pattern.charAt(cursor.pattern,) === text.charAt(cursor.subject,)))
      {
      cursor.pattern += 1;
      cursor.subject += 1;
      continue;
    }
    if (!backtrack({ cursor, },))
      return NO_BACKTRACK;
  }

  // Trailing `*` runs are allowed to match nothing.
  while (pattern.charAt(cursor.pattern,) === CHARACTER_WILDCARD)
    cursor.pattern += 1;
  return cursor.pattern;
}

/**
 Matches one path segment against one pattern segment.
 
 @param pattern - single pattern segment, `*` wildcards allowed
 
 @param text - single path segment to test
 
 @returns true when the pattern covers the whole segment
 
 @example
 ```ts
 matchSegment({ pattern: '*-helpers.ts', text: 'tree-helpers.ts' });
 ```
 */
function matchSegment({
  pattern,
  text,
}: {
  /**
   Single pattern segment, `*` wildcards allowed.
   */
  readonly pattern: string;
  /**
   Single path segment to test.
   */
  readonly text: string;
},): boolean {
  return scanSegment({
    pattern,
    text,
  },) === pattern.length;
}

/**
 Scans a segmented path against a segmented pattern containing `**` wildcards.
 
 Mirrors {@link scanSegment} one level up: `**` records a backtrack point and
 every later mismatch makes it swallow one more path segment.
 
 @param patternSegments - pattern split on separators
 
 @param pathSegments - path split on separators
 
 @returns pattern position reached, with trailing `**` runs already consumed
 */
function scanSegments({
  patternSegments,
  pathSegments,
}: {
  /**
   Pattern split on separators.
   */
  readonly patternSegments: readonly string[];
  /**
   Path split on separators.
   */
  readonly pathSegments: readonly string[];
},): number {
  /**
   Scan position across pattern and path segments.
   */
  const cursor = startCursor();

  while (cursor.subject < pathSegments.length) {
    /**
     Current pattern segment, absent once the pattern is exhausted.
     */
    const patternSegment = patternSegments[cursor.pattern];
    if (patternSegment === SEGMENT_WILDCARD) {
      cursor.wildcardPattern = cursor.pattern;
      cursor.wildcardSubject = cursor.subject;
      cursor.pattern += 1;
      continue;
    }
    if ((patternSegment !== undefined)
      && matchSegment({
        pattern: patternSegment,
        text: pathSegments[cursor.subject] ?? '',
      },))
      {
      cursor.pattern += 1;
      cursor.subject += 1;
      continue;
    }
    if (!backtrack({ cursor, },))
      return NO_BACKTRACK;
  }

  // Trailing `**` runs are allowed to match nothing.
  while (patternSegments[cursor.pattern] === SEGMENT_WILDCARD)
    cursor.pattern += 1;
  return cursor.pattern;
}

/**
 Tests one already-normalized path against one glob pattern.
 
 Both inputs are split on `/`; callers pass normalized paths so a single
 pattern spelling works regardless of host separator.
 
 @param pattern - glob supporting `**` segment and `*` character wildcards
 
 @param path - normalized absolute path to test
 
 @returns true when the pattern covers the path exactly
 
 @example
 ```ts
 matchesGlob({ pattern: '**\/*-helpers.ts', path: '/repo/package/x/src/tree-helpers.ts' });
 ```
 
 @internal
 */
export function matchesGlob({
  pattern,
  path,
}: {
  /**
   Glob supporting `**` segment and `*` character wildcards.
   */
  readonly pattern: string;
  /**
   Normalized absolute path to test.
   */
  readonly path: string;
},): boolean {
  /**
   Pattern split into segments, reused for the full-coverage comparison.
   */
  const patternSegments = pattern.split(SEPARATOR,);
  return scanSegments({
    patternSegments,
    pathSegments: path.split(SEPARATOR,),
  },) === patternSegments.length;
}

/**
 Tests one path against a list of glob patterns.
 
 @param patterns - glob patterns to try in order
 
 @param path - normalized absolute path to test
 
 @returns true when any pattern covers the path
 
 @example
 ```ts
 matchesAnyGlob({ patterns: ['**\/fixture.*'], path: '/repo/src/fixture.json' });
 ```
 
 @internal
 */
export function matchesAnyGlob({
  patterns,
  path,
}: {
  /**
   Glob patterns to try in order.
   */
  readonly patterns: readonly string[];
  /**
   Normalized absolute path to test.
   */
  readonly path: string;
},): boolean {
  return patterns.some(function coversPath(pattern,): boolean {
    return matchesGlob({
      pattern,
      path,
    },);
  },);
}
