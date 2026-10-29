/**
 Plans the default-catalog `>=` floor for one group of registry literals sharing a dependency name.

 @module
 */

import {
  compareVersions,
  formatVersion,
  type ParsedVersion,
  type RustCatalogEntry,
  satisfies,
  splitRequirement,
} from '@monochromatic-dev/dependency-catalog-core-rust/ts';

import {
  namedCatalogRemedy,
  siteLocation,
} from './adopt-shared.ts';
import type {
  SyncIssue,
  UseSite,
} from './types.ts';

//region Candidates

/**
 Version text and parse of one candidate floor.
 */
type FloorCandidate = {
  readonly text: string;
  readonly parsed: ParsedVersion;
};

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
  for (const operator of [
    '>=',
    '^',
    '~',
  ]) {
    if (trimmed.startsWith(operator,))
      return trimmed.slice(operator.length,)
        .trim();
  }
  return trimmed;
}

/**
 Candidate floors from literal requirements; exact pins and unreadable requirements become issues.

 @param literals - registry literal sites

 @param issues - collector the problems are appended to

 @returns candidates in site order

 @example
 ```ts
 literalCandidates({ literals, issues, });
 ```
 */
function literalCandidates({
  literals,
  issues,
}: {
  readonly literals: readonly UseSite[];
  readonly issues: SyncIssue[]
},): readonly FloorCandidate[] {
  /**
   Candidates collected so far.
   */
  const candidates: FloorCandidate[] = [];
  for (const site of literals) {
    if (site.literal
      .kind
      !== 'registry')
      continue;
    try {
      /**
       Operator and base version of the literal.
       */
      const {
        operator,
        version,
      } = splitRequirement(site.literal
        .version,);
      if (operator === '=') {
        issues.push({
          file: site.manifestPath,
          location: siteLocation(site,),
          problem: `exact pin ${JSON.stringify(site.literal
            .version,)} cannot become a \`>=\` floor in [catalog]`,
          remedy: namedCatalogRemedy({
            site,
            requirement: site.literal
              .version,
          },),
        },);
        continue;
      }
      candidates.push({
        text: stripOperator(site.literal
          .version,),
        parsed: version,
      },);
    } catch (error) {
      issues.push({
        file: site.manifestPath,
        location: siteLocation(site,),
        problem: `requirement ${JSON.stringify(site.literal
          .version,)} is not one comparator the catalog can adopt (${String(error,)})`,
        remedy: 'write a single requirement such as `"1.2"`, `"~1.2.3"`, or `">=1.2"`',
      },);
    }
  }
  return candidates;
}

//endregion Candidates

//region Plan

/**
 Plans one default-catalog group of registry literals.

 @param name - dependency name

 @param literals - registry literal sites

 @param existing - current default entry, as a zero- or one-item list

 @param issues - collector the problems are appended to

 @returns the entry the default catalog should hold, or none when the group conflicts

 @example
 ```ts
 planRegistryGroup({ name: 'tokio', literals, existing: [], issues, });
 ```
 */
export function planRegistryGroup(
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
  const candidates = [...literalCandidates({
    literals,
    issues,
  },),];
  for (const entry of existing) {
    if (entry.kind === 'registry')
      candidates.push({
        text: stripOperator(entry.version,),
        parsed: splitRequirement(entry.version,)
          .version,
      },);
  }
  if ((issues.length !== issuesBefore) || (candidates.length === 0))
    return [];

  /**
   Highest candidate, the floor every literal must accept.
   */
  const highest = candidates.reduce(function pickHigher(
    best: FloorCandidate,
    candidate: FloorCandidate,
  ): FloorCandidate {
    return compareVersions({
      left: candidate.parsed,
      right: best.parsed,
    },) > 0 ? candidate : best;
  },);
  for (const site of literals) {
    if ((site.literal
      .kind
      === 'registry') && (!satisfies({
        version: formatVersion(highest.parsed,),
        requirement: site.literal
          .version,
      },))) {
      issues.push({
        file: site.manifestPath,
        location: siteLocation(site,),
        problem: `requires ${JSON.stringify(site.literal
          .version,)}, which the shared floor >=${highest.text} for ${name} does not allow`,
        remedy: namedCatalogRemedy({
          site,
          requirement: site.literal
            .version,
        },),
      },);
    }
  }
  if (issues.length !== issuesBefore)
    return [];

  /**
   Renames declared across the group and the existing entry.
   */
  const renames = new Set<string>();
  for (const site of literals)
    renames.add(site.literal
      .package
      ?? '',);
  for (const entry of existing)
    renames.add(entry.package ?? '',);
  if (renames.size > 1) {
    issues.push({
      file: literals[0]
        ?.manifestPath
        ?? '',
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
  return [{
    kind: 'registry',
    version: `>=${highest.text}`,
    ...(rename === '' ? {} : { package: rename, }),
  },];
}

//endregion Plan
