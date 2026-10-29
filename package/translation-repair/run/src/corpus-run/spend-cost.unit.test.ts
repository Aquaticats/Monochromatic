/**
 Tests for pricing a spend tally.

 THE ORDERING CASE IS BUILT TO FAIL IF THE SORT IS INHERITED. `tallySpend`
 already returns seats by completion tokens, so a `priceTally` that kept that
 order would pass any case where the two agree. The seats here disagree on
 purpose: the one with ten times the tokens costs a quarter as much, because
 the output rates it sits between differ by forty times.

 THE THREE BUCKETS ARE THE POINT. Metered and priced, metered and unknown to
 the table, and flat-subscription. Only the first has a credit figure, and a
 case for each is what stops the third being converted into a currency it does
 not bill in.

 THE ROUND TRIP BINDS THE WHOLE CHAIN: a line `reportSpend` wrote, read by
 `tallySpend`, priced by `priceTally`. That is the path a real report walks.

 Model identifiers come from the catalog. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { reportSpend, } from '@monochromatic-dev/translation-repair-provider';
import {
  HYPER_PRICE_READ_ON,
  priceTally,
  tallySpend,
} from '../../dist/final/node/index.mjs';
import { SEAT_SYNTHETIC_VISION_WITHHELD, } from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

/**
 Builds a tally from record tails, which is how every case here starts.

 @param tails - record text of each line, marker word onward

 @returns Tally over those records

 @example
 ```ts
 const tally = tallyOf({ tails: ['SPEND provider=hyper model=kimi-k3 prompt=0 completion=1',], },);
 ```
 */
function tallyOf(
  { tails, }: { readonly tails: readonly string[]; },
): ReturnType<typeof tallySpend> {
  return tallySpend({ lines: tails, },);
}

await describe({
  name: priceTally.name,
  children: [
    it({
      name: 'ORDERS priced seats by what they cost, not by the token order the '
        + 'tally handed over: the seat with ten times the tokens here is about '
        + 'a sixth of the bill',
      fn: async () => {
        expect(
          priceTally({
            tally: tallyOf({
              tails: [
                'SPEND provider=hyper model=gemma-4-26b-a4b-it prompt=0 completion=1000000',
                'SPEND provider=hyper model=kimi-k3 prompt=0 completion=100000',
              ],
            },),
          },)
            .priced
            .map(function named(seat,): string {
              return `${seat.model} ${String(seat.totalCredits,)}`;
            },),
        )
          .toEqual([
            'kimi-k3 32.664',
            'gemma-4-26b-a4b-it 6.68',
          ],);
      },
    },),

    it({
      name: 'KEEPS subscription seats out of the credit total, since that '
        + 'provider bills a weekly allowance and no credits at all',
      fn: async () => {
        /**
         Tally holding one seat of each provider, same model spelling apart.
         */
        const cost = priceTally({
          tally: tallyOf({
            tails: [
              'SPEND provider=hyper model=kimi-k3 prompt=0 completion=100000',
              'SPEND provider=synthetic model=hf:moonshotai/Kimi-K3 prompt=0 completion=100000',
            ],
          },),
        },);

        expect({
          total: cost.totalCredits,
          priced: cost
            .priced
            .length,
          subscription: cost
            .subscription
            .map(function named(seat,): string {
              return seat.model;
            },),
        },)
          .toEqual({
            total: 32.664,
            priced: 1,
            subscription: [SEAT_SYNTHETIC_VISION_WITHHELD,],
          },);
      },
    },),

    it({
      name: 'KEEPS NeuralWatt seats in a bucket of their own, out of the credit total and the USD total, since the '
        + 'free daily energy bills neither and is metered in watt-hours by the energy ledger',
      fn: async () => {
        /**
         Tally holding a Hyper seat, a NeuralWatt seat on the free energy and one that reported a nominal cost.
         */
        const cost = priceTally({
          tally: tallyOf({
            tails: [
              'SPEND provider=hyper model=kimi-k3 prompt=0 completion=100000',
              'SPEND provider=neuralwatt model=deepseek-v4.1-flash prompt=3000 completion=400 cost=0',
              'SPEND provider=neuralwatt model=glm-5.3-flash prompt=2000 completion=300 cost=0.0004',
            ],
          },),
        },);

        expect({
          total: cost.totalCredits,
          usd: cost.totalUsd,
          priced: cost
            .priced
            .length,
          freeEnergy: cost
            .freeEnergy
            .map(function named(seat,): string {
              return `${seat.model} ${String(seat.costUsd,)}`;
            },)
            .toSorted(),
        },)
          .toEqual({
            total: 32.664,
            usd: 0,
            priced: 1,
            freeEnergy: ['deepseek-v4.1-flash 0', 'glm-5.3-flash 0.0004',],
          },);
      },
    },),

    it({
      name: 'REPORTS a metered seat the table has no row for as unpriced, so '
        + 'the total reads as incomplete rather than as cheap',
      fn: async () => {
        /**
         Tally holding one priced seat and one the table never heard of.
         */
        const cost = priceTally({
          tally: tallyOf({
            tails: [
              'SPEND provider=hyper model=kimi-k3 prompt=0 completion=100000',
              'SPEND provider=hyper model=cat-nap-9000 prompt=0 completion=900000',
            ],
          },),
        },);

        expect({
          total: cost.totalCredits,
          unpriced: cost
            .unpriced
            .map(function named(seat,): string {
              return seat.model;
            },),
        },)
          .toEqual({
            total: 32.664,
            unpriced: ['cat-nap-9000',],
          },);
      },
    },),

    it({
      name: 'SUMS the calls that reported no usage across every seat, so a '
        + 'reader can see how wide a floor the total is',
      fn: async () => {
        expect(
          priceTally({
            tally: tallyOf({
              tails: [
                'SPEND provider=hyper model=kimi-k3 prompt=unreported completion=unreported',
                'SPEND provider=synthetic model=hf:openai/gpt-oss-120b prompt=unreported completion=unreported',
                'SPEND provider=hyper model=kimi-k3 prompt=10 completion=20',
              ],
            },),
          },)
            .unreportedCalls,
        )
          .toBe(2,);
      },
    },),

    it({
      name: 'REPORTS an empty tally as no seats and no credits, which is the '
        + 'shape every log written before the writer landed produces',
      fn: async () => {
        /**
         Cost of a log that carried no record at all.
         */
        const cost = priceTally({ tally: tallyOf({ tails: ['ordinary log line',], },), },);

        expect({
          priced: cost
            .priced
            .length,
          unpriced: cost
            .unpriced
            .length,
          subscription: cost
            .subscription
            .length,
          total: cost.totalCredits,
        },)
          .toEqual({
            priced: 0,
            unpriced: 0,
            subscription: 0,
            total: 0,
          },);
      },
    },),

    it({
      name: 'CARRIES the date the rates were read, so a report printing credits '
        + 'can say how old the figures are',
      fn: async () => {
        expect(priceTally({ tally: tallyOf({ tails: [], },), },).pricedAsOf,)
          .toBe(HYPER_PRICE_READ_ON,);
      },
    },),

    it({
      name: 'JOINS the writer to the price: a line `reportSpend` returned, read '
        + 'back and costed, which is the path a real report walks',
      fn: async () => {
        expect(
          priceTally({
            tally: tallyOf({
              tails: [
                reportSpend({
                  provider: 'hyper',
                  label: 'qwen3.8-max',
                  extracted: {
                    text: 'The cat approved this rendering.',
                    usage: {
                      prompt_tokens: 1_000_000,
                      completion_tokens: 1_000_000,
                    },
                  },
                },),
              ],
            },),
          },).priced,
        )
          .toEqual([
            {
              provider: 'hyper',
              model: 'qwen3.8-max',
              calls: 1,
              promptTokens: 1_000_000,
              completionTokens: 1_000_000,
              unreportedCalls: 0,
              costUsd: 0,
              costedCalls: 0,
              reckonedCalls: 0,
              inputCredits: 40,
              outputCredits: 120,
              totalCredits: 160,
            },
          ],);
      },
    },),

    it({
      name: 'KEEPS OPENROUTER SEATS IN THEIR OWN USD BUCKET, summed from what the wire reported and '
        + 'never added to the credit total, since the two are different currencies',
      fn: async () => {
        /**
         Tally holding one seat per currency.
         */
        const cost = priceTally({
          tally: tallyOf({
            tails: [
              'SPEND provider=hyper model=kimi-k3 prompt=0 completion=100000',
              'SPEND provider=openrouter model=moonshotai/kimi-k3 prompt=1000 completion=500 cost=0.0105',
              'SPEND provider=openrouter model=moonshotai/kimi-k3 prompt=1000 completion=500 cost=0.0095',
              'SPEND provider=openrouter model=z-ai/glm-5.3-flash prompt=1000 completion=500',
            ],
          },),
        },);

        expect({
          credits: cost.totalCredits,
          usd: cost.totalUsd,
          openRouter: cost
            .openRouter
            .map(function named(seat,): string {
              return `${seat.model} ${String(seat.costUsd,)} over ${String(seat.costedCalls,)} of ${String(seat.calls,)}`;
            },),
          subscription: cost
            .subscription
            .length,
        },)
          .toEqual({
            credits: 32.664,
            usd: 0.02,
            openRouter: [
              'moonshotai/kimi-k3 0.02 over 2 of 2',
              'z-ai/glm-5.3-flash 0 over 0 of 1',
            ],
            subscription: 0,
          },);
      },
    },),
  ],
},);
