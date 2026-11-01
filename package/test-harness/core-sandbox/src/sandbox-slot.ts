/**
 Read-context selection for supported method properties. @module
 */
import { SandboxOwnershipError, } from './sandbox-error.ts';
import type { SandboxRuntime, } from './sandbox-owner.ts';
import { isSandboxTarget, } from './sandbox-value.ts';

import {
  methodRegistry,
  methodSlotConflict,
  methodSlotIntact,
  type MethodSlot,
} from './sandbox-registry.ts';

/**
 No contextual lease exists or the method requires ordinary Sinon semantics.
 */
export const NO_METHOD_SLOT: unique symbol = Symbol('no context-owned Sinon property slot for this object and key',);

/**
 Finds an active slot through an own property, prototype, or copied contextual accessor.

 @param target - object supplied to a Sinon operation
 
 @param key - exact property being replaced
 
 @returns active slot, or undefined when ordinary Sinon may handle this property
 
 @example
 ```ts
 const slot = findMethodSlot({ target, key: 'warn' });
 ```
 */
export function findMethodSlot({
  target,
  key,
}: {
  readonly target: object;
  readonly key: PropertyKey;
},): MethodSlot | typeof NO_METHOD_SLOT {
  /**
   Shared state recognizes getters even when an alias hides target identity.
   */
  const registry = methodRegistry();
  /**
   Proxy traps can synthesize cycles even though ordinary prototype chains cannot.
   */
  const visited = new Set<object>();
  /**
   Prototype chains are linear, so traverse with a cursor rather than recursion.
   */
  for (let cursor: unknown = target; isSandboxTarget(cursor); cursor = Object.getPrototypeOf(cursor)) {
    if (visited.has(cursor,))
      throw new SandboxOwnershipError('The supplied mock target has a cyclic prototype chain.',);
    visited.add(cursor,);
    /**
     Primary lookup also detects deletion or redefinition of an active slot.
     */
    const direct = registry.targets
      .get(cursor,)
      ?.get(key,);
    if (direct !== undefined)
      return direct;
    /**
     Inspect descriptors without executing arbitrary getters.
     */
    const descriptor = Object.getOwnPropertyDescriptor(
      cursor,
      key,
    );
    if (descriptor !== undefined) {
      /**
       Read the getter as an identity token, never as an unbound method to invoke.
       */
      const getter: unknown = Reflect.get(
        descriptor,
        'get'
      );
      return (typeof getter) === 'function' ? registry.getters
        .get(getter)
        ?? NO_METHOD_SLOT : NO_METHOD_SLOT;
    }
  }
  return NO_METHOD_SLOT;
}

/**
 Prepares, but does not install, a context-selecting descriptor.
 Unsupported targets retain ordinary Sinon behavior unless an active lease owns them.

 @param target - actual consumer object
 
 @param key - property passed to `stub` or `spy`
 
 @param runtime - current execution's context capability
 
 @returns prepared or existing slot for supported methods
 
 @example
 ```ts
 const slot = prepareMethodSlot({ target, key: 'warn', runtime });
 ```
 */
export function prepareMethodSlot({
  target,
  key,
  runtime,
}: {
  readonly target: object;
  readonly key: PropertyKey;
  readonly runtime: SandboxRuntime;
},): MethodSlot | typeof NO_METHOD_SLOT {
  /**
   Reuse a slot only through the exact object identity that originally installed it.
   */
  const existing = findMethodSlot({
    target,
    key,
  },);
  if ((typeof existing) !== 'symbol') {
    if (!runtime.contextual)
      throw new SandboxOwnershipError(`ctx.sinon.stub/spy cannot join context-owned property "${String(key,)}" without Node async context support. Finish its owning tests before using ordinary Sinon replacements.`,);
    if (runtime.isProxy(target,) || (existing.target !== target))
      throw new SandboxOwnershipError(`ctx.sinon.stub/spy cannot replace context-owned property "${String(key,)}" through an inherited, copied, or proxy alias. Use the original object, or finish its owning tests before replacing this alias.`,);
    if (!methodSlotIntact(existing,))
      throw methodSlotConflict({
        slot: existing,
        operation: 'ctx.sinon.stub/spy',
      },);
    return existing;
  }
  if ((!runtime.contextual) || runtime.isProxy(target,))
    return NO_METHOD_SLOT;
  /**
   Restrict the new contract to configurable own writable data-method properties.
   */
  const original = Object.getOwnPropertyDescriptor(
    target,
    key,
  );
  if ((original?.configurable !== true) || (original.writable !== true)
    || ((typeof original.value) !== 'function'))
    return NO_METHOD_SLOT;
  /**
   Values are selected from a private facade only while this exact owner is running.
   */
  const owners: MethodSlot['owners'] = new Map();
  return {
    target,
    key,
    original,
    owners,
    get(this: unknown,): unknown {
      /**
       Async descendants of completed attempts must never enter another attempt's fake.
       */
      const current = runtime.current();
      /**
       Missing registrations include suites, unrelated tests, and contextless consumers.
       */
      const replacement = ((typeof current) !== 'symbol') && (current.phase === 'running') ? owners.get(current,) : undefined;
      return replacement === undefined ? original.value : Reflect.get(
        replacement.facade,
        key,
        this,
      );
    },
    set(): never {
      throw new SandboxOwnershipError(
        `Assignment to context-owned property "${String(key,)}" is not supported while it has active mocks. `
          + 'Configure the returned fake or restore its owning sandbox before assigning the property.',
      );
    },
  };
}
