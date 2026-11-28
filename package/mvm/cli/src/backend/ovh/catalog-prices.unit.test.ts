/**
 Unit tests for the hourly price read: which pricing entry names an hour, how the
 catalogue's own integer becomes a currency amount, and what an addon without an
 hourly price answers.

 The stand-in catalogue is copied from one live answer of
 `GET /v1/order/catalog/public/cloud?ovhSubsidiary=CA`, read on 2026-11-28
 (scratch `price-catalog-probe-3.log` of that round): the plan codes
 `d2-2.consumption`, `d2-4.consumption`, `c3-4.consumption` and
 `b2-7.consumption`, with the `price` values 1670000, 3300000, 7320000 and
 11350000 and `formattedPrice` `$0.02 CAD`, `$0.03 CAD`, `$0.07 CAD` and
 `$0.11 CAD`; `locale` is `{"currencyCode":"CAD","subsidiary":"CA","taxRate":0}`.
 The monthly-only and missing addons in the fixtures are shaped like that answer's
 own `month` entries, and say so where they appear.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  currencyOf,
  hourlyPriceOf,
  NO_PRICE,
  PRICE_UNITS_PER_CURRENCY_UNIT,
  priceIndexOf,
  subsidiaryOf,
} from '../../../dist/final/node/index.mjs';

//region Fixtures

/**
 One addon as the live answer carried it, with only the fields this read uses.
 */
function addonAnswer({
  interval = 1,
  intervalUnit = 'hour',
  planCode,
  price,
}: {
  readonly interval?: number;
  readonly intervalUnit?: string;
  readonly planCode: string;
  readonly price: number;
},) {
  return {
    planCode,
    pricings: [{
      interval,
      intervalUnit,
      price,
      type: 'consumption',
    },],
  };
}

/**
 Hourly addons exactly as the live answer priced them, in Canadian dollars.
 */
const HOURLY = [
  addonAnswer({
    planCode: 'd2-2.consumption',
    price: 1_670_000,
  },),
  addonAnswer({
    planCode: 'd2-4.consumption',
    price: 3_300_000,
  },),
  addonAnswer({
    planCode: 'c3-4.consumption',
    price: 7_320_000,
  },),
  addonAnswer({
    planCode: 'b2-7.consumption',
    price: 11_350_000,
  },),
] as const;

/**
 A catalogue body: the live hourly addons, one addon priced by the month only,
 and the locale the live answer carried.
 */
const CATALOG = {
  addons: [
    ...HOURLY,
    // Shaped like the live answer's own `month`/1/consumption entries, which
    // state no hourly price at all.
    addonAnswer({
      intervalUnit: 'month',
      planCode: 'b3-8.monthly.postpaid',
      price: 8_500_000,
    },),
  ],
  locale: {
    currencyCode: 'CAD',
    subsidiary: 'CA',
    taxRate: 0,
  },
} as const;

//endregion Fixtures

await describe({
  name: '',
  concurrency: 1,
  children: [
    //region The unit the catalogue counts in

    describe({
      name: 'PRICE_UNITS_PER_CURRENCY_UNIT',
      children: [
        it({
          name: 'divides the live b2-7 price to the amount that answer formatted',
          fn: async () => {
            expect(11_350_000 / PRICE_UNITS_PER_CURRENCY_UNIT,).toBeCloseTo(0.1135,);
          },
        },),
      ],
    },),

    //endregion The unit the catalogue counts in

    //region Reading one addon

    describe({
      name: hourlyPriceOf.name,
      children: [
        it({
          name: 'reads the hourly consumption price of a live addon',
          fn: async () => {
            expect(hourlyPriceOf({ addon: HOURLY[0], },),).toBeCloseTo(0.0167,);
            expect(hourlyPriceOf({ addon: HOURLY[3], },),).toBeCloseTo(0.1135,);
          },
        },),
        it({
          name: 'answers NO_PRICE for an addon priced by the month, and for one with no pricing at all',
          fn: async () => {
            expect(
              hourlyPriceOf({
                addon: addonAnswer({
                  intervalUnit: 'month',
                  planCode: 'b3-8.monthly.postpaid',
                  price: 8_500_000,
                },),
              },),
            ).toBe(NO_PRICE,);
            expect(hourlyPriceOf({ addon: { planCode: 'nothing', pricings: [], }, },),).toBe(NO_PRICE,);
          },
        },),
        it({
          name: 'does not take an hour-shaped entry that is a purchase rather than consumption',
          fn: async () => {
            expect(
              hourlyPriceOf({
                addon: {
                  planCode: 'odd',
                  pricings: [{
                    interval: 1,
                    intervalUnit: 'hour',
                    price: 1_000_000,
                    type: 'purchase',
                  },],
                },
              },),
            ).toBe(NO_PRICE,);
          },
        },),
      ],
    },),

    //endregion Reading one addon

    //region Reading the whole catalogue

    describe({
      name: priceIndexOf.name,
      children: [
        it({
          name: 'maps each hourly plan code to its price and leaves the monthly one out',
          fn: async () => {
            const index = priceIndexOf({ catalog: CATALOG, },);
            expect(index.prices.get('d2-2.consumption',),).toBeCloseTo(0.0167,);
            expect(index.prices.get('c3-4.consumption',),).toBeCloseTo(0.0732,);
            expect(index.prices.size,).toBe(HOURLY.length,);
            expect(index.currency,).toBe('CAD',);
          },
        },),
      ],
    },),

    describe({
      name: currencyOf.name,
      children: [
        it({
          name: 'names the currency the live answer priced in',
          fn: async () => {
            expect(currencyOf({ catalog: CATALOG, },),).toBe('CAD',);
          },
        },),
      ],
    },),

    //endregion Reading the whole catalogue

    //region Which catalogue the price is asked of

    describe({
      name: subsidiaryOf.name,
      children: [
        it({
          name: 'names the subsidiary each API host belongs to, and refuses a host it does not know',
          fn: async () => {
            expect(subsidiaryOf({ endpointRoot: 'https://ca.api.ovh.com', },),).toBe('CA',);
            expect(subsidiaryOf({ endpointRoot: 'https://eu.api.ovh.com', },),).toBe('FR',);
            expect(subsidiaryOf({ endpointRoot: 'https://api.us.ovhcloud.com', },),).toBe('US',);
            expect(() => subsidiaryOf({ endpointRoot: 'https://elsewhere.invalid', },),)
              .toThrow('known hosts',);
          },
        },),
      ],
    },),

    //endregion Which catalogue the price is asked of
  ],
},);
