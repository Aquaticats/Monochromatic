import { rm, } from 'node:fs/promises';
import { join, } from 'node:path';
import { tagged, } from '@monochromatic-dev/module-logger/ts';

import {
  validateName,
  VMS_DIR,
} from './config.ts';
import {
  destroyVm,
  listVms,
  undefineVm,
} from './virsh.ts';

/**
 Logger root for mvm after removing the package log shim.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l, },);
 ```
 */
const l = tagged({ tag: 'mvm', },);

/**
 Destroys a single VM by force-stopping it, removing its libvirt definition,
 and deleting all associated storage and metadata.
 
 @param name - VM name without the mvm- prefix
 
 @param rl - Tagged logger for status messages
 
 @throws Error when the VM cannot be undefined (e.g. does not exist)
 
 @example
 ```ts
 await destroyOne({ name: 'dev-01', rl: logger });
 ```
 */
async function destroyOne(
  {
    name,
    rl,
  }: {
    readonly name: string;
    readonly rl: {
      readonly debug: (msg: string,) => void;
      readonly info: (msg: string,) => void;
    };
  },
): Promise<void> {
  rl.info(`destroying VM ${name}`,);

  try {
    await destroyVm({ name, },);
  }
  catch (error) {
    if (!(Error.isError(error,)))
      throw error;

    rl.debug('VM was not running, proceeding with undefine',);
  }

  await undefineVm({ name, },);

  /**
   Per-VM directory holding storage and metadata; removed to complete destruction.
   */
  const vmDir = join(
    VMS_DIR,
    name,
  );
  await rm(
    vmDir,
    {
      force: true,
      recursive: true,
    },
  );

  rl.info(`VM ${name} destroyed`,);
}

/**
 Destroys a VM by force-stopping it, removing its libvirt definition,
 and deleting all associated storage and metadata.
 
 @param name - VM name without the mvm- prefix
 
 @throws Error when the VM cannot be undefined (e.g. does not exist)
 
 @example
 ```ts
 await destroy({ name: 'dev-01' });
 ```
 */
export async function destroy({ name, }: { readonly name: string; },): Promise<void> {
  validateName(name,);
  /**
   Tagged logger so destroy entries are scoped to {@link destroy} in the output.
   */
  const rl = tagged({
    tag: destroy.name,
    l,
  },);
  await destroyOne({
    name,
    rl,
  },);
}

/**
 Destroys all managed VMs.
 
 @throws Error when any VM cannot be destroyed
 
 @example
 ```ts
 await destroyAll();
 ```
 */
export async function destroyAll(): Promise<void> {
  /**
   Tagged logger so bulk-destroy entries are scoped to {@link destroyAll} in the output.
   */
  const rl = tagged({
    tag: destroyAll.name,
    l,
  },);
  /**
   Current set of managed VM names enumerated before any destruction runs.
   */
  const vms = await listVms();

  if (vms.length
    === 0) {
    rl.info('no VMs to destroy',);
    return;
  }

  rl.info(`destroying all ${String(vms.length,)} VMs`,);
  // Destroy sequentially to avoid overwhelming libvirt with concurrent operations
  for (const name of vms) {
    // oxlint-disable-next-line no-await-in-loop -- intentionally sequential to avoid libvirt contention
    await destroyOne({
      name,
      rl,
    },);
  }
}
