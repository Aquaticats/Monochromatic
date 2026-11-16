import { writeFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import type { WireguardConfig, } from './config.ts';
import {
  run,
  runAllowingFailure,
} from './runner.ts';
import { makeTempDir, } from './tempdir.ts';
import { isV6, } from './tunnel-util.ts';

/**
 Name of the kill-switch table of one interface.

 The name carries `kill-` right after the program's prefix, and the traffic block's
 carries `block-` in the same place, so no interface name can make one table's name
 equal another's: `wgq` and `wgq-blocked` were enough to do that when the kill switch
 was `wg-quicker-<interface>` and the block was `wg-quicker-<interface>-blocked`,
 and the `down` of `wgq` then deleted the kill switch of the running `wgq-blocked`.

 @param interfaceName - Tunnel interface.

 @returns The table name.

 @example
 ```ts
 killSwitchTable({ interfaceName: 'wg0' }); // 'wg-quicker-kill-wg0'
 ```
 */
export function killSwitchTable({ interfaceName, }: { readonly interfaceName: string; },): string {
  return `wg-quicker-kill-${interfaceName}`;
}

/**
 Installs the nft kill-switch so non-tunnel packets to interface addresses drop.
 
 @param config - Parsed config (addresses drive the drop rules).
 
 @param table - Policy table whose fwmark identifies tunnel-bound packets.
 
 @example
 ```ts
 await addKillSwitch({ config, table: 51820 });
 ```
 */
export async function addKillSwitch(
  {
    config,
    table,
  }: {
    readonly config: WireguardConfig;
    readonly table: number;
  },
): Promise<void> {
  /**
   Interface receiving the kill-switch.
   */
  const iface = config.interfaceName;
  /**
   Name of the dedicated nft table for this interface.
   */
  const nftable = killSwitchTable({ interfaceName: iface, },);
  /**
   nft statements built up and applied atomically via `nft -f`.
   */
  const statements: string[] = [`add table inet ${nftable}`,];
  for (const proto of [
    'ip',
    'ip6',
  ] as const) {
    statements.push(
      `add chain inet ${nftable} preraw_${proto} { type filter hook prerouting priority -300; }`,
    );
    for (const address of config.addresses) {
      /**
       Bare address without its prefix length.
       */
      const addr = address.slice(
        0,
        address.indexOf('/',),
      );
      /**
       Address family token matching the current protocol chain.
       */
      const family = isV6({ prefix: addr, },) ? 'ip6' : 'ip';
      if (family === proto)
        statements.push(
          `add rule inet ${nftable} preraw_${proto} iifname != "${iface}" ${proto} daddr ${addr} fib saddr type != local drop`,
        );
    }
  }
  statements.push(
    `add chain inet ${nftable} postmangle { type filter hook postrouting priority -150; }`,
    `add rule inet ${nftable} postmangle meta l4proto udp meta mark ${String(table,)} ct mark set mark`,
    `add chain inet ${nftable} premangle { type filter hook prerouting priority -150; }`,
    `add rule inet ${nftable} premangle meta l4proto udp meta mark set ct mark`,
  );
  /**
   Temp file holding the nft script; `nft -f` reads a seekable file as
   `/dev/stdin`, which a pipe from a spawned child is not.
   */
  await using dir = await makeTempDir();
  /**
   Path of the nft script file.
   */
  const scriptPath = join(
    dir.path,
    'kill-switch.nft',
  );
  await writeFile(
    scriptPath,
    `${statements.join('\n',)}\n`,
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
 Removes the nft kill-switch table, idempotently.
 
 @param interfaceName - Interface whose kill-switch table is deleted.
 
 @example
 ```ts
 await removeKillSwitch({ interfaceName: 'wg0' });
 ```
 */
export async function removeKillSwitch(
  { interfaceName, }: { readonly interfaceName: string; },
): Promise<void> {
  await runAllowingFailure({
    command: 'nft',
    args: [
      'delete',
      'table',
      'inet',
      killSwitchTable({ interfaceName, },),
    ],
  },);
  // An earlier build named this table `wg-quicker-<interface>`, and a tunnel it left
  // behind still has it; nothing else ever used that name, so deleting it here is
  // this interface's own table and never another's.
  await runAllowingFailure({
    command: 'nft',
    args: [
      'delete',
      'table',
      'inet',
      `wg-quicker-${interfaceName}`,
    ],
  },);
}
