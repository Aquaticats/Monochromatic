/**
 Concurrent agents over disjoint paths and over replayed trace shapes.

 @module
 */

import {
  disjointPlans,
  repositoryOptions,
  runDisjointWorkers,
  seedTexts,
  workerPaths,
} from './scenario-helper-fixture.ts';
import {
  allSucceeded,
  type ScenarioDefinition,
} from './scenario-model-fixture.ts';
import {
  seedTracePaths,
  TRACE_MAX_CHANGES,
} from './scenario-setup-fixture.ts';
import { runTraceOperations, } from './trace-execution-fixture.ts';
import {
  planTraceOperations,
  selectTraceWindow,
} from './trace-replay-fixture.ts';

//region Constants

/**
 Largest worker start offset.
 */
export const MAX_JITTER_MS = 40;

/**
 Worker count range for the disjoint-path scenario.
 */
const WORKER_RANGE = {
  min: 4,
  max: 8,
} as const;

/**
 Trace window length for concurrent replay.
 */
const CONCURRENT_TRACE_COMMITS = 16;

/**
 In-flight bound for concurrent trace replay.
 */
const TRACE_CONCURRENCY = 4;

//endregion Constants

//region Scenarios

/**
 Concurrent explicit-path commits of disjoint files.
 */
const disjointPaths: ScenarioDefinition = {
  name: 'concurrent-disjoint-paths',
  group: 'concurrency',
  summary: '4 to 8 agents commit disjoint explicit paths at the same time',
  repository(random,) {
    return repositoryOptions({ seedFiles: seedTexts({
      random,
      paths: workerPaths({
        prefix: 'w',
        count: WORKER_RANGE.max,
      },),
    },), },);
  },
  async run(context,) {
    /**
     Finished attempts.
     */
    const attempts = await runDisjointWorkers({
      context,
      plans: disjointPlans(context.random
        .integer(WORKER_RANGE,),),
      maxJitterMs: MAX_JITTER_MS,
    },);
    return [allSucceeded({
      name: 'all-commits-succeed',
      attempts,
    },),];
  },
};

/**
 Concurrent trace replay.
 */
const traceConcurrent: ScenarioDefinition = {
  name: 'concurrent-trace-replay',
  group: 'concurrency',
  summary: `replay ${String(CONCURRENT_TRACE_COMMITS,)} trace commits with ${String(TRACE_CONCURRENCY,)} in flight`,
  // Config hooks need Git 2.54.0.
  minimumGit: '2.54.0',
  repository(random,) {
    return repositoryOptions({
      seedFiles: seedTexts({
        random,
        paths: ['README.txt',],
      },),
      hooks: 'config',
      hookEvents: ['pre-commit',],
    },);
  },
  async run(context,) {
    /**
     Window operations and the seed files they need.
     */
    const {
      operations,
      seeds,
    } = planTraceOperations(selectTraceWindow({
      trace: context.trace,
      random: context.random
        .fork('window',),
      length: CONCURRENT_TRACE_COMMITS,
      maxChanges: TRACE_MAX_CHANGES,
    },),);
    await seedTracePaths({
      context,
      seeds,
    },);
    /**
     Finished attempts.
     */
    const attempts = await runTraceOperations({
      context,
      operations,
      concurrency: TRACE_CONCURRENCY,
    },);
    return [allSucceeded({
      name: 'all-commits-succeed',
      attempts,
    },),];
  },
};

/**
 Disjoint-path and trace scenarios in report order.
 */
export const CONCURRENT_PATH_SCENARIOS: readonly ScenarioDefinition[] = [
  disjointPaths,
  traceConcurrent,
];

//endregion Scenarios
