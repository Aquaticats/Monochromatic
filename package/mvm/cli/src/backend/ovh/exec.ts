/**
 Exec, interactive shell, file transfer, and ephemeral run for the ovh backend.

 Each operation resolves the target instance once, prefix-scoped and exact, so
 it can never act on a foreign instance that merely shares the `mvm-` shape, and
 connects as the account the instance's image names. {@link ovhRun} reuses the
 shared {@link ephemeralRun} helper with this backend's own operations.

 @module
 */

import { ephemeralRun, } from '../ephemeral-run.ts';
import type { ExecResult, } from '../../exec.ts';
import { validateOvhName, } from './config.ts';
import {
  findMvmInstance,
  instanceLoginUser,
  publicIpv4,
} from './instances.ts';
import {
  ovhClone,
  ovhCreate,
  ovhDestroy,
} from './lifecycle.ts';
import {
  scpPull,
  scpPush,
  sshExec,
  sshShell,
} from './ssh.ts';

//region Target resolution

/**
 Resolves a VM name to where it answers and as whom.
 
 @param name - VM name without the mvm- prefix
 
 @returns public IPv4 and the login account its image names
 
 @throws Error when the name is invalid, no single owned instance matches, or it has no public address
 
 @example
 ```ts
 const { ip, user } = await resolveTarget({ name: 'dev-01' });
 ```
 */
async function resolveTarget({
  name,
}: {
  readonly name: string;
},): Promise<{
  readonly ip: string;
  readonly user: string;
}> {
  validateOvhName(name,);
  /**
   The one instance mvm owns under that name.
   */
  const instance = await findMvmInstance({ name, },);
  return {
    ip: publicIpv4({ instance, },),
    user: await instanceLoginUser({ instance, },),
  };
}

//endregion Target resolution

//region Operations

/**
 Runs a command inside a VM over SSH.
 
 @param command - command to run in the remote login shell
 
 @param name - VM name without the mvm- prefix
 
 @returns captured stdout, stderr, and exit code
 
 @example
 ```ts
 const result = await ovhExec({ command: 'uname -a', name: 'dev-01' });
 ```
 */
export async function ovhExec({
  command,
  name,
}: {
  readonly command: string;
  readonly name: string;
},): Promise<ExecResult> {
  return sshExec({
    command,
    ...await resolveTarget({ name, },),
  },);
}

/**
 Opens an interactive SSH session to a VM.
 
 @param name - VM name without the mvm- prefix
 
 @example
 ```ts
 await ovhShell({ name: 'dev-01' });
 ```
 */
export async function ovhShell({
  name,
}: {
  readonly name: string;
},): Promise<void> {
  await sshShell({ ...await resolveTarget({ name, },), },);
}

/**
 Pushes a host file into a VM over SCP.
 
 @param guestPath - destination path on the instance
 
 @param hostPath - local source path
 
 @param name - VM name without the mvm- prefix
 
 @returns the path the file was written to
 
 @example
 ```ts
 await ovhPush({ name: 'dev-01', hostPath: '/tmp/a', guestPath: '/tmp/a' });
 ```
 */
export async function ovhPush({
  guestPath,
  hostPath,
  name,
}: {
  readonly name: string;
  readonly hostPath: string;
  readonly guestPath: string;
},): Promise<string> {
  return scpPush({
    guestPath,
    hostPath,
    ...await resolveTarget({ name, },),
  },);
}

/**
 Pulls a file out of a VM over SCP.
 
 @param guestPath - source path on the instance
 
 @param name - VM name without the mvm- prefix
 
 @returns file contents
 
 @example
 ```ts
 const bytes = await ovhPull({ name: 'dev-01', guestPath: '/tmp/out.txt' });
 ```
 */
export async function ovhPull({
  guestPath,
  name,
}: {
  readonly name: string;
  readonly guestPath: string;
},): Promise<Buffer> {
  return scpPull({
    guestPath,
    ...await resolveTarget({ name, },),
  },);
}

/**
 Creates an ephemeral VM, runs a command in it, then destroys it, so a billed
 instance never outlives the call.
 
 @param command - command to run in the remote login shell
 
 @param from - source VM to clone from (fresh VM when omitted)
 
 @returns captured stdout, stderr, and exit code
 
 @example
 ```ts
 const result = await ovhRun({ command: 'uname -a' });
 ```
 */
export function ovhRun({
  command,
  from,
}: {
  readonly command: string;
  readonly from?: string;
},): Promise<ExecResult> {
  return ephemeralRun({
    command,
    ...(from !== undefined ? { from, } : {}),
    ops: {
      clone: ovhClone,
      create: ovhCreate,
      destroy: ovhDestroy,
      exec: ovhExec,
    },
  },);
}

//endregion Operations
