import type {
  WatchCtx,
  WatchEvent,
  WatchFilter,
} from '../types.ts';

/**
 Tests whether `c` is a path separator (`/` or `\`); both forms support
 Windows checkouts observed without normalisation.
 
 @param c - single character
 
 @returns whether `c` is `/` or `\`
 
 @example
 ```ts
 isPathSeparator('/');  // true
 isPathSeparator('a');  // false
 ```
 */
function isPathSeparator(c: string,): boolean {
  return (c === '/') || (c === '\\');
}

/**
 Tests whether `c` is a valid first char of a hidden segment body, i.e.
 present (so a trailing `.` with nothing after it does not count),
 neither `.` (so `..` does not count) nor a separator (so `./` does not
 count).
 
 `charAt` past the end of a string returns `''`; rejecting `''` keeps a
 dot at end-of-string (`.`, `foo/.`) from reading as a hidden segment,
 matching the original regex's `[^./\\]` which required a present char.
 
 @param c - single character (or `''` when the body char is absent)
 
 @returns whether `c` continues a hidden segment after the leading `.`
 
 @example
 ```ts
 isHiddenBodyChar('s'); // true (matches '.swp')
 isHiddenBodyChar('');  // false (rejects a trailing '.' with no body)
 isHiddenBodyChar('.'); // false (rejects '..')
 isHiddenBodyChar('/'); // false (rejects './')
 ```
 */
function isHiddenBodyChar(c: string,): boolean {
  return (c !== '')
    && (c !== '.')
    && (!isPathSeparator(c,));
}

/**
 Tests whether `path` has a hidden segment at `idx`: literal `.`
 followed by a non-`.` non-separator char. Bounds-safe (returns false
 past the end).
 
 @param path - path under inspection
 
 @param idx - candidate start of the hidden segment body
 
 @returns whether `path[idx..idx+2)` opens a hidden segment
 */
function hasHiddenSegmentAt({
  path,
  idx,
}: {
  readonly path: string;
  readonly idx: number;
},): boolean {
  return (path.charAt(idx,)
    === '.')
    && isHiddenBodyChar(path.charAt(idx + 1,),);
}

/**
 Tests whether `path` contains any hidden segment.
 
 Mirrors the shape of `/(?:^|[/\\])\.[^./\\]/`: hidden-segment at
 position 0, or after any separator. Admits `.foo`, `src/.git/index`,
 and `.cache/data`; rejects `.` (alone) and `..` (the body char would
 have to be non-`.`).
 
 Hot path on every event: a single left-to-right pass, O(n) time and
 O(1) stack, no regex backtracking risk.
 
 @param path - relative path under inspection
 
 @returns whether `path` contains any hidden segment
 
 @example
 ```ts
 containsHiddenSegment('.config.ts');       // true
 containsHiddenSegment('src/.cache/data');  // true
 containsHiddenSegment('src/foo.ts');       // false
 containsHiddenSegment('./foo.ts');         // false (only '.' at root, no body)
 ```
 */
function containsHiddenSegment(path: string,): boolean {
  if (hasHiddenSegmentAt({
    path,
    idx: 0,
  },)) {
    return true;
  }

  for (let cursorIndex = 0; cursorIndex < path
    .length; cursorIndex += 1) {
    if (
      isPathSeparator(path.charAt(cursorIndex,),)
        && hasHiddenSegmentAt({
          path,
          idx: cursorIndex + 1,
        },)
    ) {
      return true;
    }
  }

  return false;
}

/**
 Builds a {@link WatchFilter} that rejects events whose `relativePath`
 contains any hidden segment (a segment starting with `.`), unless the
 caller opts hidden files in via `allowHidden: true`.
 
 Default-off (rejection mode) matches the `fd` / `rg` ergonomics noted
 in the plan and avoids spurious restarts when an editor writes a `.swp`
 swap file, a build emits a `.cache/` directory, or `.git/index`
 updates after a checkout; events that have nothing to do with the
 source the dev loop is watching.
 
 `allowHidden: true` collapses to a vacuous pass-all; the orchestrator
 additionally elides this filter entirely from the chain when
 `options.hidden === true`, so the runtime cost is exactly zero in that
 mode. The helper still accepts the toggle so library consumers can
 build a hiddenFilter whose behavior can be inspected by the option
 without rebuilding the chain.
 
 Matching is over {@link WatchEvent.relativePath} (relative to the
 deepest matching watch root), not the absolute path; this keeps the
 predicate workspace-portable and consistent with {@link globFilter}
 and {@link regexFilter}.
 
 @returns watch filter that drops hidden-segment events when allowHidden is off
 
 @example
 ```ts
 const filter = hiddenFilter({},);
 filter({ event: { relativePath: '.config.ts', ... }, ctx, },); // false
 filter({ event: { relativePath: 'src/foo.ts', ... }, ctx, },); // true
 filter({ event: { relativePath: 'src/.cache/x', ... }, ctx, },); // false
 ```
 */
export function hiddenFilter(
  {
    allowHidden = false,
  }: {
    readonly allowHidden?: boolean;
  } = {},
): WatchFilter {
  return function hiddenFilterFn(
    {
      event,
    }: {
      readonly event: WatchEvent;
      readonly ctx: WatchCtx;
    },
  ): boolean {
    if (allowHidden)
      return true;
    return !containsHiddenSegment(event.relativePath,);
  };
}
