import { writeFile, } from 'node:fs/promises';
import { join, } from 'node:path';

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
    `add chain inet ${table} output { type filter hook output priority -400; policy accept; }`,
    `add rule inet ${table} output oifname != "lo" counter drop`,
    `add chain inet ${table} forward { type filter hook forward priority -400; policy accept; }`,
    `add rule inet ${table} forward counter drop`,
    '',
  ].join('\n',);
}

/**
 Blocks all traffic that does not stay on the machine.
 An existing block is removed first,
 so applying it twice leaves one block.

 @param interfaceName - Tunnel interface.

 @throws {@link CommandError} when `nft` cannot install the block.

 @example
 ```ts
 await installTrafficBlock({ interfaceName: 'wg0' });
 ```
 */
export async function installTrafficBlock({ interfaceName, }: { readonly interfaceName: string; },): Promise<void> {
  await removeTrafficBlock({ interfaceName, },);
  /**
   Temp file holding the nft script;
   `nft -f` reads a seekable file.
   */
  await using dir = await makeTempDir();
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
  await run({
    command: 'nft',
    args: [
      '-f',
      scriptPath,
    ],
  },);
}

/**
 Removes the block,
 idempotently.

 @param interfaceName - Tunnel interface.

 @example
 ```ts
 await removeTrafficBlock({ interfaceName: 'wg0' });
 ```
 */
export async function removeTrafficBlock({ interfaceName, }: { readonly interfaceName: string; },): Promise<void> {
  await runAllowingFailure({
    command: 'nft',
    args: [
      'delete',
      'table',
      'inet',
      trafficBlockTable({ interfaceName, },),
    ],
  },);
}

/**
 Tells whether the block is in place.
 A retried `down` uses this to say truthfully what an earlier one left behind.

 @param interfaceName - Tunnel interface.

 @returns Whether the nft table of the block exists.

 @example
 ```ts
 await trafficBlockExists({ interfaceName: 'wg0' });
 ```
 */
export async function trafficBlockExists({ interfaceName, }: { readonly interfaceName: string; },): Promise<boolean> {
  /**
   What `nft list table` said.
   */
  const listed = await runAllowingFailure({
    command: 'nft',
    args: [
      'list',
      'table',
      'inet',
      trafficBlockTable({ interfaceName, },),
    ],
  },);
  return listed.exitCode === 0;
}
