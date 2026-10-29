import { tagged, } from '@monochromatic-dev/module-logger/ts';

import { OpenSnitchConfigError, } from './errors.ts';
import { currentNetworkNamespace, } from './network-namespace.ts';
import { OPENSNITCH_CONFIG_ABSENT, } from './opensnitch-config-file.ts';
import {
  claimOpenSnitchInterfaceOperation,
  inspectOpenSnitchConfig,
  mergePorts,
  reconcileOpenSnitchEndpointAllowance,
  removePersistedOpenSnitchAllowance,
  resolveOpenSnitchPath,
} from './opensnitch-operation.ts';
import {
  OPENSNITCH_STATE_ABSENT,
  readOpenSnitchState,
  removeOpenSnitchState,
  writeOpenSnitchState,
} from './opensnitch-state.ts';

export {
  OPENSNITCH_CONFIG_ENVIRONMENT,
  OPENSNITCH_DAEMON_CONFIG_ENVIRONMENT,
} from './opensnitch-daemon-config.ts';

/**
 Module logger for OpenSnitch system-firewall integration.
 */
const l = tagged({ tag: 'opensnitch', },);

/**
 Adds visible OpenSnitch accept rules for WireGuard endpoint UDP ports.
 
 Any process can use accepted destination port while interface is up;
 warning makes policy widening explicit.
 
 @param interfaceName - WireGuard interface owning rules.
 
 @param endpointPorts - Distinct peer endpoint UDP ports.
 
 @example
 ```ts
 await installOpenSnitchEndpointAllowance({ interfaceName: 'wg0', endpointPorts: [51820] });
 ```
 */
export async function installOpenSnitchEndpointAllowance(
  {
    interfaceName,
    endpointPorts,
  }: {
    readonly interfaceName: string;
    readonly endpointPorts: readonly number[];
  },
): Promise<void> {
  /**
   Current network namespace establishing ownership identity.
   */
  const { key: networkNamespaceKey, } = await currentNetworkNamespace();
  /**
   Interface lock covering manifest and config transition.
   */
  await using interfaceOperation = await claimOpenSnitchInterfaceOperation({
    interfaceName,
    networkNamespaceKey,
  },);
  /**
   Prior lifecycle state surviving normal operation or interrupted transition.
   */
  const previous = await readOpenSnitchState({
    interfaceName,
    networkNamespaceKey,
  },);
  /**
   Effective validated startup path or daemon-absence sentinel.
   */
  const resolvedPath = await resolveOpenSnitchPath({ requireNftables: true, },);
  if ((typeof resolvedPath) === 'symbol') {
    if (resolvedPath !== OPENSNITCH_CONFIG_ABSENT)
      throw new OpenSnitchConfigError('Unexpected OpenSnitch installation path result.',);
    if ((typeof previous) !== 'symbol') {
      await removePersistedOpenSnitchAllowance({
        interfaceName,
        state: previous,
      },);
    }
    return;
  }
  if (((typeof previous) !== 'symbol') && (previous.path !== resolvedPath)) {
    await removePersistedOpenSnitchAllowance({
      interfaceName,
      state: previous,
    },);
  }
  /**
   State remaining on same path after stale-path cleanup.
   */
  const samePathState = ((typeof previous) !== 'symbol') && (previous.path === resolvedPath)
    ? previous
    : OPENSNITCH_STATE_ABSENT;
  /**
   Dry schema check before claiming external cleanup ownership.
   */
  const inspection = await inspectOpenSnitchConfig({
    path: resolvedPath,
    interfaceName,
    endpointPorts,
    networkNamespaceKey,
  },);
  if ((typeof inspection) === 'symbol') {
    if (inspection !== OPENSNITCH_CONFIG_ABSENT)
      throw new OpenSnitchConfigError('Unexpected OpenSnitch inspection result.',);
    if ((typeof samePathState) !== 'symbol') {
      throw new OpenSnitchConfigError(
        `Cannot update persisted OpenSnitch rules because config is absent: ${resolvedPath}`,
      );
    }
    return;
  }
  /**
   Transitional ownership includes every port a crash could leave behind.
   */
  const transitionPorts = mergePorts({
    groups: [
      inspection.managedPorts,
      inspection.forbiddenPorts,
      (typeof samePathState) === 'symbol' ? [] : samePathState.ports,
    ],
  },);
  await writeOpenSnitchState({
    interfaceName,
    state: {
      networkNamespaceKey,
      path: resolvedPath,
      ports: transitionPorts,
    },
  },);
  /**
   Reconciled and positively verified OpenSnitch state.
   */
  const result = await reconcileOpenSnitchEndpointAllowance({
    path: resolvedPath,
    interfaceName,
    endpointPorts,
    networkNamespaceKey,
    previousManagedPorts: transitionPorts,
    requireEnabled: true,
    verifyLive: true,
  },);
  if ((typeof result) === 'symbol') {
    throw new OpenSnitchConfigError(
      `OpenSnitch config disappeared during installation: ${resolvedPath}`,
    );
  }
  /**
   Managed port count after successful reconciliation.
   */
  const managedPortCount = result
    .ports
    .length;
  if (managedPortCount === 0) {
    await removeOpenSnitchState({
      interfaceName,
      networkNamespaceKey,
    },);
    return;
  }
  await writeOpenSnitchState({
    interfaceName,
    state: {
      networkNamespaceKey,
      path: result.path,
      ports: result.ports,
    },
  },);
  /**
   Human-readable accepted port list.
   */
  const renderedPorts = result
    .ports
    .join(', ',);
  l.warn(
    [
      `OpenSnitch now accepts any process's outbound UDP to destination port(s) ${renderedPorts} `,
      `before application filtering while ${interfaceName} is up. wg-quicker added visible rules to ${result.path} `,
      'and removes them on down.',
    ].join('',),
  );
}

/**
 Removes one interface's managed OpenSnitch rules during teardown.
 
 Removal is best-effort so malformed external config cannot strand tunnel routes or link.
 Every failure is logged because stale rules widen firewall policy.
 
 @param interfaceName - WireGuard interface whose rules are removed.
 
 @example
 ```ts
 await removeOpenSnitchEndpointAllowance({ interfaceName: 'wg0' });
 ```
 */
export async function removeOpenSnitchEndpointAllowance(
  { interfaceName, }: { readonly interfaceName: string; },
): Promise<void> {
  try {
    /**
     Current network namespace establishing ownership identity.
     */
    const { key: networkNamespaceKey, } = await currentNetworkNamespace();
    /**
     Interface lock covering manifest and config transition.
     */
    await using interfaceOperation = await claimOpenSnitchInterfaceOperation({
      interfaceName,
      networkNamespaceKey,
    },);
    /**
     Persisted exact cleanup target when installation reached ownership transition.
     */
    const persisted = await readOpenSnitchState({
      interfaceName,
      networkNamespaceKey,
    },);
    if ((typeof persisted) !== 'symbol') {
      await removePersistedOpenSnitchAllowance({
        interfaceName,
        state: persisted,
      },);
      return;
    }
    if (persisted !== OPENSNITCH_STATE_ABSENT)
      throw new OpenSnitchConfigError('Unexpected OpenSnitch lifecycle-state result.',);
    /**
     Fallback path supports managed rules created before lifecycle manifests existed.
     */
    const path = await resolveOpenSnitchPath({ requireNftables: false, },);
    if ((typeof path) === 'symbol') {
      if (path === OPENSNITCH_CONFIG_ABSENT)
        return;
      throw new OpenSnitchConfigError('Unexpected OpenSnitch fallback removal path result.',);
    }
    await reconcileOpenSnitchEndpointAllowance({
      path,
      interfaceName,
      endpointPorts: [],
      networkNamespaceKey,
      previousManagedPorts: [],
      requireEnabled: false,
      verifyLive: false,
    },);
  }
  catch (error) {
    l.error(
      `Cannot remove ${interfaceName} OpenSnitch endpoint rules; `
        + `remove wg-quicker-managed rules manually: ${String(error,)}`,
    );
  }
}
