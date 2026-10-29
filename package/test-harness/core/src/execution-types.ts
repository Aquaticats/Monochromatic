/**
 Execution metadata retained by asynchronous descendants for diagnostics. @module
 */

import type { Logger, } from '@monochromatic-dev/logger-core/ts';
import type { SandboxOwner, } from '@monochromatic-dev/test-harness-core-sandbox/ts';

/**
 Test and suite bodies share the same observation boundary.
 */
export type ExecutionKind = 'test' | 'suite';

/**
 Metadata describes the awaited body without exposing a mutable verdict.
 */
export type ObservedExecution = {
  /**
   Current test-body attempt; absent for suites and legacy descriptor-only contexts.
   */
  readonly sandboxOwner?: SandboxOwner;
  /**
   Body category used by diagnostic guidance.
   */
  readonly kind: ExecutionKind;
  /**
   Complete hierarchy logger inherited by detached asynchronous work.
   */
  readonly logger: Logger;
  /**
   Whether descriptor execution has settled, including a timeout before its work ends.
   */
  phase: 'running' | 'completed';
};

/**
 Distinguishes reporter failures from failures in code under test.
 */
export type ReportingExecution = {
  /**
   Reporting must not recursively invoke the same formatter and logger.
   */
  readonly kind: 'reporting';
};

/**
 Arguments for one descriptor invocation, including nested and repeated runs.
 */
export type ExecutionOptions<Result,> = {
  /**
   Body category attached to the async execution context.
   */
  readonly kind: ExecutionKind;
  /**
   Explicit or inherited base logger before adding this descriptor's name.
   */
  readonly logger?: Logger;
  /**
   Descriptor name, with empty root names remaining invisible.
   */
  readonly name: string;
  /**
   Existing runner retains ownership of its returned promise and verdict.
   */
  readonly run: () => Promise<Result>;
};
