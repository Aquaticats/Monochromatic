/**
 Lazy runtime adapter keeps Node built-ins outside neutral module imports. @module
 */
import {
  NO_SANDBOX_OWNER,
  type SandboxRuntime,
} from '@monochromatic-dev/test-harness-core-sandbox/ts';

/**
 Resolves context support using the same runtime gate as descriptor observation.
 Non-Node execution preserves ordinary Sinon behavior rather than claiming isolation.

 @returns runtime appropriate for the current descriptor execution environment
 
 @example
 ```ts
 const runtime = await sandboxRuntime();
 ```
 */
export async function sandboxRuntime(): Promise<SandboxRuntime> {
  if (((typeof process) !== 'undefined') && ((typeof process.versions
    ?.node) === 'string')) {
    /**
     Lazy load is essential for the browser artifact's import boundary.
     */
    const { nodeSandboxRuntime, } = await import('./execution-node.ts');
    return nodeSandboxRuntime();
  }
  return {
    contextual: false,
    current(): typeof NO_SANDBOX_OWNER { return NO_SANDBOX_OWNER; },
    isProxy(): boolean { return false; },
    run({ body, }): Promise<void> { return body(); },
  };
}
