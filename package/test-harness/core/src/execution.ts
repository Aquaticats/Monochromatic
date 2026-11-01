/**
 Activates runtime observation only when a descriptor actually executes. @module
 */

import type { ExecutionOptions, } from './execution-types.ts';

/**
 Runs a descriptor with Node rejection attribution when that runtime is present.
 Ordinary imports and lazy descriptor construction install no listeners.

 @param options - runner callback and diagnostic metadata for this invocation

 @returns unchanged result of the descriptor's awaited body

 @throws original runner error when the awaited body rejects

 @example
 ```ts
 await runObservedExecution({ kind: 'test', name: 'saves', run: runTest, });
 ```
 */
export async function runObservedExecution<Result,>(
  options: ExecutionOptions<Result>,
): Promise<Result> {
  if (((typeof process) === 'undefined')
    || ((typeof process.versions
      ?.node) !== 'string'))
    return options.run();

  /**
   Lazy import keeps Node built-ins outside the ordinary import boundary.
   */
  const { runNodeExecution, } = await import('./execution-node.ts');
  return runNodeExecution(options,);
}
