/**
 Resolves the cheapest currently-offered Hetzner server type.
 
 Server-type slugs are deprecated over time (cx22 -> cx23, ...), so rather
 than hardcode a default, pick the cheapest non-deprecated type offered in the
 target locations at call time. Any architecture is eligible; callers that
 boot from an architecture-specific image (a clone snapshot) constrain it.
 
 @module
 */

import { listServerTypes, } from './api-resources.ts';
import type { HetznerServerType, } from './types.ts';

//region Helpers

/**
 Sentinel hourly price meaning "not offered in the requested locations"; any
 real price compares lower.
 */
const NOT_OFFERED = Number.POSITIVE_INFINITY;

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
