/**
 Scenario run inside a disposable user and network namespace by `tunnel-traffic-block.integration.test.ts`.
 This namespace is the router that holds the traffic block;
 a second network namespace inside it is a guest whose packets the router forwards.
 The scenario prints which rule of the router's forward chain counted each forwarded packet,
 and which rule of its output chain counted a packet to an update host of the other family.
 It changes nothing outside the namespaces it runs in.

 @module
 */

import { RECOMMENDED_EXEMPT_MARK, } from '../dist/final/node/exempt-mark-reservation.mjs';
import { run, } from '../dist/final/node/runner.mjs';
import { installTrafficBlock, } from '../dist/final/node/tunnel-traffic-block.mjs';
import {
  inGuest,
  ip,
  linkAddress,
  startGuest,
  type Guest,
} from './tunnel-traffic-block-guest.ts';
import {
  probe,
  ruleOrder,
  scratchDirectory,
  type Chain,
  type Probed,
} from './tunnel-traffic-block-scenario-support.ts';

/**
 Interface name the block is made for.
 */
const INTERFACE_NAME = 'wgforward';

/**
 Update host of the IPv4 family that the block lets through,
 on the router's outside link.
 */
const HOST_V4 = '10.99.0.50';

/**
 Address next to the IPv4 update host that the block does not let through.
 */
const OTHER_V4 = '10.99.0.7';

/**
 Update host of the IPv6 family that the block lets through,
 on the router's outside link.
 */
const HOST_V6 = 'fd99::50';

/**
 Address next to the IPv6 update host that the block does not let through.
 */
const OTHER_V6 = 'fd99::7';

/**
 Seconds ping waits for an answer that never comes.
 */
const PING_WAIT_SECONDS = '1';

/**
 Sends one echo request from the guest and reports which rule of the router's forward chain counted a packet.

 @param guest - The guest.

 @param address - Destination.

 @param family - `-4` or `-6`.

 @returns What `probe` reports.

 @example
 ```ts
 await guestPing({ guest, address: '10.99.0.7', family: '-4' });
 ```
 */
function guestPing(
  {
    guest,
    address,
    family,
  }: {
    readonly guest: Guest;
    readonly address: string;
    readonly family: '-4' | '-6'
  },
): Promise<Probed> {
  return probe({
    interfaceName: INTERFACE_NAME,
    chain: 'forward',
    command: 'nsenter',
    args: [
      '--target',
      String(guest.pid,),
      '--net',
      'ping',
      family,
      '-c1',
      `-W${PING_WAIT_SECONDS}`,
      address,
    ],
  },);
}

/**
 Reports which rule of a chain counted a packet that the router itself sent.

 @param chain - Chain whose counters are read.

 @param address - Destination.

 @returns What `probe` reports.

 @example
 ```ts
 await routerPing({ chain: 'output', address: 'fd99::50' });
 ```
 */
function routerPing({
  chain,
  address,
}: {
  readonly chain: Chain;
  readonly address: string
},): Promise<Probed> {
  return probe({
    interfaceName: INTERFACE_NAME,
    chain,
    command: 'ping',
    args: [
      '-c1',
      `-W${PING_WAIT_SECONDS}`,
      address,
    ],
  },);
}

// IPv6 stays off until the IPv4 half is done, so that router solicitations cannot disturb the counters.
await run({
  command: 'sysctl',
  args: [
    '-w',
    'net.ipv4.ip_forward=1',
    'net.ipv6.conf.all.disable_ipv6=1',
    'net.ipv6.conf.default.disable_ipv6=1',
  ],
},);
await ip([
  'link',
  'set',
  'lo',
  'up',
],);
await ip([
  'link',
  'add',
  'dummy0',
  'type',
  'dummy',
],);
await ip([
  'addr',
  'add',
  '10.99.0.1/24',
  'dev',
  'dummy0',
],);
await ip([
  'link',
  'set',
  'dummy0',
  'up',
],);
await ip([
  'link',
  'add',
  'veth0',
  'type',
  'veth',
  'peer',
  'name',
  'veth1',
],);
await ip([
  'addr',
  'add',
  '10.98.0.1/24',
  'dev',
  'veth0',
],);
await ip([
  'link',
  'set',
  'veth0',
  'up',
],);
/**
 The guest whose packets the router forwards.
 */
await using guest = await startGuest();
await ip([
  'link',
  'set',
  'veth1',
  'netns',
  String(guest.pid,),
],);
await inGuest({
  guest,
  command: 'ip',
  args: [
    'link',
    'set',
    'lo',
    'up',
  ],
},);
await inGuest({
  guest,
  command: 'ip',
  args: [
    'addr',
    'add',
    '10.98.0.2/24',
    'dev',
    'veth1',
  ],
},);
await inGuest({
  guest,
  command: 'ip',
  args: [
    'link',
    'set',
    'veth1',
    'up',
  ],
},);
await inGuest({
  guest,
  command: 'ip',
  args: [
    'route',
    'add',
    'default',
    'via',
    '10.98.0.1',
  ],
},);
await installTrafficBlock({
  interfaceName: INTERFACE_NAME,
  clearnet: {
    mark: RECOMMENDED_EXEMPT_MARK,
    hosts: [
      {
        family: '-4',
        address: HOST_V4,
      },
      {
        family: '-6',
        address: HOST_V6,
      },
    ],
  },
  makeDirectory: scratchDirectory,
},);
/**
 The rules of the forward chain,
 in order.
 */
const forwardOrder = await ruleOrder({
  interfaceName: INTERFACE_NAME,
  chain: 'forward',
},);
/**
 A forwarded packet to an update host.
 */
const hostV4 = await guestPing({
  guest,
  address: HOST_V4,
  family: '-4',
},);
/**
 A forwarded packet to another address of the same link.
 */
const otherV4 = await guestPing({
  guest,
  address: OTHER_V4,
  family: '-4',
},);
// The IPv6 half: neighbors are fixed by hand, because the router's neighbor advertisements leave through the dropped output chain.
await run({
  command: 'sysctl',
  args: [
    '-w',
    'net.ipv6.conf.all.disable_ipv6=0',
    'net.ipv6.conf.default.disable_ipv6=0',
    'net.ipv6.conf.all.forwarding=1',
  ],
},);
await ip([
  'addr',
  'add',
  'fd98::1/64',
  'dev',
  'veth0',
  'nodad',
],);
await ip([
  'addr',
  'add',
  'fd99::1/64',
  'dev',
  'dummy0',
  'nodad',
],);
/**
 Hardware address of the router's end of the link.
 */
const routerAddress = await linkAddress('veth0',);
await inGuest({
  guest,
  command: 'sysctl',
  args: [
    '-w',
    'net.ipv6.conf.all.disable_ipv6=0',
  ],
},);
await inGuest({
  guest,
  command: 'ip',
  args: [
    '-6',
    'addr',
    'add',
    'fd98::2/64',
    'dev',
    'veth1',
    'nodad',
  ],
},);
await inGuest({
  guest,
  command: 'ip',
  args: [
    '-6',
    'neigh',
    'replace',
    'fd98::1',
    'lladdr',
    routerAddress,
    'dev',
    'veth1',
    'nud',
    'permanent',
  ],
},);
await inGuest({
  guest,
  command: 'ip',
  args: [
    '-6',
    'route',
    'add',
    'default',
    'via',
    'fd98::1',
    'dev',
    'veth1',
  ],
},);
/**
 A forwarded IPv6 packet to an update host.
 */
const hostV6 = await guestPing({
  guest,
  address: HOST_V6,
  family: '-6',
},);
/**
 A forwarded IPv6 packet to another address of the same link.
 */
const otherV6 = await guestPing({
  guest,
  address: OTHER_V6,
  family: '-6',
},);
/**
 A packet of the router itself to an IPv6 update host,
 counted by the output chain.
 */
const outputHostV6 = await routerPing({
  chain: 'output',
  address: HOST_V6,
},);
process.stdout
  .write(`${JSON.stringify({
    forwardOrder,
    hostV4,
    otherV4,
    hostV6,
    otherV6,
    outputHostV6,
  },)}\n`,);
