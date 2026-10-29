import { spawn, } from 'node:child_process';
import { once, } from 'node:events';
import { resolve, } from 'node:path';

import { tagged, } from '@monochromatic-dev/module-logger/ts';

import { PrivilegeError, } from './errors.ts';
import {
  createPrivilegeContextFile,
  PRIVILEGE_CONTEXT_ARGUMENT,
} from './privilege-context.ts';

/**
 Module logger for root relaunch lifecycle.
 */
const l = tagged({ tag: 'privilege', },);

/**
 Privilege broker resolved in original user's command search path.
 */
const SUDO_COMMAND = 'sudo';

/**
 Exit-status sentinel for child terminated by signal.
 */
const SIGNAL_EXIT_CODE = -1;

/**
 Relaunches exact Node runtime and CLI artifact through sudo when current process is non-root.
 
 Inherited standard streams let sudo use terminal for authentication.
 Exact runtime and script paths avoid relying on sudo's restricted command search path.
 Process globals are read at this ownership boundary so unresolved host APIs receive no
 caller-owned object or capability.
 
 @returns Whether operation was delegated to privileged child.
 
 @throws {@link PrivilegeError} when invocation lacks script path,
 sudo cannot start,
 or privileged child fails.
 
 @example
 ```ts
 await relaunchWithRootIfNeeded();
 ```
 */
export async function relaunchWithRootIfNeeded(): Promise<boolean> {
  /**
   Current effective UID,
   defaulting to root semantics when host API is unavailable.
   */
  const currentUid = process.geteuid?.() ?? 0;
  if (currentUid === 0) {
    l.debug('already running with root privileges',);
    return false;
  }
  /**
   CLI module and operation arguments supplied by Node runtime.
   */
  const [, scriptArgument, ...processArguments] = process.argv;
  if (scriptArgument === undefined)
    throw new PrivilegeError('Node did not provide wg-quicker script path.',);
  /**
   Absolute CLI module path for exact privileged re-execution.
   */
  const scriptPath = resolve(scriptArgument,);
  /**
   Private context preserving allowlisted user settings despite sudo env reset.
   */
  await using contextFile = await createPrivilegeContextFile();
  /**
   Exact command boundary passed after sudo's option terminator.
   */
  const sudoArguments = [
    '--',
    process.execPath,
    scriptPath,
    PRIVILEGE_CONTEXT_ARGUMENT,
    contextFile.path,
    ...processArguments,
  ];
  l.debug(`relaunching through sudo: ${process.execPath} ${scriptPath}`,);
  /**
   Interactive privilege broker sharing terminal streams with invoking user.
   */
  const child = spawn(
    SUDO_COMMAND,
    sudoArguments,
    { stdio: 'inherit', },
  );
  try {
    await once(
      child,
      'close',
    );
  }
  catch (error) {
    l.error(`failed to start sudo: ${String(error,)}`,);
    throw new PrivilegeError(
      'Unable to start sudo for wg-quicker.',
      { cause: error, },
    );
  }
  /**
   Exit status available after close event unless child ended by signal.
   */
  const { exitCode, } = child;
  if (exitCode !== 0) {
    /**
     Numeric status with signal termination represented by sentinel.
     */
    const renderedExitCode = String(exitCode ?? SIGNAL_EXIT_CODE,);
    throw new PrivilegeError(
      `Privileged wg-quicker process failed with exit code ${renderedExitCode}.`,
    );
  }
  return true;
}
