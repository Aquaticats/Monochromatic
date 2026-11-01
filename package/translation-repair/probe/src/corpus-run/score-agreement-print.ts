import type {
  AgreementTally,
  PrecisionTally,
} from '@monochromatic-dev/translation-repair-slice-reading/ts';

//region Score agreement print
// The lines `score-agreement` prints: precision over the graded sheet, and
// agreement against the blind pre-grades.

/**
 Decimal places every printed rate carries.

 Three, because the gate bar is quoted to one place (0.9) and a reading has to
 be comparable across rounds without a tie at the bar reading as a pass.
 */
const RATE_DECIMALS = 3;

/**
 Renders one rate to three places, naming an empty denominator rather than
 printing a division by zero.

 @param numerator - items counted in favor

 @param denominator - items the rate is taken over

 @returns Rate text

 @example
 ```ts
 const text = agreementRate({ numerator: 37, denominator: 47, },);
 ```
 */
export function agreementRate(
  {
    numerator,
    denominator,
  }: {
    readonly numerator: number;
    readonly denominator: number;
  },
): string {
  if (denominator === 0)
    return 'n/a';
  return (numerator / denominator)
    .toFixed(RATE_DECIMALS,);
}

/**
 Lists sheet positions, naming an empty list rather than printing nothing.

 @param positions - sheet positions

 @returns Positions joined by commas, or `none`

 @example
 ```ts
 const text = positionsOrNone({ positions: [3, 4,], },); // '3,4'
 ```
 */
export function positionsOrNone({ positions, }: { readonly positions: readonly number[]; },): string {
  if (positions.length === 0)
    return 'none';
  return positions.join(',',);
}

/**
 Prints precision over the graded sheet.

 Three rates, because a declined item has three defensible readings and the
 recorded verdicts quote all of them. Round two's archived sheet reproduces
 its published 0.740 / 0.787 / 0.800 exactly through these, which is the
 check that this reader agrees with how the number was reported before.

 @param items - items on the sheet

 @param precision - counts over the items the human scored

 @example
 ```ts
 printPrecision({ items: human.length, precision, },);
 ```
 */
export function printPrecision(
  {
    items,
    precision,
  }: {
    readonly items: number;
    readonly precision: PrecisionTally;
  },
): void {
  console.log(
    `PRECISION items=${String(items,)} gradeable=${
      String(precision.gradeable,)
    } scored=${
      String(precision.scored,)
    } realDefects=${String(precision.realDefects,)} strict=${
      agreementRate({
        numerator: precision.realDefects,
        denominator: precision.gradeable,
      },)
    } excluded=${
      agreementRate({
        numerator: precision.realDefects,
        denominator: precision.scored,
      },)
    } lenient=${
      agreementRate({
        numerator: precision.realDefects
          + precision.unscored
            .length,
        denominator: precision.gradeable,
      },)
    } duplicates=${positionsOrNone({ positions: precision.duplicates, },)} unscored=${
      positionsOrNone({ positions: precision.unscored, },)
    }`,
  );
}

/**
 Prints agreement between the blind pre-grades and the human's grades.

 @param agreement - agreement over the items the human scored

 @example
 ```ts
 printAgreement({ agreement, },);
 ```
 */
export function printAgreement({ agreement, }: { readonly agreement: AgreementTally; },): void {
  console.log(
    `AGREEMENT compared=${String(agreement.compared,)} agreed=${
      String(agreement.agreed,)
    } rate=${
      agreementRate({
        numerator: agreement.agreed,
        denominator: agreement.compared,
      },)
    } disagreed=${positionsOrNone({ positions: agreement.disagreed, },)}`,
  );
}

//endregion Score agreement print
