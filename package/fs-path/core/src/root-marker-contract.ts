/**
 Marker contract shared by the walker (`root-discovery.ts`), the shipped
 markers (`root-marker.ts`), and the Git marker validation
 (`git-marker.ts`). Lives apart from all three so none imports another
 for a type and no cycle forms.

 @module
 */

import type { RootFilesystem, } from './root-filesystem-contract.ts';

/**
 Arguments supplied to a marker's `matches` probe at each ancestor.
 */
export type RootMatcherArgs = {
  /**
   Directory currently being tested as root candidate.
   */
  readonly dir: string;

  /**
   Filesystem the walk runs over: the runtime adapter, or the one the
   caller passed to `findRoot`.
   */
  readonly fs: RootFilesystem;
};

/**
 Predicate that decides whether a directory is a root.
 */
export type RootMatcher = (args: RootMatcherArgs,) => Promise<boolean>;

/**
 Describes one kind of root: what to look for at each ancestor, and a name
 that identifies the kind in errors, logs, and the memo of `findRootCached`.

 @example
 ```ts
 const QUARANTINE_WARD: RootMarker = {
   name: 'quarantine ward',
   matches: async ({ dir, fs }) => await fs.isDirectory(`${dir}/quarantine`),
 };
 ```
 */
export type RootMarker = {
  /**
   Stable identifier for this kind of root. Two markers with the same name
   share one memo entry per start directory in `findRootCached`, so a
   custom marker needs a name no shipped marker uses.
   */
  readonly name: string;

  /**
   Probe applied at each ancestor, nearest first; the first directory it
   accepts is the root.
   */
  readonly matches: RootMatcher;
};
