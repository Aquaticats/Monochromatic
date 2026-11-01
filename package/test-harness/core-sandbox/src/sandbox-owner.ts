/**
 Attempt identities and runtime capabilities for test-owned mocking. @module
 */
import type { Logger, } from '@monochromatic-dev/logger-core/ts';
import { SandboxOwnershipError, } from './sandbox-error.ts';

/**
 Execution has no test-body ownership, including suites and reporting contexts.
 */
export const NO_SANDBOX_OWNER: unique symbol = Symbol('execution has no sandbox owner',);

/**
 Identity is the object itself, never the reusable test name or descriptor.
 */
export type SandboxOwner = {
  /**
   Test name used only for diagnostics.
   */
  readonly name: string;
  /**
   Already hierarchy-tagged test logger.
   */
  readonly l: Logger;
  /**
   Completion is irreversible, including when a timed-out body keeps running.
   */
  phase: 'running' | 'completed';
};

/**
 Runtime adapter; ordinary runtimes do not claim async-context isolation.
 */
export type SandboxRuntime = {
  /**
   Whether this runtime can select property values by async execution context.
   */
  readonly contextual: boolean;
  /**
   Current attempt, if any; callers must also inspect its phase.
   */
  readonly current: () => SandboxOwner | typeof NO_SANDBOX_OWNER;
  /**
   Excludes proxy targets whose traps cannot be rolled back by the harness.
   */
  readonly isProxy: (target: object,) => boolean;
  /**
   Executes a body without replacing its promise or error contract.
   */
  readonly run: (options: {
    /**
     Fresh attempt identity.
     */
    readonly owner: SandboxOwner;
    /**
     Body including its timeout boundary.
     */
    readonly body: () => Promise<void>;
  },) => Promise<void>;
};

/**
 Rejects retained sandbox factories before they can install new target state.

 @param owner - attempt whose context supplied the factory
 
 @param operation - user-facing call name
 
 @throws SandboxOwnershipError when the attempt has already settled
 
 @example
 ```ts
 requireRunningOwner({ owner, operation: 'ctx.sinon.stub', });
 ```
 */
export function requireRunningOwner({
  owner,
  operation,
}: {
  readonly owner: SandboxOwner;
  readonly operation: string;
},): void {
  if (owner.phase === 'completed') {
    throw new SandboxOwnershipError(
      `${operation} belongs to completed test "${owner.name}". `
        + 'Await or cancel this work before the test finishes; use the current test context for new work.',
    );
  }
}
