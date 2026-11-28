/**
 What an hour of an OVHcloud size costs, read from the provider's own public
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

 The instance row is not the whole hour. OVHcloud bills a machine's hour in three
 lines, and the other two are separate products with separate plan codes in the
 same answer (read from the live CA, FR and US answers of 2026-11-28; issue
 6666):

 - the public IPv4 address every machine mvm makes reaches the network on. The
   CA answer's `publicip.ip.hour.consumption` is `invoiceName` `Public IPv4`,
   `product` `publiccloud-publicip-ip`, and its one `consumption` pricing states
   `interval` 0, `intervalUnit` `none`, `description` `price for one hour` and
   `price` 432000, which is CA$0.00432 per hour. That entry names no interval
   unit, so the hour is read from the code the answer publishes the line under,
   whose own `.hour.` token states it, and from the description beside it: the
   CA and FR answers also state `blobs.commercial.price.interval` `PT1H`, and
   the US answer carries no such blob while stating the same description. The US
   base line is priced 0 there, which that subsidiary's own answer states and
   not a missing price;
 - local storage, which only the Gen 3 families bill apart from the instance.
   An instance plan names the addon that bills its own storage in its own
   `addonFamilies`, under the family name `local-storage`: `b3-8.consumption`
   names `instance.local-storage-gen3-gb.hour.consumption` and `b2-7.consumption`
   names the zero-priced `instance.local-storage-free-gb.hour.consumption`. The
   Gen 3 addon states 23330 per `hour`, which is CA$0.0002333 per GB-hour, and
   the price page's packaged 50, 100, 200 and 400 GB rows are that rate times
   the GB. The GB a size carries is in the same instance addon, under
   `blobs.technical.storage.disks`: `b3-8` states `{"number":1,"capacity":50}`
   and `b2-7` states `{"capacity":50}`.

 So a size's whole hour is its row, plus the address line, plus its own GB at the
 rate its own family names. Every one of those comes from the one answer, so a
 create still makes one extra call.

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
 The catalogue's `product` field on an instance plan, which is the only kind of
 addon whose hour mvm prices. Other products are billed by other lines: object
 storage per GB-month, file storage, bandwidth, and the managed database plans.

 Read from the live CA answer of 2026-11-28, where 318 addons carry it and no
 addon carries an empty one.
 */
const INSTANCE_PRODUCT = 'publiccloud-instance';

/**
 Plan code of the public IPv4 address line, in the catalogue's own spelling.

 The price page names the same product `Public IP IPv4 (/32)`; the catalogue
 states it as `publicip.ip` sold by the hour and consumed. The answer also
 carries local-zone variants of the line (`publicip.ip.hour.consumption.LZ`,
 `.LZ.EU`, `.LZ.AF` in CA and FR, `.US-EAST-LZ-DAL-A` in the US), which carry
 the same price as the base code in CA and FR on 2026-11-28 and a different one
 in the US. mvm creates in a standard region, so this base code is the line its
 machines carry.
 */
export const ADDRESS_PLAN_CODE = 'publicip.ip.hour.consumption';

/**
 Name of the addon family under which an instance plan names the addon that
 bills its local storage.
 */
const LOCAL_STORAGE_FAMILY = 'local-storage';

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
 Answer for an instance plan whose body states no local storage amount, so
 absence travels as a distinct value rather than a nullish union.
 */
const NO_STORAGE: unique symbol = Symbol(
  'answers that no storage amount is stated for this plan',
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
 One disk an instance plan's technical blob states.
 
 @example
 ```ts
 const disk: OvhCatalogDisk = { number: 1, capacity: 50, technology: 'NVMe' };
 ```
 */
export type OvhCatalogDisk = {
  /**
   How many disks of this kind the plan carries, absent where it carries one.
   */
  readonly number?: number;
  /**
   Capacity of one such disk, in GB.
   */
  readonly capacity: number;
};

/**
 One addon family an instance plan lists beside the plan itself, such as the
 addon that bills its local storage.
 
 @example
 ```ts
 const family: OvhCatalogAddonFamily = { name: 'local-storage', addons: ['instance.local-storage-gen3-gb.hour.consumption'] };
 ```
 */
export type OvhCatalogAddonFamily = {
  /**
   Family name, such as `local-storage` or `license`.
   */
  readonly name: string;
  /**
   Plan codes of the addons this family holds.
   */
  readonly addons: readonly string[];
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
   Plan code, which is the flavor's own `planCodes.hourly` for an instance plan.
   */
  readonly planCode: string;
  /**
   Product this addon belongs to, such as `publiccloud-instance`.
   */
  readonly product: string;
  /**
   Addon families this plan lists, absent where it lists none.
   */
  readonly addonFamilies?: readonly OvhCatalogAddonFamily[];
  /**
   The plan's own descriptive blobs, absent on some addons. The technical one
   states the GB of local storage an instance plan carries, and the commercial
   one states the interval a price is stated over.
   */
  readonly blobs?: {
    readonly commercial?: {
      readonly price?: {
        readonly interval?: string;
      };
    };
    readonly technical?: {
      readonly storage?: {
        readonly disks?: readonly OvhCatalogDisk[];
      };
    };
  };
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
 What one read of the catalogue gives: an hour of each instance size, and the
 currency those prices are stated in.
 */
export type PriceIndex = {
  /**
   Currency the prices are stated in, such as `CAD`.
   */
  readonly currency: string;
  /**
   Whole-hour price per instance plan code, counting every line the machine is
   billed for; a plan the answer does not price in full is absent.
   */
  readonly prices: ReadonlyMap<string, number>;
};

//endregion Shapes

//region Reading a price

/**
 Reads one addon's hourly price in the catalogue's own integer units, or
 {@link NO_PRICE} when it publishes none.
 
 @param addon - catalogue addon, whose `planCode` is a flavor's hourly plan code
 
 @returns the price per hour in units of 1/{@link PRICE_UNITS_PER_CURRENCY_UNIT}, or {@link NO_PRICE}
 
 @example
 ```ts
 hourlyPriceUnitsOf({ addon }); // 11350000 for a b2-7 addon
 ```
 */
function hourlyPriceUnitsOf({
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
  return hourly.price;
}

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
   Hourly price of this addon in the catalogue's own units.
   */
  const units = hourlyPriceUnitsOf({ addon, },);
  return (units === NO_PRICE)
    ? NO_PRICE
    : units / PRICE_UNITS_PER_CURRENCY_UNIT;
}

/**
 Reads the hourly price of the public IPv4 address line, or {@link NO_PRICE} when
 the answer states none.
 
 Every machine mvm makes reaches the network on a public IPv4 address, and
 OVHcloud bills that address as its own product, so its line belongs in every
 size's hour. The line's pricing entry states `interval` 0 with `intervalUnit`
 `none`, so the hour is not in that entry: the code the answer publishes the
 line under carries an `.hour.` token, the entry's own description reads `price
 for one hour`, and the CA and FR answers add
 `blobs.commercial.price.interval` `PT1H` (the US answer carries no such blob).
 A price of 0 is the answer's own (the US subsidiary charges nothing for the
 address) rather than a missing one, and a line publishing no single metered
 price is no price at all rather than a guessed one.
 
 @param catalog - the catalogue as the API answered it
 
 @returns the address's price per hour in units of 1/{@link PRICE_UNITS_PER_CURRENCY_UNIT}, or {@link NO_PRICE}
 
 @example
 ```ts
 addressPriceUnitsOf({ catalog }); // 432000 for the live CA answer
 ```
 */
function addressPriceUnitsOf({
  catalog,
}: {
  readonly catalog: OvhPublicCatalog;
},): number | typeof NO_PRICE {
  /**
   The address line's own addon, when the answer publishes one.
   */
  const addon = catalog.addons
    .find(function isAddressLine(entry,) {
      return entry.planCode === ADDRESS_PLAN_CODE;
    },);
  if (addon === undefined) {
    return NO_PRICE;
  }
  /**
   Every metered entry the line carries; exactly one is the price of an hour.
   */
  const metered = addon.pricings
    .filter(function isConsumption(pricing,) {
      return pricing.type === CONSUMPTION_TYPE;
    },);
  if (metered.length !== 1) {
    return NO_PRICE;
  }
  /**
   The line's one metered entry.
   */
  const [entry,] = metered;
  if (entry === undefined) {
    return NO_PRICE;
  }
  return entry.price;
}

/**
 Reads the hourly price of the public IPv4 address line, or {@link NO_PRICE} when
 the answer states none.
 
 @param catalog - the catalogue as the API answered it
 
 @returns the address's price per hour in currency units, or {@link NO_PRICE}
 
 @example
 ```ts
 addressPriceOf({ catalog }); // 0.00432 for the live CA answer
 ```
 */
export function addressPriceOf({
  catalog,
}: {
  readonly catalog: OvhPublicCatalog;
},): number | typeof NO_PRICE {
  /**
   The address's hour in the catalogue's own units.
   */
  const units = addressPriceUnitsOf({ catalog, },);
  return (units === NO_PRICE)
    ? NO_PRICE
    : units / PRICE_UNITS_PER_CURRENCY_UNIT;
}

/**
 Reads how many GB of local storage an instance plan carries, or
 {@link NO_STORAGE} when its body states none.
 
 The plan's technical blob lists its disks: `b3-8` states one disk of 50 GB and
 `b2-7` states one disk of 50 GB with no `number`, which is one disk of that
 kind. A plan whose body lists no disk has no storage amount this read knows.
 
 @param addon - an instance plan's own addon
 
 @returns the GB the plan carries, or {@link NO_STORAGE}
 
 @example
 ```ts
 localStorageGbOf({ addon }); // 50 for b3-8 and for b2-7
 ```
 */
function localStorageGbOf({
  addon,
}: {
  readonly addon: OvhCatalogAddon;
},): number | typeof NO_STORAGE {
  /**
   Disks the plan's technical blob lists, when it lists any.
   */
  const disks = addon.blobs
    ?.technical
    ?.storage
    ?.disks;
  if ((disks === undefined) || (disks.length === 0)) {
    return NO_STORAGE;
  }
  return disks.reduce(
    function addDisk(
      total,
      disk,
    ) {
      return total + ((disk.number ?? 1) * disk.capacity);
    },
    0,
  );
}

/**
 Reads the hourly charge for an instance plan's local storage, or {@link NO_PRICE}
 when the answer does not state it.
 
 The catalogue's own `local-storage` family names the addon that bills the
 plan's storage, so no storage plan code is written here: the Gen 3 plans name
 the per-GB addon and the earlier ranges name the zero-priced one, which makes
 both cases one rule. A plan naming a family this answer does not price, a
 family holding other than one addon, or a plan with no storage amount has no
 storage charge this read knows.
 
 @param addon - an instance plan's own addon
 
 @param addonByPlanCode - every addon of the answer, by plan code
 
 @returns the storage's price per hour in units of 1/{@link PRICE_UNITS_PER_CURRENCY_UNIT}, or {@link NO_PRICE}
 
 @example
 ```ts
 localStoragePriceUnitsOf({ addon: b3_8, addonByPlanCode }); // 1166500 for its 50 GB
 ```
 */
function localStoragePriceUnitsOf({
  addon,
  addonByPlanCode,
}: {
  readonly addon: OvhCatalogAddon;
  readonly addonByPlanCode: ReadonlyMap<string, OvhCatalogAddon>;
},): number | typeof NO_PRICE {
  /**
   The family under which this plan names its storage addon, when it names one.
   */
  const family = addon.addonFamilies
    ?.find(function isLocalStorage(entry,) {
      return entry.name === LOCAL_STORAGE_FAMILY;
    },);
  if (family === undefined) {
    return NO_PRICE;
  }
  /**
   GB of local storage the plan's own body states.
   */
  const gigabytes = localStorageGbOf({ addon, },);
  if (gigabytes === NO_STORAGE) {
    return NO_PRICE;
  }
  /**
   The one addon this family holds, when it holds exactly one, which is the line
   the storage is billed by.
   */
  const storageLines = family.addons;
  if (storageLines.length !== 1) {
    return NO_PRICE;
  }
  /**
   The plan code of that one addon.
   */
  const [planCode,] = storageLines;
  if (planCode === undefined) {
    return NO_PRICE;
  }
  /**
   The addon that bills this plan's storage, when the answer prices it.
   */
  const storage = addonByPlanCode.get(planCode,);
  if (storage === undefined) {
    return NO_PRICE;
  }
  /**
   Per-GB hourly price of the storage line, in the catalogue's own units.
   */
  const perGigabyte = hourlyPriceUnitsOf({ addon: storage, },);
  if (perGigabyte === NO_PRICE) {
    return NO_PRICE;
  }
  return perGigabyte * gigabytes;
}

/**
 Builds the whole-hour price index of a catalogue answer.
 
 A size's hour is its instance row, plus the public IPv4 address line, plus its
 own GB of local storage at the rate its own family names. The lines are added in
 the catalogue's integer units and divided once, so the sum carries no rounding
 of its own. A size whose storage line or storage amount this answer does not
 state is left out, the way a size with no published price is: it is never a
 candidate, rather than being priced as if the line were free. When the answer
 states no price for the address, no machine's hour is known, which is the price
 read the create cannot have.
 
 @param catalog - the catalogue as the API answered it
 
 @returns every instance plan code the catalogue prices in full, and the currency
 
 @throws Error when the answer states no hourly price for the address line
 
 @example
 ```ts
 priceIndexOf({ catalog }).prices.get('d2-2.consumption'); // 0.02102
 ```
 */
export function priceIndexOf({
  catalog,
}: {
  readonly catalog: OvhPublicCatalog;
},): PriceIndex {
  /**
   Hourly price of the address line, which every machine mvm makes carries.
   */
  const addressPrice = addressPriceUnitsOf({ catalog, },);
  if (addressPrice === NO_PRICE) {
    throw new Error(
      `the public order catalogue publishes no hourly price for the public IPv4 address line (${ADDRESS_PLAN_CODE}), so no machine's whole hour can be priced`,
    );
  }
  /**
   Every addon of the answer, by plan code, so a plan's storage line is one lookup.
   */
  const addonByPlanCode = new Map(catalog.addons
    .map(function entryByCode(addon,) {
      return [
        addon.planCode,
        addon,
      ] as const;
    },),);
  /**
   Prices found so far, by instance plan code.
   */
  const prices = new Map<string, number>();
  for (const addon of catalog.addons) {
    if (addon.product !== INSTANCE_PRODUCT) {
      continue;
    }
    /**
     Hourly price of this plan's own instance row, in the catalogue's units.
     */
    const row = hourlyPriceUnitsOf({ addon, },);
    if (row === NO_PRICE) {
      continue;
    }
    /**
     Hourly charge for this plan's own local storage.
     */
    const storage = localStoragePriceUnitsOf({
      addon,
      addonByPlanCode,
    },);
    if (storage === NO_PRICE) {
      continue;
    }
    /**
     The hour's three lines, added in the catalogue's own units before one
     division so the sum carries no rounding of its own.
     */
    const wholeHourUnits = row + addressPrice
      + storage;
    prices.set(
      addon.planCode,
      wholeHourUnits / PRICE_UNITS_PER_CURRENCY_UNIT,
    );
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
 
 @returns every instance plan code the catalogue prices in full, and the currency
 
 @throws Error when the API refuses or cannot be reached, or when the answer
 states no address price, which the caller treats as "the price read cannot be
 had"
 
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
