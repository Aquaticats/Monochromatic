/**
 What an OVHcloud size costs per hour, read from the provider's own public
 catalogue.

 The project decided on 2026-11-28 that a create takes the cheapest size that
 fits, so the numbers alone do not decide; the price does
 (`doc/decision/mvm-gains-an-ovhcloud-backend-billed-by-the-second.md`, "The size
 taken among those that fit"). A flavor record carries no price, so the price is
 read where OVHcloud publishes it: the public order catalogue,
 `GET /v1/order/catalog/public/cloud`, whose answer carries one addon per hourly
 plan code and that addon's pricing.

 The shape here is copied from one live answer of that call with
 `ovhSubsidiary=CA`, read on 2026-11-28:

 - the body has `catalogId`, `locale`, `plans`, `products`, `addons` and
   `planFamilies`; the hourly instance plans are in `addons`, not `plans`, whose
   seven entries are the credit and other non-instance plans;
 - `locale` is `{"currencyCode":"CAD","subsidiary":"CA","taxRate":0}`, so the
   currency and tax rate a price is stated in come from the answer itself;
 - an addon's `planCode` is the flavor's own `planCodes.hourly`, such as
   `d2-2.consumption`;
 - its `pricings` holds one entry with `interval` 1, `intervalUnit` `hour`,
   `type` `consumption` and an integer `price`.
 
 The integer counts 1/100000000 of a currency unit: the live `b2-7.consumption`
 carried 11350000 for the `$0.11 CAD` its `formattedPrice` states, and
 `c3-4.consumption` carried 7320000 for `$0.07 CAD`, which is 0.0732. The
 rounded `formattedPrice` is not used for a comparison, only the integer is.

 The same answer also corroborates what a family is: `blobs.commercial.brickSubtype`
 is `gpu` on `t1-45`, `iops` on `i1-45`, `discovery` on `d2-2` and `d2-4`, and
 `general-purpose` on `b2-7`, and `blobs.technical.gpu` is present on `t1-45`
 alone of those. The family rule reads names, not these blobs, so that it holds
 for a region whose answer carries no blobs; the blobs are recorded here as
 what OVHcloud itself calls each family.

 @module
 */

import { ovhRequest, } from './api.ts';

//region Names

/**
 Path of the public catalogue of cloud plans, under the API's versioned root.
 */
export const PRICE_CATALOG_PATH = '/v1/order/catalog/public/cloud';

/**
 Query parameter naming the subsidiary whose currency the prices are stated in.
 */
const SUBSIDIARY_PARAMETER = 'ovhSubsidiary';

/**
 Interval unit of an hourly price.
 */
const HOUR_UNIT = 'hour';

/**
 Interval of one hour.
 */
const ONE_HOUR = 1;

/**
 Pricing kind that meters a running machine, as opposed to a one-off purchase.
 */
const CONSUMPTION_TYPE = 'consumption';

/**
 How many of the catalogue's price units make one currency unit.

 Measured against the live answer of 2026-11-28: `b2-7.consumption` carried
 11350000 while `formattedPrice` stated `$0.11 CAD`, which is 0.1135.
 */
export const PRICE_UNITS_PER_CURRENCY_UNIT = 100_000_000;

/**
 Answer for an addon with no hourly consumption price, so absence travels as a
 distinct value rather than a nullish union.
 */
export const NO_PRICE: unique symbol = Symbol(
  'answers that no hourly charge is published for this plan',
);

/**
 The subsidiary each API host belongs to, which is the catalogue whose currency
 its projects are billed in. The keys are the endpoint roots mvm resolves, the
 value `OVH_ENDPOINT` names or its default (`config.ts`, `ENDPOINT_ROOTS`).
 */
const SUBSIDIARY_BY_ENDPOINT_ROOT: Readonly<Record<string, string>> = Object.freeze({
  'https://ca.api.ovh.com': 'CA',
  'https://eu.api.ovh.com': 'FR',
  'https://api.us.ovhcloud.com': 'US',
});

//endregion Names

//region Shapes

/**
 One pricing entry of a catalogue addon, from the answer's own `pricings`.
 
 @example
 ```ts
 const pricing: OvhCatalogPricing = { interval: 1, intervalUnit: 'hour', price: 33_000_000, type: 'consumption' };
 ```
 */
export type OvhCatalogPricing = {
  /**
   How many `intervalUnit` the price covers.
   */
  readonly interval: number;
  /**
   Unit the interval counts, such as `hour` or `month`.
   */
  readonly intervalUnit: string;
  /**
   Price in units of 1/{@link PRICE_UNITS_PER_CURRENCY_UNIT} of a currency unit.
   */
  readonly price: number;
  /**
   Pricing kind, such as `consumption` or `purchase`.
   */
  readonly type: string;
};

/**
 One addon of the public catalogue, which is where an instance plan's price sits.
 
 @example
 ```ts
 const addon: OvhCatalogAddon = { planCode: 'd2-2.consumption', pricings: [] };
 ```
 */
export type OvhCatalogAddon = {
  /**
   Plan code, which is the flavor's own `planCodes.hourly`.
   */
  readonly planCode: string;
  /**
   Every pricing entry the plan carries, one per interval it is sold by.
   */
  readonly pricings: readonly OvhCatalogPricing[];
};

/**
 The public catalogue of cloud plans, carrying only the fields this read uses.
 
 @example
 ```ts
 const catalog: OvhPublicCatalog = { addons: [], locale: { currencyCode: 'CAD' } };
 ```
 */
export type OvhPublicCatalog = {
  /**
   Every addon the catalogue publishes.
   */
  readonly addons: readonly OvhCatalogAddon[];
  /**
   The currency and subsidiary the prices are stated in.
   */
  readonly locale: {
    readonly currencyCode: string;
  };
};

/**
 What one read of the catalogue gives: a price per hourly plan code, and the
 currency those prices are stated in.
 */
export type PriceIndex = {
  /**
   Currency the prices are stated in, such as `CAD`.
   */
  readonly currency: string;
  /**
   Hourly price per plan code; a code the catalogue does not price is absent.
   */
  readonly prices: ReadonlyMap<string, number>;
};

//endregion Shapes

//region Reading a price

/**
 Reads one addon's hourly price, or {@link NO_PRICE} when it publishes none.
 
 @param addon - catalogue addon, whose `planCode` is a flavor's hourly plan code
 
 @returns the price per hour in currency units, or {@link NO_PRICE}
 
 @example
 ```ts
 hourlyPriceOf({ addon }); // 0.1135 for a b2-7 addon carrying 11350000
 ```
 */
export function hourlyPriceOf({
  addon,
}: {
  readonly addon: OvhCatalogAddon;
},): number | typeof NO_PRICE {
  /**
   The entry that meters one hour of consumption, when the addon carries one.
   */
  const hourly = addon.pricings
    .find(function isHourly(pricing,) {
      return (pricing.intervalUnit === HOUR_UNIT)
        && (pricing.interval === ONE_HOUR)
        && (pricing.type === CONSUMPTION_TYPE);
    },);
  if (hourly === undefined) {
    return NO_PRICE;
  }
  return hourly.price / PRICE_UNITS_PER_CURRENCY_UNIT;
}

/**
 Builds the price index of a catalogue answer.
 
 @param catalog - the catalogue as the API answered it
 
 @returns every hourly plan code the catalogue prices, and the currency
 
 @example
 ```ts
 priceIndexOf({ catalog }).prices.get('d2-2.consumption'); // 0.0167
 ```
 */
export function priceIndexOf({
  catalog,
}: {
  readonly catalog: OvhPublicCatalog;
},): PriceIndex {
  /**
   Prices found so far, by plan code.
   */
  const prices = new Map<string, number>();
  for (const addon of catalog.addons) {
    /**
     Hourly price of this addon, or {@link NO_PRICE} when it has none.
     */
    const price = hourlyPriceOf({ addon, },);
    if (price !== NO_PRICE) {
      prices.set(
        addon.planCode,
        price,
      );
    }
  }
  return {
    currency: catalog.locale
      .currencyCode,
    prices,
  };
}

/**
 Names the currency a catalogue priced in.
 
 @param catalog - the catalogue as the API answered it
 
 @returns the currency code, such as `CAD`
 
 @example
 ```ts
 currencyOf({ catalog }); // 'CAD'
 ```
 */
export function currencyOf({
  catalog,
}: {
  readonly catalog: OvhPublicCatalog;
},): string {
  return catalog.locale
    .currencyCode;
}

/**
 Names the subsidiary an API host belongs to, whose catalogue prices in the
 currency that host's projects are billed in.
 
 @param endpointRoot - the API root the credentials carry, such as `https://ca.api.ovh.com`
 
 @returns the subsidiary code the catalogue is asked for
 
 @throws Error when the root is not one this build knows
 
 @example
 ```ts
 subsidiaryOf({ endpointRoot: 'https://ca.api.ovh.com' }); // 'CA'
 ```
 */
export function subsidiaryOf({
  endpointRoot,
}: {
  readonly endpointRoot: string;
},): string {
  /**
   Subsidiary the host belongs to, when this build knows the host.
   */
  const subsidiary = SUBSIDIARY_BY_ENDPOINT_ROOT[endpointRoot];
  if (subsidiary === undefined) {
    throw new Error(
      `no price catalogue is known for the API host ${endpointRoot}; known hosts: ${
        Object.keys(SUBSIDIARY_BY_ENDPOINT_ROOT,)
          .join(', ',)
      }`,
    );
  }
  return subsidiary;
}

//endregion Reading a price

//region Reading the catalogue

/**
 Reads the public catalogue once and builds the price index from it.
 
 @param subsidiary - subsidiary whose currency the caller wants, from {@link subsidiaryOf}
 
 @returns every hourly plan code the catalogue prices, and the currency
 
 @throws Error when the API refuses or cannot be reached, which the caller
 treats as "the price read cannot be had"
 
 @example
 ```ts
 const index = await readPriceIndex({ subsidiary: 'CA' });
 ```
 */
export async function readPriceIndex({
  subsidiary,
}: {
  readonly subsidiary: string;
},): Promise<PriceIndex> {
  return priceIndexOf({
    catalog: await ovhRequest<OvhPublicCatalog>({
      method: 'GET',
      path: `${PRICE_CATALOG_PATH}?${SUBSIDIARY_PARAMETER}=${encodeURIComponent(subsidiary,)}`,
    },),
  },);
}

//endregion Reading the catalogue
