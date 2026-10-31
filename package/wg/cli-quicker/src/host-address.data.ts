/**
 IPv4 ranges that are never public unicast:
 this-network,
 private,
 shared address space,
 loopback,
 link-local,
 multicast,
 and reserved.
 */
export const BLOCKED_IPV4_RANGES: readonly string[] = [
  '0.0.0.0/8',
  '10.0.0.0/8',
  '100.64.0.0/10',
  '127.0.0.0/8',
  '169.254.0.0/16',
  '172.16.0.0/12',
  '192.168.0.0/16',
  '224.0.0.0/4',
  '240.0.0.0/4',
];

/**
 IPv6 ranges that are never public unicast:
 unspecified,
 loopback,
 link-local,
 unique local,
 and multicast.
 */
export const BLOCKED_IPV6_RANGES: readonly string[] = [
  '::/128',
  '::1/128',
  'fe80::/10',
  'fc00::/7',
  'ff00::/8',
];
