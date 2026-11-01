import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary/ts';
import {
  NEURALWATT_DAY_START_VAR,
  neuralWattDayAfter,
} from './neuralwatt-day.ts';
import type { NeuralWattKeyAllowance, } from './neuralwatt-key-allowance.ts';

//region NeuralWatt energy reading
// WHAT THE METER SAYS AND THE REFUSAL IT GIVES.
//
// THE FREE ENERGY (`https://docs.neuralwatt.com/billing/daily-free-energy`,
// fetched 2026-10-20): 500 Wh for a Max plan plus 50 Wh for subscribers to the
// product-news email, 550 Wh a day on the covered models, "used before
// anything else", with unused energy not carried over. The documentation says
// it resets at 00:00 UTC; the owner's account starts its day at the time of day
// of the subscription's period start instead (`neuralwatt-day.ts`). No API
// reports what is left of it, which is why this package keeps its own meter
// (`neuralwatt-ledger.ts`), floored by what the account's usage route says the
// whole account spent (`neuralwatt-usage.ts`).

/**
 Decimals a watt-hour figure is printed to: a call is a few thousandths of one.
 */
const WH_DECIMALS = 3;

/**
 The day a reading or a reservation is made in: where it started and what the account and the key's cap said, or that
 the start is unknown.

 @example
 ```ts
 const day: NeuralWattDay = { kind: 'anchored', start: new Date('2026-10-20T19:15:08Z',), accountWh: 4.095, };
 ```
 */
export type NeuralWattDay =
  | {
    readonly kind: 'anchored';

    /**
     First instant of the day; the day runs 24 hours from it.
     */
    readonly start: Date;

    /**
     Covered energy the whole account used since the start, as the usage route last said; absent where it was not read
     for this day.
     */
    readonly accountWh?: number;

    /**
     The key's cap as the quota last said, absent for an uncapped key.
     */
    readonly keyAllowance?: NeuralWattKeyAllowance;

    /**
     When a blocked cap lets requests through again; absent while it does not hold the day.
     */
    readonly keyBlockedUntil?: Date;
  }
    | {
      readonly kind: 'unanchored';
    };

/**
 One reading of the day's NeuralWatt energy.

 SPENT IS THE LARGER OF TWO FIGURES, never their sum: the ledger's own settled energy since the day's start
 (`spentWh`) and the whole account's covered energy since then (`accountWh`), each counting the same calls where it
 saw them. The holds of calls in flight come on top. `remainingWh` is the allowance less that.

 @example
 ```ts
 const energy: NeuralWattEnergy = {
   dayStart: '2026-10-20T19:15:08.000Z',
   dailyWh: 550,
   spentWh: 12.5,
   reckonedWh: 0.5,
   reservedWh: 0.1,
   remainingWh: 537.4,
   calls: 40,
 };
 ```
 */
export type NeuralWattEnergy = {
  /**
   First instant of the day the reading covers, as `toISOString` spells it; absent where the day start is unknown,
   when nothing can be sent.
   */
  readonly dayStart?: string;

  /**
   Free energy the day holds, in watt-hours.
   */
  readonly dailyWh: number;

  /**
   Watt-hours the ledger lines of the day add up to.
   */
  readonly spentWh: number;

  /**
   Covered watt-hours the whole account used since the day's start, as the usage route last said; absent where it
   could not be read for this day.
   */
  readonly accountWh?: number;

  /**
   The part of that which is an estimate, not a measurement.
   */
  readonly reckonedWh: number;

  /**
   Watt-hours held by calls now in flight, not yet in the ledger.
   */
  readonly reservedWh: number;

  /**
   What the day has left after the spend and the holds; negative once a
   measurement came in over its estimate.
   */
  readonly remainingWh: number;

  /**
   Ledger lines of the day.
   */
  readonly calls: number;

  /**
   The key's cap as the quota last said, absent for an uncapped key or one that would not read.
   */
  readonly keyAllowance?: NeuralWattKeyAllowance;

  /**
   When a blocked cap lets requests through again, as `toISOString` spells it; present only while it holds the day.
   */
  readonly keyBlockedUntil?: string;
};

/**
 Says why a call was not sent, in the day's own terms.

 @param model - NeuralWatt model id the call was for

 @param estimateWh - energy the call would have held

 @param energy - the day's reading the refusal rests on

 @returns The sentence

 @example
 ```ts
 const says = refusalTextOf({ model: 'glm-5.3-flash', estimateWh: 0.3, energy, },);
 ```
 */
function refusalTextOf(
  {
    model,
    estimateWh,
    energy,
  }: {
    readonly model: string;
    readonly estimateWh: number;
    readonly energy: NeuralWattEnergy;
  },
): string {
  if (energy.dayStart === undefined)
    return `NeuralWatt's day start is unknown, so the call to ${model} was not sent: the quota reply gave no `
      + `subscription.current_period_start and ${NEURALWATT_DAY_START_VAR} is not set; make GET /v1/quota readable `
      + `or set ${NEURALWATT_DAY_START_VAR}=HH:MM (UTC)`;
  if (energy.keyBlockedUntil !== undefined)
    return `NeuralWatt's allowance on this key is blocked until ${energy.keyBlockedUntil} (UTC), so the call to ${model} `
      + 'was not sent';

  /**
   End of the day the reading covers.
   */
  const dayEnd = neuralWattDayAfter({ start: new Date(energy.dayStart,), },)
    .toISOString();
  return `NeuralWatt has ${energy.remainingWh
    .toFixed(WH_DECIMALS,)} Wh of the ${String(energy.dailyWh,)} Wh free energy `
    + `left in the day that began ${energy.dayStart} (UTC) after ${energy.reservedWh
      .toFixed(WH_DECIMALS,)} Wh held by calls in flight${
     (energy.accountWh === undefined)
      ? ''
      : `, counting the ${energy.accountWh
        .toFixed(WH_DECIMALS,)} Wh the whole account used since then`
     }, and the call to ${model} is estimated at ${estimateWh.toFixed(WH_DECIMALS,)} Wh; it was not sent, and the `
    + `energy returns when that day ends at ${dayEnd} (UTC)`;
}

/**
 Raised, before any request leaves, when the day cannot take a call: the
 estimate does not fit in what is left, the key's cap is blocked, or the day's
 start is unknown.

 A BUDGET REFUSAL the router moves on from (`isBudgetRefusal`), and not a
 payment refusal: nothing is wrong with the account, the day's free energy is
 spent or held, and it returns when the day ends.

 @example
 ```ts
 throw new NeuralWattAllowanceError({ model: 'glm-5.3-flash', estimateWh: 0.3, energy, },);
 ```
 */
export class NeuralWattAllowanceError extends StatedRefusalError {
  /**
   Declares this message safe to forward: a model id, a day, and figures the
   ledger computed.
   */
  override readonly messageNamesOnly: true = true;

  /**
   What the day held when the call was refused.
   */
  readonly energy: NeuralWattEnergy;

  /**
   Names the model, the estimate and what the day had left.

   @param model - NeuralWatt model id the call was for

   @param estimateWh - energy the call would have held

   @param energy - the day's reading the refusal rests on

   @example
   ```ts
   new NeuralWattAllowanceError({ model: 'glm-5.3-flash', estimateWh: 0.3, energy, },);
   ```
   */
  public constructor(
    {
      model,
      estimateWh,
      energy,
    }: {
      readonly model: string;
      readonly estimateWh: number;
      readonly energy: NeuralWattEnergy;
    },
  ) {
    super({ says: refusalTextOf({
      model,
      estimateWh,
      energy,
    },), },);
    this.name = 'NeuralWattAllowanceError';
    this.energy = energy;
  }
}

//endregion NeuralWatt energy reading
