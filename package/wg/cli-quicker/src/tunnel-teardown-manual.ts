import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { resolveApplicationExemptionCommand, } from './application-exemption-command.ts';
import { shellQuote, } from './shell-quote.ts';
import { bypassStatePath, } from './tunnel-bypass-path.ts';
import { readFwmark, } from './tunnel-fwmark.ts';
import { trafficBlockTable, } from './tunnel-traffic-block.ts';

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
};

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
  return {
    statePath: statePathOf({ interfaceName, },),
    companionPath: companion.found ? companion.path : COMPANION_COMMAND,
    ...(fwmark.found ? { policyTable: fwmark.table, } : {}),
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
  } = facts;
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
    `if the state file is damaged, move it away and run \`wg-quicker down ${interfaceName}\` again; wg-quicker then forgets the bypass rules and routes the file owned, and the block lets nothing through because no exceptions are known:`,
    `  sudo mv ${state} ${shellQuote({ value: `${statePath}.damaged`, },)}`,
    'to take the tunnel down by hand instead, run these in order (each is what wg-quicker itself runs; none needs the state file):',
    `  sudo ${shellQuote({ value: companionPath, },)} watch-stop ${iface}`,
    '  # the `pid` field of the next record is the route watcher; stop it with `sudo kill <pid>`',
    `  sudo cat ${watcherRecord}`,
    ...policyRules,
    '  sudo ip -4 rule delete table main suppress_prefixlength 0',
    '  sudo ip -6 rule delete table main suppress_prefixlength 0',
    `  sudo nft delete table inet ${shellQuote({ value: `wg-quicker-${interfaceName}`, },)}`,
    `  sudo ip link delete dev ${iface}`,
    '  # only when the config sets DNS',
    `  sudo resolvectl revert ${iface}`,
    '  # only when a block of an earlier attempt stands',
    `  sudo nft delete table inet ${shellQuote({ value: trafficBlockTable({ interfaceName, },), },)}`,
    'the bypass rules and routes carry protocol 201 and are not removed by those commands: list them with `sudo ip -4 rule show` and `sudo ip -6 rule show`,',
    'and for each line that ends in `proto 201` run `sudo ip -4 rule delete pref <pref> protocol 201` (with `-6` for IPv6) and `sudo ip -4 route flush table <table> protocol 201`, where the table is the number after `lookup`',
    'last, remove the records of the tunnel:',
    `  sudo rm --force ${state} ${watcherRecord}`,
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
