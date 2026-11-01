import picomatch from 'picomatch';
import type {
  WatchCtx,
  WatchEvent,
  WatchFilter,
} from '../types.ts';

/**
 Compiled picomatch matcher: takes a path string, returns whether it
 matches the pattern compiled into this matcher.
 
 Aliased so the call sites read as predicate functions (the picomatch
 namespace declares `Matcher` but the type does not survive
 `export = picomatch` cleanly in this consumer's TS setup).
 
 @param path - candidate path (relative to the watch root in our use)
 
 @returns `true` when the path matches the matcher's compiled pattern
 */
type Matcher = (path: string,) => boolean;

/**
 Compiles a list of glob patterns into a list of picomatch matchers.
 Done once per filter construction; the hot path only calls the matcher
 functions, never compiles.
 
 @param patterns - glob patterns understood by picomatch
 
 @returns one matcher per input pattern, in input order
 
 @example
 ```ts
 const matchers = buildMatchers(['*.ts',],);
 matchers[0]('foo.ts',); // true
 ```
 */
function buildMatchers(patterns: readonly string[],): readonly Matcher[] {
  return patterns.map(function compileOne(pattern,): Matcher {
    return picomatch(pattern,);
  },);
}

/**
 Builds a {@link WatchFilter} that gates events by include and exclude
 globs against `event.relativePath`.
 
 Semantics, evaluated in this order on each event:
 
 1. If any `exclude` glob matches `event.relativePath`, return `false`.
    Exclude beats include; an exclude match short-circuits the decision.
 2. If no `include` globs were configured, return `true` (no positive
    constraint to satisfy).
 3. Return `true` iff at least one `include` glob matches.
 
 Both lists default to empty; an entirely default call (`globFilter({})`)
 passes every event, which lets the CLI flag-to-filter compiler hand
 raw flag arrays through without branching on emptiness.
 
 Globs are matched against {@link WatchEvent.relativePath} (relative to
 the deepest matching watch root), not the absolute path; this keeps
 patterns workspace-portable.
 
 @returns watch filter combining include / exclude glob lists
 
 @example
 ```ts
 const filter = globFilter({
   include: ['*.ts',],
   exclude: ['*.test.ts',],
 },);
 filter({ event: { relativePath: 'foo.ts', ... }, ctx, },);      // true
 filter({ event: { relativePath: 'foo.test.ts', ... }, ctx, },); // false (exclude wins)
 ```
 */
export function globFilter(
  {
    include = [],
    exclude = [],
  }: {
    readonly include?: readonly string[];
    readonly exclude?: readonly string[];
  },
): WatchFilter {
  /**
   Compiled include matchers; one per input pattern.
   */
  const includeMatchers: readonly Matcher[] = buildMatchers(include,);
  /**
   Compiled exclude matchers; one per input pattern.
   */
  const excludeMatchers: readonly Matcher[] = buildMatchers(exclude,);

  return function globFilterFn(
    {
      event,
    }: {
      readonly event: WatchEvent;
      readonly ctx: WatchCtx;
    },
  ): boolean {
    if (
      excludeMatchers.some(function matchExclude(m,): boolean {
        return m(event.relativePath,);
      },)
    ) {
      return false;
    }
    if (includeMatchers.length
      === 0)
      return true;
    return includeMatchers.some(function matchInclude(m,): boolean {
      return m(event.relativePath,);
    },);
  };
}
