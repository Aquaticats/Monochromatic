import { isIP, } from 'node:net';

import { isValidInterfaceName, } from './interface-name.ts';
import { BYPASS_ROUTE_PROTOCOL, } from './tunnel-bypass-types.ts';

/**
 Metric of the unreachable fallback default that stands in for a family without a physical default.
 */
export const UNREACHABLE_METRIC = 42_760;

/**
 Largest value of an unsigned 32-bit route field.
 */
const MAX_UINT32 = 0xFF_FF_FF_FF;

/**
 Attributes of a displayed default route that carry a value and that the bypass table does not need:
 the preferred source address, the IPv6 router preference, the scope, the path metrics and the expiry.
 Dropping them is not free, in two ways:
 an IPv6 default with `pref high` or `pref low` is copied as `medium`,
 and the kernel ranks IPv6 defaults of the same metric by that preference;
 and without `src` a marked socket that is not bound to an address, on an interface that has several addresses,
 gets the interface's primary source address instead of the preferred source the physical route named.
 The scope and the path metrics (`mtu`, `hoplimit`, `advmss`) are not copied either,
 so the copy does not carry the path's MTU or hop limit.
 */
const DROPPED_WITH_VALUE: ReadonlySet<string> = new Set([
  'src',
  'pref',
  'scope',
  'mtu',
  'hoplimit',
  'advmss',
  'expires',
],);

/**
 Attributes of a displayed default route that are flags and that the bypass table does not need.
 They are shown by `ip route show` and are not set by `ip route replace`.
 */
const DROPPED_FLAGS: ReadonlySet<string> = new Set([
  'linkdown',
  'dead',
],);

/**
 Attributes that carry a value and are kept or judged by the grammar,
 so their value is never read as a keyword of its own.
 */
const KEPT_WITH_VALUE: ReadonlySet<string> = new Set([
  'via',
  'dev',
  'metric',
  'table',
  'proto',
  'nhid',
],);

/**
 Reduces the tokens of a displayed default route to what the bypass table carries.
 `ip route show` prints attributes that `ip route replace` does not need,
 and the unreachable fallback is shown with a device that its writer never gives it.
 Every other token stays,
 so a form the grammar does not know is still seen,
 and is refused by {@link isOwnedRoute} where it matters.

 @param tokens - Tokens of one displayed default route.

 @returns The tokens without display-only attributes,
 in their order.

 @example
 ```ts
 projectRoute({ tokens: ['default', 'via', '192.0.2.1', 'dev', 'eth0', 'src', '192.0.2.5', 'metric', '600'] });
 // ['default', 'via', '192.0.2.1', 'dev', 'eth0', 'metric', '600']
 ```
 */
export function projectRoute({ tokens, }: { readonly tokens: readonly string[]; },): readonly string[] {
  /**
   Tokens kept so far.
   */
  const kept: string[] = [];
  /**
   Whether the route is the unreachable fallback,
   which has no device.
   */
  const unreachable = tokens[0] === 'unreachable';
  /**
   Cursor over the tokens.
   */
  const cursor = { index: 0, };
  while (cursor.index < tokens.length) {
    /**
     Token under the cursor,
     guaranteed by the loop bound.
     */
    const token = tokens[cursor.index] ?? '';
    if (DROPPED_WITH_VALUE.has(token,) || (unreachable && (token === 'dev'))) {
      cursor.index += 2;
      continue;
    }
    if (DROPPED_FLAGS.has(token,)) {
      cursor.index += 1;
      continue;
    }
    kept.push(token,);
    if (KEPT_WITH_VALUE.has(token,)) {
      /**
       Value of the kept keyword,
       absent when the tokens end after the keyword.
       */
      const value = tokens[cursor.index + 1];
      if (value !== undefined)
        kept.push(value,);
      cursor.index += 2;
      continue;
    }
    cursor.index += 1;
  }
  return kept;
}

/**
 Checks a decimal unsigned 32-bit number written in plain digits,
 the only spelling that `ip` reads the same under every base rule.

 @param value - Candidate value.

 @returns Whether it is such a number.

 @example
 ```ts
 isPlainUint('600'); // true
 ```
 */
function isPlainUint(value: string,): boolean {
  if (value === '')
    return false;
  /**
   Cursor over the written characters.
   */
  const cursor = { index: 0, };
  while (cursor.index < value.length) {
    /**
     Character under the cursor,
     guaranteed by the loop bound.
     */
    const character = value[cursor.index] ?? '';
    if ((character < '0') || (character > '9'))
      return false;
    cursor.index += 1;
  }
  /**
   Whether the number is written without a leading zero,
   the lone `0` aside,
   because `ip` reads a leading zero under a base rule of its own.
   */
  const plainSpelling = (value === '0') || (!value.startsWith('0',));
  return plainSpelling && (Number(value,) <= MAX_UINT32);
}

/**
 Checks an address as `ip` prints it:
 an IPv4 or IPv6 address without a zone.

 @param value - Candidate value.

 @returns Whether it is such an address.

 @example
 ```ts
 isPlainAddress('fe80::1'); // true
 ```
 */
function isPlainAddress(value: string,): boolean {
  return (isIP(value,) !== 0) && (!value.includes('%',));
}

/**
 Checks a route grammar token that is a value,
 by the keyword it follows.

 @param keyword - Keyword the value belongs to.

 @param value - Candidate value.

 @param table - The table the route must name.

 @returns Whether the value is acceptable for the keyword.

 @example
 ```ts
 isValueOf({ keyword: 'metric', value: '600', table: '52000' }); // true
 ```
 */
function isValueOf(
  {
    keyword,
    value,
    table,
  }: {
    readonly keyword: string;
    readonly value: string;
    readonly table: string;
  },
): boolean {
  if (keyword === 'via')
    return isPlainAddress(value,);
  if (keyword === 'dev')
    return isValidInterfaceName({ interfaceName: value, },);
  if (keyword === 'metric')
    return isPlainUint(value,);
  if (keyword === 'table')
    return value === table;
  if (keyword === 'proto')
    return value === String(BYPASS_ROUTE_PROTOCOL,);
  return false;
}

/**
 Checks that tokens are a route that wg-quicker itself writes into its bypass table,
 and nothing else.

 The grammar is positive and exact.
 The route is `default` or `unreachable default`,
 then any of `via <address>`, `dev <name>`, `onlink`, `metric <number>`,
 and exactly `table <claimed table>` and `proto 201`,
 each keyword at most once and in the one spelling the writers use.
 The unreachable fallback takes only `metric 42760`.
 `ip` reads a keyword by any prefix and lets a later keyword overwrite an earlier one
 (iproute2 e4cd9f1, `lib/utils.c:909-919` `matches()`; `ip/iproute.c:1467-1473` `protocol`,
 `ip/iproute.c:1474-1485` `table` and `ip/iproute.c:1487-1500` `vrf` all overwrite the route's protocol or table;
 `rtnl_rtprot_a2n` and `rtnl_rttable_a2n` in `lib/rt_names.c:267` and `:552` also read names and any number base),
 so a spelling or a repeat that the writers never use could redirect a delete in the table or protocol,
 and an unknown word is read as a destination.
 Every token outside the grammar is therefore refused.

 @param tokens - Candidate route tokens.

 @param table - The table the route must name.

 @returns Whether the tokens are exactly such a route.

 @example
 ```ts
 isOwnedRoute({ tokens: ['default', 'dev', 'eth0', 'table', '52000', 'proto', '201'], table: 52_000 }); // true
 ```
 */
export function isOwnedRoute(
  {
    tokens,
    table,
  }: {
    readonly tokens: readonly string[];
    readonly table: number;
  },
): boolean {
  /**
   Spelling of the claimed table in the route.
   */
  const tableText = String(table,);
  /**
   Whether the route is the unreachable fallback.
   */
  const unreachable = tokens[0] === 'unreachable';
  if (tokens[unreachable ? 1 : 0] !== 'default')
    return false;
  /**
   Keywords seen so far.
   */
  const seen = new Set<string>();
  /**
   Cursor after the head of the route.
   */
  const cursor = { index: unreachable ? 2 : 1, };
  while (cursor.index < tokens.length) {
    /**
     Keyword under the cursor,
     guaranteed by the loop bound.
     */
    const keyword = tokens[cursor.index] ?? '';
    if (seen.has(keyword,))
      return false;
    seen.add(keyword,);
    if (keyword === 'onlink') {
      if (unreachable)
        return false;
      cursor.index += 1;
      continue;
    }
    /**
     Value of the keyword.
     */
    const value = tokens[cursor.index + 1];
    if ((value === undefined) || (!isValueOf({
      keyword,
      value,
      table: tableText,
    },)))
      return false;
    if (unreachable && (keyword !== 'table')
      && (keyword !== 'proto')
      && ((keyword !== 'metric') || (value !== String(UNREACHABLE_METRIC,))))
      return false;
    cursor.index += 2;
  }
  return seen.has('table',) && seen.has('proto',);
}

/**
 Checks that a displayed physical default route can be copied into the bypass table:
 after the display-only attributes are dropped,
 it is a route the grammar of {@link isOwnedRoute} allows once the bypass table and protocol are added.

 @param tokens - Tokens of a physical default route as {@link projectRoute} left them,
 without table and protocol.

 @returns Whether the route can be copied.

 @example
 ```ts
 isCopyableDefault({ tokens: ['default', 'via', '192.0.2.1', 'dev', 'eth0', 'metric', '600'] }); // true
 ```
 */
export function isCopyableDefault({ tokens, }: { readonly tokens: readonly string[]; },): boolean {
  /**
   Any claimed table works for the check,
   because the table is only compared with itself.
   */
  const table = 1;
  return isOwnedRoute({
    tokens: [
      ...tokens,
      'table',
      String(table,),
      'proto',
      String(BYPASS_ROUTE_PROTOCOL,),
    ],
    table,
  },);
}
