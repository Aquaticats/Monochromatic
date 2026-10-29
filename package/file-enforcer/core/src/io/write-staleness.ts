import {
  globs,
  reads,
  type TrackedGlob,
} from '../tracker.ts';
import {
  rememberFreshStalenessEntry,
  stalenessKeyForDest,
  stalenessKeyForDestGlob,
} from './staleness.ts';
import type { GlobDestination, } from './write-each-destinations.ts';

/**
 Returns globally tracked glob expansions in manifest-ready form.
 
 @returns Glob expansions observed during the current config execution.
 
 @example
 ```ts
 const trackedGlobs = currentTrackedGlobs();
 ```
 */
function currentTrackedGlobs(): readonly TrackedGlob[] {
  return [...globs.entries(),]
    .map(function toTrackedGlob(entry,): TrackedGlob {
      /**
       Glob pattern and matched paths tuple.
       */
      const [pattern, paths,] = entry;
      return {
        pattern,
        paths,
      };
    },);
}

/**
 Records staleness metadata for an eager write call by delegating to
 {@link rememberFreshStalenessEntry} with a key from {@link stalenessKeyForDest}
 and globs from {@link currentTrackedGlobs}.
 
 @param dest - Destination file path.
 
 @param content - Generated destination content.
 
 @param manifestPath - Resolved staleness manifest path.
 
 @example
 ```ts
 await rememberEagerWrite({ dest: './CLAUDE.md', content: '...', manifestPath });
 ```
 */
export async function rememberEagerWrite(
  {
    dest,
    content,
    manifestPath,
  }: {
    readonly dest: string;
    readonly content: string;
    readonly manifestPath: string;
  },
): Promise<void> {
  await rememberFreshStalenessEntry({
    manifestPath,
    key: stalenessKeyForDest(dest,),
    kind: 'single',
    trackedReads: [...reads,],
    trackedGlobs: currentTrackedGlobs(),
    destinations: [{
      path: dest,
      content,
    },],
  },);
}

/**
 Records staleness metadata for an eager glob mirror rule by delegating to
 {@link rememberFreshStalenessEntry} with a key from {@link stalenessKeyForDestGlob}
 and globs from {@link currentTrackedGlobs}.
 
 @param destGlob - Destination glob pattern.
 
 @param destinations - Concrete destinations written by the rule.
 
 @param manifestPath - Resolved staleness manifest path.
 
 @example
 ```ts
 await rememberEagerEach({ destGlob: './out/*.md', destinations, manifestPath });
 ```
 */
export async function rememberEagerEach(
  {
    destGlob,
    destinations,
    manifestPath,
  }: {
    readonly destGlob: string;
    readonly destinations: readonly GlobDestination[];
    readonly manifestPath: string;
  },
): Promise<void> {
  await rememberFreshStalenessEntry({
    manifestPath,
    key: stalenessKeyForDestGlob(destGlob,),
    kind: 'each',
    trackedReads: [...reads,],
    trackedGlobs: currentTrackedGlobs(),
    destinations,
  },);
}
