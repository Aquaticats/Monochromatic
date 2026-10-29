/**
 Formats the diagnosis shown when pnpm's strict `minimumReleaseAge` gate
 refuses `pnpm update --no-save`.

 @module
 */

import {
  NO_MINIMUM_RELEASE_AGE,
} from './exclude-list.ts';

//region Types

/**
 One version the age gate would block, with the facts needed to decide.
 */
export type ImmaturePick = {
  /**
   npm package name.
   */
  readonly name: string;
  /**
   Exact version pnpm picked.
   */
  readonly version: string;
  /**
   Registry publish instant.
   */
  readonly publishedAt: Date;
  /**
   `name@version` of packages that depend on this pick directly.
   */
  readonly dependents: readonly string[];
};

//endregion Types

//region Format

/**
 Milliseconds per minute, for converting `minimumReleaseAge`.
 */
const MS_PER_MINUTE = 60_000;

/**
 Computes when a pick passes the age gate.

 @param publishedAt - registry publish instant

 @param minutes - configured `minimumReleaseAge`

 @returns instant the pick stops being immature

 @example
 ```ts
 matureAt({ publishedAt: new Date(0), minutes: 1 }); // 1970-01-01T00:01:00.000Z
 ```
 */
export function matureAt({
  publishedAt,
  minutes,
}: {
  readonly publishedAt: Date;
  readonly minutes: number;
},): Date {
  return new Date(publishedAt.getTime() + (minutes
    * MS_PER_MINUTE),);
}

/**
 Renders the report for immature picks, including both remediation paths.

 @param picks - immature picks, already sorted by caller preference

 @param minutes - configured `minimumReleaseAge`, or its absence sentinel

 @param followUp - pnpm diagnostic the resolution hit after the age check, when it hit one

 @returns multi-line report text

 @example
 ```ts
 formatImmatureReport({ picks, minutes: 1440 });
 ```
 */
export function formatImmatureReport({
  picks,
  minutes,
  followUp,
}: {
  readonly picks: readonly ImmaturePick[];
  readonly minutes: number | typeof NO_MINIMUM_RELEASE_AGE;
  readonly followUp?: string;
},): string {
  if (picks.length === 0) {
    return [
      'pnpm refused `update --no-save` under minimumReleaseAgeStrict,',
      'but a loose-mode resolution of the same workspace appended no minimumReleaseAgeExclude entries.',
      'Rerun `pnpm update --recursive` in a terminal to see pnpm\'s approval prompt.',
    ].join('\n',);
  }
  /**
   One block per pick: spec, publish time, maturity, and who pulls it in.
   */
  const blocks = picks.map(function formatPick(pick,): string {
    /**
     Maturity line; omitted detail when the age is not configured.
     */
    const maturity = minutes === NO_MINIMUM_RELEASE_AGE
      ? '  passes the age gate: unknown (minimumReleaseAge unset)'
      : `  passes the age gate: ${matureAt({
        publishedAt: pick.publishedAt,
        minutes,
      },)
        .toISOString()}`;
    /**
     Dependents line; a pick with none is a direct workspace dependency.
     */
    const pulledBy = pick.dependents
      .length
      === 0
      ? '  pulled in by: a workspace package directly'
      : `  pulled in by: ${pick.dependents
        .join(', ',)}`;
    return [
      `${pick.name}@${pick.version}`,
      `  published: ${pick.publishedAt
        .toISOString()}`,
      maturity,
      pulledBy,
    ].join('\n',);
  },);
  /**
   Latest maturity instant across picks; rerunning after it succeeds.
   */
  const retryAfter = minutes === NO_MINIMUM_RELEASE_AGE
    ? undefined
    : new Date(
      Math.max(...picks.map(function pickMaturity(pick,): number {
        return matureAt({
          publishedAt: pick.publishedAt,
          minutes,
        },)
          .getTime();
      },),),
    );
  return [
    `pnpm update refused ${String(picks.length,)} version(s) younger than minimumReleaseAge:`,
    '',
    ...blocks,
    '',
    'Choose one:',
    '  - Trust the publisher: add the package name (or its scope glob) to',
    '    minimumReleaseAgeExclude in pnpm-workspace.yaml, then rerun.',
    retryAfter === undefined
      ? '  - Wait until every pick above passes the age gate, then rerun.'
      : `  - Wait: rerun after ${retryAfter.toISOString()}.`,
    ...(followUp === undefined
      ? []
      : [
        '',
        'The update will then stop on a separate failure the scratch resolution also hit:',
        `  ${followUp}`,
      ]),
  ].join('\n',);
}

//endregion Format
