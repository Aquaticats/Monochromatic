import { DEFAULT_CHECK, } from './p.ts';
import type { PackageEntry, } from './types.ts';

//region Merge

/**
 Merges hand-maintained override entries into auto-generated package entries.
 For each generated entry, looks up its effname in the overrides array
 and applies `bin` and `check` if present (an override `check` equal to
 {@link DEFAULT_CHECK} is treated as unset).
 Generated entries without a matching override pass through unchanged.
 
 @param generated - Auto-generated entries from Repology (effname + manager overrides)
 
 @param overrideEntries - Hand-maintained entries with bin/check corrections, keyed by effname
 
 @returns Merged array of package entries with overrides applied
 
 @example
 ```ts
 const packages = mergeOverrides({
   generated,
   overrideEntries: [
     p({ bin: 'rg', effname: 'ripgrep' }),
     p({ check: 'version', effname: 'openssl' }),
   ],
 });
 ```
 */
export function mergeOverrides(
  {
    generated,
    overrideEntries,
  }: {
    readonly generated: readonly PackageEntry[];
    readonly overrideEntries: readonly PackageEntry[];
  },
): readonly PackageEntry[] {
  /**
   Build lookup map from overrides array, keyed by effname
   */
  const overrideMap = new Map<string, PackageEntry>();
  for (const entry of overrideEntries) {
    overrideMap.set(
      entry.effname,
      entry,
    );
  }

  return generated.map(function applyOverride(entry,): PackageEntry {
    /**
     Override entry matched by effname, or `undefined` when none registered.
     */
    const override = overrideMap.get(entry.effname,);
    if (!override)
      return entry;
    return {
      ...entry,
      bin: override.bin
        !== override
          .effname ? override.bin : entry.bin,
      check: override.check
        !== DEFAULT_CHECK ? override.check : entry.check,
    };
  },);
}

//endregion Merge
