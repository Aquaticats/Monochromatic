import { mkdir, } from 'node:fs/promises';

import { tagged, } from '@monochromatic-dev/module-logger/ts';

import {
  OpenSnitchConfigError,
  OpenSnitchLiveReloadError,
} from './errors.ts';
import { claimOperationLock, } from './operation-lock.ts';
import {
  isOpenSnitchConfigAbsent,
  OPENSNITCH_CONFIG_ABSENT,
  readOpenSnitchConfig,
  writeOpenSnitchConfig,
} from './opensnitch-config-file.ts';
import {
  OPENSNITCH_DAEMON_CONFIG_ABSENT,
  resolveOpenSnitchSystemFirewallPath,
} from './opensnitch-daemon-config.ts';
import {
  type OpenSnitchConfigMutation,
  parseOpenSnitchConfig,
  reconcileOpenSnitchConfig,
  renderOpenSnitchConfig,
} from './opensnitch-config-tree.ts';
import { verifyOpenSnitchLiveReload, } from './opensnitch-live.ts';
import {
  removeOpenSnitchState,
  type OpenSnitchState,
} from './opensnitch-state.ts';
import {
  bypassRuntimeDirectory,
  bypassStateKey,
} from './tunnel-bypass-path.ts';

/**
 Module logger for OpenSnitch reconciliation operations.
 */
const l = tagged({ tag: 'opensnitch-operation', },);

/**
 Existing OpenSnitch config plus managed endpoint ports.
 */
export type OpenSnitchReconcileResult = {
  /**
   System-firewall config path changed or inspected.
   */
  readonly path: string;

  /**
   Sorted managed endpoint ports after reconciliation.
   */
  readonly ports: readonly number[];
};

/**
 Creates OpenSnitch-specific error for shared lock implementation.
 
 @param message - Lock failure diagnostic.
 
 @returns OpenSnitch configuration failure.
 
 @example
 ```ts
 makeOpenSnitchConfigError('busy');
 ```
 */
function makeOpenSnitchConfigError(message: string,): OpenSnitchConfigError {
  return new OpenSnitchConfigError(message,);
}

/**
 Ensures private runtime directory exists.
 
 @example
 ```ts
 await ensureOpenSnitchRuntimeDirectory();
 ```
 */
async function ensureOpenSnitchRuntimeDirectory(): Promise<void> {
  await mkdir(
    bypassRuntimeDirectory(),
    {
      mode: 0o700,
      recursive: true,
    },
  );
}

/**
 Acquires config-path lock serializing wg-quicker read-modify-write operations.
 
 @param path - OpenSnitch system-firewall config path.
 
 @returns Crash-safe advisory lock guard.
 
 @example
 ```ts
 await using lock = await claimOpenSnitchConfigOperation({ path: '/etc/opensnitchd/system-fw.json' });
 ```
 */
async function claimOpenSnitchConfigOperation(
  { path, }: { readonly path: string; },
): Promise<AsyncDisposable> {
  await ensureOpenSnitchRuntimeDirectory();
  return await claimOperationLock({
    lockPath: `${bypassRuntimeDirectory()}/opensnitch-${bypassStateKey({ interfaceName: path, },)}.operation.lock`,
    conflictMessage: 'Another wg-quicker lifecycle is editing OpenSnitch system firewall.',
    errorFactory: makeOpenSnitchConfigError,
  },);
}

/**
 Acquires interface lock covering lifecycle manifest and config transition.
 
 @param interfaceName - WireGuard interface identity.
 
 @param networkNamespaceKey - Namespace-specific ownership identity.
 
 @returns Crash-safe advisory lock guard.
 
 @example
 ```ts
 await using lock = await claimOpenSnitchInterfaceOperation({
   interfaceName: 'wg0',
   networkNamespaceKey: 'abc123',
 });
 ```
 */
export async function claimOpenSnitchInterfaceOperation(
  {
    interfaceName,
    networkNamespaceKey,
  }: {
    readonly interfaceName: string;
    readonly networkNamespaceKey: string;
  },
): Promise<AsyncDisposable> {
  await ensureOpenSnitchRuntimeDirectory();
  return await claimOperationLock({
    lockPath: `${bypassRuntimeDirectory()}/opensnitch-interface-${bypassStateKey({
      interfaceName: `${interfaceName}\0${networkNamespaceKey}`,
    },)}.operation.lock`,
    conflictMessage: `Another wg-quicker lifecycle is changing ${interfaceName} OpenSnitch rules.`,
    errorFactory: makeOpenSnitchConfigError,
  },);
}

/**
 Resolves effective config path while mapping daemon absence to integration sentinel.
 
 @param requireNftables - Whether startup requires validated nftables backend.
 
 @returns Absolute system-firewall path or absence sentinel.
 
 @example
 ```ts
 await resolveOpenSnitchPath({ requireNftables: true });
 ```
 */
export async function resolveOpenSnitchPath(
  { requireNftables, }: { readonly requireNftables: boolean; },
): Promise<string | typeof OPENSNITCH_CONFIG_ABSENT> {
  /**
   Daemon-derived path or daemon-absence sentinel.
   */
  const resolved = await resolveOpenSnitchSystemFirewallPath({ requireNftables, },);
  if ((typeof resolved) !== 'symbol')
    return resolved;
  if (resolved === OPENSNITCH_DAEMON_CONFIG_ABSENT)
    return OPENSNITCH_CONFIG_ABSENT;
  throw new OpenSnitchConfigError('Unexpected OpenSnitch system-firewall path result.',);
}

/**
 Inspects config before lifecycle ownership is persisted.
 
 @param path - Concrete system-firewall path.
 
 @param interfaceName - WireGuard interface owning rules.
 
 @param endpointPorts - Desired endpoint ports.
 
 @param networkNamespaceKey - Namespace-specific ownership identity.
 
 @returns Dry reconciliation metadata or absence sentinel.
 
 @example
 ```ts
 await inspectOpenSnitchConfig({
   path,
   interfaceName: 'wg0',
   endpointPorts: [51820],
   networkNamespaceKey: 'abc123',
 });
 ```
 */
export async function inspectOpenSnitchConfig(
  {
    path,
    interfaceName,
    endpointPorts,
    networkNamespaceKey,
  }: {
    readonly path: string;
    readonly interfaceName: string;
    readonly endpointPorts: readonly number[];
    readonly networkNamespaceKey: string;
  },
): Promise<OpenSnitchConfigMutation | typeof OPENSNITCH_CONFIG_ABSENT> {
  /**
   Current safe config source.
   */
  const source = await readOpenSnitchConfig({ path, },);
  if (isOpenSnitchConfigAbsent(source,))
    return OPENSNITCH_CONFIG_ABSENT;
  return reconcileOpenSnitchConfig({
    document: parseOpenSnitchConfig({
      text: source,
      path,
    },),
    interfaceName,
    endpointPorts,
    networkNamespaceKey,
    path,
    requireEnabled: true,
  },);
}

/**
 Reconciles one interface's managed rules under config-path lock.
 
 @param path - Concrete system-firewall path.
 
 @param interfaceName - WireGuard interface owning rules.
 
 @param endpointPorts - Desired endpoint ports; empty removes rules.
 
 @param networkNamespaceKey - Namespace-specific ownership identity.
 
 @param previousManagedPorts - Persisted ports requiring crash-recovery verification.
 
 @param requireEnabled - Whether disabled system firewall rejects operation.
 
 @param verifyLive - Whether unchanged config still requires kernel verification.
 
 @returns Existing config path and managed ports or file-absence sentinel.
 
 @example
 ```ts
 await reconcileOpenSnitchEndpointAllowance({
   path,
   interfaceName: 'wg0',
   endpointPorts: [51820],
   networkNamespaceKey: 'abc123',
   previousManagedPorts: [],
   requireEnabled: true,
   verifyLive: true,
 });
 ```
 */
export async function reconcileOpenSnitchEndpointAllowance(
  {
    path,
    interfaceName,
    endpointPorts,
    networkNamespaceKey,
    previousManagedPorts,
    requireEnabled,
    verifyLive,
  }: {
    readonly path: string;
    readonly interfaceName: string;
    readonly endpointPorts: readonly number[];
    readonly networkNamespaceKey: string;
    readonly previousManagedPorts: readonly number[];
    readonly requireEnabled: boolean;
    readonly verifyLive: boolean;
  },
): Promise<OpenSnitchReconcileResult | typeof OPENSNITCH_CONFIG_ABSENT> {
  /**
   Unlocked existence probe avoiding runtime config lock when file is absent.
   */
  const initial = await readOpenSnitchConfig({ path, },);
  if (isOpenSnitchConfigAbsent(initial,))
    return OPENSNITCH_CONFIG_ABSENT;
  /**
   Config-path operation lock held through disk and live-kernel convergence.
   */
  await using lock = await claimOpenSnitchConfigOperation({ path, },);
  /**
   Locked source re-read protecting against concurrent pre-lock changes.
   */
  const current = await readOpenSnitchConfig({ path, },);
  if (isOpenSnitchConfigAbsent(current,))
    return OPENSNITCH_CONFIG_ABSENT;
  /**
   Immutable managed-rule reconciliation result.
   */
  const mutation = reconcileOpenSnitchConfig({
    document: parseOpenSnitchConfig({
      text: current,
      path,
    },),
    interfaceName,
    endpointPorts,
    networkNamespaceKey,
    path,
    requireEnabled,
    previousManagedPorts,
  },);
  if (mutation.changed) {
    await writeOpenSnitchConfig({
      path,
      original: current,
      rendered: renderOpenSnitchConfig({ document: mutation.document, },),
    },);
  }
  if (verifyLive || mutation.changed) {
    try {
      await verifyOpenSnitchLiveReload({
        path,
        requiredPorts: mutation.managedPorts,
        forbiddenPorts: mutation.forbiddenPorts,
      },);
    }
    catch (error) {
      if (!(error instanceof OpenSnitchLiveReloadError))
        throw error;
      l.warn(
        `OpenSnitch missed a watched-file convergence after ${path}; retrying one same-inode write: ${String(error,)}`,
      );
      /**
       Latest config preserving external edits before bounded reload retry.
       */
      const retryCurrent = await readOpenSnitchConfig({ path, },);
      if (isOpenSnitchConfigAbsent(retryCurrent,)) {
        throw new OpenSnitchLiveReloadError(
          `OpenSnitch config disappeared before live-reload retry: ${path}`,
          { cause: error, },
        );
      }
      /**
       Recomputed retry document retaining unknown fields and concurrent changes.
       */
      const retryMutation = reconcileOpenSnitchConfig({
        document: parseOpenSnitchConfig({
          text: retryCurrent,
          path,
        },),
        interfaceName,
        endpointPorts,
        networkNamespaceKey,
        path,
        requireEnabled,
        previousManagedPorts,
      },);
      await writeOpenSnitchConfig({
        path,
        original: retryCurrent,
        rendered: renderOpenSnitchConfig({ document: retryMutation.document, },),
      },);
      await verifyOpenSnitchLiveReload({
        path,
        requiredPorts: retryMutation.managedPorts,
        forbiddenPorts: retryMutation.forbiddenPorts,
      },);
    }
  }
  return {
    path,
    ports: mutation.managedPorts,
  };
}

/**
 Produces sorted distinct port union for transitional crash recovery.
 
 @param groups - Port groups whose ownership must remain recoverable.
 
 @returns Sorted distinct ports.
 
 @example
 ```ts
 mergePorts({ groups: [[51820], [2049]] });
 ```
 */
export function mergePorts(
  { groups, }: { readonly groups: readonly (readonly number[])[]; },
): readonly number[] {
  return [...new Set(groups.flat(),),]
    .toSorted(function ascending(
      a,
      b,
    ): number {
      return a - b;
    },);
}

/**
 Removes rules described by persisted lifecycle state and clears state after proof.
 
 @param interfaceName - WireGuard interface identity.
 
 @param state - Persisted config path and potential ports.
 
 @throws {@link OpenSnitchConfigError} when config disappeared before cleanup proof.
 
 @example
 ```ts
 await removePersistedOpenSnitchAllowance({ interfaceName: 'wg0', state });
 ```
 */
export async function removePersistedOpenSnitchAllowance(
  {
    interfaceName,
    state,
  }: {
    readonly interfaceName: string;
    readonly state: OpenSnitchState;
  },
): Promise<void> {
  /**
   Removal result proving config was available for reconciliation.
   */
  const result = await reconcileOpenSnitchEndpointAllowance({
    path: state.path,
    interfaceName,
    endpointPorts: [],
    networkNamespaceKey: state.networkNamespaceKey,
    previousManagedPorts: state.ports,
    requireEnabled: false,
    verifyLive: true,
  },);
  if ((typeof result) === 'symbol') {
    if (result === OPENSNITCH_CONFIG_ABSENT) {
      throw new OpenSnitchConfigError(
        `Cannot confirm removal because managed OpenSnitch config is absent: ${state.path}`,
      );
    }
    throw new OpenSnitchConfigError('Unexpected OpenSnitch removal result.',);
  }
  await removeOpenSnitchState({
    interfaceName,
    networkNamespaceKey: state.networkNamespaceKey,
  },);
}
