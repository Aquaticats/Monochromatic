import { writeFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import { tagged, } from '@monochromatic-dev/module-logger/ts';

import { startApplicationExemptions, } from './application-exemption.ts';
import type { WireguardConfig, } from './config.ts';
import { CommandError, } from './errors.ts';
import {
  run,
  runAllowingFailure,
} from './runner.ts';
import { makeTempDir, } from './tempdir.ts';
import { cleanup, } from './tunnel-cleanup.ts';
import { linkExists, } from './tunnel-link.ts';
import {
  installOpenSnitchEndpointAllowance,
  removeOpenSnitchEndpointAllowance,
} from './opensnitch.ts';
import { assertNoPolicyRoutingConflict, } from './policy-routing-conflict.ts';
import { setupRoutes, } from './tunnel-route.ts';
import {
  deviceChar,
  digitChar,
  tokenAfter,
} from './text.ts';
import {
  executeHooks,
  protoFlag,
} from './tunnel-util.ts';

/**
 Module logger for tunnel orchestration.
 */
const l = tagged({ tag: 'tunnel', },);

/**
 WireGuard per-packet overhead subtracted from the base MTU, matching wg-quick.
 */
const WG_OVERHEAD = 80;

/**
 Fallback base MTU when the config does not specify one, matching wg-quick.
 */
const DEFAULT_BASE_MTU = 1_500;

/**
 Pushes the reconstructed config into the interface via `wg addconf`.
 
 The `AllowedIPs` line travels here verbatim; the `wg` binary parses it in
 constant time regardless of size.
 
 @param config - Parsed config carrying the reconstructed config text.
 
 @example
 ```ts
 await applyPeerConfig({ config });
 ```
 */
async function applyPeerConfig({ config, }: { readonly config: WireguardConfig; },): Promise<void> {
  /**
   Self-deleting private temp directory for the `wg addconf` config file.
   `wg` requires a real config file it can `fopen`; a `/dev/stdin` pipe from a
   spawned child is not seekable as a config file and fails with `fopen`.
   */
  await using dir = await makeTempDir();
  /**
   Config file carrying the private key, readable only by root.
   */
  const path = join(
    dir.path,
    'addconf.conf',
  );
  await writeFile(
    path,
    config.wgConfig,
    {
      mode: 0o600,
    },
  );
  await run({
    command: 'wg',
    args: [
      'addconf',
      config.interfaceName,
      path,
    ],
  },);
}

/**
 Creates the WireGuard kernel link for the interface.
 
 @param interfaceName - Interface to create.
 
 @example
 ```ts
 await addLink({ interfaceName: 'wg0' });
 ```
 */
async function addLink({ interfaceName, }: { readonly interfaceName: string; },): Promise<void> {
  await run({
    command: 'ip',
    args: [
      'link',
      'add',
      'dev',
      interfaceName,
      'type',
      'wireguard',
    ],
  },);
}

/**
 Assigns each configured address to the interface, then sets MTU and brings it up.
 
 @param config - Parsed config.
 
 @example
 ```ts
 await addAddressesAndUp({ config });
 ```
 */
async function addAddressesAndUp(
  { config, }: { readonly config: WireguardConfig; },
): Promise<void> {
  /**
   Interface whose addresses are configured.
   */
  const iface = config.interfaceName;
  /* oxlint-disable eslint/no-await-in-loop -- Addresses are added sequentially so a failure surfaces before dependent setup. */
  for (const address of config.addresses) {
    await run({
      command: 'ip',
      args: [
        protoFlag({ prefix: address, },),
        'address',
        'add',
        address,
        'dev',
        iface,
      ],
    },);
  }
  /* oxlint-enable eslint/no-await-in-loop */
  /**
   Effective MTU: configured value, else discovered from the path minus overhead.
   */
  const mtu = config.mtu ?? await discoverMtu();
  await run({
    command: 'ip',
    args: [
      'link',
      'set',
      'mtu',
      String(mtu,),
      'up',
      'dev',
      iface,
    ],
  },);
}

/**
 Discovers the tunnel MTU from the current default-route path MTU.
 
 Mirrors wg-quick's `set_mtu`: the MTU of the device carrying the default
 route (the endpoint path) minus the WireGuard per-packet overhead. Falls back
 to the standard base MTU when no default route is present.
 
 @returns Tunnel MTU to apply.
 
 @example
 ```ts
 await discoverMtu();
 ```
 */
async function discoverMtu(): Promise<number> {
  /**
   Default-route listing used to find the egress device.
   */
  const route = await runAllowingFailure({
    command: 'ip',
    args: [
      '-4',
      'route',
      'show',
      'default',
    ],
  },);
  /**
   Egress device named after `dev` in the default route, when present.
   */
  const dev = tokenAfter({
    value: route.stdout,
    keyword: 'dev',
    isChar: deviceChar,
  },);
  if (!dev.found)
    return DEFAULT_BASE_MTU - WG_OVERHEAD;
  /**
   Link-detail output carrying the device MTU.
   */
  const link = await runAllowingFailure({
    command: 'ip',
    args: [
      'link',
      'show',
      'dev',
      dev.token,
    ],
  },);
  /**
   Parsed MTU digits from the link output, when present.
   */
  const base = tokenAfter({
    value: link.stdout,
    keyword: 'mtu',
    isChar: digitChar,
  },);
  if (!base.found)
    return DEFAULT_BASE_MTU - WG_OVERHEAD;
  /**
   Numeric path MTU before subtracting overhead.
   */
  const path = Math.trunc(Number(base.token,),);
  if ((!Number.isSafeInteger(path,)) || (path <= WG_OVERHEAD))
    return DEFAULT_BASE_MTU - WG_OVERHEAD;
  return path - WG_OVERHEAD;
}

/**
 Configures DNS servers and search domains through systemd-resolved.
 
 @param config - Parsed config carrying DNS settings.
 
 @example
 ```ts
 await setDns({ config });
 ```
 */
async function setDns({ config, }: { readonly config: WireguardConfig; },): Promise<void> {
  /**
   Interface receiving DNS configuration.
   */
  const iface = config.interfaceName;
  if (config.dns
    .length
    === 0)
    return;
  await run({
    command: 'resolvectl',
    args: [
      'dns',
      iface,
      ...config.dns,
    ],
  },);
  /**
   Search domains plus the `~.` routing domain so all lookups use the tunnel.
   */
  const domains = [
    '~.',
    ...config.dnsSearch,
  ];
  await run({
    command: 'resolvectl',
    args: [
      'domain',
      iface,
      ...domains,
    ],
  },);
  await run({
    command: 'resolvectl',
    args: [
      'default-route',
      iface,
      'true',
    ],
  },);
}

/**
 Performs the up sequence and rolls back on failure.
 
 @param config - Parsed config.
 
 @example
 ```ts
 await upInner({ config });
 ```
 */
async function upInner({ config, }: { readonly config: WireguardConfig; },): Promise<void> {
  /**
   Interface brought up by this sequence.
   */
  const iface = config.interfaceName;
  try {
    await addLink({ interfaceName: iface, },);
    await executeHooks({
      hooks: config.preUp,
      interfaceName: iface,
    },);
    await applyPeerConfig({ config, },);
    await installOpenSnitchEndpointAllowance({
      interfaceName: iface,
      endpointPorts: config.endpointPorts,
    },);
    await addAddressesAndUp({ config, },);
    await setDns({ config, },);
    await setupRoutes({ config, },);
    if (config.exemptMark !== undefined)
      await startApplicationExemptions({
        interfaceName: iface,
        mark: config.exemptMark,
      },);
    await executeHooks({
      hooks: config.postUp,
      interfaceName: iface,
    },);
  }
  catch (error: unknown) {
    await cleanup({ config, },);
    throw error;
  }
}

/**
 Brings the interface up to match the parsed config, mirroring `wg-quick up`.
 
 On any failure the partially configured interface is torn back down.
 
 @param config - Parsed config.
 
 @throws {@link CommandError} when the interface already exists or a command fails.
 
 @example
 ```ts
 await up({ config });
 ```
 */
export async function up({ config, }: { readonly config: WireguardConfig; },): Promise<void> {
  /**
   Function-scoped logger for the up lifecycle.
   */
  const fl = tagged({
    tag: up.name,
    l,
  },);
  /**
   Interface being brought up.
   */
  const iface = config.interfaceName;
  fl.debug(`bringing ${iface} up`,);
  if (await linkExists({ interfaceName: iface, },)) {
    throw new CommandError({
      command: 'ip',
      args: [
        'link',
        'show',
        'dev',
        iface,
      ],
      exitCode: 0,
      stderr: `\`${iface}' already exists`,
    },);
  }
  await assertNoPolicyRoutingConflict();
  await upInner({ config, },);
  fl.debug(`${iface} is up`,);
}

/**
 Removes interface, policy rules, firewall, and DNS, mirroring `wg-quick down`.
 
 @param config - Parsed config.
 
 @param tolerateMissing - When true, a missing interface is not an error.
 
 @example
 ```ts
 await teardown({ config });
 ```
 */
async function teardown(
  {
    config,
    tolerateMissing = false,
  }: {
    readonly config: WireguardConfig;
    readonly tolerateMissing?: boolean;
  },
): Promise<void> {
  /**
   Interface torn down by this sequence.
   */
  const iface = config.interfaceName;
  if (!(await linkExists({ interfaceName: iface, },))) {
    await removeOpenSnitchEndpointAllowance({ interfaceName: iface, },);
    if (tolerateMissing)
      return;
    throw new CommandError({
      command: 'ip',
      args: [
        'link',
        'show',
        'dev',
        iface,
      ],
      exitCode: 1,
      stderr: `\`${iface}' is not present`,
    },);
  }
  await executeHooks({
    hooks: config.preDown,
    interfaceName: iface,
  },);
  await cleanup({ config, },);
  await executeHooks({
    hooks: config.postDown,
    interfaceName: iface,
  },);
}

/**
 Tears the interface down, mirroring `wg-quick down`.
 
 @param config - Parsed config.
 
 @example
 ```ts
 await down({ config });
 ```
 */
export async function down({ config, }: { readonly config: WireguardConfig; },): Promise<void> {
  /**
   Function-scoped logger for the down lifecycle.
   */
  const fl = tagged({
    tag: down.name,
    l,
  },);
  fl.debug(`bringing ${config.interfaceName} down`,);
  await teardown({ config, },);
  fl.debug(`${config.interfaceName} is down`,);
}
