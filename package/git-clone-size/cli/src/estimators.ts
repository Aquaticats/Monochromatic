import {
  PRIOR_ABSENT_TIP_BYTES,
  PRIOR_MULTIPLIER,
} from './constants.ts';
import type {
  Confidence,
  Estimate,
} from './types.ts';

//region Weights (precision proxies; higher = more trusted/tighter)

/**
 Weight for an exact local pack-objects measurement; dominates the fusion.
 */
const LOCAL_EXACT_WEIGHT = 1_000;

/**
 Weight for the local size-pack proxy (huge-repo fallback).
 */
const LOCAL_SIZEPACK_WEIGHT = 120;

/**
 Weight for the host storage proxy before calibration.
 */
const HOST_PROXY_WEIGHT = 10;

/**
 Weight for the deepen extrapolation when the commit count is known.
 */
const DEEPEN_WEIGHT = 24;

/**
 Weight for the deepen extrapolation when the commit count is a lower bound.
 */
const DEEPEN_UNCERTAIN_WEIGHT = 7;

/**
 Weight for the blobless churn estimator (approximate; downweighted).
 */
const CHURN_WEIGHT = 3;

/**
 Weight for prior-only estimators (last resort).
 */
const PRIOR_WEIGHT = 1;

//endregion Weights

//region Band and factor constants

/**
 Half-width fraction for the exact local band (packer cross-version variance).
 */
const LOCAL_EXACT_BAND = 0.005;

/**
 Half-width fraction for the size-pack proxy band.
 */
const LOCAL_SIZEPACK_BAND = 0.1;

/**
 Central factor mapping host server storage to a packed clone size.
 */
const HOST_PROXY_POINT_FACTOR = 0.85;

/**
 Lower factor for the host storage proxy band.
 */
const HOST_PROXY_LO_FACTOR = 0.4;

/**
 Upper factor for the host storage proxy band.
 */
const HOST_PROXY_HI_FACTOR = 1.2;

/**
 Central delta-compression discount applied to the churn factor.
 */
const CHURN_POINT_FACTOR = 0.5;

/**
 Lower delta-compression discount for the churn band.
 */
const CHURN_LO_FACTOR = 0.2;

/**
 Upper delta-compression discount for the churn band.
 */
const CHURN_HI_FACTOR = 1;

/**
 Per-side-branch fraction added to the full estimate by branch correction.
 */
const BRANCH_PER_BRANCH = 0.02;

/**
 Cap on the cumulative side-branch fraction.
 */
const BRANCH_MAX_FRAC = 0.5;

/**
 Share of the branch fraction applied to the point (vs the high end).
 */
const BRANCH_POINT_SHARE = 0.5;

/**
 Multiplier on the commit count for the high end when the count is a lower
 bound, widening the deepen interval upward.
 */
const UNCERTAIN_N_FACTOR = 2;

//endregion Band and factor constants

/**
 Builds the estimator for an exact (or size-pack proxy) local measurement.
 
 @param fullBytes - measured full-clone object-store bytes
 
 @param confidence - `very high` for exact pack-objects, `high` for size-pack
 
 @param basis - basis label propagated into the snapshot
 
 @returns tiny-band estimate that dominates the fusion
 
 @example
 ```ts
 localExactEstimate({ fullBytes: 1_000_000, confidence: 'very high', basis: 'local pack-objects (exact)' });
 ```
 */
export function localExactEstimate(
  {
    fullBytes,
    confidence,
    basis,
  }: {
    readonly fullBytes: number;
    readonly confidence: Confidence;
    readonly basis: string
  },
): Estimate {
  /**
   Half-width fraction selected by confidence tier.
   */
  const band = confidence === 'very high' ? LOCAL_EXACT_BAND : LOCAL_SIZEPACK_BAND;
  /**
   Weight selected by confidence tier.
   */
  const weight = confidence === 'very high' ? LOCAL_EXACT_WEIGHT : LOCAL_SIZEPACK_WEIGHT;
  return {
    confidence,
    hi: fullBytes * (1 + band),
    lo: fullBytes * (1 - band),
    name: basis,
    point: fullBytes,
    weight,
  };
}

/**
 Builds the estimator for the host-reported storage proxy, mapped to a clone
 size via an uncalibrated bias band (medium confidence until calibrated).
 
 @param storageBytes - host-reported repository storage in bytes
 
 @returns medium-confidence proxy estimate
 
 @example
 ```ts
 hostProxyEstimate({ storageBytes: 5_000_000 });
 ```
 */
export function hostProxyEstimate({ storageBytes, }: { readonly storageBytes: number; },): Estimate {
  return {
    confidence: 'medium',
    hi: storageBytes * HOST_PROXY_HI_FACTOR,
    lo: storageBytes * HOST_PROXY_LO_FACTOR,
    name: 'host storage proxy',
    point: storageBytes * HOST_PROXY_POINT_FACTOR,
    weight: HOST_PROXY_WEIGHT,
  };
}

/**
 Builds the deepen-extrapolation estimator: `full ~= C1 + m * (N - 1)`, with
 the marginal already repack-corrected. A lower-bound commit count widens the
 high end and drops confidence; the branch-coverage correction adds the
 side-branch contribution unless restricted to the default branch.
 
 @param c1Bytes - shallow tip compressed bytes
 
 @param marginal - repack-corrected marginal bytes per commit (lo/point/hi)
 
 @param commitCount - default-branch commit count (or a lower bound)
 
 @param commitUncertain - whether `commitCount` is only a lower bound
 
 @param branches - total branch heads from ls-remote, for branch correction
 
 @param defaultBranchOnly - skip branch correction when true
 
 @returns medium/low-confidence extrapolation estimate
 
 @example
 ```ts
 deepenEstimate({ c1Bytes: 4_000_000, marginal: { lo: 800, point: 1_000, hi: 1_400 }, commitCount: 500, commitUncertain: false, branches: 3, defaultBranchOnly: false });
 ```
 */
export function deepenEstimate(
  {
    c1Bytes,
    marginal,
    commitCount,
    commitUncertain,
    branches,
    defaultBranchOnly,
  }: {
    readonly c1Bytes: number;
    readonly marginal: {
      readonly lo: number;
      readonly point: number;
      readonly hi: number
    };
    readonly commitCount: number;
    readonly commitUncertain: boolean;
    readonly branches: number;
    readonly defaultBranchOnly: boolean;
  },
): Estimate {
  /**
   Effective commit count, floored at 1.
   */
  const n = Math.max(
    1,
    commitCount
  );
  /**
   Commits beyond the tip on the default branch.
   */
  const extras = n - 1;
  /**
   Upper-end commit count: doubled when the count is only a lower bound.
   */
  const extrasHi = commitUncertain ? ((n * UNCERTAIN_N_FACTOR) - 1) : extras;
  /**
   Side branches beyond the default branch (0 under default-branch-only).
   */
  const sideBranches = defaultBranchOnly ? 0 : Math.max(
    0,
    branches - 1
  );
  /**
   Cumulative side-branch fraction, capped.
   */
  const frac = Math.min(
    BRANCH_MAX_FRAC,
    sideBranches * BRANCH_PER_BRANCH
  );
  return {
    confidence: commitUncertain ? 'low' : 'medium',
    hi: (c1Bytes + (marginal.hi
      * extrasHi)) * (1 + frac),
    lo: c1Bytes + (marginal.lo
      * extras),
    name: defaultBranchOnly
      ? 'deepen-extrapolation(repack-corrected)'
      : 'deepen-extrapolation(repack-corrected)+branch correction',
    point: (c1Bytes + (marginal.point
      * extras)) * (1 + (frac
        * BRANCH_POINT_SHARE)),
    weight: commitUncertain ? DEEPEN_UNCERTAIN_WEIGHT : DEEPEN_WEIGHT,
  };
}

/**
 Builds the churn estimator from a blobless clone: `full ~= C1 * (distinct
 path objects / tip files)` discounted for delta compression. Approximate
 (the listing counts trees alongside blobs), so it carries a low weight and a
 wide band.
 
 @param c1Bytes - shallow tip compressed bytes
 
 @param distinctPathObjects - count of historical path-bearing objects
 
 @param tipFiles - file count at the tip (the churn denominator)
 
 @returns low-confidence churn estimate
 
 @example
 ```ts
 churnEstimate({ c1Bytes: 4_000_000, distinctPathObjects: 9_000, tipFiles: 1_500 });
 ```
 */
export function churnEstimate(
  {
    c1Bytes,
    distinctPathObjects,
    tipFiles,
  }: {
    readonly c1Bytes: number;
    readonly distinctPathObjects: number;
    readonly tipFiles: number
  },
): Estimate {
  /**
   Historical-to-tip object ratio, floored denominator to avoid divide-by-zero.
   */
  const churnFactor = distinctPathObjects / Math.max(
    1,
    tipFiles
  );
  return {
    confidence: 'low',
    hi: c1Bytes * churnFactor
      * CHURN_HI_FACTOR,
    lo: c1Bytes * churnFactor
      * CHURN_LO_FACTOR,
    name: 'blobless churn',
    point: c1Bytes * churnFactor
      * CHURN_POINT_FACTOR,
    weight: CHURN_WEIGHT,
  };
}

/**
 Builds the snapshot-multiplier prior from the shallow tip size, the lowest-
 confidence last resort when richer signals are unavailable.
 
 @param c1Bytes - shallow tip compressed bytes
 
 @returns low-confidence prior estimate
 
 @example
 ```ts
 priorEstimate({ c1Bytes: 4_000_000 });
 ```
 */
export function priorEstimate({ c1Bytes, }: { readonly c1Bytes: number; },): Estimate {
  return {
    confidence: 'low',
    hi: c1Bytes * PRIOR_MULTIPLIER.hi,
    lo: c1Bytes * PRIOR_MULTIPLIER.lo,
    name: 'snapshot-multiplier prior',
    point: c1Bytes * PRIOR_MULTIPLIER.point,
    weight: PRIOR_WEIGHT,
  };
}

/**
 Builds a prior estimate when even the tip size is unknown, so the very first
 snapshot can emit a wide range rather than nothing.
 
 @returns lowest-confidence absolute-byte prior estimate
 
 @example
 ```ts
 priorAbsentEstimate();
 ```
 */
export function priorAbsentEstimate(): Estimate {
  return {
    confidence: 'low',
    hi: PRIOR_ABSENT_TIP_BYTES.hi,
    lo: PRIOR_ABSENT_TIP_BYTES.lo,
    name: 'prior (no signals yet)',
    point: PRIOR_ABSENT_TIP_BYTES.point,
    weight: PRIOR_WEIGHT,
  };
}
