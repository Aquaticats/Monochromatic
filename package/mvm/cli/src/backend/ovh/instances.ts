/**
 Reading and waiting on the instances of the one project.

 Every read here is scoped twice: to the project the credentials name, and to
 the instances whose name carries mvm's prefix. The schema gives an instance no
 label, so the prefix is the only ownership mark available, and it is applied
 before anything is listed, connected to, or deleted.

 @module
 */

import { wait, } from '@monochromatic-dev/async-time-core/ts';
import {
  tagged,
  type Logger,
} from '@monochromatic-dev/logger-core/ts';

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
  INSTANCE_DELETED_STATUS,
  INSTANCE_DELETING_STATUS,
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

/**
 How long to wait for an accepted delete to take the machine out of the
 project's listing.

 Measured on 2026-11-28 against the one project: a delete OVHcloud accepted at
 11:40:42.648Z was listed as `DELETED` at 11:40:54.346Z, 11.7s later, and a
 delete accepted at 11:52:45.307Z was still `DELETING` at 11:53:01.246Z, 15.9s
 later. Five minutes is more than an order of magnitude past both readings,
 equals the wait a create already gives for a machine to start, and costs one
 read of the project's listing every {@link POLL_INTERVAL_MS}.
 */
const DELETE_TIMEOUT_MS = 300_000;

//endregion Constants

//region Ownership

/**
 Keeps the instances mvm owns: those whose name carries {@link VM_PREFIX} and
 that the project has not deleted.
 Everything else in the project belongs to someone else and is never listed,
 connected to, or deleted.
 An instance the project lists as {@link INSTANCE_DELETED_STATUS} is gone:
 counting it would report a machine that runs nowhere, and matching it would
 make a name ambiguous as soon as a new machine takes it.
 An instance the project lists as {@link INSTANCE_DELETING_STATUS} is not gone:
 it is still billed, so it stays one of mvm's machines, it counts against any
 limit on how many run at once, and a lookup by its name matches it.
 
 @param instances - every instance of the project
 
 @returns the instances whose name starts with the prefix and that still exist
 
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
  return instances.filter(function ownedAndNotDeleted(instance,) {
    return (instance.status !== INSTANCE_DELETED_STATUS)
      && instance.name
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

/**
 Refuses an instance the project is still deleting.

 A machine under {@link INSTANCE_DELETING_STATUS} still bills and still answers
 this backend's listing, so every lookup by name matches it; reaching it over
 SSH cannot work, so the operations that would reach it say why instead.
 
 @param instance - instance the lookup resolved
 
 @param name - VM name without the mvm- prefix, named in the message
 
 @throws Error when the project lists the instance as deleting
 
 @example
 ```ts
 assertNotDeleting({ instance, name: 'dev-01' });
 ```
 */
export function assertNotDeleting({
  instance,
  name,
}: {
  readonly instance: OvhInstance;
  readonly name: string;
},): void {
  if (instance.status !== INSTANCE_DELETING_STATUS) {
    return;
  }
  throw new Error(
    `instance ${name} is being deleted: OVHcloud lists it as ${instance.status}, so there is nothing to connect to and nothing to run a command in. Check the listing with mvm --backend ovh list, and mvm --backend ovh destroy ${name} waits until the project stops listing it.`,
  );
}

/**
 Refuses a name the project is still deleting.

 OVHcloud does not make an instance name unique, so a create under a name the
 project still lists as deleting would leave two machines whose names mvm
 cannot tell apart, and every later lookup by that name would refuse as
 ambiguous. The two settings that create a machine, create and clone, ask this
 first.
 
 @param name - VM name without the mvm- prefix
 
 @throws Error when the project lists a machine of that name as deleting
 
 @example
 ```ts
 await assertNameIsNotDeleting({ name: 'dev-01' });
 ```
 */
export async function assertNameIsNotDeleting({
  name,
}: {
  readonly name: string;
},): Promise<void> {
  /**
   Full instance name; the exact match target.
   */
  const fullName = `${VM_PREFIX}${name}`;
  /**
   The listed machine of that name, when the project still lists one as deleting.
   */
  const deleting = (await listInstances())
    .find(function sameNameAndDeleting(instance,) {
      return (instance.name === fullName) && (instance.status === INSTANCE_DELETING_STATUS);
    },);
  if (deleting === undefined) {
    return;
  }
  throw new Error(
    `OVHcloud is still deleting ${fullName}: the project lists it as ${deleting.status}, and OVHcloud bills an instance until it is deleted. Wait for it to leave the listing (mvm --backend ovh list), or delete it again (mvm --backend ovh destroy ${name}), before creating another machine with that name.`,
  );
}

//endregion Ownership

//region Shapes

/**
 A machine a delete has been accepted for, and the name its messages use.
 */
export type DeletedInstance = {
  /**
   Instance identifier, the value the project's listing carries.
   */
  readonly id: string;
  /**
   VM name without the mvm- prefix, named in the messages.
   */
  readonly name: string;
};

/**
 One machine the project still lists after its delete was accepted, with the
 status the last read gave it.
 */
export type StillDeleting = {
  /**
   VM name without the mvm- prefix.
   */
  readonly name: string;
  /**
   Status the project's listing gave the machine at the last read.
   */
  readonly status: string;
};

//endregion Shapes

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
 Says why a wait for a running instance gave up, naming where the instance stood,
 so a reader can tell an instance still building from one that never answered.

 @param lastStatus - status the last poll read, or an empty string when none was read

 @param name - VM name without the prefix, for the message

 @returns the sentence the wait throws

 @example
 ```ts
 runningTimeoutMessage({ lastStatus: 'BUILD', name: 'dev-01' });
 ```
 */
export function runningTimeoutMessage({
  lastStatus,
  name,
}: {
  readonly lastStatus: string;
  readonly name: string;
},): string {
  /**
   What the wait last saw of the instance.
   */
  const stood = (lastStatus === '')
    ? 'no status was read from OVHcloud in that time'
    : `its last status was ${lastStatus}`;
  return `instance ${name} did not reach ${INSTANCE_RUNNING_STATUS} within ${
    String(RUNNING_TIMEOUT_MS,)
  }ms; ${stood}`;
}

/**
 Polls an instance until it reports itself running, saying each status it meets
 once.
 
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
  /**
   Statuses the polls read, each kept once when it changes, so a status is said
   once and the give-up can name where the instance stood.
   */
  const seen: string[] = [];
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
    if (instance.status !== seen.at(-1,)) {
      rl.info(`instance ${name} is ${instance.status}`,);
      seen.push(instance.status,);
    }
    // oxlint-disable-next-line no-await-in-loop -- deliberate serial polling of one instance
    await wait(POLL_INTERVAL_MS,);
  }
  throw new Error(
    runningTimeoutMessage({
      lastStatus: seen.at(-1,) ?? '',
      name,
    },),
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

/**
 Says why a wait for accepted deletes gave up, naming every machine still
 listed, where each stood, what that costs and both commands that answer it.
 
 @param pending - machines the project still lists, each with its last status
 
 @param timeoutMs - how long the wait had
 
 @returns the sentence the wait throws
 
 @example
 ```ts
 destructionTimeoutMessage({ pending: [{ name: 'dev-01', status: 'DELETING' }], timeoutMs: 300_000 });
 ```
 */
export function destructionTimeoutMessage({
  pending,
  timeoutMs,
}: {
  readonly pending: readonly StillDeleting[];
  readonly timeoutMs: number;
},): string {
  /**
   Machine names, in the order the wait was given them.
   */
  const names = pending.map(function nameOf(entry,) {
    return entry.name;
  },);
  /**
   Where each machine stood at the last read.
   */
  const listed = pending.map(function nameAndStatus(entry,) {
    return `${entry.name} as ${entry.status}`;
  },);
  /**
   One command per machine, so a reader can repeat any of them.
   */
  const commands = pending.map(function destroyCommand(entry,) {
    return `mvm --backend ovh destroy ${entry.name}`;
  },);
  return `deleting ${names.join(', ',)} did not finish within ${
    String(timeoutMs,)
  }ms: OVHcloud still lists ${listed.join(', ',)}. OVHcloud bills an instance until it is deleted, so every machine the project still lists is still being paid for. Check the listing with mvm --backend ovh list, and delete again with ${commands.join(', ',)}.`;
}

/**
 Waits for accepted deletes to take their machines out of the project's
 listing, saying each status once.

 A machine is gone when the project stops listing it or lists it as
 {@link INSTANCE_DELETED_STATUS}: only then does nothing bill for it and is its
 name free. A machine the project still lists as {@link INSTANCE_DELETING_STATUS}
 is still billed, so its status is said once and the wait keeps reading.
 
 @param instances - machines whose deletes OVHcloud has accepted
 
 @param pollIntervalMs - delay between reads of the listing, the create wait's by default
 
 @param rl - logger for the status lines, a module one by default
 
 @param timeoutMs - how long to read before giving up, {@link DELETE_TIMEOUT_MS} by default
 
 @throws Error from {@link destructionTimeoutMessage} when a machine is still listed at the bound
 
 @example
 ```ts
 await waitUntilDeleted({ instances: [{ id: 'abc', name: 'dev-01' }] });
 ```
 */
export async function waitUntilDeleted({
  instances,
  pollIntervalMs = POLL_INTERVAL_MS,
  rl,
  timeoutMs = DELETE_TIMEOUT_MS,
}: {
  readonly instances: readonly DeletedInstance[];
  readonly pollIntervalMs?: number;
  readonly rl?: Logger;
  readonly timeoutMs?: number;
},): Promise<void> {
  /**
   Logger scoped to this wait so its lines are namespaced.
   */
  const logger = rl ?? tagged({
    tag: waitUntilDeleted.name,
    l,
  },);
  /**
   Deadline after which the wait gives up and names what is still listed.
   */
  const deadline = Date.now() + timeoutMs;
  /**
   Statuses already said, per instance, so a status met twice is said once.
   */
  const said = new Map<string, string>();
  /**
   Machines the last read of the listing still held, by instance identifier.
   */
  const stillListed = new Map<string, StillDeleting>();
  do {
    /**
     The project's listing as this read found it.
     */
    // oxlint-disable-next-line no-await-in-loop -- deliberate serial reading of one project's listing
    const listed = await listInstances();
    /**
     The listing keyed by instance identifier, which is what a delete names.
     */
    const byId = new Map(listed.map(function byIdentifier(instance,) {
      return [
        instance.id,
        instance,
      ] as const;
    },),);
    stillListed.clear();
    for (const target of instances) {
      /**
       The machine as this read found it, absent once the project dropped it.
       */
      const current = byId.get(target.id,);
      if ((current === undefined) || (current.status === INSTANCE_DELETED_STATUS)) {
        continue;
      }
      if (current.status !== said.get(target.id,)) {
        logger.info(`instance ${target.name} is ${current.status}`,);
        said.set(
          target.id,
          current.status,
        );
      }
      stillListed.set(
        target.id,
        {
          name: target.name,
          status: current.status,
        },
      );
    }
    if (stillListed.size === 0) {
      return;
    }
    // oxlint-disable-next-line no-await-in-loop -- deliberate serial reading of one project's listing
    await wait(pollIntervalMs,);
  } while (Date.now() < deadline);
  throw new Error(
    destructionTimeoutMessage({
      pending: [...stillListed.values(),],
      timeoutMs,
    },),
  );
}

//endregion Waiting
