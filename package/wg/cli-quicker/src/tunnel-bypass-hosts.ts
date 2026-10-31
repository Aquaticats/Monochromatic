import { normalizeAddress, } from './host-address.ts';
import { runIpDelete, } from './ip-delete.ts';
import { run, } from './runner.ts';
import {
  type BypassProto,
  BYPASS_ROUTE_PROTOCOL,
  type BypassState,
} from './tunnel-bypass-types.ts';

/**
 Refreshes an address may be missing from the answers before its rule is removed.

 DNS round-robin and CDN rotation drop addresses for one answer and bring them back for the next,
 so a rule stays through one absent refresh.
 */
export const ABSENT_REFRESHES_BEFORE_REMOVAL = 2;

/**
 One resolved address of an update host.
 */
export type HostAddress = {
  /**
   Address family flag accepted by `ip`.
   */
  readonly family: BypassProto;

  /**
   Normalized address text.
   */
  readonly address: string;

  /**
   Remaining DNS time to live in seconds.
   */
  readonly ttlSeconds: number;
};

/**
 Answers for one host,
 with the failure text when nothing resolved.
 */
export type HostLookupResult = {
  readonly addresses: readonly HostAddress[];
  readonly failure?: string;
};

/**
 Resolves one host.
 */
export type HostLookup = (call: { readonly host: string; },) => Promise<HostLookupResult>;

/**
 One owned rule as the kernel lists it.
 */
export type OwnedHostRule = {
  readonly family: BypassProto;
  readonly address: string;
};

/**
 Side effects on policy rules,
 replaceable in tests.
 */
export type HostRuleTools = {
  /**
   Lists rules of one family as `ip -json rule show` text.
   */
  readonly list: (call: { readonly family: BypassProto; },) => Promise<string>;

  /**
   Adds one `to <address>` rule in the claimed table.
   */
  readonly add: (call: {
    readonly rule: OwnedHostRule;
    readonly state: BypassState
  },) => Promise<void>;

  /**
   Deletes one `to <address>` rule.
   */
  readonly remove: (call: {
    readonly rule: OwnedHostRule;
    readonly state: BypassState
  },) => Promise<void>;
};

/**
 Result of one reconciliation.
 */
export type HostSyncReport = {
  /**
   Rules added.
   */
  readonly added: number;

  /**
   Rules removed.
   */
  readonly removed: number;

  /**
   Everything that left the policy unenforced for some host or address.
   */
  readonly failures: readonly string[];

  /**
   Seconds until the next refresh.
   */
  readonly nextRefreshSeconds: number;
};

/**
 Reports whether a parsed JSON value is a non-null object.
 
 @param value - Parsed JSON.
 
 @returns Whether the value has named fields.
 
 @example
 ```ts
 isFieldRecord({ priority: 50 });
 ```
 */
function isFieldRecord(value: unknown,): value is Readonly<Record<string, unknown>> {
  return ((typeof value) === 'object') && (value !== null)
    && (!Array.isArray(value,));
}

/**
 Reads the `to` rules this lifecycle owns from `ip -json rule show` text.

 A rule is owned when it carries the private protocol,
 the claimed table,
 the claimed preference,
 and a destination selector.
 The table is allocated exclusively to this lifecycle,
 so no other rule can match all four.

 @param text - JSON text for one family.

 @param family - Family the text was listed for.

 @param state - Claimed table and preference.

 @returns Owned rules.

 @example
 ```ts
 parseOwnedHostRules({ text, family: '-4', state });
 ```
 */
export function parseOwnedHostRules(
  {
    text,
    family,
    state,
  }: {
    readonly text: string;
    readonly family: BypassProto;
    readonly state: BypassState;
  },
): readonly OwnedHostRule[] {
  /**
   Parsed listing.
   */
  const parsed: unknown = JSON.parse(text === '' ? '[]' : text,);
  if (!Array.isArray(parsed,))
    return [];
  return parsed.flatMap(function ownedRule(entry: unknown,): readonly OwnedHostRule[] {
    if (!isFieldRecord(entry,))
      return [];
    if (String(entry.protocol,) !== String(BYPASS_ROUTE_PROTOCOL,))
      return [];
    if (String(entry.table,) !== String(state.table,))
      return [];
    if (Number(entry.priority,) !== state.preference)
      return [];
    if ((typeof entry.dst) !== 'string')
      return [];
    /**
     Destination normalized for comparison.
     */
    const address = normalizeAddress({
      family,
      text: entry.dst,
    },);
    return address === '' ? [] : [{
      family,
      address,
    },];
  },);
}

/**
 Identity of a rule for set comparison.

 @param rule - Rule or desired address.

 @returns Unique key.

 @example
 ```ts
 ruleKey({ rule: { family: '-4', address: '203.0.113.7' } });
 ```
 */
export function ruleKey({ rule, }: { readonly rule: OwnedHostRule; },): string {
  return `${rule.family}|${rule.address}`;
}

/**
 Decides which rules to add and remove.

 @param desired - Rules wanted now.

 @param owned - Rules installed.

 @param absentCounts - Consecutive refreshes each installed rule was missing,
 updated in place.

 @returns Rules to add and rules to remove.

 @example
 ```ts
 planHostRules({ desired, owned, absentCounts: new Map() });
 ```
 */
export function planHostRules(
  {
    desired,
    owned,
    absentCounts,
  }: {
    readonly desired: readonly OwnedHostRule[];
    readonly owned: readonly OwnedHostRule[];
    readonly absentCounts: Map<string, number>;
  },
): {
  readonly add: readonly OwnedHostRule[];
  readonly remove: readonly OwnedHostRule[]
} {
  /**
   Keys wanted now.
   */
  const wanted = new Set(desired.map(function keyOf(rule,): string {
    return ruleKey({ rule, },);
  },),);
  /**
   Keys installed.
   */
  const installed = new Set(owned.map(function keyOf(rule,): string {
    return ruleKey({ rule, },);
  },),);
  for (const key of wanted)
    absentCounts.delete(key,);
  /**
   Installed rules no longer wanted that have been missing long enough.
   */
  const remove = owned.filter(function expired(rule,): boolean {
    /**
     Identity of the installed rule.
     */
    const key = ruleKey({ rule, },);
    if (wanted.has(key,))
      return false;
    /**
     Refreshes this rule has now been missing.
     */
    const missing = (absentCounts.get(key,) ?? 0) + 1;
    absentCounts.set(
      key,
      missing,
    );
    return missing >= ABSENT_REFRESHES_BEFORE_REMOVAL;
  },);
  for (const rule of remove)
    absentCounts.delete(ruleKey({ rule, },),);
  return {
    add: desired.filter(function missing(rule,): boolean {
      return !installed.has(ruleKey({ rule, },),);
    },),
    remove,
  };
}

/**
 Builds the `ip` arguments for a `to` rule.

 @param verb - `add` or `delete`.

 @param rule - Rule identity.

 @param state - Claimed table and preference.

 @returns Arguments after `ip`.

 @example
 ```ts
 hostRuleArguments({ verb: 'add', rule, state });
 ```
 */
export function hostRuleArguments(
  {
    verb,
    rule,
    state,
  }: {
    readonly verb: 'add' | 'delete';
    readonly rule: OwnedHostRule;
    readonly state: BypassState;
  },
): readonly string[] {
  return [
    rule.family,
    'rule',
    verb,
    'to',
    `${rule.address}/${rule.family === '-4' ? '32' : '128'}`,
    'table',
    String(state.table,),
    'pref',
    String(state.preference,),
    'protocol',
    String(BYPASS_ROUTE_PROTOCOL,),
  ];
}

/**
 The real `ip` side effects.
 */
export const IP_HOST_RULE_TOOLS: HostRuleTools = {
  list: async function list({ family, },): Promise<string> {
    return (await run({
      command: 'ip',
      args: [
        '-json',
        family,
        'rule',
        'show',
      ],
    },)).stdout;
  },
  add: async function add({
    rule,
    state,
  },): Promise<void> {
    await run({
      command: 'ip',
      args: hostRuleArguments({
        verb: 'add',
        rule,
        state,
      },),
    },);
  },
  remove: async function remove({
    rule,
    state,
  },): Promise<void> {
    await runIpDelete({ args: hostRuleArguments({
      verb: 'delete',
      rule,
      state,
    },), },);
  },
};


export {
  isPublicAddress,
  normalizeAddress,
  refreshSeconds,
} from './host-address.ts';
export type { BypassState, } from './tunnel-bypass-types.ts';
