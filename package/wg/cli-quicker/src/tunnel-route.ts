import { tagged, } from '@monochromatic-dev/logger-core/ts';

import type { WireguardConfig, } from './config.ts';
import {
  run,
  runAllowingFailure,
} from './runner.ts';
import { addExemptRule, } from './tunnel-bypass.ts';
import { claimBypassAllocationOperation, } from './tunnel-bypass-operation-lock.ts';
import { BYPASS_ROUTE_PROTOCOL, } from './tunnel-bypass-types.ts';
import { isAbsentDelete, } from './ip-delete.ts';
import { addKillSwitch, } from './tunnel-firewall.ts';
import { readFwmark, } from './tunnel-fwmark.ts';
import {
  addRoutes,
  setFwmark,
} from './tunnel-route-add.ts';
import {
  POLICY_CLAIM_ABSENT,
  readPolicyClaim,
  releasePolicyClaim,
  writePolicyClaim,
  type ClaimFamily,
  type PolicyClaim,
} from './tunnel-policy-claim.ts';
import { policyResourcesTakenByOthers, } from './tunnel-policy-claims.ts';
import {
  findFreePolicyPair,
  findFreeTable,
} from './tunnel-table.ts';
import { POLICY_RULE_PROTOCOL, } from './tunnel-rule-protocol.ts';
import { removeSuppressRule, } from './tunnel-suppression-warning.ts';
import type { UpMadeThing, } from './tunnel-up-made.ts';
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
 Adds one `not fwmark` and one `suppress_prefixlength` rule for a routed family.
 
 The main-table lookup preserves physical connected routes before the policy table,
 while WireGuard's own marked transport packets skip that table and reach the
 physical default. This also prevents a peer endpoint covered by a non-default
 `AllowedIPs` prefix from routing recursively into its own interface.
 
 @param proto - Address family receiving the rules.

 @param table - Shared policy table carried in the interface fwmark.

 @param suppressionPreference - Preference this tunnel's suppression rule takes,
 whose `not fwmark` rule takes the next one.
 The two are written with explicit preferences so that a teardown deletes exactly
 this interface's rules: the suppression names no interface and no table,
 so a delete by shape takes whichever rule of that shape has the lowest preference,
 which belongs to another tunnel as often as to this one.

 @example
 ```ts
 await addPolicyRules({ proto: '-4', table: 51820, suppressionPreference: 32764 });
 ```
 */
async function addPolicyRules(
  {
    proto,
    table,
    suppressionPreference,
  }: {
    readonly proto: Proto;
    readonly table: number;
    readonly suppressionPreference: number;
  },
): Promise<void> {
  await run({
    command: 'ip',
    args: [
      proto,
      'rule',
      'add',
      'pref',
      String(suppressionPreference,),
      'table',
      'main',
      'suppress_prefixlength',
      '0',
      'protocol',
      String(POLICY_RULE_PROTOCOL,),
    ],
  },);
  await run({
    command: 'ip',
    args: [
      proto,
      'rule',
      'add',
      'pref',
      String(suppressionPreference + 1,),
      'not',
      'fwmark',
      String(table,),
      'table',
      String(table,),
    ],
  },);
}

/**
 Claims the policy table and the suppression preference of an interface,
 writing the claim before anything is visible in the kernel.

 The claim is what closes the window between `wg set fwmark` and the first `ip rule add`:
 a lifecycle killed there leaves a table the kernel shows as free,
 and the claim is what tells the next allocator that the interface owns it.

 @param interfaceName - Interface whose resources are claimed.

 @param families - Address families the tunnel installs rules for.

 @returns The claim, whose families are the union of this start's and the record's.

 @example
 ```ts
 await claimPolicyTable({ interfaceName: 'wg0', families: ['-4'] });
 ```
 */
async function claimPolicyTable(
  {
    interfaceName,
    families,
  }: {
    readonly interfaceName: string;
    readonly families: readonly ClaimFamily[];
  },
): Promise<PolicyClaim> {
  /**
   The claim this interface already holds, when a start of it was killed earlier.
   */
  const existing = await readPolicyClaim({ interfaceName, },);
  if (existing !== POLICY_CLAIM_ABSENT) {
    /**
     Families this start and the record together install rules for.
     */
    const together = [...new Set([
      ...existing.families,
      ...families,
    ],),];
    if (together.length
      !== existing.families
        .length) {
      /**
       The record with this start's families added.
       */
      const widened: PolicyClaim = {
        ...existing,
        families: together,
      };
      await writePolicyClaim({ claim: widened, },);
      return widened;
    }
    return existing;
  }
  /**
   Tables and preferences other interfaces hold, which the kernel may not show yet.
   */
  const taken = await policyResourcesTakenByOthers({ interfaceName, },);
  /**
   Policy table carried in the interface fwmark.
   */
  const table = await findFreeTable({ taken: taken.tables, },);
  /**
   Preference of this tunnel's suppression rule, with its `not fwmark` rule one above it.
   */
  const suppressionPreference = await findFreePolicyPair({ taken: taken.preferences, },);
  /**
   The claim this start makes.
   */
  const claim: PolicyClaim = {
    version: 1,
    interfaceName,
    table,
    suppressionPreference,
    families,
  };
  await writePolicyClaim({ claim, },);
  return claim;
}

/**
 Chooses the policy table of an interface and installs the rules that name it,
 inside the global allocation lock.

 The lock spans the claim, the fwmark and the rules that make the table visibly
 this interface's, because a later allocator sees only the kernel and the claims:
 a table that is claimed but not yet named by a rule looks free to it otherwise,
 and two starts would then share one table and route each other's traffic.
 The lock is released as soon as the rules name the table;
 the routes added afterwards do not decide who owns it.

 @param interfaceName - Interface whose table is chosen.

 @param protos - Address families the interface routes.

 @param recordThing - Records one thing this start made.

 @returns The claim the interface holds.

 @example
 ```ts
 await choosePolicyTable({ interfaceName: 'wg0', protos: new Set(['-4']), recordThing });
 ```
 */
async function choosePolicyTable(
  {
    interfaceName,
    protos,
    recordThing,
  }: {
    readonly interfaceName: string;
    readonly protos: ReadonlySet<Proto>;
    readonly recordThing: (thing: UpMadeThing,) => void;
  },
): Promise<PolicyClaim> {
  /**
   The global allocation lock, held until the rules name the claimed table.
   */
  await using allocation = await claimBypassAllocationOperation();
  /**
   What this interface owns, written before anything is visible in the kernel.
   */
  const claim = await claimPolicyTable({
    interfaceName,
    families: [...protos,],
  },);
  // Recorded before the rules, so a failure after the first family still removes the rules of this claim.
  recordThing('policyRules',);
  await setFwmark({
    interfaceName,
    table: claim.table,
  },);
  /* oxlint-disable eslint/no-await-in-loop -- Policy rules install per family in deterministic order. */
  for (const proto of claim.families) {
    await addPolicyRules({
      proto,
      table: claim.table,
      suppressionPreference: claim.suppressionPreference,
    },);
  }
  /* oxlint-enable eslint/no-await-in-loop */
  return claim;
}

/**
 Installs routes and policy rules for the allowed prefixes.
 
 Automatic routing places every allowed prefix in one policy table carried by
 the interface fwmark. A `not fwmark` rule per represented family routes inner
 traffic through that table, while WireGuard's marked outer packets skip it.
 Keeping every allowed prefix out of the main table prevents endpoint recursion
 even when a non-default prefix contains the peer's public endpoint.
 
 @param config - Parsed config.

 @param recordThing - Records one thing this start made, so its rollback removes exactly that.

 @example
 ```ts
 await setupRoutes({ config, recordThing });
 ```
 */
export async function setupRoutes(
  {
    config,
    recordThing,
  }: {
    readonly config: WireguardConfig;
    readonly recordThing: (thing: UpMadeThing,) => void;
  },
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
    if (config.exemptMark !== undefined) {
      recordThing('exemptRouting',);
      await addExemptRule({
        interfaceName: iface,
        mark: config.exemptMark,
        watchRouteChanges: true,
      },);
    }
    return;
  }
  if (prefixes.length === 0) {
    if (config.exemptMark !== undefined) {
      recordThing('exemptRouting',);
      await addExemptRule({
        interfaceName: iface,
        mark: config.exemptMark,
        watchRouteChanges: true,
      },);
    }
    return;
  }
  /**
   Families represented by at least one allowed prefix, deduplicated.
   */
  const protos = new Set(prefixes.map(function toProto(prefix,): Proto {
    return protoFlag({ prefix, },);
  },),);
  /**
   Shared policy table carried in the interface fwmark for both families,
   chosen under the allocation lock together with the rules that name it.
   */
  const claim = await choosePolicyTable({
    interfaceName: iface,
    protos,
    recordThing,
  },);
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
    table: claim.table,
  },);
  /**
   Exempt rule added last so its fixed priority (below every auto-allocated
   tunnel rule) is evaluated before the tunnel's `not fwmark` rule.
   */
  if (config.exemptMark !== undefined) {
    recordThing('exemptRouting',);
    await addExemptRule({
      interfaceName: iface,
      mark: config.exemptMark,
      watchRouteChanges: true,
    },);
  }
  recordThing('killSwitch',);
  await addKillSwitch({
    config,
    table: claim.table,
  },);
}


/**
 Removes one rule of one family at an exact preference, tolerating its absence.
 
 @param proto - Address family.

 @param preference - Preference of the rule.

 @example
 ```ts
 await removeRuleAtPreference({ proto: '-4', preference: 32764 });
 ```
 */
async function removeRuleAtPreference(
  {
    proto,
    preference,
  }: {
    readonly proto: ClaimFamily;
    readonly preference: number;
  },
): Promise<void> {
  /**
   What the delete ended with.
   */
  const deletion = await runAllowingFailure({
    command: 'ip',
    args: [
      proto,
      'rule',
      'delete',
      'pref',
      String(preference,),
    ],
  },);
  if ((deletion.exitCode === 0) || isAbsentDelete({ result: deletion, },))
    return;
  l.warn(
    `could not remove the ${proto} rule at preference ${String(preference,)}: ${deletion.stderr
      .trim()}`,
  );
}

/**
 Removes the policy rules an interface's claim names, and the claim itself.

 The preferences are what makes this exact: the suppression rule names no interface
 and no table, so a delete by shape takes whichever rule of that shape comes first,
 which is how the `down` of one tunnel used to delete another tunnel's suppression.
 The families are the ones the tunnel installed rules for, so a tunnel that never
 had an IPv6 rule leaves its neighbour's IPv6 rule alone.

 @param claim - What the interface owns.

 @example
 ```ts
 await removeClaimedRules({ claim });
 ```
 */
async function removeClaimedRules({ claim, }: { readonly claim: PolicyClaim; },): Promise<void> {
  /* oxlint-disable eslint/no-await-in-loop -- Each preference is removed in turn so a failure names it. */
  for (const proto of claim.families) {
    await removeRuleAtPreference({
      proto,
      preference: claim.suppressionPreference,
    },);
    await removeRuleAtPreference({
      proto,
      preference: claim.suppressionPreference + 1,
    },);
  }
  /* oxlint-enable eslint/no-await-in-loop */
  await releasePolicyClaim({ claim, },);
}

/**
 Removes the policy rules for an interface using its claim, or its live fwmark.
 
 The claim names the exact table and suppression preference, so a teardown removes
 this interface's rules and no other tunnel's.
 A tunnel an earlier build set up has no claim, and its rules are found the way that
 build's teardown found them: from the live fwmark, with the untagged-suppression warning.

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
   What this interface owns, when its lifecycle recorded it.
   */
  const claim = await readPolicyClaim({ interfaceName, },);
  if (claim !== POLICY_CLAIM_ABSENT) {
    await removeClaimedRules({ claim, },);
    return;
  }
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
