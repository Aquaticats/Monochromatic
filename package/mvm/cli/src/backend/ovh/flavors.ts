/**
 Which flavor (plan) an instance is created with in one region.

 OVHcloud's flavor identifiers churn the way any provider's plans do, so no
 identifier is written here: the smallest available flavor of the region is
 chosen at call time from what the API reports, unless `--server-type` or
 `MVM_OVH_FLAVOR` names one.

 "Smallest" is virtual CPUs first, then memory, then the identifier, so the
 choice is fixed rather than whatever order OVHcloud answers in. A flavor whose
 `osType` is `linux` is preferred, because mvm boots Linux images; when a region
 offers no Linux flavor the smallest of any kind is used and the caller's create
 reports OVHcloud's own refusal.

 @module
 */

import { listFlavors, } from './api-resources.ts';
import { flavorOverride, } from './config.ts';
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
 The `osType` a flavor that can boot a Linux image carries. The schema types the
 field as a bare string, so this one value is taken from OVHcloud's public
 catalog as the Linux family rather than from an enum.
 */
const LINUX_OS_TYPE = 'linux';

/**
 Sorts flavors smallest first: virtual CPUs, then memory, then identifier.
 Pure, and used by both the default choice and the error text.
 
 @param flavors - flavors to order
 
 @returns a new array in ascending size order
 
 @example
 ```ts
 orderFlavors({ flavors }); // [b3-8, b3-16, ...]
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
    return left.id
      .localeCompare(right.id,);
  },);
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
