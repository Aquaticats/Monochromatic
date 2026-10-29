/**
 Sinon overload dispatch at the owned sandbox seam. @module
 */
import {
  invokeSandboxMethod,
  type SandboxInvocation,
} from './sandbox-guard.ts';
import { createMethodReplacement, } from './sandbox-lease.ts';
import { guardFakeMutators, } from './sandbox-fake.ts';
import { invokeCollectionFactory, } from './sandbox-install.ts';
import {
  guardCollectionFakes,
  guardTimerController,
} from './sandbox-collection.ts';
import type {
  SandboxOwner,
  SandboxRuntime,
} from './sandbox-owner.ts';
import {
  injectOwnedFactories,
  guardMockController,
} from './sandbox-result.ts';
import { prepareMethodSlot, } from './sandbox-slot.ts';
import {
  isSandboxTarget,
  requireUnownedObject,
  requireUnownedProperty,
  sandboxPropertyKey,
  SINON_VALIDATES_PROPERTY,
} from './sandbox-target.ts';

/**
 All operations share lifecycle state without exposing raw cleanup to test code.
 */
export type SandboxPolicy = {
  /**
   Fresh attempt authority.
   */
  readonly owner: SandboxOwner;
  /**
   Runtime-specific context selection.
   */
  readonly runtime: SandboxRuntime;
  /**
   Cleanup independent of overwritten fake.restore methods.
   */
  readonly leases: Set<() => void>;
  /**
   Restoration of leases and ordinary Sinon replacements.
   */
  readonly restore: () => void;
  /**
   Identifies internal cleanup when guarding a retained fake.restore reference.
   */
  readonly restoring: () => boolean;
};

/**
 Preflights ordinary replacement families against currently leased target properties.

 @param invocation - unmodified Sinon call
 
 @example
 ```ts
 preflightOrdinaryMutation(invocation);
 ```
 */
function preflightOrdinaryMutation(invocation: SandboxInvocation,): void {
  /**
   Only these operations interpret their first argument as a replacement target.
   */
  const targetOperations = new Set([
    'ctx.sinon.stub',
    'ctx.sinon.spy',
    'ctx.sinon.replace',
    'ctx.sinon.replace.usingAccessor',
    'ctx.sinon.replaceGetter',
    'ctx.sinon.replaceSetter',
    'ctx.sinon.define',
  ],);
  /**
   No getters or user key coercions are needed for primitive-key preflight.
   */
  const [target, property,] = invocation.args;
  if (targetOperations.has(invocation.operation,) && isSandboxTarget(target,)) {
    /**
     Unsupported key overloads remain subject to ordinary Sinon validation.
     */
    const key = sandboxPropertyKey(property,);
    if (((typeof key) === 'string') || (key !== SINON_VALIDATES_PROPERTY))
      requireUnownedProperty({
        target,
        key,
        operation: invocation.operation,
      },);
    else if ((property === undefined) && ((invocation.operation === 'ctx.sinon.stub')
      || ((invocation.operation === 'ctx.sinon.spy') && ((typeof target) !== 'function'))))
      requireUnownedObject({
        target,
        operation: invocation.operation,
      },);
  }
  if (invocation.operation === 'ctx.sinon.useFakeTimers') {
    /**
     Sinon supports a custom global in its runtime configuration.
     */
    const custom: unknown = isSandboxTarget(target,) ? Reflect.get(
      target,
      'global',
    ) : undefined;
    /**
     Fake timers are not a contextual operation: prevent overwriting active global method leases.
     */
    const global = isSandboxTarget(custom,) ? custom : globalThis;
    requireUnownedObject({
      target: global,
      operation: invocation.operation,
    },);
    for (const key of [
      'process',
      'performance',
    ] as const) {
      /**
       Nested timer targets also carry process-wide methods.
       */
      const nested: unknown = Reflect.get(
        global,
        key,
      );
      if (isSandboxTarget(nested,))
        requireUnownedObject({
          target: nested,
          operation: invocation.operation,
        },);
    }
  }
}

/**
 Routes supported method factories while preserving ordinary Sinon overloads elsewhere.

 @param invocation - raw factory call after the attempt guard
 
 @param policy - context and cleanup capabilities
 
 @returns unchanged fake identity or a guarded deferred controller
 
 @example
 ```ts
 return dispatchSandboxOperation({ invocation, policy });
 ```
 */
export function dispatchSandboxOperation({
  invocation,
  policy,
}: {
  readonly invocation: SandboxInvocation;
  readonly policy: SandboxPolicy;
},): unknown {
  /**
   Actual input values, without inventing overload shapes from function length.
   */
  const [target, property,] = invocation.args;
  /**
   Root factories distinguish method overloads from detached and whole-object forms.
   */
  const methodFactory = (invocation.operation === 'ctx.sinon.stub') || (invocation.operation === 'ctx.sinon.spy');
  /**
   Normalize numeric properties using JavaScript's primitive key spelling.
   */
  const key = sandboxPropertyKey(property,);
  if (methodFactory && (invocation.args
    .length
    === 2)
    && isSandboxTarget(target,)
    && (((typeof key) === 'string') || (key !== SINON_VALIDATES_PROPERTY))) {
    /**
     Unsupported descriptors delegate to Sinon after collision preflight.
     */
    const slot = prepareMethodSlot({
      target,
      key,
      runtime: policy.runtime,
    },);
    if ((typeof slot) !== 'symbol') {
      /**
       Target configuration must remain private even when fake.value/get/set are used later.
       */
      const replacement = createMethodReplacement({
        slot,
        owner: policy.owner,
        invocation,
        leases: policy.leases,
      },);
      guardFakeMutators({
        value: replacement.fake,
        owner: policy.owner,
        restoring: policy.restoring,
        target: replacement.facade,
        key,
        contextualMethod: true,
      },);
      return replacement.fake;
    }
  }
  if (invocation.operation === `ctx.sinon.${String(Symbol.asyncDispose,)}`) {
    policy.restore();
    return Promise.resolve();
  }
  if ((invocation.operation === 'ctx.sinon.restore') || (invocation.operation === `ctx.sinon.${String(Symbol.dispose,)}`)) {
    policy.restore();
    return undefined;
  }
  if (invocation.operation === 'ctx.sinon.verifyAndRestore') {
    /**
     Restoration remains mandatory when mock verification fails.
     */
    using cleanup = { [Symbol.dispose]: policy.restore, };
    /**
     Verify uses the same sandbox receiver, but cleanup remains runner-owned.
     */
    const verify: unknown = isSandboxTarget(invocation.receiver,) ? Reflect.get(
      invocation.receiver,
      'verify',
    ) : undefined;
    if ((typeof verify) !== 'function')
      return invokeSandboxMethod(invocation,);
    return Reflect.apply(
      verify,
      invocation.receiver,
      [],
    );
  }
  if (invocation.operation === 'ctx.sinon.inject') {
    return injectOwnedFactories({
      invocation,
      owner: policy.owner,
      invoke(next: SandboxInvocation,): unknown {
        return dispatchSandboxOperation({
          invocation: next,
          policy,
        },);
      },
    },);
  }
  preflightOrdinaryMutation(invocation,);
  if (methodFactory && isSandboxTarget(target,)
    && (property === undefined)
    && ((invocation.operation === 'ctx.sinon.stub') || ((typeof target) !== 'function'))) {
    return invokeCollectionFactory({
      invocation,
      target,
      owner: policy.owner,
      restoring: policy.restoring,
    },);
  }
  /**
   Factory behavior not selected for contextual isolation remains owned by Sinon.
   */
  const result = invokeSandboxMethod(invocation,);
  if (methodFactory && ((typeof result) === 'function')) {
    guardFakeMutators({
      value: result,
      owner: policy.owner,
      restoring: policy.restoring,
      ...isSandboxTarget(target,) ? { target, } : {},
      ...((typeof key) === 'symbol') && (key === SINON_VALIDATES_PROPERTY) ? {} : { key, },
    },);
  }
  if ((methodFactory && ((typeof result) !== 'function')) || (invocation.operation === 'ctx.sinon.createStubInstance'))
    guardCollectionFakes({
      value: result,
      owner: policy.owner,
      restoring: policy.restoring,
    },);
  if (invocation.operation === 'ctx.sinon.useFakeTimers')
    guardTimerController({
      value: result,
      owner: policy.owner,
      restoring: policy.restoring,
    },);
  if (invocation.operation === 'ctx.sinon.mock')
    return guardMockController({
      value: result,
      target,
      owner: policy.owner,
      restoring: policy.restoring,
    },);
  return result;
}
