/**
 Shared types for the clone-size estimator: the public JSONL snapshot shape
 and the internal estimator/distribution shapes the fusion operates on.
 
 @module
 */

/**
 A size carrying both the raw byte count (machine) and a human string
 (skimmable), so a single JSONL line serves parsers and people at once.
 
 @example
 ```ts
 const s: Size = { bytes: 422, human: '422 B' };
 ```
 */
export type Size = {
  readonly bytes: number;
  readonly human: string;
};

/**
 A fused credible interval over a size: a central point plus low and high
 ends. Every estimator contributes one of these (a point treated as a spread).
 */
export type EstimateRange = {
  readonly point: Size;
  readonly lo: Size;
  readonly hi: Size;
};

/**
 Confidence ladder, ordered from most to least trustworthy. `very high` is a
 local exact pack-objects measurement; `low` is a prior-only or budget-aborted
 estimate.
 */
export type Confidence = 'very high' | 'high' | 'medium' | 'low';

/**
 A unitless ratio range (shallow / full) with low and high ends.
 */
export type RatioRange = {
  readonly point: number;
  readonly lo: number;
  readonly hi: number;
};

/**
 A plain numeric band (lo/point/hi), used for prior multiplier and byte
 intervals that are not yet wrapped as {@link Size} values.
 */
export type Band = {
  readonly lo: number;
  readonly point: number;
  readonly hi: number;
};

/**
 One progressive snapshot, emitted as a single JSONL line. Fields are optional
 because they land at different times; `pending` names what is still in flight
 and `done` marks the final, tightest fusion.
 */
export type EstimateSnapshot = {
  readonly shallow?: Size;
  readonly full: EstimateRange & { readonly confidence: Confidence; };
  readonly ratio?: RatioRange;
  readonly savings?: RatioRange;
  readonly metric: string;
  readonly scope: string;
  readonly basis: readonly string[];
  readonly pending: readonly string[];
  readonly done: boolean;
};

/**
 One estimator's contribution to the fusion: a central byte estimate plus a
 lo..hi byte interval, a precision weight (higher = tighter/more trusted), the
 confidence the estimator alone would justify, and a basis label for `basis`.
 */
export type Estimate = {
  readonly point: number;
  readonly lo: number;
  readonly hi: number;
  readonly weight: number;
  readonly confidence: Confidence;
  readonly name: string;
};

/**
 Running fusion state: the combined full-size belief plus the basis labels and
 confidence accumulated from every signal folded so far.
 */
export type FusionState = {
  readonly point: number;
  readonly lo: number;
  readonly hi: number;
  readonly confidence: Confidence;
  readonly basis: readonly string[];
};
