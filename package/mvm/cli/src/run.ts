import { ephemeralRun, } from './backend/ephemeral-run.ts';
import { clone, } from './clone.ts';
import { create, } from './create.ts';
import { destroy, } from './destroy.ts';
import {
  exec,
  type ExecResult,
} from './exec.ts';

/**
 Creates an ephemeral libvirt VM, executes a command inside it, and destroys
 the VM. When `from` is provided, the VM is cloned from that source; otherwise
 a fresh VM is created from the base image.
 
 Delegates to the backend-neutral {@link ephemeralRun}, supplying the libvirt
 create/clone/destroy/exec operations, so teardown on interruption is handled
 in one shared place.
 
 @param command - shell command to run inside the VM
 
 @param from - source VM to clone from (creates fresh VM when undefined)
 
 @returns captured stdout, stderr, and exit code from the command
 
 @throws Error when VM creation, command execution, or cleanup fails
 
 @example
 ```ts
 const result = await run({ command: 'uname -a' });
 // Creates a fresh VM, runs the command, destroys it
 
 const cloned = await run({ command: 'cat /etc/hostname', from: 'dev-01' });
 // Clones from dev-01, runs the command, destroys the clone
 ```
 */
export function run(
  {
    command,
    from,
  }: {
    readonly command: string;
    readonly from?: string;
  },
): Promise<ExecResult> {
  return ephemeralRun({
    command,
    ...(from !== undefined ? { from, } : {}),
    ops: {
      clone,
      create,
      destroy,
      exec,
    },
  },);
}
