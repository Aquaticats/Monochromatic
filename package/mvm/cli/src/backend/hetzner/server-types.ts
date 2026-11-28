/**
 Resolves the server type a create makes its machine from.
 
 Server-type slugs are deprecated over time (cx22 -> cx23, ...), so rather
 than hardcode a default, pick the cheapest non-deprecated type offered in the
 target locations at call time. Any architecture is eligible; callers that
 boot from an architecture-specific image (a clone snapshot) constrain it.
 
 When a create asks for a number of virtual CPUs or an amount of memory, a
 different walk runs over the same records: a type is a candidate only when it is
 not deprecated, offered in one of the requested locations, sold with shared
 virtual CPUs, and has at least the asked cores and memory; the smallest candidate
 wins. A refused request names what was asked, the largest type the locations
 had, and the option that names a type outright.
 
 Every field this file reads is one Hetzner documents on its server type object.
 No call to Hetzner's API is made by the tests of this file; they replace the
 HTTP layer with stand-ins.
 
 @module
 */

import {
  describeSizeRequest,
  NoSizeFitsError,
  type SizeRequest,
} from '../size-request.ts';
import { listServerTypes, } from './api-resources.ts';
import type { HetznerServerType, } from './types.ts';

//region Helpers

/**
 Sentinel hourly price meaning "not offered in the requested locations"; any
 real price compares lower.
 */
const NOT_OFFERED = Number.POSITIVE_INFINITY;

/**
 The `cpu_type` a general-purpose plan carries: virtual CPUs shared with other
 machines. A plan sold with dedicated cores is a different product, so mvm never
 picks one without being told its name.
 */
const SHARED_CPU_TYPE = 'shared';

/**
 How a caller names a server type outright, on the command line and in the
 environment.
 */
const HETZNER_OVERRIDE_OPTIONS = '--server-type or MVM_HCLOUD_SERVER_TYPE';

/**
 Observational server candidate paired with cheapest offered price.
 */
type PricedServerCandidate = {
  readonly name: string;
  readonly price: number;
};

/**
 Whether a server type is current (not deprecated). Hetzner sends `null` for
 current types and an object for deprecated ones.
 
 @param type - server type to inspect
 
 @returns whether the type is not deprecated
 
 @example
 ```ts
 isCurrent({ name: 'cx23', architecture: 'x86', deprecation: null, prices: [] }); // true
 ```
 */
function isCurrent(type: HetznerServerType,): boolean {
  /**
   Deprecation marker, typed `unknown` so the null/undefined check is legal.
   */
  const dep = type.deprecation;
  return (dep === undefined) || (dep === null);
}

/**
 Lowest hourly price of a server type across the requested locations, or
 {@link NOT_OFFERED} when it is offered in none of them.
 
 @param locations - location codes to consider
 
 @param type - server type whose prices are scanned
 
 @returns lowest hourly gross price, or {@link NOT_OFFERED}
 
 @example
 ```ts
 minHourlyPrice({ type, locations: ['fsn1', 'nbg1'] }); // 0.008
 ```
 */
function minHourlyPrice(
  {
    locations,
    type,
  }: {
    readonly locations: readonly string[];
    readonly type: HetznerServerType;
  },
): number {
  return type.prices
    .reduce(
    function cheaper(
      best,
      price,
    ) {
      /**
       This location's hourly price, only when the location is in scope.
       */
      const here = locations.includes(price.location,)
        ? Number(price.price_hourly
          .gross,)
        : NOT_OFFERED;
      return (here < best) ? here : best;
    },
    NOT_OFFERED,
    );
}

//endregion Helpers

//region Resolution

/**
 Resolves the cheapest non-deprecated server type offered in the given
 locations, optionally constrained to one CPU architecture.
 
 @param architecture - required CPU architecture (e.g. `arm`), or any when omitted
 
 @param locations - location codes the type must be offered in
 
 @returns cheapest matching server type slug
 
 @throws Error when no non-deprecated type matches in the given locations
 
 @example
 ```ts
 await resolveCheapestServerType({ locations: ['fsn1', 'nbg1', 'hel1'] }); // 'cx23'
 await resolveCheapestServerType({ architecture: 'arm', locations: ['fsn1'] }); // 'cax11'
 ```
 */
export async function resolveCheapestServerType(
  {
    architecture,
    locations,
  }: {
    readonly architecture?: string;
    readonly locations: readonly string[];
  },
): Promise<string> {
  /**
   Non-deprecated, architecture-matching types paired with their cheapest
   in-location price, keeping only those offered in a requested location.
   */
  const priced = (await listServerTypes())
    .filter(function current(type,) {
      return isCurrent(type,);
    },)
    .filter(function archMatches(type,) {
      return (architecture === undefined) || (type.architecture === architecture);
    },)
    .map(function withPrice(type,): PricedServerCandidate {
      return {
        name: type.name,
        price: minHourlyPrice({
          locations,
          type,
        },),
      };
    },)
    .filter(function offered(candidate,) {
      return candidate.price < NOT_OFFERED;
    },);
  /**
   Head and tail of the candidate list; head seeds the cheapest-wins reduce.
   */
  const [first, ...others] = priced;
  if (first === undefined) {
    throw new Error(
      `no non-deprecated Hetzner server type${
        architecture === undefined ? '' : ` (${architecture})`
      } offered in: ${locations.join(', ',)}`,
    );
  }
  return others.reduce(
    function pickCheaper(
      best,
      candidate,
    ) {
      return (candidate.price < best.price) ? candidate : best;
    },
    first,
  )
    .name;
}

//endregion Resolution

//region Sizes a request may use

/**
 Whether a server type is offered in at least one of the requested locations.
 
 @param locations - location codes to consider
 
 @param type - server type whose prices are scanned
 
 @returns whether any requested location appears among its prices
 
 @example
 ```ts
 offeredIn({ locations: ['fsn1'], type }); // true
 ```
 */
function offeredIn({
  locations,
  type,
}: {
  readonly locations: readonly string[];
  readonly type: HetznerServerType;
},): boolean {
  return type.prices.some(function pricedIn(price,) {
    return locations.includes(price.location,);
  },);
}

/**
 Orders server types smallest first: cores, then memory, then disk, then name.
 Pure, and used by both the choice and the refusal.
 
 @param types - types to order
 
 @returns a new array in ascending size order
 
 @example
 ```ts
 orderServerTypes({ types }); // [cx23, cx33, ...]
 ```
 */
export function orderServerTypes({
  types,
}: {
  readonly types: readonly HetznerServerType[];
},): readonly HetznerServerType[] {
  return types.toSorted(function bySize(
    left,
    right,
  ) {
    if (left.cores !== right.cores) {
      return left.cores - right.cores;
    }
    if (left.memory !== right.memory) {
      return left.memory - right.memory;
    }
    if (left.disk !== right.disk) {
      return left.disk - right.disk;
    }
    return left.name
      .localeCompare(right.name,);
  },);
}

/**
 Describes one server type as a person reads it in a report or a refusal.
 
 @param type - server type to describe
 
 @returns its name with its cores, memory and disk
 
 @example
 ```ts
 describeServerType(type); // 'cx23 (2 cores, 4 GB of memory, 40 GB of disk)'
 ```
 */
export function describeServerType(type: HetznerServerType,): string {
  return `${type.name} (${String(type.cores,)} cores, ${String(type.memory,)} GB of memory, ${String(type.disk,)} GB of disk)`;
}

/**
 Resolves the smallest server type that meets a size request.
 
 A candidate is a type that is not deprecated, is offered in one of the requested
 locations, is sold with shared virtual CPUs, and has at least the asked cores and
 memory. The smallest such candidate wins, by cores, then memory, then disk, then
 name.
 
 @param locations - location codes the type must be offered in
 
 @param request - the floors the caller set; a request with no floor is the
 cheapest-type walk and is not what this function is for
 
 @param types - server types to choose from, read from Hetzner when omitted
 
 @returns the chosen server type
 
 @throws {@link NoSizeFitsError} when no type the locations offer meets the request
 
 @example
 ```ts
 const chosen = await resolveServerType({ locations: ['fsn1'], request: { vcpus: 2, memoryGiB: 4 } });
 chosen.name; // 'cx23'
 ```
 */
export async function resolveServerType({
  locations,
  request,
  types,
}: {
  readonly locations: readonly string[];
  readonly request: SizeRequest;
  readonly types?: readonly HetznerServerType[];
},): Promise<HetznerServerType> {
  /**
   Types to choose from, read from Hetzner only when the caller did not hand a list.
   */
  const all = types ?? (await listServerTypes());
  /**
   Current types the requested locations offer, before the CPU kind is consulted.
   */
  const current = all.filter(function usable(type,) {
    return isCurrent(type,) && offeredIn({
      locations,
      type,
    },);
  },);
  /**
   Of those, the ones sold with shared virtual CPUs.
   */
  const shared = current.filter(function sharedCpu(type,) {
    return type.cpu_type === SHARED_CPU_TYPE;
  },);
  /**
   The ones that meet both floors, smallest first.
   */
  const fitting = orderServerTypes({
    types: shared.filter(function meetsFloors(type,) {
      return ((request.vcpus === undefined) || (type.cores >= request.vcpus))
        && ((request.memoryGiB === undefined) || (type.memory >= request.memoryGiB));
    },),
  },);
  /**
   Smallest candidate that meets the request.
   */
  const [smallest,] = fitting;
  if (smallest !== undefined) {
    return smallest;
  }
  /**
   Largest type the locations could have offered, which a refusal names.
   */
  const largest = orderServerTypes({ types: shared, },)
    .at(-1,);
  throw new NoSizeFitsError({
    asked: describeSizeRequest({ request, },),
    largest: (largest === undefined) ? '' : describeServerType(largest,),
    leftOut: current.length - shared.length,
    overrideOptions: HETZNER_OVERRIDE_OPTIONS,
    place: `in ${locations.join(', ',)} on Hetzner`,
  },);
}

//endregion Sizes a request may use
