/**
 Hetzner exec, interactive shell, file transfer, and ephemeral run.
 
 Each operation resolves the target server label-scoped (so it can never act
 on an unrelated `mvm-<name>` server) to obtain its public IPv4, then connects
 over SSH/SCP. {@link hetznerRun} reuses the shared {@link ephemeralRun} helper with
 the Hetzner operations rather than the libvirt-bound exported `run`.
 
 @module
 */

import { ephemeralRun, } from '../ephemeral-run.ts';
import type { ExecResult, } from '../../exec.ts';
import { getMvmServerByName, } from './api-resources.ts';
import { validateHetznerName, } from './config.ts';
import {
  hetznerClone,
  hetznerCreate,
  hetznerDestroy,
} from './lifecycle.ts';
import {
  scpPull,
  scpPush,
  sshExec,
  sshShell,
} from './ssh.ts';

//region Server resolution

/**
 Resolves a VM name to its public IPv4, label-scoped and exact.
 
 @param name - VM name without the mvm- prefix
 
 @returns public IPv4 address of the matching server
 
 @throws Error when the name is invalid, no single server matches, or it has no IPv4
 
 @example
 ```ts
 const ip = await resolveIp('dev-01');
 ```
 */
async function resolveIp(name: string,): Promise<string> {
  validateHetznerName(name,);
  /**
   Matching server, resolved through the label-scoped exact lookup.
   */
  const server = await getMvmServerByName({ name, },);
  /**
   Public IPv4 of the server, when one is attached.
   */
  const ip = server.public_net
    .ipv4
    ?.ip;
  if ((ip === undefined) || (ip === '')) {
    throw new Error(`VM ${name} has no public IPv4`,);
  }
  return ip;
}

//endregion Server resolution

//region Operations

/**
 Runs a command inside a Hetzner VM over SSH.
 
 @param command - command to run in the remote login shell
 
 @param name - VM name without the mvm- prefix
 
 @returns captured stdout, stderr, and exit code
 
 @example
 ```ts
 const result = await hetznerExec({ command: 'uname -a', name: 'dev-01' });
 ```
 */
export async function hetznerExec(
  {
    command,
    name,
  }: {
    readonly command: string;
    readonly name: string;
  },
): Promise<ExecResult> {
  return sshExec({
    command,
    ip: await resolveIp(name,),
  },);
}

/**
 Opens an interactive SSH session to a Hetzner VM.
 
 @param name - VM name without the mvm- prefix
 
 @example
 ```ts
 await hetznerShell({ name: 'dev-01' });
 ```
 */
export async function hetznerShell({ name, }: { readonly name: string; },): Promise<void> {
  await sshShell({ ip: await resolveIp(name,), },);
}

/**
 Pushes a host file into a Hetzner VM over SCP.
 
 @param guestPath - absolute destination path on the server
 
 @param hostPath - local source path
 
 @param name - VM name without the mvm- prefix
 
 @returns remote path the file was written to
 
 @example
 ```ts
 await hetznerPush({ name: 'dev-01', hostPath: '/tmp/a', guestPath: '/root/a' });
 ```
 */
export async function hetznerPush(
  {
    guestPath,
    hostPath,
    name,
  }: {
    readonly name: string;
    readonly hostPath: string;
    readonly guestPath: string;
  },
): Promise<string> {
  return scpPush({
    guestPath,
    hostPath,
    ip: await resolveIp(name,),
  },);
}

/**
 Pulls a file from a Hetzner VM over SCP.
 
 @param guestPath - absolute source path on the server
 
 @param name - VM name without the mvm- prefix
 
 @returns file contents
 
 @example
 ```ts
 const bytes = await hetznerPull({ name: 'dev-01', guestPath: '/root/out.txt' });
 ```
 */
export async function hetznerPull(
  {
    guestPath,
    name,
  }: {
    readonly name: string;
    readonly guestPath: string;
  },
): Promise<Buffer> {
  return scpPull({
    guestPath,
    ip: await resolveIp(name,),
  },);
}

/**
 Creates an ephemeral Hetzner VM, runs a command, then destroys it.
 
 @param command - command to run in the remote login shell
 
 @param from - source VM to clone from (fresh VM when omitted)
 
 @param memoryGiB - least memory the fresh machine may have, in gibibytes; a
 clone keeps the source's size, so a floor beside `from` is refused
 
 @param vcpus - fewest virtual CPUs the fresh machine may have; a clone keeps
 the source's size, so a floor beside `from` is refused
 
 @returns captured stdout, stderr, and exit code
 
 @throws Error when a floor is asked beside a source machine to clone
 
 @example
 ```ts
 const result = await hetznerRun({ command: 'uname -a' });
 const sized = await hetznerRun({ command: 'nproc', memoryGiB: 4, vcpus: 2 });
 ```
 */
export function hetznerRun(
  {
    command,
    from,
    memoryGiB,
    vcpus,
  }: {
    readonly command: string;
    readonly from?: string;
    readonly memoryGiB?: number;
    readonly vcpus?: number;
  },
): Promise<ExecResult> {
  return ephemeralRun({
    command,
    ...(from !== undefined ? { from, } : {}),
    ...((memoryGiB === undefined) ? {} : { memoryGiB, }),
    ...((vcpus === undefined) ? {} : { vcpus, }),
    ops: {
      clone: hetznerClone,
      create: hetznerCreate,
      destroy: hetznerDestroy,
      exec: hetznerExec,
    },
  },);
}

//endregion Operations
