/**
 Library API for the mvm ephemeral VM manager.
 
 VM lifecycle ({@link create}, {@link clone}, {@link destroy},
 {@link destroyAll}, {@link list}, {@link update}), command execution
 ({@link exec}, {@link run}), file transfer ({@link pushFile}, {@link pullFile}),
 image registry, and per-VM metadata, plus the backend registry
 ({@link selectBackend}, {@link resolveBackendKind}, {@link BACKENDS})
 selecting the libvirt or Hetzner backend.
 The `mvm` executable lives in `./cli.ts`.
 
 @module
 */

export * from './agent-command.ts';
/**
 Hetzner API paging and action waits, exposed for built-artifact verification.

 @internal
 */
export {
  fetchAllPages,
  waitForAction,
} from './backend/hetzner/api.ts';
/**
 Hetzner resource calls, exposed for built-artifact verification.

 @internal
 */
export {
  createServer,
  deleteServer,
  getMvmServerByName,
} from './backend/hetzner/api-resources.ts';
/**
 Hetzner configuration readers, exposed for built-artifact verification.

 @internal
 */
export {
  requireToken,
  resolveLocations,
  serverTypeOverride,
  validateHetznerName,
} from './backend/hetzner/config.ts';
/**
 Hetzner image resolution, exposed for built-artifact verification.

 @internal
 */
export { resolveHetznerImage, } from './backend/hetzner/images.ts';
/**
 Hetzner creation with location fallback, exposed for built-artifact verification.

 @internal
 */
export { createWithFallback, } from './backend/hetzner/lifecycle.ts';
/**
 Hetzner server-type selection, exposed for built-artifact verification.

 @internal
 */
export { resolveCheapestServerType, } from './backend/hetzner/server-types.ts';
/**
 Hetzner SSH and scp argument builders, exposed for built-artifact verification.

 @internal
 */
export {
  scpPullArgs,
  scpPushArgs,
  sshBaseOpts,
  sshExecArgs,
} from './backend/hetzner/ssh.ts';
/**
 OVHcloud request signing, masking and error type, exposed for built-artifact
 verification.

 @internal
 */
export {
  AUTH_TIME_PATH,
  forgetServerClockOffset,
  MASKED_PROJECT,
  maskProjectId,
  OvhApiError,
  ovhRequest,
  signRequest,
  signaturePayload,
} from './backend/ovh/api.ts';
/**
 OVHcloud resource calls, exposed for built-artifact verification.

 @internal
 */
export {
  createInstance,
  createSnapshot,
  createSshKey,
  deleteInstance,
  deleteSnapshot,
  getImage,
  getInstance,
  getRegion,
  listFlavors,
  listImages,
  listInstances,
  listQuotas,
  listRegionNames,
  listSnapshots,
  listSshKeys,
} from './backend/ovh/api-resources.ts';
/**
 OVHcloud configuration readers, exposed for built-artifact verification.

 @internal
 */
export {
  DEFAULT_ENDPOINT,
  envRegions,
  flavorOverride,
  isEndpointName,
  ovhDataDir,
  parseRegions,
  requireCredentials,
  resolveEndpointRoot,
  scpCommand,
  sshCommand,
  validateOvhName,
} from './backend/ovh/config.ts';
/**
 OVHcloud flavor selection, exposed for built-artifact verification.

 @internal
 */
export {
  NoFlavorInStockError,
  orderFlavors,
  resolveFlavor,
  selectFlavor,
} from './backend/ovh/flavors.ts';
/**
 OVHcloud image selection, exposed for built-artifact verification.

 @internal
 */
export {
  normalizeImageName,
  resolveImage,
  selectImage,
} from './backend/ovh/images.ts';
/**
 OVHcloud instance reading and waiting, exposed for built-artifact verification.

 @internal
 */
export {
  findMvmInstance,
  instanceImageUser,
  instanceLoginUser,
  listMvmInstances,
  mvmInstances,
  publicIpv4,
  runningTimeoutMessage,
  waitForSnapshotReady,
  waitUntilRunning,
} from './backend/ovh/instances.ts';
/**
 OVHcloud lifecycle operations, exposed for built-artifact verification.

 @internal
 */
export {
  ovhClone,
  ovhCreate,
  ovhDestroy,
  ovhDestroyAll,
  ovhList,
} from './backend/ovh/lifecycle.ts';
/**
 OVHcloud region selection, exposed for built-artifact verification.

 @internal
 */
export {
  listDefaultRegions,
  resolveRegions,
  selectDefaultRegions,
} from './backend/ovh/regions.ts';
/**
 OVHcloud API models, copied from the provider's schema and exposed for
 built-artifact verification.

 @internal
 */
export type {
  OvhFailureBody,
  OvhFlavor,
  OvhImage,
  OvhInstance,
  OvhInstanceQuota,
  OvhIpAddress,
  OvhInstanceState,
  OvhQuotas,
  OvhRegion,
  OvhRegionService,
  OvhRegionServiceStatus,
  OvhRegionStatus,
  OvhSshKey,
  OvhSshKeyDetail,
} from './backend/ovh/types.ts';
/**
 OVHcloud SSH and scp argument builders, exposed for built-artifact verification.

 @internal
 */
export {
  ovhConnectionTarget,
  ovhScpPullArgs,
  ovhScpPushArgs,
  ovhSshBaseOpts,
  ovhSshExecArgs,
} from './backend/ovh/ssh.ts';
export * from './backend/registry.ts';
export * from './backend/size-request.ts';
export type {
  Backend,
  BackendKind,
  BackendMeta,
} from './backend/types.ts';
export * from './clone.ts';
/**
 The cloud-init user-data generators and the guest agent drop-in path, exposed
 for built-artifact verification.

 @internal
 */
export {
  createSeedIso,
  templateUserData,
} from './cloud-init.ts';
/**
 The systemd drop-in paths mvm writes for the guest agent, exposed for
 built-artifact verification.

 @internal
 */
export {
  GUEST_AGENT_COMMAND_OOM_DROP_IN_PATH,
  GUEST_AGENT_DROP_IN_PATH,
} from './cloud-init-init-systems.ts';
/**
 The session daemon's unit name and idle limit, exposed for built-artifact
 verification.

 @internal
 */
export {
  SESSION_DAEMON_IDLE_TIMEOUT_SECONDS,
  SESSION_DAEMON_UNIT,
} from './config.ts';
export * from './create.ts';
export * from './destroy.ts';
/**
 The libvirt domain definition generator, exposed for built-artifact
 verification.

 @internal
 */
export { domainXml, } from './domain-xml.ts';
export * from './exec.ts';
export * from './file-transfer-route.ts';
export * from './file-transfer.ts';
export * from './guest-exec-errors.ts';
export * from './guest-exec.ts';
export * from './guest-file-retry.ts';
export * from './guest-file-transfer.ts';
/**
 The ISO 9660 writer used for cloud-init seed images, exposed for
 built-artifact verification.

 @internal
 */
export { createIso, } from './iso9660.ts';
export * from './libvirt-errors.ts';
export * from './libvirt-tools.ts';
export * from './list.ts';
export * from './meta.ts';
export * from './meta-errors.ts';
/**
 The existence probe, exposed for built-artifact verification.

 @internal
 */
export * from './path-exists.ts';
export * from './registry.ts';
export * from './run.ts';
export * from './session-daemon.ts';
export * from './session-daemon-fallback.ts';
export * from './session-daemon-memory.ts';
export * from './session-daemon-question.ts';
export * from './spawn-errors.ts';
/**
 Host command runner, exposed for built-artifact verification.

 @internal
 */
export { spawn, } from './spawn.ts';
export * from './update.ts';
/**
 Guest agent readiness and shutdown waits, exposed for built-artifact verification.

 @internal
 */
export * from './virsh-wait.ts';
/**
 The virsh wrapper, exposed for built-artifact verification.

 @internal
 */
export {
  virsh,
  VIRSH_DEADLINE_MS,
} from './virsh.ts';
