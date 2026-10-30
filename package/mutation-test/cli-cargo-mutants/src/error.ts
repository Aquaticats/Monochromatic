/**
 Failure of the launcher or the limiter to accept its input.
 */
export class CargoMutantsToolingError extends Error {
  /**
   Names the failing input in the message.

   @param message - affected input and the valid remediation
   */
  public constructor(message: string) {
    super(message);
    this.name = 'CargoMutantsToolingError';
  }
}
