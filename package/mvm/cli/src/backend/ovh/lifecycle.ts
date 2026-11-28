/**
 Instance lifecycle: create, clone through a snapshot, destroy, destroy all, and
 list.

 A create walks the region fallback list and creates in the first region whose
 flavor list has something in stock; a region that offers nothing is skipped
 before any request that would create something. Every failure after an instance
 exists deletes it, so a create that cannot finish leaves nothing billed, and a
 delete that itself fails names the one command that removes the instance.

 A clone snapshots the running source, waits for the snapshot to become usable,
 boots a new instance from it, and deletes the snapshot on scope exit, so a
 clone's snapshot never outlives the clone.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { VM_PREFIX, } from '../../config.ts';
import type { VmInfo, } from '../../list.ts';
import {
  refuseSizeAndOverride,
  sizeRequest,
  type SizeRequest,
} from '../size-request.ts';
import {
  createInstance,
  createSnapshot,
  deleteInstance,
  deleteSnapshot,
  listFlavors,
  listSnapshots,
} from './api-resources.ts';
import {
  flavorOverride,
  validateOvhName,
} from './config.ts';
import { safeDeleteSnapshot, } from './discarding.ts';
import { resolveFlavor, } from './flavors.ts';
import {
  findMvmInstance,
  listMvmInstances,
  waitForSnapshotReady,
} from './instances.ts';
import {
  provisionInstance,
  reachClone,
  reachOrDiscard,
} from './provisioning.ts';
import { ensureSshKeyId, } from './ssh-key.ts';

/**
 Logger root for mvm after removing the package log shim.

 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l, },);
 ```
 */
const l = tagged({ tag: 'mvm', },);

//region Create

/**
 Creates and starts an OVHcloud instance, waiting until SSH answers on it.
 
 @param image - image shorthand or literal catalogue name, defaults to `ubuntu`
 
 @param location - per-invocation region or comma-separated fallback series
 
 @param memoryGiB - least memory the machine may have, in gibibytes, or omitted
 
 @param name - VM name without the mvm- prefix
 
 @param serverType - per-invocation flavor name or identifier
 
 @param vcpus - fewest virtual CPUs the machine may have, or omitted
 
 @throws Error on an invalid name, on no region with stock or with a size that fits, on a size named beside a floor, or when the instance never answers
 
 @example
 ```ts
 await ovhCreate({ name: 'dev-01' });
 await ovhCreate({ name: 'big', serverType: 'b3-16', location: 'bhs5' });
 await ovhCreate({ memoryGiB: 4, name: 'build', vcpus: 2 });
 ```
 */
export async function ovhCreate({
  image,
  location,
  memoryGiB,
  name,
  serverType,
  vcpus,
}: {
  readonly name: string;
  readonly image?: string;
  readonly memoryGiB?: number;
  readonly serverType?: string;
  readonly location?: string;
  readonly vcpus?: number;
},): Promise<void> {
  validateOvhName(name,);
  /**
   Logger scoped to this create so steps are namespaced.
   */
  const rl = tagged({
    tag: ovhCreate.name,
    l,
  },);
  /**
   The floors this create set; empty when it set none, which keeps the smallest
   size in stock the walk it has always been.
   */
  const request: SizeRequest = sizeRequest({
    memoryGiB,
    vcpus,
  },);
  refuseSizeAndOverride({
    override: flavorOverride(serverType,),
    overrideOptions: '--server-type or MVM_OVH_FLAVOR',
    request,
  },);
  /**
   Instance OVHcloud created; from here on a failure has to delete it.
   */
  const created = await provisionInstance({
    name,
    request,
    rl,
    ...((image === undefined) ? {} : { image, }),
    ...((location === undefined) ? {} : { location, }),
    ...((serverType === undefined) ? {} : { serverType, }),
  },);
  /**
   Where the finished instance answers, reported to the operator.
   */
  const target = await reachOrDiscard({
    instance: created,
    name,
    rl,
  },);
  rl.info(
    `VM ${name} is ready at ${target.ip} as ${target.user}. Connect with: mvm --backend ovh shell ${name}`,
  );
}

//endregion Create

//region Clone

/**
 Clones an OVHcloud instance by snapshotting it and booting a new instance from
 that snapshot. The source keeps running; the snapshot is deleted once the clone
 is created, and a clone that fails deletes what it made.
 
 @param destination - destination VM name without the mvm- prefix
 
 @param source - source VM name without the mvm- prefix
 
 @throws Error on an invalid name, a missing source, or a failed provision
 
 @example
 ```ts
 await ovhClone({ destination: 'dev-02', source: 'dev-01' });
 ```
 */
export async function ovhClone({
  destination,
  source,
}: {
  readonly destination: string;
  readonly source: string;
},): Promise<void> {
  validateOvhName(destination,);
  /**
   Logger scoped to this clone so steps are namespaced.
   */
  const rl = tagged({
    tag: ovhClone.name,
    l,
  },);
  /**
   Source instance, resolved prefix-scoped and exact so a foreign instance is never snapshotted.
   */
  const src = await findMvmInstance({ name: source, },);
  rl.info(`snapshotting ${source} to clone into ${destination}`,);
  /**
   Snapshot of the source; its identifier is what the clone boots from.
   */
  const { snapshotId, } = await createSnapshot({
    instanceId: src.id,
    snapshotName: `${VM_PREFIX}clone-${source}-${destination}`,
  },);
  /**
   Guard deleting the snapshot when this scope exits, so it never outlives the clone.
   */
  await using _snapshotCleanup = {
    async [Symbol.asyncDispose](): Promise<void> {
      await safeDeleteSnapshot({
        imageId: snapshotId,
        rl,
      },);
    },
  };
  await waitForSnapshotReady({ imageId: snapshotId, },);
  /**
   Flavor for the clone, in the source's region, where the snapshot lives.
   */
  const flavor = await resolveFlavor({
    override: flavorOverride(),
    region: src.region,
  },);
  /**
   The new instance, booted from the snapshot in the source's own region.
   */
  const created = await createInstance({
    flavorId: flavor.id,
    imageId: snapshotId,
    name: `${VM_PREFIX}${destination}`,
    region: src.region,
    sshKeyId: await ensureSshKeyId(),
  },);
  /**
   Where the clone answers, reported to the operator.
   */
  const target = await reachClone({
    created,
    destination,
    rl,
    source: src,
  },);
  rl.info(
    `VM ${destination} is ready at ${target.ip} as ${target.user} (cloned from ${source}).`,
  );
}

//endregion Clone

//region Destroy and list

/**
 Destroys one mvm-owned instance by name. A stopped instance still bills, so this
 is the only operation that ends a machine's cost.
 
 @param name - VM name without the mvm- prefix
 
 @throws Error when no single owned instance has that name
 
 @example
 ```ts
 await ovhDestroy({ name: 'dev-01' });
 ```
 */
export async function ovhDestroy({
  name,
}: {
  readonly name: string;
},): Promise<void> {
  validateOvhName(name,);
  /**
   Logger scoped to this destroy so steps are namespaced.
   */
  const rl = tagged({
    tag: ovhDestroy.name,
    l,
  },);
  /**
   Target instance, resolved prefix-scoped and exact.
   */
  const instance = await findMvmInstance({ name, },);
  rl.info(`destroying ${name} (id ${instance.id})`,);
  await deleteInstance({ instanceId: instance.id, },);
  rl.info(`VM ${name} destroyed`,);
}

/**
 Destroys every mvm-owned instance, then every snapshot whose name carries the
 tool's prefix, so an interrupted clone cannot leave one billing per GB per month.
 Both passes are scoped by the name prefix, never by region.
 
 @throws Error when a delete fails
 
 @example
 ```ts
 await ovhDestroyAll();
 ```
 */
export async function ovhDestroyAll(): Promise<void> {
  /**
   Logger scoped to this bulk destroy so steps are namespaced.
   */
  const rl = tagged({
    tag: ovhDestroyAll.name,
    l,
  },);
  /**
   Every instance mvm owns, the only deletion scope.
   */
  const instances = await listMvmInstances();
  if (instances.length === 0) {
    rl.info('no VMs to destroy',);
  }
  else {
    rl.info(`destroying all ${String(instances.length,)} VMs`,);
    for (const instance of instances) {
      // oxlint-disable-next-line no-await-in-loop -- sequential to avoid hammering the API
      await deleteInstance({ instanceId: instance.id, },);
    }
  }
  /**
   Snapshots carrying the tool's prefix; swept so an interrupted clone cannot leak one.
   */
  const snapshots = await listSnapshots();
  for (const snapshot of snapshots) {
    if (!snapshot.name
      .startsWith(VM_PREFIX,)) {
      continue;
    }
    // oxlint-disable-next-line no-await-in-loop -- sequential to avoid hammering the API
    await deleteSnapshot({ imageId: snapshot.id, },);
  }
}

/**
 Lists the instances mvm owns with their state and the size they run.
 
 The size is read from the flavor each instance names, in that instance's own
 region, because a flavor identifier means nothing in another region. One flavor
 list is read per region in play, never one per instance.
 
 @returns VM entries with the unprefixed name, the lower-cased OVHcloud status,
 and the flavor's virtual CPUs and memory where the region still lists it
 
 @example
 ```ts
 const vms = await ovhList();
 ```
 */
export async function ovhList(): Promise<readonly VmInfo[]> {
  /**
   Instances mvm owns, of those the project holds.
   */
  const owned = await listMvmInstances();
  /**
   Regions the listed instances run in, each read once.
   */
  const regions = [...new Set(owned.map(function regionOf(instance,) {
    return instance.region;
  },),),];
  /**
   Each region's flavors, by region name, so an instance's flavor id resolves.
   */
  const flavorsByRegion = new Map(await Promise.all(regions.map(async function readRegion(region,) {
    return [region, await listFlavors({ region, },),] as const;
  },),),);
  return owned.map(function toVmInfo(instance,) {
    /**
     Flavor the instance names, when its region still lists it.
     */
    const flavor = flavorsByRegion.get(instance.region,)
      ?.find(function sameId(candidate,) {
        return candidate.id === instance.flavorId;
      },);
    return {
      name: instance.name
        .slice(VM_PREFIX.length,),
      state: instance.status
        .toLowerCase(),
      ...((flavor === undefined) ? {} : { memoryGiB: flavor.ram, }),
      ...((flavor === undefined) ? {} : { vcpus: flavor.vcpus, }),
    };
  },);
}

//endregion Destroy and list
