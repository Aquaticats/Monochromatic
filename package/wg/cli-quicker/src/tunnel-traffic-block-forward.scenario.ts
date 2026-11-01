/**
 Scenario run inside a disposable user and network namespace by `tunnel-traffic-block.integration.test.ts`.
 This namespace is the router that holds the traffic block;
 two network namespaces inside it are a guest, whose packets the router forwards,
 and the outside, which holds the update hosts and answers
 (see `tunnel-traffic-block-trio.ts`).
 The scenario prints which rule of the router's forward chain counted each forwarded packet,
 including a TCP connect from the guest to an update host that fails without the rules for the replies and succeeds with them,
 and which rule of its output chain counted a packet to an update host of the other family.
 It changes nothing outside the namespaces it runs in.

 @module
 */

import { RECOMMENDED_EXEMPT_MARK, } from '../dist/final/node/exempt-mark-reservation.mjs';
import { installTrafficBlock, } from '../dist/final/node/tunnel-traffic-block.mjs';
import type {
  Guest,
  Listener,
} from './tunnel-traffic-block-guest.ts';
import {
  deleteRules,
  probe,
  ruleOrder,
  scratchDirectory,
  type Chain,
  type Probed,
} from './tunnel-traffic-block-scenario-support.ts';
import {
  addIpv6,
  buildTrio,
  giveGuestHostAddresses,
  takeGuestHostAddresses,
  HOST_V4,
  HOST_V6,
  OTHER_V4,
  OTHER_V6,
} from './tunnel-traffic-block-trio.ts';

/**
 Interface name the block is made for.
 */
const INTERFACE_NAME = 'wgforward';

/**
 TCP port the outside listens on.
 */
const LISTEN_PORT = 8_080;

/**
 Seconds ping waits for an answer.
 */
const PING_WAIT_SECONDS = '2';

/**
 What the block lets through:
 the exempt mark and an update host of each family.
 */
const CLEARNET = {
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
  // The router's side towards the outside, where the replies of the update hosts arrive.
  interfaces: [
    {
      family: '-4',
      name: 'vetho0',
    },
    {
      family: '-6',
      name: 'vetho0',
    },
  ],
} as const;

/**
 Python that connects to a TCP port and says whether it did,
 giving up after a short time.
 */
const CONNECT_PROGRAM = [
  'import socket, sys',
  's = socket.socket()',
  's.settimeout(2)',
  'try:',
  '    s.connect((sys.argv[1], int(sys.argv[2])))',
  '    print("connected")',
  'except OSError as error:',
  '    print("failed", error.errno)',
].join('\n',);

/**
 Sends a packet from the guest and reports which rules of the router's forward chain counted a packet.

 @param guest - The guest.

 @param command - Program run in the guest's namespace.

 @param args - Arguments of the program.

 @returns What `probe` reports.

 @example
 ```ts
 await fromGuest({ guest, command: 'ping', args: ['-c1', '10.99.0.7'] });
 ```
 */
function fromGuest(
  {
    guest,
    command,
    args,
  }: {
    readonly guest: Guest;
    readonly command: string;
    readonly args: readonly string[]
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
      command,
      ...args,
    ],
  },);
}

/**
 Sends one echo request from the guest.

 @param guest - The guest.

 @param address - Destination.

 @param family - `-4` or `-6`.

 @param source - Source address to send from,
 which the guest owns,
 or nothing for the guest's own address.

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
    source,
  }: {
    readonly guest: Guest;
    readonly address: string;
    readonly family: '-4' | '-6';
    readonly source?: string
  },
): Promise<Probed> {
  return fromGuest({
    guest,
    command: 'ping',
    args: [
      family,
      '-c1',
      `-W${PING_WAIT_SECONDS}`,
      ...(source === undefined ? [] : [
        '-I',
        source,
      ]),
      address,
    ],
  },);
}

/**
 Connects from the guest to the TCP port of the outside.

 @param guest - The guest.

 @param address - Destination.

 @param listener - Listener of the outside whose port is connected to.

 @returns What `probe` reports,
 where the output says whether the connection was made.

 @example
 ```ts
 await guestConnect({ guest, address: '10.99.0.50', listener });
 ```
 */
function guestConnect(
  {
    guest,
    address,
    listener,
  }: {
    readonly guest: Guest;
    readonly address: string;
    readonly listener: Listener
  },
): Promise<Probed> {
  return fromGuest({
    guest,
    command: 'python3',
    args: [
      '-I',
      '-c',
      CONNECT_PROGRAM,
      address,
      String(listener.port,),
    ],
  },);
}

/**
 Reports which rules of a chain counted a packet that the router itself sent.

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

/**
 Installs the block with what it lets through,
 replacing a block that is in place.

 @example
 ```ts
 await installBlock();
 ```
 */
function installBlock(): Promise<void> {
  return installTrafficBlock({
    interfaceName: INTERFACE_NAME,
    clearnet: CLEARNET,
    makeDirectory: scratchDirectory,
  },);
}

/**
 The router with its guest and its outside.
 */
await using trio = await buildTrio({ port: LISTEN_PORT, });
/**
 The guest behind the router.
 */
const {guest} = trio;
/**
 The listener of the outside.
 */
const {listener} = trio;
await installBlock();
/**
 The rules of the forward chain,
 in order.
 */
const forwardOrder = await ruleOrder({
  interfaceName: INTERFACE_NAME,
  chain: 'forward',
},);
/**
 A forwarded echo request to an update host and its reply.
 */
const hostV4 = await guestPing({
  guest,
  address: HOST_V4,
  family: '-4',
},);
/**
 A forwarded echo request to another address of the outside,
 which the outside would answer if it arrived.
 */
const otherV4 = await guestPing({
  guest,
  address: OTHER_V4,
  family: '-4',
},);
/**
 A connect to another address of the outside,
 where the same listener would answer if the request arrived.
 */
const connectOther = await guestConnect({
  guest,
  address: OTHER_V4,
  listener,
},);
await addIpv6({ trio, },);
/**
 A forwarded IPv6 echo request to an update host and its reply.
 */
const hostV6 = await guestPing({
  guest,
  address: HOST_V6,
  family: '-6',
},);
/**
 A forwarded IPv6 echo request to another address of the outside.
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
await giveGuestHostAddresses({ trio, },);
/**
 A forwarded packet whose source is the IPv4 update host, sent by the guest to an address that is not an update host.
 The guest owns the address, so the source is forged.
 */
const forgedV4 = await guestPing({
  guest,
  address: OTHER_V4,
  family: '-4',
  source: HOST_V4,
},);
/**
 The same for the IPv6 update host.
 */
const forgedV6 = await guestPing({
  guest,
  address: OTHER_V6,
  family: '-6',
  source: HOST_V6,
},);
await takeGuestHostAddresses({ trio, },);
/**
 How many rules for the replies were taken out to put the block back as it was before they existed.
 This comes last because the connect below leaves a half-open connection on the outside,
 whose retransmissions would count for a later probe.
 */
const removedReplyRules = await deleteRules({
  interfaceName: INTERFACE_NAME,
  chain: 'forward',
  kinds: [
    'replies4',
    'replies6',
  ],
},);
/**
 The block without the rules for the replies:
 a TCP connect to the update host sends its request through and loses the answer.
 */
const connectWithoutReplyRules = await guestConnect({
  guest,
  address: HOST_V4,
  listener,
},);
await installBlock();
/**
 The rules of the forward chain after the block was installed again,
 which must be the ones it had before the replies were taken out.
 */
const forwardOrderAfterReinstall = await ruleOrder({
  interfaceName: INTERFACE_NAME,
  chain: 'forward',
},);
/**
 The same connect with the block as it is installed now.
 */
const connectWithReplyRules = await guestConnect({
  guest,
  address: HOST_V4,
  listener,
},);
process.stdout
  .write(`${JSON.stringify({
    forwardOrder,
    forwardOrderAfterReinstall,
    hostV4,
    otherV4,
    connectOther,
    hostV6,
    otherV6,
    outputHostV6,
    forgedV4,
    forgedV6,
    removedReplyRules,
    connectWithoutReplyRules,
    connectWithReplyRules,
  },)}\n`,);
