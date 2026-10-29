/**
 Plans how literal versions left in manifests (for example by `cargo add`) join the catalog.

 Literals are grouped by dependency name across every crate first, so the
 result never depends on the order crates are visited: the new `>=` floor is
 the highest base version, and it must satisfy every literal's own
 requirement, or the group is reported as a conflict for a named catalog.

 @module
 */

import {
  compareVersions,
  formatVersion,
  type ParsedVersion,
  type RustCatalogDocument,
  type RustCatalogEntry,
  satisfies,
  splitRequirement,
} from '@monochromatic-dev/module-rust-catalog/ts';

import type {
  SyncIssue,
  UseSite,
} from './types.ts';

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

/**
 Named-catalog selections: manifest path, then dependency name, to catalog name.

 @example
 ```ts
 const selections: Selections = new Map([['a/Cargo.toml', new Map([['windows', 'windows-legacy']])]]);
 ```
 */
export type Selections = ReadonlyMap<string, ReadonlyMap<string, string>>;

//endregion Types

//region Helpers

/**
 Dotted location of a site for diagnostics.

 @param site - use site

 @returns e.g. `target.cfg(unix).dependencies.libc`

 @example
 ```ts
 siteLocation(site);
 ```
 */
export function siteLocation(site: UseSite,): string {
  return [...site.table, site.name,].join('.',);
}

/**
 Whether two catalog entries are the same source.

 @param left - entry

 @param right - entry

 @returns whether kind, version or git pin, and rename agree

 @example
 ```ts
 sameEntry({ left, right, });
 ```
 */
export function sameEntry({ left, right, }: { readonly left: RustCatalogEntry; readonly right: RustCatalogEntry; },): boolean {
  if (left.package !== right.package)
    return false;
  if ((left.kind === 'git') && (right.kind === 'git'))
    return (left.git === right.git) && (left.rev === right.rev);
  if ((left.kind === 'registry') && (right.kind === 'registry'))
    return left.version === right.version;
  return false;
}

/**
 Version text of a requirement without its `>=`, `^`, or `~` operator, keeping the author's precision.

 @param requirement - single-comparator requirement

 @returns version text, e.g. `1.50` for `^1.50`

 @example
 ```ts
 stripOperator('~0.58.1'); // '0.58.1'
 ```
 */
function stripOperator(requirement: string,): string {
  /**
   Requirement without surrounding spaces.
   */
  const trimmed = requirement.trim();
  for (const operator of ['>=', '^', '~',]) {
    if (trimmed.startsWith(operator,))
      return trimmed.slice(operator.length,).trim();
  }
  return trimmed;
}

/**
 Version text and parse of one candidate floor.
 */
type FloorCandidate = {
  readonly text: string;
  readonly parsed: ParsedVersion;
};

/**
 Remedy text for a literal that needs a named catalog.

 @param site - literal site

 @param requirement - the literal's requirement

 @returns instructions naming the catalog table and the crate's selection table

 @example
 ```ts
 namedCatalogRemedy({ site, requirement: '=1.18.1', });
 ```
 */
function namedCatalogRemedy({ site, requirement, }: { readonly site: UseSite; readonly requirement: string; },): string {
  return [
    `add \`${site.name} = ${JSON.stringify(requirement,)}\` to a named catalog in rust-catalog.toml (for example \`[catalogs.pinned]\`)`,
    `and select it in ${site.manifestPath} with \`[package.metadata.rust-catalog.catalogs]\` \`${site.name} = "pinned"\``,
  ].join(' ',);
}

//endregion Helpers

//region Named selections

/**
 Checks literals whose crate selects a named catalog for that dependency.

 @param document - parsed catalog

 @param sites - literal sites with a selection

 @param selections - named-catalog selections

 @param issues - collector the problems are appended to

 @example
 ```ts
 checkSelectedLiterals({ document, sites, selections, issues, });
 ```
 */
function checkSelectedLiterals(
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
    const catalogName = selections.get(site.manifestPath,)?.get(site.name,) ?? '';
    /**
     Named entry, if the catalog has one.
     */
    const entry = document.catalogs.get(catalogName,)?.get(site.name,);
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
    const compatible = (entry.kind === 'registry') && (site.literal.kind === 'registry')
      ? satisfies({ version: formatVersion(splitRequirement(entry.version,).version,), requirement: site.literal.version, },)
      : sameEntry({ left: entry, right: site.literal, },);
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

//endregion Named selections

//region Default catalog

/**
 Plans one default-catalog group of registry literals sharing a dependency name.

 @param name - dependency name

 @param literals - registry literal sites

 @param existing - current default entry, when the catalog has one

 @param issues - collector the problems are appended to

 @returns the requirement the default entry should hold, or none when the group conflicts

 @example
 ```ts
 planRegistryGroup({ name: 'tokio', literals, existing: [], issues, });
 ```
 */
function planRegistryGroup(
  {
    name,
    literals,
    existing,
    issues,
  }: {
    readonly name: string;
    readonly literals: readonly UseSite[];
    readonly existing: readonly RustCatalogEntry[];
    readonly issues: SyncIssue[];
  },
): readonly RustCatalogEntry[] {
  /**
   Issue count before this group.
   */
  const issuesBefore = issues.length;
  /**
   Candidate floors: each literal's base version, then the current floor.
   */
  const candidates: FloorCandidate[] = [];
  for (const site of literals) {
    if (site.literal.kind !== 'registry')
      continue;
    try {
      /**
       Operator and base version of the literal.
       */
      const { operator, version, } = splitRequirement(site.literal.version,);
      if (operator === '=') {
        issues.push({
          file: site.manifestPath,
          location: siteLocation(site,),
          problem: `exact pin ${JSON.stringify(site.literal.version,)} cannot become a \`>=\` floor in [catalog]`,
          remedy: namedCatalogRemedy({ site, requirement: site.literal.version, },),
        },);
        continue;
      }
      candidates.push({ text: stripOperator(site.literal.version,), parsed: version, },);
    } catch (error) {
      issues.push({
        file: site.manifestPath,
        location: siteLocation(site,),
        problem: `requirement ${JSON.stringify(site.literal.version,)} is not one comparator the catalog can adopt (${String(error,)})`,
        remedy: 'write a single requirement such as `"1.2"`, `"~1.2.3"`, or `">=1.2"`',
      },);
    }
  }
  for (const entry of existing) {
    if (entry.kind === 'registry')
      candidates.push({ text: entry.version.slice(2,).trim(), parsed: splitRequirement(entry.version,).version, },);
  }
  if ((issues.length !== issuesBefore) || (candidates.length === 0))
    return [];

  /**
   Highest candidate, the floor every literal must accept.
   */
  const highest = candidates.reduce(function pickHigher(best, candidate,): FloorCandidate {
    return compareVersions({ left: candidate.parsed, right: best.parsed, },) > 0 ? candidate : best;
  },);
  for (const site of literals) {
    if ((site.literal.kind === 'registry') && !satisfies({ version: formatVersion(highest.parsed,), requirement: site.literal.version, },)) {
      issues.push({
        file: site.manifestPath,
        location: siteLocation(site,),
        problem: `requires ${JSON.stringify(site.literal.version,)}, which the shared floor >=${highest.text} for ${name} does not allow`,
        remedy: namedCatalogRemedy({ site, requirement: site.literal.version, },),
      },);
    }
  }
  if (issues.length !== issuesBefore)
    return [];
  /**
   Rename shared by the group.
   */
  const renames = new Set([...literals, ...existing,].map(function toRename(item,): string {
    return 'literal' in item ? item.literal.package ?? '' : item.package ?? '';
  },),);
  if (renames.size > 1) {
    issues.push({
      file: literals[0]?.manifestPath ?? '',
      location: name,
      problem: `crates rename ${name} to different packages`,
      remedy: 'give each renamed package its own dependency name',
    },);
    return [];
  }
  /**
   The single rename, empty when none.
   */
  const [rename = '',] = renames;
  return [{ kind: 'registry', version: `>=${highest.text}`, ...(rename === '' ? {} : { package: rename, }), },];
}

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
   Literal sites, split by whether their crate selects a named catalog.
   */
  const literals = sites.filter(function isLiteral(site,): boolean {
    return site.kind === 'literal';
  },);
  /**
   Whether a site's crate selects a named catalog for it.
   */
  function isSelected(site: UseSite,): boolean {
    return selections.get(site.manifestPath,)?.has(site.name,) === true;
  }
  checkSelectedLiterals({ document, sites: literals.filter(isSelected,), selections, issues, },);

  /**
   Default-catalog literals grouped by name.
   */
  const groups = Map.groupBy(literals.filter(function isDefault(site,): boolean {
    return !isSelected(site,);
  },), function byName(site,): string {
    return site.name;
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
     Current default entry, as a zero- or one-item list.
     */
    const existing = [...document.catalog.entries(),].filter(function isName([key,],): boolean {
      return key === name;
    },).map(function toEntry([, entry,],): RustCatalogEntry {
      return entry;
    },);
    /**
     Distinct literal kinds in the group, plus the existing entry's kind.
     */
    const kinds = new Set([...group.map(function toKind(site,): string {
      return site.literal.kind;
    },), ...existing.map(function toKind(entry,): string {
      return entry.kind;
    },),],);
    if (kinds.size > 1) {
      issues.push({
        file: group[0]?.manifestPath ?? '',
        location: name,
        problem: `${name} is a git dependency in some places and a registry dependency in others`,
        remedy: 'use one source for every crate, or give the other source its own named catalog',
      },);
      continue;
    }
    if (kinds.has('git')) {
      /**
       The pin every git literal must share.
       */
      const pin = existing[0] ?? group[0]?.literal;
      if ((pin === undefined) || !group.every(function matchesPin(site,): boolean {
        return sameEntry({ left: site.literal, right: pin, },);
      },)) {
        issues.push({ file: group[0]?.manifestPath ?? '', location: name, problem: `crates pin ${name} to different git revisions`, remedy: 'pin every crate to one `rev`', },);
        continue;
      }
      if (existing.length === 0) {
        additions.set(name, pin,);
        defaultCatalog.set(name, pin,);
      }
      continue;
    }
    /**
     Planned default entry for the registry group.
     */
    const [planned,] = planRegistryGroup({ name, literals: group, existing, issues, },);
    if (planned === undefined)
      continue;
    if (existing.length === 0)
      additions.set(name, planned,);
    else if (!sameEntry({ left: planned, right: existing[0] ?? planned, },) && planned.kind === 'registry')
      floorRaises.set(name, planned.version,);
    defaultCatalog.set(name, planned,);
  }
  return { additions, floorRaises, defaultCatalog, };
}

//endregion Default catalog
