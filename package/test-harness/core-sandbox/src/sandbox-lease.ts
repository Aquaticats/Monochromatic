/**
 Installation and generation-aware restoration of one contextual method fake. @module
 */
import {
  invokeSandboxMethod,
  type SandboxInvocation,
} from './sandbox-guard.ts';
import { SandboxOwnershipError, } from './sandbox-error.ts';
import type { SandboxOwner, } from './sandbox-owner.ts';
import {
  methodRegistry,
  methodSlotConflict,
  methodSlotIntact,
  type MethodReplacement,
  type MethodSlot,
} from './sandbox-registry.ts';

/**
 Installs a new slot only after Sinon has successfully created its private fake.

 @param slot - original target snapshot and prepared accessor
 
 @throws SandboxOwnershipError if target state changed during fake creation
 
 @example
 ```ts
 installMethodSlot(slot);
 ```
 */
function installMethodSlot(slot: MethodSlot,): void {
  /**
   Registry lookup distinguishes first installation from joining another owner.
   */
  const registry = methodRegistry();
  /**
   Per-object properties share one weakly retained map.
   */
  const properties = registry.targets
    .get(slot.target,)
    ?? new Map<PropertyKey, MethodSlot>();
  if (properties.get(slot.key,) === slot) {
    if (!methodSlotIntact(slot,))
      throw methodSlotConflict({
        slot,
        operation: 'join mock',
      },);
    return;
  }
  /**
   Guard side effects from custom function metadata inspected by Sinon.
   */
  const current = Object.getOwnPropertyDescriptor(
    slot.target,
    slot.key,
  );
  if ((current?.value
    !== slot.original
      .value) || (current?.writable
        !== slot.original
          .writable)
    || (current?.configurable
      !== slot.original
        .configurable)
    || (current?.enumerable
      !== slot.original
        .enumerable))
    throw methodSlotConflict({
      slot,
      operation: 'install mock',
    },);
  Object.defineProperty(
    slot.target,
    slot.key,
    {
      get: slot.get,
      set: slot.set,
      configurable: true,
      enumerable: slot.original
        .enumerable
        ?? false,
    },
  );
  properties.set(
    slot.key,
    slot,
  );
  registry.targets
    .set(
    slot.target,
    properties,
    );
  registry.getters
    .set(
    slot.get,
    slot,
    );
}

/**
 Releases one exact generation without overwriting a descriptor changed outside this lease.

 @param slot - shared property lease
 
 @param owner - identity that installed this replacement
 
 @param replacement - exact generation, not merely its owner's identity
 
 @throws SandboxOwnershipError on a foreign descriptor change, after registry cleanup
 
 @example
 ```ts
 releaseMethodReplacement({ slot, owner, replacement });
 ```
 */
function releaseMethodReplacement({
  slot,
  owner,
  replacement,
}: {
  readonly slot: MethodSlot;
  readonly owner: SandboxOwner;
  readonly replacement: MethodReplacement;
},): void {
  if (slot.owners
    .get(owner,)
    !== replacement)
    return;
  slot.owners
    .delete(owner,);
  /**
   Inspect before deleting the registry so an external descriptor remains untouched.
   */
  const intact = methodSlotIntact(slot,);
  if (slot.owners
    .size
    === 0) {
    /**
     Internal ownership must be released even if descriptor restoration is impossible.
     */
    const registry = methodRegistry();
    /**
     The primary map is present for every installed slot.
     */
    const properties = registry.targets
      .get(slot.target,);
    properties?.delete(slot.key,);
    if (properties?.size === 0)
      registry.targets
        .delete(slot.target,);
    registry.getters
      .delete(slot.get,);
    if (intact)
      Object.defineProperty(
        slot.target,
        slot.key,
        slot.original,
      );
  }
  if (!intact)
    throw methodSlotConflict({
      slot,
      operation: 'restore mock',
    },);
}

/**
 Creates an ordinary Sinon fake on a private facade and joins the shared property lease.
 Independent lease callbacks ensure even a broken fake's restore cannot strand registry ownership.

 @param slot - supported property prepared before invoking Sinon
 
 @param owner - current context's capability owner
 
 @param invocation - original factory, receiver, and overload arguments
 
 @param leases - runner-owned cleanup callbacks independent of Sinon collection internals
 
 @returns private facade and original Sinon fake, preserving identity and its behavior API
 
 @throws SandboxOwnershipError if Sinon returns a nonrestorable method fake
 
 @example
 ```ts
 const fake = createMethodReplacement({ slot, owner, invocation, leases });
 ```
 */
export function createMethodReplacement({
  slot,
  owner,
  invocation,
  leases,
}: {
  readonly slot: MethodSlot;
  readonly owner: SandboxOwner;
  readonly invocation: SandboxInvocation;
  readonly leases: Set<() => void>;
},): MethodReplacement {
  /**
   Reusing an active facade preserves Sinon's same-owner double-wrap rejection.
   */
  const facade: object = slot.owners
    .get(owner,)
    ?.facade
    ?? Object.defineProperty(
      {},
      slot.key,
      slot.original,
    );
  /**
   Sinon remains responsible for matching, history, call-through, and fake construction.
   */
  const fake = invokeSandboxMethod({
    ...invocation,
    args: [
      facade,
      slot.key,
    ],
  },);
  if ((typeof fake) !== 'function')
    throw new SandboxOwnershipError(`${invocation.operation} did not return a callable method fake.`,);
  /**
   Capture the private-facade restoration before installing the ownership hook.
   */
  const originalRestore: unknown = Reflect.get(
    fake,
    'restore',
  );
  if ((typeof originalRestore) !== 'function')
    throw new SandboxOwnershipError(`${invocation.operation} did not return a restorable method fake.`,);
  /**
   Object identity is a generation token for repeated restoration and restubbing.
   */
  const replacement: MethodReplacement = {
    facade,
    fake,
  };
  /**
   Release registry state separately from Sinon's private-object restoration.
   */
  function release(): void {
    leases.delete(release,);
    releaseMethodReplacement({
      slot,
      owner,
      replacement,
    },);
  }
  Reflect.set(
    fake,
    'restore',
    new Proxy(
      originalRestore,
      {
        apply(
          method: typeof originalRestore,
          receiver: unknown,
          args: unknown[],
        ): unknown {
          release();
          return Reflect.apply(
            method,
            receiver,
            args,
          );
        },
      }
    ),
  );
  installMethodSlot(slot,);
  slot.owners
    .set(
    owner,
    replacement,
    );
  leases.add(release,);
  return replacement;
}
