/**
 OVHcloud image and quota reporting.

 OVHcloud manages images server-side, so there is nothing to download or bake
 locally. `update` validates the keys and reports, per Canadian region, the
 smallest flavor in stock, how many active images the region offers, and how much
 of the instance quota is in use. The quota is the number to read before a heavy
 run: a create refused for quota fails before any instance exists.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';

import {
  listFlavors,
  listImages,
  listQuotas,
} from './api-resources.ts';
import { requireCredentials, } from './config.ts';
import { orderFlavors, } from './flavors.ts';
import { listDefaultRegions, } from './regions.ts';
import type { OvhQuotas, } from './types.ts';

/**
 Logger root for mvm after removing the package log shim.

 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l, },);
 ```
 */
const l = tagged({ tag: 'mvm', },);

//region Reporting

/**
 Reports one region's smallest in-stock flavor, image count and instance quota.
 
 @param quotas - quota entries of every region, as the API answers them
 
 @param region - region to report
 
 @returns one line of text describing the region
 
 @example
 ```ts
 const line = await describeRegion({ quotas, region: 'bhs5' });
 ```
 */
async function describeRegion({
  quotas,
  region,
}: {
  readonly quotas: readonly OvhQuotas[];
  readonly region: string;
},): Promise<string> {
  /**
   Flavors of this region that are in stock, smallest first.
   */
  const inStock = orderFlavors({
    flavors: (await listFlavors({ region, },)).filter(function available(flavor,) {
      return flavor.available;
    },),
  },);
  /**
   Smallest in-stock flavor, absent when the region has none.
   */
  const [smallest,] = inStock;
  /**
   Active images the region offers.
   */
  const images = (await listImages({ region, },)).filter(function active(image,) {
    return image.status === 'active';
  },);
  /**
   This region's instance quota, absent when the API reports none.
   */
  const quota = quotas.find(function inRegion(entry,) {
    return entry.region === region;
  },)
    ?.instance;
  /**
   Smallest flavor text, or a statement that nothing is in stock.
   */
  const flavorText = (smallest === undefined)
    ? 'no flavor in stock'
    : `${smallest.id} (${String(smallest.vcpus,)} vCPU, ${String(smallest.ram,)} GiB)`;
  /**
   Quota text, or a statement that the API reported none.
   */
  const quotaText = (quota === undefined)
    ? 'quota not reported'
    : `${String(quota.usedInstances,)} of ${String(quota.maxInstances,)} instances used`;
  return `${region}: ${flavorText}, ${String(images.length,)} active images, ${quotaText}`;
}

/**
 Validates the keys and reports what the project offers in its Canadian regions.
 
 @throws Error naming the first missing key
 
 @example
 ```ts
 await ovhUpdate();
 ```
 */
export async function ovhUpdate(): Promise<void> {
  /**
   Logger scoped to this report so output is namespaced.
   */
  const rl = tagged({
    tag: ovhUpdate.name,
    l,
  },);
  requireCredentials();
  /**
   Regions the default fallback list holds, read from the project's own country codes.
   */
  const regions = await listDefaultRegions();
  /**
   Quota entries of every region, read once for all of them.
   */
  const quotas = await listQuotas();
  /**
   One line per region, read together because the regions do not depend on each other.
   */
  const lines = await Promise.all(regions.map(function describe(region,) {
    return describeRegion({
      quotas,
      region,
    },);
  },),);
  rl.info(
    `OVHcloud manages images server-side; ${String(regions.length,)} Canadian regions in this project, nothing to build locally.`,
  );
  for (const line of lines) {
    rl.info(line,);
  }
}

//endregion Reporting
