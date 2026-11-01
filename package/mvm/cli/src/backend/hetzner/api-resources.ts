/**
 Typed Hetzner Cloud resource helpers for servers, SSH keys, and images.
 
 Thin wrappers over the {@link hcloud} request core and {@link fetchAllPages}
 pagination helper. Listing and bulk operations filter by the mvm ownership
 label so they can never touch unrelated resources.
 
 @module
 */

import { VM_PREFIX, } from '../../config.ts';
import {
  fetchAllPages,
  hcloud,
} from './api.ts';
import { MVM_LABEL_SELECTOR, } from './config.ts';
import type {
  HetznerAction,
  HetznerImage,
  HetznerServer,
  HetznerServerType,
  HetznerSshKey,
} from './types.ts';

//region Servers

/**
 Creates a server in a single location.
 
 @param image - image slug or id to boot from
 
 @param labels - labels to attach (must include the mvm ownership label)
 
 @param location - location code to provision in
 
 @param name - full server name including the mvm- prefix
 
 @param serverType - server type slug
 
 @param sshKeyId - numeric id of the SSH key to inject
 
 @returns created server and the boot action to poll
 
 @throws {@link HetznerApiError} with code `resource_unavailable` when out of stock
 
 @example
 ```ts
 const { server, action } = await createServer({ name: 'mvm-dev', serverType: 'cx22', location: 'fsn1', image: 'ubuntu-24.04', sshKeyId: 42, labels: { mvm: 'true' } });
 ```
 */
export function createServer(
  {
    image,
    labels,
    location,
    name,
    serverType,
    sshKeyId,
  }: {
    readonly image: number | string;
    readonly labels: Readonly<Record<string, string>>;
    readonly location: string;
    readonly name: string;
    readonly serverType: string;
    readonly sshKeyId: number;
  },
): Promise<{
  readonly server: HetznerServer;
  readonly action: HetznerAction
}> {
  return hcloud<{
    readonly server: HetznerServer;
    readonly action: HetznerAction
  }>({
    method: 'POST',
    path: '/servers',
    body: {
      image,
      labels,
      location,
      name,
      server_type: serverType,
      ssh_keys: [sshKeyId,],
      start_after_create: true,
    },
  },);
}

/**
 Lists every mvm-managed server (filtered by the ownership label selector).
 
 @returns all mvm-labelled servers
 
 @example
 ```ts
 const servers = await listMvmServers();
 ```
 */
export function listMvmServers(): Promise<readonly HetznerServer[]> {
  return fetchAllPages<HetznerServer>({
    key: 'servers',
    path: `/servers?label_selector=${encodeURIComponent(MVM_LABEL_SELECTOR,)}`,
  },);
}

/**
 Resolves a single mvm-managed server by its unprefixed name.
 Filters by the ownership label and exact full name, so it can never operate
 on an unrelated server that merely shares the `mvm-<name>` name.
 
 @param name - VM name without the mvm- prefix
 
 @returns the matching server
 
 @throws Error when zero or more than one mvm-labelled server matches
 
 @example
 ```ts
 const server = await getMvmServerByName({ name: 'dev-01' });
 ```
 */
export async function getMvmServerByName(
  { name, }: { readonly name: string; },
): Promise<HetznerServer> {
  /**
   Full server name including the mvm- prefix; the exact match target.
   */
  const fullName = `${VM_PREFIX}${name}`;
  /**
   mvm-labelled servers whose name exactly matches; the API name filter plus a
   defensive exact compare guard against substring or unrelated matches.
   */
  const matches = (await fetchAllPages<HetznerServer>({
    key: 'servers',
    path: `/servers?label_selector=${encodeURIComponent(MVM_LABEL_SELECTOR,)}&name=${
      encodeURIComponent(fullName,)
    }`,
  },)).filter(function exactName(server,) {
    return server.name === fullName;
  },);
  /**
   Destructured head plus tail so the single-match guarantee is enforced
   without a non-null assertion.
   */
  const [match, ...rest] = matches;
  if (match === undefined) {
    throw new Error(`no mvm-managed Hetzner server named ${name}`,);
  }
  if (rest.length > 0) {
    throw new Error(
      `ambiguous: ${String(matches.length,)} mvm-managed Hetzner servers named ${name}`,
    );
  }
  return match;
}

/**
 Deletes a server by id.
 
 @param id - numeric server id
 
 @example
 ```ts
 await deleteServer({ id: 576675 });
 ```
 */
export async function deleteServer({ id, }: { readonly id: number; },): Promise<void> {
  await hcloud<unknown>({
    method: 'DELETE',
    path: `/servers/${String(id,)}`,
  },);
}

/**
 Lists every server type (plan), including deprecated ones.
 
 @returns all server types
 
 @example
 ```ts
 const types = await listServerTypes();
 ```
 */
export function listServerTypes(): Promise<readonly HetznerServerType[]> {
  return fetchAllPages<HetznerServerType>({
    key: 'server_types',
    path: '/server_types',
  },);
}

//endregion Servers

//region SSH keys

/**
 Lists every SSH key in the project.
 
 @returns all SSH keys
 
 @example
 ```ts
 const keys = await listSshKeys();
 ```
 */
export function listSshKeys(): Promise<readonly HetznerSshKey[]> {
  return fetchAllPages<HetznerSshKey>({
    key: 'ssh_keys',
    path: '/ssh_keys',
  },);
}

/**
 Uploads an SSH public key.
 
 @param name - key name within the project
 
 @param publicKey - full public key line
 
 @returns the created key
 
 @example
 ```ts
 const key = await createSshKey({ name: 'mvm-abc', publicKey: 'ssh-ed25519 AAAA...' });
 ```
 */
export async function createSshKey(
  {
    name,
    publicKey,
  }: {
    readonly name: string;
    readonly publicKey: string;
  },
): Promise<HetznerSshKey> {
  return (await hcloud<{ readonly ssh_key: HetznerSshKey; }>({
    method: 'POST',
    path: '/ssh_keys',
    body: {
      name,
      public_key: publicKey,
    },
  },)).ssh_key;
}

//endregion SSH keys

//region Images

/**
 Lists images, optionally filtered by type and ownership label.
 
 @param labelSelector - optional label selector
 
 @param type - optional image type (e.g. `system`, `snapshot`)
 
 @returns matching images
 
 @example
 ```ts
 const systemImages = await listImages({ type: 'system' });
 ```
 */
export function listImages(
  {
    labelSelector,
    type,
  }: {
    readonly labelSelector?: string;
    readonly type?: string;
  } = {},
): Promise<readonly HetznerImage[]> {
  /**
   Query parameters assembled from the optional filters.
   */
  const params = new URLSearchParams();
  if (type !== undefined) {
    params.set(
      'type',
      type,
    );
  }
  if (labelSelector !== undefined) {
    params.set(
      'label_selector',
      labelSelector,
    );
  }
  /**
   Query string with a leading `?` only when filters are present.
   */
  const query = (params.toString() === '') ? '' : `?${params.toString()}`;
  return fetchAllPages<HetznerImage>({
    key: 'images',
    path: `/images${query}`,
  },);
}

/**
 Snapshots a server into an independent image.
 
 @param description - human-readable snapshot description
 
 @param labels - labels to attach (must include the mvm ownership label)
 
 @param serverId - source server id
 
 @returns the snapshot action to poll and the created image
 
 @example
 ```ts
 const { action, image } = await createImage({ serverId: 1, description: 'clone of dev-01', labels: { mvm: 'true' } });
 ```
 */
export function createImage(
  {
    description,
    labels,
    serverId,
  }: {
    readonly description: string;
    readonly labels: Readonly<Record<string, string>>;
    readonly serverId: number;
  },
): Promise<{
  readonly action: HetznerAction;
  readonly image: HetznerImage
}> {
  return hcloud<{
    readonly action: HetznerAction;
    readonly image: HetznerImage
  }>({
    method: 'POST',
    path: `/servers/${String(serverId,)}/actions/create_image`,
    body: {
      description,
      labels,
      type: 'snapshot',
    },
  },);
}

/**
 Deletes an image (snapshot) by id.
 
 @param id - numeric image id
 
 @example
 ```ts
 await deleteImage({ id: 12345 });
 ```
 */
export async function deleteImage({ id, }: { readonly id: number; },): Promise<void> {
  await hcloud<unknown>({
    method: 'DELETE',
    path: `/images/${String(id,)}`,
  },);
}

//endregion Images
