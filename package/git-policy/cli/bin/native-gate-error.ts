/**
 The one error the wrapper's release gate commands throw for a refusal a person can act on.

 A refusal names the input that was refused and what a person does next, and carries no stack trace in the log a
 coordinator reads: the commands start a unit, move a worktree and wait for a log, and every way they can refuse is a
 sentence about the gate rather than a defect in this package.
 */
export class NativeGateError extends Error {
  /**
   Stable class name that tells a refusal of the gate commands apart from an unrelated error in a log.
   */
  override name = 'NativeGateError';
}
