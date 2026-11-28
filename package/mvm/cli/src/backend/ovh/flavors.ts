/**
 Which flavor (plan) an instance is created with in one region.

 OVHcloud's flavor identifiers churn the way any provider's plans do, so no
 identifier is written here: the smallest available flavor of the region is
 chosen at call time from what the API reports, unless `--server-type` or
 `MVM_OVH_FLAVOR` names one.

 "Smallest" is virtual CPUs first, then memory, then disk, then the identifier,
 so the choice is fixed rather than whatever order OVHcloud answers in. A flavor
 whose `osType` is `linux` is preferred, because mvm boots Linux images; when a
 region offers no Linux flavor the smallest of any kind is used and the caller's
 create reports OVHcloud's own refusal.

 When a create asks for a number of virtual CPUs or an amount of memory, a
 different walk runs: only families OVHcloud sells as general-purpose Linux
 machines are candidates (see `flavor-families.ts`), a size is kept when its
 virtual CPUs and its memory are both at least what was asked, and the smallest
 survivor wins. A request no size satisfies refuses and names what was asked, the
 largest size the region had, and how to name a size outright.

 @module
 */

import {
  describeSizeRequest,
  NoSizeFitsError,
  refuseSizeAndOverride,
  type SizeRequest,
} from '../size-request.ts';
import { NO_PRICE, } from './catalog-prices.ts';
import { listFlavors, } from './api-resources.ts';
import { flavorOverride, } from './config.ts';
import {
  isKeptFlavor,
  LINUX_OS_TYPE,
} from './flavor-families.ts';
import type { OvhFlavor, } from './types.ts';

//region Errors

/**
 One region offers no flavor in stock, so a create advances to the next region
 rather than failing the whole request.
 
 @example
 ```ts
 try { await resolveFlavor({ region: 'bhs5' }); }
 catch (error) { if (error instanceof NoFlavorInStockError) nextRegion(); }
 ```
 */
export class NoFlavorInStockError extends Error {
  /**
   Region that offered nothing.
   */
  readonly region: string;

  /**
   @param region - region that offered no available flavor
   */
  constructor({ region, }: { readonly region: string; },) {
    super(`no flavor is in stock in region ${region}`,);
    this.name = 'NoFlavorInStockError';
    this.region = region;
  }
}

//endregion Errors

//region Selection

/**
 How a caller names a flavor outright, on the command line and in the environment.
 */
const OVH_OVERRIDE_OPTIONS = '--server-type or MVM_OVH_FLAVOR';

/**
 Part of a flavor name that marks the flex variation of the same size.
 */
const FLEX_SUFFIX = '-flex';

/**
 Answer for a search that found no flavor, so absence travels as a distinct value
 rather than a nullish union.
 */
export const NO_FLAVOR: unique symbol = Symbol(
  'returned for a search that found no flavor',
);

/**
 Whether a flavor's name is the flex variation of a size.
 
 OVHcloud documents flex instances as the ones whose local disk stays at 50 GB so
 the size can be resized up and down ("Flex instances allow resizing to higher or
 lower models due to a locked single disk size",
 `https://docs.ovhcloud.com/en/guides/public-cloud/compute/resize-instance-manager`,
 read on 2026-11-28). mvm never resizes a machine, it destroys and remakes one, so
 the flex benefit is unused; and the same guide says a flex instance cannot be
 turned back into a classic one, a door mvm has no reason to walk through for a
 machine whose whole life is one command. Two sizes equal in virtual CPUs, memory
 and disk are therefore ordered with the plain one first.
 
 @param flavor - flavor whose name is read
 
 @returns whether the name carries the flex suffix
 
 @example
 ```ts
 isFlexFlavor({ flavor: { name: 'c3-4-flex' } }); // true
 ```
 */
function isFlexFlavor(flavor: OvhFlavor,): boolean {
  return flavor.name
    .toLowerCase()
    .endsWith(FLEX_SUFFIX,);
}

/**
 Sorts flavors smallest first: virtual CPUs, then memory, then disk, then the
 plain size before its flex twin, then the identifier. Pure, and used by both the
 default choice and the error text.
 
 @param flavors - flavors to order
 
 @returns a new array in ascending size order
 
 @example
 ```ts
 orderFlavors({ flavors }); // [c3-4, c3-4-flex, b3-8, ...]
 ```
 */
export function orderFlavors({
  flavors,
}: {
  readonly flavors: readonly OvhFlavor[];
},): readonly OvhFlavor[] {
  return flavors.toSorted(function bySize(
    left,
    right,
  ) {
    if (left.vcpus !== right.vcpus) {
      return left.vcpus - right.vcpus;
    }
    if (left.ram !== right.ram) {
      return left.ram - right.ram;
    }
    if (left.disk !== right.disk) {
      return left.disk - right.disk;
    }
    /**
     Whether each side is the flex variation; the plain one sorts first.
     */
    const flexOrder = Number(isFlexFlavor(left,),) - Number(isFlexFlavor(right,),);
    if (flexOrder !== 0) {
      return flexOrder;
    }
    return left.id
      .localeCompare(right.id,);
  },);
}

/**
 Describes one flavor as a person reads it in a report or a refusal.
 
 @param flavor - flavor to describe
 
 @returns its name with its virtual CPUs, memory and disk
 
 @example
 ```ts
 describeFlavor({ flavor }); // 'b3-8 (2 vCPUs, 8 GiB of memory, 50 GB of disk)'
 ```
 */
export function describeFlavor(flavor: OvhFlavor,): string {
  return `${flavor.name} (${String(flavor.vcpus,)} vCPUs, ${String(flavor.ram,)} GiB of memory, ${String(flavor.disk,)} GB of disk)`;
}

/**
 Chooses the flavor to create with.
 
 @param flavors - flavors the region offers
 
 @param override - flavor name or identifier from `--server-type` or the environment, `''` for the smallest
 
 @param region - region the flavors belong to, named in errors
 
 @returns the flavor to create with
 
 @throws Error when the region offers nothing in stock, or the override names nothing in stock there
 
 @example
 ```ts
 selectFlavor({ flavors, override: '', region: 'bhs5' });      // the smallest available
 selectFlavor({ flavors, override: 'b3-8', region: 'bhs5' });  // that one, by name or id
 ```
 */
export function selectFlavor({
  flavors,
  override,
  region,
}: {
  readonly flavors: readonly OvhFlavor[];
  readonly override: string;
  readonly region: string;
},): OvhFlavor {
  /**
   Flavors the region has in stock; an unavailable one is refused at creation.
   */
  const available = flavors.filter(function inStock(flavor,) {
    return flavor.available;
  },);
  if (override !== '') {
    /**
     The named flavor, by identifier or by name.
     */
    const named = available.find(function matches(flavor,) {
      return (flavor.id === override) || (flavor.name === override);
    },);
    if (named === undefined) {
      throw new Error(
        `no flavor named "${override}" is in stock in region ${region}; available: ${
          orderFlavors({ flavors: available, },)
            .map(function flavorId(flavor,) {
              return flavor.id;
            },)
            .join(', ',)
        }`,
      );
    }
    return named;
  }
  /**
   Smallest available flavors, in size order.
   */
  const ordered = orderFlavors({ flavors: available, },);
  /**
   The smallest one that boots a Linux image, when the region offers any.
   */
  const smallestLinux = ordered.find(function isLinux(flavor,) {
    return flavor.osType === LINUX_OS_TYPE;
  },);
  if (smallestLinux !== undefined) {
    return smallestLinux;
  }
  /**
   The smallest of any kind; the create reports OVHcloud's own refusal when
   that flavor cannot boot the image.
   */
  const [smallest,] = ordered;
  if (smallest === undefined) {
    throw new NoFlavorInStockError({ region, },);
  }
  return smallest;
}

//endregion Selection

//region Reading

/**
 Resolves the flavor to create with in one region, from the region's own list.
 
 @param override - per-invocation flavor name or identifier, or `undefined`
 
 @param region - region to create in
 
 @returns the flavor to create with
 
 @throws Error when the region offers nothing in stock
 
 @example
 ```ts
 const flavor = await resolveFlavor({ region: 'bhs5' });
 ```
 */
export async function resolveFlavor({
  override,
  region,
}: {
  readonly override?: string;
  readonly region: string;
},): Promise<OvhFlavor> {
  return selectFlavor({
    flavors: await listFlavors({ region, },),
    override: flavorOverride(override,),
    region,
  },);
}

//endregion Reading

//region Sizes a request may use

/**
 Whether the hourly prices could be read, which decides the order among sizes
 that fit: by price when they could, by the numbers alone when they could not.
 
 @example
 ```ts
 const priced: PriceChoice = { kind: 'priced', currency: 'CAD', prices: new Map() };
 const fallback: PriceChoice = { kind: 'unpriced' };
 ```
 */
export type PriceChoice =
  | {
    /**
     The prices were read, so the cheapest size that fits wins.
     */
    readonly kind: 'priced';
    /**
     Currency the prices are stated in.
     */
    readonly currency: string;
    /**
     Hourly price per plan code.
     */
    readonly prices: ReadonlyMap<string, number>;
  }
    | {
    /**
     The prices could not be read, so the smallest numbers win.
     */
      readonly kind: 'unpriced';
    };

/**
 The flavors a request may use, in the order a create tries them, with the count
 of in-stock sizes the family rule left out.
 
 @example
 ```ts
 const fit: FlavorFit = { flavors: [c3_4], chosenBy: 'price', leftOut: 3 };
 ```
 */
export type FlavorFit = {
  /**
   Sizes that satisfy the request, smallest first; empty when none does.
   */
  readonly flavors: readonly OvhFlavor[];
  /**
   How many sizes in stock the family rule left out, so a refusal can say so.
   */
  readonly leftOut: number;
  /**
   The largest size the family rule left out, so a refusal can name what it
   declined to use, or {@link NO_FLAVOR} when it left none out.
   */
  readonly largestLeftOut: OvhFlavor | typeof NO_FLAVOR;
  /**
   Which order put the sizes in their place: the cheapest hourly price first, or
   the smallest numbers first because no price could be read.
   */
  readonly order: 'price' | 'size-numbers';
};

/**
 Keeps the sizes a request may use: in stock, from a family OVHcloud sells as a
 general-purpose Linux machine, and at least the asked virtual CPUs and memory.
 
 With the prices in hand the survivors are ordered by hourly price, cheapest
 first, so the machine costs the least that meets the ask rather than being the
 smallest that meets it; the project chose that order on 2026-11-28. A size the
 catalogue publishes no hourly price for is never a candidate then. Without the
 prices the survivors are ordered by the numbers alone, and the fit says so.
 
 @param flavors - flavors the region offers
 
 @param request - the floors the caller set
 
 @param priceChoice - the hourly prices, or the record that they could not be read
 
 @returns the surviving sizes in the order a create tries them, and the count left out
 
 @example
 ```ts
 flavorsThatFit({ flavors, priceChoice: { kind: 'unpriced' }, request: { vcpus: 2, memoryGiB: 4 } });
 ```
 */
export function flavorsThatFit({
  flavors,
  priceChoice,
  request,
}: {
  readonly flavors: readonly OvhFlavor[];
  readonly priceChoice: PriceChoice;
  readonly request: SizeRequest;
},): FlavorFit {
  /**
   Flavors the region has in stock; an unavailable one is refused at creation.
   */
  const inStock = flavors.filter(function inStockFlavor(flavor,) {
    return flavor.available;
  },);
  /**
   Of those, the ones from a family mvm may pick without being told a size.
   */
  const kept = inStock.filter(function keptFamily(flavor,) {
    return isKeptFlavor({ flavor, },);
  },);
  /**
   Sizes the family rule left out, which a refusal may name the largest of.
   */
  const leftOutSizes = inStock.filter(function specialFamily(flavor,) {
    return !isKeptFlavor({ flavor, },);
  },);
  /**
   Sizes from a kept family that meet both floors.
   */
  const fitting = kept.filter(function fitsFloors(flavor,) {
    return ((request.vcpus === undefined) || (flavor.vcpus >= request.vcpus))
      && ((request.memoryGiB === undefined) || (flavor.ram >= request.memoryGiB));
  },);
  return {
    flavors: (priceChoice.kind === 'priced')
      ? orderFlavorsByPrice({
        flavors: fitting.filter(function priced(flavor,) {
          return hourlyPriceFor({
            flavor,
            priceChoice,
          },) !== NO_PRICE;
        },),
        priceChoice,
      },)
      : orderFlavors({ flavors: fitting, },),
    largestLeftOut: largestFlavor({ flavors: leftOutSizes, },),
    leftOut: inStock.length - kept.length,
    order: (priceChoice.kind === 'priced') ? 'price' : 'size-numbers',
  };
}

/**
 Orders sizes by their hourly price, cheapest first, breaking a tie on price with
 the size order so the answer is fixed rather than whatever order the catalogue
 answered in.
 
 @param flavors - sizes to order, each with a published hourly price
 
 @param priceChoice - the hourly prices each size's own plan code is priced by
 
 @returns a new array, cheapest first
 
 @example
 ```ts
 orderFlavorsByPrice({ flavors, priceChoice }); // [d2-8, b2-7]
 ```
 */
function orderFlavorsByPrice({
  flavors,
  priceChoice,
}: {
  readonly flavors: readonly OvhFlavor[];
  readonly priceChoice: PriceChoice;
},): readonly OvhFlavor[] {
  return orderFlavors({ flavors, },)
    .toSorted(function byPrice(
      left,
      right,
    ) {
      /**
       Hourly price of the left size; both sides are priced, which is what
       selected them before this order ran.
       */
      const leftPrice = hourlyPriceFor({
        flavor: left,
        priceChoice,
      },);
      /**
       Hourly price of the right size.
       */
      const rightPrice = hourlyPriceFor({
        flavor: right,
        priceChoice,
      },);
      if ((leftPrice === NO_PRICE) || (rightPrice === NO_PRICE)) {
        return 0;
      }
      return leftPrice - rightPrice;
    },);
}

/**
 Hourly price of one size, for a caller that prints the pick.
 
 @param flavor - size to price
 
 @param priceChoice - the hourly prices, or the record that they could not be read
 
 @returns the price per hour in the catalogue's currency, or {@link NO_PRICE}
 
 @example
 ```ts
 hourlyPriceFor({ flavor, priceChoice }); // 0.0596
 ```
 */
export function hourlyPriceFor({
  flavor,
  priceChoice,
}: {
  readonly flavor: OvhFlavor;
  readonly priceChoice: PriceChoice;
},): number | typeof NO_PRICE {
  if (priceChoice.kind !== 'priced') {
    return NO_PRICE;
  }
  /**
   Plan code the catalogue prices this flavor by, when the body carries one.
   */
  const planCode = flavor.planCodes
    ?.hourly;
  if (planCode === undefined) {
    return NO_PRICE;
  }
  return priceChoice.prices
    .get(planCode,)
    ?? NO_PRICE;
}

/**
 The largest size among those given, which is what a refusal offers when nothing
 satisfies the request.
 
 @param flavors - flavors to search
 
 @returns the largest of them, or {@link NO_FLAVOR} when there are none
 
 @example
 ```ts
 largestFlavor({ flavors }); // b3-256
 ```
 */
export function largestFlavor({
  flavors,
}: {
  readonly flavors: readonly OvhFlavor[];
},): OvhFlavor | typeof NO_FLAVOR {
  return orderFlavors({ flavors, },)
    .at(-1,)
    ?? NO_FLAVOR;
}

/**
 Builds the refusal for a region whose kept sizes all miss the request.
 
 @param fit - the sizes a request may use, and what the family rule left out
 
 @param kept - the region's in-stock kept sizes, before the floors
 
 @param region - region the search ran in
 
 @param request - the floors the caller set
 
 @returns the refusal to throw
 
 @example
 ```ts
 throw noSizeFits({ kept, leftOut: 0, region: 'bhs5', request });
 ```
 */
function noSizeFits({
  fit,
  kept,
  region,
  request,
}: {
  readonly fit: FlavorFit;
  readonly kept: readonly OvhFlavor[];
  readonly region: string;
  readonly request: SizeRequest;
},): NoSizeFitsError {
  /**
   Largest size the region could have given, which a caller may name outright.
   */
  const largest = largestFlavor({ flavors: kept, },);
  return new NoSizeFitsError({
    asked: describeSizeRequest({ request, },),
    largest: (largest === NO_FLAVOR) ? '' : describeFlavor(largest,),
    largestLeftOut: (fit.largestLeftOut === NO_FLAVOR) ? '' : describeFlavor(fit.largestLeftOut,),
    leftOut: fit.leftOut,
    overrideOptions: OVH_OVERRIDE_OPTIONS,
    place: `region ${region} on OVHcloud`,
  },);
}

/**
 Reads the region's own flavor list and returns the sizes a request may use, in
 the order a create tries them.
 
 @param override - per-invocation flavor name or identifier from `--server-type`, or `undefined`
 
 @param priceChoice - the hourly prices, or the record that they could not be read
 
 @param region - region to create in
 
 @param request - the floors the caller set
 
 @returns the surviving sizes in the order a create tries them, and the count the family rule left out
 
 @throws Error when a size is named outright beside a floor
 
 @throws Error when the prices were read and no size that fits has a published one
 
 @throws {@link NoSizeFitsError} when no size the region has in stock satisfies the request
 
 @example
 ```ts
 const fit = await resolveFlavorsThatFit({ priceChoice, region: 'bhs5', request: { vcpus: 2 } });
 ```
 */
export async function resolveFlavorsThatFit({
  override,
  priceChoice,
  region,
  request,
}: {
  readonly override?: string;
  readonly priceChoice: PriceChoice;
  readonly region: string;
  readonly request: SizeRequest;
},): Promise<FlavorFit> {
  refuseSizeAndOverride({
    override: flavorOverride(override,),
    overrideOptions: OVH_OVERRIDE_OPTIONS,
    request,
  },);
  /**
   Flavors the region offers, before any rule is applied.
   */
  const flavors = await listFlavors({ region, },);
  /**
   Sizes a request may use, and how many the family rule left out.
   */
  const fit = flavorsThatFit({
    flavors,
    priceChoice,
    request,
  },);
  if (fit.flavors
    .length
    === 0) {
    /**
     Sizes that meet the floors and the family rule, before the price rule.
     */
    const floorsOnly = flavorsThatFit({
      flavors,
      priceChoice: { kind: 'unpriced', },
      request,
    },)
      .flavors;
    /**
     Every kept size in stock, before the floors, which is what a refusal offers
     as the largest the region had.
     */
    const keptBeforeFloors = flavorsThatFit({
      flavors,
      priceChoice: { kind: 'unpriced', },
      request: {},
    },)
      .flavors;
    if ((floorsOnly.length > 0) && (priceChoice.kind === 'priced')) {
      throw new Error(
        `no size in region ${region} is in stock with both a published hourly price and ${
          describeSizeRequest({ request, },)
        }: ${floorsOnly.length} size${(floorsOnly.length === 1) ? '' : 's'} would fit but ${
          (floorsOnly.length === 1) ? 'has' : 'have'
        } no price in the ${priceChoice.currency} catalogue; name a size outright with ${OVH_OVERRIDE_OPTIONS}`,
      );
    }
    throw noSizeFits({
      fit,
      kept: keptBeforeFloors,
      region,
      request,
    },);
  }
  return fit;
}

//endregion Sizes a request may use
