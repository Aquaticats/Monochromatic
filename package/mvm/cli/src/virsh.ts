/**
 Core libvirt/virsh operations for VM management.
 
 @module
 */

import { writeFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import { MS_PER_SECOND, } from '@monochromatic-dev/const-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import {
  LIBVIRT_URI,
  VM_PREFIX,
} from './config.ts';
import {
  CANNOT_CONNECT_TEXT,
  LibvirtSessionUnavailableError,
  LibvirtUnresponsiveError,
} from './libvirt-errors.ts';
import {
  libvirtTools,
  type ToolCommand,
  toolNotFoundRemedy,
  VIRSH_COMMAND_ENV,
} from './libvirt-tools.ts';
import {
  ensureSessionDaemon,
  LIBVIRT_ENV,
} from './session-daemon.ts';
import {
  CommandFailedError,
  CommandTimedOutError,
} from './spawn-errors.ts';
import { spawn, } from './spawn.ts';

/**
 Logger root for mvm after removing the package log shim.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l, },);
 ```
 */
const l = tagged({ tag: 'mvm', },);

/**
 Milliseconds a virsh call gets unless its caller knows better.

 mvm starts the session daemon itself before its first call (see
 `session-daemon.ts`), measured at a few seconds on a cold host; no operation
 mvm asks for takes minutes, so a call still running after two is stuck.
 */
export const VIRSH_DEADLINE_MS = 120_000;

/**
 Runs one virsh command against the user session QEMU/KVM connection, through
 the command {@link libvirtTools} chose for this host.

 @param args - Array of command-line arguments for virsh

 @param deadlineMs - Milliseconds after which the call is stopped

 @param tool - Command that runs virsh on this host

 @returns Trimmed stdout output

 @throws {@link LibvirtSessionUnavailableError} when virsh cannot connect to libvirt's session daemon

 @throws {@link LibvirtUnresponsiveError} when virsh is still running at its deadline

 @throws {@link ExecutableNotFoundError} when the command's executable does not exist; the message says how to configure another

 @throws {@link CommandFailedError} when virsh exits with non-zero code for another reason

 @example
 ```ts
 const output = await runVirsh({ args: ['list', '--all'], deadlineMs: VIRSH_DEADLINE_MS, tool });
 ```
 */
async function runVirsh({
  args,
  deadlineMs,
  tool,
}: {
  readonly args: readonly string[];
  readonly deadlineMs: number;
  readonly tool: ToolCommand;
},): Promise<string> {
  /**
   Logger scoped to the call so a connection failure is attributable.
   */
  const rl = tagged({
    tag: runVirsh.name,
    l,
  },);
  /**
   Executable and the arguments that precede virsh's own.
   */
  const [command, ...leading] = tool.argv;
  try {
    return await spawn({
      args: [
        ...leading,
        '--connect',
        LIBVIRT_URI,
        ...args,
      ],
      command,
      deadlineMs,
      env: LIBVIRT_ENV,
      notFoundRemedy: toolNotFoundRemedy({
        command: tool,
        tool: 'virsh',
        variable: VIRSH_COMMAND_ENV,
      },),
    },);
  }
  catch (error) {
    if (error instanceof CommandTimedOutError) {
      rl.debug(`virsh was stopped at its deadline of ${String(deadlineMs,)} ms`,);
      throw new LibvirtUnresponsiveError({
        cause: error,
        seconds: Math.round(deadlineMs / MS_PER_SECOND,),
        virsh: tool,
      },);
    }
    if ((error instanceof CommandFailedError)
      && error.stderr
        .includes(CANNOT_CONNECT_TEXT,)) {
      rl.debug('virsh could not connect to the session daemon',);
      throw new LibvirtSessionUnavailableError({
        cause: error,
        stderr: error.stderr,
        virsh: tool,
      },);
    }
    throw error;
  }
}

/**
 Runs a virsh command against the user session QEMU/KVM connection,
 through the command {@link libvirtTools} chose for this host.

 Makes sure libvirt's session daemon answers first, so that no `virsh` call
 starts one inside this command's cgroup (see `session-daemon.ts`). When a call
 finds the daemon gone anyway, the daemon is started again and the call is
 repeated once.
 
 @param args - Array of command-line arguments for virsh
 
 @param deadlineMs - Milliseconds after which the call is stopped; defaults to {@link VIRSH_DEADLINE_MS}
 
 @returns Trimmed stdout output
 
 @throws {@link LibvirtSessionUnavailableError} when virsh cannot connect to libvirt's session daemon
 
 @throws {@link LibvirtUnresponsiveError} when virsh is still running at its deadline
 
 @throws {@link ExecutableNotFoundError} when the command's executable does not exist; the message says how to configure another
 
 @throws {@link CommandFailedError} when virsh exits with non-zero code for another reason
 
 @example
 ```ts
 const output = await virsh({ args: ['list', '--all'] });
 ```
 */
export async function virsh({
  args,
  deadlineMs = VIRSH_DEADLINE_MS,
}: {
  readonly args: readonly string[];
  readonly deadlineMs?: number;
},): Promise<string> {
  /**
   Logger scoped to the call so a restart is attributable.
   */
  const rl = tagged({
    tag: virsh.name,
    l,
  },);
  /**
   Command that runs virsh on this host.
   */
  const { virsh: tool, } = await libvirtTools();
  await ensureSessionDaemon();
  try {
    return await runVirsh({
      args,
      deadlineMs,
      tool,
    },);
  }
  catch (error) {
    if (!(error instanceof LibvirtSessionUnavailableError))
      throw error;

    // The daemon exited since mvm asked, as it does after its idle timeout. Starting it again here keeps virsh from
    // starting it inside this command's cgroup.
    rl.debug('the session daemon stopped answering; starting it again',);
    await ensureSessionDaemon({ force: true, },);
    return runVirsh({
      args,
      deadlineMs,
      tool,
    },);
  }
}

/**
 Defines a VM in libvirt from an XML string.
 Writes the XML to a file in the VM directory, then calls `virsh define`.
 
 @param vmDir - Directory to write the XML file into
 
 @param xml - XML content for the domain definition
 
 @example
 ```ts
 await defineVm({ vmDir: '/vms/myvm', xml: domainXmlString });
 ```
 */
export async function defineVm(
  {
    vmDir,
    xml,
  }: {
    readonly vmDir: string;
    readonly xml: string;
  },
): Promise<void> {
  /**
   XML written to disk because virsh define expects a file path, not stdin.
   */
  const xmlPath = join(
    vmDir,
    'domain.xml',
  );
  await writeFile(
    xmlPath,
    xml,
  );
  await virsh({ args: [
    'define',
    xmlPath,
  ], },);
}

/**
 Starts a defined VM.
 
 @param name - VM name without the mvm- prefix
 
 @example
 ```ts
 await startVm({ name: 'win11' });
 ```
 */
export async function startVm({ name, }: { readonly name: string; },): Promise<void> {
  await virsh({ args: [
    'start',
    `${VM_PREFIX}${name}`,
  ], },);
}

/**
 Force-stops a running VM (equivalent to pulling the power cord).
 
 @param name - VM name without the mvm- prefix
 
 @example
 ```ts
 await destroyVm({ name: 'win11' });
 ```
 */
export async function destroyVm({ name, }: { readonly name: string; },): Promise<void> {
  await virsh({ args: [
    'destroy',
    `${VM_PREFIX}${name}`,
  ], },);
}

/**
 Removes a VM definition and deletes all associated storage volumes.
 
 @param name - VM name without the mvm- prefix
 
 @example
 ```ts
 await undefineVm({ name: 'win11' });
 ```
 */
export async function undefineVm({ name, }: { readonly name: string; },): Promise<void> {
  await virsh({ args: [
    'undefine',
    `${VM_PREFIX}${name}`,
    '--remove-all-storage',
  ], },);
}

/**
 Lists all VMs managed by this tool (those with the `mvm-` prefix).
 
 @returns Array of VM names without the prefix
 
 @example
 ```ts
 const vms = await listVms(); // e.g. ['win11', 'fedora']
 ```
 */
export async function listVms(): Promise<readonly string[]> {
  /**
   Raw virsh list output filtered for the mvm- prefix on subsequent lines.
   */
  const output = await virsh({ args: [
    'list',
    '--all',
    '--name',
  ], },);
  return output
    .split('\n',)
    .filter(function startsWithPrefix(line,) {
      return line.startsWith(VM_PREFIX,);
    },)
    .map(function stripPrefix(line,) {
      return line.slice(VM_PREFIX.length,);
    },);
}
