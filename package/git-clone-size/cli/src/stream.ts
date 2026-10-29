import { mergeAsArrived, } from './async-queue.ts';
import {
  buildEstimates,
  buildSnapshot,
  computePending,
  type Signals,
} from './assemble.ts';
import { combineEstimates, } from './combine.ts';
import {
  METRIC_DEFAULT,
  METRIC_DEFAULT_BRANCH_ONLY,
  MS_PER_SECOND,
  SCOPE,
} from './constants.ts';
import { localExact, } from './local-exact.ts';
import {
  applySignal,
  churnSignal,
  commitSignal,
  deepenSignal,
  refsSignal,
  shallowSignal,
  storageSignal,
  tree0Signal,
} from './probe-tasks.ts';
import { cloneShallow, } from './probe-shallow.ts';
import type {
  LocalSource,
  RemoteSource,
  Source,
} from './source.ts';
import { makeTempDir, } from './temp-dir.ts';
import type { EstimateSnapshot, } from './types.ts';

/**
 Runtime options controlling the estimate, all with defaults supplied by the
 CLI layer (no required knob; streaming plus SIGINT remain the primary control).
 */
export type EstimateOptions = {
  readonly defaultBranchOnly: boolean;
  readonly maxProbeSeconds: number;
  readonly maxDeepenCommits: number;
  readonly maxPackBytes: number;
  readonly signal?: AbortSignal;
};

/**
 Selects the metric contract line for the active mode.
 
 @param defaultBranchOnly - whether both sides are restricted to the default branch
 
 @returns the metric contract string
 
 @example
 ```ts
 metricOf({ defaultBranchOnly: false }); // default-metric line
 ```
 */
function metricOf({ defaultBranchOnly, }: { readonly defaultBranchOnly: boolean; },): string {
  return defaultBranchOnly ? METRIC_DEFAULT_BRANCH_ONLY : METRIC_DEFAULT;
}

/**
 Builds one snapshot from the current signals: rebuild estimators, fuse, and
 assemble. `pending` is empty on the final snapshot.
 
 @param signals - current accumulated signals
 
 @param source - the source being estimated
 
 @param metric - metric contract line
 
 @param defaultBranchOnly - whether branch correction is skipped
 
 @param done - whether this is the final snapshot
 
 @returns one snapshot ready to serialize
 
 @example
 ```ts
 snapshotOf({ signals: {}, source, metric, defaultBranchOnly: false, done: false });
 ```
 */
function snapshotOf(
  {
    signals,
    source,
    metric,
    defaultBranchOnly,
    done,
  }: {
    readonly signals: Signals;
    readonly source: Source;
    readonly metric: string;
    readonly defaultBranchOnly: boolean;
    readonly done: boolean;
  },
): EstimateSnapshot {
  /**
   Fused belief over the current estimators.
   */
  const fused = combineEstimates({ estimates: buildEstimates({
    signals,
    defaultBranchOnly,
  },), },);
  return buildSnapshot({
    done,
    fused,
    metric,
    pending: done ? [] : computePending({
      signals,
      source,
    },),
    scope: SCOPE,
    ...signals.shallowBytes === undefined ? {} : { shallowBytes: signals.shallowBytes, },
  },);
}

/**
 Estimates a complete local repository exactly: a coarse prior snapshot first,
 then the exact pack-objects measurement as the final, very-high-confidence
 snapshot.
 
 @param source - local repository source
 
 @param options - runtime options
 
 @returns async generator of snapshots
 */
async function* estimateLocal(
  {
    source,
    options,
  }: {
    readonly source: LocalSource;
    readonly options: EstimateOptions
  },
): AsyncGenerator<EstimateSnapshot> {
  /**
   Metric contract line for this run.
   */
  const metric = metricOf({ defaultBranchOnly: options.defaultBranchOnly, },);
  /**
   Mutable container holding the immutable signal accumulator.
   */
  const state: { signals: Signals; } = { signals: {}, };
  yield snapshotOf({
    signals: state.signals,
    source,
    metric,
    defaultBranchOnly: options.defaultBranchOnly,
    done: false,
  },);

  /**
   Exact (or size-pack proxy) local measurement. Byte fields are omitted when
   unmeasurable, so a degenerate repo degrades to the prior instead of
   recording a fabricated zero or crashing the stream.
   */
  const result = await localExact({
    path: source.path,
    maxPackBytes: options.maxPackBytes,
  },);
  state.signals = {
    ...state.signals,
    ...result.fullBytes === undefined ? {} : {
      local: {
        basis: result.basis,
        confidence: result.confidence,
        fullBytes: result.fullBytes,
      },
    },
    ...result.shallowBytes === undefined ? {} : { shallowBytes: result.shallowBytes, },
  };
  yield snapshotOf({
    signals: state.signals,
    source,
    metric,
    defaultBranchOnly: options.defaultBranchOnly,
    done: true,
  },);
}

/**
 Combines a probe-budget signal with an optional caller cancellation signal.
 
 @param budgetSignal - Owned timeout signal for probe budget.
 
 @param callerSignal - Optional caller cancellation capability.
 
 @returns Budget signal alone or combined dependent signal.
 
 @mutates budgetSignal through DOM commit 5796f716 AbortSignal.any dependent-signal relations.
 
 @mutates callerSignal through DOM commit 5796f716 AbortSignal.any dependent-signal relations.
 
 @example
 ```ts
 combineProbeSignals({ budgetSignal: AbortSignal.timeout(1), callerSignal: undefined });
 ```
 */
function combineProbeSignals({
  budgetSignal,
  callerSignal,
}: {
  readonly budgetSignal: AbortSignal;
  readonly callerSignal?: AbortSignal;
},): AbortSignal {
  return callerSignal === undefined
    ? budgetSignal
    : AbortSignal.any([
      budgetSignal,
      callerSignal,
    ],);
}

/**
 Estimates a remote repository: launches every cheap probe concurrently, folds
 each signal in as it lands (fastest first), and yields a refined snapshot per
 arrival, ending with the tightest fused snapshot. Temp clones live in
 disposable directories removed when the generator finishes or is closed.
 
 @param source - remote repository source
 
 @param options - runtime options
 
 @returns async generator of snapshots
 */
async function* estimateRemote(
  {
    source,
    options,
  }: {
    readonly source: RemoteSource;
    readonly options: EstimateOptions
  },
): AsyncGenerator<EstimateSnapshot> {
  /**
   Metric contract line for this run.
   */
  const metric = metricOf({ defaultBranchOnly: options.defaultBranchOnly, },);
  /**
   Mutable container holding the immutable signal accumulator.
   */
  const state: { signals: Signals; } = { signals: {}, };
  yield snapshotOf({
    signals: state.signals,
    source,
    metric,
    defaultBranchOnly: options.defaultBranchOnly,
    done: false,
  },);

  /**
   Disposable temp dir for the shallow + deepen clone.
   */
  await using shallowDir = await makeTempDir({ prefix: 'gcs-shallow-', },);
  /**
   Disposable temp dir for the tree:0 commit-count clone.
   */
  await using tree0Dir = await makeTempDir({ prefix: 'gcs-tree0-', },);
  /**
   Disposable temp dir for the blobless churn clone.
   */
  await using blobDir = await makeTempDir({ prefix: 'gcs-blob-', },);

  /**
   Wall-clock budget signal; aborting kills in-flight clones.
   */
  const budgetSignal = AbortSignal.timeout(options.maxProbeSeconds * MS_PER_SECOND,);
  /**
   Effective abort signal: the budget alone, or merged with the caller's
   signal (SIGINT) so either source can abort the probes.
   */
  const signal = combineProbeSignals({
    budgetSignal,
    ...options.signal === undefined
      ? {}
      : { callerSignal: options.signal, },
  },);
  /**
   Single shallow clone, shared by the shallow and deepen tasks.
   */
  const shallowPromise = cloneShallow({
    url: source.url,
    dest: shallowDir.path,
    signal,
  },);

  /**
   All probe tasks, launched concurrently and merged fastest-first.
   */
  const tasks = [
    shallowSignal({ shallowPromise, },),
    deepenSignal({
      shallowPromise,
      maxDeepenCommits: options.maxDeepenCommits,
      signal,
    },),
    refsSignal({ url: source.url, },),
    storageSignal({ source, },),
    commitSignal({ source, },),
    tree0Signal({
      url: source.url,
      dest: tree0Dir.path,
      signal,
    },),
    churnSignal({
      url: source.url,
      dest: blobDir.path,
      signal,
    },),
  ];

  for await (const arrived of mergeAsArrived({ tasks, },)) {
    state.signals = applySignal({
      signals: state.signals,
      signal: arrived,
    },);
    yield snapshotOf({
      signals: state.signals,
      source,
      metric,
      defaultBranchOnly: options.defaultBranchOnly,
      done: false,
    },);
  }
  yield snapshotOf({
    signals: state.signals,
    source,
    metric,
    defaultBranchOnly: options.defaultBranchOnly,
    done: true,
  },);
}

/**
 Streams progressive clone-size estimates for a source. A local complete repo
 is measured exactly; a remote (or incomplete local repo via its origin) is
 estimated from cheap probes fused into a tightening range. Yields a snapshot
 the instant the first signal lands and again on every refinement, never
 blocking on the slowest probe and never refusing.
 
 @param source - resolved local or remote source
 
 @param options - runtime options
 
 @returns async generator of progressive snapshots
 
 @example
 ```ts
 for await (const snapshot of estimate({ source, options })) {
   process.stdout.write(`${JSON.stringify(snapshot)}\n`);
 }
 ```
 */
export async function* estimate(
  {
    source,
    options,
  }: {
    readonly source: Source;
    readonly options: EstimateOptions
  },
): AsyncGenerator<EstimateSnapshot> {
  if (source.kind === 'local') {
    yield* estimateLocal({
      source,
      options,
    },);
    return;
  }
  yield* estimateRemote({
    source,
    options,
  },);
}
