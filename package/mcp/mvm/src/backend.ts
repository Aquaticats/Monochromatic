/**
 Shared backend resolution for MCP tool handlers.
 
 Routes through the same `resolveBackendKind` + `selectBackend` path the CLI
 uses, so the default, `MVM_BACKEND` env, and unknown-kind handling are
 identical and never bypassed.
 
 @module
 */

import {
  type Backend,
  resolveBackendKind,
  selectBackend,
} from '@monochromatic-dev/mvm-cli/ts';
import {
  type OptionalDescribedString,
  optionalString,
} from './tool-arguments.ts';

/**
 Reusable `backend` argument shared by every mvm tool so the per-invocation contract
 reads identically to clients.
 
 Deliberately a free string rather than a picklist of the known kinds: an unrecognised
 value must reach {@link resolveBackendKind}, which reports what it could not resolve.
 Narrowing it here would turn that into a schema rejection naming no backend at all.
 
 @example
 ```ts
 strictArguments({ backend: BACKEND_ARGUMENT });
 ```
 */
export const BACKEND_ARGUMENT: OptionalDescribedString = optionalString(
  'Backend to target: libvirt (default, local KVM, Linux only) or hetzner (Hetzner Cloud; requires HCLOUD_TOKEN). There is no record of which backend a VM lives on, so pass the same backend used at create to every follow-up call.',
);

/**
 Resolves the backend from a tool's `backend` argument via
 {@link resolveBackendKind} and {@link selectBackend}.
 
 @param args - raw tool arguments; the optional `backend` string is read
 
 @returns selected backend implementation
 
 @throws Error when `backend` names an unknown kind or is unavailable on this platform
 
 @example
 ```ts
 const backend = await backendFromArgs(args);
 await backend.list();
 ```
 */
export function backendFromArgs(
  args: Readonly<Record<string, unknown>>,
): Promise<Backend> {
  /**
   Optional per-invocation backend selector; absence falls back to env/default.
   */
  const raw = ((typeof args.backend) === 'string') ? args.backend : undefined;
  return selectBackend(resolveBackendKind(raw,),);
}
