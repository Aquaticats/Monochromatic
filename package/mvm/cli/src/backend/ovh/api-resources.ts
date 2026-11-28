/**
 Typed OVHcloud cloud API calls for instances, flavors, images, SSH keys, snapshots and regions.

 Thin wrappers over {@link ovhRequest}. Every path is scoped to the one project
 {@link PROJECT_ID_ENV} names, and the project identifier is taken from the
 environment per call rather than captured at module load, so one process can
 never carry another project's identifier into a request.

 The schema shows no label on an instance, so listing returns everything the
 project holds and the lifecycle filters by mvm's name prefix.

 @module
 */

import { requireCredentials, } from './config.ts';
import { ovhRequest, } from './api.ts';
import type {
  OvhFlavor,
  OvhImage,
  OvhInstance,
  OvhInstanceSnapshot,
  OvhQuotas,
  OvhRegion,
  OvhSshKey,
  OvhSshKeyDetail,
} from './types.ts';

//region Paths

/**
 Version and resource prefix every cloud path starts with, from the schema's
 `basePath` (`https://ca.api.ovh.com/v1`) and `resourcePath` (`/cloud`).
 */
const CLOUD_PREFIX = '/v1/cloud/project';

/**
 Builds one project-scoped path.
 
 @param suffix - resource path below the project, beginning with `/`
 
 @returns path from the API root
 
 @example
 ```ts
 projectPath('/instance'); // '/v1/cloud/project/<the project>/instance'
 ```
 */
function projectPath(suffix: string,): string {
  return `${CLOUD_PREFIX}/${encodeURIComponent(requireCredentials()
    .projectId,)}${suffix}`;
}

//endregion Paths

//region Instances

/**
 Lists every instance of the project, from `GET /cloud/project/{serviceName}/instance`.
 
 @returns all instances, in OVHcloud's order
 
 @example
 ```ts
 const instances = await listInstances();
 ```
 */
export function listInstances(): Promise<readonly OvhInstance[]> {
  return ovhRequest<readonly OvhInstance[]>({
    method: 'GET',
    path: projectPath('/instance',),
  },);
}

/**
 Reads one instance, from `GET /cloud/project/{serviceName}/instance/{instanceId}`.
 
 @param instanceId - instance identifier
 
 @returns the instance as it stands now
 
 @example
 ```ts
 const instance = await getInstance({ instanceId: 'abc' });
 ```
 */
export function getInstance({
  instanceId,
}: {
  readonly instanceId: string;
},): Promise<OvhInstance> {
  return ovhRequest<OvhInstance>({
    method: 'GET',
    path: projectPath(`/instance/${encodeURIComponent(instanceId,)}`,),
  },);
}

/**
 Creates one instance, from `POST /cloud/project/{serviceName}/instance`.
 
 The body carries `monthlyBilling: false`. The schema marks the field optional
 and nullable and states no default, so an absent field leaves the billing
 mode to OVHcloud; sending `false` states hourly billing outright, which is the
 mode billed by the second, and no other value is ever sent.
 
 @param flavorId - flavor identifier the instance runs
 
 @param imageId - image to boot from, a system image or a snapshot
 
 @param name - full instance name including the mvm- prefix
 
 @param region - region to create in
 
 @param sshKeyId - identifier of the uploaded public key to inject
 
 @returns the created instance, as the API reports it at creation time
 
 @example
 ```ts
 const instance = await createInstance({ flavorId: 'b3-8', imageId: 'img', name: 'mvm-dev', region: 'BHS5', sshKeyId: 'key' });
 ```
 */
export function createInstance({
  flavorId,
  imageId,
  name,
  region,
  sshKeyId,
}: {
  readonly flavorId: string;
  readonly imageId: string;
  readonly name: string;
  readonly region: string;
  readonly sshKeyId: string;
},): Promise<OvhInstance> {
  return ovhRequest<OvhInstance>({
    method: 'POST',
    path: projectPath('/instance',),
    body: {
      flavorId,
      imageId,
      monthlyBilling: false,
      name,
      region,
      sshKeyId,
    },
  },);
}

/**
 Deletes one instance, from `DELETE /cloud/project/{serviceName}/instance/{instanceId}`.
 This is the only call that stops an instance being billed.
 
 @param instanceId - instance identifier
 
 @example
 ```ts
 await deleteInstance({ instanceId: 'abc' });
 ```
 */
export function deleteInstance({
  instanceId,
}: {
  readonly instanceId: string;
},): Promise<void> {
  return ovhRequest<void>({
    method: 'DELETE',
    path: projectPath(`/instance/${encodeURIComponent(instanceId,)}`,),
  },);
}

//endregion Instances

//region Flavors and images

/**
 Lists the flavors offered in one region, from `GET /cloud/project/{serviceName}/flavor`.
 
 @param region - region to list for
 
 @returns every flavor the region offers, in OVHcloud's order
 
 @example
 ```ts
 const flavors = await listFlavors({ region: 'BHS5' });
 ```
 */
export function listFlavors({
  region,
}: {
  readonly region: string;
},): Promise<readonly OvhFlavor[]> {
  return ovhRequest<readonly OvhFlavor[]>({
    method: 'GET',
    path: `${projectPath('/flavor',)}?region=${encodeURIComponent(region,)}`,
  },);
}

/**
 Lists the images offered in one region, from `GET /cloud/project/{serviceName}/image`.
 
 @param region - region to list for
 
 @returns every image the region offers, in OVHcloud's order
 
 @example
 ```ts
 const images = await listImages({ region: 'BHS5' });
 ```
 */
export function listImages({
  region,
}: {
  readonly region: string;
},): Promise<readonly OvhImage[]> {
  return ovhRequest<readonly OvhImage[]>({
    method: 'GET',
    path: `${projectPath('/image',)}?region=${encodeURIComponent(region,)}`,
  },);
}

/**
 Reads one image, from `GET /cloud/project/{serviceName}/image/{imageId}`.
 Used when an instance's own copy of its image carries no login user.
 
 @param imageId - image identifier, a system image or a snapshot
 
 @returns the image
 
 @example
 ```ts
 const image = await getImage({ imageId: 'abc' });
 ```
 */
export function getImage({
  imageId,
}: {
  readonly imageId: string;
},): Promise<OvhImage> {
  return ovhRequest<OvhImage>({
    method: 'GET',
    path: projectPath(`/image/${encodeURIComponent(imageId,)}`,),
  },);
}

//endregion Flavors and images

//region Regions and quota

/**
 Lists the region names of the project, from `GET /cloud/project/{serviceName}/region`.
 The names are bare strings; the country and status come from {@link getRegion}.
 
 @returns region names, in OVHcloud's order
 
 @example
 ```ts
 const names = await listRegionNames();
 ```
 */
export function listRegionNames(): Promise<readonly string[]> {
  return ovhRequest<readonly string[]>({
    method: 'GET',
    path: projectPath('/region',),
  },);
}

/**
 Reads one region, from `GET /cloud/project/{serviceName}/region/{regionName}`.
 
 @param regionName - region name
 
 @returns the region's country, status and availability zones
 
 @example
 ```ts
 const region = await getRegion({ regionName: 'BHS5' });
 ```
 */
export function getRegion({
  regionName,
}: {
  readonly regionName: string;
},): Promise<OvhRegion> {
  return ovhRequest<OvhRegion>({
    method: 'GET',
    path: projectPath(`/region/${encodeURIComponent(regionName,)}`,),
  },);
}

/**
 Lists the quotas of every region, from `GET /cloud/project/{serviceName}/quota`.
 
 @returns one entry per region
 
 @example
 ```ts
 const quotas = await listQuotas();
 ```
 */
export function listQuotas(): Promise<readonly OvhQuotas[]> {
  return ovhRequest<readonly OvhQuotas[]>({
    method: 'GET',
    path: projectPath('/quota',),
  },);
}

//endregion Regions and quota

//region SSH keys

/**
 Lists the project's SSH keys, from `GET /cloud/project/{serviceName}/sshkey`.
 
 @returns every key in the project
 
 @example
 ```ts
 const keys = await listSshKeys();
 ```
 */
export function listSshKeys(): Promise<readonly OvhSshKey[]> {
  return ovhRequest<readonly OvhSshKey[]>({
    method: 'GET',
    path: projectPath('/sshkey',),
  },);
}

/**
 Uploads one public key, from `POST /cloud/project/{serviceName}/sshkey`.
 No region is sent, so OVHcloud places the key in every region of the project.
 
 @param name - key name within the project
 
 @param publicKey - the whole `"<type> <base64> <comment>"` line
 
 @returns the created key, with the identifier instance creation takes
 
 @example
 ```ts
 const key = await createSshKey({ name: 'mvm-abc', publicKey: 'ssh-ed25519 AAAA...' });
 ```
 */
export function createSshKey({
  name,
  publicKey,
}: {
  readonly name: string;
  readonly publicKey: string;
},): Promise<OvhSshKeyDetail> {
  return ovhRequest<OvhSshKeyDetail>({
    method: 'POST',
    path: projectPath('/sshkey',),
    body: {
      name,
      publicKey,
    },
  },);
}

//endregion SSH keys

//region Snapshots

/**
 Snapshots an instance, from `POST /cloud/project/{serviceName}/instance/{instanceId}/snapshot`.
 
 @param instanceId - instance to snapshot
 
 @param snapshotName - name for the image the snapshot becomes
 
 @returns the identifier of the image to boot a clone from
 
 @example
 ```ts
 const { snapshotId } = await createSnapshot({ instanceId: 'abc', snapshotName: 'mvm-clone' });
 ```
 */
export function createSnapshot({
  instanceId,
  snapshotName,
}: {
  readonly instanceId: string;
  readonly snapshotName: string;
},): Promise<OvhInstanceSnapshot> {
  return ovhRequest<OvhInstanceSnapshot>({
    method: 'POST',
    path: projectPath(`/instance/${encodeURIComponent(instanceId,)}/snapshot`,),
    body: { snapshotName, },
  },);
}

/**
 Lists the project's snapshots, from `GET /cloud/project/{serviceName}/snapshot`.
 
 @returns the snapshot images, with the names they were created under
 
 @example
 ```ts
 const snapshots = await listSnapshots();
 ```
 */
export function listSnapshots(): Promise<readonly OvhImage[]> {
  return ovhRequest<readonly OvhImage[]>({
    method: 'GET',
    path: projectPath('/snapshot',),
  },);
}

/**
 Deletes one snapshot, from `DELETE /cloud/project/{serviceName}/snapshot/{imageId}`.
 A snapshot left behind is billed per GB per month, so this runs on every clone.
 
 @param imageId - snapshot image identifier
 
 @example
 ```ts
 await deleteSnapshot({ imageId: 'abc' });
 ```
 */
export function deleteSnapshot({
  imageId,
}: {
  readonly imageId: string;
},): Promise<void> {
  return ovhRequest<void>({
    method: 'DELETE',
    path: projectPath(`/snapshot/${encodeURIComponent(imageId,)}`,),
  },);
}

//endregion Snapshots
