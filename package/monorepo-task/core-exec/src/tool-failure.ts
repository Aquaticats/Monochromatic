/**
 Failures of the `task-*` wrappers that run one external tool.

 Each wrapper forwards the tool's own output and keeps its own exit status.
 These classes cover the two cases where the tool never produces a status of
 its own, so a throw, not an assignment, decides what the wrapper exits with.

 @module
 */

/**
 What a wrapper knows about a tool it could not start or could not let finish.
 */
export type ToolFailureFacts = {
  /**
   Task that owns the wrapper, such as `task-oxlint`; it prefixes the message
   the person reads in the task output.
   */
  readonly task: string;
  /**
   Command the wrapper runs, such as `oxlint`.
   */
  readonly tool: string;
  /**
   What happened: the spawn failure text, or the name of the signal that
   ended the tool.
   */
  readonly detail: string;
};

/**
 Captured result of a command that ran and failed, as `nano-spawn` reports it.
 */
export type CapturedSubprocessFailure = {
  /**
   Captured standard output, empty when the command printed nothing.
   */
  readonly stdout?: string;
  /**
   Captured standard error, empty when the command printed nothing.
   */
  readonly stderr?: string;
  /**
   Status the command exited with; absent when a signal ended it.
   */
  readonly exitCode?: number;
  /**
   Signal that ended the command; absent when it exited on its own.
   */
  readonly signalName?: string;
};

/**
 Tell a command that ran and failed from one that never started.

 `nano-spawn` gives a command that ran either an `exitCode` number or a
 `signalName` string. A command that never started carries `exitCode:
 undefined` and no signal, so the presence of the key alone proves nothing;
 the value does.

 @param error - value caught from a spawn call

 @returns whether the caught value is a failed command's captured result
 
 @example
 ```ts
 if (isCapturedSubprocessFailure(error,)) forward(error.stdout, error.stderr);
 ```
 */
export function isCapturedSubprocessFailure(error: unknown,): error is CapturedSubprocessFailure {
  if ((error === null) || ((typeof error) !== 'object')) return false;
  if (('signalName' in error) && ((typeof error.signalName) === 'string')) return true;
  return ('exitCode' in error) && ((typeof error.exitCode) === 'number');
}

/**
 Thrown when the wrapped command never started, as when its executable is
 absent from `PATH`.

 @example
 ```ts
 throw new ToolNotStartedError({ task: 'task-tsc', tool: 'tsc', detail: 'spawn tsc ENOENT', });
 ```
 */
export class ToolNotStartedError extends Error {
  /**
   @param task - task that owns the wrapper, which prefixes the message
   
   @param tool - command the wrapper could not start
   
   @param detail - what the spawn call reported
   */
  constructor({
    task,
    tool,
    detail,
  }: ToolFailureFacts,) {
    super(`[${task}] failed to execute ${tool}: ${detail}`,);
    this.name = 'ToolNotStartedError';
  }
}

/**
 Thrown when the wrapped command was killed by a signal, so it reported no
 exit status and its diagnostics are incomplete.

 @example
 ```ts
 throw new ToolKilledBySignalError({ task: 'task-oxlint', tool: 'oxlint', detail: 'SIGKILL', });
 ```
 */
export class ToolKilledBySignalError extends Error {
  /**
   @param task - task that owns the wrapper, which prefixes the message
   
   @param tool - command the signal ended
   
   @param detail - name of the signal that ended it
   */
  constructor({
    task,
    tool,
    detail,
  }: ToolFailureFacts,) {
    super(`[${task}] ${tool} terminated by signal: ${detail}`,);
    this.name = 'ToolKilledBySignalError';
  }
}
