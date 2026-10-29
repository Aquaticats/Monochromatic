/**
 Runs one host command to completion and returns what it printed.

 The command's output goes to private temporary files and the call completes
 when the command itself exits. It does not wait for the output streams to
 close: a background process the command leaves behind, such as the libvirt
 session daemon that `virsh` starts on demand, keeps inherited streams open
 for as long as it lives, which made one piped `virsh` call take two minutes.

 @module
 */

import { spawn as startProcess, } from 'node:child_process';
import { once, } from 'node:events';
import {
  mkdtemp,
  open,
  readFile,
  rm,
} from 'node:fs/promises';
import { tmpdir, } from 'node:os';
import { join, } from 'node:path';
import { tagged, } from '@monochromatic-dev/module-logger/ts';

import {
  CommandFailedError,
  CommandTimedOutError,
  ExecutableNotFoundError,
  isMissingExecutable,
  renderCommandLine,
} from './spawn-errors.ts';

/**
 Logger root for mvm after removing the package log shim.

 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l, },);
 ```
 */
const l = tagged({ tag: 'mvm', },);

//region Output capture

/**
 Private files receiving one command's output, removed when the scope ends.
 */
type Capture = AsyncDisposable & {
  /**
   Reads everything the command wrote so far.
   */
  readonly read: () => Promise<{
    readonly stderr: string;
    readonly stdout: string;
  }>;
  /**
   Descriptor the command's standard error is connected to.
   */
  readonly stderrFd: number;
  /**
   Descriptor the command's standard output is connected to.
   */
  readonly stdoutFd: number;
};

/**
 Creates the files one command writes its output to, in a directory only this user can enter.

 @returns Capture whose disposal empties and removes the files

 @example
 ```ts
 await using capture = await openCapture();
 ```
 */
async function openCapture(): Promise<Capture> {
  /**
   Fresh directory readable by this user only.
   */
  const directory = await mkdtemp(join(
    tmpdir(),
    'mvm-spawn-',
  ),);
  /**
   Path of the file receiving standard output.
   */
  const stdoutPath = join(
    directory,
    'stdout',
  );
  /**
   Path of the file receiving standard error.
   */
  const stderrPath = join(
    directory,
    'stderr',
  );
  /**
   Open files handed to the command.
   */
  const [stdoutFile, stderrFile,] = await Promise.all([
    open(
      stdoutPath,
      'w',
    ),
    open(
      stderrPath,
      'w',
    ),
  ],);
  return {
    async read() {
      /**
       Both streams, read by path so the read starts at the beginning of each file.
       */
      const [stdout, stderr,] = await Promise.all([
        readFile(
          stdoutPath,
          'utf8',
        ),
        readFile(
          stderrPath,
          'utf8',
        ),
      ],);
      return {
        stderr,
        stdout,
      };
    },
    stderrFd: stderrFile.fd,
    stdoutFd: stdoutFile.fd,
    async [Symbol.asyncDispose]() {
      // A process that inherited the files may keep them open after they are removed; emptying them frees their space now.
      await Promise.all([
        stdoutFile.truncate(0,),
        stderrFile.truncate(0,),
      ],);
      await Promise.all([
        stdoutFile.close(),
        stderrFile.close(),
      ],);
      await rm(
        directory,
        {
          force: true,
          recursive: true,
        },
      );
    },
  };
}

//endregion Output capture

//region Exit

/**
 How a command ended: an exit status, or the name of the signal that ended it.
 */
type Ending = {
  readonly exitCode?: number;
  readonly signal?: string;
};

/**
 Waits for a started command to exit.

 @param child - Process returned by `node:child_process` spawn

 @param command - Executable name or path, named when it does not exist

 @param notFoundRemedy - Advice added to the error when the executable does not exist

 @returns Exit status or signal name

 @throws {@link ExecutableNotFoundError} when the executable does not exist

 @example
 ```ts
 const ending = await endingOf({ child, command: 'virsh', notFoundRemedy: '' });
 ```
 */
async function endingOf({
  child,
  command,
  notFoundRemedy,
}: {
  readonly child: ReturnType<typeof startProcess>;
  readonly command: string;
  readonly notFoundRemedy: string;
},): Promise<Ending> {
  /**
   Logger scoped to the wait so a start failure is attributable.
   */
  const rl = tagged({
    tag: endingOf.name,
    l,
  },);
  try {
    /**
     Values of the `exit` event: exit status or `null`, then signal name or `null`.
     */
    const exited: readonly unknown[] = await once(
      child,
      'exit',
    );
    /**
     Exit status of the command; `null` when a signal ended it.
     */
    const [exitCode, signal,] = exited;
    return {
      ...(((typeof exitCode) === 'number') ? { exitCode, } : {}),
      ...(((typeof signal) === 'string') ? { signal, } : {}),
    };
  }
  catch (error) {
    if (isMissingExecutable(error,)) {
      rl.debug(`executable ${command} does not exist`,);
      throw new ExecutableNotFoundError({
        cause: error,
        executable: command,
        remedy: notFoundRemedy,
      },);
    }
    throw error;
  }
}

//endregion Exit

//region Spawn

/**
 Signal that stops a command at its deadline.
 */
const DEADLINE_SIGNAL = 'SIGKILL';

/**
 Spawns a command and returns its trimmed stdout.
 Logs the command at debug level before execution.

 @param args - Arguments array for the command

 @param command - Command name or path to execute

 @param deadlineMs - Milliseconds after which a still-running command is stopped; omitted for commands that may run long

 @param env - Environment variables set on top of the current environment, when the command's output must not depend on the caller's settings

 @param notFoundRemedy - Advice added to the error when the executable does not exist, such as how to configure another one

 @returns Trimmed stdout output from the command

 @throws {@link ExecutableNotFoundError} when the executable does not exist

 @throws {@link CommandFailedError} when the command exits with a nonzero status or a signal ends it

 @throws {@link CommandTimedOutError} when the command is still running at its deadline

 @example
 ```ts
 const output = await spawn({ command: 'virsh', args: ['list', '--all'] });
 ```
 */
export async function spawn(
  {
    args,
    command,
    deadlineMs,
    env = {},
    notFoundRemedy = '',
  }: {
    readonly args: readonly string[];
    readonly command: string;
    readonly deadlineMs?: number;
    readonly env?: Readonly<Record<string, string>>;
    readonly notFoundRemedy?: string;
  },
): Promise<string> {
  /**
   Tagged logger so the debug line names the spawn call site.
   */
  const rl = tagged({
    tag: spawn.name,
    l,
  },);
  /**
   Command line as shown in logs and errors.
   */
  const commandLine = renderCommandLine({
    args,
    command,
  },);
  rl.debug(commandLine,);

  /**
   Files the command writes to; a process it leaves behind cannot hold this call open through them.
   */
  await using capture = await openCapture();
  /**
   When the command started, to tell a deadline stop from another kill.
   */
  const startedAt = Date.now();
  /**
   Running command.
   */
  const child = startProcess(
    command,
    [...args,],
    {
      env: {
        ...process.env,
        ...env,
      },
      stdio: [
        'ignore',
        capture.stdoutFd,
        capture.stderrFd,
      ],
      ...(deadlineMs === undefined
        ? {}
        : {
          killSignal: DEADLINE_SIGNAL,
          timeout: deadlineMs,
        }),
    },
  );
  /**
   How the command ended.
   */
  const ending = await endingOf({
    child,
    command,
    notFoundRemedy,
  },);
  /**
   Everything the command printed before it exited.
   */
  const {
    stderr,
    stdout,
  } = await capture.read();

  if ((deadlineMs !== undefined)
    && (ending.signal === DEADLINE_SIGNAL)
    && ((Date.now() - startedAt) >= deadlineMs))
    {
    throw new CommandTimedOutError({
      commandLine,
      deadlineMs,
    },);
  }
  if (ending.exitCode !== 0) {
    throw new CommandFailedError({
      commandLine,
      ...ending,
      stderr,
      stdout,
    },);
  }
  return stdout.trim();
}

//endregion Spawn
