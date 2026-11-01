/**
 The three network namespaces of the forward scenario:
 this one is the router that holds the traffic block,
 a guest behind it sends packets that the router forwards,
 and the outside holds the update hosts and answers.

 @module
 */

import { run, } from '../dist/final/node/runner.mjs';
import {
  inGuest,
  ip,
  linkAddress,
  startGuest,
  startListener,
  type Guest,
  type Listener,
} from './tunnel-traffic-block-guest.ts';

/**
 Update host of the IPv4 family on the outside.
 */
export const HOST_V4 = '10.99.0.50';

/**
 Address next to the IPv4 update host on the outside.
 */
export const OTHER_V4 = '10.99.0.7';

/**
 Update host of the IPv6 family on the outside.
 */
export const HOST_V6 = 'fd99::50';

/**
 Address next to the IPv6 update host on the outside.
 */
export const OTHER_V6 = 'fd99::7';

/**
 The router with its guest and its outside.
 */
export type Trio = {
  /**
   The guest whose packets the router forwards.
   */
  readonly guest: Guest;

  /**
   The outside, which holds the update hosts.
   */
  readonly outside: Guest;

  /**
   The TCP listener of the outside.
   */
  readonly listener: Listener;

  /**
   Stops the listener and both namespaces.
   */
  [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Runs `ip` in this namespace.

 @param line - Arguments of `ip`, separated by single spaces.

 @example
 ```ts
 await here('link set lo up');
 ```
 */
async function here(line: string,): Promise<void> {
  await ip(line.split(' ',),);
}

/**
 Runs `ip` in a guest's namespace.

 @param guest - Guest whose namespace gets the command.

 @param line - Arguments of `ip`, separated by single spaces.

 @example
 ```ts
 await within({ guest, line: 'link set lo up' });
 ```
 */
async function within({
  guest,
  line,
}: {
  readonly guest: Guest;
  readonly line: string
},): Promise<void> {
  await inGuest({
    guest,
    command: 'ip',
    args: line.split(' ',),
  },);
}

/**
 Builds the three namespaces with IPv4 addresses and routes,
 forwarding on in the router,
 and a TCP listener on every address of the outside.

 @param port - TCP port the outside listens on.

 @returns The trio,
 disposed by `await using`.

 @example
 ```ts
 await using trio = await buildTrio({ port: 8080 });
 ```
 */
export async function buildTrio({ port, }: { readonly port: number; },): Promise<Trio> {
  await run({
    command: 'sysctl',
    args: [
      '-w',
      'net.ipv4.ip_forward=1',
      'net.ipv6.conf.all.disable_ipv6=1',
      'net.ipv6.conf.default.disable_ipv6=1',
    ],
  },);
  await here('link set lo up',);
  /**
   The guest whose packets the router forwards.
   */
  const guest = await startGuest();
  /**
   The outside, which holds the update hosts.
   */
  const outside = await startGuest();
  await here('link add vethg0 type veth peer name vethg1',);
  await here('link add vetho0 type veth peer name vetho1',);
  await here(`link set vethg1 netns ${String(guest.pid,)}`,);
  await here(`link set vetho1 netns ${String(outside.pid,)}`,);
  await here('addr add 10.98.0.1/24 dev vethg0',);
  await here('link set vethg0 up',);
  await here('addr add 10.99.0.1/24 dev vetho0',);
  await here('link set vetho0 up',);
  await within({
    guest,
    line: 'link set lo up',
  },);
  await within({
    guest,
    line: 'addr add 10.98.0.2/24 dev vethg1',
  },);
  await within({
    guest,
    line: 'link set vethg1 up',
  },);
  await within({
    guest,
    line: 'route add default via 10.98.0.1',
  },);
  await within({
    guest: outside,
    line: 'link set lo up',
  },);
  await within({
    guest: outside,
    line: `addr add ${HOST_V4}/24 dev vetho1`,
  },);
  await within({
    guest: outside,
    line: `addr add ${OTHER_V4}/24 dev vetho1`,
  },);
  await within({
    guest: outside,
    line: 'link set vetho1 up',
  },);
  await within({
    guest: outside,
    line: 'route add default via 10.99.0.1',
  },);
  /**
   The listener on the outside,
   which answers a connection to any of its addresses.
   */
  const listener = await startListener({
    guest: outside,
    port,
  },);
  return {
    guest,
    outside,
    listener,
    [Symbol.asyncDispose]: async function stopTrio(): Promise<void> {
      await listener[Symbol.asyncDispose]();
      await outside[Symbol.asyncDispose]();
      await guest[Symbol.asyncDispose]();
    },
  };
}

/**
 Turns IPv6 on in the three namespaces and gives them addresses and routes.
 Neighbors are fixed by hand,
 because the neighbor advertisements of the router and the outside would leave through the dropped output chain.

 @param trio - The namespaces.

 @example
 ```ts
 await addIpv6({ trio });
 ```
 */
export async function addIpv6({ trio, }: { readonly trio: Trio; },): Promise<void> {
  /**
   Guest behind the router.
   */
  const {guest} = trio;
  /**
   Outside of the router.
   */
  const {outside} = trio;
  await run({
    command: 'sysctl',
    args: [
      '-w',
      'net.ipv6.conf.all.disable_ipv6=0',
      'net.ipv6.conf.default.disable_ipv6=0',
      'net.ipv6.conf.all.forwarding=1',
    ],
  },);
  await inGuest({
    guest,
    command: 'sysctl',
    args: [
      '-w',
      'net.ipv6.conf.all.disable_ipv6=0',
    ],
  },);
  await inGuest({
    guest: outside,
    command: 'sysctl',
    args: [
      '-w',
      'net.ipv6.conf.all.disable_ipv6=0',
    ],
  },);
  await here('addr add fd98::1/64 dev vethg0 nodad',);
  await here('addr add fd99::1/64 dev vetho0 nodad',);
  await within({
    guest,
    line: '-6 addr add fd98::2/64 dev vethg1 nodad',
  },);
  await within({
    guest: outside,
    line: `-6 addr add ${HOST_V6}/64 dev vetho1 nodad`,
  },);
  await within({
    guest: outside,
    line: `-6 addr add ${OTHER_V6}/64 dev vetho1 nodad`,
  },);
  await within({
    guest,
    line: `-6 neigh replace fd98::1 lladdr ${await linkAddress({ name: 'vethg0', },)} dev vethg1 nud permanent`,
  },);
  await within({
    guest,
    line: '-6 route add default via fd98::1 dev vethg1',
  },);
  await within({
    guest: outside,
    line: `-6 neigh replace fd99::1 lladdr ${await linkAddress({ name: 'vetho0', },)} dev vetho1 nud permanent`,
  },);
  await within({
    guest: outside,
    line: '-6 route add default via fd99::1 dev vetho1',
  },);
  /**
   Hardware address of the outside's end of its link.
   */
  const outsideEnd = await linkAddress({
    name: 'vetho1',
    guest: outside,
  },);
  await here(`-6 neigh replace fd98::2 lladdr ${await linkAddress({
    name: 'vethg1',
    guest,
  },)} dev vethg0 nud permanent`,);
  await here(`-6 neigh replace ${HOST_V6} lladdr ${outsideEnd} dev vetho0 nud permanent`,);
  await here(`-6 neigh replace ${OTHER_V6} lladdr ${outsideEnd} dev vetho0 nud permanent`,);
}
