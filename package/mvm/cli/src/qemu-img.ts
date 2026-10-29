/**
 Disk image operations through `qemu-img`.

 @module
 */

import {
  libvirtTools,
  QEMU_IMG_COMMAND_ENV,
  toolNotFoundRemedy,
} from './libvirt-tools.ts';
import { spawn, } from './spawn.ts';

/**
 Runs `qemu-img` through the command {@link libvirtTools} chose for this host.
 No deadline applies: converting a disk image can take minutes.

 @param args - Array of command-line arguments for qemu-img

 @returns Trimmed stdout output

 @throws {@link ExecutableNotFoundError} when the command's executable does not exist; the message says how to configure another

 @throws {@link CommandFailedError} when qemu-img exits with non-zero code

 @example
 ```ts
 await qemuImg({ args: ['create', '-f', 'qcow2', '/vms/dev/disk.qcow2', '20G'] });
 ```
 */
export async function qemuImg({ args, }: { readonly args: readonly string[]; },): Promise<string> {
  /**
   Command that runs qemu-img on this host.
   */
  const { qemuImg: tool, } = await libvirtTools();
  /**
   Executable and the arguments that precede qemu-img's own.
   */
  const [command, ...leading] = tool.argv;
  return await spawn({
    args: [
      ...leading,
      ...args,
    ],
    command,
    notFoundRemedy: toolNotFoundRemedy({
      command: tool,
      tool: 'qemu-img',
      variable: QEMU_IMG_COMMAND_ENV,
    },),
  },);
}
