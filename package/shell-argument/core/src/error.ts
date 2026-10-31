/**
 Failure of a value to be usable as a command-line argument.
 */
export class ShellArgumentError extends Error {
  /**
   Names the failing input in the message.

   @param message - affected input and the valid remediation
   */
  public constructor(message: string) {
    super(message);
    this.name = 'ShellArgumentError';
  }
}
