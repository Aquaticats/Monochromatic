/**
 Checks literals whose crate selects a named catalog for that dependency.

 @module
 */

import {
  formatVersion,
  type RustCatalogDocument,
  satisfies,
  splitRequirement,
} from '@monochromatic-dev/module-rust-catalog/ts';

import {
  sameEntry,
  type Selections,
  siteLocation,
} from './adopt-shared.ts';
import type {
  SyncIssue,
  UseSite,
} from './types.ts';

/**
 Reports selected literals the named entry cannot replace.

 @param document - parsed catalog

 @param sites - literal sites with a selection

 @param selections - named-catalog selections

 @param issues - collector the problems are appended to

 @example
 ```ts
 checkSelectedLiterals({ document, sites, selections, issues, });
 ```
 */
export function checkSelectedLiterals(
  {
    document,
    sites,
    selections,
    issues,
  }: {
    readonly document: RustCatalogDocument;
    readonly sites: readonly UseSite[];
    readonly selections: Selections;
    readonly issues: SyncIssue[];
  },
): void {
  for (const site of sites) {
    /**
     Catalog the crate selected for this dependency.
     */
    const catalogName = selections.get(site.manifestPath,)
      ?.get(site.name,)
      ?? '';
    /**
     Named entry, if the catalog has one.
     */
    const entry = document.catalogs
      .get(catalogName,)
      ?.get(site.name,);
    if (entry === undefined) {
      issues.push({
        file: site.manifestPath,
        location: siteLocation(site,),
        problem: `selects catalog "${catalogName}", which has no ${site.name} entry`,
        remedy: `add \`${site.name}\` under \`[catalogs.${catalogName}]\` in rust-catalog.toml, or remove the selection`,
      },);
      continue;
    }
    /**
     Whether the named entry can replace the literal.
     */
    const compatible = (entry.kind === 'registry') && (site.literal
      .kind
      === 'registry')
      ? satisfies({
        version: formatVersion(splitRequirement(entry.version,)
          .version,),
        requirement: site.literal
          .version,
      },)
      : sameEntry({
        left: entry,
        right: site.literal,
      },);
    if (!compatible) {
      issues.push({
        file: site.manifestPath,
        location: siteLocation(site,),
        problem: `literal does not match the selected entry in [catalogs.${catalogName}]`,
        remedy: `change the entry in [catalogs.${catalogName}] or replace the literal with \`workspace = true\``,
      },);
    }
  }
}
