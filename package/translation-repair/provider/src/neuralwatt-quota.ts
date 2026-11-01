import { errorName, } from '@monochromatic-dev/translation-repair-markup/ts';
import { isJsonRecord, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  keyAllowanceOf,
  type NeuralWattKeyAllowance,
} from './neuralwatt-key-allowance.ts';

//region NeuralWatt quota
// THE ONE SIGNAL OF PLAN ENERGY SPENT, read twice a run.
//
// `GET /v1/quota` (`https://docs.neuralwatt.com/api/quota`, fetched
// 2026-10-20) reports the account's subscription: `kwh_included`, `kwh_used`
// (the energy CHARGED against the plan, which is consumed times the lane's
// multiplier), `kwh_remaining` and `in_overage`. It says NOTHING about the
// daily free energy, and the free energy is "used before anything else"
// (`https://docs.neuralwatt.com/billing/daily-free-energy`), so while it lasts
// `kwh_used` should not move. A run reads the quota once before its first
// NeuralWatt call and once at its end; a difference above zero is energy the
// run (or anything else on the account) took from the plan.
//
// WHAT THE DIFFERENCE CANNOT SEPARATE. The owner also uses the covered models
// on the same account, which spends the same free energy where this package's
// ledger cannot see it, and anything else the owner does on the account moves
// `kwh_used` too. A nonzero difference is a signal to look, not proof this
// run spent it; a zero difference proves the run spent no plan energy but
// not that the free energy lasted, since the free energy is not in this
// figure. The rate limit is one request a second per customer, far above two
// reads a run.

/**
 Watt-hours in one kilowatt-hour.
 */
const WH_PER_KWH = 1_000;

/**
 Decimals a watt-hour figure is printed to: enough to see a few thousandths of one.
 */
const WH_DECIMALS = 3;

/**
 Raised when a quota reply will not read as the documented shape.

 NEVER QUOTES THE BODY: the reply is an account's billing record.

 @example
 ```ts
 throw new NeuralWattQuotaShapeError({ detail: 'snapshot_at is not a string', },);
 ```
 */
export class NeuralWattQuotaShapeError extends Error {
  /**
   Declares this message safe to forward: an authored phrase.
   */
  readonly messageNamesOnly: true = true;

  /**
   Names what was wrong with the reply.

   @param detail - authored phrase naming the broken expectation

   @example
   ```ts
   new NeuralWattQuotaShapeError({ detail: 'not a JSON object', },);
   ```
   */
  public constructor({ detail, }: { readonly detail: string; },) {
    super(`NeuralWatt quota reply violated expectations: ${detail}`,);
    this.name = 'NeuralWattQuotaShapeError';
  }
}

/**
 The subscription's figures, each absent where the reply gave null.

 @example
 ```ts
 const subscription: NeuralWattSubscription = { plan: 'max', kwhUsed: 13.9, inOverage: false, };
 ```
 */
export type NeuralWattSubscription = {
  /**
   Plan name.
   */
  readonly plan?: string;

  /**
   Start of the billing period the figures belong to, ISO 8601.
   */
  readonly periodStart?: string;

  /**
   Energy the plan includes per period, in kWh.
   */
  readonly kwhIncluded?: number;

  /**
   Energy charged against the plan this period, in kWh.
   */
  readonly kwhUsed?: number;

  /**
   Energy the plan has left this period, in kWh.
   */
  readonly kwhRemaining?: number;

  /**
   Whether the plan is past its included energy.
   */
  readonly inOverage?: boolean;
};

/**
 One quota reading.

 @example
 ```ts
 const reading: NeuralWattQuota = { snapshotAt: '2026-10-20T20:00:00Z', };
 ```
 */
export type NeuralWattQuota = {
  /**
   When NeuralWatt took the snapshot, as it states it.
   */
  readonly snapshotAt: string;

  /**
   The subscription, absent for an account with none.
   */
  readonly subscription?: NeuralWattSubscription;

  /**
   The per-key cap, absent for an uncapped key or one that will not read
   (`neuralwatt-key-allowance.ts`).
   */
  readonly keyAllowance?: NeuralWattKeyAllowance;

  /**
   Why `key.allowance` will not read, an authored phrase; the rest of the reply is still good.
   */
  readonly keyAllowanceProblem?: string;
};

/**
 What two readings say about the plan.

 @example
 ```ts
 const spend: NeuralWattPlanSpend = { kind: 'measured', kwh: 0, inOverage: false, };
 ```
 */
export type NeuralWattPlanSpend =
  | {
    readonly kind: 'measured';
    readonly kwh: number;
    readonly inOverage: boolean;

    /**
     When the starting reading was taken, as an ISO 8601 instant, present only where it was not the run's first read:
     the first read failed and a later one took its place, so energy spent before this instant is not counted.
     */
    readonly laterStartAt?: string;
  }
    | {
      readonly kind: 'not-comparable';
      readonly reason: string;
    };

/**
 Reads an optional field of a subscription as the value it holds.

 @param record - the subscription object

 @param field - its field

 @param kind - the type the field must have when it is not null

 @returns The value in a list of one, or an empty list for null or absent

 @throws {@link NeuralWattQuotaShapeError} when the field has another type

 @example
 ```ts
 const [kwhUsed,] = fieldValuesOf({ record, field: 'kwh_used', kind: 'number', },);
 ```
 */
function fieldValuesOf(
  {
    record,
    field,
    kind,
  }: {
    readonly record: Readonly<Record<string, unknown>>;
    readonly field: string;
    readonly kind: 'string' | 'number' | 'boolean';
  },
): readonly unknown[] {
  /**
   Field as the reply carries it.
   */
  const value = record[field];
  if ((value === undefined) || (value === null))
    return [];

  /**
   Whether it has the type asked for and, for a number, is a usable amount.
   */
  const fits = ((typeof value) === kind) && (((typeof value) !== 'number') || (Number.isFinite(value,) && (value >= 0)));
  if (!fits)
    throw new NeuralWattQuotaShapeError({ detail: `${field} is neither a ${kind} nor null`, },);
  return [value,];
}

/**
 Reads the subscription object of a quota reply.

 @param subscription - the object

 @returns The subscription, each absent field left out

 @throws {@link NeuralWattQuotaShapeError} when a field has the wrong type

 @example
 ```ts
 const read = subscriptionOf({ subscription: { plan: 'max', kwh_used: 1, }, },);
 ```
 */
function subscriptionOf(
  { subscription, }: { readonly subscription: Readonly<Record<string, unknown>>; },
): NeuralWattSubscription {
  /**
   Plan name, if the reply gave one.
   */
  const [plan,] = fieldValuesOf({
    record: subscription,
    field: 'plan',
    kind: 'string',
  },);
  /**
   Start of the billing period, if the reply gave one.
   */
  const [periodStart,] = fieldValuesOf({
    record: subscription,
    field: 'current_period_start',
    kind: 'string',
  },);
  /**
   Energy the plan includes, if the reply gave it.
   */
  const [kwhIncluded,] = fieldValuesOf({
    record: subscription,
    field: 'kwh_included',
    kind: 'number',
  },);
  /**
   Energy charged this period, if the reply gave it.
   */
  const [kwhUsed,] = fieldValuesOf({
    record: subscription,
    field: 'kwh_used',
    kind: 'number',
  },);
  /**
   Energy left this period, if the reply gave it.
   */
  const [kwhRemaining,] = fieldValuesOf({
    record: subscription,
    field: 'kwh_remaining',
    kind: 'number',
  },);
  /**
   Whether the plan is in overage, if the reply said.
   */
  const [inOverage,] = fieldValuesOf({
    record: subscription,
    field: 'in_overage',
    kind: 'boolean',
  },);
  return {
    // Conditional spreads keep an absent field absent instead of undefined.
    ...(((typeof plan) === 'string') ? { plan, } : {}),
    ...(((typeof periodStart) === 'string') ? { periodStart, } : {}),
    ...(((typeof kwhIncluded) === 'number') ? { kwhIncluded, } : {}),
    ...(((typeof kwhUsed) === 'number') ? { kwhUsed, } : {}),
    ...(((typeof kwhRemaining) === 'number') ? { kwhRemaining, } : {}),
    ...(((typeof inOverage) === 'boolean') ? { inOverage, } : {}),
  };
}

/**
 Reads a quota reply.

 @param bodyText - body of a 200 reply

 @returns The reading

 @throws {@link NeuralWattQuotaShapeError} when the body is not the documented shape

 @example
 ```ts
 const quota = parseNeuralWattQuota({ bodyText: reply.bodyText, },);
 ```
 */
export function parseNeuralWattQuota({ bodyText, }: { readonly bodyText: string; },): NeuralWattQuota {
  /**
   Parsed body, unknown until checked.
   */
  const parsed: unknown = (function parse(): unknown {
    try {
      return JSON.parse(bodyText,);
    }
    catch (error) {
      throw new NeuralWattQuotaShapeError({ detail: `not valid JSON (${errorName({ error, },)})`, },);
    }
  })();
  if (!isJsonRecord(parsed,))
    throw new NeuralWattQuotaShapeError({ detail: 'not a JSON object', },);

  /**
   The two top-level fields read.
   */
  const {
    snapshot_at: snapshotAt,
    subscription,
  } = parsed;
  if ((typeof snapshotAt) !== 'string')
    throw new NeuralWattQuotaShapeError({ detail: 'snapshot_at is not a string', },);
  if (subscription === undefined)
    throw new NeuralWattQuotaShapeError({ detail: 'subscription is absent', },);

  /**
   The per-key cap, or why it will not read; neither breaks the rest of the reply.
   */
  const {
    allowance,
    problem,
  } = keyAllowanceOf({ key: parsed.key, },);
  /**
   The key's part of the reading, each absent field left out.
   */
  const keyPart = {
    ...((allowance === undefined) ? {} : { keyAllowance: allowance, }),
    ...((problem === undefined) ? {} : { keyAllowanceProblem: problem, }),
  };
  if (subscription === null)
    return {
      snapshotAt,
      ...keyPart,
    };
  if (!isJsonRecord(subscription,))
    throw new NeuralWattQuotaShapeError({ detail: 'subscription is neither an object nor null', },);
  return {
    snapshotAt,
    subscription: subscriptionOf({ subscription, },),
    ...keyPart,
  };
}

/**
 What the plan's charged energy did between two readings.

 @param before - reading at the start of the run

 @param after - reading at its end

 @returns The difference, or why the readings cannot be compared

 @example
 ```ts
 const spend = planSpendBetween({ before, after, },);
 ```
 */
export function planSpendBetween(
  {
    before,
    after,
  }: {
    readonly before: NeuralWattQuota;
    readonly after: NeuralWattQuota;
  },
): NeuralWattPlanSpend {
  if ((before.subscription === undefined) || (after.subscription === undefined))
    return {
      kind: 'not-comparable',
      reason: 'the account has no subscription, so the quota reports no plan energy',
    };
  if (before.subscription
    .periodStart
    !== after.subscription
      .periodStart)
    return {
      kind: 'not-comparable',
      reason: 'the billing period rolled over between the two readings, so the charged energy restarted',
    };

  /**
   Charged energy at the start.
   */
  const { kwhUsed: startKwh, } = before.subscription;

  /**
   Charged energy at the end.
   */
  const { kwhUsed: endKwh, } = after.subscription;
  if ((startKwh === undefined) || (endKwh === undefined))
    return {
      kind: 'not-comparable',
      reason: 'a quota reply did not report the energy charged this period',
    };
  if (endKwh < startKwh)
    return {
      kind: 'not-comparable',
      reason: 'the energy charged this period fell between the readings',
    };
  return {
    kind: 'measured',
    kwh: endKwh - startKwh,
    inOverage: after.subscription
      .inOverage
      === true,
  };
}

/**
 What the end report says of a comparison that starts at a later reading, so that the operator does not read it as
 covering the whole run.
 */
const LATER_START_NOTE = 'the first reading of the run failed, so this comparison starts at that later reading and '
  + 'plan energy spent before it is not counted';

/**
 The sentence a run's end prints about plan energy.

 @param spend - difference or reason for none, carrying `laterStartAt` where starting reading followed failed first read,
 because comparison from later start must not read as covering whole run

 @returns Plain statement, with what the figure cannot see

 @example
 ```ts
 rl.info(describePlanSpend({ spend, },),);
 ```
 */
export function describePlanSpend({ spend, }: { readonly spend: NeuralWattPlanSpend; },): string {
  if (spend.kind === 'not-comparable')
    return `NeuralWatt plan energy: no difference to report, because ${spend.reason}`;

  /**
   Where the comparison starts and what it leaves out, said only where the starting reading was taken after the run's
   first read failed.
   */
  const laterStart = (spend.laterStartAt === undefined)
    ? undefined
    : `the later reading taken at ${spend.laterStartAt}`;
  if (spend.kwh === 0)
    return (laterStart === undefined)
      ? 'NeuralWatt plan energy: no plan energy was spent between the start and the end of the run; '
        + 'the free daily energy is not in this figure'
      : `NeuralWatt plan energy: no plan energy was spent between ${laterStart} and the end of the run; ${LATER_START_NOTE}; `
        + 'the free daily energy is not in this figure';

  /**
   The difference in watt-hours, without float dust.
   */
  const wh = Number((spend.kwh * WH_PER_KWH).toFixed(WH_DECIMALS,),);
  return `NeuralWatt PLAN ENERGY SPENT: ${String(wh,)} Wh of the owner's plan were charged between ${
    laterStart ?? 'the start'
  } and the end of the run${spend.inOverage ? ', and the plan is in overage' : ''}; ${
    (laterStart === undefined) ? '' : `${LATER_START_NOTE}; `
  }anything else done on the account in that time is in this figure too`;
}

//endregion NeuralWatt quota
