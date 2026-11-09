import { writeFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import { tagged, } from '@monochromatic-dev/logger-core/ts';

import {
  ClearnetUnreadableError,
  TrafficBlockError,
} from './errors.ts';
import {
  run,
  runAllowingFailure,
} from './runner.ts';
import { makeTempDir, } from './tempdir.ts';
import {
  readClearnet,
  type Clearnet,
} from './tunnel-clearnet.ts';
import {
  tablePresence,
  type TablePresence,
} from './tunnel-traffic-block-listing.ts';
import {
  hostAcceptRules,
  hostReplyRules,
  markAcceptRules,
} from './tunnel-traffic-block-rules.ts';

/**
 Module logger for the traffic block.
 */
const l = tagged({ tag: 'tunnel-traffic-block', },);

/**
 What the traffic block lets through,
 in words for the messages.
 It states what the block was installed to let through,
 which is nothing when the interface has no application bypass.
 */
export const BLOCK_EXCEPTIONS: string = 'except the clearnet traffic it was installed to let through '
  + '(traffic with the exempt mark, which the updates slice and exempted applications use, and traffic to the update hosts, '
  + 'including forwarded traffic to them, when the interface has an application bypass)';

/**
 Name of the nft table that blocks traffic while a `down` is half done.
 It is a different table from the kill switch of the tunnel (`wg-quicker-<interface>`),
 so removing either never removes the other.

 @param interfaceName - Tunnel interface.

 @returns The table name.

 @example
 ```ts
 trafficBlockTable({ interfaceName: 'wg0' }); // 'wg-quicker-wg0-blocked'
 ```
 */
export function trafficBlockTable({ interfaceName, }: { readonly interfaceName: string; },): string {
  return `wg-quicker-${interfaceName}-blocked`;
}

/**
 The nft script of the block:
 everything that leaves through an interface other than loopback is dropped,
 and so is everything forwarded,
 except the traffic that the routing sends around the tunnel by design,
 which is let through ahead of the drop:
 in the output chain a packet with the exempt mark and a packet to an update host,
 in the forward chain a packet to an update host and a reply from one
 (a guest's connection needs both directions, and the reply rule matches `ct direction reply` and the physical default interface it arrives on,
 so that it cannot be used with a forged source or with two forged packets that make a "reply";
 the mark rule is not in the forward chain, because no socket of this machine marked a forwarded packet).
 The tunnel's own encrypted packets leave through the physical interface,
 so the tunnel stops carrying anything too.
 That is the point:
 nothing leaves outside the tunnel unprotected while the teardown is unfinished,
 and what was always meant to leave outside it is not cut off.

 The script is one transaction (`nft -f` applies a file atomically) that creates the table when it is absent,
 empties it when it is present,
 and fills it,
 so replacing a block that is already in place never leaves a moment without one:
 a `down` that is run again after a failed one has its routing removed and only the block protecting it.

 @param interfaceName - Tunnel interface.

 @param clearnet - What the routing sends around the tunnel,
 let through ahead of the drop.

 @returns The script for `nft -f`.

 @example
 ```ts
 trafficBlockScript({ interfaceName: 'wg0' });
 ```
 */
export function trafficBlockScript(
  {
    interfaceName,
    clearnet = {
      hosts: [],
      interfaces: [],
    },
  }: {
    readonly interfaceName: string;
    readonly clearnet?: Clearnet;
  },
): string {
  /**
   Table that holds the block.
   */
  const table = trafficBlockTable({ interfaceName, },);
  return [
    `add table inet ${table}`,
    `flush table inet ${table}`,
    `add chain inet ${table} output { type filter hook output priority -400; policy accept; }`,
    ...markAcceptRules({
      table,
      clearnet,
    },),
    ...hostAcceptRules({
      table,
      chain: 'output',
      clearnet,
    },),
    `add rule inet ${table} output oifname != "lo" counter drop`,
    `add chain inet ${table} forward { type filter hook forward priority -400; policy accept; }`,
    ...hostAcceptRules({
      table,
      chain: 'forward',
      clearnet,
    },),
    ...hostReplyRules({
      table,
      clearnet,
    },),
    `add rule inet ${table} forward counter drop`,
    '',
  ].join('\n',);
}

/**
 Blocks all traffic that does not stay on the machine.
 A block that is already in place is replaced by the same transaction that installs the new one,
 so applying it twice leaves one block and never a moment without one.

 @param interfaceName - Tunnel interface.

 @param clearnet - What the routing sends around the tunnel,
 let through ahead of the drop.

 @param runCommand - Command runner,
 replaceable in tests.

 @param makeDirectory - Maker of the scratch directory that holds the script,
 replaceable in tests that run without the privileges of the real one.

 @throws {@link CommandError} when `nft` cannot install the block.

 @example
 ```ts
 await installTrafficBlock({ interfaceName: 'wg0' });
 ```
 */
export async function installTrafficBlock(
  {
    interfaceName,
    clearnet = {
      hosts: [],
      interfaces: [],
    },
    runCommand = run,
    makeDirectory = makeTempDir,
  }: {
    readonly interfaceName: string;
    readonly clearnet?: Clearnet;
    readonly runCommand?: typeof run;
    readonly makeDirectory?: typeof makeTempDir;
  },
): Promise<void> {
  /**
   Temp file holding the nft script;
   `nft -f` reads a seekable file.
   */
  await using dir = await makeDirectory();
  /**
   Path of the nft script file.
   */
  const scriptPath = join(
    dir.path,
    'traffic-block.nft',
  );
  await writeFile(
    scriptPath,
    trafficBlockScript({
      interfaceName,
      clearnet,
    },),
  );
  await runCommand({
    command: 'nft',
    args: [
      '-f',
      scriptPath,
    ],
  },);
}

/**
 What the listing of the nft tables shows about the block.
 */
export type TrafficBlockState = TablePresence;

/**
 Looks at the block.
 The table is `absent` only when the listing of the tables,
 read as structured JSON,
 shows no such table;
 any other outcome,
 such as missing privilege or output that is not the listing,
 leaves the state `unknown`,
 because an unreadable ruleset may still hold the table.
 No message text is matched,
 because the system's locale words it.

 @param interfaceName - Tunnel interface.

 @param runCommand - Command runner,
 replaceable in tests.

 @returns Whether the nft table of the block is present,
 absent or not known.

 @example
 ```ts
 await trafficBlockState({ interfaceName: 'wg0' });
 ```
 */
export async function trafficBlockState(
  {
    interfaceName,
    runCommand = runAllowingFailure,
  }: {
    readonly interfaceName: string;
    readonly runCommand?: typeof runAllowingFailure;
  },
): Promise<TrafficBlockState> {
  /**
   What `nft --json list tables` said.
   */
  const listed = await runCommand({
    command: 'nft',
    args: [
      '--json',
      'list',
      'tables',
    ],
  },);
  if (listed.exitCode !== 0)
    return 'unknown';
  return tablePresence({
    text: listed.stdout,
    family: 'inet',
    name: trafficBlockTable({ interfaceName, },),
  },);
}

/**
 Removes the block and shows that it is gone.
 A table that was never there is not a failure,
 but a table that is still there after the delete,
 or one that cannot be shown gone,
 is:
 the network stays blocked,
 and nobody has been told.

 @param interfaceName - Tunnel interface.

 @param runCommand - Command runner,
 replaceable in tests.

 @throws {@link TrafficBlockError} naming the table and the command that removes it,
 when the block is still in place or cannot be shown gone.

 @example
 ```ts
 await removeTrafficBlock({ interfaceName: 'wg0' });
 ```
 */
export async function removeTrafficBlock(
  {
    interfaceName,
    runCommand = runAllowingFailure,
  }: {
    readonly interfaceName: string;
    readonly runCommand?: typeof runAllowingFailure;
  },
): Promise<void> {
  /**
   Table that holds the block.
   */
  const table = trafficBlockTable({ interfaceName, },);
  /**
   What `nft delete table` said;
   it fails when there is no table,
   which is the idempotent case.
   */
  const deleted = await runCommand({
    command: 'nft',
    args: [
      'delete',
      'table',
      'inet',
      table,
    ],
  },);
  /**
   Whether the table is gone.
   */
  const state = await trafficBlockState({
    interfaceName,
    runCommand,
  },);
  if (state === 'absent')
    return;
  /**
   What `nft delete table` printed,
   for the person to read.
   */
  const said = `nft delete table exited ${String(deleted.exitCode,)}: ${deleted.stderr
    .trim()}`;
  throw new TrafficBlockError(
    state === 'present'
      ? `the traffic block (nft table inet ${table}) is still in place after the attempt to remove it (${said}); `
        + `everything that leaves the machine ${BLOCK_EXCEPTIONS} is dropped until it is removed: sudo nft delete table inet ${table}`
      : `could not tell whether the traffic block (nft table inet ${table}) was removed (${said}); `
        + `check with \`sudo nft list table inet ${table}\` and remove it if it is there: sudo nft delete table inet ${table}`,
  );
}

/**
 Tells whether the block is in place.
 A retried `down` uses this to say truthfully what an earlier one left behind,
 so a table that cannot be read counts as not known to be there.

 @param interfaceName - Tunnel interface.

 @param runCommand - Command runner,
 replaceable in tests.

 @returns Whether the nft table of the block is known to exist.

 @example
 ```ts
 await trafficBlockExists({ interfaceName: 'wg0' });
 ```
 */
export async function trafficBlockExists(
  {
    interfaceName,
    runCommand = runAllowingFailure,
  }: {
    readonly interfaceName: string;
    readonly runCommand?: typeof runAllowingFailure;
  },
): Promise<boolean> {
  return await trafficBlockState({
    interfaceName,
    runCommand,
  },) === 'present';
}

/**
 Reads what the routing sends around the tunnel,
 and when that cannot be read says what could not be and what that stops.

 @param interfaceName - Tunnel interface.

 @param readExceptions - Reader of what the routing sends around the tunnel.

 @returns What the reader returned.

 @throws {@link ClearnetUnreadableError} when the reader fails.

 @example
 ```ts
 await readOrExplain({ interfaceName: 'wg0', readExceptions: readClearnet });
 ```
 */
async function readOrExplain(
  {
    interfaceName,
    readExceptions,
  }: {
    readonly interfaceName: string;
    readonly readExceptions: typeof readClearnet
  },
): ReturnType<typeof readClearnet> {
  try {
    return await readExceptions({ interfaceName, },);
  }
  catch (error) {
    l.error(`${interfaceName}: what the routing sends around the tunnel could not be read: ${String(error,)}`,);
    throw new ClearnetUnreadableError(
      `could not read what the routing sends around the tunnel (${String(error,)}), `
        + 'which the traffic block lets through; a block built without it would cut that traffic',
      { cause: error, },
    );
  }
}

/**
 Installs the block that protects a `down`,
 letting through what the routing sends around the tunnel by design.
 The exceptions are read from the ownership state and the kernel's rules, which are still in place at this point.
 When the ownership state is already gone and a block stands,
 the block is kept as it is:
 an earlier `down` that failed after it removed the bypass left it,
 with the exceptions it was installed with,
 and replacing it from a state that no longer says anything would cut the traffic that was let through.

 @param interfaceName - Tunnel interface.

 @param readExceptions - Reader of what the routing sends around the tunnel,
 replaceable in tests.

 @param blockExists - Check for a block in place,
 replaceable in tests.

 @param install - Installer of the block,
 replaceable in tests.

 @returns Whether the block was installed,
 and not kept as an earlier `down` left it.

 @throws {@link ClearnetUnreadableError} when the ownership state or the rules cannot be read,
 before anything is installed.

 @throws {@link CommandError} when `nft` cannot install the block.

 @example
 ```ts
 await blockTraffic({ interfaceName: 'wg0' });
 ```
 */
export async function blockTraffic(
  {
    interfaceName,
    readExceptions = readClearnet,
    blockExists = trafficBlockExists,
    install = installTrafficBlock,
  }: {
    readonly interfaceName: string;
    readonly readExceptions?: typeof readClearnet;
    readonly blockExists?: typeof trafficBlockExists;
    readonly install?: typeof installTrafficBlock;
  },
): Promise<boolean> {
  /**
   What to let through,
   or nothing when the ownership state is gone.
   */
  const read = await readOrExplain({
    interfaceName,
    readExceptions,
  },);
  if ((!read.bypass) && await blockExists({ interfaceName, },))
    return false;
  await install({
    interfaceName,
    ...(read.bypass ? { clearnet: read.clearnet, } : {}),
  },);
  return true;
}
