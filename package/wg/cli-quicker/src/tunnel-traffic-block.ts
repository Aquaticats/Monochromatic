import { writeFile, } from 'node:fs/promises';
import { isIP, } from 'node:net';
import { join, } from 'node:path';

import { TrafficBlockError, } from './errors.ts';
import { formatMark, } from './exempt-mark-reservation.ts';
import {
  run,
  runAllowingFailure,
} from './runner.ts';
import { makeTempDir, } from './tempdir.ts';
import {
  readClearnet,
  type Clearnet,
} from './tunnel-clearnet.ts';

/**
 Largest packet mark: the mark is 32 bits.
 */
const MAX_MARK = 0xFF_FF_FF_FF;

/**
 What the traffic block lets through,
 in words for the messages.
 It states what the block was installed to let through,
 which is nothing when the interface has no application bypass.
 */
export const BLOCK_EXCEPTIONS: string = 'except the clearnet traffic it was installed to let through '
  + '(traffic with the exempt mark, which the updates slice and exempted applications use, and traffic to the update hosts, '
  + 'when the interface has an application bypass)';

/**
 Address families of the clearnet destinations,
 with the nft keyword and the IP version of each.
 */
const DESTINATION_FAMILIES = [
  {
    family: '-4',
    keyword: 'ip',
    version: 4,
  },
  {
    family: '-6',
    keyword: 'ip6',
    version: 6,
  },
] as const;

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
 The rules that let through what the routing sends around the tunnel by design,
 placed ahead of the drop:
 a packet whose mark is the exempt mark,
 which is exactly what the policy rule `fwmark <mark> table <bypass table>` matches
 (that rule carries no mask, so neither does this one),
 and a packet to one of the update-host destinations of the rules `to <address> table <bypass table>`.
 The mark reaches the packet from the socket,
 where the cgroup programs of the updates slice and of the exempted applications set it,
 and the kernel refuses to set it without `CAP_NET_RAW` or `CAP_NET_ADMIN` in the user namespace of the socket's network namespace
 (`sk_setsockopt` and `sock_cmsg_send` in `net/core/sock.c` of Linux v7.2.11, lines 1527 to 1532 and 3041 to 3044),
 so an unprivileged process cannot give its packets the mark.
 A process in a user and network namespace of its own can mark its own sockets there,
 but its packets reach this host through the forward hook,
 which stays dropped,
 and not through the output hook that these rules sit in.

 @param table - Table that holds the block.

 @param clearnet - What the routing sends around the tunnel.

 @returns The nft rules,
 empty when nothing is.

 @throws {@link TrafficBlockError} when the mark or an address is not what it must be,
 because both are written into the script.

 @example
 ```ts
 clearnetRules({ table: 'wg-quicker-wg0-blocked', clearnet: { mark: 100, hosts: [] } });
 ```
 */
function clearnetRules({
  table,
  clearnet,
}: {
  readonly table: string;
  readonly clearnet: Clearnet
},): readonly string[] {
  /**
   Rules in the order they are applied.
   */
  const rules: string[] = [];
  if (clearnet.mark !== undefined) {
    if ((!Number.isInteger(clearnet.mark,)) || (clearnet.mark < 0)
      || (clearnet.mark > MAX_MARK))
      throw new TrafficBlockError(`the exempt mark ${String(clearnet.mark,)} is not a 32-bit packet mark, so it cannot be let through the traffic block`,);
    rules.push(`add rule inet ${table} output meta mark ${formatMark({ mark: clearnet.mark, },)} counter accept`,);
  }
  for (const {
    family,
    keyword,
    version,
  } of DESTINATION_FAMILIES) {
    /**
     Destinations of this family without repeats,
     which a set literal refuses.
     */
    const addresses = [...new Set(clearnet.hosts
      .filter(function ofFamily(host,): boolean {
        return host.family === family;
      },)
      .map(function addressOf(host,): string {
        return host.address;
      },),),];
    for (const address of addresses) {
      if (isIP(address,) !== version)
        throw new TrafficBlockError(`${JSON.stringify(address,)} is not an IPv${String(version,)} address, so it cannot be let through the traffic block`,);
    }
    if (addresses.length > 0)
      rules.push(`add rule inet ${table} output ${keyword} daddr { ${addresses.join(', ',)} } counter accept`,);
  }
  return rules;
}

/**
 The nft script of the block:
 everything that leaves through an interface other than loopback is dropped,
 and so is everything forwarded,
 except the traffic that the routing sends around the tunnel by design,
 which is let through ahead of the drop in the output chain.
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
    clearnet = { hosts: [], },
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
    ...clearnetRules({
      table,
      clearnet,
    },),
    `add rule inet ${table} output oifname != "lo" counter drop`,
    `add chain inet ${table} forward { type filter hook forward priority -400; policy accept; }`,
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
    clearnet = { hosts: [], },
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
 What `nft list table` shows about the block.
 */
export type TrafficBlockState = 'present' | 'absent' | 'unknown';

/**
 Looks at the block.
 The table is `absent` only when `nft` says that no such table exists;
 any other failure, such as missing privilege,
 leaves the state `unknown`,
 because an unreadable table may still be in place.

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
   What `nft list table` said.
   */
  const listed = await runCommand({
    command: 'nft',
    args: [
      'list',
      'table',
      'inet',
      trafficBlockTable({ interfaceName, },),
    ],
  },);
  if (listed.exitCode === 0)
    return 'present';
  return listed.stderr
    .toLowerCase()
    .includes('no such file or directory',)
    ? 'absent'
    : 'unknown';
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

 @throws {@link BypassStateError} when the ownership state cannot be read.

 @throws {@link CommandError} when the rules cannot be listed or `nft` cannot install the block.

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
  const read = await readExceptions({ interfaceName, },);
  if ((!read.bypass) && await blockExists({ interfaceName, },))
    return false;
  await install({
    interfaceName,
    ...(read.bypass ? { clearnet: read.clearnet, } : {}),
  },);
  return true;
}
