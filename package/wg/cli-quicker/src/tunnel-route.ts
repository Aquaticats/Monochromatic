import type { WireguardConfig, } from './config.ts';
import {
  run,
  runAllowingFailure,
} from './runner.ts';
import { addExemptRule, } from './tunnel-bypass.ts';
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
 Address-family flag accepted by the `ip` tool.
 */
type Proto = '-4' | '-6';

/**
 A `not fwmark`/`suppress_prefixlength` rule pair tracked for teardown.
 */
type PolicyRule = {
  /**
   Address family the rule applies to.
   */
  readonly proto: Proto;

  /**
   Policy table number the `not fwmark` rule targets, absent for the suppress rule.
   */
  readonly table?: number;
};


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
    ],
  },);
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
  /**
   Rule descriptors to remove: per-family `not fwmark` then `suppress_prefixlength`.
   */
  const rules: PolicyRule[] = [];
  if (fwmark.found) {
    rules.push(
      {
        proto: '-4',
        table: fwmark.table,
      },
      {
        proto: '-6',
        table: fwmark.table,
      },
    );
  }
  rules.push(
    { proto: '-4', },
    { proto: '-6', },
  );
  await Promise.all(rules.map(function remove(rule,): Promise<unknown> {
    /**
     Base of the delete command, optionally naming a specific table.
     */
    const base: string[] = rule.table === undefined ? [
      rule.proto,
      'rule',
      'delete',
      'table',
      'main',
      'suppress_prefixlength',
      '0',
    ] : [
      rule.proto,
      'rule',
      'delete',
      'table',
      String(rule.table,),
    ];
    return runAllowingFailure({
      command: 'ip',
      args: base,
    },);
  },),);
}
