/**
 Getting an instance up and reachable: the region fallback, the flavor and image
 choice inside each region, and the wait until SSH answers as the account the
 image itself names.

 A create walks the region fallback list and creates in the first region whose
 flavor list has something in stock; a region that offers nothing is skipped
 before any request that would create something. Everything after the create
 runs under one guard: if it fails, the instance is deleted before the failure is
 reported.

 @module
 */

import {
  tagged,
  type Logger,
} from '@monochromatic-dev/logger-core/ts';

import { VM_PREFIX, } from '../../config.ts';
import {
  describeSizeRequest,
  isEmptySizeRequest,
  type SizeRequest,
  NoSizeFitsError
} from '../size-request.ts';
import { NO_PRICE, } from './catalog-prices.ts';
import { readPriceChoice, } from './read-price-choice.ts';
import {
  createInstance,
  listImages,
} from './api-resources.ts';
import {
  DEFAULT_IMAGE,
  requireCredentials,
} from './config.ts';
import { failedProvisionError, } from './discarding.ts';
import {
  hourlyPriceFor,
  NoFlavorInStockError,
  resolveFlavor,
  resolveFlavorsThatFit,
  type PriceChoice,
} from './flavors.ts';
import {
  resolveImage,
  selectImage,
} from './images.ts';
import {
  assertNameIsNotDeleting,
  instanceImageUser,
  instanceLoginUser,
  publicIpv4,
  waitUntilRunning,
} from './instances.ts';
import { resolveRegions, } from './regions.ts';
import { ensureSshKeyId, } from './ssh-key.ts';
import { waitForSsh, } from './ssh.ts';
import type {
  OvhFlavor,
  OvhImage,
  OvhInstance,
} from './types.ts';

/**
 Logger root for mvm after removing the package log shim.

 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l, },);
 ```
 */
const l = tagged({ tag: 'mvm', },);

//region Shapes

/**
 Where an SSH client reaches an instance, and as whom.
 */
export type SshTarget = {
  readonly ip: string;
  readonly user: string;
};

//endregion Shapes

//region Provisioning

/**
 Creates an instance in one region, from that region's own flavor and image lists.
 
 @param image - image shorthand such as `ubuntu`
 
 @param name - full instance name including the mvm- prefix
 
 @param region - region to create in
 
 @param rl - logger for progress
 
 @param serverType - per-invocation flavor name or identifier, or omitted
 
 @param sshKeyId - identifier of the key to inject
 
 @returns the instance as OVHcloud answers the create
 
 @throws {@link NoFlavorInStockError} when the region has nothing in stock
 
 @throws Error when the image matches nothing in the region
 
 @example
 ```ts
 const created = await provisionInRegion({ name: 'mvm-dev', region: 'bhs5', rl, sshKeyId });
 ```
 */
async function provisionInRegion({
  image,
  name,
  priceChoice,
  region,
  request,
  rl,
  serverType,
  sshKeyId,
}: {
  readonly image?: string;
  readonly name: string;
  readonly priceChoice: PriceChoice;
  readonly region: string;
  readonly request: SizeRequest;
  readonly rl: Logger;
  readonly serverType?: string;
  readonly sshKeyId: string;
},): Promise<OvhInstance> {
  /**
   Size and image this region gives the instance.
   */
  const chosen = await chooseFlavorAndImage({
    priceChoice,
    region,
    request,
    rl,
    ...((image === undefined) ? {} : { image, }),
    ...((serverType === undefined) ? {} : { override: serverType, }),
  },);
  rl.info(
    `creating ${name} (flavor ${chosen.flavor
      .name} with ${String(chosen.flavor
        .vcpus,)} vCPUs, ${String(chosen.flavor
          .ram,)} GiB of memory and ${String(chosen.flavor
            .disk,)} GB of disk, image ${chosen.image
              .name}) in ${region}`,
  );
  return createInstance({
    flavorId: chosen.flavor
      .id,
    imageId: chosen.image
      .id,
    name,
    region,
    sshKeyId,
  },);
}

/**
 Chooses the size and the image to create with in one region.
 
 Without a size request this is the walk mvm has always made: the smallest flavor
 in stock, then the newest plain image that flavor can boot. With a request it is
 the size walk: the kept families in stock that meet both floors, smallest first,
 and the first of them that boots an image the region offers.
 
 @param image - image shorthand such as `ubuntu`, or omitted for `ubuntu`
 
 @param override - per-invocation flavor name or identifier, or omitted
 
 @param region - region to create in
 
 @param request - the floors the caller set
 
 @param rl - logger for the image the size walk settled on
 
 @returns the flavor and the image to create with
 
 @throws {@link NoFlavorInStockError} when the region has nothing in stock
 
 @throws {@link NoSizeFitsError} when no size the region may use meets the request
 
 @throws Error when the image matches nothing, or no size that fits boots one
 
 @example
 ```ts
 const chosen = await chooseFlavorAndImage({ region: 'bhs5', request: { vcpus: 2 }, rl });
 ```
 */
async function chooseFlavorAndImage({
  image,
  override,
  priceChoice,
  region,
  request,
  rl,
}: {
  readonly image?: string;
  readonly override?: string;
  readonly priceChoice: PriceChoice;
  readonly region: string;
  readonly request: SizeRequest;
  readonly rl: Logger;
}): Promise<{
  readonly flavor: OvhFlavor;
  readonly image: OvhImage;
  readonly priceChoice: PriceChoice;
}> {
  /**
   Image shorthand this create asked for.
   */
  const shorthand = image ?? DEFAULT_IMAGE;
  if (isEmptySizeRequest({ request, },)) {
    /**
     Smallest in-stock size of a family mvm may pick without being told one; a
     region whose sizes are all families it leaves out refuses instead.
     */
    const flavor = await resolveFlavor({
      region,
      ...((override === undefined) ? {} : { override, }),
    },);
    return {
      flavor,
      image: await resolveImage({
        flavor,
        region,
        shorthand,
      },),
      priceChoice,
    };
  }
  /**
   Images this region offers, read once for every candidate the size walk tries.
   */
  const images = await listImages({ region, },);
  /**
   Sizes that meet the request, smallest first.
   */
  const fit = await resolveFlavorsThatFit({
    priceChoice,
    region,
    request,
    ...((override === undefined) ? {} : { override, }),
  },);
  /**
   Refusals the image walk made, in candidate order; the first is reported when
   no candidate could boot one.
   */
  const refusals: unknown[] = [];
  for (const flavor of fit.flavors) {
    try {
      /**
       Image this candidate would boot.
       */
      const imageForCandidate = selectImage({
        flavor,
        images,
        region,
        shorthand,
      },);
      /**
       Hourly price of this candidate, or {@link NO_PRICE} when none could be read.
       */
      const price = hourlyPriceFor({
        flavor,
        priceChoice,
      },);
      rl.info(
        `size ${describeSizeRequest({ request, },)} fits ${flavor.name} (${String(flavor.vcpus,)} vCPUs, ${String(flavor.ram,)} GiB of memory, ${String(flavor.disk,)} GB of disk)${
          (price === NO_PRICE) ? ', no published hourly price could be read so the smallest numbers win' : `, ${String(price,)} ${(priceChoice.kind === 'priced') ? priceChoice.currency : ''} per hour${(fit.order === 'price') ? '' : ', read by size because no price was available'}`
        }`,
      );
      return {
        flavor,
        image: imageForCandidate,
        priceChoice,
      };
    }
    catch (error: unknown) {
      refusals.push(error,);
    }
  }
  /**
   Refusal the smallest candidate earned, which is the one a create would hit.
   */
  const [firstRefusal,] = refusals;
  if (Error.isError(firstRefusal,)) {
    throw firstRefusal;
  }
  throw new Error(
    `no size in ${region} boots an image matching "${shorthand}": ${refusals.length === 0 ? 'no size that fits was in stock' : 'every candidate was refused'}`,
  );
}

/**
 Creates an instance in the first region of the fallback list that has a flavor
 in stock.
 
 @param image - image shorthand such as `ubuntu`, or omitted for `ubuntu`
 
 @param location - per-invocation region or comma-separated fallback series, or omitted
 
 @param name - VM name without the mvm- prefix
 
 @param rl - logger for progress and skipped regions
 
 @param serverType - per-invocation flavor name or identifier, or omitted
 
 @returns the instance as OVHcloud answers the create
 
 @throws Error when no region has a flavor in stock, or the image matches nothing
 
 @example
 ```ts
 const created = await provisionInstance({ name: 'dev-01', rl });
 ```
 */
export async function provisionInstance({
  image,
  location,
  name,
  request,
  rl,
  serverType,
}: {
  readonly image?: string;
  readonly location?: string;
  readonly name: string;
  readonly rl: Logger;
  readonly request: SizeRequest;
  readonly serverType?: string;
},): Promise<OvhInstance> {
  /**
   Full instance name including the mvm- prefix.
   */
  const fullName = `${VM_PREFIX}${name}`;
  await assertNameIsNotDeleting({ name, },);
  /**
   Key identifier injected into the instance so SSH can connect afterwards.
   */
  const sshKeyId = await ensureSshKeyId();
  /**
   Regions to try, in fallback order, each with the same request.
   */
  const regions = await resolveRegions(location,);
  /**
   Hourly prices of this create, read once, or the record that they could not be.
   */
  const priceChoice = await readPriceChoice({ rl, },);
  /**
   One refusal per region that had nothing usable, reported together when no
   region can give the machine.
   */
  const refusals: NoSizeFitsError[] = [];
  for (const region of regions) {
    try {
      // oxlint-disable-next-line no-await-in-loop -- sequential so a region with nothing in stock falls through to the next
      return await provisionInRegion({
        name: fullName,
        priceChoice,
        region,
        request,
        rl,
        sshKeyId,
        ...((image === undefined) ? {} : { image, }),
        ...((serverType === undefined) ? {} : { serverType, }),
      },);
    }
    catch (error: unknown) {
      if (error instanceof NoFlavorInStockError) {
        rl.info(`${region} has no flavor in stock, trying the next region`,);
        continue;
      }
      if (error instanceof NoSizeFitsError) {
        rl.info(`${region} has no size that fits, trying the next region`,);
        refusals.push(error,);
        continue;
      }
      throw error;
    }
  }
  if (refusals.length > 0) {
    /**
     What the regions could not give: the floors asked back, or, when no number
     was asked, the kind of size a create that names none takes.
     */
    const wanted = isEmptySizeRequest({ request, },)
      ? 'a size mvm picks without being told one'
      : describeSizeRequest({ request, },);
    throw new Error(
      `no region among ${regions.join(', ',)} can give ${wanted}: ${
        refusals.map(function oneRefusal(refusal: { readonly message: string; },) {
          return refusal.message;
        },)
          .join(' | ')}`,
    );
  }
  throw new Error(
    `no flavor is in stock in any of: ${regions.join(', ',)}`,
  );
}

//endregion Provisioning



//region Reaching

/**
 Waits until an instance runs, then until SSH answers on it.
 
 @param instance - instance just created
 
 @param name - VM name without the mvm- prefix, for messages
 
 @param rl - logger for progress
 
 @returns where the instance answers and as whom
 
 @throws Error when the instance fails, never runs, or never answers SSH
 
 @example
 ```ts
 const target = await awaitReachable({ instance, name: 'dev-01', rl });
 ```
 */
async function awaitReachable({
  instance,
  name,
  rl,
}: {
  readonly instance: OvhInstance;
  readonly name: string;
  readonly rl: Logger;
},): Promise<SshTarget> {
  /**
   Instance as it stands once OVHcloud reports it running.
   */
  const running = await waitUntilRunning({
    instanceId: instance.id,
    name,
  },);
  /**
   Public IPv4 an SSH client reaches it on.
   */
  const ip = publicIpv4({ instance: running, },);
  /**
   Account the image itself names, never assumed to be `root`.
   */
  const user = await instanceLoginUser({ instance: running, },);
  rl.info(`instance ${name} is running at ${ip}; waiting for SSH as ${user}`,);
  await waitForSsh({
    ip,
    user,
  },);
  return {
    ip,
    user,
  };
}

/**
 Reaches a freshly created instance, deleting it when that fails.
 
 @param instance - instance just created
 
 @param name - VM name without the mvm- prefix
 
 @param rl - logger for progress
 
 @returns where the instance answers and as whom
 
 @throws Error from {@link failedProvisionError} when the instance cannot be reached
 
 @example
 ```ts
 const target = await reachOrDiscard({ instance, name: 'dev-01', rl });
 ```
 */
export async function reachOrDiscard({
  instance,
  name,
  rl,
}: {
  readonly instance: OvhInstance;
  readonly name: string;
  readonly rl: Logger;
},): Promise<SshTarget> {
  try {
    return await awaitReachable({
      instance,
      name,
      rl,
    },);
  }
  catch (error: unknown) {
    throw await failedProvisionError({
      failure: error,
      instance,
      name,
      rl,
    },);
  }
}

/**
 The account to log in as on a clone: the new instance's own image when it names
 one, else the source's, since a snapshot boots the source's operating system.
 
 @param instance - the instance booted from the snapshot
 
 @param source - the instance that was snapshotted
 
 @returns the login account
 
 @throws Error when neither image names one
 
 @example
 ```ts
 const user = await cloneLoginUser({ instance: created, source });
 ```
 */
async function cloneLoginUser({
  instance,
  source,
}: {
  readonly instance: OvhInstance;
  readonly source: OvhInstance;
},): Promise<string> {
  /**
   Account the snapshot image names, when it names one.
   */
  const fromSnapshot = await instanceImageUser({ instance, },);
  if (fromSnapshot !== '') {
    return fromSnapshot;
  }
  return instanceLoginUser({ instance: source, },);
}

/**
 Reaches a freshly created clone, deleting the clone when that fails. The
 snapshot is removed by the scope guard in the clone, whatever happens here.
 
 @param created - instance booted from the snapshot
 
 @param destination - VM name without the mvm- prefix
 
 @param rl - logger for progress
 
 @param source - instance that was snapshotted
 
 @returns where the clone answers and as whom
 
 @throws Error from {@link failedProvisionError} when the clone cannot be reached
 
 @example
 ```ts
 const target = await reachClone({ created, destination: 'dev-02', rl, source });
 ```
 */
export async function reachClone({
  created,
  destination,
  rl,
  source,
}: {
  readonly created: OvhInstance;
  readonly destination: string;
  readonly rl: Logger;
  readonly source: OvhInstance;
},): Promise<SshTarget> {
  try {
    /**
     Clone as it stands once OVHcloud reports it running.
     */
    const running = await waitUntilRunning({
      instanceId: created.id,
      name: destination,
    },);
    /**
     Public IPv4 of the running clone.
     */
    const ip = publicIpv4({ instance: running, },);
    /**
     Account the clone's own image, or the source's, names.
     */
    const user = await cloneLoginUser({
      instance: running,
      source,
    },);
    await waitForSsh({
      ip,
      user,
    },);
    return {
      ip,
      user,
    };
  }
  catch (error: unknown) {
    throw await failedProvisionError({
      failure: error,
      instance: created,
      name: destination,
      rl,
    },);
  }
}

//endregion Reaching
