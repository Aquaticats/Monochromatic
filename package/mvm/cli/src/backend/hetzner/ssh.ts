/**
 SSH exec, interactive shell, and SCP file transfer for the Hetzner backend.
 
 Commands run over `ssh` with the managed key; transfers use `scp`, which uses
 the SFTP protocol by default on OpenSSH 9.0+ (so remote paths are literal SFTP
 paths, never re-parsed by a remote shell). The command in `exec` is passed as
 a single argv element and runs in the remote login shell, matching libvirt's
 `bash -c`. Host keys are not persisted because Hetzner recycles public IPv4
 and the address was just obtained over the authenticated API.
 
 @module
 */

import {
  mkdtemp,
  readFile,
  rm,
} from 'node:fs/promises';
import { tmpdir, } from 'node:os';
import { join, } from 'node:path';

import { wait, } from '@monochromatic-dev/module-async-time/ts';
import nanoSpawn from 'nano-spawn';
import { tagged, } from '@monochromatic-dev/module-logger/ts';

import type { ExecResult, } from '../../exec.ts';
import {
  ExecutableNotFoundError,
  isMissingExecutable,
} from '../../spawn-errors.ts';
import { spawn, } from '../../spawn.ts';
import {
  OPENSSH_REMEDY,
  SSH_USER,
} from './config.ts';
import { PRIVATE_KEY_PATH, } from './ssh-key.ts';

/**
 Logger root for mvm after removing the package log shim.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l, },);
 ```
 */
const l = tagged({ tag: 'mvm', },);

//region Constants

/**
 Per-connection timeout in seconds passed to ssh/scp.
 */
const CONNECT_TIMEOUT_S = 10;

/**
 Maximum time to wait for SSH to answer after a server boots.
 */
const SSH_READY_TIMEOUT_MS = 120_000;

/**
 Delay between SSH-readiness probes.
 */
const SSH_READY_POLL_MS = 2_000;

/**
 Exit code ssh returns for its own connection-level failures (vs the remote
 command's exit code).
 */
const SSH_CONNECTION_FAILURE = 255;

//endregion Constants

//region Arg builders

/**
 Shared ssh/scp options: managed key, no host-key persistence, quiet logging,
 and a bounded connect timeout.
 
 @returns option tokens common to ssh and scp
 
 @example
 ```ts
 sshBaseOpts(); // ['-i', '/path/id_ed25519', '-o', 'UserKnownHostsFile=/dev/null', ...]
 ```
 */
export function sshBaseOpts(): readonly string[] {
  return [
    '-i',
    PRIVATE_KEY_PATH,
    '-o',
    'UserKnownHostsFile=/dev/null',
    '-o',
    'StrictHostKeyChecking=no',
    '-o',
    'LogLevel=ERROR',
    '-o',
    `ConnectTimeout=${String(CONNECT_TIMEOUT_S,)}`,
  ];
}

/**
 Builds the `user@host` connection target.
 
 @param ip - server public IPv4
 
 @returns `root@<ip>` connection target
 
 @example
 ```ts
 connectionTarget('203.0.113.7'); // builds the root SSH login target
 ```
 */
export function connectionTarget(ip: string,): string {
  return `${SSH_USER}@${ip}`;
}

/**
 Builds the ssh argv for running a single command remotely.
 `command` is one argv element, so ssh forwards it verbatim to the remote
 login shell (no local shell, no interpolation by mvm).
 
 @param command - command to run in the remote login shell
 
 @param ip - server public IPv4
 
 @returns ssh argument vector
 
 @example
 ```ts
 sshExecArgs({ ip: '203.0.113.7', command: 'echo "a;b"' });
 ```
 */
export function sshExecArgs(
  {
    command,
    ip,
  }: {
    readonly command: string;
    readonly ip: string;
  },
): readonly string[] {
  return [
    ...sshBaseOpts(),
    connectionTarget(ip,),
    command,
  ];
}

/**
 Builds the scp argv for pushing a local file to a remote path.
 The remote target is a single argv element; scp's default SFTP mode treats
 the path literally, so it is not shell-quoted.
 
 @param guestPath - destination path on the server
 
 @param hostPath - local source path
 
 @param ip - server public IPv4
 
 @returns scp argument vector
 
 @example
 ```ts
 scpPushArgs({ ip: '203.0.113.7', hostPath: '/tmp/a', guestPath: '/root/a' });
 ```
 */
export function scpPushArgs(
  {
    guestPath,
    hostPath,
    ip,
  }: {
    readonly guestPath: string;
    readonly hostPath: string;
    readonly ip: string;
  },
): readonly string[] {
  return [
    ...sshBaseOpts(),
    hostPath,
    `${connectionTarget(ip,)}:${guestPath}`,
  ];
}

/**
 Builds the scp argv for pulling a remote file to a local path.
 
 @param guestPath - source path on the server
 
 @param ip - server public IPv4
 
 @param localPath - local destination path
 
 @returns scp argument vector
 
 @example
 ```ts
 scpPullArgs({ ip: '203.0.113.7', guestPath: '/root/a', localPath: '/tmp/a' });
 ```
 */
export function scpPullArgs(
  {
    guestPath,
    ip,
    localPath,
  }: {
    readonly guestPath: string;
    readonly ip: string;
    readonly localPath: string;
  },
): readonly string[] {
  return [
    ...sshBaseOpts(),
    `${connectionTarget(ip,)}:${guestPath}`,
    localPath,
  ];
}

//endregion Arg builders

//region Exec

/**
 Converts a nano-spawn error into an {@link ExecResult}, preserving the remote
 command's captured output and exit code.
 
 @param error - thrown value from nano-spawn
 
 @returns exec result reconstructed from the error
 
 @throws the original value when it is not a subprocess error
 
 @throws {@link ExecutableNotFoundError} when `ssh` itself does not exist, which is not a result of the remote command
 
 @example
 ```ts
 try { await nanoSpawn('ssh', args); } catch (err) { return execResultFromError(err); }
 ```
 */
function execResultFromError(error: unknown,): ExecResult {
  if (isMissingExecutable(error,)) {
    throw new ExecutableNotFoundError({
      cause: error,
      executable: 'ssh',
      remedy: OPENSSH_REMEDY,
    },);
  }
  if ((error !== null) && ((typeof error) === 'object')
    && ('stdout' in error)
    && ('stderr' in error)) {
    /**
     Captured stdout if the error carries a string one, else empty.
     */
    const stdout = ((typeof error.stdout) === 'string') ? error.stdout : '';
    /**
     Captured stderr if the error carries a string one, else empty.
     */
    const stderr = ((typeof error.stderr) === 'string') ? error.stderr : '';
    /**
     Remote/ssh exit code, defaulting to the ssh connection-failure code.
     */
    const exitCode = (('exitCode' in error) && ((typeof error.exitCode) === 'number'))
      ? error.exitCode
      : SSH_CONNECTION_FAILURE;
    return {
      exitCode,
      stderr,
      stdout,
    };
  }
  throw error;
}

/**
 Runs a command on a server over SSH and captures its output and exit code.
 
 @param command - command to run in the remote login shell
 
 @param ip - server public IPv4
 
 @returns captured stdout, stderr, and exit code
 
 @example
 ```ts
 const result = await sshExec({ ip: '203.0.113.7', command: 'uname -a' });
 ```
 */
export async function sshExec(
  {
    command,
    ip,
  }: {
    readonly command: string;
    readonly ip: string;
  },
): Promise<ExecResult> {
  try {
    /**
     Successful run captures both streams; exit code is zero by definition here.
     */
    const {
      stdout,
      stderr,
    } = await nanoSpawn(
      'ssh',
      [...sshExecArgs({
        command,
        ip,
      },),],
    );
    return {
      exitCode: 0,
      stderr,
      stdout,
    };
  }
  catch (error: unknown) {
    return execResultFromError(error,);
  }
}

/**
 Polls until SSH answers on a freshly booted server, or the timeout elapses.
 
 @param ip - server public IPv4
 
 @throws Error when SSH does not become reachable before the timeout
 
 @example
 ```ts
 await waitForSsh({ ip: '203.0.113.7' });
 ```
 */
export async function waitForSsh({ ip, }: { readonly ip: string; },): Promise<void> {
  /**
   Logger scoped to this wait so readiness polling is namespaced.
   */
  const rl = tagged({
    tag: waitForSsh.name,
    l,
  },);
  /**
   Deadline after which readiness polling gives up.
   */
  const deadline = Date.now() + SSH_READY_TIMEOUT_MS;
  rl.debug(`waiting for SSH on ${ip}`,);
  while (Date.now() < deadline) {
    /**
     Probe result; a zero exit means the server accepted the connection.
     */
    // oxlint-disable-next-line no-await-in-loop -- deliberate serial readiness polling
    const probe = await sshExec({
      command: 'true',
      ip,
    },);
    if (probe.exitCode === 0) {
      return;
    }
    // oxlint-disable-next-line no-await-in-loop -- deliberate serial readiness polling
    await wait(SSH_READY_POLL_MS,);
  }
  throw new Error(
    `SSH did not become ready on ${ip} within ${String(SSH_READY_TIMEOUT_MS,)}ms`,
  );
}

//endregion Exec

//region Interactive shell

/**
 Opens an interactive SSH session to a server, forwarding the exit code.
 
 @param ip - server public IPv4
 
 @example
 ```ts
 await sshShell({ ip: '203.0.113.7' });
 ```
 */
export async function sshShell({ ip, }: { readonly ip: string; },): Promise<void> {
  /**
   Logger scoped to this session so connect messages are namespaced.
   */
  const rl = tagged({
    tag: sshShell.name,
    l,
  },);
  rl.info(`connecting to ${connectionTarget(ip,)} (Ctrl+D or exit to disconnect)`,);
  try {
    await nanoSpawn(
      'ssh',
      [
        ...sshBaseOpts(),
        connectionTarget(ip,),
      ],
      {
        stderr: 'inherit',
        stdin: 'inherit',
        stdout: 'inherit',
      },
    );
  }
  catch (error: unknown) {
    if (isMissingExecutable(error,)) {
      rl.debug('ssh does not exist on PATH',);
      throw new ExecutableNotFoundError({
        cause: error,
        executable: 'ssh',
        remedy: OPENSSH_REMEDY,
      },);
    }
    if ((error !== null) && ((typeof error) === 'object')
      && ('exitCode' in error)) {
      /**
       Forwarded so the shell exit code reflects the ssh session outcome.
       */
      const exitCode = ((typeof error.exitCode) === 'number') ? error.exitCode : undefined;
      if (exitCode !== undefined) {
        process.exitCode = exitCode;
      }
    }
  }
}

//endregion Interactive shell

//region Transfer

/**
 Pushes a local file to a remote path over SCP (SFTP mode).
 
 @param guestPath - destination path on the server
 
 @param hostPath - local source path
 
 @param ip - server public IPv4
 
 @returns the remote path the file was written to
 
 @throws Error when the transfer fails
 
 @example
 ```ts
 await scpPush({ ip: '203.0.113.7', hostPath: '/tmp/setup.sh', guestPath: '/root/setup.sh' });
 ```
 */
export async function scpPush(
  {
    guestPath,
    hostPath,
    ip,
  }: {
    readonly guestPath: string;
    readonly hostPath: string;
    readonly ip: string;
  },
): Promise<string> {
  await spawn({
    command: 'scp',
    args: [...scpPushArgs({
      guestPath,
      hostPath,
      ip,
    },),],
    notFoundRemedy: OPENSSH_REMEDY,
  },);
  return guestPath;
}

/**
 Pulls a remote file over SCP (SFTP mode) and returns its bytes.
 Downloads to a throwaway temp directory that is removed on scope exit.
 
 @param guestPath - source path on the server
 
 @param ip - server public IPv4
 
 @returns file contents
 
 @throws Error when the transfer fails
 
 @example
 ```ts
 const bytes = await scpPull({ ip: '203.0.113.7', guestPath: '/root/out.txt' });
 ```
 */
export async function scpPull(
  {
    guestPath,
    ip,
  }: {
    readonly guestPath: string;
    readonly ip: string;
  },
): Promise<Buffer> {
  /**
   Throwaway directory holding the downloaded file; removed on scope exit.
   */
  const dir = await mkdtemp(join(
    tmpdir(),
    'mvm-pull-',
  ),);
  /**
   Disposable guard removing the temp directory once the bytes are read.
   */
  await using _cleanup = {
    async [Symbol.asyncDispose](): Promise<void> {
      await rm(
        dir,
        {
          force: true,
          recursive: true,
        },
      );
    },
  };
  /**
   Local destination for the downloaded file inside the temp directory.
   */
  const localPath = join(
    dir,
    'pulled',
  );
  await spawn({
    command: 'scp',
    args: [...scpPullArgs({
      guestPath,
      ip,
      localPath,
    },),],
    notFoundRemedy: OPENSSH_REMEDY,
  },);
  return readFile(localPath,);
}

//endregion Transfer
