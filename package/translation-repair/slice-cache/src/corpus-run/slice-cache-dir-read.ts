import { join, } from 'node:path';

import { readTextOrEmptyIfMissing, } from '@monochromatic-dev/translation-repair-typography/ts';
import type { SliceNamespace, } from './slice-cache-claims.ts';

//region Slice cache directory reads
// Which generation a lane last stamped in an entry directory. Lifted out of
// `slice-cache-namespace.ts` for the line cap; that module re-exports it, so
// no call site moved. What the directory holds is listed through
// `directory-listing.ts`, files only, since a directory named like a slice is
// no slice (ledger B65).

/**
 Reads the pipeline that filled one lane's slices.

 @param dir - per-entry cache directory

 @param namespace - lane asking

 @returns Recorded digest, empty when this lane never wrote here

 @throws Error when the marker exists and cannot be read, since treating an
 unreadable marker as absent would DELETE the lane's settled slices

 @example
 ```ts
 const cached = await readNamespaceGeneration({ dir, namespace, },);
 ```
 */
export async function readNamespaceGeneration(
  {
    dir,
    namespace,
  }: {
    readonly dir: string;
    readonly namespace: SliceNamespace;
  },
): Promise<string> {
  // Absent is the ordinary state for a lane that has not written here yet.
  // Anything else, a permission fault above all, must NOT read as absent:
  // that answer discards every settled slice this lane owns, so the read
  // raises it.
  /**
   Raw marker text, including its trailing newline; empty where the lane has
   not written here.
   */
  const text = await readTextOrEmptyIfMissing({
    path: join(
      dir,
      namespace.marker,
    ),
  },);
  return text.trim();
}

//endregion Slice cache directory reads
