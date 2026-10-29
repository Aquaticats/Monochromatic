import { countWithNoun, } from './editor-fidelity-arm-facts.ts';
import type {
  EffortReading,
  EffortVerdict,
} from './editor-fidelity-decide-types.ts';
import { ALPHA, } from './editor-fidelity-paired.ts';

//region Editor fidelity decision lines
// THE LINES THE READ STEP PRINTS FOR THE DECISION RULE: a paired line and a kinds line for each effort, and the verdict. The
// rule is `editor-fidelity-decide.ts`; this is how its numbers are said, the points to a tenth with their sign and the
// p-value to four decimals.

/**
 Points in a fraction.
 */
const POINTS_PER_UNIT = 100;

/**
 Decimals of a printed point.
 */
const POINT_DECIMALS = 1;

/**
 Tenths of a point in a point, which a printed figure is rounded to.
 */
const TENTHS = 10;

/**
 The decimals of a printed p-value.
 */
const P_DECIMALS = 4;

/**
 A p-value under this is printed as under it, since four decimals would print it as zero.
 */
const P_PRINTED_UNDER = 0.0001;

/**
 A fraction as signed points to a tenth, `+12.5`, `-3.0`, `+0.0`.

 @param fraction - fraction

 @returns Points, signed

 @example
 ```ts
 // yields '+50.0'
 const said = pointsSaid({ fraction: 0.5, },);
 ```
 */
function pointsSaid({ fraction, }: { readonly fraction: number; },): string {
  /**
   The points rounded to a tenth.
   */
  const rounded = Math.round(fraction * POINTS_PER_UNIT
    * TENTHS,) / TENTHS;
  return `${(rounded >= 0) ? '+' : '-'}${Math.abs(rounded,)
    .toFixed(POINT_DECIMALS,)}`;
}

/**
 A p-value as the lines print it.

 @param p - p-value

 @returns Four decimals, or `under 0.0001` for a value four decimals would print as zero

 @example
 ```ts
 // yields 'under 0.0001'
 const said = pSaid({ p: 0.00000095, },);
 ```
 */
export function pSaid({ p, }: { readonly p: number; },): string {
  return (p < P_PRINTED_UNDER) ? `under ${String(P_PRINTED_UNDER,)}` : p.toFixed(P_DECIMALS,);
}

/**
 The paired line of an effort.

 @param reading - how the effort stands

 @param trials - shared trials the verdict is over

 @param entries - entries those trials come from

 @returns Line, without a newline

 @example
 ```ts
 const line = pairedLine({ reading, trials: 80, entries: 47, },);
 ```
 */
export function pairedLine(
  {
    reading,
    trials,
    entries,
  }: {
    readonly reading: EffortReading;
    readonly trials: number;
    readonly entries: number;
  },
): string {
  return `PAIRED ${reading.effort} vs medium: ${pointsSaid({ fraction: reading.estimate, },)} points (95% interval ${
    pointsSaid({ fraction: reading.interval
      .low, },)
  } to ${pointsSaid({ fraction: reading.interval
    .high, },)} points, seeded bootstrap over ${
    countWithNoun({
      count: entries,
      one: 'entry',
      many: 'entries',
    },)
  }), p ${pSaid({ p: reading.p, },)}, shown better at ${String(ALPHA,)}: ${reading.shown ? 'yes' : 'no'}; ${
    countWithNoun({
      count: trials,
      one: 'shared unit',
      many: 'shared units',
    },)
  } over ${
    countWithNoun({
      count: entries,
      one: 'entry',
      many: 'entries',
    },)
  }`;
}

/**
 The kinds line of an effort, the secondary figure that vetoes nothing.

 @param reading - how the effort stands

 @returns Line, without a newline

 @example
 ```ts
 const line = kindsLine({ reading, },);
 ```
 */
export function kindsLine({ reading, }: { readonly reading: EffortReading; },): string {
  return `KINDS ${reading.effort} vs medium (secondary, no veto): insertion ${
    pointsSaid({ fraction: reading.kinds
      .insertion
      .estimate, },)
  } points over ${
    countWithNoun({
      count: reading.kinds
        .insertion
        .trials,
      one: 'unit',
      many: 'units',
    },)
  }, alteration ${pointsSaid({ fraction: reading.kinds
    .alteration
    .estimate, },)} points over ${
    countWithNoun({
      count: reading.kinds
        .alteration
        .trials,
      one: 'unit',
      many: 'units',
    },)
  }`;
}

/**
 Settles which effort the rule picks among the efforts shown better, and says so.

 @param readings - how each effort stands

 @param shown - efforts shown better

 @returns Effort picked, clause that decided and verdict line

 @example
 ```ts
 const { verdict, clause, said, } = decidedOf({ readings, shown, },);
 ```
 */
export function decidedOf(
  {
    readings,
    shown,
  }: {
    readonly readings: {
      readonly high: EffortReading;
      readonly low: EffortReading
    };
    readonly shown: readonly ('high' | 'low')[];
  },
): Pick<EffortVerdict, 'clause' | 'said' | 'verdict'> {
  if (shown.length === 0) {
    return {
      verdict: 'medium',
      clause: 'none-shown',
      said: `VERDICT medium (none-shown): no effort was shown better than medium at ${String(ALPHA,)}, so the default `
        + 'effort stays; that does not show the others are as good',
    };
  }
  if (shown.length === 1) {
    /**
     The one effort shown better.
     */
    const effort = shown[0] ?? 'high';
    return {
      verdict: effort,
      clause: 'shown-better',
      said: `VERDICT ${effort} (shown-better): ${effort} is shown better than medium by ${
        pointsSaid({ fraction: readings[effort]
          .estimate, },)
      } points (p ${pSaid({ p: readings[effort]
        .p, },)}) and the other effort is not, so ${effort} is chosen`,
    };
  }
  /**
   Both efforts are shown better.
   */
  const {
    high,
    low,
  } = readings;
  if (high.estimate === low.estimate) {
    return {
      verdict: 'low',
      clause: 'cheaper-on-equal',
      said: `VERDICT low (cheaper-on-equal): both efforts are shown better than medium with equal estimates (${
        pointsSaid({ fraction: high.estimate, },)
      } points), so the cheaper effort, low, is chosen`,
    };
  }
  /**
   The effort with the larger estimate.
   */
  const larger = (high.estimate > low.estimate) ? high : low;
  /**
   The other.
   */
  const smaller = (larger === high) ? low : high;
  return {
    verdict: larger.effort,
    clause: 'larger-estimate',
    said: `VERDICT ${larger.effort} (larger-estimate): both efforts are shown better than medium; ${larger.effort} has the `
      + `larger estimate (${pointsSaid({ fraction: larger.estimate, },)} against ${
        pointsSaid({ fraction: smaller.estimate, },)
      } points), so ${larger.effort} is chosen`,
  };
}

//endregion Editor fidelity decision lines
