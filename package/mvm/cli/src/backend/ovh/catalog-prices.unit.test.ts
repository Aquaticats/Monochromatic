/**
 Unit tests for the hourly price read: which pricing entry names an hour, how the
 catalogue's own integer becomes a currency amount, what an addon without an
 hourly price answers, and what an hour of an instance size sums.
 
 The stand-in catalogue is copied from one live answer of
 `GET /v1/order/catalog/public/cloud?ovhSubsidiary=CA`, read on 2026-11-28 (the
 round's `ovh-public-cloud-catalog-CA-2026-11-28.json`): the plan codes
 `d2-2.consumption`, `d2-4.consumption`, `c3-4.consumption` and
 `b2-7.consumption`, with the `price` values 1670000, 3300000, 7320000 and
 11350000 and `formattedPrice` `$0.02 CAD`, `$0.03 CAD`, `$0.07 CAD` and
 `$0.11 CAD`; the address line `publicip.ip.hour.consumption` at 432000 with
 `interval` 0, `intervalUnit` `none` and `blobs.commercial.price.interval`
 `PT1H`; the storage lines `instance.local-storage-gen3-gb.hour.consumption` at
 23330 per GB-hour and `instance.local-storage-free-gb.hour.consumption` at 0;
 `locale` is `{"currencyCode":"CAD","subsidiary":"CA","taxRate":0}`.
 The monthly-only and missing addons in the fixtures are shaped like that answer's
 own `month` entries, and say so where they appear. One case uses the US
 answer's shape of the address line instead, which carries no commercial blob
 and states a price of 0, and says so where it appears. No claim here is about a
 price the read produces in a live region: the sums are stand-ins made from those
 answers' own numbers, and the page-built figures beside them are arithmetic on
 the same numbers.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  ADDRESS_PLAN_CODE,
  addressPriceOf,
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
  product = 'publiccloud-instance',
}: {
  readonly interval?: number;
  readonly intervalUnit?: string;
  readonly planCode: string;
  readonly price: number;
  readonly product?: string;
},) {
  return {
    planCode,
    pricings: [{
      interval,
      intervalUnit,
      price,
      type: 'consumption',
    },],
    product,
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

/**
 The address line as the live answer states it: one metered entry whose
 `intervalUnit` is `none`, and its hour in the commercial blob.
 */
const ADDRESS = {
  blobs: { commercial: { price: { interval: 'PT1H', }, }, },
  planCode: ADDRESS_PLAN_CODE,
  pricings: [{ interval: 0, intervalUnit: 'none', price: 432_000, type: 'consumption', },],
  product: 'publiccloud-publicip-ip',
} as const;

/**
 The Gen 3 storage line as the live answer states it: one price per GB-hour,
 whose GB a plan states separately.
 */
const GEN3_STORAGE = addonAnswer({
  planCode: 'instance.local-storage-gen3-gb.hour.consumption',
  price: 23_330,
  product: 'publiccloud-local-storage-gen3-gb',
},);

/**
 The storage line the earlier ranges name, which the live answer prices at zero
 because their local storage is included in the row.
 */
const FREE_STORAGE = addonAnswer({
  planCode: 'instance.local-storage-free-gb.hour.consumption',
  price: 0,
  product: 'publiccloud-local-storage-free-gb',
},);

/**
 One instance plan as the live answer states it, with the fields this read uses:
 its own row, the family that names the addon billing its storage, and the disks
 its technical blob lists.
 */
function planAnswer({
  addonFamilies,
  disks,
  planCode,
  price,
}: {
  readonly addonFamilies: readonly { readonly name: string; readonly addons: readonly string[]; }[];
  readonly disks?: readonly { readonly number?: number; readonly capacity: number; }[];
  readonly planCode: string;
  readonly price: number;
},) {
  return {
    addonFamilies,
    ...((disks === undefined)
      ? {}
      : { blobs: { technical: { storage: { disks, }, }, }, }),
    planCode,
    pricings: [{ interval: 1, intervalUnit: 'hour', price, type: 'consumption', },],
    product: 'publiccloud-instance',
  };
}

/**
 A catalogue of four sizes, one from each of four families, exactly as the live
 answer states their rows, families and disks: a Gen 3 size at 100 GB of local
 storage, a Gen 3 size at 50 GB, a memory-optimized Gen 3 size at 50 GB, and two
 earlier ranges whose storage addon is the zero-priced one.
 */
const WHOLE_HOUR = {
  addons: [
    ADDRESS,
    GEN3_STORAGE,
    FREE_STORAGE,
    planAnswer({
      addonFamilies: [
        { addons: ['instance.windows-vcore-license.hour.consumption',], name: 'license', },
        { addons: ['instance.local-storage-gen3-gb.hour.consumption',], name: 'local-storage', },
      ],
      disks: [{ capacity: 100, number: 1, },],
      planCode: 'b3-16.consumption',
      price: 16_370_000,
    },),
    planAnswer({
      addonFamilies: [
        { addons: ['instance.local-storage-gen3-gb.hour.consumption',], name: 'local-storage', },
      ],
      disks: [{ capacity: 50, number: 1, },],
      planCode: 'c3-4.consumption',
      price: 7_320_000,
    },),
    planAnswer({
      addonFamilies: [
        { addons: ['instance.local-storage-gen3-gb.hour.consumption',], name: 'local-storage', },
      ],
      disks: [{ capacity: 50, number: 1, },],
      planCode: 'r3-16.consumption',
      price: 10_610_000,
    },),
    // The live `b2-7` disk states no `number`: one disk of that kind.
    planAnswer({
      addonFamilies: [
        { addons: ['instance.local-storage-free-gb.hour.consumption',], name: 'local-storage', },
      ],
      disks: [{ capacity: 50, },],
      planCode: 'b2-7.consumption',
      price: 11_350_000,
    },),
    planAnswer({
      addonFamilies: [
        { addons: ['instance.local-storage-free-gb.hour.consumption',], name: 'local-storage', },
      ],
      disks: [{ capacity: 25, number: 1, },],
      planCode: 'd2-2.consumption',
      price: 1_670_000,
    },),
    // A product the catalogue also sells by the hour but not as a machine: an
    // object storage GB-hour, priced like the live `storage-standard` addon.
    addonAnswer({
      planCode: 'storage-standard.consumption',
      price: 1_400,
      product: 'publiccloud-storage',
    },),
    // Shaped like the live answer's bare-metal plans, whose product is the
    // instance product but which name no local storage family at all.
    addonAnswer({
      planCode: 'metal.eg-32.consumption',
      price: 65_000_000,
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
            expect(
              hourlyPriceOf({ addon: { planCode: 'nothing', pricings: [], product: 'publiccloud-instance', }, },),
            ).toBe(NO_PRICE,);
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
                  product: 'publiccloud-instance',
                },
              },),
            ).toBe(NO_PRICE,);
          },
        },),
      ],
    },),

    //endregion Reading one addon

    //region Reading the address line

    describe({
      name: addressPriceOf.name,
      children: [
        it({
          name: 'reads the one metered price the answer publishes for the address line',
          fn: async () => {
            expect(addressPriceOf({ catalog: WHOLE_HOUR, },),).toBeCloseTo(0.00432,);
          },
        },),
        it({
          name: 'reads the US answer\'s shaped line, which carries no commercial blob, at the zero it states',
          fn: async () => {
            // Shaped like the live US `publicip.ip.hour.consumption` of
            // 2026-11-28: one metered entry, no blobs, price 0.
            expect(
              addressPriceOf({
                catalog: {
                  addons: [{
                    planCode: ADDRESS_PLAN_CODE,
                    pricings: [{ interval: 0, intervalUnit: 'none', price: 0, type: 'consumption', },],
                    product: 'publiccloud-publicip-ip',
                  },],
                  locale: { currencyCode: 'USD', },
                },
              },),
            ).toBe(0,);
          },
        },),
        it({
          name: 'answers NO_PRICE for a line publishing other than one metered price, and for no line at all',
          fn: async () => {
            expect(
              addressPriceOf({
                catalog: {
                  addons: [{
                    planCode: ADDRESS_PLAN_CODE,
                    pricings: [
                      { interval: 0, intervalUnit: 'none', price: 500_000, type: 'consumption', },
                      { interval: 1, intervalUnit: 'hour', price: 700_000, type: 'consumption', },
                    ],
                    product: 'publiccloud-publicip-ip',
                  },],
                  locale: { currencyCode: 'CAD', },
                },
              },),
            ).toBe(NO_PRICE,);
            expect(addressPriceOf({ catalog: CATALOG, },),).toBe(NO_PRICE,);
          },
        },),
      ],
    },),

    //endregion Reading the address line

    //region The whole hour

    describe({
      name: priceIndexOf.name,
      children: [
        it({
          name: 'maps an instance plan code to its whole hour: the row, the address line and its own GB of storage',
          fn: async () => {
            const index = priceIndexOf({ catalog: WHOLE_HOUR, },);
            // 16370000 + 432000 + 23330 for the 100 GB of a Gen 3 size.
            expect(index.prices.get('b3-16.consumption',),).toBeCloseTo(0.19135,);
            // 7320000 + 432000 + 23330 for 50 GB.
            expect(index.prices.get('c3-4.consumption',),).toBeCloseTo(0.089185,);
            // 10610000 + 432000 + 23330 for 50 GB.
            expect(index.prices.get('r3-16.consumption',),).toBeCloseTo(0.122085,);
            // 11350000 + 432000, its storage line priced at zero.
            expect(index.prices.get('b2-7.consumption',),).toBeCloseTo(0.11782,);
            // 1670000 + 432000.
            expect(index.prices.get('d2-2.consumption',),).toBeCloseTo(0.02102,);
            expect(index.currency,).toBe('CAD',);
          },
        },),
        it({
          name: 'prices no plan whose product is not an instance, and none that names no local storage family',
          fn: async () => {
            const index = priceIndexOf({ catalog: WHOLE_HOUR, },);
            expect(index.prices.has('storage-standard.consumption',),).toBe(false,);
            expect(index.prices.has('metal.eg-32.consumption',),).toBe(false,);
          },
        },),
        it({
          name: 'gives up a Gen 3 size whose storage line the answer does not price, and never prices it as if free',
          fn: async () => {
            const index = priceIndexOf({
              catalog: {
                addons: WHOLE_HOUR.addons.filter(function notTheGen3Line(addon,) {
                  return addon.planCode !== 'instance.local-storage-gen3-gb.hour.consumption';
                },),
                locale: WHOLE_HOUR.locale,
              },
            },);
            expect(index.prices.has('b3-16.consumption',),).toBe(false,);
            expect(index.prices.has('c3-4.consumption',),).toBe(false,);
            expect(index.prices.get('b2-7.consumption',),).toBeCloseTo(0.11782,);
          },
        },),
        it({
          name: 'gives up a size whose body states no storage amount, and one whose family holds two lines',
          fn: async () => {
            const index = priceIndexOf({
              catalog: {
                addons: [
                  ADDRESS,
                  GEN3_STORAGE,
                  planAnswer({
                    addonFamilies: [
                      { addons: ['instance.local-storage-gen3-gb.hour.consumption',], name: 'local-storage', },
                    ],
                    planCode: 'b3-16.consumption',
                    price: 16_370_000,
                  },),
                  planAnswer({
                    addonFamilies: [
                      {
                        addons: [
                          'instance.local-storage-gen3-gb.hour.consumption',
                          'instance.local-storage-free-gb.hour.consumption',
                        ],
                        name: 'local-storage',
                      },
                    ],
                    disks: [{ capacity: 50, number: 1, },],
                    planCode: 'c3-4.consumption',
                    price: 7_320_000,
                  },),
                ],
                locale: WHOLE_HOUR.locale,
              },
            },);
            expect(index.prices.has('b3-16.consumption',),).toBe(false,);
            expect(index.prices.has('c3-4.consumption',),).toBe(false,);
          },
        },),
        it({
          name: 'leaves the whole read unpriced when the answer states no address price',
          fn: async () => {
            let caught: unknown;
            try {
              priceIndexOf({ catalog: CATALOG, },);
            }
            catch (error: unknown) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(Error,);
            expect((caught as Error).message,).toContain(ADDRESS_PLAN_CODE,);
            expect((caught as Error).message,).toContain('no machine\'s whole hour',);
          },
        },),
      ],
    },),

    //endregion The whole hour

    //region Reading the whole catalogue

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
