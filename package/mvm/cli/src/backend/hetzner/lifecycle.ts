/**
 Hetzner server lifecycle: create (with location fallback), clone (snapshot),
 destroy, destroy-all, and list.
 
 Create tries each location in the fallback list, advancing past out-of-stock
 responses. Clone snapshots the source, provisions from the snapshot, and
 reaps the snapshot via an `await using` guard so it is never left billing.
 Single-server operations resolve through the label-scoped lookup so they can
 never touch an unrelated server.
 
 @module
 */
import { caughtValueText, } from '@monochromatic-dev/module-caught-value/ts';
import { tagged, } from '@monochromatic-dev/module-logger/ts';

import { VM_PREFIX, } from '../../config.ts';
import type { VmInfo, } from '../../list.ts';
import {
  HetznerApiError,
  waitForAction,
} from './api.ts';
import {
  createImage,
  createServer,
  deleteImage,
  deleteServer,
  getMvmServerByName,
  listImages,
  listMvmServers,
} from './api-resources.ts';
import {
  DEFAULT_IMAGE,
  MVM_LABEL_KEY,
  MVM_LABEL_SELECTOR,
  MVM_LABEL_VALUE,
  resolveLocations,
  serverTypeOverride,
  validateHetznerName,
} from './config.ts';
import { resolveHetznerImage, } from './images.ts';
import { resolveCheapestServerType, } from './server-types.ts';
import { ensureSshKeyId, } from './ssh-key.ts';
import { waitForSsh, } from './ssh.ts';
import type {
  HetznerAction,
  HetznerServer,
} from './types.ts';

/**
 Logger root for mvm after removing the package log shim.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l, },);
 ```
 */
const l = tagged({ tag: 'mvm', },);

//region Shared helpers

/**
 Hetzner error code returned when a server type is out of stock or not offered
 in a location.
 */
const OUT_OF_STOCK_CODE = 'resource_unavailable';

/**
 Created-server result paired with its boot action.
 */
type Provisioned = {
  readonly server: HetznerServer;
  readonly action: HetznerAction;
};

/**
 Tagged-logger surface used by the helpers below.
 */
type RunLogger = {
  readonly debug: (msg: string,) => void;
  readonly info: (msg: string,) => void;
};

/**
 Labels applied to every mvm-created Hetzner resource.
 
 @returns ownership labels including the OS marker
 
 @example
 ```ts
 mvmLabels(); // { mvm: 'true', 'mvm-os': 'linux' }
 ```
 */
function mvmLabels(): Readonly<Record<string, string>> {
  return {
    [MVM_LABEL_KEY]: MVM_LABEL_VALUE,
    'mvm-os': 'linux',
  };
}

/**
 Extracts the public IPv4 of a server or throws when none is attached.
 
 @param server - server resource
 
 @returns public IPv4 address
 
 @throws Error when the server has no usable public IPv4
 
 @example
 ```ts
 const ip = ipv4OrThrow(server);
 ```
 */
function ipv4OrThrow(server: HetznerServer,): string {
  /**
   Public IPv4 of the server, when one is attached.
   */
  const ip = server.public_net
    .ipv4
    ?.ip;
  if ((ip === undefined) || (ip === '')) {
    throw new Error(`server ${server.name} has no public IPv4`,);
  }
  return ip;
}

/**
 Creates a server, advancing through the location fallback list when a
 location is out of stock.
 
 @param image - image slug or numeric snapshot id to boot from
 
 @param fullName - full server name including the mvm- prefix
 
 @param locations - ordered location fallback list
 
 @param rl - logger for progress and fallback messages
 
 @param serverType - server type slug
 
 @param sshKeyId - SSH key id to inject
 
 @returns the created server and its boot action
 
 @throws Error when every location is out of stock ({@link HetznerApiError} instances with other codes propagate)
 
 @example
 ```ts
 const created = await createWithFallback({ fullName: 'mvm-dev', image: 'ubuntu-24.04', serverType: 'cx22', sshKeyId: 1, locations: ['fsn1', 'nbg1'], rl });
 ```
 */
export async function createWithFallback(
  {
    image,
    fullName,
    locations,
    rl,
    serverType,
    sshKeyId,
  }: {
    readonly image: number | string;
    readonly fullName: string;
    readonly locations: readonly string[];
    readonly rl: RunLogger;
    readonly serverType: string;
    readonly sshKeyId: number;
  },
): Promise<Provisioned> {
  for (const location of locations) {
    rl.info(`creating ${fullName} (${serverType}, image ${String(image,)}) in ${location}`,);
    try {
      // oxlint-disable-next-line no-await-in-loop -- sequential so out-of-stock can fall through to the next location
      return await createServer({
        image,
        labels: mvmLabels(),
        location,
        name: fullName,
        serverType,
        sshKeyId,
      },);
    }
    catch (error: unknown) {
      if ((error instanceof HetznerApiError) && (error.code === OUT_OF_STOCK_CODE)) {
        rl.info(`${location} unavailable (${error.code}), trying next location`,);
        continue;
      }
      throw error;
    }
  }
  throw new Error(
    `no capacity for ${serverType} in any of: ${locations.join(', ',)} (all out of stock)`,
  );
}

//endregion Shared helpers

//region Create

/**
 Creates and starts a Hetzner server, waiting for boot and SSH readiness.
 
 @param image - image shorthand or literal slug (defaults to ubuntu)
 
 @param location - per-invocation location or comma-separated fallback series
 
 @param name - VM name without the mvm- prefix
 
 @param serverType - per-invocation server type override
 
 @throws Error on invalid name, unsupported image, or exhausted locations
 
 @example
 ```ts
 await hetznerCreate({ name: 'dev-01' });
 await hetznerCreate({ name: 'big', serverType: 'cpx41', location: 'ash' });
 ```
 */
export async function hetznerCreate(
  {
    image,
    location,
    name,
    serverType,
  }: {
    readonly name: string;
    readonly image?: string;
    readonly serverType?: string;
    readonly location?: string;
  },
): Promise<void> {
  validateHetznerName(name,);
  /**
   Logger scoped to this create so steps are namespaced.
   */
  const rl = tagged({
    tag: hetznerCreate.name,
    l,
  },);
  /**
   SSH key id injected into the server so we can connect afterwards.
   */
  const sshKeyId = await ensureSshKeyId();
  /**
   Concrete Hetzner image slug resolved from the shorthand.
   */
  const imageSlug = await resolveHetznerImage({ shorthand: image ?? DEFAULT_IMAGE, },);
  /**
   Locations to try, in fallback order.
   */
  const locations = resolveLocations(location,);
  /**
   Explicit server-type override, or `''` to pick the cheapest type.
   */
  const explicitType = serverTypeOverride(serverType,);
  /**
   Server type to provision: the override, or the cheapest non-deprecated type
   offered in the target locations (the image is by name, so any architecture
   matches).
   */
  const serverTypeName = (explicitType !== '')
    ? explicitType
    : await resolveCheapestServerType({ locations, },);
  /**
   Created server plus boot action, after location fallback.
   */
  const created = await createWithFallback({
    fullName: `${VM_PREFIX}${name}`,
    image: imageSlug,
    locations,
    rl,
    serverType: serverTypeName,
    sshKeyId,
  },);
  await waitForAction({ id: created.action
    .id, },);
  /**
   Public IPv4 to probe for SSH readiness.
   */
  const ip = ipv4OrThrow(created.server,);
  await waitForSsh({ ip, },);
  rl.info(
    `VM ${name} is ready at ${ip}. Connect with: mvm --backend hetzner shell ${name}`,
  );
}

//endregion Create

//region Clone

/**
 Best-effort image deletion that logs rather than throwing.
 
 @param id - snapshot image id
 
 @param rl - logger for cleanup-failure messages
 
 @example
 ```ts
 await safeDeleteImage({ id: 1, rl });
 ```
 */
async function safeDeleteImage(
  {
    id,
    rl,
  }: {
    readonly id: number;
    readonly rl: RunLogger;
  },
): Promise<void> {
  rl.debug(`deleting intermediate snapshot ${String(id,)}`,);
  try {
    await deleteImage({ id, },);
  }
  catch (error: unknown) {
    rl.info(
      `snapshot ${String(id,)} cleanup failed: ${
        caughtValueText(error,)
      }`,
    );
  }
}

/**
 Provisions the destination server from a snapshot, reaping the snapshot on
 scope exit (success or failure) before the caller waits on SSH.
 
 @param architecture - source architecture the snapshot requires the type to match
 
 @param fullName - full destination server name
 
 @param imageId - snapshot image id to boot from and then delete
 
 @param rl - logger for progress
 
 @param snapshotActionId - action to await before the snapshot is usable
 
 @returns the created destination server and its boot action
 
 @example
 ```ts
 const created = await provisionFromSnapshot({ architecture: 'x86', fullName: 'mvm-dev-02', imageId: 9, snapshotActionId: 7, rl });
 ```
 */
async function provisionFromSnapshot(
  {
    architecture,
    fullName,
    imageId,
    rl,
    snapshotActionId,
  }: {
    readonly architecture: string;
    readonly fullName: string;
    readonly imageId: number;
    readonly rl: RunLogger;
    readonly snapshotActionId: number;
  },
): Promise<Provisioned> {
  /**
   Guard reaping the snapshot when this scope exits, so it never leaks even if
   provisioning throws (`using` cleanup, never `try...finally`).
   */
  await using _snapshotCleanup = {
    async [Symbol.asyncDispose](): Promise<void> {
      await safeDeleteImage({
        id: imageId,
        rl,
      },);
    },
  };
  await waitForAction({ id: snapshotActionId, },);
  /**
   SSH key id injected into the destination server.
   */
  const sshKeyId = await ensureSshKeyId();
  /**
   Locations to try, in fallback order.
   */
  const locations = resolveLocations();
  /**
   Explicit server-type override, or `''` to pick the cheapest type.
   */
  const explicitType = serverTypeOverride();
  /**
   Server type for the clone: the override, or the cheapest non-deprecated
   type of the snapshot's architecture (the snapshot image is arch-specific).
   */
  const serverTypeName = (explicitType !== '')
    ? explicitType
    : await resolveCheapestServerType({
      architecture,
      locations,
    },);
  /**
   Destination server provisioned from the snapshot image.
   */
  const created = await createWithFallback({
    fullName,
    image: imageId,
    locations,
    rl,
    serverType: serverTypeName,
    sshKeyId,
  },);
  await waitForAction({ id: created.action
    .id, },);
  return created;
}

/**
 Clones a Hetzner server by snapshotting the source and provisioning a new
 server from that snapshot. The source is snapshotted live (not shut down),
 so a running source is not disrupted but the snapshot may catch in-flight
 disk state.
 
 @param destination - destination VM name without the mvm- prefix
 
 @param source - source VM name without the mvm- prefix
 
 @throws Error on invalid name, missing source, or provisioning failure
 
 @example
 ```ts
 await hetznerClone({ destination: 'dev-02', source: 'dev-01' });
 ```
 */
export async function hetznerClone(
  {
    destination,
    source,
  }: {
    readonly destination: string;
    readonly source: string;
  },
): Promise<void> {
  validateHetznerName(destination,);
  /**
   Logger scoped to this clone so steps are namespaced.
   */
  const rl = tagged({
    tag: hetznerClone.name,
    l,
  },);
  /**
   Source server, resolved label-scoped so an unrelated server is never used.
   */
  const src = await getMvmServerByName({ name: source, },);
  rl.info(`snapshotting ${source} to clone into ${destination}`,);
  /**
   Snapshot action and image; the image is reaped inside {@link provisionFromSnapshot}.
   */
  const snapshot = await createImage({
    description: `mvm clone ${source} -> ${destination}`,
    labels: mvmLabels(),
    serverId: src.id,
  },);
  /**
   Destination server, created from the snapshot which is then deleted. The
   server type must match the source architecture the snapshot carries.
   */
  const created = await provisionFromSnapshot({
    architecture: src.server_type
      .architecture,
    fullName: `${VM_PREFIX}${destination}`,
    imageId: snapshot.image
      .id,
    rl,
    snapshotActionId: snapshot.action
      .id,
  },);
  /**
   Public IPv4 to probe for SSH readiness.
   */
  const ip = ipv4OrThrow(created.server,);
  await waitForSsh({ ip, },);
  rl.info(
    `VM ${destination} is ready at ${ip} (cloned from ${source}).`,
  );
}

//endregion Clone

//region Destroy and list

/**
 Destroys a single mvm-managed server by name.
 
 @param name - VM name without the mvm- prefix
 
 @throws Error when no single mvm-managed server matches
 
 @example
 ```ts
 await hetznerDestroy({ name: 'dev-01' });
 ```
 */
export async function hetznerDestroy({ name, }: { readonly name: string; },): Promise<void> {
  validateHetznerName(name,);
  /**
   Logger scoped to this destroy so steps are namespaced.
   */
  const rl = tagged({
    tag: hetznerDestroy.name,
    l,
  },);
  /**
   Target server, resolved label-scoped and exact so unrelated servers are safe.
   */
  const server = await getMvmServerByName({ name, },);
  rl.info(`destroying ${name} (id ${String(server.id,)})`,);
  await deleteServer({ id: server.id, },);
  rl.info(`VM ${name} destroyed`,);
}

/**
 Destroys every mvm-managed server, then sweeps mvm-managed snapshot images as
 a backstop against leaked clone snapshots. Both are scoped by the ownership
 label selector, never by name prefix.
 
 @throws Error when a delete fails
 
 @example
 ```ts
 await hetznerDestroyAll();
 ```
 */
export async function hetznerDestroyAll(): Promise<void> {
  /**
   Logger scoped to this bulk destroy so steps are namespaced.
   */
  const rl = tagged({
    tag: hetznerDestroyAll.name,
    l,
  },);
  /**
   Every mvm-labelled server, the only deletion scope.
   */
  const servers = await listMvmServers();
  if (servers.length === 0) {
    rl.info('no VMs to destroy',);
  }
  else {
    rl.info(`destroying all ${String(servers.length,)} VMs`,);
    for (const server of servers) {
      // oxlint-disable-next-line no-await-in-loop -- sequential to avoid hammering the API
      await deleteServer({ id: server.id, },);
    }
  }

  /**
   mvm-labelled snapshot images; swept so an interrupted clone cannot leak one.
   */
  const images = await listImages({
    labelSelector: MVM_LABEL_SELECTOR,
    type: 'snapshot',
  },);
  for (const image of images) {
    // oxlint-disable-next-line no-await-in-loop -- sequential to avoid hammering the API
    await deleteImage({ id: image.id, },);
  }
}

/**
 Lists mvm-managed servers (by ownership label) with their state.
 
 @returns VM entries with the unprefixed name and Hetzner status
 
 @example
 ```ts
 const vms = await hetznerList();
 ```
 */
export async function hetznerList(): Promise<readonly VmInfo[]> {
  return (await listMvmServers()).map(function toVmInfo(server,) {
    return {
      name: server.name
        .startsWith(VM_PREFIX,)
        ? server.name
          .slice(VM_PREFIX.length,)
        : server.name,
      state: server.status,
    };
  },);
}

//endregion Destroy and list
