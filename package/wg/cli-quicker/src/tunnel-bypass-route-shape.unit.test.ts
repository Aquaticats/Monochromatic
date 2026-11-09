import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  isCopyableDefault,
  isOwnedRoute,
  projectRoute,
  UNREACHABLE_METRIC,
} from '../dist/final/node/tunnel-bypass-route-shape.mjs';

/**
 Table the routes of these tests claim.
 */
const TABLE = 52_000;

/**
 Splits a displayed route into tokens.

 @param line - Route text.

 @returns Its words.

 @example
 ```ts
 words('default dev eth0');
 ```
 */
function words(line: string,): readonly string[] {
  return line.split(' ',);
}

/**
 Routes the writers produce,
 which the grammar must accept.
 */
const OWNED: readonly (readonly [label: string, line: string])[] = [
  ['a default with a gateway, device and metric', 'default via 192.0.2.1 dev wlp9s0 metric 600 table 52000 proto 201',],
  ['a default without a gateway', 'default dev eth0 table 52000 proto 201',],
  ['an IPv6 default', 'default via fe80::1 dev eth0 metric 1024 table 52000 proto 201',],
  ['an on-link default', 'default via 192.0.2.1 dev eth0 onlink table 52000 proto 201',],
  ['the unreachable fallback', `unreachable default metric ${String(UNREACHABLE_METRIC,)} table 52000 proto 201`,],
  ['the keywords in another order', 'default table 52000 proto 201 dev eth0',],
];

/**
 Routes the grammar must refuse,
 each with the reason it is refused.
 `ip` reads a keyword by any prefix and lets a later keyword overwrite an earlier one
 (iproute2 e4cd9f1: `lib/utils.c:909-919` `matches()`, `ip/iproute.c:1467-1473` `protocol`, `ip/iproute.c:1474-1485` `table`,
 `ip/iproute.c:1487-1500` `vrf`, `ip/iproute.c:1548-1573` any other word is a destination),
 so each of the first group would send a root `ip route delete` to a route the bypass did not install.
 */
const REFUSED: readonly (readonly [label: string, line: string])[] = [
  ['a later protocol keyword that overwrites the protocol', 'default table 52000 proto 201 protocol boot',],
  ['an abbreviated protocol keyword', 'default table 52000 proto 201 pro 0',],
  ['the shortest protocol keyword', 'default table 52000 proto 201 p 0',],
  ['a protocol written in hexadecimal', 'default table 52000 proto 0xc9',],
  ['a protocol that is another number', 'default table 52000 proto 202',],
  ['the protocol repeated with another value', 'default table 52000 proto 201 proto 4',],
  ['the protocol repeated with the same value', 'default table 52000 proto 201 proto 201',],
  ['the long protocol keyword alone', 'default table 52000 protocol 201',],
  ['the table repeated', 'default table 52000 proto 201 table 52000',],
  ['the table repeated with another table', 'default table 52000 proto 201 table 254',],
  ['an abbreviated table keyword', 'default table 52000 proto 201 tab main',],
  ['a shorter abbreviated table keyword', 'default table 52000 proto 201 ta 254',],
  ['a table written in hexadecimal', 'default table 0xcb20 proto 201',],
  ['a table that is not the claimed one', 'default table 52001 proto 201',],
  ['a vrf keyword that moves the table', 'default table 52000 proto 201 vrf vrf0',],
  ['an abbreviated vrf keyword', 'default table 52000 proto 201 v vrf0',],
  ['a nexthop group', 'default table 52000 proto 201 nexthop via 192.0.2.1 dev eth0',],
  ['a nexthop object', 'default nhid 12 table 52000 proto 201',],
  ['a second destination', 'default table 52000 proto 201 10.0.0.0/8',],
  ['a type word', 'default table 52000 proto 201 blackhole',],
  ['an unknown keyword', 'default table 52000 proto 201 realm 5',],
  ['an option token', '-force default table 52000 proto 201',],
  ['an option among the tokens', 'default -batch table 52000 proto 201',],
  ['a missing destination', 'dev eth0 table 52000 proto 201',],
  ['another destination', '10.0.0.0/8 dev eth0 table 52000 proto 201',],
  ['an unreachable route that is not a default', 'unreachable 10.0.0.0/8 table 52000 proto 201',],
  ['an unreachable fallback with another metric', 'unreachable default metric 1 table 52000 proto 201',],
  ['an unreachable fallback with a device', 'unreachable default dev eth0 metric 42760 table 52000 proto 201',],
  ['a blackhole default', 'blackhole default table 52000 proto 201',],
  ['no table', 'default dev eth0 proto 201',],
  ['no protocol', 'default dev eth0 table 52000',],
  ['a table keyword without a value', 'default proto 201 table',],
  ['a metric that is not a plain number', 'default metric 0x10 table 52000 proto 201',],
  ['a metric with a leading zero', 'default metric 0600 table 52000 proto 201',],
  ['a metric beyond 32 bits', 'default metric 4294967296 table 52000 proto 201',],
  ['a repeated metric', 'default metric 1 metric 2 table 52000 proto 201',],
  ['an abbreviated metric keyword', 'default met 1 table 52000 proto 201',],
  ['a gateway that is not an address', 'default via eth0 table 52000 proto 201',],
  ['a gateway with an address family word', 'default via inet6 fe80::1 table 52000 proto 201',],
  ['a gateway with a zone', 'default via fe80::1%eth0 table 52000 proto 201',],
  ['a repeated gateway', 'default via 192.0.2.1 via 192.0.2.2 table 52000 proto 201',],
  ['a device that is not an interface name', 'default dev eth0;id table 52000 proto 201',],
  ['a device spelled oif', 'default oif eth0 table 52000 proto 201',],
  ['a repeated device', 'default dev eth0 dev eth1 table 52000 proto 201',],
  ['a repeated on-link flag', 'default onlink onlink table 52000 proto 201',],
  ['an empty token', 'default  table 52000 proto 201',],
];

await describe({
  name: isOwnedRoute.name,
  children: [
    ...OWNED.map(function ownedCase([label, line,],): ReturnType<typeof it> {
      return it({
        name: `accepts ${label}`,
        fn: async () => {
          expect(isOwnedRoute({ tokens: words(line,), table: TABLE, },),).toBe(true,);
        },
      },);
    },),
    ...REFUSED.map(function refusedCase([label, line,],): ReturnType<typeof it> {
      return it({
        name: `refuses ${label}`,
        fn: async () => {
          expect(isOwnedRoute({ tokens: words(line,), table: TABLE, },),).toBe(false,);
        },
      },);
    },),
    it({
      name: 'refuses a route with no tokens, and a route whose value is missing at the end',
      fn: async () => {
        expect(isOwnedRoute({ tokens: [], table: TABLE, },),).toBe(false,);
        expect(isOwnedRoute({ tokens: ['default', 'table', '52000', 'proto', '201', 'dev',], table: TABLE, },),).toBe(false,);
      },
    },),
  ],
},);

await describe({
  name: projectRoute.name,
  children: [
    it({
      name: 'drops what the display adds and keeps what the bypass table carries, in order',
      fn: async () => {
        expect(projectRoute({ tokens: words('default via 192.168.253.1 dev wlp9s0 src 192.168.253.108 metric 600',), },),).toEqual(words('default via 192.168.253.1 dev wlp9s0 metric 600',),);
        expect(projectRoute({ tokens: words('default via fe80::1 dev wlp9s0 metric 600 pref medium',), },),).toEqual(words('default via fe80::1 dev wlp9s0 metric 600',),);
        expect(projectRoute({ tokens: words('default via 192.0.2.1 dev eth0 onlink linkdown scope global mtu 1400 hoplimit 64',), },),).toEqual(words('default via 192.0.2.1 dev eth0 onlink',),);
      },
    },),
    it({
      name: 'drops the device of the unreachable fallback, and keeps every word it does not know',
      fn: async () => {
        expect(projectRoute({ tokens: words('unreachable default dev lo metric 42760 pref medium',), },),).toEqual(words('unreachable default metric 42760',),);
        expect(projectRoute({ tokens: words('default table 52000 proto 201 protocol boot',), },),).toEqual(words('default table 52000 proto 201 protocol boot',),);
        expect(projectRoute({ tokens: words('default nexthop via 192.0.2.1',), },),).toEqual(words('default nexthop via 192.0.2.1',),);
      },
    },),
    it({
      name: 'reads the value of a kept keyword as a value even when it spells a dropped keyword',
      fn: async () => {
        expect(projectRoute({ tokens: words('default dev src metric 5',), },),).toEqual(words('default dev src metric 5',),);
      },
    },),
  ],
},);

await describe({
  name: isCopyableDefault.name,
  children: [
    it({
      name: 'accepts the physical defaults of this machine after the display attributes are dropped, and refuses a form the grammar does not carry',
      fn: async () => {
        expect(isCopyableDefault({ tokens: projectRoute({ tokens: words('default via 192.168.253.1 dev wlp9s0 src 192.168.253.108 metric 600',), },), },),).toBe(true,);
        expect(isCopyableDefault({ tokens: projectRoute({ tokens: words('default via fe80::1 dev wlp9s0 metric 600 pref medium',), },), },),).toBe(true,);
        expect(isCopyableDefault({ tokens: projectRoute({ tokens: words('default nhid 12',), },), },),).toBe(false,);
        expect(isCopyableDefault({ tokens: projectRoute({ tokens: words('default via 192.0.2.1 dev eth0 weight 1',), },), },),).toBe(false,);
        expect(isCopyableDefault({ tokens: words('default table 52000',), },),).toBe(false,);
      },
    },),
  ],
},);
