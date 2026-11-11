import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { resolveApplicationExemptionCommand, } from './application-exemption-command.ts';
import { shellQuote, } from './shell-quote.ts';
import { bypassStatePath, } from './tunnel-bypass-path.ts';
import { readWatcherRecord, } from './tunnel-bypass-watcher-sidecar.ts';
import {
  readClearnet,
  type Clearnet,
} from './tunnel-clearnet.ts';
import { readFwmark, } from './tunnel-fwmark.ts';
import {
  trafficBlockScript,
  trafficBlockTable,
} from './tunnel-traffic-block.ts';

/**
 Module logger for collecting the facts of the by-hand steps.
 */
const l = tagged({ tag: 'tunnel-teardown-manual', },);

/**
 Name `wg-quicker-exempt` is run under when wg-quicker found no program by that name.
 */
const COMPANION_COMMAND = 'wg-quicker-exempt';

/**
 What the by-hand steps need to name exactly.
 */
export type ManualFacts = {
  /**
   Path of the application-bypass state file of the interface.
   */
  readonly statePath: string;

  /**
   Path of `wg-quicker-exempt`,
   or its bare name when wg-quicker found none.
   */
  readonly companionPath: string;

  /**
   Policy table that the rules of the tunnel name,
   absent when the interface has none (`Table = off`) or it could not be read.
   */
  readonly policyTable?: number;

  /**
   What the routing sends around the tunnel,
   which the traffic block of the by-hand steps lets through,
   absent when it could not be read or the interface has no application bypass.
   */
  readonly clearnet?: Clearnet;

  /**
   Process ID that the record of the route watcher names,
   absent when the record could not be read.
   */
  readonly routeWatcherPid?: number;
};

/**
 The nft statements of the traffic block,
 joined into the one argument that `nft` applies as a single transaction,
 as the script of wg-quicker is.

 @param interfaceName - Tunnel interface.

 @param clearnet - What the routing sends around the tunnel,
 or nothing for a block without exceptions.

 @returns The statements, separated by semicolons.

 @throws {@link TrafficBlockError} when a mark or an address is not what it must be.

 @example
 ```ts
 blockStatements({ interfaceName: 'wg0' });
 ```
 */
export function blockStatements(
  {
    interfaceName,
    clearnet,
  }: {
    readonly interfaceName: string;
    readonly clearnet?: Clearnet
  },
): string {
  return trafficBlockScript({
    interfaceName,
    ...(clearnet === undefined ? {} : { clearnet, }),
  },)
    .split('\n',)
    .filter(function isStatement(line,): boolean {
      return line !== '';
    },)
    .join('; ',);
}

/**
 Reads the process ID that the record of the route watcher names,
 for the check before the process is stopped.
 The record is a different file from the state file, so it is often readable when the state file is not.

 @param statePath - Path of the state file, next to which the record stands.

 @returns The process ID under `routeWatcherPid`,
 or nothing when there is no readable record.

 @example
 ```ts
 await readableWatcherPid({ statePath: '/run/wg-quicker/interface-x.json' });
 ```
 */
async function readableWatcherPid(
  { statePath, }: { readonly statePath: string; },
): Promise<{ readonly routeWatcherPid?: number; }> {
  try {
    /**
     Identity the record holds,
     absent when it holds none.
     */
    const record = await readWatcherRecord({ statePath, },);
    return record.kind === 'identity' ? { routeWatcherPid: record.identity.pid, } : {};
  }
  catch (error) {
    l.error(`the record of the route watcher could not be read for the by-hand steps: ${String(error,)}`,);
    return {};
  }
}

/**
 Reads what the routing sends around the tunnel when that is possible,
 for the exceptions of the traffic block in the by-hand steps.
 The refusal these steps go under is usually the failure to read exactly this,
 so a failure is not an error here:
 the block then has no exceptions, and the steps say so.

 @param interfaceName - Tunnel interface.

 @returns The clearnet traffic under `clearnet`,
 or nothing.

 @example
 ```ts
 await readableClearnet({ interfaceName: 'wg0' });
 ```
 */
async function readableClearnet(
  { interfaceName, }: { readonly interfaceName: string; },
): Promise<{ readonly clearnet?: Clearnet; }> {
  try {
    /**
     What the routing sends around the tunnel.
     */
    const read = await readClearnet({ interfaceName, },);
    if (!read.bypass)
      return {};
    // The statements are built here so that a mark or an address the script refuses leaves the block without exceptions.
    blockStatements({
      interfaceName,
      clearnet: read.clearnet,
    },);
    return { clearnet: read.clearnet, };
  }
  catch (error) {
    l.error(`${interfaceName}: what the routing sends around the tunnel could not be read for the by-hand steps: ${String(error,)}`,);
    return {};
  }
}

/**
 Words for the path of the state file when it cannot be worked out.

 @param error - Why it could not.

 @returns A sentence that stands where the path would.

 @example
 ```ts
 unknownStatePath({ error: new Error('x') });
 ```
 */
function unknownStatePath({ error, }: { readonly error: unknown; },): string {
  l.error(`the state path could not be worked out: ${String(error,)}`,);
  return `(the state file; its path could not be worked out: ${String(error,)})`;
}

/**
 Looks up what the by-hand steps need to name exactly:
 the state file path,
 the program that stops the application watcher,
 and the policy table of the live interface.
 Nothing here changes anything;
 `wg show` and the lookup of the program only read.

 @param interfaceName - Tunnel interface.

 @returns The facts.

 @example
 ```ts
 await readManualFacts({ interfaceName: 'wg0' });
 ```
 */
export async function readManualFacts({ interfaceName, }: { readonly interfaceName: string; },): Promise<ManualFacts> {
  /**
   Probe of the live fwmark,
   which names the policy table.
   */
  const fwmark = await readFwmark({ interfaceName, },);
  /**
   The program that stops the application watcher.
   */
  const companion = await resolveApplicationExemptionCommand();
  /**
   Path of the state file.
   */
  const statePath = statePathOf({ interfaceName, },);
  return {
    statePath,
    companionPath: companion.found ? companion.path : COMPANION_COMMAND,
    ...(fwmark.found ? { policyTable: fwmark.table, } : {}),
    ...(await readableClearnet({ interfaceName, },)),
    ...(await readableWatcherPid({ statePath, },)),
  };
}

/**
 Works out the state file path without letting a bad runtime directory setting hide the refusal.

 @param interfaceName - Tunnel interface.

 @returns The path,
 or words that say why there is none.

 @example
 ```ts
 statePathOf({ interfaceName: 'wg0' });
 ```
 */
function statePathOf({ interfaceName, }: { readonly interfaceName: string; },): string {
  try {
    return bypassStatePath({ interfaceName, },);
  }
  catch (error) {
    return unknownStatePath({ error, },);
  }
}

/**
 The exact commands for a person to finish a `down` without wg-quicker,
 for the case that wg-quicker refuses because it cannot read the state of the application bypass or the rules.
 Each command is what wg-quicker itself runs,
 in the order it runs them,
 and none needs the state file.

 @param interfaceName - Tunnel interface.

 @param facts - What the commands name exactly.

 @returns The lines to put under the refusal.

 @example
 ```ts
 manualFinishSteps({ interfaceName: 'wg0', facts: { statePath: '/run/wg-quicker/interface-x.json', companionPath: 'wg-quicker-exempt' } });
 ```
 */
export function manualFinishSteps(
  {
    interfaceName,
    facts,
  }: {
    readonly interfaceName: string;
    readonly facts: ManualFacts
  },
): readonly string[] {
  /**
   What the commands name exactly.
   */
  const {
    statePath,
    companionPath,
    policyTable,
    clearnet,
    routeWatcherPid,
  } = facts;
  /**
   The traffic block table as a word of a command.
   */
  const blockTable = shellQuote({ value: trafficBlockTable({ interfaceName, },), },);
  /**
   What the traffic block lets through,
   in a remark.
   */
  const blockNote = clearnet === undefined
    ? 'no exceptions could be read, so this block lets nothing through except loopback, the updates slice included, until the last command of this list lifts it'
    : 'this block lets through the traffic with the exempt mark and the traffic to the update hosts that were read just now, and drops the rest, until the last command of this list lifts it';
  /**
   Interface name as a word of a command.
   */
  const iface = shellQuote({ value: interfaceName, },);
  /**
   State file as a word of a command.
   */
  const state = shellQuote({ value: statePath, },);
  /**
   Record of the route watcher,
   next to the state file,
   as a word of a command.
   */
  const watcherRecord = shellQuote({ value: `${statePath}.watcher.json`, },);
  /**
   The process ID in the words and commands of the check,
   or the placeholder when the record gave none.
   */
  const pid = routeWatcherPid === undefined ? '<pid>' : String(routeWatcherPid,);
  /**
   The record of the route watcher, the check that the process is the watcher, and the stop.
   The process ID is read from the record, so a number that was reused by another program is not taken for it:
   the check shows what the number runs now.
   */
  const stopWatcher = routeWatcherPid === undefined
    ? [
      `  sudo cat ${watcherRecord}`,
      `  # the \`pid\` field is the route watcher's; check it with \`ps -o args= -p ${pid}\`, which must print a line that ends with ${state}, and only then stop it with \`sudo kill ${pid}\`; any other output means the number is not the watcher any more, so do not kill it`,
    ]
    : [
      `  sudo cat ${watcherRecord}`,
      `  # the pid is ${pid}; the next command must print a line that ends with ${state}; any other output, or none, means ${pid} is not the watcher any more, so do not run the kill after it`,
      `  ps -o args= -p ${pid}`,
      `  sudo kill ${pid}`,
    ];
  /**
   Commands that remove the policy rules of the tunnel.
   */
  const policyRules = policyTable === undefined
    ? [
      '  # the next command prints the policy table, or off; if it prints a number, run `sudo ip -4 rule delete table <number>` and `sudo ip -6 rule delete table <number>`; if it prints off, skip them',
      `  sudo wg show ${iface} fwmark`,
    ]
    : [
      `  sudo ip -4 rule delete table ${String(policyTable,)}`,
      `  sudo ip -6 rule delete table ${String(policyTable,)}`,
    ];
  return [
    'to look at what could not be read:',
    `  sudo cat ${state}`,
    '  sudo ip -4 rule show',
    '  sudo ip -6 rule show',
    'first stop the route watcher, whichever way you go on: without the state file wg-quicker no longer looks for it, and a watcher left running adds rules again:',
    ...stopWatcher,
    `then, if the state file is damaged, move it away and run \`wg-quicker down ${interfaceName}\` again; wg-quicker then forgets the bypass rules and routes the file owned, and the block lets nothing through because no exceptions are known:`,
    `  sudo mv ${state} ${shellQuote({ value: `${statePath}.damaged`, },)}`,
    'or take the tunnel down by hand instead, with the route watcher stopped as above, and run these in order (each is what wg-quicker itself runs, except the first and the last; none needs the state file).'
      + ' The first puts the traffic block in place in one nft transaction, as wg-quicker does before it removes any routing,'
      + ' so that a step that fails leaves traffic blocked and not outside the tunnel; the last lifts it:',
    `  # ${blockNote}`,
    `  sudo nft ${shellQuote({ value: blockStatements({
      interfaceName,
      ...(clearnet === undefined ? {} : { clearnet, }),
    },), },)}`,
    `  sudo ${shellQuote({ value: companionPath, },)} watch-stop ${iface}`,
    ...policyRules,
    '  sudo ip -4 rule delete table main suppress_prefixlength 0',
    '  sudo ip -6 rule delete table main suppress_prefixlength 0',
    `  sudo nft delete table inet ${shellQuote({ value: `wg-quicker-${interfaceName}`, },)}`,
    `  sudo ip link delete dev ${iface}`,
    '  # only when the config sets DNS',
    `  sudo resolvectl revert ${iface}`,
    'the bypass rules and routes carry protocol 201 and are not removed by those commands: list them with `sudo ip -4 rule show` and `sudo ip -6 rule show`,',
    'and for each line that ends in `proto 201` run `sudo ip -4 rule delete pref <pref> protocol 201` (with `-6` for IPv6) and `sudo ip -4 route flush table <table> protocol 201`, where the table is the number after `lookup`',
    'last, remove the records of the tunnel:',
    `  sudo rm --force ${state} ${watcherRecord}`,
    '  # lift the traffic block; the network works again',
    `  sudo nft delete table inet ${blockTable}`,
  ];
}

/**
 Works out the by-hand steps for an interface from the live system.

 @param interfaceName - Tunnel interface.

 @returns The lines to put under the refusal.

 @example
 ```ts
 await describeManualSteps({ interfaceName: 'wg0' });
 ```
 */
export async function describeManualSteps({ interfaceName, }: { readonly interfaceName: string; },): Promise<readonly string[]> {
  return manualFinishSteps({
    interfaceName,
    facts: await readManualFacts({ interfaceName, },),
  },);
}
