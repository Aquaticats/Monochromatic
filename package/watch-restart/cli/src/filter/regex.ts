import type {
  WatchCtx,
  WatchEvent,
  WatchFilter,
} from '../types.ts';

/**
 Builds a {@link WatchFilter} that gates events by include and exclude
 regex patterns against `event.relativePath`.
 
 Why a sibling to {@link globFilter} rather than a flag on it: regex and
 glob have different semantics (anchored vs. shell-glob; character class
 vs. brace alternation), and a watchexec consumer expressing intent in
 regex form should not have to translate it into picomatch idioms just
 to reach the same chokidar event. Both filters run in series; either
 side rejecting an event is enough to skip it.
 
 Semantics, evaluated in this order on each event:
 
 1. If any `exclude` pattern matches `event.relativePath`, return `false`.
    Exclude beats include; an exclude match short-circuits the decision.
 2. If no `include` patterns were configured, return `true` (no positive
    constraint to satisfy).
 3. Return `true` iff at least one `include` pattern matches.
 
 Both lists default to empty; an entirely default call (`regexFilter({})`)
 passes every event, matching {@link globFilter}'s vacuous-pass-all
 convention so the orchestrator can hand raw flag arrays through without
 branching on emptiness when both lists happen to be empty.
 
 Patterns are matched against {@link WatchEvent.relativePath} (relative
 to the deepest matching watch root), not the absolute path; this keeps
 patterns workspace-portable and consistent with {@link globFilter}.
 
 @returns watch filter combining include / exclude regex lists
 
 @example
 ```ts
 const filter = regexFilter({
   include: [/\.story\.ts$/,],
   exclude: [/\.test\.ts$/,],
 },);
 filter({ event: { relativePath: 'foo.story.ts', ... }, ctx, },); // true
 filter({ event: { relativePath: 'foo.test.ts', ... }, ctx, },);  // false
 ```
 */
export function regexFilter(
  {
    include = [],
    exclude = [],
  }: {
    readonly include?: readonly RegExp[];
    readonly exclude?: readonly RegExp[];
  },
): WatchFilter {
  return function regexFilterFn(
    {
      event,
    }: {
      readonly event: WatchEvent;
      readonly ctx: WatchCtx;
    },
  ): boolean {
    if (
      exclude.some(function matchExclude(re,): boolean {
        return re.test(event.relativePath,);
      },)
    ) {
      return false;
    }
    if (include.length
      === 0)
      return true;
    return include.some(function matchInclude(re,): boolean {
      return re.test(event.relativePath,);
    },);
  };
}
