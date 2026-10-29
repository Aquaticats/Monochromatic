/**
 Guard callable sandbox capabilities at invocation, including retained factories. @module
 */
import {
  requireRunningOwner,
  type SandboxOwner,
} from './sandbox-owner.ts';

/**
 Dynamic Sinon entry point, kept behind the typed public sandbox.
 */
export type SandboxInvocation = {
  /**
   Callable obtained from Sinon rather than a guessed overload.
   */
  readonly method: object;
  /**
   Original receiver, preserving Sinon namespace and controller behavior.
   */
  readonly receiver: unknown;
  /**
   Actual call arguments; the outer Sinon type checks consumer calls.
   */
  readonly args: readonly unknown[];
  /**
   Qualified capability name used in diagnostics.
   */
  readonly operation: string;
};

/**
 Forwards a validated callable without interpreting Sinon's overloads.

 @param invocation - captured call boundary
 
 @returns unchanged Sinon return value
 
 @throws TypeError if an internal caller supplied a noncallable capability
 
 @example
 ```ts
 const result = invokeSandboxMethod(invocation);
 ```
 */
export function invokeSandboxMethod(invocation: SandboxInvocation,): unknown {
  if ((typeof invocation.method) !== 'function')
    throw new TypeError(`Noncallable sandbox capability: ${invocation.operation}`,);
  return Reflect.apply(
    invocation.method,
    invocation.receiver,
    invocation.args,
  );
}

/**
 Guards a sandbox or controller, preserving its public type and function metadata.
 Function namespaces are visited lazily, so `fake.returns` cannot bypass completion.

 @param target - owned sandbox, factory, or deferred mock controller
 
 @param owner - attempt that supplied this capability
 
 @param operation - qualified name for errors and operation dispatch
 
 @param invoke - boundary-specific forwarding policy
 
 @returns typed proxy with cached method wrappers
 
 @example
 ```ts
 const guarded = guardSandboxCapability({ target: sandbox, owner, operation: 'ctx.sinon', invoke });
 ```
 */
export function guardSandboxCapability<Target extends object,>({
  target,
  owner,
  operation,
  invoke,
}: {
  readonly target: Target;
  readonly owner: SandboxOwner;
  readonly operation: string;
  readonly invoke: (invocation: SandboxInvocation,) => unknown;
},): Target {
  /**
   Cache property wrappers without retaining unrelated targets process-wide.
   */
  const methods = new Map<PropertyKey, {
    readonly original: object;
    readonly guarded: object
  }>();
  return new Proxy(
    target,
    {
      apply(
        method: Target,
        receiver: unknown,
        args: unknown[],
      ): unknown {
        requireRunningOwner({
          owner,
          operation,
        },);
        return invoke({
          method,
          receiver,
          args,
          operation,
        },);
      },
      get(
        object: Target,
        property: string | symbol,
        receiver: unknown,
      ): unknown {
      /**
       Function metadata and noncallable values pass through unchanged.
       */
        const value: unknown = Reflect.get(
          object,
          property,
          receiver,
        );
        if ((typeof value) !== 'function')
          return value;
      /**
       Stable wrappers preserve equality when callers repeatedly read a factory.
       */
        const existing = methods.get(property,);
        if (existing?.original === value)
          return existing.guarded;
      /**
       Object methods receive their owning object. Function call/apply/bind retain the guarded callable receiver.
       */
        const guarded = guardSandboxCapability({
          target: value,
          owner,
          operation: `${operation}.${String(property,)}`,
          invoke(invocation: SandboxInvocation,): unknown {
            return invoke({
              ...invocation,
              receiver: (invocation.receiver === receiver) && ((typeof object) !== 'function') ? object : invocation.receiver,
            },);
          },
        },);
        methods.set(
          property,
          {
            original: value,
            guarded,
          },
        );
        return guarded;
      },
    },
  );
}
