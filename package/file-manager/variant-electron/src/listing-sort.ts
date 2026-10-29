/**
 Pure sort for directory listings crossing the bridge.
 
 Mirrors `sort_entries` in `package/file-manager/app/src/fs.rs`:
 directories first, then case-insensitive by name, so the Electron prototype
 and the GTK original render identical listings for the same fixture.
 
 @example
 ```ts
 const sorted = sortBridgeEntries({ entries: [] });
 ```
 
 @packageDocumentation
 */

import type { BridgeFileEntry, } from './bridge-types.js';

/**
 Whether an entry sorts into the directories-first group; symlinks stay with
 files because their target kind is unresolved at read time.
 
 @param entry - Entry to classify.
 
 @returns Whether the entry is a real directory.
 
 @example
 ```ts
 isDirectoryEntry({ entry: { kind: 'directory', name: 'a', path: '/a' } });
 ```
 */
function isDirectoryEntry({ entry, }: { readonly entry: BridgeFileEntry; },): boolean {
  return entry.kind === 'directory';
}

/**
 Compares two entries directories-first, then case-insensitively by name
 using plain code-unit order, matching the Rust original's
 `to_lowercase().cmp()` so both apps show one deterministic order.
 
 @param left - First entry.
 
 @param right - Second entry.
 
 @returns Negative, zero, or positive comparator value.
 
 @example
 ```ts
 compareBridgeEntries({
   left: { kind: 'file', name: 'b', path: '/b' },
   right: { kind: 'directory', name: 'a', path: '/a' },
 });
 ```
 */
export function compareBridgeEntries(
  {
    left,
    right,
  }: {
    readonly left: BridgeFileEntry;
    readonly right: BridgeFileEntry;
  },
): number {
  if (isDirectoryEntry({ entry: left, },) && (!isDirectoryEntry({ entry: right, },)))
    return -1;

  if ((!isDirectoryEntry({ entry: left, },)) && isDirectoryEntry({ entry: right, },))
    return 1;

  /**
   Case-folded names compared by plain code-unit order.
   */
  const leftName = left.name
    .toLowerCase();

  /**
   Case-folded counterpart of the right entry's name.
   */
  const rightName = right.name
    .toLowerCase();

  if (leftName < rightName)
    return -1;

  if (leftName > rightName)
    return 1;

  return 0;
}

/**
 Sorts entries directories-first, then case-insensitively by name.
 
 @param entries - Unsorted listing.
 
 @returns Sorted copy of the listing.
 
 @example
 ```ts
 sortBridgeEntries({ entries: [{ kind: 'file', name: 'z', path: '/z' }] });
 ```
 */
export function sortBridgeEntries(
  { entries, }: { readonly entries: readonly BridgeFileEntry[]; },
): readonly BridgeFileEntry[] {
  return entries.toSorted(function byGroupThenName(
    left,
    right,
  ): number {
    return compareBridgeEntries({
      left,
      right,
    },);
  },);
}
