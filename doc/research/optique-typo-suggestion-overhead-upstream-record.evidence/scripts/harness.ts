// Builds the callable that parses one cell with one arm's installed Optique,
// plus the validation that proves the call did the work. Shared by the timed
// runner and the profiling runner so both measure the identical call.

import { createHash, } from 'node:crypto';
import { pathToFileURL, } from 'node:url';

import type { CommitRegionFacts, } from './workloads.ts';
import {
  commitFactsAreValid,
  commitRegionArgs,
} from './workloads.ts';

//region Types

/**
 * What validation reports about the last parse.
 */
export type Inspection = {
  readonly ok: boolean;
  readonly digest: string;
  readonly handled: number;
};

/**
 * One arm on one cell: a repeatable parse call plus the validation that proves
 * the call did the work.
 */
export type Harness = {
  readonly run: () => unknown;
  readonly inspect: (result: unknown,) => Inspection;
};

//endregion Types

//region Helpers

/**
 * Hashes a JSON-serialisable value, so arms can be compared without shipping
 * full fact records.
 *
 * @param value - Value to hash.
 *
 * @returns Hex SHA-256 digest.
 */
export function digestOf(value: unknown,): string {
  return createHash('sha256',)
    .update(JSON.stringify(value,),)
    .digest('hex',);
}

//endregion Helpers

//region Builders

/**
 * Builds the commit-region harness from an arm's unchanged parser sources.
 *
 * @param arm - Arm label, which is also the npm version.
 *
 * @param workload - Commit-region workload to run.
 *
 * @param paths - Paths the workload appends.
 *
 * @returns Harness over `parseCommitRegion`.
 */
async function buildCommitRegionHarness(
  arm: string,
  workload: 'no-separator' | 'separator',
  paths: readonly string[],
): Promise<Harness> {
  /** Arm-local parser module URL. */
  const moduleUrl = pathToFileURL(
    `/bench/arms/${arm}/src/parser/commit.ts`,
  ).href;
  /** Arm-local parser module. */
  const module = await import(moduleUrl,) as {
    parseCommitRegion: (args: readonly string[],) => CommitRegionFacts;
  };
  /** Argv reused across parses. */
  const args = commitRegionArgs(
    workload,
    paths,
  );

  return {
    run() {
      return module.parseCommitRegion(args,);
    },
    inspect(result,) {
      /** Facts returned by the last parse. */
      const facts = result as CommitRegionFacts;

      return {
        ok: commitFactsAreValid(
          paths.length,
          facts,
        ),
        digest: digestOf(facts,),
        handled: facts.pathspecs.length,
      };
    },
  };
}

/**
 * Builds the issue-988 harness from an arm's installed Optique.
 *
 * @param arm - Arm label, which is also the npm version.
 *
 * @param paths - Long distinct paths the reproduction parses.
 *
 * @returns Harness over the issue-988 parser.
 */
async function buildIssue988Harness(arm: string, paths: readonly string[],): Promise<Harness> {
  /** Arm-local reproduction module URL. */
  const moduleUrl = pathToFileURL(
    `/bench/arms/${arm}/bench/issue988-parser.ts`,
  ).href;
  /** Arm-local reproduction module. */
  const module = await import(moduleUrl,) as {
    createIssue988Runner: () => (pathsIn: readonly string[],) => {
      readonly success: boolean;
      readonly handled: number;
    };
  };
  /** Reproduction runner for this arm. */
  const runner = module.createIssue988Runner();

  return {
    run() {
      return runner(paths,);
    },
    inspect(result,) {
      /** Outcome of the last parse. */
      const outcome = result as {
        readonly success: boolean;
        readonly handled: number;
      };

      return {
        ok: outcome.success && outcome.handled === paths.length,
        digest: digestOf(outcome,),
        handled: outcome.handled,
      };
    },
  };
}

/**
 * Builds the harness for one arm and one cell.
 *
 * @param arm - Arm label, which is also the npm version.
 *
 * @param cell - Workload, path set, and path count.
 *
 * @param paths - The cell's path prefix.
 *
 * @returns Harness for that arm and cell.
 */
export async function buildHarness(
  arm: string,
  cell: {
    readonly workload: string;
    readonly pathSet: string;
    readonly count: number;
  },
  paths: readonly string[],
): Promise<Harness> {
  if (cell.workload === 'issue988') {
    return buildIssue988Harness(
      arm,
      paths,
    );
  }

  if (cell.workload !== 'no-separator' && cell.workload !== 'separator') {
    throw new Error(`unknown workload: ${cell.workload}`,);
  }

  return buildCommitRegionHarness(
    arm,
    cell.workload,
    paths,
  );
}

//endregion Builders
