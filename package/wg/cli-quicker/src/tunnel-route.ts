import { tagged, } from '@monochromatic-dev/logger-core/ts';

import type { WireguardConfig, } from './config.ts';
import {
  run,
  runAllowingFailure,
} from './runner.ts';
import { addExemptRule, } from './tunnel-bypass.ts';
import { BYPASS_ROUTE_PROTOCOL, } from './tunnel-bypass-types.ts';
import { isAbsentDelete, } from './ip-delete.ts';
import { addKillSwitch, } from './tunnel-firewall.ts';
import { readFwmark, } from './tunnel-fwmark.ts';
import {
  addRoutes,
  ensureFwmark,
} from './tunnel-route-add.ts';
import {
  isV6,
  prefixLength,
  protoFlag,
} from './tunnel-util.ts';

/**
 Module logger for the routing rules.
 */
const l = tagged({ tag: 'tunnel-route', },);

/**
 Address-family flag accepted by the `ip` tool.
 */
type Proto = '-4' | '-6';


/**
 Reads the live allowed-ips prefixes for the interface, longest-prefix first.
 
 @param interfaceName - Interface to query.
 
 @returns Deduplicated prefixes across all peers, sorted longest-prefix first.
 
 @example
 ```ts
 await readAllowedPrefixes({ interfaceName: 'wg0' });
 ```
 */
async function readAllowedPrefixes(
  { interfaceName, }: { readonly interfaceName: string; },
): Promise<readonly string[]> {
  /**
   Captured `wg show allowed-ips` output.
   */
  const { stdout, } = await run({
    command: 'wg',
    args: [
      'show',
      interfaceName,
      'allowed-ips',
    ],
  },);
  /**
   Unique prefixes gathered from every peer line, skipping the leading key field.
   */
  const seen = new Set<string>();
  for (const line of stdout.split('\n',)) {
    /**
     Tab-separated fields: peer key, then the space-separated prefix list.
     */
    const fields = line.split('\t',);
    /**
     Space-separated prefix list for this peer, absent when the line has no peer.
     */
    const [, list,] = fields;
    if (list === undefined)
      continue;
    for (const prefix of list.split(' ',)) {
      if (prefix !== '')
        seen.add(prefix,);
    }
  }
  return [...seen,].toSorted(function byPrefixLengthDesc(
    a,
    b,
  ): number {
    return prefixLength({ prefix: b, },) - prefixLength({ prefix: a, },);
  },);
}


/**
 The protocol tag this program writes on the main-table suppression it installs.

 A `suppress_prefixlength 0` rule has no other identity:
 it names no interface and no table,
 so without a tag a teardown cannot tell the rule this program created from one
 another tunnel or an administrator installed,
 and it used to delete whatever it found.

 The tag is the one the bypass routes and rules already carry,
 so the rules this program created are the ones carrying it.
 */
const POLICY_RULE_PROTOCOL = BYPASS_ROUTE_PROTOCOL;

/**
 Adds one `not fwmark` and one `suppress_prefixlength` rule for a routed family.
 
 The main-table lookup preserves physical connected routes before the policy table,
 while WireGuard's own marked transport packets skip that table and reach the
 physical default. This also prevents a peer endpoint covered by a non-default
 `AllowedIPs` prefix from routing recursively into its own interface.
 
 @param proto - Address family receiving the rules.
 
 @param table - Shared policy table carried in the interface fwmark.
 
 @example
 ```ts
 await addPolicyRules({ proto: '-4', table: 51820 });
 ```
 */
async function addPolicyRules(
  {
    proto,
    table,
  }: {
    readonly proto: Proto;
    readonly table: number;
  },
): Promise<void> {
  await run({
    command: 'ip',
    args: [
      proto,
      'rule',
      'add',
      'not',
      'fwmark',
      String(table,),
      'table',
      String(table,),
    ],
  },);
  await run({
    command: 'ip',
    args: [
      proto,
      'rule',
      'add',
      'table',
      'main',
      'suppress_prefixlength',
      '0',
      'protocol',
      String(POLICY_RULE_PROTOCOL,),
    ],
  },);
}

/**
 Removes the main-table suppression of one family,
 and only the rule this program created.

 A tunnel an earlier build set up carries the same rule without the tag,
 so an absent tagged rule whose interface still has a policy fwmark
 is that earlier build's suppression and is removed by its shape.

 @param proto - Address family whose suppression is removed.

 @param automaticRouting - Whether the interface carries the fwmark that automatic routing sets,
 which is what makes a main-table suppression this interface's own.

 @example
 ```ts
 await removeSuppressRule({ proto: '-4', automaticRouting: true });
 ```
 */
async function removeSuppressRule(
  {
    proto,
    automaticRouting,
  }: {
    readonly proto: Proto;
    readonly automaticRouting: boolean;
  },
): Promise<void> {
  /**
   What the exact, tagged delete ended with.
   */
  const tagged = await runAllowingFailure({
    command: 'ip',
    args: [
      proto,
      'rule',
      'delete',
      'table',
      'main',
      'suppress_prefixlength',
      '0',
      'protocol',
      String(POLICY_RULE_PROTOCOL,),
    ],
  },);
  if (tagged.exitCode === 0)
    return;
  if (!isAbsentDelete({ result: tagged, },)) {
    l.warn(`could not remove the ${proto} main-table suppression: ${tagged.stderr.trim()}`,);
    return;
  }
  if (!automaticRouting)
    return;
  /**
   What removing the untagged rule of an earlier build ended with.
   */
  const untagged = await runAllowingFailure({
    command: 'ip',
    args: [
      proto,
      'rule',
      'delete',
      'table',
      'main',
      'suppress_prefixlength',
      '0',
    ],
  },);
  if ((untagged.exitCode !== 0) && (!isAbsentDelete({ result: untagged, },)))
    l.warn(`could not remove the ${proto} main-table suppression of an earlier build: ${untagged.stderr.trim()}`,);
}


/**
 Installs routes and policy rules for the allowed prefixes.
 
 Automatic routing places every allowed prefix in one policy table carried by
 the interface fwmark. A `not fwmark` rule per represented family routes inner
 traffic through that table, while WireGuard's marked outer packets skip it.
 Keeping every allowed prefix out of the main table prevents endpoint recursion
 even when a non-default prefix contains the peer's public endpoint.
 
 @param config - Parsed config.
 
 @example
 ```ts
 await setupRoutes({ config });
 ```
 */
export async function setupRoutes(
  { config, }: { readonly config: WireguardConfig; },
): Promise<void> {
  if (config.table === 'off')
    return;
  /**
   Interface being configured.
   */
  const iface = config.interfaceName;
  /**
   Live allowed-ips prefixes, longest first.
   */
  const prefixes = await readAllowedPrefixes({ interfaceName: iface, },);
  if ((config.table !== undefined) && (config.table !== 'auto')) {
    await addRoutes({
      interfaceName: iface,
      prefixes,
      table: Math.trunc(Number(config.table,),),
    },);
    if (config.exemptMark !== undefined)
      await addExemptRule({
        interfaceName: iface,
        mark: config.exemptMark,
        watchRouteChanges: true,
      },);
    return;
  }
  if (prefixes.length === 0) {
    if (config.exemptMark !== undefined)
      await addExemptRule({
        interfaceName: iface,
        mark: config.exemptMark,
        watchRouteChanges: true,
      },);
    return;
  }
  /**
   Shared policy table carried in the interface fwmark for both families.
   */
  const table = await ensureFwmark({ interfaceName: iface, },);
  /**
   Families represented by at least one allowed prefix, deduplicated.
   */
  const protos = new Set(prefixes.map(function toProto(prefix,): Proto {
    return protoFlag({ prefix, },);
  },),);
  /* oxlint-disable eslint/no-await-in-loop -- Policy rules install per family in deterministic order. */
  for (const proto of protos) {
    await addPolicyRules({
      proto,
      table,
    },);
  }
  /* oxlint-enable eslint/no-await-in-loop */
  if (protos.has('-4',))
    await runAllowingFailure({
      command: 'sysctl',
      args: [
        '-q',
        'net.ipv4.conf.all.src_valid_mark=1',
      ],
    },);
  await addRoutes({
    interfaceName: iface,
    prefixes,
    table,
  },);
  /**
   Exempt rule added last so its fixed priority (below every auto-allocated
   tunnel rule) is evaluated before the tunnel's `not fwmark` rule.
   */
  if (config.exemptMark !== undefined)
    await addExemptRule({
      interfaceName: iface,
      mark: config.exemptMark,
      watchRouteChanges: true,
    },);
  await addKillSwitch({
    config,
    table,
  },);
}


/**
 Removes the policy rules for an interface using its live fwmark.
 
 Rediscovering the fwmark ensures the exact allocated table is removed even
 when it differs from the 51820 base, matching wg-quick's down path.
 
 @param interfaceName - Interface whose policy rules are removed.
 
 @example
 ```ts
 await removePolicyRules({ interfaceName: 'wg0' });
 ```
 */
export async function removePolicyRules(
  { interfaceName, }: { readonly interfaceName: string; },
): Promise<void> {
  /**
   Live policy table for the interface, absent when no default was routed.
   */
  const fwmark = await readFwmark({ interfaceName, },);
  if (!fwmark.found) {
    // A config with a numeric `Table` installs routes and no policy rule at all,
    // so there is nothing of this interface's to remove,
    // and a main-table suppression found here belongs to another tunnel or an administrator.
    l.debug(`no policy routing to remove for ${interfaceName}`,);
    return;
  }
  await Promise.all(([
    '-4',
    '-6',
  ] as const).map(function remove(proto,): Promise<void> {
    return (async function removeFamily(): Promise<void> {
      await runAllowingFailure({
        command: 'ip',
        args: [
          proto,
          'rule',
          'delete',
          'table',
          String(fwmark.table,),
        ],
      },);
      await removeSuppressRule({
        proto,
        automaticRouting: true,
      },);
    })();
  },),);
}
