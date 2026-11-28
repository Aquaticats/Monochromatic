/**
 Reading and waiting on the instances of the one project.

 Every read here is scoped twice: to the project the credentials name, and to
 the instances whose name carries mvm's prefix. The schema gives an instance no
 label, so the prefix is the only ownership mark available, and it is applied
 before anything is listed, connected to, or deleted.

 @module
 */

import { wait, } from '@monochromatic-dev/async-time-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { VM_PREFIX, } from '../../config.ts';
import {
  getImage,
  getInstance,
  listInstances,
} from './api-resources.ts';
import {
  type OvhImage,
  type OvhInstance,
  IPV4_VERSION,
  INSTANCE_FAILED_STATUSES,
  INSTANCE_RUNNING_STATUS,
  PUBLIC_IP_TYPE,
} from './types.ts';

/**
 Logger root for mvm after removing the package log shim.

 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l, },);
 ```
 */
const l = tagged({ tag: 'mvm', },);

//region Constants

/**
 Delay between instance-status polls.
 */
const POLL_INTERVAL_MS = 3_000;

/**
 How long to wait for an instance to report itself running. Unmeasured against
 OVHcloud: taken from the Hetzner backend's action timeout and corrected once
 the billed test reports what a Canadian instance needs.
 */
const RUNNING_TIMEOUT_MS = 300_000;

/**
 The image status that means a snapshot can be booted.
 */
const USABLE_IMAGE_STATUS = 'active';

/**
 How long to wait for a snapshot to become usable before trying the create
 anyway. A clone whose snapshot is still being written fails in OVHcloud's own
 words, which is better than blocking on a status name this backend has no
 schema value for.
 */
const SNAPSHOT_TIMEOUT_MS = 120_000;

//endregion Constants

//region Ownership

/**
 Keeps the instances mvm owns: those whose name carries {@link VM_PREFIX}.
 Everything else in the project belongs to someone else and is never listed,
 connected to, or deleted.
 
 @param instances - every instance of the project
 
 @returns the instances whose name starts with the prefix
 
 @example
 ```ts
 mvmInstances({ instances }); // drops 'web-1' from a project that also holds it
 ```
 */
export function mvmInstances({
  instances,
}: {
  readonly instances: readonly OvhInstance[];
},): readonly OvhInstance[] {
  return instances.filter(function owned(instance,) {
    return instance.name
      .startsWith(VM_PREFIX,);
  },);
}

/**
 Resolves one mvm-owned instance by its unprefixed name.
 
 @param name - VM name without the mvm- prefix
 
 @returns the single matching instance
 
 @throws Error when no instance, or more than one, has that name
 
 @example
 ```ts
 const instance = await findMvmInstance({ name: 'dev-01' });
 ```
 */
export async function findMvmInstance({
  name,
}: {
  readonly name: string;
},): Promise<OvhInstance> {
  /**
   Full instance name; the exact match target.
   */
  const fullName = `${VM_PREFIX}${name}`;
  /**
   Owned instances whose name is exactly the full name.
   */
  const matches = mvmInstances({ instances: await listInstances(), },)
    .filter(function exactName(instance,) {
      return instance.name === fullName;
    },);
  /**
   Head plus tail so the single-match guarantee needs no non-null assertion.
   */
  const [match, ...rest] = matches;
  if (match === undefined) {
    throw new Error(`no mvm-managed OVHcloud instance named ${name}`,);
  }
  if (rest.length > 0) {
    throw new Error(
      `ambiguous: ${String(matches.length,)} mvm-managed OVHcloud instances named ${name}`,
    );
  }
  return match;
}

/**
 Lists the instances mvm owns.
 
 @returns the owned instances, in OVHcloud's order
 
 @example
 ```ts
 const owned = await listMvmInstances();
 ```
 */
export async function listMvmInstances(): Promise<readonly OvhInstance[]> {
  return mvmInstances({ instances: await listInstances(), },);
}

//endregion Ownership

//region Addresses

/**
 The public IPv4 of an instance, the address an SSH client reaches it on.
 
 @param instance - instance whose addresses are read
 
 @returns the public IPv4 address
 
 @throws Error when the instance has no public IPv4, naming what it does have
 
 @example
 ```ts
 const ip = publicIpv4({ instance });
 ```
 */
export function publicIpv4({
  instance,
}: {
  readonly instance: OvhInstance;
},): string {
  /**
   Public IPv4 the instance holds, when the API reports one.
   */
  const address = instance.ipAddresses
    .find(function isPublicIpv4(candidate,) {
      return (candidate.version === IPV4_VERSION) && (candidate.type === PUBLIC_IP_TYPE);
    },);
  if (address === undefined) {
    throw new Error(
      `instance ${instance.name} has no public IPv4 address (it reports ${
        instance.ipAddresses
          .map(function describe(candidate,) {
            return `v${String(candidate.version,)} ${candidate.type} ${candidate.ip}`;
          },)
          .join(', ',)
          || 'none'
      })`,
    );
  }
  return address.ip;
}

/**
 The account an SSH client logs in as on an instance, read from its image.
 
 @param instance - instance whose image is read
 
 @returns the image's `user`, `''` when the image names none
 
 @example
 ```ts
 const user = await instanceImageUser({ instance }); // 'ubuntu'
 ```
 */
export async function instanceImageUser({
  instance,
}: {
  readonly instance: OvhInstance;
},): Promise<string> {
  /**
   The image as the instance carries it, or as the project reports it.
   */
  const image: OvhImage = instance.image ?? await getImage({ imageId: instance.imageId, },);
  return image.user;
}

/**
 The account an SSH client logs in as on an instance, read from its image.
 
 @param instance - instance whose image is read
 
 @returns the image's `user`
 
 @throws Error when the image names no account
 
 @example
 ```ts
 const user = await instanceLoginUser({ instance }); // 'ubuntu'
 ```
 */
export async function instanceLoginUser({
  instance,
}: {
  readonly instance: OvhInstance;
},): Promise<string> {
  /**
   Account the image names, `''` when it names none.
   */
  const user = await instanceImageUser({ instance, },);
  if (user === '') {
    throw new Error(
      `the image of instance ${instance.name} names no login account to connect with, so mvm cannot know who to connect as`,
    );
  }
  return user;
}

//endregion Addresses

//region Waiting

/**
 Polls an instance until it reports itself running.
 
 @param instanceId - instance to poll
 
 @param name - VM name without the prefix, for messages
 
 @returns the instance as it stands once running
 
 @throws Error when the instance reaches a failed status or outlasts the timeout
 
 @example
 ```ts
 const running = await waitUntilRunning({ instanceId: 'abc', name: 'dev-01' });
 ```
 */
export async function waitUntilRunning({
  instanceId,
  name,
}: {
  readonly instanceId: string;
  readonly name: string;
},): Promise<OvhInstance> {
  /**
   Logger scoped to this wait so its lines are namespaced.
   */
  const rl = tagged({
    tag: waitUntilRunning.name,
    l,
  },);
  /**
   Deadline after which polling gives up.
   */
  const deadline = Date.now() + RUNNING_TIMEOUT_MS;
  while (Date.now() < deadline) {
    /**
     The instance as it stands now.
     */
    // oxlint-disable-next-line no-await-in-loop -- deliberate serial polling of one instance
    const instance = await getInstance({ instanceId, },);
    if (instance.status === INSTANCE_RUNNING_STATUS) {
      return instance;
    }
    if (INSTANCE_FAILED_STATUSES.has(instance.status,)) {
      throw new Error(
        `instance ${name} reached the OVHcloud status ${instance.status} instead of running`,
      );
    }
    rl.debug(`instance ${name} is ${instance.status}`,);
    // oxlint-disable-next-line no-await-in-loop -- deliberate serial polling of one instance
    await wait(POLL_INTERVAL_MS,);
  }
  throw new Error(
    `instance ${name} did not reach ${INSTANCE_RUNNING_STATUS} within ${
      String(RUNNING_TIMEOUT_MS,)
    }ms`,
  );
}

/**
 Waits for a snapshot image to become usable, then stops waiting.
 
 OVHcloud's image status values are not enumerated in the schema, so an
 unexpected value is not treated as a failure: the wait gives up at its deadline,
 says what it saw, and lets the create report OVHcloud's own answer.
 
 @param imageId - snapshot image to poll
 
 @example
 ```ts
 await waitForSnapshotReady({ imageId: 'abc' });
 ```
 */
export async function waitForSnapshotReady({
  imageId,
}: {
  readonly imageId: string;
},): Promise<void> {
  /**
   Logger scoped to this wait so its lines are namespaced.
   */
  const rl = tagged({
    tag: waitForSnapshotReady.name,
    l,
  },);
  /**
   Deadline after which the create is attempted regardless.
   */
  const deadline = Date.now() + SNAPSHOT_TIMEOUT_MS;
  while (Date.now() < deadline) {
    /**
     The image as it stands now.
     */
    // oxlint-disable-next-line no-await-in-loop -- deliberate serial polling of one image
    const image = await getImage({ imageId, },);
    if (image.status === USABLE_IMAGE_STATUS) {
      return;
    }
    // oxlint-disable-next-line no-await-in-loop -- deliberate serial polling of one image
    await wait(POLL_INTERVAL_MS,);
  }
  rl.info(
    `snapshot ${imageId} did not report the status "${USABLE_IMAGE_STATUS}" within ${
      String(SNAPSHOT_TIMEOUT_MS,)
    }ms; trying to boot it anyway`,
  );
}

//endregion Waiting
