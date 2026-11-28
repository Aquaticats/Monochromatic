/**
 Which regions an operation may use, and in what order.

 The default list is the project's own: every region the API reports whose
 `countryCode` is Canada's, whose status is `UP`, and whose `services` hold an
 `instance` service that is `UP`, in name order. The country comes from that
 field alone, so a region whose name suggests a country it is not in is never
 selected, and no region name is written into this source. The service matters
 because a project also lists regions that offer storage only, under names that
 differ from an instance region's by a digit, and OVHcloud answers 404 to a
 flavor list for one of those.

 An explicit `--location`, then a comma series in `MVM_OVH_REGIONS`, replaces the
 default list, in that order, as the Hetzner backend's location reading does.

 @module
 */

import {
  getRegion,
  listRegionNames,
} from './api-resources.ts';
import {
  CANADA_COUNTRY_CODE,
  envRegions,
  parseRegions,
  REGION_INSTANCE_SERVICE,
  REGION_STATUS_UP,
  SERVICE_STATUS_UP,
} from './config.ts';
import type { OvhRegion, } from './types.ts';

//region Selection

/**
 Whether a region can create an instance: its `services` hold one named
 `instance` whose status is `UP`. A region that offers storage only lists no
 such service, and OVHcloud answers 404 to a flavor list for it.

 @param region - region record from `GET /cloud/project/{serviceName}/region/{regionName}`

 @returns `true` when a create may be attempted in the region

 @example
 ```ts
 offersInstances(region); // false for a region that offers storage only
 ```
 */
function offersInstances(region: OvhRegion,): boolean {
  return region
    .services
    .some(function isInstanceServiceUp(service,) {
      return (service.name === REGION_INSTANCE_SERVICE)
        && (service.status === SERVICE_STATUS_UP);
    },);
}

/**
 Keeps the regions a create may use by default: those the API reports as
 Canadian, up, and offering an instance service that is up, in name order so
 the fallback order never depends on the order OVHcloud happens to answer in.
 
 @param details - region records from `GET /cloud/project/{serviceName}/region/{regionName}`
 
 @returns selected region names, ascending
 
 @example
 ```ts
 selectDefaultRegions({ details }); // ['bhs5', 'gra9']
 ```
 */
export function selectDefaultRegions({
  details,
}: {
  readonly details: readonly OvhRegion[];
},): readonly string[] {
  return details
    .filter(function isUsableCanadian(region,) {
      return (region.countryCode === CANADA_COUNTRY_CODE)
        && (region.status === REGION_STATUS_UP)
        && offersInstances(region,);
    },)
    .map(function regionName(region,) {
      return region.name;
    },)
    .toSorted(function byName(
      left,
      right,
    ) {
      return left.localeCompare(right,);
    },);
}

//endregion Selection

//region Reading

/**
 Reads every region of the project and keeps the default ones.
 
 @returns the project's Canadian, up regions in name order
 
 @throws Error when the project offers no such region
 
 @example
 ```ts
 const regions = await listDefaultRegions();
 ```
 */
export async function listDefaultRegions(): Promise<readonly string[]> {
  /**
   Region names the project holds.
   */
  const names = await listRegionNames();
  /**
   Detail of every region, read together because they do not depend on each other.
   */
  const details = await Promise.all(names.map(function readOne(name,) {
    return getRegion({ regionName: name, },);
  },),);
  /**
   Default list built from the field that decides the country, never from a name.
   */
  const selected = selectDefaultRegions({ details, },);
  if (selected.length === 0) {
    throw new Error(
      `the project holds no region whose countryCode is "${CANADA_COUNTRY_CODE}" , whose status is ${REGION_STATUS_UP} and which offers an "${REGION_INSTANCE_SERVICE}" service that is ${SERVICE_STATUS_UP}; set --location or MVM_OVH_REGIONS to name regions yourself`,
    );
  }
  return selected;
}

/**
 Resolves the ordered region fallback list from an explicit override, the
 environment, or the project's own Canadian regions.
 
 @param override - per-invocation comma-separated regions, or `undefined`
 
 @returns ordered region fallback list (never empty)
 
 @throws Error when the default list is empty
 
 @example
 ```ts
 await resolveRegions('bhs5,gra9'); // ['bhs5', 'gra9']
 await resolveRegions(undefined);   // the project's Canadian regions
 ```
 */
export function resolveRegions(override?: string,): Promise<readonly string[]> {
  /**
   Regions from the explicit per-invocation override, when any.
   */
  const fromOverride = parseRegions(override,);
  if (fromOverride.length > 0) {
    return Promise.resolve(fromOverride,);
  }
  /**
   Regions from the environment, when the override was empty.
   */
  const fromEnv = envRegions();
  if (fromEnv.length > 0) {
    return Promise.resolve(fromEnv,);
  }
  return listDefaultRegions();
}

//endregion Reading
