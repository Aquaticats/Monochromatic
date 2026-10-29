/**
 Status command; displays the current state of a managed VM.
 
 @module
 */

import { BYTES_PER_GIB, } from '@monochromatic-dev/module-const/ts';
import { tagged, } from '@monochromatic-dev/module-logger/ts';

import { readConfig, } from './config.ts';

/**
 Logger root for vmsync after removing the package log shim.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l, },);
 ```
 */
const l = tagged({ tag: 'vmsync', },);

/**
 Prints the status of a named VM's {@link VmsyncConfig} to stdout.
 Shows name, disk size, boot settings, last hypervisor, sync state, and checksums.
 
 @param name - VM name
 
 @throws Error when the VM config is missing
 
 @example
 ```ts
 await showStatus('alpine');
 ```
 */
export async function showStatus(name: string,): Promise<void> {
  /**
   Tagged logger so status entries are scoped to `showStatus` in the output.
   */
  const rl = tagged({
    tag: showStatus.name,
    l,
  },);
  rl.info(`reading status for "${name}"`,);

  /**
   Current VM configuration.
   */
  const config = await readConfig(name,);

  /**
   Disk size formatted in GiB.
   */
  const sizeGib = (config.diskSizeBytes
    / BYTES_PER_GIB).toFixed(1,);

  console.log(`name:       ${config.name}`,);
  console.log(`imported:   ${config.importedFrom}`,);
  console.log(`disk size:  ${sizeGib} GiB`,);
  console.log(`memory:     ${config.boot
    .memory}`,);
  console.log(`cpus:       ${String(config.boot
    .cpus,)}`,);
  console.log(`synced:     ${String(config.state
    .synced,)}`,);
  console.log(
    `last boot:  ${config.state
      .lastBootHypervisor
      ?? 'never'} (${
      config
        .state
        .lastBootAt
        ?? 'n/a'
    })`,
  );
  console.log(`qcow2 hash: ${config.state
    .checksums
    .qcow2}`,);
  console.log(`vhdx hash:  ${config.state
    .checksums
    .vhdx}`,);
}
