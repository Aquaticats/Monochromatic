/**
 Failure of the release helpers to accept their input or to find what a release edit needs.
 */
export class CrateReleaseError extends Error {
  /**
   Names the failing input in the message.

   @param message - affected input and the valid remediation

   @param options - `cause`, the error that made the input unusable, kept for diagnosis
   */
  public constructor(
    message: string,
    options?: { readonly cause?: unknown }
  ) {
    super(
      message,
      options
    );
    this.name = 'CrateReleaseError';
  }
}
