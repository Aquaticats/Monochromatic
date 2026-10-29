/**
 Per-crate `default-features` choice for generated `[workspace.dependencies]` entries.

 Cargo before 1.99 rejects `default-features = false` beside `workspace = true`
 when the workspace entry keeps default features ("`default-features = false`
 cannot override workspace's `default-features`"). So a crate whose catalog
 sites all disable default features gets `default-features = false` in its own
 generated entry, and its sites keep adding the features they need.

 @module
 */

import type { RustCatalogEntry, } from '@monochromatic-dev/module-rust-catalog/ts';

import type { CrateInfo, } from './crate-info.ts';
import type {
  GeneratedEntry,
  SyncIssue,
  UseSite,
} from './types.ts';

//region Default features

/**
 Pairs each resolved entry with the crate's default-features choice.

 Sites that disagree cannot share one generated entry and are reported.

 @param info - crate information

 @param resolvedEntries - dependency name to catalog entry; sites of other names are ignored

 @param issues - collector the disagreements are appended to

 @returns generated entries for this crate

 @example
 ```ts
 generatedEntries({ info, resolvedEntries, issues, });
 ```
 */
export function generatedEntries(
  {
    info,
    resolvedEntries,
    issues,
  }: {
    readonly info: CrateInfo;
    readonly resolvedEntries: ReadonlyMap<string, RustCatalogEntry>;
    readonly issues: SyncIssue[];
  },
): ReadonlyMap<string, GeneratedEntry> {
  /**
   Generated entries for this crate.
   */
  const entries = new Map<string, GeneratedEntry>();
  for (const [name, entry,] of resolvedEntries) {
    /**
     Per catalog site of this dependency, whether it disables default features; path sites never inherit.
     */
    const flags = info.sites
      .filter(function isCatalogSite(site: UseSite,): boolean {
        return (site.kind !== 'path') && (site.name === name);
      },)
      .map(function disablesDefaults(site: UseSite,): boolean {
        return site.disablesDefaultFeatures;
      },);
    /**
     Sites that disable default features.
     */
    const disablingSites = flags.filter(function disables(flag: boolean,): boolean {
      return flag;
    },);
    /**
     Number of sites that disable default features.
     */
    const disabling = disablingSites.length;
    /**
     Whether every site disables default features.
     */
    const allDisable = (flags.length > 0) && (disabling === flags.length);
    /**
     Whether some sites disable default features and others keep them.
     */
    const mixed = (disabling > 0) && (disabling < flags.length);
    if (mixed) {
      issues.push({
        file: info.manifest
          .path,
        location: name,
        problem: `some ${name} entries write \`default-features = false\` and others do not, but the crate's one generated [workspace.dependencies] entry can hold only one choice`,
        remedy: `write \`default-features = false\` on every ${name} entry in this manifest (adding the needed features explicitly), or remove it from all of them`,
      },);
    }
    entries.set(
      name,
      {
        entry,
        defaultFeatures: !allDisable,
      },
    );
  }
  return entries;
}

//endregion Default features
