import {
  isIPv4,
  isIPv6,
} from 'node:net';

import type { BypassProto, } from './tunnel-bypass-types.ts';
import {
  BLOCKED_IPV4_RANGES,
  BLOCKED_IPV6_RANGES,
} from './host-address.data.ts';

/**
 Shortest pause between two DNS refreshes,
 in seconds.
 */
const MIN_REFRESH_SECONDS = 30;

/**
 Longest pause between two DNS refreshes,
 in seconds.
 */
const MAX_REFRESH_SECONDS = 900;

/**
 Bits in an IPv4 address.
 */
const IPV4_BITS = 32;

/**
 Bits in an IPv6 address.
 */
const IPV6_BITS = 128;

/**
 Bits in one IPv4 octet.
 */
const OCTET_BITS = 8;

/**
 Bits in one IPv6 group.
 */
const GROUP_BITS = 16;

/**
 Groups in a full IPv6 address.
 */
const IPV6_GROUPS = 8;

/**
 Radix of an IPv4 octet.
 */
const DECIMAL_RADIX = 10;

/**
 Radix of an IPv6 group.
 */
const HEX_RADIX = 16;

/**
 Splits one side of an IPv6 `::` into its groups.

 @param half - Text on one side of the `::`,
 possibly absent.

 @returns Groups,
 none for an empty side.

 @example
 ```ts
 groupsOf({ half: '2606:4700' }); // ['2606', '4700']
 ```
 */
function groupsOf({ half, }: { readonly half?: string; },): readonly string[] {
  if ((half === undefined) || (half === ''))
    return [];
  return half.split(':',);
}

/**
 Expands an IPv6 literal into eight lowercase hexadecimal groups.

 @param text - IPv6 literal without scope.

 @returns Normalized text,
 or an empty string for an IPv4-mapped or malformed literal.

 @example
 ```ts
 normalizeIpv6({ text: '2606:4700::6810:84e5' }); // '2606:4700:0:0:0:0:6810:84e5'
 ```
 */
function normalizeIpv6({ text, }: { readonly text: string; },): string {
  if (text.includes('.',))
    return '';
  /**
   Halves around the one allowed `::`.
   */
  const halves = text.toLowerCase()
    .split('::',);
  if (halves.length > 2)
    return '';
  /**
   Groups before the `::`.
   */
  const head = groupsOf((halves[0] === undefined ? {} : { half: halves[0], }),);
  /**
   Groups after the `::`.
   */
  const tail = halves.length === 2 ? groupsOf((halves[1] === undefined ? {} : { half: halves[1], }),) : [];
  /**
   Zero groups the `::` stands for.
   */
  const fill = halves.length === 2 ? IPV6_GROUPS - head.length
    - tail.length : 0;
  if (fill < 0)
    return '';
  /**
   All eight groups.
   */
  const groups = [
    ...head,
    ...Array.from(
      { length: fill, },
      function zeroGroup(): string {
        return '0';
      },
    ),
    ...tail,
  ];
  if (groups.length !== IPV6_GROUPS)
    return '';
  return groups
    .map(function trimGroup(group,): string {
      return Number.parseInt(
        group,
        HEX_RADIX,
      )
        .toString(HEX_RADIX,);
    },)
    .join(':',);
}

/**
 Normalizes an address so a DNS answer and a kernel listing compare equal.

 @param family - Address family flag.

 @param text - Address text.

 @returns Normalized text,
 or an empty string when the text is not an address of that family.

 @example
 ```ts
 normalizeAddress({ family: '-4', text: '203.0.113.7' });
 ```
 */
export function normalizeAddress(
  {
    family,
    text,
  }: {
    readonly family: BypassProto;
    readonly text: string;
  },
): string {
  if (family === '-4')
    return isIPv4(text,) ? text : '';
  return isIPv6(text,) ? normalizeIpv6({ text, },) : '';
}

/**
 Converts a normalized address to its integer value.

 @param family - Address family flag.

 @param address - Normalized address.

 @returns Integer of 32 or 128 bits.

 @example
 ```ts
 addressValue({ family: '-4', address: '10.0.0.1' });
 ```
 */
function addressValue(
  {
    family,
    address,
  }: {
    readonly family: BypassProto;
    readonly address: string;
  },
): bigint {
  /**
   Parts of the address,
   octets or groups.
   */
  const parts = family === '-4' ? address.split('.',) : address.split(':',);
  /**
   Bits each part carries.
   */
  const partBits = family === '-4' ? OCTET_BITS : GROUP_BITS;
  /**
   Radix each part is written in.
   */
  const radix = family === '-4' ? DECIMAL_RADIX : HEX_RADIX;
  return parts.reduce(
    function accumulate(
      value: bigint,
      part: string,
    ): bigint {
      return (value << BigInt(partBits,)) + BigInt(Number.parseInt(
        part,
        radix,
      ),);
    },
    0n,
  );
}

/**
 Reports whether an address lies inside a CIDR range.

 @param family - Address family flag.

 @param address - Normalized address.

 @param range - Range such as `10.0.0.0/8`.

 @returns Whether the address is in the range.

 @example
 ```ts
 inRange({ family: '-4', address: '10.1.2.3', range: '10.0.0.0/8' }); // true
 ```
 */
function inRange(
  {
    family,
    address,
    range,
  }: {
    readonly family: BypassProto;
    readonly address: string;
    readonly range: string;
  },
): boolean {
  /**
   Base text and prefix length of the range.
   */
  const [base = '', prefixText = '0',] = range.split('/',);
  /**
   Low bits that the prefix leaves free.
   */
  const free = BigInt((family === '-4' ? IPV4_BITS : IPV6_BITS) - Number(prefixText,),);
  /**
   Base address in normalized form.
   */
  const normalizedBase = normalizeAddress({
    family,
    text: base,
  },);
  return (addressValue({
    family,
    address,
  },) >> free) === (addressValue({
    family,
    address: normalizedBase,
  },) >> free);
}

/**
 Reports whether an address is a public unicast address.

 Loopback,
 private,
 link-local,
 shared,
 unspecified,
 reserved,
 and multicast answers are refused:
 a hostile or broken answer must not route local traffic through the bypass table.

 @param family - Address family flag.

 @param address - Normalized address.

 @returns Whether a rule may be installed for the address.

 @example
 ```ts
 isPublicAddress({ family: '-4', address: '127.0.0.1' }); // false
 ```
 */
export function isPublicAddress(
  {
    family,
    address,
  }: {
    readonly family: BypassProto;
    readonly address: string;
  },
): boolean {
  return !(family === '-4' ? BLOCKED_IPV4_RANGES : BLOCKED_IPV6_RANGES).some(function blocks(range,): boolean {
    return inRange({
      family,
      address,
      range,
    },);
  },);
}

/**
 Clamps the smallest answer TTL into the refresh window.

 @param ttlSeconds - Smallest TTL among the answers,
 or zero when there is none.

 @returns Seconds to wait before the next refresh.

 @example
 ```ts
 refreshSeconds({ ttlSeconds: 5 }); // 30
 ```
 */
export function refreshSeconds({ ttlSeconds, }: { readonly ttlSeconds: number; },): number {
  if (ttlSeconds <= 0)
    return MIN_REFRESH_SECONDS;
  return Math.min(
    MAX_REFRESH_SECONDS,
    Math.max(
      MIN_REFRESH_SECONDS,
      ttlSeconds,
    ),
  );
}
