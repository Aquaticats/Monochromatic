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
 IPv6 ranges that are never public unicast:
 unspecified,
 loopback,
 IPv4-mapped (in the hex form `::ffff:7f00:1` the kernel prints too),
 the NAT64 translation prefix and its local-use prefix,
 the discard-only prefix,
 the whole IETF protocol assignments block of `2001::/23`
 (Teredo, benchmarking, the AMT prefix, the AS112 prefix, ORCHIDv2 and the documentation block among them),
 the IPv4-IPv6 translation prefix `100:0:0:1::/64`,
 6to4,
 segment-routing identifiers,
 link-local,
 deprecated site-local,
 unique local,
 and multicast.
 Blocking the parent block and not only Teredo matters:
 the watcher installs a route rule for every whole address a resolver answers,
 and `2001:2::/48` (benchmarking) or `2001:20::/28` (ORCHIDv2) are no more public unicast than `2001::/32`.
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
