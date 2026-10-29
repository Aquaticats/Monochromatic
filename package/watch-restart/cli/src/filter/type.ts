import type {
  WatchCtx,
  WatchEntityType,
  WatchEvent,
  WatchFilter,
} from '../types.ts';

/**
 Builds a {@link WatchFilter} that admits only events whose `entity`
 matches one of `types`.
 
 The list is held in a {@link Set} for O(1) lookup. An empty `types`
 argument is a vacuous pass-all (returns `true` for every event);
 the orchestrator never constructs the empty form, but the behaviour
 keeps the helper composable in pipelines that build the type list
 conditionally.
 
 @param types - allowed entity types (`'file'`, `'dir'`); duplicates
   are harmless because the lookup is set membership
 
 @returns watch filter that returns `true` when `event.entity` is admitted
 
 @example
 ```ts
 const filesOnly = typeFilter(['file',],);
 await filesOnly({ event: { entity: 'file', ... }, ctx, },); // true
 await filesOnly({ event: { entity: 'dir', ... }, ctx, },);  // false
 
 const both = typeFilter(['file', 'dir',],);
 await both({ event: { entity: 'dir', ... }, ctx, },); // true
 ```
 */
export function typeFilter(
  types: readonly WatchEntityType[],
): WatchFilter {
  /**
   Set built once at construction so the hot path does one `Set.has`
   per event regardless of how many type tokens the caller supplied.
   */
  const allowed: ReadonlySet<WatchEntityType> = new Set<WatchEntityType>(
    types,
  );
  return function typeFilterFn(
    {
      event,
    }: {
      readonly event: WatchEvent;
      readonly ctx: WatchCtx;
    },
  ): boolean {
    if (allowed.size
      === 0)
      return true;
    return allowed.has(event.entity,);
  };
}
