/**
 Scenario run inside a disposable user and network namespace by `tunnel-traffic-block.integration.test.ts`.
 It installs the traffic block with the exceptions for the clearnet traffic,
 sends real packets of this machine through the real `nft`,
 and prints which rule of the output chain counted each one.
 It changes nothing outside the namespace it runs in.

 @module
 */

import { RECOMMENDED_EXEMPT_MARK, } from '../dist/final/node/exempt-mark-reservation.mjs';
import { run, } from '../dist/final/node/runner.mjs';
import { installTrafficBlock, } from '../dist/final/node/tunnel-traffic-block.mjs';
import {
  probe,
  ruleOrder,
  scratchDirectory,
  type Probed,
} from './tunnel-traffic-block-scenario-support.ts';

/**
 Interface name the block is made for.
 */
const INTERFACE_NAME = 'wgclear';

/**
 Address of the dummy interface.
 */
const LOCAL_ADDRESS = '10.99.0.1';

/**
 Address on the dummy link that nobody answers for and that is not an exception.
 */
const OTHER_ADDRESS = '10.99.0.7';

/**
 Address on the dummy link that is an update-host exception.
 */
const HOST_ADDRESS = '10.99.0.50';

/**
 Update-host exception of the other family,
 loaded by the real `nft` but never sent to in this scenario.
 */
const HOST_ADDRESS_V6 = 'fd99::50';

/**
 Seconds ping waits for an answer that never comes.
 */
const PING_WAIT_SECONDS = '1';

/**
 Python that tries to mark a socket and sends one datagram,
 printing what the kernel said about the mark.
 */
const MARKING_PROGRAM = [
  'import socket, sys',
  's = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)',
  'try:',
  '    s.setsockopt(socket.SOL_SOCKET, socket.SO_MARK, int(sys.argv[1]))',
  '    print("marked")',
  'except OSError as error:',
  '    print("refused", error.errno)',
  `s.sendto(b"x", ("${OTHER_ADDRESS}", 9))`,
].join('\n',);

/**
 Sends one command whose packet the output chain counts.

 @param command - Executable that sends one packet.

 @param args - Arguments.

 @returns What `probe` reports.

 @example
 ```ts
 await outputProbe({ command: 'ping', args: ['-c1', '10.99.0.7'] });
 ```
 */
function outputProbe({
  command,
  args,
}: {
  readonly command: string;
  readonly args: readonly string[]
},): Promise<Probed> {
  return probe({
    interfaceName: INTERFACE_NAME,
    chain: 'output',
    command,
    args,
  },);
}

/**
 Sends one echo request.

 @param address - Destination.

 @param mark - Socket mark to ask `ping` to set,
 or nothing.

 @returns What `probe` reports.

 @example
 ```ts
 await ping({ address: '10.99.0.7', mark: 100 });
 ```
 */
function ping({
  address,
  mark,
}: {
  readonly address: string;
  readonly mark?: number
},): Promise<Probed> {
  return outputProbe({
    command: 'ping',
    args: [
      '-c1',
      `-W${PING_WAIT_SECONDS}`,
      ...(mark === undefined ? [] : [
        '-m',
        String(mark,),
      ]),
      address,
    ],
  },);
}

// IPv6 is switched off so that router solicitations and multicast reports cannot disturb the counters.
await run({
  command: 'sysctl',
  args: [
    '-w',
    'net.ipv6.conf.all.disable_ipv6=1',
    'net.ipv6.conf.default.disable_ipv6=1',
  ],
},);
await run({
  command: 'ip',
  args: [
    'link',
    'set',
    'lo',
    'up',
  ],
},);
await run({
  command: 'ip',
  args: [
    'link',
    'add',
    'dummy0',
    'type',
    'dummy',
  ],
},);
await run({
  command: 'ip',
  args: [
    'addr',
    'add',
    `${LOCAL_ADDRESS}/24`,
    'dev',
    'dummy0',
  ],
},);
await run({
  command: 'ip',
  args: [
    'link',
    'set',
    'dummy0',
    'up',
  ],
},);
await installTrafficBlock({
  interfaceName: INTERFACE_NAME,
  clearnet: {
    mark: RECOMMENDED_EXEMPT_MARK,
    hosts: [
      {
        family: '-4',
        address: HOST_ADDRESS,
      },
      {
        family: '-6',
        address: HOST_ADDRESS_V6,
      },
    ],
    interfaces: [
      {
        family: '-4',
        name: 'dummy0',
      },
      {
        family: '-6',
        name: 'dummy0',
      },
    ],
  },
  makeDirectory: scratchDirectory,
},);
/**
 The rules of the output chain,
 in order.
 */
const outputOrder = await ruleOrder({
  interfaceName: INTERFACE_NAME,
  chain: 'output',
},);
/**
 The rules of the forward chain.
 */
const forwardOrder = await ruleOrder({
  interfaceName: INTERFACE_NAME,
  chain: 'forward',
},);
/**
 A marking program run with the capabilities of the namespace's root.
 */
const withCapabilities = await outputProbe({
  command: 'python3',
  args: [
    '-I',
    '-c',
    MARKING_PROGRAM,
    String(RECOMMENDED_EXEMPT_MARK,),
  ],
},);
/**
 The same program without the capabilities the kernel asks for to set a mark.
 */
const withoutCapabilities = await outputProbe({
  command: 'setpriv',
  args: [
    '--bounding-set=-net_raw,-net_admin',
    'python3',
    '-I',
    '-c',
    MARKING_PROGRAM,
    String(RECOMMENDED_EXEMPT_MARK,),
  ],
},);
/**
 A packet with the exempt mark to an address that is not an exception.
 */
const marked = await ping({
  address: OTHER_ADDRESS,
  mark: RECOMMENDED_EXEMPT_MARK,
},);
/**
 A packet with a mark one above the exempt mark.
 */
const markedPlusOne = await ping({
  address: OTHER_ADDRESS,
  mark: RECOMMENDED_EXEMPT_MARK + 1,
},);
/**
 A packet without a mark.
 */
const unmarked = await ping({ address: OTHER_ADDRESS, },);
/**
 An unmarked packet to an update host.
 */
const host = await ping({ address: HOST_ADDRESS, },);
/**
 An echo request to loopback.
 */
const loopback = await ping({ address: '127.0.0.1', },);
process.stdout
  .write(`${JSON.stringify({
    outputOrder,
    forwardOrder,
    marked,
    markedPlusOne,
    unmarked,
    host,
    loopback,
    withCapabilities,
    withoutCapabilities,
  },)}\n`,);
