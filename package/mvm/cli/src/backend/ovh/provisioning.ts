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
import { createInstance, } from './api-resources.ts';
import { DEFAULT_IMAGE, } from './config.ts';
import { failedProvisionError, } from './discarding.ts';
import {
  NoFlavorInStockError,
  resolveFlavor,
} from './flavors.ts';
import { resolveImage, } from './images.ts';
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
import type { OvhInstance, } from './types.ts';

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
  region,
  rl,
  serverType,
  sshKeyId,
}: {
  readonly image?: string;
  readonly name: string;
  readonly region: string;
  readonly rl: Logger;
  readonly serverType?: string;
  readonly sshKeyId: string;
},): Promise<OvhInstance> {
  /**
   Flavor to create with in this region, or a thrown {@link NoFlavorInStockError}.
   */
  const flavor = await resolveFlavor({
    region,
    ...((serverType === undefined) ? {} : { override: serverType, }),
  },);
  /**
   Image to boot from in this region, matched against the region's own list.
   */
  const bootImage = await resolveImage({
    flavor,
    region,
    shorthand: image ?? DEFAULT_IMAGE,
  },);
  rl.info(
    `creating ${name} (flavor ${flavor.name}, image ${bootImage.name}) in ${region}`,
  );
  return createInstance({
    flavorId: flavor.id,
    imageId: bootImage.id,
    name,
    region,
    sshKeyId,
  },);
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
  rl,
  serverType,
}: {
  readonly image?: string;
  readonly location?: string;
  readonly name: string;
  readonly rl: Logger;
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
   Regions to try, in fallback order.
   */
  const regions = await resolveRegions(location,);
  for (const region of regions) {
    try {
      // oxlint-disable-next-line no-await-in-loop -- sequential so a region with nothing in stock falls through to the next
      return await provisionInRegion({
        name: fullName,
        region,
        rl,
        sshKeyId,
        ...((image === undefined) ? {} : { image, }),
        ...((serverType === undefined) ? {} : { serverType, }),
      },);
    }
    catch (error: unknown) {
      if (!(error instanceof NoFlavorInStockError)) {
        throw error;
      }
      rl.info(`${region} has no flavor in stock, trying the next region`,);
    }
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
