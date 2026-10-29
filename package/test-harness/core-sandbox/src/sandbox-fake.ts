/**
 Deferred descriptor capabilities share their root fake's restoration generation. @module
 */
import { SandboxOwnershipError, } from './sandbox-error.ts';
import {
  requireRunningOwner,
  type SandboxOwner,
} from './sandbox-owner.ts';
import {
  isSandboxTarget,
  requireUnownedProperty,
} from './sandbox-target.ts';

/**
 Guards descriptor-changing methods without replacing fake or behavior-object identity.
 Local behavior/history configuration remains available after completion.

 @param value - original factory result
 
 @param owner - attempt that created the result
 
 @param restoring - runner-owned restoration phase
 
 @param target - real target or private contextual facade
 
 @param key - property changed by deferred descriptor configuration
 
 @param contextualMethod - whether setter conversion would contradict the data-method lease

 @example
 ```ts
 guardFakeMutators({ value, owner, restoring, target, key });
 ```
 */
export function guardFakeMutators({
  value,
  owner,
  restoring,
  target,
  key,
  contextualMethod = false,
}: {
  readonly value: unknown;
  readonly owner: SandboxOwner;
  readonly restoring: () => boolean;
  readonly target?: object;
  readonly key?: PropertyKey;
  readonly contextualMethod?: boolean;
},): void {
  /**
   Successful root restoration retires every associated descriptor capability.
   */
  const generation = { restored: false, };
  /**
   Repeated call selectors can return an already guarded behavior object.
   */
  const seen = new WeakSet<object>();

  /**
   Installs guards lazily when Sinon exposes a call-sequence behavior object.

   @param current - root fake or behavior object sharing its descriptor generation
   
   @example
   ```ts
   guardResult(value);
   ```
   */
  function guardResult(current: unknown,): void {
    if ((!isSandboxTarget(current,)) || seen.has(current,))
      return;
    seen.add(current,);
    for (const operation of [
      'get',
      'set',
      'value',
      'restore',
    ] as const) {
      /**
       Inspect only Sinon descriptor capabilities, not arbitrary application properties.
       */
      const method: unknown = Reflect.get(
        current,
        operation,
      );
      if ((typeof method) !== 'function')
        continue;
      Reflect.set(
        current,
        operation,
        new Proxy(
          method,
          {
            apply(
              original: typeof method,
              receiver: unknown,
              args: unknown[],
            ): unknown {
              if (operation === 'restore') {
                if (generation.restored || ((owner.phase === 'completed') && (!restoring())))
                  return undefined;
              }
              else {
                requireRunningOwner({
                  owner,
                  operation: `Sinon fake.${operation}`,
                },);
                if (contextualMethod && (operation === 'set'))
                  throw new SandboxOwnershipError(`Sinon fake.set cannot convert context-owned data-method property "${String(key,)}" to a setter. Stub an existing accessor property, or use callsFake/value for method behavior.`,);
                if (generation.restored)
                  throw new SandboxOwnershipError(`Sinon fake.${operation} belongs to a restored replacement. Create a new fake instead.`,);
                if ((target !== undefined) && (key !== undefined))
                  requireUnownedProperty({
                    target,
                    key,
                    operation: `Sinon fake.${operation}`,
                  },);
              }
          /**
           A failed restoration remains retryable and reportable.
           */
              const result: unknown = Reflect.apply(
                original,
                receiver,
                args,
              );
              if (operation === 'restore')
                generation.restored = true;
              return result;
            },
          },
        ),
      );
    }
    for (const selector of [
      'onCall',
      'onFirstCall',
      'onSecondCall',
      'onThirdCall',
    ] as const) {
      /**
       These public selectors expose the root stub through Sinon's behavior object.
       */
      const method: unknown = Reflect.get(
        current,
        selector,
      );
      if ((typeof method) !== 'function')
        continue;
      Reflect.set(
        current,
        selector,
        new Proxy(
          method,
          {
            apply(
              original: typeof method,
              receiver: unknown,
              args: unknown[],
            ): unknown {
          /**
           Configure local behavior freely, but preserve root descriptor authority on the result.
           */
              const result: unknown = Reflect.apply(
                original,
                receiver,
                args,
              );
              guardResult(result,);
              return result;
            },
          },
        ),
      );
    }
  }
  guardResult(value,);
}
