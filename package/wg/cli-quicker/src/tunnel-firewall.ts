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
  const nftable = `wg-quicker-${iface}`;
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
      `wg-quicker-${interfaceName}`,
    ],
  },);
}
