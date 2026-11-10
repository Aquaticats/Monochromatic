/**
 Failures of the `task-*` wrappers that run one external tool.

 Each wrapper forwards the tool's own output and keeps its own exit status.
 Two classes cover the cases where the tool never produces a status of its own,
 so a throw, not an assignment, decides what the wrapper exits with. The third
 covers a tool that does report a status which its own children made untrue: a
 crashed analysis process leaves the tool counting the diagnostics that survived,
 so the run reports a clean or failing lint while an analyzer's findings are
 missing.

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

/**
 Thrown when one of the wrapped command's own child processes crashed during the
 run, so the diagnostics it reported are those of the analyzers that survived.

 The wrapped tool has an exit status of its own in this case, and that status is
 the reason this class exists: a crashed analyzer contributes no diagnostics, so
 the tool counts what is left and reports a clean run whenever the rest of the
 tree is clean. Treating that status as evidence reads the crash as a clean tree.

 @example
 ```ts
 throw new AnalysisCrashError({
   task: 'task-oxlint',
   tool: 'oxlint',
   detail: 'panic: runtime error: invalid memory address or nil pointer dereference',
 },);
 ```
 */
export class AnalysisCrashError extends Error {
  /**
   @param task - task that owns the wrapper, which prefixes the message
   
   @param tool - command whose child crashed
   
   @param detail - first line of the crash report the child printed
   */
  constructor({
    task,
    tool,
    detail,
  }: ToolFailureFacts,) {
    super(
      `[${task}] ${tool} reported a crash in one of its analysis processes, so this run's findings are incomplete: ${detail}. Find or report that crash; a run whose analyzer died cannot say the code is clean.`,
    );
    this.name = 'AnalysisCrashError';
  }
}
