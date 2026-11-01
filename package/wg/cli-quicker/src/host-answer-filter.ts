import {
  isPublicAddress,
  normalizeAddress,
} from './host-address.ts';
import {
  MAX_ADDRESSES_PER_HOST,
  MAX_ADDRESSES_TOTAL,
} from './host-rule-limits.data.ts';
import {
  type HostLookupResult,
  type OwnedHostRule,
  ruleKey,
} from './tunnel-bypass-hosts.ts';

/**
 The answer of one update host.
 */
export type HostAnswer = {
  /**
   Host name asked.
   */
  readonly host: string;

  /**
   What the resolver returned.
   */
  readonly result: HostLookupResult;
};

/**
 A rule wanted now,
 with the least TTL that applies to it and the hosts that answered it.
 */
export type WantedRule = {
  /**
   Rule identity.
   */
  readonly rule: OwnedHostRule;

  /**
   Shortest TTL among the answers that named the address,
   in seconds.
   */
  readonly ttlSeconds: number;

  /**
   Hosts whose answers named the address.
   An address several hosts share is a shared CDN address,
   and its rule carries traffic to every site on it.
   */
  readonly hosts: readonly string[];
};

/**
 Rules to install and everything refused or cut on the way.
 */
export type FilteredAnswers = {
  /**
   Rules wanted now,
   without duplicates.
   */
  readonly wanted: readonly WantedRule[];

  /**
   Problems to report loudly.
   */
  readonly failures: readonly string[];
};

/**
 Adds a host to the hosts that already named an address.

 @param earlier - Rule recorded from earlier answers,
 absent for the first one.

 @param host - Host whose answer names the address now.

 @returns Hosts without duplicates.

 @example
 ```ts
 mergeHosts({ host: 'pypi.org' }); // ['pypi.org']
 ```
 */
function mergeHosts({
  earlier,
  host,
}: {
  readonly earlier?: WantedRule;
  readonly host: string
},): readonly string[] {
  if (earlier === undefined)
    return [host,];
  return earlier.hosts
    .includes(host,) ? earlier.hosts : [
      ...earlier.hosts,
      host,
    ];
}

/**
 Turns the answers of all hosts into the rules to install.

 An address gets a rule only when it is a well-formed public unicast address,
 is not one the tunnel itself depends on,
 and fits inside the per-host and total caps.
 The per-host cap counts only addresses that passed every check.
 Each refusal and each cut is named in the failures,
 so a cut can never pass unnoticed.

 @param answers - Answers per host.

 @param protectedKeys - Keys of the tunnel's DNS servers and peer endpoints.

 @returns Rules wanted and problems.

 @example
 ```ts
 filterHostAnswers({ answers, protectedKeys: new Set() });
 ```
 */
export function filterHostAnswers(
  {
    answers,
    protectedKeys,
  }: {
    readonly answers: readonly HostAnswer[];
    readonly protectedKeys: ReadonlySet<string>;
  },
): FilteredAnswers {
  /**
   Problems collected without stopping.
   */
  const failures: string[] = [];
  /**
   Rules by identity,
   merged across hosts.
   */
  const merged = new Map<string, WantedRule>();
  for (const {
    host,
    result,
  } of answers) {
    if (result.failure !== undefined)
      failures.push(result.failure,);
    /**
     Addresses of this host that passed every check,
     counted toward the per-host cap.
     Refused and malformed answers do not use up the cap,
     so a resolver cannot hide the real addresses behind unusable ones.
     */
    const usable = new Set<string>();
    /**
     Usable addresses of this host beyond the cap.
     */
    const cut = { count: 0, };
    for (const item of result.addresses) {
      /**
       Normalized address text.
       */
      const address = normalizeAddress({
        family: item.family,
        text: item.address,
      },);
      if (address === '') {
        failures.push(`ignored malformed ${item.family} answer ${item.address} for ${host}`,);
        continue;
      }
      if (!isPublicAddress({
        family: item.family,
        address,
      },)) {
        failures.push(`refused non-public answer ${address} for ${host}`,);
        continue;
      }
      /**
       Identity of the rule this answer would need.
       */
      const rule: OwnedHostRule = {
        family: item.family,
        address,
      };
      /**
       Identity key.
       */
      const key = ruleKey({ rule, },);
      if (protectedKeys.has(key,)) {
        failures.push(`refused ${address} for ${host}: the tunnel depends on that address as a DNS server or a peer endpoint`,);
        continue;
      }
      if (usable.has(key,))
        continue;
      if (usable.size >= MAX_ADDRESSES_PER_HOST) {
        usable.add(key,);
        cut.count += 1;
        continue;
      }
      usable.add(key,);
      /**
       Earlier answer for the same address.
       */
      const earlier = merged.get(key,);
      merged.set(
        key,
        {
          rule,
          ttlSeconds: earlier === undefined ? item.ttlSeconds : Math.min(
            earlier.ttlSeconds,
            item.ttlSeconds,
          ),
          hosts: mergeHosts({
            ...(earlier === undefined ? {} : { earlier, }),
            host,
          },),
        },
      );
    }
    if (cut.count > 0)
      failures.push(
        `${host} answered ${String(usable.size,)} usable addresses; only the first ${String(MAX_ADDRESSES_PER_HOST,)} get rules`,
      );
  }
  /**
   Rules in answer order.
   */
  const all = [...merged.values(),];
  if (all.length > MAX_ADDRESSES_TOTAL)
    failures.push(
      `the update hosts answered ${String(all.length,)} distinct addresses; only the first ${String(MAX_ADDRESSES_TOTAL,)} get rules `
        + `and the other ${String(all.length - MAX_ADDRESSES_TOTAL,)} use the tunnel`,
    );
  return {
    wanted: all.slice(
      0,
      MAX_ADDRESSES_TOTAL,
    ),
    failures,
  };
}
