/**
 An operational failure the mutation tooling can explain without a stack trace.
 */
export class CargoMutantsToolingError extends Error {
  /**
   Names the failing input in the message.

   @param message - affected input and the valid remediation

   @param options - underlying failure retained for diagnostics
   */
  public constructor(
    message: string,
    options?: ErrorOptions,
  ) {
    super(
      message,
      options,
    );
    this.name = 'CargoMutantsToolingError';
  }
}
