/**
 Errors at the test-owned mocking seam. @module
 */

/**
 Signals an operation that cannot preserve sandbox ownership.

 @example
 ```ts
 throw new SandboxOwnershipError('ctx.sinon.stub belongs to completed test "saves"');
 ```
 */
export class SandboxOwnershipError extends Error {
  /**
   Distinguishes harness ownership diagnostics from ordinary Sinon errors.
   */
  override readonly name = 'SandboxOwnershipError';
}

/**
 Retains every cleanup failure, including a body failure when both boundaries fail.

 @example
 ```ts
 throw new SandboxCleanupError(errors, 'Sandbox cleanup failed');
 ```
 */
export class SandboxCleanupError extends AggregateError {
  /**
   Distinguishes aggregated ownership cleanup from an ordinary assertion failure.
   */
  override readonly name = 'SandboxCleanupError';
}
