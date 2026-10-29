/**
 Test-owned sandbox lifecycle, separate from standalone createSinon configuration. @module
 */
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  createSinon,
  type DisposableSandbox,
} from './sinon.ts';
import {
  guardSandboxCapability,
  type SandboxInvocation,
} from './sandbox-guard.ts';
import type {
  SandboxOwner,
  SandboxRuntime,
} from './sandbox-owner.ts';
import { restoreSandboxSteps, } from './sandbox-cleanup.ts';
import {
  dispatchSandboxOperation,
  type SandboxPolicy,
} from './sandbox-operation.ts';

/**
 Context facade plus runner-only lifetime disposal.
 */
export type OwnedSandbox = Disposable & {
  /**
   Sinon-compatible context surface; retained methods enforce attempt completion.
   */
  readonly sinon: DisposableSandbox;
};

/**
 Creates a fresh context capability for one attempt, not for all repeats of a test.

 @param owner - fresh attempt identity whose completion precedes restoration
 
 @param runtime - async context selection when the execution runtime supports it
 
 @returns guarded sandbox and runner-owned cleanup
 
 @example
 ```ts
 using sandbox = createOwnedSandbox({ owner, runtime });
 await body({ sinon: sandbox.sinon, expect });
 ```
 */
export function createOwnedSandbox({
  owner,
  runtime,
}: {
  readonly owner: SandboxOwner;
  readonly runtime: SandboxRuntime;
},): OwnedSandbox {
  /**
   Never expose the raw sandbox as a retained test capability.
   */
  const raw = createSinon();
  /**
   Cleanup diagnostics carry the test's existing hierarchy.
   */
  const l = tagged({
    tag: createOwnedSandbox.name,
    l: owner.l,
  },);
  /**
   Lease callbacks do not depend on fake.restore remaining unmodified.
   */
  const leases = new Set<() => void>();
  /**
   Only internal restoration may invoke an ordinary fake's restore after completion.
   */
  const cleanupState = { active: false, };
  /**
   Delegate ordinary fake restoration to the sandbox that owns them.
   */
  function restoreOrdinarySandbox(): void {
    raw.restore();
  }
  /**
   Restore every lease before delegating to Sinon's own collection cleanup.
   */
  function restore(): void {
    cleanupState.active = true;
    /**
     Restore internal cleanup authority even if a restorer throws.
     */
    using completion = {
      [Symbol.dispose](): void {
        cleanupState.active = false;
      },
    };
    restoreSandboxSteps([
      ...leases,
      restoreOrdinarySandbox,
    ],);
  }
  /**
   Shared operation policy also guards factories exposed through Sinon injection.
   */
  const policy: SandboxPolicy = {
    owner,
    runtime,
    leases,
    restore,
    restoring(): boolean { return cleanupState.active; },
  };
  l.debug(`creating attempt sandbox (contextual methods: ${String(runtime.contextual,)})`,);
  return {
    sinon: guardSandboxCapability({
      target: raw,
      owner,
      operation: 'ctx.sinon',
      invoke(invocation: SandboxInvocation,): unknown {
        return dispatchSandboxOperation({
          invocation,
          policy,
        },);
      },
    },),
    [Symbol.dispose](): void {
      if (owner.phase === 'completed')
        return;
      owner.phase = 'completed';
      l.debug('closing attempt before sandbox restoration',);
      restore();
    },
  };
}
