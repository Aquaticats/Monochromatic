/**
 Scenario contract and runner:
 fresh repositories per scenario,
 the scenario's workload,
 observation,
 and the invariant check.

 @module
 */

import { mkdir, } from 'node:fs/promises';
import { join, } from 'node:path';
import { setTimeout as sleep, } from 'node:timers/promises';

import { caughtValueStack, } from '@monochromatic-dev/caught-value-core/ts';

import type { CommitShapeTrace, } from './commit-shape-trace-fixture.ts';
import type {
  RunObservation,
  Violation,
} from './invariant-model-fixture.ts';
import { checkInvariants, } from './invariant-fixture.ts';
import {
  type AttemptRecord,
  createLedger,
  workloadDigest,
} from './ledger-fixture.ts';
import { observeRun, } from './observation-fixture.ts';
import { startScenarioService, } from './service-fixture.ts';
import { killActiveGroups, } from './process-fixture.ts';
import {
  createScenarioRepository,
  type RepositoryOptions,
} from './repository-fixture.ts';
import {
  createSeededRandom,
  deriveSeed,
  type SeededRandom,
} from './seeded-random-fixture.ts';
import type { ScenarioActors, } from './worker-fixture.ts';
import {
  compareVersions,
  REPLAY_PLUMBING_FITS,
  REPLAY_PLUMBING_GIT,
  type ReplayPlumbingDependency,
  replayPlumbingSkip,
} from './git-version-fixture.ts';

export {
  compareVersions,
  REPLAY_PLUMBING_GIT,
} from './git-version-fixture.ts';

export {
  allSucceeded,
  isAlreadyCommitted,
  isReportedConflict,
  landedOrConflicted,
} from './scenario-expectation-fixture.ts';

//region Types

/**
 Scenario family:
 single-commit baselines prove the harness,
 concurrency scenarios exercise the accepted design.
 */
export type ScenarioGroup = 'baseline' | 'concurrency';

/**
 Handles a scenario workload receives.
 */
export type ScenarioContext = ScenarioActors & Readonly<{
  /**
   Scenario-local seeded source.
   */
  random: SeededRandom;
  /**
   Committed commit-shape trace.
   */
  trace: CommitShapeTrace;
  /**
   Git version under test.
   */
  gitVersion: string;
}>;

/**
 Scenario-specific expectations of the accepted design.
 */
export type Expectations = RunObservation['expectations'];

/**
 One catalog entry.
 */
export type ScenarioDefinition = Readonly<{
  /**
   Unique name,
   usable as a filter.
   */
  name: string;
  /**
   Family.
   */
  group: ScenarioGroup;
  /**
   One-line description for the report.
   */
  summary: string;
  /**
   Oldest Git version the scenario needs,
   when newer than the suite minimum.
   */
  minimumGit?: string;
  /**
   How the scenario depends on replay plumbing,
   `git merge-tree --merge-base` from Git {@link REPLAY_PLUMBING_GIT} on:
   `required`,
   the default,
   skips it on an older Git,
   where a commit that loses a landing race fails fast instead of replaying;
   `unused` runs it on every Git,
   because no commit in it can lose a landing race;
   `absent` runs it only on an older Git,
   to prove that degradation.
   */
  replayPlumbing?: ReplayPlumbingDependency;
  /**
   Invariants a positive-control scenario must violate, exactly;
   the scenario passes only when the checker reports this set.
   */
  expectedViolations?: readonly string[];
  /**
   Repository setup derived from the scenario's seeded source.
   */
  repository: (random: SeededRandom,) => RepositoryOptions;
  /**
   Workload;
   resolves with scenario-specific expectations.
   */
  run: (context: ScenarioContext,) => Promise<Expectations>;
}>;

/**
 Result of one scenario on one Git version.
 */
export type ScenarioResult = Readonly<{
  /**
   Scenario name.
   */
  name: string;
  /**
   Family.
   */
  group: ScenarioGroup;
  /**
   Git version.
   */
  gitVersion: string;
  /**
   Outcome:
   invariants held,
   invariants violated,
   not applicable to this Git,
   or the harness itself failed.
   */
  status: 'error' | 'fail' | 'pass' | 'skip';
  /**
   Violated invariants.
   */
  violations: readonly Violation[];
  /**
   Wall-clock duration.
   */
  durationMs: number;
  /**
   Skip reason or harness error.
   */
  detail?: string;
  /**
   Workload digest from {@link workloadDigest};
   absent for skipped scenarios.
   */
  workload?: string;
  /**
   Attempt outcomes for the report.
   */
  attempts: readonly Readonly<{
    label: string;
    exitCode: number;
    killed: boolean;
    note?: string;
  }>[];
}>;

//endregion Types

//region Runner

/**
 Longest note the report prints for an attempt that failed.
 */
const NOTE_LENGTH = 400;

/**
 Upper bound for one scenario's workload, in milliseconds.
 */
export const SCENARIO_TIMEOUT_MS = 180_000;

/**
 Decides a scenario's verdict.
 Ordinary scenarios pass with no violations;
 positive controls pass when the violated invariant names equal `expectedViolations`.

 @param definition - catalog entry

 @param violations - checker output

 @returns verdict, reported violations, and detail

 @example
 ```ts
 judge({ definition, violations: [] }).passed; // => true for ordinary scenarios
 ```
 */
export function judge({
  definition,
  violations,
}: Readonly<{
  definition: Pick<ScenarioDefinition, 'expectedViolations'>;
  violations: readonly Violation[];
}>,): Readonly<{
  passed: boolean;
  violations: readonly Violation[];
  detail?: string
}> {
  if (definition.expectedViolations === undefined)
    return {
      passed: violations.length === 0,
      violations,
    };
  /**
   Distinct violated invariant names.
   */
  const found = [...new Set(violations.map(function invariantName(violation,) {
    return violation.invariant;
  },),),].toSorted();
  /**
   Required names, sorted for comparison.
   */
  const expected = definition.expectedViolations
    .toSorted();
  /**
   Whether the checker detected exactly the planted violations.
   */
  const passed = JSON.stringify(found,) === JSON.stringify(expected,);
  return passed
    ? {
      passed,
      violations: [],
      detail: `detected planted violations: ${found.join(', ',)}`,
    }
    : {
      passed,
      violations: [
        {
          invariant: 'positive-control',
          subject: 'checker',
          detail: `expected ${expected.join(', ',)}; found ${found.join(', ',)}`,
        },
        ...violations,
      ],
    };
}

/**
 Runs one scenario on one Git version.

 @param definition - catalog entry

 @param gitVersion - release of the installed Git

 @param seed - run seed

 @param workRoot - container work directory

 @param trace - commit-shape trace

 @returns scenario result

 @example
 ```ts
 await runScenario({ definition, gitVersion: '2.56.0', seed: 42, workRoot: '/work', trace });
 ```
 */
export async function runScenario({
  definition,
  gitVersion,
  seed,
  workRoot,
  trace,
}: Readonly<{
  definition: ScenarioDefinition;
  gitVersion: string;
  seed: number;
  workRoot: string;
  trace: CommitShapeTrace;
}>,): Promise<ScenarioResult> {
  /**
   Start time.
   */
  const startedAt = performance.now();
  /**
   Result fields shared by every outcome.
   */
  const base = {
    name: definition.name,
    group: definition.group,
    gitVersion,
  };
  if ((definition.minimumGit !== undefined) && (compareVersions({
    left: gitVersion,
    right: definition.minimumGit,
  },) < 0))
    return {
      ...base,
      status: 'skip',
      violations: [],
      durationMs: 0,
      detail: `requires Git ${definition.minimumGit}`,
      attempts: [],
    };
  /**
   Why this Git's replay plumbing does not fit the scenario, if it does not.
   */
  const replaySkip = replayPlumbingSkip({
    replayPlumbing: definition.replayPlumbing ?? 'required',
    gitVersion,
  },);
  if (replaySkip !== REPLAY_PLUMBING_FITS)
    return {
      ...base,
      status: 'skip',
      violations: [],
      durationMs: 0,
      detail: replaySkip,
      attempts: [],
    };
  /**
   Scenario-local seeded source, independent of filters.
   */
  const random = createSeededRandom(deriveSeed({
    seed,
    label: `${definition.name}@${gitVersion}`,
  },),);
  /**
   Scenario root.
   */
  const root = join(
    workRoot,
    gitVersion,
    definition.name,
  );
  await mkdir(
    root,
    { recursive: true, },
  );
  /**
   Ledger shared by workload and observation.
   */
  const ledger = createLedger();
  try {
    /**
     Repository options.
     */
    const options = definition.repository(random.fork('repository',),);
    /**
     The scenario's own commit service, stopped when the scenario ends.
     */
    await using service = await startScenarioService({ root, },);
    /**
     Fresh repositories.
     */
    const repository = await createScenarioRepository({
      root,
      options,
      service,
    },);
    /**
     Cancels the timeout once the workload settles.
     */
    const controller = new AbortController();
    /**
     Workload settlement captured as a value, so a late rejection after a timeout is never unhandled.
     */
    const workload = (async function settleWorkload(): Promise<Readonly<{ expectations: Expectations; }> | Readonly<{ error: unknown; }>> {
      try {
        return { expectations: await definition.run({
          repository,
          ledger,
          random: random.fork('workload',),
          trace,
          gitVersion,
        },), };
      }
      catch (error: unknown) {
        return { error, };
      }
    })();
    /**
     Timeout that resolves only when the workload overruns.
     */
    const timeout = (async function scenarioTimeout(): Promise<'timeout' | 'cancelled'> {
      try {
        await sleep(
          SCENARIO_TIMEOUT_MS,
          undefined,
          { signal: controller.signal, },
        );
        return 'timeout';
      }
      catch (error: unknown) {
        if (Error.isError(error,) && (error.name === 'AbortError'))
          return 'cancelled';
        throw error;
      }
    })();
    /**
     First settlement.
     */
    const first = await Promise.race([
      workload,
      timeout,
    ],);
    controller.abort();
    if ((typeof first) === 'string') {
      killActiveGroups();
      throw new Error(`scenario exceeded ${String(SCENARIO_TIMEOUT_MS,)} ms; killed its process groups`,);
    }
    if (((typeof first) === 'object') && ('error' in first))
      throw first.error;
    /**
     Scenario-specific expectations.
     */
    const { expectations, } = first;
    // A scenario that killed the service leaves it for the observation, which waits for the after-commit work.
    await service.ensureRunning();
    /**
     Repository and ledger facts.
     */
    const observation = await observeRun({
      repository,
      ledger,
      checks: {
        editorTrailer: options.editorTrailer === true,
        afterCommitRecords: options.afterCommitRecords === true,
        signature: options.signing,
      },
      expectations,
    },);
    /**
     Violated invariants.
     */
    const violations = checkInvariants(observation,);
    /**
     Pass decision, inverted for positive controls.
     */
    const verdict = judge({
      definition,
      violations,
    },);
    return {
      ...base,
      status: verdict.passed ? 'pass' : 'fail',
      violations: verdict.violations,
      ...(verdict.detail === undefined ? {} : { detail: verdict.detail, }),
      durationMs: Math.round(performance.now() - startedAt,),
      workload: workloadDigest(ledger.snapshot()
        .attempts,),
      attempts: observation.attempts
        .map(function summarize(attempt,) {
          /**
           Last line of standard error of an attempt that failed without being killed.
           */
          const failure = (attempt.exitCode === 0) || attempt.killed ? undefined : ledger.snapshot()
            .attempts
            .find(function sameLabel(recorded,) {
              return recorded.label === attempt.label;
            },)
            ?.outcome
            .stderr
            .trim()
            .split('\n',)
            .at(-1,);
          return {
            label: attempt.label,
            exitCode: attempt.exitCode,
            killed: attempt.killed,
            ...(failure === undefined
              ? {}
              : {
                note: failure.slice(
                  0,
                  NOTE_LENGTH,
                ),
              }),
          };
        },),
    };
  }
  catch (error: unknown) {
    killActiveGroups();
    return {
      ...base,
      status: 'error',
      violations: [],
      durationMs: Math.round(performance.now() - startedAt,),
      detail: caughtValueStack(error,),
      workload: workloadDigest(ledger.snapshot()
        .attempts,),
      attempts: ledger.snapshot()
        .attempts
        .map(function summarize(attempt,) {
          return {
            label: attempt.label,
            exitCode: attempt.outcome
              .exitCode,
            killed: attempt.killed,
          };
        },),
    };
  }
}

//endregion Runner
