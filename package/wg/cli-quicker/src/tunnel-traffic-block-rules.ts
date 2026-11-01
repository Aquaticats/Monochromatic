import { isIP, } from 'node:net';

import { TrafficBlockError, } from './errors.ts';
import { formatMark, } from './exempt-mark-reservation.ts';
import type { Clearnet, } from './tunnel-clearnet.ts';

/**
 Largest packet mark: the mark is 32 bits.
 */
const MAX_MARK = 0xFF_FF_FF_FF;

/**
 Address families of the clearnet destinations,
 with the nft keyword and the IP version of each.
 */
const DESTINATION_FAMILIES = [
  {
    family: '-4',
    keyword: 'ip',
    version: 4,
  },
  {
    family: '-6',
    keyword: 'ip6',
    version: 6,
  },
] as const;

/**
 The match that restricts a rule to the reply direction of a tracked flow.
 It comes before the address match of the rules for the packets from the update hosts.
 */
const REPLY_ONLY = 'ct direction reply ';

/**
 The chains of the block that let the update hosts through.
 */
export type BlockChain = 'output' | 'forward';

/**
 The rule that lets through a packet whose mark is the exempt mark,
 which is exactly what the policy rule `fwmark <mark> table <bypass table>` matches
 (that rule carries no mask, so neither does this one).
 The mark reaches the packet from the socket of this machine,
 where the cgroup programs of the updates slice and of the exempted applications set it,
 so only the output chain gets this rule:
 a forwarded packet carries no socket of this machine,
 and its mark is not one that this machine's sockets set.
 The kernel refuses to set the mark without `CAP_NET_RAW` or `CAP_NET_ADMIN` in the user namespace of the socket's network namespace
 (`sk_setsockopt` and `sock_cmsg_send` in `net/core/sock.c` of Linux v7.2.11, lines 1527 to 1532 and 3041 to 3044),
 so an unprivileged process cannot give its packets the mark.
 A process in a user and network namespace of its own can mark its own sockets there,
 but its packets reach this host through the forward hook,
 which the mark rule is not in.

 @param table - Table that holds the block.

 @param clearnet - What the routing sends around the tunnel.

 @returns The nft rule,
 empty when the interface has no exempt mark.

 @throws {@link TrafficBlockError} when the mark is not a 32-bit packet mark,
 because it is written into the script.

 @example
 ```ts
 markAcceptRules({ table: 'wg-quicker-wg0-blocked', clearnet: { mark: 100, hosts: [] } });
 ```
 */
export function markAcceptRules({
  table,
  clearnet,
}: {
  readonly table: string;
  readonly clearnet: Clearnet
},): readonly string[] {
  if (clearnet.mark === undefined)
    return [];
  if ((!Number.isInteger(clearnet.mark,)) || (clearnet.mark < 0)
    || (clearnet.mark > MAX_MARK))
    throw new TrafficBlockError(`the exempt mark ${String(clearnet.mark,)} is not a 32-bit packet mark, so it cannot be let through the traffic block`,);
  return [`add rule inet ${table} output meta mark ${formatMark({ mark: clearnet.mark, },)} counter accept`,];
}

/**
 Which address of the packet a host rule matches.
 */
type HostDirection = 'daddr' | 'saddr';

/**
 The rules that match the update-host addresses in one address field of a chain,
 one rule for each address family that has addresses.

 @param table - Table that holds the block.

 @param chain - Chain the rules are added to.

 @param direction - Address field the hosts are matched in.

 @param clearnet - What the routing sends around the tunnel.

 @returns The nft rules,
 empty when there are no update hosts.

 @throws {@link TrafficBlockError} when an address is not an address of its family,
 because it is written into the script.

 @example
 ```ts
 hostRules({ table: 'wg-quicker-wg0-blocked', chain: 'forward', direction: 'saddr', clearnet: { hosts: [] } });
 ```
 */
function hostRules(
  {
    table,
    chain,
    direction,
    clearnet,
  }: {
    readonly table: string;
    readonly chain: BlockChain;
    readonly direction: HostDirection;
    readonly clearnet: Clearnet;
  },
): readonly string[] {
  /**
   Rules in the order they are applied.
   */
  const rules: string[] = [];
  for (const {
    family,
    keyword,
    version,
  } of DESTINATION_FAMILIES) {
    /**
     Destinations of this family without repeats,
     which a set literal refuses.
     */
    const addresses = [...new Set(clearnet.hosts
      .filter(function ofFamily(host,): boolean {
        return host.family === family;
      },)
      .map(function addressOf(host,): string {
        return host.address;
      },),),];
    for (const address of addresses) {
      if (isIP(address,) !== version)
        throw new TrafficBlockError(`${JSON.stringify(address,)} is not an IPv${String(version,)} address, so it cannot be let through the traffic block`,);
    }
    if (addresses.length > 0)
      rules.push(`add rule inet ${table} ${chain} ${direction === 'saddr' ? REPLY_ONLY : ''}${keyword} ${direction} { ${addresses.join(', ',)} } counter accept`,);
  }
  return rules;
}

/**
 The rules that let through a packet to one of the update-host destinations of the rules `to <address> table <bypass table>`,
 one rule for each address family that has addresses.
 They go into the output chain,
 for this machine's own traffic,
 and into the forward chain,
 so that guests behind this machine reach those hosts directly while the block stands.

 @param table - Table that holds the block.

 @param chain - Chain the rules are added to.

 @param clearnet - What the routing sends around the tunnel.

 @returns The nft rules,
 empty when there are no update hosts.

 @throws {@link TrafficBlockError} when an address is not an address of its family,
 because it is written into the script.

 @example
 ```ts
 hostAcceptRules({ table: 'wg-quicker-wg0-blocked', chain: 'forward', clearnet: { hosts: [] } });
 ```
 */
export function hostAcceptRules(
  {
    table,
    chain,
    clearnet,
  }: {
    readonly table: string;
    readonly chain: BlockChain;
    readonly clearnet: Clearnet;
  },
): readonly string[] {
  return hostRules({
    table,
    chain,
    direction: 'daddr',
    clearnet,
  },);
}

/**
 The rules that let the update hosts answer the guests behind this machine:
 a forwarded packet from one of the update-host addresses that is a reply of a tracked flow.
 A guest's connection to an update host needs both directions through the forward chain,
 and a rule on the destination alone passes the request and drops every reply.
 A rule on the source address alone would pass any forwarded packet that carries an update host's address as its source,
 whoever sent it: a guest that owns such an address could send anything anywhere through it.
 The rule therefore matches `ct direction reply` as well:
 only a packet that the connection tracker sees as the reply direction of a flow, which a guest's own packet is not,
 and only from an update host.
 It does not match the state of the connection (`established`),
 which would also pass the guests' flows that were bound for the tunnel once the routing is gone.
 A packet from an update host that starts a flow is the original direction of a new flow and is dropped.
 A flow that was already running when the block was installed is known to the tracker only if the tracker was already running;
 the rule needs the tracker, and nft turns it on for this namespace when the rule is loaded.
 They go into the forward chain only:
 the replies to this machine's own packets arrive through the input hook,
 which the block does not filter.

 @param table - Table that holds the block.

 @param clearnet - What the routing sends around the tunnel.

 @returns The nft rules,
 empty when there are no update hosts.

 @throws {@link TrafficBlockError} when an address is not an address of its family,
 because it is written into the script.

 @example
 ```ts
 hostReplyRules({ table: 'wg-quicker-wg0-blocked', clearnet: { hosts: [] } });
 ```
 */
export function hostReplyRules({
  table,
  clearnet,
}: {
  readonly table: string;
  readonly clearnet: Clearnet
},): readonly string[] {
  return hostRules({
    table,
    chain: 'forward',
    direction: 'saddr',
    clearnet,
  },);
}
