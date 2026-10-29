import {
  type ArmLog,
  countWithNoun,
} from './editor-fidelity-arm-facts.ts';
import {
  effortOf,
  sharedBenchOf,
  withFiguresOf,
} from './editor-fidelity-arm-shared.ts';
import {
  DEFAULT_FIX_RATE_WARNED_AT_PERCENT,
  FALSE_PICK_BOUND_ALIKE,
  FALSE_PICK_READINGS_ABOVE_NINE_TENTHS,
} from './editor-fidelity-decide.ts';

//region Editor fidelity arm fix rate
// THE FIX RATE OF THE DEFAULT EFFORT, which the repeat warning's reach depends on. The rule states its false pick bounds
// for true fix rates up to 0.90; above that the chance of choosing another effort when none is better rose (0.108 at 0.91,
// 0.123 at 0.92 and about 0.13 at 0.93 in the fixture), while an arm that gives a fresh reading in each pass reads its
// passes alike more often as the fix rate rises, so the repeat comparison sees it less. When the default effort cleanly
// fixes 90 percent of the units every arm holds, or more, the read step says so, beside the repeat lines, so that a verdict
// of high or low is read under that caveat whether or not a repeat warning printed. The share is observed and not true,
// so the line meets the rule's stated region at its edge. It is counted by cross-multiplying, so that a share exactly at
// the limit warns.

/**
 Says that the default effort fixes nine in ten of the units the arms share, or more.

 @param arm - name of the arm of the default effort

 @param fixed - shared units it cleanly fixed

 @param units - shared units

 @returns Line, without a newline

 @example
 ```ts
 const line = warningLine({ arm: 'medium', fixed: 90, units: 100, },);
 ```
 */
function warningLine(
  {
    arm,
    fixed,
    units,
  }: {
    readonly arm: string;
    readonly fixed: number;
    readonly units: number;
  },
): string {
  return `WARN high fix rate: ${arm} cleanly fixed ${String(fixed,)} of ${
    countWithNoun({
      count: units,
      one: 'shared unit',
      many: 'shared units',
    },)
  } (${String(Math.round((100 * fixed) / units,),)} percent), at or above ${
    String(DEFAULT_FIX_RATE_WARNED_AT_PERCENT,)
  } percent; at fix rates this high the repeat warning can miss unequal repeat agreement, so any verdict of high or low `
    + `stands under the caveat that its chance of being a false pick can be above ${String(FALSE_PICK_BOUND_ALIKE,)}, `
    + 'whether or not a WARN repeat agreement line printed; the committed fixture read '
    + `${FALSE_PICK_READINGS_ABOVE_NINE_TENTHS} when the default effort repeats itself far less than the others`;
}

/**
 The lines the read step prints for the fix rate of the arms of the default effort: one for each that cleanly fixed the
 stated share of the units every arm holds, or more.

 @param arms - arms in the order they ran, one or more

 @returns WARN lines, none when the logs share no unit, which is how one log read alone reads

 @example
 ```ts
 for (const said of fixRateLinesOf({ arms, },))
   process.stdout.write(`${said}\n`,);
 ```
 */
export function fixRateLinesOf({ arms, }: { readonly arms: readonly ArmLog[]; },): readonly string[] {
  if (arms.length < 2)
    return [];
  /**
   Each arm with the figures its log states.
   */
  const withFigures = withFiguresOf({ arms, },);
  /**
   The trials every arm completed and none threw on.
   */
  const { trials, } = sharedBenchOf({ arms: withFigures, },);
  if (trials.length === 0)
    return [];
  /**
   The names of the arms of the default effort, in the order the shared trials list their readings.
   */
  const defaultArms = withFigures
    .filter(function isDefault({ figures, },): boolean {
      return effortOf({ card: figures.card, },) === 'medium';
    },)
    .map(function nameOf({ arm, },): string {
      return arm;
    },);
  return defaultArms.flatMap(function warningOf(
    arm,
    at,
  ): readonly string[] {
    /**
     The shared units this arm cleanly fixed.
     */
    const fixed = trials.filter(function fixedBy({ mediums, },): boolean {
      return mediums[at] === true;
    },)
      .length;
    if ((100 * fixed) < (DEFAULT_FIX_RATE_WARNED_AT_PERCENT * trials.length))
      return [];
    return [
      warningLine({
        arm,
        fixed,
        units: trials.length,
      },),
    ];
  },);
}

//endregion Editor fidelity arm fix rate
