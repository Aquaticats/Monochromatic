/**
 Type definitions for the subset of the OVHcloud cloud API (`/cloud/project`) mvm uses.

 Every field name, every enum value and every nullability marker here is copied
 from OVHcloud's own schema of that API, `https://ca.api.ovh.com/v1/cloud.json`
 (and its European twin `https://eu.api.ovh.com/v1/cloud.json`), read on
 2026-11-28. The schema's model name is named beside each type, so a reader can
 look the shape up again. Every field this backend reads is here, with the
 neighbouring fields of the same model that a stand-in answer carries to look
 like a whole one; nothing here claims to be a complete model, and responses
 carry more.

 @module
 */

//region Instance

/**
 Instance status, exactly the values of the schema's `cloud.instance.InstanceStatusEnum`.

 `ACTIVE` is the only status in which the machine is running and answering; the
 billing guide states the meter starts there, so nothing in `BUILD` is billed.

 @example
 ```ts
 const status: OvhInstanceState = 'ACTIVE';
 ```
 */
export type OvhInstanceState =
  | 'ACTIVE'
    | 'BUILD'
    | 'BUILDING'
    | 'DELETED'
    | 'DELETING'
    | 'ERROR'
    | 'HARD_REBOOT'
    | 'MIGRATING'
    | 'PASSWORD'
    | 'PAUSED'
    | 'REBOOT'
    | 'REBUILD'
    | 'RESCUE'
    | 'RESCUED'
    | 'RESCUING'
    | 'RESIZE'
    | 'RESIZED'
    | 'RESUMING'
    | 'REVERT_RESIZE'
    | 'SHELVED'
    | 'SHELVED_OFFLOADED'
    | 'SHELVING'
    | 'SHUTOFF'
    | 'SNAPSHOTTING'
    | 'SOFT_DELETED'
    | 'STOPPED'
    | 'SUSPENDED'
    | 'UNKNOWN'
    | 'UNRESCUING'
    | 'UNSHELVING'
    | 'VERIFY_RESIZE';

/**
 One address attached to an instance, from the schema's `cloud.instance.IpAddress`.

 The schema types `type` as a bare `string`, so the set of values it can hold is
 not stated there; see {@link PUBLIC_IP_TYPE}.

 @example
 ```ts
 const address: OvhIpAddress = { ip: '192.0.2.10', type: 'public', version: 4 };
 ```
 */
export type OvhIpAddress = {
  /**
   The address itself.
   */
  readonly ip: string;
  /**
   Address family: `4` or `6`.
   */
  readonly version: number;
  /**
   Address kind, `public` or `private` in OVHcloud's public tooling. Not an
   enum in the schema.
   */
  readonly type: string;
  /**
   Network the address belongs to.
   */
  readonly networkId?: string;
  /**
   Gateway of that network.
   */
  readonly gatewayIp?: string;
};

/**
 Instance, from the schema's `cloud.instance.Instance`.

 The schema gives an instance no label field, which is why mvm scopes its own
 machines by the name prefix it already puts on every machine it makes.

 @example
 ```ts
 const instance: OvhInstance = listed[0];
 if (instance.status === 'ACTIVE') { use(instance.name); }
 ```
 */
export type OvhInstance = {
  /**
   Instance identifier, used by every per-instance path and by delete.
   */
  readonly id: string;
  /**
   Instance name; mvm's machines carry the `mvm-` prefix here.
   */
  readonly name: string;
  /**
   Region the instance runs in.
   */
  readonly region: string;
  /**
   Lifecycle status; see {@link OvhInstanceState}.
   */
  readonly status: OvhInstanceState;
  /**
   Identifier of the image the instance booted from.
   */
  readonly imageId: string;
  /**
   Identifier of the flavor (plan) the instance runs.
   */
  readonly flavorId: string;
  /**
   Every address attached to the instance, public and private.
   */
  readonly ipAddresses: readonly OvhIpAddress[];
  /**
   The image the instance booted from, when the API includes it; its `user`
   field is the account an SSH client must log in as.
   */
  readonly image?: OvhImage;
  /**
   Creation timestamp.
   */
  readonly created?: string;
  /**
   Identifiers of operations still pending on the instance.
   */
  readonly operationIds?: readonly string[];
};

/**
 The address kind the schema does not enumerate, taken from OVHcloud's public
 tooling as the value for a routable address.
 */
export const PUBLIC_IP_TYPE = 'public';

/**
 IPv4 as a number, the value `version` carries for an IPv4 address.
 */
export const IPV4_VERSION = 4;

/**
 The one status in which an instance is running, from the schema's
 `cloud.instance.InstanceStatusEnum`.
 */
export const INSTANCE_RUNNING_STATUS: OvhInstanceState = 'ACTIVE';

/**
 The status under which the project still lists an instance it has deleted.
 The listing of a live run on 2026-11-28 named a destroyed instance with this
 status seconds after its delete call was accepted. Such an instance runs
 nowhere, so it is none of mvm's machines any more and its name is free.
 */
export const INSTANCE_DELETED_STATUS: OvhInstanceState = 'DELETED';

/**
 Statuses from which an instance never becomes usable on its own, so a wait for
 it must stop and report the failure.
 */
export const INSTANCE_FAILED_STATUSES: ReadonlySet<OvhInstanceState> = new Set([
  'DELETED',
  'ERROR',
  'SOFT_DELETED',
],);

/**
 Snapshot request answer, from the schema's `cloud.instance.InstanceSnapshot`.

 @example
 ```ts
 const created: OvhInstanceSnapshot = { snapshotId: 'abc' };
 ```
 */
export type OvhInstanceSnapshot = {
  /**
   Identifier of the image the snapshot became; the value to boot a clone from.
   */
  readonly snapshotId: string;
};

//endregion Instance

//region Image

/**
 Image, from the schema's `cloud.image.Image`.

 `user` is the field that decides the SSH login account: the schema describes
 it as "User to connect with".

 @example
 ```ts
 const image: OvhImage = images[0];
 if (image.status === 'active') { boot(image.id, image.user); }
 ```
 */
export type OvhImage = {
  /**
   Image identifier, the value instance creation takes as `imageId`.
   */
  readonly id: string;
  /**
   Human name, such as `Ubuntu 24.04`.
   */
  readonly name: string;
  /**
   Region the image is offered in.
   */
  readonly region: string;
  /**
   Image kind, such as `image` or `snapshot`; the schema types it as a bare
   string and lists no values.
   */
  readonly type: string;
  /**
   Image status, such as `active`; the schema types it as a bare string.
   */
  readonly status: string;
  /**
   Who can see the image, such as `public` or `private`; a bare string in the
   schema.
   */
  readonly visibility: string;
  /**
   The account an SSH client logs in as on a machine booted from this image.
   */
  readonly user: string;
  /**
   Creation timestamp, used to pick the newest match for a shorthand.
   */
  readonly creationDate: string;
  /**
   Size in GiB.
   */
  readonly size: number;
  /**
   Smallest disk the image fits on, in GB.
   */
  readonly minDisk: number;
  /**
   Smallest memory the image needs, in MB.
   */
  readonly minRam: number;
  /**
   Flavor family the image is restricted to, when it is restricted.
   */
  readonly flavorType?: string;
  /**
   Order plan code.
   */
  readonly planCode?: string;
  /**
   Tags about the image.
   */
  readonly tags?: readonly string[];
};

//endregion Image

//region Flavor

/**
 Flavor (the plan an instance is created with), from the schema's
 `cloud.flavor.Flavor`.

 `available` is the schema's "Available in stock", so a region whose flavors are
 all unavailable is skipped before any create is attempted.

 @example
 ```ts
 const smallest: OvhFlavor = flavors[0];
 create(smallest.id, smallest.region);
 ```
 */
export type OvhFlavor = {
  /**
   Flavor identifier, the value instance creation takes as `flavorId`.
   */
  readonly id: string;
  /**
   Flavor name, which for OVHcloud's public flavors repeats the identifier.
   */
  readonly name: string;
  /**
   Region the flavor is offered in.
   */
  readonly region: string;
  /**
   Number of virtual CPUs.
   */
  readonly vcpus: number;
  /**
   Memory in GiB, as the schema describes it.
   */
  readonly ram: number;
  /**
   Disk size in GB.
   */
  readonly disk: number;
  /**
   Operating system the flavor targets, such as `linux` or `windows`; a bare
   string in the schema.
   */
  readonly osType: string;
  /**
   Flavor family, such as `general`; a bare string in the schema.
   */
  readonly type: string;
  /**
   Whether the flavor is in stock; creation is refused when it is not.
   */
  readonly available: boolean;
  /**
   How many more instances of this flavor the project's quota allows.
   */
  readonly quota: number;
  /**
   Order plan codes of this flavor, from the schema's flavor plan codes; the
   `hourly` one is the code the public catalogue prices the machine by. Absent
   in a body that carries none, which is why it is optional.
   */
  readonly planCodes?: {
    readonly hourly: string;
  };
};

//endregion Flavor

//region Region

/**
 Region status, exactly the values of the schema's `cloud.RegionStatusEnum`.
 */
export type OvhRegionStatus = 'DOWN' | 'MAINTENANCE' | 'UP';

/**
 Status of one service of a region, exactly the values of the schema's
 `cloud.ServiceStatusEnum`.
 */
export type OvhRegionServiceStatus = 'DOWN' | 'UP';

/**
 One service a region offers, from the schema's `cloud.Component`.

 A project lists regions that offer storage only beside the regions that create
 instances, under names that differ by a digit (`BHS` beside `BHS5`). The
 service named `instance` tells them apart: read from a live project on
 2026-11-28, every region that listed it answered the flavor list, and every
 region that did not answered 404. The schema types `name` as a bare string, so
 that name is taken from the live answer and not from the schema.

 @example
 ```ts
 const createsInstances = region.services.some(function isInstance(service) { return service.name === 'instance'; });
 ```
 */
export type OvhRegionService = {
  /**
   Service name; `instance` is the service that creates machines.
   */
  readonly name: string;
  /**
   Service status.
   */
  readonly status: OvhRegionServiceStatus;
  /**
   Endpoint URL of the service.
   */
  readonly endpoint: string;
};

/**
 Region detail, from the schema's `cloud.Region`.

 `countryCode` is the field that decides which regions an operator's decision
 about a country selects: the schema describes it as "Region country code",
 typed with its ISO 3166-1 alpha-2 lowercase enum, so Canada is `ca` and the
 region's name is not consulted.

 @example
 ```ts
 const canadian = detail.filter(function isCanada(region) { return region.countryCode === 'ca'; });
 ```
 */
export type OvhRegion = {
  /**
   Region name, the value instance creation takes as `region`.
   */
  readonly name: string;
  /**
   ISO 3166-1 alpha-2 country code, lowercase, from the schema's
   `countryCode2LettersEnum`.
   */
  readonly countryCode: string;
  /**
   Continent code, from the schema's `cloud.RegionContinentEnum`.
   */
  readonly continentCode: string;
  /**
   Region status; a region that is not `UP` cannot take an instance.
   */
  readonly status: OvhRegionStatus;
  /**
   Region kind, from the schema's `cloud.RegionTypeEnum`.
   */
  readonly type: string;
  /**
   Datacenter the region sits in.
   */
  readonly datacenterLocation: string;
  /**
   Availability zones of the region.
   */
  readonly availabilityZones: readonly string[];
  /**
   Services the region offers; a region with no `instance` service that is `UP`
   cannot take an instance.
   */
  readonly services: readonly OvhRegionService[];
};

//endregion Region

//region SSH key

/**
 SSH key as the list endpoint returns it, from the schema's `cloud.sshkey.SshKey`.

 @example
 ```ts
 const match = keys.find(function sameMaterial(key) { return key.publicKey === ours; });
 ```
 */
export type OvhSshKey = {
  /**
   Identifier, the value instance creation takes as `sshKeyId`.
   */
  readonly id: string;
  /**
   Name within the project.
   */
  readonly name: string;
  /**
   The whole `"<type> <base64> <comment>"` public key line.
   */
  readonly publicKey: string;
  /**
   Regions the key was created in.
   */
  readonly regions: readonly string[];
};

/**
 SSH key as the create endpoint returns it, from the schema's `cloud.sshkey.SshKeyDetail`.

 @example
 ```ts
 const created: OvhSshKeyDetail = await createSshKey({ name, publicKey });
 ```
 */
export type OvhSshKeyDetail = OvhSshKey & {
  /**
   Fingerprint OVHcloud computed for the key.
   */
  readonly fingerPrint: string;
};

//endregion SSH key

//region Quota

/**
 Instance quota in one region, from the schema's `cloud.quota.InstanceUsageQuotas`.

 @example
 ```ts
 const left = quota.maxInstances - quota.usedInstances;
 ```
 */
export type OvhInstanceQuota = {
  /**
   Instances the project may have in the region.
   */
  readonly maxInstances: number;
  /**
   Instances the project has in the region.
   */
  readonly usedInstances: number;
  /**
   Cores the project may use in the region.
   */
  readonly maxCores: number;
  /**
   Cores the project uses in the region.
   */
  readonly usedCores: number;
  /**
   Memory the project may use in the region.
   */
  readonly maxRam: number;
  /**
   Memory the project uses in the region.
   */
  readonly usedRAM: number;
};

/**
 Quotas of one region, from the schema's `cloud.quota.Quotas`.

 @example
 ```ts
 const inRegion = quotas.find(function here(entry) { return entry.region === name; });
 ```
 */
export type OvhQuotas = {
  /**
   Region the quota applies to.
   */
  readonly region: string;
  /**
   Instance quota, present for every region this backend reads.
   */
  readonly instance?: OvhInstanceQuota;
};

//endregion Quota

//region Failure

/**
 Body of a failure answer, as OVHcloud's own client reads it.

 `github.com/ovh/python-ovh`, `ovh/client.py` lines 516-545 of the clone read on
 2026-11-28, reads `errorCode` and `message` from the decoded body and maps
 `errorCode` values such as `NOT_GRANTED_CALL`, `INVALID_KEY` and
 `NOT_CREDENTIAL` to typed exceptions. Both fields are optional because a failed
 request's body is not guaranteed to be JSON at all.

 @example
 ```ts
 const body: OvhFailureBody = { message: 'This flavor is not available', errorCode: 'INVALID_PARAMETER' };
 ```
 */
export type OvhFailureBody = {
  /**
   OVHcloud's own explanation of the refusal.
   */
  readonly message?: string;
  /**
   Machine-readable code, when the answer carries one.
   */
  readonly errorCode?: string;
};

//endregion Failure
