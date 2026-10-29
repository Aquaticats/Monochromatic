/**
 Cleanup runs every independent lease release even when a Sinon restorer fails. @module
 */
import { SandboxCleanupError, } from './sandbox-error.ts';

/**
 Runs independent restoration steps and retains every failure instead of stopping at the first.

 @param steps - captured lease releases followed by ordinary Sinon restoration
 
 @throws SandboxCleanupError when any restoration step failed
 
 @example
 ```ts
 restoreSandboxSteps([...leases, restoreRawSandbox]);
 ```
 */
export function restoreSandboxSteps(steps: readonly (() => void)[],): void {
  /**
   Aggregate errors without silently abandoning later restoration steps.
   */
  const errors: unknown[] = [];
  for (const restore of steps) {
    try {
      restore();
    }
    catch (error) {
      errors.push(error,);
    }
  }
  if (errors.length > 0)
    throw new SandboxCleanupError(
      errors,
      'Sandbox restoration failed',
    );
}
