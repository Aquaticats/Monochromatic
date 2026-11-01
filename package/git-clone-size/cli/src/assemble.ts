import {
  churnEstimate,
  deepenEstimate,
  hostProxyEstimate,
  localExactEstimate,
  priorAbsentEstimate,
  priorEstimate,
} from './estimators.ts';
import {
  computeRatio,
  computeSavings,
  toSize,
} from './format.ts';
import type {
  HostCommitCountResult,
  LsRemoteResult,
} from './host-api.ts';
import type { DeepenResult, } from './probe-deepen.ts';
import type {
  ChurnResult,
  CommitCountResult,
} from './probe-partial.ts';
import type { Source, } from './source.ts';
import type {
  Confidence,
  Estimate,
  EstimateSnapshot,
  FusionState,
} from './types.ts';

/**
 Immutable accumulator of every probe/metadata signal seen so far. Fields are
 optional and filled as signals land; the estimate is rebuilt from this on
 each refinement, and a landed signal produces a new value rather than mutating
 in place.
 */
export type Signals = {
  readonly shallowBytes?: number;
  readonly storageBytes?: number;
  readonly refs?: LsRemoteResult;
  readonly commit?: HostCommitCountResult;
  readonly tree0?: CommitCountResult;
  readonly churn?: ChurnResult;
  readonly deepen?: DeepenResult;
  readonly local?: {
    readonly fullBytes: number;
    readonly confidence: Confidence;
    readonly basis: string;
  };
};

/**
 Builds the deepen estimate from the current signals, resolving the commit
 count from the best available source and flagging uncertainty.
 
 @param signals - current accumulated signals
 
 @param c1Bytes - shallow tip bytes
 
 @param defaultBranchOnly - whether to skip branch correction
 
 @returns the deepen estimate
 */
function deepenFromSignals(
  {
    signals,
    c1Bytes,
    defaultBranchOnly,
    deepen,
  }: {
    readonly signals: Signals;
    readonly c1Bytes: number;
    readonly defaultBranchOnly: boolean;
    readonly deepen: DeepenResult;
  },
): Estimate {
  /**
   Commit count: host API, else tree:0 count, else the deepen walk count.
   */
  const commitCount = signals.commit
    ?.count
    ?? signals.tree0
      ?.count
    ?? deepen.observedCommits;
  /**
   Whether the count is only a lower bound (host lower bound or capped walk
   with no exact source).
   */
  const commitUncertain = (signals.commit
    ?.lowerBound
    ?? false)
    || ((signals.commit === undefined) && (signals.tree0 === undefined)
      && deepen.hitCap);
  return deepenEstimate({
    branches: signals.refs
      ?.branches
      ?? 1,
    c1Bytes,
    commitCount,
    commitUncertain,
    defaultBranchOnly,
    marginal: {
      hi: deepen.marginalHi,
      lo: deepen.marginalLo,
      point: deepen.marginalPoint,
    },
  },);
}

/**
 Rebuilds the estimator set from the current signals. A local exact value
 ({@link localExactEstimate}) or host proxy ({@link hostProxyEstimate})
 contributes directly; with a shallow tip known, the deepen
 ({@link deepenEstimate}), churn ({@link churnEstimate}), and prior
 ({@link priorEstimate}) estimators are added. Always returns at least one
 estimator so the fusion never sees an empty set, falling back to
 {@link priorAbsentEstimate} when no signal has landed.
 
 @param signals - current accumulated signals
 
 @param defaultBranchOnly - whether to skip branch correction
 
 @returns estimators reflecting all signals seen so far
 
 @example
 ```ts
 const estimates = buildEstimates({ signals: { shallowBytes: 4_000_000 }, defaultBranchOnly: false });
 ```
 */
export function buildEstimates(
  {
    signals,
    defaultBranchOnly,
  }: {
    readonly signals: Signals;
    readonly defaultBranchOnly: boolean
  },
): readonly Estimate[] {
  /**
   Estimators accumulated from the present signals.
   */
  const estimates: Estimate[] = [];
  if (signals.local !== undefined)
    estimates.push(localExactEstimate(signals.local,),);
  if (signals.storageBytes !== undefined)
    estimates.push(hostProxyEstimate({ storageBytes: signals.storageBytes, },),);
  /**
   Shallow tip bytes, the basis for the extrapolation and prior estimators.
   */
  const c1Bytes = signals.shallowBytes;
  if ((c1Bytes !== undefined) && (c1Bytes > 0)) {
    if (signals.deepen !== undefined)
      estimates.push(deepenFromSignals({
        signals,
        c1Bytes,
        defaultBranchOnly,
        deepen: signals.deepen,
      },),);
    if (signals.churn !== undefined)
      estimates.push(churnEstimate({
        c1Bytes,
        distinctPathObjects: signals.churn
          .distinctPathObjects,
        tipFiles: signals.churn
          .tipFiles,
      },),);
    estimates.push(priorEstimate({ c1Bytes, },),);
  }
  if (estimates.length === 0)
    estimates.push(priorAbsentEstimate(),);
  return estimates;
}

/**
 Computes which signals are still in flight, for the snapshot's `pending` list.
 
 @param signals - current accumulated signals
 
 @param source - the source being estimated
 
 @returns names of signals not yet landed
 
 @example
 ```ts
 computePending({ signals: {}, source: { kind: 'local', path: '/repo' } }); // ['local-exact']
 ```
 */
export function computePending(
  {
    signals,
    source,
  }: {
    readonly signals: Signals;
    readonly source: Source
  },
): readonly string[] {
  if (source.kind === 'local')
    return signals.local === undefined ? ['local-exact',] : [];
  /**
   Each expected remote signal paired with whether it has landed.
   */
  const labels: readonly (readonly [
    string,
    boolean
  ])[] = [
    [
      'shallow',
      signals.shallowBytes !== undefined,
    ],
    [
      'deepen',
      signals.deepen !== undefined,
    ],
    [
      'commit-count',
      (signals.commit !== undefined) || (signals.tree0 !== undefined),
    ],
    [
      'refs',
      signals.refs !== undefined,
    ],
    [
      'churn',
      signals.churn !== undefined,
    ],
    [
      'host-proxy',
      (source.host === 'unknown') || (signals.storageBytes !== undefined),
    ],
  ];
  return labels
    .filter(function notPresent([, present,],) {
      return !present;
    },)
    .map(function nameOf([name,],) {
      return name;
    },);
}

/**
 Assembles one JSONL snapshot from the fused belief and the shallow size.
 `ratio` and `savings` appear only once a shallow measurement exists.
 
 @param fused - combined full-size belief
 
 @param shallowBytes - measured shallow bytes, when available
 
 @param metric - metric contract line
 
 @param scope - scope line
 
 @param pending - signals still in flight
 
 @param done - whether this is the final snapshot
 
 @returns one snapshot object ready to serialize
 
 @example
 ```ts
 buildSnapshot({ fused, metric, scope, pending: [], done: true });
 ```
 */
export function buildSnapshot(
  {
    fused,
    shallowBytes,
    metric,
    scope,
    pending,
    done,
  }: {
    readonly fused: FusionState;
    readonly shallowBytes?: number;
    readonly metric: string;
    readonly scope: string;
    readonly pending: readonly string[];
    readonly done: boolean;
  },
): EstimateSnapshot {
  /**
   Full-size range with confidence, always present.
   */
  const full = {
    confidence: fused.confidence,
    hi: toSize(fused.hi,),
    lo: toSize(fused.lo,),
    point: toSize(fused.point,),
  };
  if (shallowBytes === undefined)
    return {
      basis: fused.basis,
      done,
      full,
      metric,
      pending,
      scope,
    };
  /**
   Shallow/full ratio derived from the exact shallow bytes.
   */
  const ratio = computeRatio({
    full: {
      hi: full.hi,
      lo: full.lo,
      point: full.point,
    },
    shallowBytes,
  },);
  return {
    basis: fused.basis,
    done,
    full,
    metric,
    pending,
    ratio,
    savings: computeSavings({ ratio, },),
    scope,
    shallow: toSize(shallowBytes,),
  };
}
