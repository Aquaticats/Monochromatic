/**
 Shared Node observer with async attribution and a file-lifetime failure flag. @module
 */

import { AsyncLocalStorage, } from 'node:async_hooks';
import { writeSync, } from 'node:fs';
import process from 'node:process';
import { types, } from 'node:util';
import {
  NO_SANDBOX_OWNER,
  type SandboxRuntime,
  type SandboxOwner,
  SandboxOwnershipError,
} from '@monochromatic-dev/test-harness-core-sandbox/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import type {
  ExecutionOptions,
  ObservedExecution,
  ReportingExecution,
} from './execution-types.ts';
import { reportUnhandledRejection, } from './rejection-report.ts';

//region Shared state — module copies use one observation protocol

/**
 Shared key also joins source imports and separately bundled artifact copies.
 */
const RUNTIME_KEY: unique symbol = Symbol.for('@monochromatic-dev/test-harness-core/async-failure-runtime/v1',);

/**
 All copies in one global realm must share storage as well as the observer.
 */
type RejectionRuntime = {
  /**
   Sticky file failure, independent of completed test-body results.
   */
  failed: boolean;
  /**
   Reports not yet finished when Node reaches its final exit callback.
   */
  pendingReports: number;
  /**
   Reporter work has a separate context to prevent recursive diagnostics.
   */
  readonly reporting: ReportingExecution;
  /**
   Rejecting execution context survives nested, concurrent, and completed bodies.
   */
  readonly storage: AsyncLocalStorage<ObservedExecution | ReportingExecution>;
};

//endregion Shared state

//region Failure reporting — retain file status without recursive diagnostics

/**
 Preserves existing nonzero statuses while preventing a leaked rejection from passing.

 @example
 ```ts
 markFileFailed();
 ```
 */
function markFileFailed(): void {
  if ((process.exitCode === undefined) || (Number(process.exitCode,) === 0))
    process.exitCode = 1;
}

/**
 Writes emergency diagnostics even during exit or with a corked stderr stream.
 The normal logger remains responsible for all rich diagnostics.

 @param message - bounded terminal diagnostic without user-value formatting

 @example
 writeTerminalFallback('[error] [module-test] reporter did not finish\n');
 */
function writeTerminalFallback(message: string,): void {
  // oxlint-disable-next-line no-restricted-syntax/no-sync -- Node exit cannot drain async writes; corked-stderr regression and no option in rule schema documented in doc/troubleshooting/module-test-unhandled-rejection.md.
  writeSync(
    2,
    message,
  );
}

/**
 Writes a terminal fallback without inspecting a potentially failing diagnostic object.
 Reusing the formatter or logger here could create another detached rejection.

 @param reason - value whose category explains why rich reporting was unavailable

 @example
 ```ts
 reportDiagnosticFailure(new Error('formatter failed'));
 ```
 */
function reportDiagnosticFailure(reason: unknown,): void {
  writeTerminalFallback(
    '[error] [module-test] [async work] [FAIL] Rejection reporting failed '
      + `(diagnostic value type: ${typeof reason}); the test file remains failed. `
      + 'The original diagnostic could not be fully reported without re-entering failing reporting code.\n',
  );
}

/**
 Consumes formatter and logger failures so the reporting task does not leak another promise.

 @param runtime - shared state containing the outstanding-report count

 @param reason - original rejected value

 @param execution - rejecting test or suite context, when available

 @example
 ```ts
 await finishReport({ runtime, reason, execution, });
 ```
 */
async function finishReport({
  runtime,
  reason,
  execution,
}: {
  readonly execution?: ObservedExecution;
  readonly reason: unknown;
  readonly runtime: RejectionRuntime;
},): Promise<void> {
  /**
   Count completion on both normal reporting and the fallback path.
   */
  using completion = {
    [Symbol.dispose](): void {
      runtime.pendingReports -= 1;
    },
  };
  try {
    await reportUnhandledRejection({
      reason,
      ...execution === undefined ? {} : { execution, },
    },);
  }
  catch (error: unknown) {
    reportDiagnosticFailure(error,);
  }
}

//endregion Failure reporting

//region Runtime observation — install once and retain ownership until exit

/**
 Acquires a single observer for this Node global realm without changing import behavior.

 @returns shared observation state retained until process exit

 @example
 ```ts
 const runtime = rejectionRuntime();
 ```
 */
function rejectionRuntime(): RejectionRuntime {
  /**
   Versioned registry isolates this protocol from unrelated global properties.
   */
  const registry = globalThis as typeof globalThis & { [RUNTIME_KEY]?: RejectionRuntime; };
  /**
   Previously installed runtime may come from another bundled copy of module-test.
   */
  const existing = registry[RUNTIME_KEY];
  if (existing !== undefined)
    return existing;

  /**
   Publish the shared state before EventEmitter new-listener callbacks can run.
   */
  const runtime: RejectionRuntime = {
    failed: false,
    pendingReports: 0,
    reporting: { kind: 'reporting', },
    storage: new AsyncLocalStorage<ObservedExecution | ReportingExecution>(),
  };
  registry[RUNTIME_KEY] = runtime;

  /**
   Observes escaped work without rejecting an already settled test-body promise.

   @param reason - value supplied by Node's rejection event
   */
  function onUnhandledRejection(reason: unknown,): void {
    runtime.failed = true;
    markFileFailed();
    /**
     Node restores the rejection's async context before emitting this event.
     */
    const context = runtime.storage
      .getStore();
    if (context?.kind === 'reporting') {
      reportDiagnosticFailure(reason,);
      return;
    }
    runtime.pendingReports += 1;
    // The task consumes its own rejection; reporting descendants use a separate context.
    void runtime.storage
      .run(
      runtime.reporting,
      function reportInOwnContext(): Promise<void> {
        return finishReport({
          runtime,
          reason,
          ...context === undefined ? {} : { execution: context, },
        },);
      },
      );
  }

  /**
   Reassert sticky failure even if later test code assigned an exit status of zero.
   */
  function onExit(): void {
    if (!runtime.failed)
      return;
    markFileFailed();
    if (runtime.pendingReports > 0)
      writeTerminalFallback(
        '[error] [module-test] [async work] [FAIL] Process exited before rejection diagnostics finished.\n',
      );
  }

  process.on(
    'unhandledRejection',
    onUnhandledRejection,
  );
  process.on(
    'exit',
    onExit,
  );
  return runtime;
}

//endregion Runtime observation

//region Descriptor execution — async descendants retain their diagnostic context

/**
 Preserves the existing body promise while retaining attribution in asynchronous descendants.

 @param options - runner callback and current descriptor metadata

 @returns unchanged descriptor result after the awaited body settles

 @throws original runner failure without consuming it as an unhandled rejection

 @example
 ```ts
 await runNodeExecution({ kind: 'test', name: 'saves', run: runTest, });
 ```
 */
export function runNodeExecution<Result,>(options: ExecutionOptions<Result>,): Promise<Result> {
  /**
   Single runtime shared by every descriptor invocation in this realm.
   */
  const runtime = rejectionRuntime();
  /**
   Nested explicit awaits can inherit a diagnostic logger from their async parent.
   */
  const parent = runtime.storage
    .getStore();
  /**
   Existing caller-provided logger takes precedence over implicit ancestry.
   */
  const baseLogger = options.logger ?? (parent?.kind === 'reporting' ? undefined : parent?.logger);
  /**
   Empty root descriptors do not add a visible hierarchy segment.
   */
  const logger = options.name === ''
    ? baseLogger ?? tagged({ tag: 'module-test', },)
    : tagged({
      tag: options.name,
      ...baseLogger === undefined ? {} : { l: baseLogger, },
    },);
  /**
   Descriptor settlement updates timing metadata, never an emitted verdict.
   */
  const execution: ObservedExecution = {
    kind: options.kind,
    logger,
    phase: 'running',
  };
  return runtime.storage
    .run(
    execution,
    async function runBody(): Promise<Result> {
        /**
         Detached descendants keep this context after its returned promise settles.
         */
      using lifetime = {
        [Symbol.dispose](): void {
          execution.phase = 'completed';
        },
      };
      return await options.run();
    },
    );
}

//endregion Descriptor execution

/**
 Supplies attempt context using the observer's existing realm-shared storage.
 The optional owner field extends the observation protocol without changing its required shape.

 @returns Node sandbox runtime, with ordinary suites and reporting contexts excluded
 
 @example
 ```ts
 const runtime = nodeSandboxRuntime();
 await runtime.run({ owner, body });
 ```
 */
export function nodeSandboxRuntime(): SandboxRuntime {
  /**
   Reuse storage across source imports and artifact copies.
   */
  const runtime = rejectionRuntime();
  return {
    contextual: true,
    current(): SandboxOwner | typeof NO_SANDBOX_OWNER {
      /**
       Suites never implicitly inherit a parent's method replacements.
       */
      const context = runtime.storage
        .getStore();
      return context?.kind === 'test' ? context.sandboxOwner ?? NO_SANDBOX_OWNER : NO_SANDBOX_OWNER;
    },
    isProxy: types.isProxy,
    async run({
      owner,
      body,
    }): Promise<void> {
      /**
       Attempt execution must be nested in the already observed test descriptor.
       */
      const parent = runtime.storage
        .getStore();
      if ((parent === undefined) || (parent.kind !== 'test'))
        throw new SandboxOwnershipError('A sandbox attempt must run inside an observed test execution.',);
      /**
       Detached descendants keep this attempt's diagnostic phase, not a later repeat's phase.
       */
      const execution: ObservedExecution = {
        ...parent,
        sandboxOwner: owner,
        phase: 'running',
      };
      await runtime.storage
        .run(
        execution,
        async function runAttempt(): Promise<void> {
            /**
             Keep late rejection attribution tied to this completed attempt.
             */
          using completion = {
            [Symbol.dispose](): void {
              execution.phase = 'completed';
            },
          };
          await body();
        },
        );
    },
  };
}
