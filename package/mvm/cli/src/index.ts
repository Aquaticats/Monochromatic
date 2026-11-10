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
export * from './backend/registry.ts';
export type {
  Backend,
  BackendKind,
  BackendMeta,
} from './backend/types.ts';
export * from './clone.ts';
export * from './create.ts';
export * from './destroy.ts';
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
