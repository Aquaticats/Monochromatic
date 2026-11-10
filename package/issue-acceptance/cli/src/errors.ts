/**
 Failure vocabulary for the issue-acceptance command.

 Every failure names the input it refused and the remedy,
 so a session reading the output can act without the source.

 @module
 */

/**
 Reports a failure this command names in its own words.
 */
export class IssueAcceptanceError extends Error {
  /**
   Creates a named failure.

   @param message - Text the operator reads, naming the input and the remedy.

   @example
   ```ts
   throw new IssueAcceptanceError('the plan file names no issue');
   ```
   */
  public constructor(message: string,) {
    super(message,);
    this.name = 'IssueAcceptanceError';
  }
}

/**
 Reports command invocation misuse,
 which the entry point maps to exit status two.
 */
export class CliInvocationError extends IssueAcceptanceError {
  /**
   Creates an invocation failure.

   @param message - Text naming the flag or argument to correct.

   @example
   ```ts
   throw new CliInvocationError('settle needs --plan <file>');
   ```
   */
  public constructor(message: string,) {
    super(message,);
    this.name = 'CliInvocationError';
  }
}

/**
 Reports a plan file that is not JSON or does not match the plan schema.
 */
export class PlanError extends IssueAcceptanceError {
  /**
   Creates a plan failure.

   @param message - Text naming the plan field and what it must hold.

   @example
   ```ts
   throw new PlanError('the plan\'s issues[0].issue must be a positive integer');
   ```
   */
  public constructor(message: string,) {
    super(message,);
    this.name = 'PlanError';
  }
}

/**
 Reports a child process that could not run,
 ran past its deadline,
 or answered something unreadable.
 */
export class ChildProcessError extends IssueAcceptanceError {
  /**
   Standard error the child wrote,
   kept so the operator sees the tool's own words.
   */
  public readonly stderr: string;

  /**
   Child exit status when the child ran and exited.

   Absent when the child never started or was ended by its deadline.
   */
  public readonly exitCode?: number;

  /**
   Creates a child process failure.

   @param message - Text naming the child and the work it was doing.

   @param stderr - Standard error the child wrote.

   @param exitCode - Child exit status, absent when it never exited.

   @example
   ```ts
   throw new ChildProcessError({ message: 'git grep failed', stderr: '', exitCode: 128 });
   ```
   */
  public constructor({
    message,
    stderr,
    exitCode,
  }: {
    readonly message: string;
    readonly stderr: string;
    readonly exitCode?: number;
  },) {
    super(message,);
    this.name = 'ChildProcessError';
    this.stderr = stderr;
    if (exitCode !== undefined) {
      this.exitCode = exitCode;
    }
  }
}
