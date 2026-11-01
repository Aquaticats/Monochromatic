import type {
  WatchCtx,
  WatchEvent,
  WatchFilter,
} from '../types.ts';

/**
 Combines filters with logical AND (all-of).
 
 Returns a {@link WatchFilter} that runs the input filters in input
 order and short-circuits on the first `false`: filters past the first
 skip do not execute. Vacuous truth: an empty `filters` array makes the
 resulting filter return `true` for every event (matches the
 mathematical convention of "all of nothing is true").
 
 Always-async because individual filters may be sync OR async; the
 unified return type avoids branching at every call site downstream.
 
 Accepts a single array parameter instead of rest args; rest params are
 banned by AGENTS.md, and the array form composes more readably in
 fixtures and helper functions that build filter pipelines.
 
 @param filters - filters to AND together; order is preserved
 
 @returns watch filter that returns `true` iff every input returns `true`
 
 @example
 ```ts
 const filter = composeFilters([
   extFilter(['.ts',],),
   contentHashFilter(),
 ],);
 await filter({ event, ctx, },);
 ```
 */
export function composeFilters(filters: readonly WatchFilter[],): WatchFilter {
  return async function composedAll(
    {
      event,
      ctx,
    }: {
      readonly event: WatchEvent;
      readonly ctx: WatchCtx;
    },
  ): Promise<boolean> {
    for (const filter of filters) {
      /**
       Per-iteration filter verdict; a `false` short-circuits the AND chain.
       */
      /* oxlint-disable-next-line eslint/no-await-in-loop -- intentional short-circuit: filter N+1 must not run when filter N said skip */
      const passed = await filter({
        event,
        ctx,
      },);
      if (!passed)
        return false;
    }
    return true;
  };
}

/**
 Combines filters with logical OR (any-of).
 
 Returns a {@link WatchFilter} that runs the input filters in input
 order and short-circuits on the first `true`: filters past the first
 pass do not execute. Vacuous falsehood: an empty `filters` array makes
 the resulting filter return `false` for every event (matches the
 mathematical convention of "any of nothing is false").
 
 Always-async for the same reason as {@link composeFilters}.
 
 @param filters - filters to OR together; order is preserved
 
 @returns watch filter that returns `true` iff at least one input returns `true`
 
 @example
 ```ts
 const filter = anyFilter([
   extFilter(['.ts',],),
   extFilter(['.tsx',],),
 ],);
 await filter({ event, ctx, },);
 ```
 */
export function anyFilter(filters: readonly WatchFilter[],): WatchFilter {
  return async function anyOf(
    {
      event,
      ctx,
    }: {
      readonly event: WatchEvent;
      readonly ctx: WatchCtx;
    },
  ): Promise<boolean> {
    for (const filter of filters) {
      /**
       Per-iteration filter verdict; a `true` short-circuits the OR chain.
       */
      /* oxlint-disable-next-line eslint/no-await-in-loop -- intentional short-circuit: filter N+1 must not run when filter N said fire */
      const passed = await filter({
        event,
        ctx,
      },);
      if (passed)
        return true;
    }
    return false;
  };
}
