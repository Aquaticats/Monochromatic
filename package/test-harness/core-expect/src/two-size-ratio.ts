/**
 Two-size timing measurement and the linear-growth assertion built on it,
 so a test can separate linear from quadratic growth on an exposing shape
 instead of trusting a complexity comment.

 @module
 */

//region Measurement

/**
 Timing of one input size,
 best and worst of the measured rounds.
 */
export type SizeTiming = {
  /**
   Input size the generator produced.
   */
  readonly size: number;
  /**
   Fastest measured round,
   the figure the ratio reads.
   */
  readonly bestMs: number;
  /**
   Slowest measured round,
   which shows the run-to-run spread the ratio must clear.
   */
  readonly worstMs: number;
};

/**
 Outcome of timing one function at a base size and at its double.
 */
export type TwoSizeMeasurement = {
  readonly base: SizeTiming;
  readonly doubled: SizeTiming;
  /**
   Doubled best time over base best time.
   Linear work lands near 2,
   quadratic work near 4.
   */
  readonly ratio: number;
};

/**
 Smallest denominator the ratio accepts,
 so a base timing at the clock floor cannot fabricate a huge ratio.
 */
const MIN_BASE_MS = 0.001;

/**
 Rounds discarded before measurement,
 so engine warmup costs cannot masquerade as superlinear growth.
 */
const WARMUP_ROUNDS = 2;

/**
 Decimal places the printed ratio keeps.
 */
const RATIO_DECIMALS = 2;

/**
 Decimal places the printed millisecond figures keep.
 */
const MS_DECIMALS = 3;

/**
 Smallest of a list of timings,
 folded one element at a time
 so no unbounded argument list reaches a call.

 @param values - Timings to fold; at least one

 @returns Smallest value,
 or the positive infinity placeholder when the list is empty
 */
function smallest(values: readonly number[],): number {
  /**
   Smallest value seen so far.
   */
  let best = Number.POSITIVE_INFINITY;
  for (const value of values) {
    if (value < best) best = value;
  }
  return best;
}

/**
 Largest of a list of timings,
 folded one element at a time
 so no unbounded argument list reaches a call.

 @param values - Timings to fold; at least one

 @returns Largest value,
 or the negative infinity placeholder when the list is empty
 */
function largest(values: readonly number[],): number {
  /**
   Largest value seen so far.
   */
  let worst = Number.NEGATIVE_INFINITY;
  for (const value of values) {
    if (value > worst) worst = value;
  }
  return worst;
}

/**
 Times one call of the work under test.

 @param run - Work under test whose single call is timed

 @param input - Input that call consumes

 @returns Elapsed milliseconds of that one call
 */
function timeOne<Input>({
  run,
  input,
}: {
  run: (input: Input,) => unknown;
  input: Input;
},): number {
  /**
   Timestamp before the call.
   */
  const started = performance.now();
  run(input,);
  /**
   Timestamp after the call.
   */
  const finished = performance.now();
  return finished - started;
}

/**
 Times `run` on generated inputs at `size` and at its double.

 Two finite sizes estimate growth,
 they do not prove O(n) for unbounded input:
 a ratio near 2 also fits n log n and n^1.2,
 so this separates the linear and quadratic shapes on the caller's input
 and nothing stronger.
 Keep `repeats` independent of `size`,
 since measurement cost is proportional to both.
 Rounds alternate between the two sizes
 so host load drift hits both arms alike,
 each size keeps its best and worst round,
 and two warmup rounds at the doubled size run first and are discarded
 (the cold series in `doc/handover/superlinear-cost-dismissal-gate.md`
 is what that buys).

 @param grow - Produces the input at a given size, and owns the exposing shape

 @param run - Work under test; its return value is ignored

 @param size - Base input size; pick one whose base timing clears the clock floor

 @param repeats - Measured rounds per size; best of these feeds the ratio, and it must stay independent of size

 @returns Timing of both sizes and their ratio

 @example
 ```ts
 const measurement = measureTwoSizeRatio({
   grow: (n,) => 'a'.repeat(n,),
   run: (text,) => { for (const char of text) void char; },
   size: 50_000,
 },);
 ```
 */
export function measureTwoSizeRatio<Input>({
  grow,
  run,
  size = 2_000,
  repeats = 5,
}: {
  grow: (size: number,) => Input;
  run: (input: Input,) => unknown;
  size?: number;
  repeats?: number;
},): TwoSizeMeasurement {
  for (let round = 0; round < WARMUP_ROUNDS; round += 1) {
    /**
     Input of one discarded warmup round.
     */
    const warmupInput = grow(size * 2,);
    timeOne({
      run,
      input: warmupInput,
    },);
  }
  /**
   Measured rounds at the base size.
   */
  const baseRounds: number[] = [];
  /**
   Measured rounds at the doubled size.
   */
  const doubledRounds: number[] = [];
  for (let round = 0; round < repeats; round += 1) {
    /**
     Input of one base-size round.
     */
    const baseInput = grow(size,);
    /**
     Elapsed time of one base-size round.
     */
    const baseMs = timeOne({
      run,
      input: baseInput,
    },);
    baseRounds.push(baseMs,);
    /**
     Input of one doubled-size round.
     */
    const doubledInput = grow(size * 2,);
    /**
     Elapsed time of one doubled-size round.
     */
    const doubledMs = timeOne({
      run,
      input: doubledInput,
    },);
    doubledRounds.push(doubledMs,);
  }
  /**
   Best and worst of the base-size rounds.
   */
  const base: SizeTiming = {
    size,
    bestMs: smallest(baseRounds,),
    worstMs: largest(baseRounds,),
  };
  /**
   Best and worst of the doubled-size rounds.
   */
  const doubled: SizeTiming = {
    size: size * 2,
    bestMs: smallest(doubledRounds,),
    worstMs: largest(doubledRounds,),
  };
  /**
   Denominator of the ratio,
   floored so a clock-floor base timing cannot fabricate growth.
   */
  const denominator = Math.max(
    base.bestMs,
    MIN_BASE_MS,
  );
  /**
   Doubled best time,
   the numerator of the ratio.
   */
  const doubledBestMs = doubled.bestMs;
  return {
    base,
    doubled,
    ratio: doubledBestMs / denominator,
  };
}

//endregion Measurement

//region Assertion

/**
 Thrown when measured growth outruns the allowed doubling ratio.
 */
export class TwoSizeRatioError extends Error {
  /**
   Measurement that failed the bound,
   kept so a caller can report the numbers themselves.
   */
  readonly measurement: TwoSizeMeasurement;

  /**
   Allowed doubling ratio the measurement exceeded.
   */
  readonly maxRatio: number;

  /**
   Builds the failure message from the measurement it carries.

   @param measurement - Measurement that failed the bound

   @param maxRatio - Allowed doubling ratio the measurement exceeded
   */
  constructor({
    measurement,
    maxRatio,
  }: {
    measurement: TwoSizeMeasurement;
    maxRatio: number;
  },) {
    /**
     Destructured measurement values,
     each named once for the message below.
     */
    const {
      base,
      doubled,
      ratio,
    } = measurement;
    /**
     Base input size,
     named for the message.
     */
    const baseSize = base.size;
    /**
     Doubled input size,
     named for the message.
     */
    const doubledSize = doubled.size;
    /**
     Measured ratio,
     named for the printed figure.
     */
    const ratioText = ratio.toFixed(RATIO_DECIMALS,);
    /**
     Base best time,
     named for the printed figure.
     */
    const baseBestMs = base.bestMs;
    /**
     Base worst time,
     named for the printed figure.
     */
    const baseWorstMs = base.worstMs;
    /**
     Doubled best time,
     named for the printed figure.
     */
    const doubledBestMs = doubled.bestMs;
    /**
     Doubled worst time,
     named for the printed figure.
     */
    const doubledWorstMs = doubled.worstMs;
    /**
     Base best figure in milliseconds.
     */
    const baseBestText = baseBestMs.toFixed(MS_DECIMALS,);
    /**
     Base worst figure in milliseconds.
     */
    const baseWorstText = baseWorstMs.toFixed(MS_DECIMALS,);
    /**
     Doubled best figure in milliseconds.
     */
    const doubledBestText = doubledBestMs.toFixed(MS_DECIMALS,);
    /**
     Doubled worst figure in milliseconds.
     */
    const doubledWorstText = doubledWorstMs.toFixed(MS_DECIMALS,);
    super(`Time growth from ${baseSize} to ${doubledSize} inputs is ${ratioText}x per doubling, above the ${maxRatio}x bound: base best ${baseBestText} ms (worst ${baseWorstText} ms), doubled best ${doubledBestText} ms (worst ${doubledWorstText} ms). Author an input whose shape reaches the suspected path (a positive control), prove the path is exercised, and land the one-pass rewrite this bound demands.`);
    this.name = 'TwoSizeRatioError';
    this.measurement = measurement;
    this.maxRatio = maxRatio;
  }
}

/**
 Asserts that doubling the input size stays within `maxRatio` of the base time.

 The bound only reads the caller's input,
 so the caller must supply an exposing shape in `grow`
 and a test that shows the shape reaches the suspected code path;
 a shape that exits early passes a quadratic function
 (`doc/handover/superlinear-cost-dismissal-gate.md`,
 heading `The ratio gate separates the parsers only on an exposing shape`).
 What passing means is that growth stayed near-linear on that shape;
 it is not a proof of O(n) for unbounded input.

 @param grow - Produces the input at a given size, and owns the exposing shape

 @param run - Work under test; its return value is ignored

 @param size - Base input size; pick one whose base timing clears the clock floor

 @param repeats - Measured rounds per size; best of these feeds the ratio, and it must stay independent of size

 @param maxRatio - Largest acceptable time ratio per doubling

 @returns The measurement that passed

 @throws {@link TwoSizeRatioError} when growth exceeds `maxRatio`

 @example
 ```ts
 expectLinearTimeGrowth({
   grow: (n,) => `z${'  a b'.repeat(n,)}`,
   run: (row,) => parseBinList(row,),
 },);
 ```
 */
export function expectLinearTimeGrowth<Input>({
  grow,
  run,
  size = 2_000,
  repeats = 5,
  maxRatio = 2.5,
}: {
  grow: (size: number,) => Input;
  run: (input: Input,) => unknown;
  size?: number;
  repeats?: number;
  maxRatio?: number;
},): TwoSizeMeasurement {
  /**
   Measurement of the two sizes under the bound.
   */
  const measurement = measureTwoSizeRatio({
    grow,
    run,
    size,
    repeats,
  },);
  if (measurement.ratio > maxRatio) {
    throw new TwoSizeRatioError({
      measurement,
      maxRatio,
    },);
  }
  return measurement;
}

//endregion Assertion
