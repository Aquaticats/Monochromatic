/**
 Scenario run inside a disposable user and network namespace by `tunnel-traffic-block.integration.test.ts`.
 It installs the traffic block with the exceptions for the clearnet traffic,
 sends real packets through the real `nft`,
 and prints which rule counted each one.
 It changes nothing outside the namespace it runs in.

 @module
 */

import {
  mkdtemp,
  rm,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

import { RECOMMENDED_EXEMPT_MARK, } from '../dist/final/node/exempt-mark-reservation.mjs';
import {
  run,
  runAllowingFailure,
} from '../dist/final/node/runner.mjs';
import {
  installTrafficBlock,
  trafficBlockTable,
} from '../dist/final/node/tunnel-traffic-block.mjs';

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
 loaded by the real `nft` but never sent to.
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
 Rule kinds the scenario tells apart.
 */
type RuleKind = 'mark' | 'hosts4' | 'hosts6' | 'drop';

/**
 One expression of a rule in the JSON of `nft -j list table`,
 reduced to the parts the scenario reads.
 */
type Expression = {
  readonly match?: {
    readonly left?: {
      readonly meta?: { readonly key?: string; };
      readonly payload?: { readonly protocol?: string; };
    };
  };
  readonly counter?: { readonly packets?: number; };
  readonly drop?: unknown;
};

/**
 One rule of the JSON listing.
 */
type ListedRule = {
  readonly chain?: string;
  readonly expr?: readonly Expression[];
};

/**
 The JSON listing of a table,
 reduced to its rules.
 */
type Listing = {
  readonly nftables: readonly { readonly rule?: ListedRule; }[];
};

/**
 One rule's kind and the packets it counted.
 */
type Counted = {
  readonly kind: RuleKind;
  readonly packets: number;
};

/**
 What one probe saw.
 */
type Probed = {
  /**
   Kinds of rule whose counters grew while the packet was sent.
   */
  readonly counted: readonly RuleKind[];

  /**
   Exit status of the sender.
   */
  readonly status: number;

  /**
   Standard output of the sender.
   */
  readonly output: string;
};

/**
 Narrows parsed JSON to the listing the scenario reads.

 @param value - Parsed JSON.

 @returns Whether it has the list of `nftables` entries.

 @example
 ```ts
 isListing({ nftables: [] }); // true
 ```
 */
function isListing(value: unknown,): value is Listing {
  return ((typeof value) === 'object') && (value !== null)
    && ('nftables' in value)
    && Array.isArray(value.nftables,);
}

/**
 Makes a scratch directory in the test scratch root,
 removed when the scope exits,
 because the real one may belong to another user inside the namespace.

 @returns A disposable directory.

 @example
 ```ts
 await using dir = await scratchDirectory();
 ```
 */
async function scratchDirectory(): ReturnType<NonNullable<Parameters<typeof installTrafficBlock>[0]['makeDirectory']>> {
  /**
   Path of the new directory.
   */
  const path = await mkdtemp(join(
    scratchRoot(),
    'wg-quicker-clearnet-',
  ),);
  return {
    path,
    [Symbol.asyncDispose]: async function removeScratchDirectory(): Promise<void> {
      await rm(
        path,
        {
          recursive: true,
          force: true,
        },
      );
    },
  };
}

/**
 Which kind a listed rule is.

 @param rule - Rule of the listing.

 @returns The kinds it has,
 which is none for a rule the scenario does not know.

 @example
 ```ts
 kindsOf({ rule: { expr: [{ drop: null }] } }); // ['drop']
 ```
 */
function kindsOf({ rule, }: { readonly rule: ListedRule; },): readonly RuleKind[] {
  /**
   Expressions of the rule.
   */
  const expressions = rule.expr ?? [];
  if (expressions.some(function dropsPacket(expression,): boolean {
    return 'drop' in expression;
  },))
    return ['drop',];
  if (expressions.some(function matchesMark(expression,): boolean {
    return expression.match
      ?.left
      ?.meta
      ?.key
      === 'mark';
  },))
    return ['mark',];
  if (expressions.some(function matchesV4(expression,): boolean {
    return expression.match
      ?.left
      ?.payload
      ?.protocol
      === 'ip';
  },))
    return ['hosts4',];
  if (expressions.some(function matchesV6(expression,): boolean {
    return expression.match
      ?.left
      ?.payload
      ?.protocol
      === 'ip6';
  },))
    return ['hosts6',];
  return [];
}

/**
 Reads the packet counters of the rules of one chain.

 @param chain - Chain of the block.

 @returns The kinds in the order of the rules,
 each with the packets it counted.

 @example
 ```ts
 await listCounters({ chain: 'output' });
 ```
 */
async function listCounters({ chain, }: { readonly chain: string; },): Promise<readonly Counted[]> {
  /**
   Listing of the table as JSON.
   */
  const parsed: unknown = JSON.parse((await run({
    command: 'nft',
    args: [
      '-j',
      'list',
      'table',
      'inet',
      trafficBlockTable({ interfaceName: INTERFACE_NAME, },),
    ],
  },)).stdout,);
  if (!isListing(parsed,))
    throw new Error('nft did not list the table as JSON with an nftables array',);
  return parsed.nftables
    .flatMap(function ruleOf(entry,): readonly ListedRule[] {
      return (entry.rule === undefined) || (entry.rule
        .chain
        !== chain) ? [] : [entry.rule,];
    },)
    .flatMap(function counted(rule,): readonly Counted[] {
      return kindsOf({ rule, },)
        .map(function withPackets(kind,): Counted {
          return {
            kind,
            packets: rule.expr
              ?.find(function hasCounter(expression,): boolean {
                return expression.counter !== undefined;
              },)
              ?.counter
              ?.packets
              ?? 0,
          };
        },);
    },);
}

/**
 Runs a command and reports which rules counted a packet while it ran.

 @param command - Executable that sends one packet.

 @param args - Arguments.

 @returns The kinds whose counters grew,
 and the exit status and output of the command.

 @example
 ```ts
 await probe({ command: 'ping', args: ['-c1', '10.99.0.7'] });
 ```
 */
async function probe({
  command,
  args,
}: {
  readonly command: string;
  readonly args: readonly string[]
},): Promise<Probed> {
  /**
   Counters before.
   */
  const before = await listCounters({ chain: 'output', },);
  /**
   The command's outcome.
   */
  const result = await runAllowingFailure({
    command,
    args,
  },);
  /**
   Counters after.
   */
  const after = await listCounters({ chain: 'output', },);
  return {
    counted: after
      .filter(function grew(
        rule,
        index,
      ): boolean {
        return rule.packets > (before[index]
          ?.packets
          ?? 0);
      },)
      .map(function kindOfGrown(rule,): RuleKind {
        return rule.kind;
      },),
    status: result.exitCode,
    output: result.stdout,
  };
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
  return probe({
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
  },
  makeDirectory: scratchDirectory,
},);
/**
 The rules of the output chain,
 in order.
 */
const outputOrder = (await listCounters({ chain: 'output', },)).map(function kindOfRule(rule,): RuleKind {
  return rule.kind;
},);
/**
 The rules of the forward chain.
 */
const forwardOrder = (await listCounters({ chain: 'forward', },)).map(function kindOfRule(rule,): RuleKind {
  return rule.kind;
},);
/**
 A marking program run with the capabilities of the namespace's root.
 */
const withCapabilities = await probe({
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
const withoutCapabilities = await probe({
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
