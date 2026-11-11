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

 A rule without the tag is left in place,
 because it cannot be told apart from one an administrator or another tunnel installed:
 the shape names no interface and no table,
 and `ip rule delete table main suppress_prefixlength 0` removes the first rule of that shape,
 which is whichever one has the lowest preference,
 not necessarily this program's.
 A tunnel an earlier build set up therefore leaves its untagged suppression behind,
 and the warning names the exact command that removes it.

 @param proto - Address family whose suppression is removed.

 @param automaticRouting - Whether the interface carries the fwmark that automatic routing sets,
 which is what would make an untagged main-table suppression an earlier build's leftover.

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
   What the probe of the family's rules ended with.
   */
  const shown = await runAllowingFailure({
    command: 'ip',
    args: [
      proto,
      'rule',
      'show',
    ],
  },);
  if (shown.exitCode !== 0) {
    /**
     What the rule listing said when it failed.
     */
    const refusal = shown.stderr
      .trim();
    l.debug(`could not list the ${proto} rules to look for a suppression without the tag: ${refusal}`,);
    return;
  }
  /**
   Every suppression of the family that carries no tag.
   */
  for (const rule of untaggedSuppressionRules({ listed: shown.stdout, },)) {
    l.warn(untaggedSuppressionWarning({
      proto,
      rule,
    },),);
  }
}

/**
 Finds the main-table suppressions of one family that carry no protocol tag,
 among the lines `ip -4 rule show` or `ip -6 rule show` printed.

 A tagged rule prints the tag,
 so a line that names the suppression and no tag is a rule this program never created.
 Nothing is deleted on this finding:
 the shape names no interface and no table,
 so the line is reported to the person who can tell whose rule it is.

 @param listed - Everything the rule listing printed.

 @returns Every untagged suppression line the listing printed, in its order.

 @example
 ```ts
 untaggedSuppressionRule({ listed: '32765: from all lookup main suppress_prefixlength 0\n' });
 ```
 */
export function untaggedSuppressionRules(
  { listed, }: { readonly listed: string; },
): readonly string[] {
  /**
   Lines of untagged suppressions, in the order the listing printed them.
   */
  const found: string[] = [];
  for (const line of listed.split('\n',)) {
    /**
     One rule line without the whitespace around it.
     */
    const trimmed = line.trim();
    /**
     Words of that rule line.
     */
    /* oxlint-disable-next-line no-restricted-syntax/no-regex -- One line of `ip rule show` output is bounded by a family's rule count, and its fields are separated by runs of whitespace a string split cannot express. */
    const words = trimmed.split(/\s+/u,);
    if (!words.includes('suppress_prefixlength',))
      continue;
    if (words.includes('proto',) || words.includes('protocol',))
      continue;
    found.push(trimmed,);
  }
  return found;
}

/**
 Explains an untagged main-table suppression that was left in place,
 and names every command that removes it.

 Deleted only by a person who knows whose rule it is:
 an earlier build of this program for an interface with automatic routing left one,
 and an administrator or another tunnel can leave the same shape,
 so removing it by shape could take routing another owner depends on.

 @param proto - Address family the rule belongs to.

 @param rule - The line the rule listing printed for it.

 @returns The warning text.

 @example
 ```ts
 untaggedSuppressionWarning({ proto: '-4', rule: '32765: from all lookup main suppress_prefixlength 0' });
 ```
 */
export function untaggedSuppressionWarning(
  {
    proto,
    rule,
  }: {
    readonly proto: Proto;
    readonly rule: string;
  },
): string {
  return `the ${proto} rule \`${rule}\` suppresses the main table without this program's protocol tag ${String(POLICY_RULE_PROTOCOL,)}, so this program did not create it and left it in place: an earlier build of this program for an interface with automatic routing left the same shape, and so can an administrator or another tunnel. If it is a leftover of an earlier build, remove it with \`sudo ip ${proto} rule delete table main suppress_prefixlength 0\`; if another owner depends on it, keep it.`;
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
