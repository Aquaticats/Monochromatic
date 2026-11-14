// Records the exact CommitRegion record the old parser returns on every arm,
// for every timed commit-region workload plus a set of option-shaped inputs.
// Also records the two upstream issue reproductions, so a behavior change in
// the library can be told apart from a change the wrapper's facts can see.

import { createHash, } from 'node:crypto';
import { pathToFileURL, } from 'node:url';

import type { CommitRegionFacts, } from './workloads.ts';
import {
  CELLS,
  commitRegionArgs,
  loadPaths,
  OWNED_PARSER_ARM,
} from './workloads.ts';

//region Inputs

/**
 * One fixed input compared across versions.
 */
type BehaviorCase = {
  readonly name: string;
  readonly argv: readonly string[];
};

/**
 * Option-shaped argv that the two behavior-changing releases touch: joined
 * long options, unknown flags next to paths, known options missing a value,
 * and the wrapper's own inline-cluster and escape-hatch spellings.
 */
const OPTION_CASES: readonly BehaviorCase[] = [
  {
    name: 'unknown-joined-long',
    argv: [
      '--unknown=v',
      'a.txt',
    ],
  },
  {
    name: 'unknown-joined-long-then-flag',
    argv: [
      '--unknown=v',
      '--amend',
      'a.txt',
    ],
  },
  {
    name: 'unknown-separated',
    argv: [
      '--unknown',
      'v',
      'a.txt',
    ],
  },
  {
    name: 'unknown-short-then-path',
    argv: [
      '-q',
      'a.txt',
    ],
  },
  {
    name: 'known-joined-long',
    argv: [
      '--message=hello',
      'a.txt',
    ],
  },
  {
    name: 'known-joined-long-empty-value',
    argv: [
      '--message=',
      'a.txt',
    ],
  },
  {
    name: 'known-missing-value-long',
    argv: ['--message',],
  },
  {
    name: 'known-missing-value-short',
    argv: ['-m',],
  },
  {
    name: 'known-missing-value-short-then-path',
    argv: [
      '-m',
      'a.txt',
    ],
  },
  {
    name: 'inline-cluster',
    argv: [
      '-am',
      'hello',
      'a.txt',
    ],
  },
  {
    name: 'escape-hatch',
    argv: [
      '--no-enforce-only',
      'a.txt',
    ],
  },
  {
    name: 'pathspec-file-joined',
    argv: ['--pathspec-from-file=list.txt',],
  },
  {
    name: 'separator-only',
    argv: [
      '--',
      'a.txt',
    ],
  },
  {
    name: 'joined-after-separator',
    argv: [
      '--',
      '--unknown=v',
      'a.txt',
    ],
  },
  {
    name: 'amend-and-path',
    argv: [
      '--amend',
      'a.txt',
    ],
  },
];

//endregion Inputs

//region Records

/**
 * Normalised shape used to hold a fact record even when it lists a thousand
 * pathspecs: long lists are replaced by their count plus a content digest,
 * which keeps equality exact.
 */
type CompactFacts = {
  readonly [key: string]: unknown;
};

/**
 * Hashes a JSON-serialisable value.
 *
 * @param value - Value to hash.
 *
 * @returns Hex SHA-256 digest.
 */
function digestOf(value: unknown,): string {
  return createHash('sha256',)
    .update(JSON.stringify(value,),)
    .digest('hex',);
}

/**
 * Shrinks a fact record for transport while keeping equality decidable.
 *
 * @param facts - Facts returned by `parseCommitRegion`.
 *
 * @returns Facts with long pathspec lists replaced by count and digest.
 */
function compactFacts(facts: CommitRegionFacts,): CompactFacts {
  /** Pathspec list of this record. */
  const { pathspecs, } = facts;

  if (pathspecs.length <= 4) {
    return { ...facts, };
  }

  return {
    ...facts,
    pathspecs: {
      count: pathspecs.length,
      digest: digestOf(pathspecs,),
    },
  };
}

//endregion Records

//region Entry point

/**
 * Records one arm's behavior on every case and prints one JSON line per case
 * plus one line of upstream probe outcomes.
 */
async function main(): Promise<void> {
  /** Arm label from `--arm=`. */
  const armArgument = process.argv
    .slice(2,)
    .find(function isArm(argument,) {
      return argument.startsWith('--arm=',);
    },);

  if (armArgument === undefined) {
    throw new Error('missing --arm=<version>',);
  }

  /** Arm label, which is also the npm version. */
  const arm = armArgument.slice('--arm='.length,);
  /** Arm-local commit parser module. */
  const commitModule = await import(
    pathToFileURL(`/bench/arms/${arm}/src/parser/commit.ts`,).href,
  ) as {
    parseCommitRegion: (args: readonly string[],) => CommitRegionFacts;
  };
  /** Arm-local issue reproduction module, absent on the baseline arm. */
  const probeModule = arm === OWNED_PARSER_ARM
    ? undefined
    : await import(
      pathToFileURL(`/bench/arms/${arm}/bench/upstream-probes.ts`,).href,
    ) as {
      runUpstreamProbes: () => Readonly<Record<string, unknown>>;
    };

  /** Task-shaped cases for every commit-region cell. */
  const cellCases: BehaviorCase[] = CELLS
    .filter(function isCommitRegion(cell,) {
      return cell.workload !== 'issue988';
    },)
    .map(function asCase(cell,) {
      return {
        name: `workload/${cell.workload}/${cell.pathSet}/${cell.count}`,
        argv: commitRegionArgs(
          cell.workload === 'separator' ? 'separator' : 'no-separator',
          loadPaths(cell.pathSet,)
            .slice(
              0,
              cell.count,
            ),
        ),
      };
    },);

  for (const behaviorCase of [
    ...cellCases,
    ...OPTION_CASES,
  ]) {
    /** Outcome of this input: the fact record, or the error it threw. */
    let outcome: Record<string, unknown>;

    try {
      outcome = { facts: compactFacts(
        commitModule.parseCommitRegion(behaviorCase.argv,),
      ), };
    }
    catch (error) {
      /** Thrown value, which the owned parser uses for argv it rejects. */
      const thrown = error as Error;

      outcome = {
        threw: {
          name: thrown.name,
          message: thrown.message,
        },
      };
    }

    process.stdout.write(`${
      JSON.stringify({
        kind: 'behavior',
        arm,
        case: behaviorCase.name,
        outcome,
      },)
    }\n`,);
  }

  if (probeModule !== undefined) {
    process.stdout.write(`${
      JSON.stringify({
        kind: 'upstream',
        arm,
        probes: probeModule.runUpstreamProbes(),
      },)
    }\n`,);
  }
}

await main();

//endregion Entry point
