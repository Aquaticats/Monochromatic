/**
 The size of a libvirt guest, as `mvm list` reports it.

 mvm writes the numbers it gave a guest into that guest's `meta.json` at create
 time, so the numbers `list` shows are the numbers the guest was made with rather
 than a second reading of the running domain. A guest made before mvm recorded
 them has no such fields, and `list` then says nothing about its size, which is
 why this reader answers with what it can prove and not with a default.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { readVmMeta, } from './meta.ts';

/**
 Logger root for mvm after removing the package log shim.

 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l, },);
 ```
 */
const l = tagged({ tag: 'mvm', },);

//region Shape

/**
 The size a VM's metadata records, with either number absent when the record was
 written before mvm recorded it.
 
 @example
 ```ts
 const sizes: VmSizes = { vcpus: 4, memoryGiB: 8 };
 ```
 */
export type VmSizes = {
  /**
   Memory the guest was given, in gibibytes.
   */
  readonly memoryGiB?: number;
  /**
   Virtual CPUs the guest was given.
   */
  readonly vcpus?: number;
};

//endregion Shape

//region Reading

/**
 Reads a VM's size out of the metadata in its own directory.
 
 A missing or unreadable record is not a failure of listing: a VM directory mvm
 can see in libvirt but whose metadata it cannot read still has a name and a
 state worth showing, so the size is simply absent.
 
 @param vmDir - absolute path to the VM's directory, holding `meta.json`
 
 @returns the numbers the record carries, or an empty record when it carries none
 
 @example
 ```ts
 const sizes = await readVmSizes({ vmDir: '/home/user/.local/share/mvm/vms/dev' });
 // { vcpus: 4, memoryGiB: 8 }
 ```
 */
export async function readVmSizes({
  vmDir,
}: {
  readonly vmDir: string;
},): Promise<VmSizes> {
  /**
   Logger scoped to this read so a skipped record is attributable.
   */
  const rl = tagged({
    tag: readVmSizes.name,
    l,
  },);
  try {
    /**
     Metadata mvm wrote for this VM.
     */
    const meta = await readVmMeta(vmDir,);
    return {
      ...((meta.memoryGiB === undefined) ? {} : { memoryGiB: meta.memoryGiB, }),
      ...((meta.vcpus === undefined) ? {} : { vcpus: meta.vcpus, }),
    };
  }
  catch (error: unknown) {
    rl.debug(`no size to report from ${vmDir}: ${String(error,)}`,);
    return {};
  }
}

//endregion Reading
