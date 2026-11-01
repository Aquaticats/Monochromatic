/**
 Deferred target-changing methods on whole-object fakes and timer controllers. @module
 */
import { guardFakeMutators, } from './sandbox-fake.ts';
import { collectionFakes, } from './sandbox-member.ts';
import { SandboxOwnershipError, } from './sandbox-error.ts';
import {
  requireRunningOwner,
  type SandboxOwner,
} from './sandbox-owner.ts';
import {
  isSandboxTarget,
  sandboxPropertyKey,
  SINON_VALIDATES_PROPERTY,
} from './sandbox-value.ts';

/**
 Guards fakes returned as members without replacing the installed member's identity.

 @param value - whole-object stub, spy, or stub-instance result
 
 @param owner - owning attempt

 @param previous - descriptors preceding a whole-object operation
 
 @param restoring - internal raw-restoration authority
 
 @example
 ```ts
 guardCollectionFakes({ value, owner, restoring });
 ```
 */
export function guardCollectionFakes({
  value,
  owner,
  restoring,
  previous = {},
}: {
  readonly value: unknown;
  readonly owner: SandboxOwner;
  readonly restoring: () => boolean;
  readonly previous?: PropertyDescriptorMap;
},): void {
  for (const member of collectionFakes({
    value,
    previous,
  },)) {
    /**
     Sinon records target metadata on descriptor-changing stubs, but not on detached spies.
     */
    const target: unknown = Reflect.get(
      member,
      'rootObj',
    );
    /**
     Only primitive keys can be preflighted without coercion.
     */
    const key = sandboxPropertyKey(Reflect.get(
      member,
      'propName',
    ),);
    guardFakeMutators({
      value: member,
      owner,
      restoring,
      ...isSandboxTarget(target,) ? { target, } : {},
      ...((typeof key) === 'symbol') && (key === SINON_VALIDATES_PROPERTY) ? {} : { key, },
    },);
  }
}

/**
 Prevents retained fake-timer controllers from restoring over a newer clock or restarting host tick automation.
 Local clock/history operations keep their ordinary Sinon behavior.

 @param value - clock returned by useFakeTimers
 
 @param owner - owning attempt
 
 @param restoring - runner-owned cleanup phase
 
 @example
 ```ts
 guardTimerController({ value, owner, restoring });
 ```
 */
export function guardTimerController({
  value,
  owner,
  restoring,
}: {
  readonly value: unknown;
  readonly owner: SandboxOwner;
  readonly restoring: () => boolean;
},): void {
  if (!isSandboxTarget(value,))
    return;
  /**
   Preserve aliases such as clock.restore === clock.uninstall.
   */
  const wrappers = new WeakMap<object, object>();
  /**
   A manually uninstalled clock cannot restore over a subsequent installation.
   */
  const generation = { restored: false, };
  for (const property of [
    'restore',
    'uninstall',
    'setTickMode',
  ] as const) {
    /**
     These controller methods can affect state outside the clock's local history.
     */
    const method: unknown = Reflect.get(
      value,
      property,
    );
    if ((typeof method) !== 'function')
      continue;
    /**
     Reuse a wrapper when Sinon gives restoration methods identical function identities.
     */
    const guarded = wrappers.get(method,) ?? new Proxy(
      method,
      {
        apply(
          original: typeof method,
          receiver: unknown,
          args: unknown[],
        ): unknown {
          if (property === 'setTickMode') {
          /**
           Sinon uninstall stops automated ticking through this public method during cleanup.
           */
            const [mode,] = args;
          /**
           Cleanup may stop ticking, but must not start another host automation loop.
           */
            const stopping = restoring() && isSandboxTarget(mode,)
              && (Reflect.get(
                mode,
                'mode',
              ) === 'manual');
            if (!stopping) {
              requireRunningOwner({
                owner,
                operation: 'Sinon clock.setTickMode',
              },);
              if (generation.restored)
                throw new SandboxOwnershipError('Sinon clock.setTickMode belongs to an uninstalled clock. Install a new clock instead.',);
            }
          }
          else if (generation.restored || ((owner.phase === 'completed') && (!restoring())))
            return undefined;
        /**
         Preserve retry eligibility when underlying restoration throws.
         */
          const result: unknown = Reflect.apply(
            original,
            receiver,
            args,
          );
          if (property !== 'setTickMode')
            generation.restored = true;
          return result;
        },
      }
    );
    wrappers.set(
      method,
      guarded,
    );
    Reflect.set(
      value,
      property,
      guarded,
    );
  }
}
