/**
 Tests the reading of `GET /v1/quota` and the difference of two readings, which
 is the only signal this pipeline has of plan energy a run spent: the free
 daily energy is reported nowhere, and the owner's other use of the covered
 models on the same account spends the same free energy where this meter
 cannot see it (`https://docs.neuralwatt.com/api/quota`, fetched 2026-10-20).

 The sample body is the documentation's own. Fixtures are otherwise invention.

 @module
 */

import {
  caught,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  describePlanSpend,
  NeuralWattQuotaShapeError,
  parseNeuralWattQuota,
  planSpendBetween,
} from '../dist/final/node/index.mjs';

/**
 The documentation's example response, whole.
 */
const DOCUMENTED = {
  snapshot_at: '2026-04-16T18:30:00Z',
  balance: {
    credits_remaining_usd: 32.6774,
    total_credits_usd: 52.34,
    credits_used_usd: 19.6626,
    accounting_method: 'energy',
  },
  usage: {
    lifetime: { cost_usd: 243.9145, requests: 37_801, tokens: 1_235_477_176, energy_kwh: 15.6009, },
    current_month: { cost_usd: 160.1463, requests: 23_902, tokens: 1_116_658_995, energy_kwh: 9.7278, },
  },
  limits: { overage_limit_usd: null, rate_limit_tier: 'standard', },
  subscription: {
    plan: 'standard',
    status: 'active',
    billing_interval: 'month',
    current_period_start: '2026-04-11T05:05:25Z',
    current_period_end: '2026-05-11T05:05:25Z',
    auto_renew: true,
    kwh_included: 20,
    kwh_used: 13.9023,
    kwh_remaining: 6.0977,
    in_overage: false,
  },
  key: { name: 'my-production-key', allowance: null, },
};

/**
 The documentation's example response as the text a reply carries.
 */
const DOCUMENTED_BODY = JSON.stringify(DOCUMENTED,);

/**
 The documented body with its subscription fields replaced.

 @param subscription - fields to merge over the documented subscription

 @returns Body text

 @example
 ```ts
 const bodyText = withSubscription({ kwh_used: 14, },);
 ```
 */
function withSubscription(subscription: Readonly<Record<string, unknown>>,): string {
  return JSON.stringify({
    ...DOCUMENTED,
    subscription: {
      ...DOCUMENTED.subscription,
      ...subscription,
    },
  },);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: parseNeuralWattQuota.name,
      children: [
        it({
          name: 'READS THE SUBSCRIPTION\'S CHARGED ENERGY, PERIOD AND OVERAGE OFF THE DOCUMENTED BODY, and ignores '
            + 'the fields it has no use for',
          fn: async () => {
            expect(parseNeuralWattQuota({ bodyText: DOCUMENTED_BODY, },),).toEqual({
              snapshotAt: '2026-04-16T18:30:00Z',
              subscription: {
                plan: 'standard',
                periodStart: '2026-04-11T05:05:25Z',
                kwhIncluded: 20,
                kwhUsed: 13.9023,
                kwhRemaining: 6.0977,
                inOverage: false,
              },
            },);
          },
        },),

        it({
          name: 'READS AN ACCOUNT WITH NO SUBSCRIPTION AS ONE, since the field is always present and null then',
          fn: async () => {
            expect(parseNeuralWattQuota({
              bodyText: JSON.stringify({ snapshot_at: '2026-10-20T20:00:00Z', subscription: null, },),
            },),).toEqual({
              snapshotAt: '2026-10-20T20:00:00Z',
            },);
          },
        },),

        it({
          name: 'READS A NULL FIELD AS UNKNOWN RATHER THAN AS ZERO, so an unreported figure is never counted as unspent',
          fn: async () => {
            expect(parseNeuralWattQuota({
              bodyText: withSubscription({ kwh_used: null, in_overage: null, current_period_start: null, },),
            },).subscription,).toEqual({
              plan: 'standard',
              kwhIncluded: 20,
              kwhRemaining: 6.0977,
            },);
          },
        },),

        it({
          name: 'REFUSES A BODY THAT IS NOT THE SHAPE, naming the broken expectation and never quoting the body',
          fn: async () => {
            for (
              const [bodyText, detail,] of [
                ['<html>gateway</html>', 'not valid JSON',],
                ['[1]', 'not a JSON object',],
                [JSON.stringify({ subscription: null, },), 'snapshot_at is not a string',],
                [JSON.stringify({ snapshot_at: '2026-10-20T20:00:00Z', },), 'subscription is absent',],
                [JSON.stringify({ snapshot_at: '2026-10-20T20:00:00Z', subscription: 5, },), 'subscription is neither an object nor null',],
                [withSubscription({ kwh_used: '13.9', },), 'kwh_used is neither a number nor null',],
                [withSubscription({ kwh_used: -1, },), 'kwh_used is neither a number nor null',],
                [withSubscription({ in_overage: 'no', },), 'in_overage is neither a boolean nor null',],
              ] as const
            ) {
              /**
               What reading that body throws.
               */
              const refusal = caught(function read(): void {
                parseNeuralWattQuota({ bodyText, },);
              },);
              expect(refusal,).toBeInstanceOf(NeuralWattQuotaShapeError,);
              expect(String(refusal,),).toContain(detail,);
              expect(String(refusal,),).not.toContain('gateway',);
            }
          },
        },),
      ],
    },),

    describe({
      name: planSpendBetween.name,
      children: [
        it({
          name: 'READS THE DIFFERENCE OF THE CHARGED ENERGY as plan energy spent, zero when nothing moved',
          fn: async () => {
            /**
             Reading at the start of a run.
             */
            const before = parseNeuralWattQuota({ bodyText: DOCUMENTED_BODY, },);
            expect(planSpendBetween({ before, after: before, },),).toEqual({ kind: 'measured', kwh: 0, inOverage: false, },);

            /**
             Reading at its end, after 0.05 kWh were charged.
             */
            const after = parseNeuralWattQuota({ bodyText: withSubscription({ kwh_used: 13.9523, kwh_remaining: 6.0477, },), },);
            const spent = planSpendBetween({ before, after, },);
            expect(spent.kind,).toBe('measured',);
            if (spent.kind === 'measured')
              expect(spent.kwh,).toBeCloseTo(0.05, 9,);
          },
        },),

        it({
          name: 'CARRIES THE OVERAGE FLAG OF THE LATER READING',
          fn: async () => {
            /**
             Reading at the start of a run.
             */
            const before = parseNeuralWattQuota({ bodyText: DOCUMENTED_BODY, },);
            /**
             Reading at its end, past the included energy.
             */
            const after = parseNeuralWattQuota({ bodyText: withSubscription({ kwh_used: 20.5, in_overage: true, },), },);
            expect(planSpendBetween({ before, after, },),).toMatchObject({ kind: 'measured', inOverage: true, },);
          },
        },),

        it({
          name: 'DECLINES TO COMPARE READINGS THAT CANNOT BE COMPARED, and says why: a billing period that rolled over '
            + 'between them, an account with no subscription, a figure the reply left out, or a total that fell',
          fn: async () => {
            /**
             Reading at the start of a run.
             */
            const before = parseNeuralWattQuota({ bodyText: DOCUMENTED_BODY, },);
            /**
             Readings that cannot be compared with it, with the reason each states.
             */
            const cases = [
              [withSubscription({ current_period_start: '2026-05-11T05:05:25Z', kwh_used: 0.2, },), 'billing period',],
              [JSON.stringify({ snapshot_at: '2026-10-20T20:00:00Z', subscription: null, },), 'no subscription',],
              [withSubscription({ kwh_used: null, },), 'did not report',],
              [withSubscription({ kwh_used: 3, },), 'fell',],
            ] as const;
            for (const [bodyText, reason,] of cases) {
              /**
               Comparison of the start with this reading.
               */
              const spent = planSpendBetween({ before, after: parseNeuralWattQuota({ bodyText, },), },);
              expect(spent.kind,).toBe('not-comparable',);
              if (spent.kind === 'not-comparable')
                expect(spent.reason,).toContain(reason,);
            }
          },
        },),
      ],
    },),

    describe({
      name: describePlanSpend.name,
      children: [
        it({
          name: 'SAYS PLAINLY THAT NO PLAN ENERGY WAS SPENT, and that the free energy is not in this figure',
          fn: async () => {
            /**
             The sentence for a run that moved nothing.
             */
            const said = describePlanSpend({ spend: { kind: 'measured', kwh: 0, inOverage: false, }, },);
            expect(said,).toContain('no plan energy was spent',);
            expect(said,).toContain('free daily energy',);
          },
        },),

        it({
          name: 'REPORTS A NONZERO DIFFERENCE AS PLAN ENERGY SPENT, in watt-hours, and says when the plan is in overage',
          fn: async () => {
            /**
             The sentence for a run that moved the plan's energy into overage.
             */
            const said = describePlanSpend({ spend: { kind: 'measured', kwh: 0.05, inOverage: true, }, },);
            expect(said,).toContain('PLAN ENERGY SPENT',);
            expect(said,).toContain('50 Wh',);
            expect(said,).toContain('in overage',);
          },
        },),

        it({
          name: 'SAYS THE COMPARISON STARTS AT A LATER READING WHEN THE FIRST READ FAILED, with that reading\'s time, '
            + 'and that energy spent before it is not counted, whether the difference is nonzero or zero',
          fn: async () => {
            /**
             The sentence for a nonzero difference measured from a later reading.
             */
            const spent = describePlanSpend({
              spend: { kind: 'measured', kwh: 0.05, inOverage: false, laterStartAt: '2026-10-20T12:01:00.000Z', },
            },);
            expect(spent,).toContain('PLAN ENERGY SPENT',);
            expect(spent,).toContain('50 Wh',);
            expect(spent,).toContain('2026-10-20T12:01:00.000Z',);
            expect(spent,).toContain('the first reading of the run failed',);
            expect(spent,).toContain('starts at that later reading',);
            expect(spent,).toContain('before it is not counted',);
            expect(spent,).not.toContain('between the start and the end of the run',);
            /**
             The sentence for no difference measured from a later reading.
             */
            const still = describePlanSpend({
              spend: { kind: 'measured', kwh: 0, inOverage: false, laterStartAt: '2026-10-20T12:01:00.000Z', },
            },);
            expect(still,).toContain('no plan energy was spent',);
            expect(still,).toContain('2026-10-20T12:01:00.000Z',);
            expect(still,).toContain('before it is not counted',);
            expect(still,).toContain('free daily energy',);
          },
        },),

        it({
          name: 'SAYS NOTHING OF A LATER READING WHEN THE FIRST READ WAS THE START, the sentences as they were',
          fn: async () => {
            expect(describePlanSpend({ spend: { kind: 'measured', kwh: 0.05, inOverage: false, }, },),).not.toContain('later',);
            expect(describePlanSpend({ spend: { kind: 'measured', kwh: 0, inOverage: false, }, },),).not.toContain('later',);
          },
        },),

        it({
          name: 'SAYS WHY WHEN THE READINGS COULD NOT BE COMPARED',
          fn: async () => {
            expect(describePlanSpend({ spend: { kind: 'not-comparable', reason: 'the billing period rolled over', }, },),)
              .toContain('the billing period rolled over',);
          },
        },),

        it({
          name: 'PRINTS WHAT THE RUN\'S OWN METER DREW BESIDE THE PLAN FIGURE and says the plan figure is the account\'s '
            + 'charge, not the run\'s draw (XingZ6022: 1730.5 Wh charged against 238.272 Wh drawn)',
          fn: async () => {
            /**
             The sentence for a plan figure far above what the run's own meter drew.
             */
            const said = describePlanSpend({ spend: { kind: 'measured', kwh: 1.7305, inOverage: false, }, drawnWh: 238.272, },);
            expect(said,).toContain('1730.5 Wh',);
            expect(said,).toContain('this run\'s own meter drew 238.272 Wh',);
            expect(said,).toContain('not this run\'s draw',);
            expect(said,).toContain('can include energy this run did not draw',);
            expect(said,).toContain('anything else done on the account in that time is in this figure too',);
          },
        },),

        it({
          name: 'PRINTS THE RUN\'S OWN DRAW ON EVERY BRANCH: a zero difference, a difference measured from a later '
            + 'reading, and readings that could not be compared; and leaves the sentences as they were where none is given',
          fn: async () => {
            /**
             The sentence for a zero difference.
             */
            const zero = describePlanSpend({ spend: { kind: 'measured', kwh: 0, inOverage: false, }, drawnWh: 0.0005, },);
            expect(zero,).toContain('no plan energy was spent',);
            expect(zero,).toContain('this run\'s own meter drew 0.0005 Wh',);
            expect(zero,).toContain('free daily energy',);
            /**
             The sentence for a difference measured from a later reading.
             */
            const later = describePlanSpend({
              spend: { kind: 'measured', kwh: 0.05, inOverage: false, laterStartAt: '2026-10-20T12:01:00.000Z', },
              drawnWh: 12,
            },);
            expect(later,).toContain('this run\'s own meter drew 12 Wh',);
            expect(later,).toContain('before it is not counted',);
            /**
             The sentence for readings that could not be compared.
             */
            const none = describePlanSpend({
              spend: { kind: 'not-comparable', reason: 'the billing period rolled over', },
              drawnWh: 3.25,
            },);
            expect(none,).toContain('the billing period rolled over',);
            expect(none,).toContain('this run\'s own meter drew 3.25 Wh',);
            expect(describePlanSpend({ spend: { kind: 'measured', kwh: 0.05, inOverage: false, }, },),)
              .not.toContain('own meter',);
          },
        },),
      ],
    },),
  ],
},);
