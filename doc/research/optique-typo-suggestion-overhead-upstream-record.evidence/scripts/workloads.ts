// Shared vocabulary for the Optique measurement image: arms (one per
// @optique/core version), measured cells, argv construction, and result
// validation. Nothing here imports @optique; each arm imports its own copy.

import { readFileSync, } from 'node:fs';

//region Types

/**
 * Name of a measured workload.
 *
 * - `no-separator`: the commit parser sees the pathspec tokens, which is the
 *   shape that made the wrapper slow.
 * - `separator`: every path sits after `--`, so the parser never sees them.
 *   Control that must stay fast in every version.
 * - `issue988`: the reproduction in upstream issue 988, a root parser with
 *   thirty options and a thousand long distinct paths.
 */
export type WorkloadName = 'no-separator' | 'separator' | 'issue988';

/**
 * Name of a baked path list.
 */
export type PathSetName = 'real' | 'short' | 'issue988';

/**
 * One measured configuration: workload, path list, and path count.
 */
export type Cell = {
  readonly workload: WorkloadName;
  readonly pathSet: PathSetName;
  readonly count: number;
};

/**
 * Facts the commit parser returns, described only by the fields this
 * measurement compares across versions.
 */
export type CommitRegionFacts = {
  readonly hasAllFlag: boolean;
  readonly hasPathspec: boolean;
  readonly pathspecs: readonly string[];
};

//endregion Types

//region Config

/**
 * Every measured `@optique/core` version, in release order. `1.4.0-dev.2664`
 * is the last dev build published before pull request 997's only commit
 * landed, which makes it the arm that measures that change.
 */
export const ARMS: readonly string[] = [
  '1.1.1',
  '1.3.1',
  '1.3.2',
  '1.3.6',
  '1.4.0-dev.2664',
  '1.4.0-dev.2665',
  '1.4.0-dev.2667',
  '1.4.0-dev.2971',
];

/**
 * Arm label of the repository-owned parser that replaced Optique at this
 * migration commit. It has no Optique at all, so it is the baseline every
 * version is measured against.
 */
export const OWNED_PARSER_ARM = 'owned-4879a44e';

/**
 * Every measured arm: the Optique versions, then the owned-parser baseline.
 */
export const MEASURED_ARMS: readonly string[] = [
  ...ARMS,
  OWNED_PARSER_ARM,
];

/**
 * Arms the scaling experiment measures: the version before the 1.3.2 patch,
 * the patched stable release, the 1.4 line at the patch level, and the 1.4
 * line after deferred diagnostics.
 */
export const SCALING_ARMS: readonly string[] = [
  '1.1.1',
  '1.3.6',
  '1.4.0-dev.2664',
  '1.4.0-dev.2971',
];

/**
 * Arms that exist only to measure one suggestion search: 0.7.0 is the first
 * release with the feature, and it predates the API the commit parser uses.
 */
export const SEARCH_ARMS: readonly string[] = ['0.7.0'];

/**
 * Every arm directory the image must hold.
 */
export const IMAGE_ARMS: readonly string[] = [
  ...MEASURED_ARMS,
  ...SEARCH_ARMS,
];

/**
 * Whether an arm can run a cell. The owned parser has no Optique, so it cannot
 * run the upstream issue-988 reproduction, which is written against Optique.
 *
 * @param arm - Arm label.
 *
 * @param cell - Cell to check.
 *
 * @returns Whether this arm measures this cell.
 */
export function cellIsSupportedByArm(arm: string, cell: Cell,): boolean {
  return !(arm === OWNED_PARSER_ARM && cell.workload === 'issue988');
}

/**
 * Path counts spanning three orders of magnitude up to a thousand.
 */
export const PATH_COUNTS: readonly number[] = [
  1,
  10,
  100,
  1000,
];

/**
 * Every measured cell, generated so no cell is silently dropped.
 */
export const CELLS: readonly Cell[] = [
  ...PATH_COUNTS.map(function commitWithoutSeparator(count,): Cell {
    return {
      workload: 'no-separator',
      pathSet: 'real',
      count,
    };
  },),
  ...PATH_COUNTS.map(function commitWithSeparator(count,): Cell {
    return {
      workload: 'separator',
      pathSet: 'real',
      count,
    };
  },),
  ...PATH_COUNTS.map(function shortWithoutSeparator(count,): Cell {
    return {
      workload: 'no-separator',
      pathSet: 'short',
      count,
    };
  },),
  ...PATH_COUNTS.map(function shortWithSeparator(count,): Cell {
    return {
      workload: 'separator',
      pathSet: 'short',
      count,
    };
  },),
  {
    workload: 'issue988',
    pathSet: 'issue988',
    count: 1000,
  },
];

//endregion Config

//region Path lists

/**
 * Reads a baked path list from the image.
 *
 * @param pathSet - Which list to read.
 *
 * @returns Every path in the list.
 */
export function loadPaths(pathSet: PathSetName,): readonly string[] {
  /** Baked list file for this path set. */
  const fileName = `${pathSet}-paths.json`;
  /** Parsed list. */
  const parsed: unknown = JSON.parse(
    readFileSync(
      `/bench/data/${fileName}`,
      'utf8',
    ),
  );

  if (!Array.isArray(parsed,)) {
    throw new TypeError(`${fileName} does not hold a JSON array.`,);
  }

  return parsed.map(function asString(value,) {
    if (typeof value !== 'string') {
      throw new TypeError(`${fileName} holds a non-string entry.`,);
    }
    return value;
  },);
}

/**
 * Builds the argv handed to `parseCommitRegion` for a commit-region workload.
 *
 * @param workload - Which commit-region workload to build.
 *
 * @param paths - Paths to append, in list order.
 *
 * @returns Argv tokens after the `commit` subcommand.
 */
export function commitRegionArgs(
  workload: 'no-separator' | 'separator',
  paths: readonly string[],
): readonly string[] {
  /** Message option both workloads carry. */
  const messageArgs = [
    '--message',
    'x',
  ];

  return workload === 'separator'
    ? [
      ...messageArgs,
      '--',
      ...paths,
    ]
    : [
      ...messageArgs,
      ...paths,
    ];
}

//endregion Path lists

//region Validation

/**
 * Checks that a parse produced the facts this workload must produce, so a
 * version that silently fails to parse cannot be timed as if it succeeded.
 *
 * @param count - Number of paths the argv carried.
 *
 * @param facts - Facts returned by `parseCommitRegion`.
 *
 * @returns Whether the facts match the expected shape.
 */
export function commitFactsAreValid(
  count: number,
  facts: CommitRegionFacts,
): boolean {
  /** Whether paths silently dropped out of the parse. */
  const pathspecsComplete = facts.pathspecs.length === count;
  /** Whether the parse agreed that a pathspec is present. */
  const pathspecSeen = facts.hasPathspec === (count > 0);

  return pathspecsComplete && pathspecSeen && facts.hasAllFlag === false;
}

//endregion Validation
