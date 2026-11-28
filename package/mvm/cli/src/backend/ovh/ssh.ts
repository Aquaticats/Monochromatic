/**
 SSH exec, interactive shell, and SCP file transfer for the ovh backend.

 Commands run over `ssh` with the managed key as the login account the image
 names, never a fixed one: OVHcloud's Linux images log in as an unprivileged
 account, so `root` would fail. Transfers use `scp`, which uses the SFTP
 protocol by default on OpenSSH 9.0+, so remote paths are literal SFTP paths and
 are never re-parsed by a remote shell. The command in `exec` is passed as a
 single argv element and runs in the remote login shell, matching the libvirt
 backend's `bash -c`. Host keys are not persisted because OVHcloud recycles
 public addresses and this address was just read over the authenticated API.

 Both tools come from {@link sshCommand} and {@link scpCommand}, so a wrapper, or
 a test, can replace either.

 @module
 */

import {
  mkdtemp,
  readFile,
  rm,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { wait, } from '@monochromatic-dev/async-time-core/ts';
import nanoSpawn from 'nano-spawn';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import type { ExecResult, } from '../../exec.ts';
import {
  ExecutableNotFoundError,
  isMissingExecutable,
} from '../../spawn-errors.ts';
import { spawn, } from '../../spawn.ts';
import { scratchParent, } from '../../scratch-parent.ts';
import {
  OPENSSH_REMEDY,
  scpCommand,
  sshCommand,
} from './config.ts';
import { privateKeyPath, } from './ssh-key.ts';

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
 Per-connection timeout in seconds passed to ssh and scp.
 */
const CONNECT_TIMEOUT_S = 10;

/**
 How long to wait for SSH to answer after an instance reports itself running.
 Unmeasured against OVHcloud: taken from the Hetzner backend's equivalent and
 corrected once the billed test reports what a Canadian instance needs.
 */
const SSH_READY_TIMEOUT_MS = 120_000;

/**
 Delay between SSH-readiness probes.
 */
const SSH_READY_POLL_MS = 2_000;

/**
 Exit code ssh returns for its own connection-level failures, as opposed to the
 remote command's exit code.
 */
const SSH_CONNECTION_FAILURE = 255;

//endregion Constants

//region Arg builders

/**
 Shared ssh and scp options: managed key, no host-key persistence, quiet
 logging, and a bounded connect timeout.
 
 @returns option tokens common to ssh and scp
 
 @example
 ```ts
 ovhSshBaseOpts(); // ['-i', '<data dir>/id_ed25519', '-o', 'UserKnownHostsFile=/dev/null', ...]
 ```
 */
export function ovhSshBaseOpts(): readonly string[] {
  return [
    '-i',
    privateKeyPath(),
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
 Builds the `user@host` connection target from the account the image names.
 
 @param ip - instance public IPv4
 
 @param user - login account the image names
 
 @returns `<user>@<ip>` connection target
 
 @example
 ```ts
 ovhConnectionTarget({ ip: '192.0.2.10', user: 'ubuntu' }); // 'ubuntu@192.0.2.10'
 ```
 */
export function ovhConnectionTarget({
  ip,
  user,
}: {
  readonly ip: string;
  readonly user: string;
},): string {
  return `${user}@${ip}`;
}

/**
 Builds the ssh argv for running one command remotely. The command is one argv
 element, so ssh forwards it verbatim to the remote login shell.
 
 @param command - command to run in the remote login shell
 
 @param ip - instance public IPv4
 
 @param user - login account the image names
 
 @returns ssh argument vector, without the executable
 
 @example
 ```ts
 ovhSshExecArgs({ ip: '192.0.2.10', user: 'ubuntu', command: 'echo "a;b"' });
 ```
 */
export function ovhSshExecArgs({
  command,
  ip,
  user,
}: {
  readonly command: string;
  readonly ip: string;
  readonly user: string;
},): readonly string[] {
  return [
    ...ovhSshBaseOpts(),
    ovhConnectionTarget({
      ip,
      user,
    },),
    command,
  ];
}

/**
 Builds the scp argv for pushing a local file to a remote path. The remote
 target is one argv element; scp's SFTP mode treats the path literally.
 
 @param guestPath - destination path on the instance
 
 @param hostPath - local source path
 
 @param ip - instance public IPv4
 
 @param user - login account the image names
 
 @returns scp argument vector, without the executable
 
 @example
 ```ts
 ovhScpPushArgs({ ip: '192.0.2.10', user: 'ubuntu', hostPath: '/tmp/a', guestPath: '/home/ubuntu/a' });
 ```
 */
export function ovhScpPushArgs({
  guestPath,
  hostPath,
  ip,
  user,
}: {
  readonly guestPath: string;
  readonly hostPath: string;
  readonly ip: string;
  readonly user: string;
},): readonly string[] {
  return [
    ...ovhSshBaseOpts(),
    hostPath,
    `${ovhConnectionTarget({
      ip,
      user,
    },)}:${guestPath}`,
  ];
}

/**
 Builds the scp argv for pulling a remote file to a local path.
 
 @param guestPath - source path on the instance
 
 @param ip - instance public IPv4
 
 @param localPath - local destination path
 
 @param user - login account the image names
 
 @returns scp argument vector, without the executable
 
 @example
 ```ts
 ovhScpPullArgs({ ip: '192.0.2.10', user: 'ubuntu', guestPath: '/home/ubuntu/a', localPath: '/tmp/a' });
 ```
 */
export function ovhScpPullArgs({
  guestPath,
  ip,
  localPath,
  user,
}: {
  readonly guestPath: string;
  readonly ip: string;
  readonly localPath: string;
  readonly user: string;
},): readonly string[] {
  return [
    ...ovhSshBaseOpts(),
    `${ovhConnectionTarget({
      ip,
      user,
    },)}:${guestPath}`,
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
 
 @throws {@link ExecutableNotFoundError} when `ssh` itself does not exist
 
 @example
 ```ts
 try { await nanoSpawn('ssh', args); } catch (error) { return execResultFromError(error); }
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
     Remote or ssh exit code, defaulting to the connection-failure code.
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
 Runs a command on an instance over SSH and captures its output and exit code.
 
 @param command - command to run in the remote login shell
 
 @param ip - instance public IPv4
 
 @param user - login account the image names
 
 @returns captured stdout, stderr, and exit code
 
 @example
 ```ts
 const result = await sshExec({ ip: '192.0.2.10', user: 'ubuntu', command: 'uname -a' });
 ```
 */
export async function sshExec({
  command,
  ip,
  user,
}: {
  readonly command: string;
  readonly ip: string;
  readonly user: string;
},): Promise<ExecResult> {
  /**
   Command and its leading arguments, from the environment or `ssh`.
   */
  const [executable, ...leading] = sshCommand();
  try {
    /**
     Successful run captures both streams; exit code is zero by definition here.
     */
    const {
      stderr,
      stdout,
    } = await nanoSpawn(
      executable,
      [
        ...leading,
        ...ovhSshExecArgs({
          command,
          ip,
          user,
        },),
      ],
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
 Polls until SSH answers on a freshly booted instance, or the timeout elapses.
 
 @param ip - instance public IPv4
 
 @param user - login account the image names
 
 @throws Error when SSH does not answer before the timeout
 
 @example
 ```ts
 await waitForSsh({ ip: '192.0.2.10', user: 'ubuntu' });
 ```
 */
export async function waitForSsh({
  ip,
  user,
}: {
  readonly ip: string;
  readonly user: string;
},): Promise<void> {
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
  rl.debug(`waiting for SSH on ${ovhConnectionTarget({
    ip,
    user,
  },)}`,);
  while (Date.now() < deadline) {
    /**
     Probe result; a zero exit means the instance accepted the connection.
     */
    // oxlint-disable-next-line no-await-in-loop -- deliberate serial readiness polling
    const probe = await sshExec({
      command: 'true',
      ip,
      user,
    },);
    if (probe.exitCode === 0) {
      return;
    }
    // oxlint-disable-next-line no-await-in-loop -- deliberate serial readiness polling
    await wait(SSH_READY_POLL_MS,);
  }
  throw new Error(
    `SSH did not answer on ${ovhConnectionTarget({
      ip,
      user,
    },)} within ${
      String(SSH_READY_TIMEOUT_MS,)
    }ms`,
  );
}

//endregion Exec

//region Interactive shell

/**
 Opens an interactive SSH session to an instance, forwarding the exit code.
 
 @param ip - instance public IPv4
 
 @param user - login account the image names
 
 @example
 ```ts
 await sshShell({ ip: '192.0.2.10', user: 'ubuntu' });
 ```
 */
export async function sshShell({
  ip,
  user,
}: {
  readonly ip: string;
  readonly user: string;
},): Promise<void> {
  /**
   Logger scoped to this session so connect messages are namespaced.
   */
  const rl = tagged({
    tag: sshShell.name,
    l,
  },);
  /**
   Command and its leading arguments, from the environment or `ssh`.
   */
  const [executable, ...leading] = sshCommand();
  rl.info(`connecting to ${ovhConnectionTarget({
    ip,
    user,
  },)} (Ctrl+D or exit to disconnect)`,);
  try {
    await nanoSpawn(
      executable,
      [
        ...leading,
        ...ovhSshBaseOpts(),
        ovhConnectionTarget({
          ip,
          user,
        },),
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
 
 @param guestPath - destination path on the instance
 
 @param hostPath - local source path
 
 @param ip - instance public IPv4
 
 @param user - login account the image names
 
 @returns the remote path the file was written to
 
 @throws Error when the transfer fails
 
 @example
 ```ts
 await scpPush({ ip: '192.0.2.10', user: 'ubuntu', hostPath: '/tmp/s.sh', guestPath: '/tmp/s.sh' });
 ```
 */
export async function scpPush({
  guestPath,
  hostPath,
  ip,
  user,
}: {
  readonly guestPath: string;
  readonly hostPath: string;
  readonly ip: string;
  readonly user: string;
},): Promise<string> {
  /**
   Command and its leading arguments, from the environment or `scp`.
   */
  const [executable, ...leading] = scpCommand();
  await spawn({
    command: executable,
    args: [
      ...leading,
      ...ovhScpPushArgs({
        guestPath,
        hostPath,
        ip,
        user,
      },),
    ],
    notFoundRemedy: OPENSSH_REMEDY,
  },);
  return guestPath;
}

/**
 Pulls a remote file over SCP (SFTP mode) and returns its bytes. Downloads to a
 throwaway directory removed on scope exit.
 
 @param guestPath - source path on the instance
 
 @param ip - instance public IPv4
 
 @param user - login account the image names
 
 @returns file contents
 
 @throws Error when the transfer fails
 
 @example
 ```ts
 const bytes = await scpPull({ ip: '192.0.2.10', user: 'ubuntu', guestPath: '/tmp/out.txt' });
 ```
 */
export async function scpPull({
  guestPath,
  ip,
  user,
}: {
  readonly guestPath: string;
  readonly ip: string;
  readonly user: string;
},): Promise<Buffer> {
  /**
   Throwaway directory holding the downloaded file; removed on scope exit.
   */
  const dir = await mkdtemp(join(
    await scratchParent(),
    'mvm-pull-',
  ),);
  /**
   Disposable guard removing the directory once the bytes are read.
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
   Local destination for the downloaded file.
   */
  const localPath = join(
    dir,
    'pulled',
  );
  /**
   Command and its leading arguments, from the environment or `scp`.
   */
  const [executable, ...leading] = scpCommand();
  await spawn({
    command: executable,
    args: [
      ...leading,
      ...ovhScpPullArgs({
        guestPath,
        ip,
        localPath,
        user,
      },),
    ],
    notFoundRemedy: OPENSSH_REMEDY,
  },);
  return readFile(localPath,);
}

//endregion Transfer
