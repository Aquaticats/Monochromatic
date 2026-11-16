/**
 IPv4 ranges that are never public unicast:
 this-network,
 private,
 shared address space,
 loopback,
 link-local,
 protocol assignments,
 the three documentation networks,
 the retired 6to4 relay anycast,
 benchmarking,
 multicast,
 and reserved.

 A rule for an address in one of them would route traffic the tunnel's owner never meant to send outside it,
 or would be meaningless on the public internet.
 */
export const BLOCKED_IPV4_RANGES: readonly string[] = [
  '0.0.0.0/8',
  '10.0.0.0/8',
  '100.64.0.0/10',
  '127.0.0.0/8',
  '169.254.0.0/16',
  '172.16.0.0/12',
  '192.0.0.0/24',
  '192.0.2.0/24',
  '192.88.99.0/24',
  '192.168.0.0/16',
  '198.18.0.0/15',
  '198.51.100.0/24',
  '203.0.113.0/24',
  '224.0.0.0/4',
  '240.0.0.0/4',
];

/**
 Addresses inside `2001::/23` that the IANA registry does answer as globally reachable.

 The registry gives `2001::/23` the answer `False` with a footnote,
 and answers each sub-block it lists separately on its own row.
 These seven are the rows of the IPv6 Special-Purpose Address Registry whose
 `Globally Reachable` column says `True` inside the parent block,
 read from
 https://www.iana.org/assignments/iana-ipv6-special-registry/iana-ipv6-special-registry-1.csv
 on 2026-11-11:

 - `2001:1::1/128` (Port Control Protocol Anycast),
 - `2001:1::2/128` (Traversal Using Relays around NAT Anycast),
 - `2001:1::3/128` (DNS-SD Service Registration Protocol Anycast),
 - `2001:3::/32` (AMT),
 - `2001:4:112::/48` (AS112-v6),
 - `2001:20::/28` (ORCHIDv2),
 - `2001:30::/28` (Drone Remote ID Protocol Entity Tags Prefix).

 The rows inside the parent block that answer `False` stay blocked:
 `2001:2::/48` (benchmarking),
 the terminated `2001:10::/28` (previously ORCHID, which carries no answer at all),
 and Teredo, whose answer is not `True`.
 */
export const PUBLIC_IPV6_EXCEPTIONS: readonly string[] = [
  '2001:1::1/128',
  '2001:1::2/128',
  '2001:1::3/128',
  '2001:3::/32',
  '2001:4:112::/48',
  '2001:20::/28',
  '2001:30::/28',
];

/**
 IPv6 ranges that are never public unicast:
 unspecified,
 loopback,
 IPv4-mapped (in the hex form `::ffff:7f00:1` the kernel prints too),
 the NAT64 translation prefix and its local-use prefix,
 the discard-only prefix,
 the whole IETF protocol assignments block of `2001::/23`
 except the sub-blocks {@link PUBLIC_IPV6_EXCEPTIONS} lists
 (Teredo, benchmarking and the deprecated ORCHID block among the ones that stay),
 the IPv4-IPv6 translation prefix `100:0:0:1::/64`,
 6to4,
 segment-routing identifiers,
 link-local,
 deprecated site-local,
 unique local,
 and multicast.
 Blocking the parent block and not only Teredo matters:
 the watcher installs a route rule for every whole address a resolver answers,
 and `2001:2::/48` (benchmarking) or `2001:10::/28` (deprecated ORCHID) are no more public unicast than `2001::/32`.
 The exceptions matter the other way:
 `2001:4:112::1` serves AS112 and `2001:1::1` serves port control,
 and a resolver that answers one of them is answering a globally reachable address.
 */
export const BLOCKED_IPV6_RANGES: readonly string[] = [
  '::/128',
  '::1/128',
  '::ffff:0:0/96',
  '64:ff9b::/96',
  '64:ff9b:1::/48',
  '100::/64',
  '100:0:0:1::/64',
  '2001::/23',
  '2001:db8::/32',
  '2002::/16',
  '3fff::/20',
  '5f00::/16',
  'fe80::/10',
  'fec0::/10',
  'fc00::/7',
  'ff00::/8',
];
