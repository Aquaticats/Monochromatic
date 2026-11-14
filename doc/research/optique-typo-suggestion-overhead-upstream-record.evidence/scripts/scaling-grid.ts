// The three one-factor-at-a-time sweeps of the scaling experiment, shared by
// the container runner and the host-side fits so both agree on the levels.

//region Types

/**
 * Which factor a sweep varies.
 */
export type SweepName = 'tokens' | 'options' | 'length';

/**
 * One sweep: the factor it varies, the levels of that factor, and the fixed
 * values of the other two.
 */
export type Sweep = {
  readonly name: SweepName;
  readonly levels: readonly number[];
  readonly fixedOptionCount: number;
  readonly fixedTokenLength: number;
  readonly fixedTokenCount: number;
};

//endregion Types

//region Grid

/**
 * Every sweep. Levels are log-spaced across the range the brief asks for, and
 * the fixed values keep each point inside about a second for the slowest arm.
 */
export const SWEEPS: readonly Sweep[] = [
  {
    name: 'tokens',
    levels: [
      100,
      200,
      500,
      1000,
      2000,
      5000,
    ],
    fixedOptionCount: 16,
    fixedTokenLength: 12,
    fixedTokenCount: 0,
  },
  {
    name: 'options',
    levels: [
      4,
      8,
      16,
      32,
      64,
      128,
    ],
    fixedOptionCount: 0,
    fixedTokenLength: 12,
    fixedTokenCount: 100,
  },
  {
    name: 'length',
    levels: [
      4,
      8,
      12,
      16,
      32,
      64,
      128,
      256,
    ],
    fixedOptionCount: 16,
    fixedTokenLength: 0,
    fixedTokenCount: 500,
  },
];

/**
 * Builds the point a sweep measures at one level.
 *
 * @param sweep - Sweep definition.
 *
 * @param level - Value of the varied factor.
 *
 * @returns Option count, token length and token count for that point.
 */
export function pointAt(sweep: Sweep, level: number,): {
  readonly optionCount: number;
  readonly tokenLength: number;
  readonly tokenCount: number;
} {
  return {
    optionCount: sweep.name === 'options' ? level : sweep.fixedOptionCount,
    tokenLength: sweep.name === 'length' ? level : sweep.fixedTokenLength,
    tokenCount: sweep.name === 'tokens' ? level : sweep.fixedTokenCount,
  };
}

//endregion Grid
