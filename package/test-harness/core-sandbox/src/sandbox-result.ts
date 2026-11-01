/**
 Mock-controller and injected factory lifetimes at the sandbox boundary. @module
 */
import { SandboxOwnershipError, } from './sandbox-error.ts';
import {
  guardSandboxCapability,
  invokeSandboxMethod,
  type SandboxInvocation,
} from './sandbox-guard.ts';
import type { SandboxOwner, } from './sandbox-owner.ts';
import {
  isSandboxTarget,
  requireUnownedProperty,
  sandboxPropertyKey,
  SINON_VALIDATES_PROPERTY,
} from './sandbox-target.ts';

/**
 Protects mock controllers whose `expects` method installs the object proxy later.

 @param value - raw controller registered with Sinon's collection
 
 @param target - input object supplied to `ctx.sinon.mock`
 
 @param owner - capability owner
 
 @param restoring - runner-owned restoration authority
 
 @returns guarded controller or unchanged standalone expectation

 @example
 ```ts
 return guardMockController({ value, target, owner, restoring });
 ```
 */
export function guardMockController({
  value,
  target,
  owner,
  restoring,
}: {
  readonly value: unknown;
  readonly target: unknown;
  readonly owner: SandboxOwner;
  readonly restoring: () => boolean;
},): unknown {
  if ((!isSandboxTarget(value,)) || (!isSandboxTarget(target,)))
    return value;
  /**
   Mock verification itself calls restore on the raw controller.
   */
  const generation = { restored: false, };
  /**
   Capture the real restorer before exposing the controller.
   */
  const restore: unknown = Reflect.get(
    value,
    'restore',
  );
  if ((typeof restore) === 'function') {
    Reflect.set(
      value,
      'restore',
      new Proxy(
        restore,
        {
          apply(
            method: typeof restore,
            receiver: unknown,
            args: unknown[],
          ): unknown {
            if (generation.restored || ((owner.phase === 'completed') && (!restoring())))
              return undefined;
        /**
         A failed restoration remains eligible for subsequent cleanup.
         */
            const result: unknown = Reflect.apply(
              method,
              receiver,
              args,
            );
            generation.restored = true;
            return result;
          },
        },
      ),
    );
  }
  return guardSandboxCapability({
    target: value,
    owner,
    operation: 'ctx.sinon.mock()',
    invoke(invocation: SandboxInvocation,): unknown {
      if (invocation.operation === 'ctx.sinon.mock().expects') {
        if (generation.restored)
          throw new SandboxOwnershipError('Sinon mock.expects belongs to a restored controller. Create a new mock controller instead.',);
        /**
         Invalid property arguments remain Sinon's validation responsibility.
         */
        const key = sandboxPropertyKey(invocation.args[0],);
        if (((typeof key) === 'string') || (key !== SINON_VALIDATES_PROPERTY))
          requireUnownedProperty({
            target,
            key,
            operation: invocation.operation,
          },);
      }
      return invokeSandboxMethod(invocation,);
    },
  },);
}

/**
 Guards injected factories before application setters or partial failure can expose them.

 @param invocation - raw Sinon injection call
 
 @param owner - attempt supplying the factories
 
 @param invoke - operation policy shared with the main context sandbox
 
 @returns unchanged destination identity

 @example
 ```ts
 return injectOwnedFactories({ invocation, owner, invoke });
 ```
 */
export function injectOwnedFactories({
  invocation,
  owner,
  invoke,
}: {
  readonly invocation: SandboxInvocation;
  readonly owner: SandboxOwner;
  readonly invoke: (invocation: SandboxInvocation,) => unknown;
},): unknown {
  /**
   Invalid destinations remain subject to native Sinon validation.
   */
  const [target,] = invocation.args;
  if (!isSandboxTarget(target,))
    return invokeSandboxMethod(invocation,);
  /**
   Preserve application setter receivers while guarding each factory before assignment.
   */
  const destination = new Proxy(
    target,
    {
      set(
        object: object,
        property: PropertyKey,
        value: unknown,
      ): boolean {
      /**
       Clock objects already carry timer guards; only installed functions need factory wrapping.
       */
        const guarded = (typeof value) === 'function'
          ? guardSandboxCapability({
            target: value,
            owner,
            operation: `ctx.sinon.${String(property,)}`,
            invoke,
          },)
          : value;
        if (!Reflect.set(
          object,
          property,
          guarded,
          object,
        ))
          throw new SandboxOwnershipError(`ctx.sinon.inject cannot assign factory property "${String(property,)}" on its destination. Use a writable destination or call the context sandbox directly.`,);
        return true;
      },
    },
  );
  invokeSandboxMethod({
    ...invocation,
    args: [
      destination,
      ...invocation.args
        .slice(1,),
    ],
  },);
  return target;
}
