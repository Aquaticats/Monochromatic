/**
 Plans how literal versions left in manifests (for example by `cargo add`) join the catalog.

 Literals are grouped by dependency name across every crate first, so the
 result never depends on the order crates are visited: the new `>=` floor is
 the highest base version, and it must satisfy every literal's own
 requirement, or the group is reported as a conflict for a named catalog.

 @module
 */

import type {
  RustCatalogDocument,
  RustCatalogEntry,
} from '@monochromatic-dev/dependency-catalog-core-rust/ts';

import { planRegistryGroup, } from './adopt-registry.ts';
import { checkSelectedLiterals, } from './adopt-selected.ts';
import {
  sameEntry,
  type Selections,
} from './adopt-shared.ts';
import type {
  SyncIssue,
  UseSite,
} from './types.ts';

export {
  type Selections,
  siteLocation,
} from './adopt-shared.ts';

//region Types

/**
 Catalog changes adoption needs, plus the default catalog as it will read afterwards.

 @example
 ```ts
 const plan: AdoptionPlan = { additions: new Map(), floorRaises: new Map(), defaultCatalog: new Map(), };
 ```
 */
export type AdoptionPlan = {
  readonly additions: ReadonlyMap<string, RustCatalogEntry>;
  readonly floorRaises: ReadonlyMap<string, string>;
  readonly defaultCatalog: ReadonlyMap<string, RustCatalogEntry>;
};

//endregion Types

//region Git groups

/**
 Plans one default-catalog group of git literals: every pin must agree with the others and the existing entry.

 @param name - dependency name

 @param group - git literal sites

 @param existing - current default entry, as a zero- or one-item list

 @param issues - collector the problems are appended to

 @returns the pin to add, or none when it already exists or the pins disagree

 @example
 ```ts
 planGitGroup({ name: 'helix-core', group, existing: [], issues, });
 ```
 */
function planGitGroup(
  {
    name,
    group,
    existing,
    issues,
  }: {
    readonly name: string;
    readonly group: readonly UseSite[];
    readonly existing: readonly RustCatalogEntry[];
    readonly issues: SyncIssue[];
  },
): readonly RustCatalogEntry[] {
  /**
   The pin every literal must share.
   */
  const pin = existing[0]
    ?? group[0]
      ?.literal;
  /**
   Whether every literal matches the pin.
   */
  const agrees = (pin !== undefined) && group.every(function matchesPin(site: UseSite,): boolean {
    return sameEntry({
      left: site.literal,
      right: pin,
    },);
  },);
  if (!agrees) {
    issues.push({
      file: group[0]
        ?.manifestPath
        ?? '',
      location: name,
      problem: `crates pin ${name} to different git revisions`,
      remedy: 'pin every crate to one `rev`',
    },);
    return [];
  }
  return (existing.length === 0) && (pin !== undefined) ? [pin,] : [];
}

//endregion Git groups

//region Plan

/**
 Plans adoption of every literal and returns the catalog changes it implies.

 @param document - parsed catalog

 @param sites - every use site in every manifest

 @param selections - named-catalog selections

 @param issues - collector the problems are appended to

 @returns additions, floor raises, and the resulting default catalog

 @example
 ```ts
 const plan = planAdoption({ document, sites, selections, issues, });
 ```
 */
export function planAdoption(
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
): AdoptionPlan {
  /**
   Literal sites whose crate selects a named catalog for them.
   */
  const selected: UseSite[] = [];
  /**
   Default-catalog literal sites grouped by name.
   */
  const groups = new Map<string, UseSite[]>();
  for (const site of sites) {
    if (site.kind !== 'literal')
      continue;
    if (selections.get(site.manifestPath,)
      ?.has(site.name,)
      === true) {
      selected.push(site,);
      continue;
    }
    groups.set(
      site.name,
      [
        ...(groups.get(site.name,) ?? []),
        site,
      ],
    );
  }
  checkSelectedLiterals({
    document,
    sites: selected,
    selections,
    issues,
  },);

  /**
   New default entries.
   */
  const additions = new Map<string, RustCatalogEntry>();
  /**
   Raised floors of existing default entries.
   */
  const floorRaises = new Map<string, string>();
  /**
   Default catalog after adoption.
   */
  const defaultCatalog = new Map(document.catalog,);

  for (const [name, group,] of groups) {
    /**
     Current default entry.
     */
    const current = document.catalog
      .get(name,);
    /**
     Current default entry as a zero- or one-item list.
     */
    const existing = current === undefined ? [] : [current,];
    /**
     Distinct source kinds across the group and the existing entry.
     */
    const kinds = new Set<string>();
    for (const site of group)
      kinds.add(site.literal
        .kind,);
    if (current !== undefined)
      kinds.add(current.kind,);
    if (kinds.size > 1) {
      issues.push({
        file: group[0]
          ?.manifestPath
          ?? '',
        location: name,
        problem: `${name} is a git dependency in some places and a registry dependency in others`,
        remedy: 'use one source for every crate, or give the other source its own named catalog',
      },);
      continue;
    }
    /**
     Planned default entry, empty when the group conflicts or needs no change.
     */
    const [planned,] = kinds.has('git')
      ? planGitGroup({
        name,
        group,
        existing,
        issues,
      },)
      : planRegistryGroup({
        name,
        literals: group,
        existing,
        issues,
      },);
    if (planned === undefined)
      continue;
    if (current === undefined)
      additions.set(
        name,
        planned,
      );
    else if ((planned.kind === 'registry') && (!sameEntry({
      left: planned,
      right: current,
    },)))
      floorRaises.set(
        name,
        planned.version,
      );
    defaultCatalog.set(
      name,
      planned,
    );
  }
  return {
    additions,
    floorRaises,
    defaultCatalog,
  };
}

//endregion Plan
