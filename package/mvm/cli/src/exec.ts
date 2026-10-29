import { join, } from 'node:path';
import { tagged, } from '@monochromatic-dev/module-logger/ts';

import {
  validateName,
  VM_PREFIX,
  VMS_DIR,
} from './config.ts';
import {
  type ExecResult,
  runGuestCommand,
} from './guest-exec.ts';
import { readVmMeta, } from './meta.ts';

export type { ExecResult, } from './guest-exec.ts';

/**
 Logger root for mvm after removing the package log shim.

 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l, },);
 ```
 */
const l = tagged({ tag: 'mvm', },);

/**
 Executes a command inside a running VM via the QEMU guest agent.
 Reads VM metadata to determine the correct shell for the guest OS:
 Linux VMs use bash/ash, Windows VMs use PowerShell.
 The result is the one that belongs to this command; see {@link runGuestCommand}.

 @param command - Shell command to run inside the VM

 @param name - VM name without the mvm- prefix

 @returns Captured stdout, stderr, and exit code

 @throws {@link GuestExecLaunchError} when the guest agent refused the launch or did not confirm it

 @throws {@link GuestExecStatusUnavailableError} when the VM is gone or its agent stays silent past the limit

 @throws {@link GuestExecAttributionError} when no result the agent reports belongs to this command

 @throws {@link GuestExecOutputTooLargeError} when the command's output is larger than libvirt hands over

 @example
 ```ts
 // Linux VM
 const result = await exec({ command: 'uname -a', name: 'dev-01' });

 // Windows VM
 const winResult = await exec({ command: 'Get-ComputerInfo', name: 'win-01' });
 ```
 */
export async function exec(
  {
    command,
    name,
  }: {
    readonly command: string;
    readonly name: string;
  },
): Promise<ExecResult> {
  validateName(name,);
  /**
   Logger scoped to this exec call so log lines carry the function name.
   */
  const rl = tagged({
    tag: exec.name,
    l,
  },);
  /**
   Stored VM metadata; the osFamily and shell drive the guest-exec invocation shape.
   */
  const meta = await readVmMeta(join(
    VMS_DIR,
    name,
  ),);
  rl.debug(`executing command in VM ${name} (${meta.osFamily}, ${meta.shell})`,);
  return await runGuestCommand({
    command,
    // libvirt namespaces VMs under the prefix.
    domain: `${VM_PREFIX}${name}`,
    osFamily: meta.osFamily,
    shell: meta.shell,
  },);
}
