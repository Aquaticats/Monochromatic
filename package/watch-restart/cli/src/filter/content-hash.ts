import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';

import { OVERSIZED, } from '../hash-cache.ts';
import type {
  WatchCtx,
  WatchEvent,
  WatchFilter,
} from '../types.ts';

/**
 Builds a {@link WatchFilter} that suppresses byte-identical writes.
 
 The filter consults `ctx.hashCache` (shared with the watcher; pre-populated
 during the initial walk) and hashes the event's file on each post-`ready`
 event:
 
 - `unlink`: passes through (`true`); there is no content to compare and
   the watcher already cleared the cache entry, so the decision to fire
   belongs to other filters (e.g. `--events`).
 - `add` / `change`: hashes the file. Same hash as the stored value → skip
   (`false`). Different hash or no stored hash → store the new hash and
   fire (`true`).
 - File exceeds `hashCache.maxHashSize` ({@link HashCache.hashFile} returns
   `null`): returns `true` (fire-without-comparing) so a multi-GB file does
   not block the dev loop on a slow read.
 - Read error (e.g. ENOENT when the file vanishes between event dispatch
   and read): logs a warning and returns `true`; a transient race must
   not silently drop a real change.
 
 The cache is owned by {@link startWatchRestart} and lives on {@link WatchCtx};
 keeping it off the filter's closure means a single shared cache flows
 from the watcher's pre-populate phase straight through to the live
 compare phase without a second copy.
 
 @returns watch filter that returns `false` for byte-identical writes
 
 @example
 ```ts
 const filter = contentHashFilter();
 const passed = await filter({ event, ctx, },);
 ```
 */
export function contentHashFilter(): WatchFilter {
  return async function contentHashFilterFn(
    {
      event,
      ctx,
    }: {
      readonly event: WatchEvent;
      readonly ctx: WatchCtx;
    },
  ): Promise<boolean> {
    if ((event.entity
      === 'dir') || (event.kind
        === 'unlink')) {
      // Directories have no content to hash; `unlink` already cleared
      // the cache entry. In both cases the decision to fire belongs to
      // other filters (e.g. `typeFilter`, `--events`).
      return true;
    }
    try {
      /**
       Hash computed off the current file bytes; the OVERSIZED sentinel means the file exceeds the size cap.
       */
      const fresh = await ctx.hashCache
        .hashFile(event.path,);
      if (fresh === OVERSIZED)
        return true;
      /**
       Previously stored hash for this path; `undefined` when the watcher has never seen the file.
       */
      const prior = ctx.hashCache
        .get(event.path,);
      if (prior === fresh)
        return false;
      ctx.hashCache
        .set({
          path: event.path,
          hash: fresh,
        },);
      return true;
    }
    catch (error) {
      /**
       Human-readable error string used in the fire-on-failure warning log.
       */
      const message = caughtValueText(error,);
      ctx.logger
        .warn(
        `content-hash filter failed for ${event.path}: ${message}; firing`,
        );
      return true;
    }
  };
}
