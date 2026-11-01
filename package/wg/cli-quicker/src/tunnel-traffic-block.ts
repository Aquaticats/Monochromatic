import { writeFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import { TrafficBlockError, } from './errors.ts';
import {
  run,
  runAllowingFailure,
} from './runner.ts';
import { makeTempDir, } from './tempdir.ts';

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
 and so is everything forwarded.
 The tunnel's own encrypted packets leave through the physical interface,
 so the tunnel stops carrying anything too.
 That is the point:
 nothing leaves outside the tunnel unprotected while the teardown is unfinished.

 The script is one transaction (`nft -f` applies a file atomically) that creates the table when it is absent,
 empties it when it is present,
 and fills it,
 so replacing a block that is already in place never leaves a moment without one:
 a `down` that is run again after a failed one has its routing removed and only the block protecting it.

 @param interfaceName - Tunnel interface.

 @returns The script for `nft -f`.

 @example
 ```ts
 trafficBlockScript({ interfaceName: 'wg0' });
 ```
 */
export function trafficBlockScript({ interfaceName, }: { readonly interfaceName: string; },): string {
  /**
   Table that holds the block.
   */
  const table = trafficBlockTable({ interfaceName, },);
  return [
    `add table inet ${table}`,
    `flush table inet ${table}`,
    `add chain inet ${table} output { type filter hook output priority -400; policy accept; }`,
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
    runCommand = run,
    makeDirectory = makeTempDir,
  }: {
    readonly interfaceName: string;
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
    trafficBlockScript({ interfaceName, },),
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
        + `everything that leaves the machine is dropped until it is removed: sudo nft delete table inet ${table}`
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
