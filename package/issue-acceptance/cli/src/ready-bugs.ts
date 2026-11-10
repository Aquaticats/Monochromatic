/**
 Groups the open issues a batch agent may take by the package they name.

 Parallel agents must stay out of each other's files,
 so the grouping names one package per issue:
 the first package path the issue's title or body mentions.

 @module
 */

import type { IssueListing, } from './issue-tracker.ts';

/**
 Labels an issue needs before a batch agent may take it.
 */
export const READY_BUG_LABELS: readonly string[] = [
  'bug',
  'ready-for-agent',
];

/**
 Words that put an issue out of reach until its subject is unpaused,
 because those subjects cannot be worked on from this host.
 */
// oxlint-disable-next-line no-restricted-syntax/no-regex -- one decision over a bounded issue title; the alternatives are three disjoint substring scans plus a two-word phrase, and this pattern has no nested quantifier, so it matches in linear time. Justification recorded in this package's README.
const PAUSED_WORDS = /\b(?:windows|macos|mac os|lfs)\b/iu;

/**
 First package path an issue's text names.
 */
// oxlint-disable-next-line no-restricted-syntax/no-regex -- one match over one issue's text, bounded by that issue; no nested quantifier, so it matches in linear time. Justification recorded in this package's README.
const PACKAGE_PATH = /package\/[a-z0-9-]+(?:\/[a-z0-9.-]+)?/u;

/**
 Group name for issues that name no package.
 */
const NO_PACKAGE = '(no package named)';

/**
 One package and the issues that named it.
 */
export type ReadyBugGroup = {
  readonly packagePath: string;
  readonly issues: readonly number[];
};

/**
 What the grouping found.
 */
export type ReadyBugReport = {
  readonly considered: number;
  readonly paused: number;
  readonly groups: readonly ReadyBugGroup[];
};

/**
 Groups open issues by the first package each names.
 
 @param listings - Open issues carrying the ready labels.
 
 @returns Groups ordered by issue count, then by package path.
 
 @example
 ```ts
 const report = groupReadyBugs({ listings: [{ number: 1, title: 'package/a/x', body: '' }] });
 ```
 */
export function groupReadyBugs({
  listings,
}: {
  readonly listings: readonly IssueListing[];
},): ReadyBugReport {
  /**
   Issues whose subject is not paused, and so may be handed out.
   */
  const ready = listings.filter(function isReady(listing,): boolean {
    return !PAUSED_WORDS.test(listing.title,);
  },);
  /**
   Issue numbers grouped by the first package each issue names.
   */
  const byPackage = new Map<string, number[]>();
  for (const listing of ready) {
    /**
     First package path the issue's text names, when it names one.
     */
    const match = PACKAGE_PATH.exec(`${listing.title}\n${listing.body}`,);
    /**
     Group this issue joins.
     */
    const packagePath = match === null ? NO_PACKAGE : match[0];
    /**
     Issues already grouped under that package.
     */
    const members = byPackage.get(packagePath,) ?? [];
    members.push(listing.number,);
    byPackage.set(
      packagePath,
      members,
    );
  }
  /**
   Groups by package, ordered by size and then by path.
   */
  const groups = [...byPackage]
    .map(function toGroup(entry: readonly [
      string,
      readonly number[]
    ],): ReadyBugGroup {
      return {
        packagePath: entry[0],
        issues: entry[1],
      };
    },)
    .toSorted(function bySizeThenPath(
      left,
      right,
    ): number {
      return (right.issues
        .length
        - left.issues
          .length)
        || left.packagePath
          .localeCompare(right.packagePath,);
    },);
  return {
    considered: listings.length,
    paused: listings.length - ready.length,
    groups,
  };
}

/**
 Renders the grouping as one header line and one line per package.
 
 @param report - Grouping to render.
 
 @returns Lines a batch session reads to hand out one package at a time.
 
 @example
 ```ts
 const lines = renderReadyBugReport({ report });
 ```
 */
export function renderReadyBugReport({ report, }: { readonly report: ReadyBugReport; },): readonly string[] {
  return [
    `issues ${String(report.considered,)}, left out as paused ${String(report.paused,)}, packages ${String(report.groups
      .length,)}`,
    ...report.groups
      .map(function toLine(group,): string {
        return `${String(group.issues
          .length,)}\t${group.packagePath}\t${group.issues
            .join(' ',)}`;
      },),
  ];
}
