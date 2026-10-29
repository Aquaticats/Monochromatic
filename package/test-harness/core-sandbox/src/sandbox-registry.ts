/**
 Shared property leases across source imports and bundled harness copies. @module
 */
import { SandboxOwnershipError, } from './sandbox-error.ts';
import type { SandboxOwner, } from './sandbox-owner.ts';

/**
 One owner's private Sinon target and exact fake generation.
 */
export type MethodReplacement = {
  /**
   Sinon mutates this object, never a sibling's target.
   */
  readonly facade: object;
  /**
   Generation identity distinguishes restubbing from an older fake's restoration.
   */
  readonly fake: object;
};

/**
 Original descriptor and the accessor temporarily selecting owner-private methods.
 */
export type MethodSlot = {
  /**
   Actual consumer object, excluding proxy-mediated targets.
   */
  readonly target: object;
  /**
   Exact property identity, including symbols.
   */
  readonly key: PropertyKey;
  /**
   Snapshot restored only if our installed descriptor remains intact.
   */
  readonly original: PropertyDescriptor;
  /**
   Context-selecting property reader.
   */
  readonly get: () => unknown;
  /**
   Assignment is rejected rather than accidentally modifying another owner.
   */
  readonly set: (value: unknown,) => void;
  /**
   Running owners and their independent replacements.
   */
  readonly owners: Map<SandboxOwner, MethodReplacement>;
};

/**
 Versioned realm protocol; the registry is not coupled to a bundled module instance.
 */
type MethodRegistry = {
  /**
   Schema discriminator prevents silently sharing incompatible state.
   */
  readonly version: 1;
  /**
   Primary lookup by actual object identity and property key.
   */
  readonly targets: WeakMap<object, Map<PropertyKey, MethodSlot>>;
  /**
   Recognizes contextual accessors reached through aliases or copied descriptors.
   */
  readonly getters: WeakMap<object, MethodSlot>;
};

/**
 Namespaced, versioned protocol distinct from rejection observation state.
 */
const REGISTRY_KEY: unique symbol = Symbol.for('@monochromatic-dev/test-harness-core/method-ownership/v1',);

/**
 Retrieves or initializes the registry without touching any test target.

 @returns realm-shared method ownership state
 
 @throws SandboxOwnershipError if the protocol was overwritten incompatibly
 
 @example
 ```ts
 const registry = methodRegistry();
 ```
 */
export function methodRegistry(): MethodRegistry {
  /**
   Typed access is confined to this owned versioned protocol.
   */
  const global = globalThis as typeof globalThis & { [REGISTRY_KEY]?: MethodRegistry; };
  /**
   Existing copies must agree on both schema and capability shape.
   */
  const existing = global[REGISTRY_KEY];
  if (existing !== undefined) {
    if ((existing.version !== 1) || (!(existing.targets instanceof WeakMap))
      || (!(existing.getters instanceof WeakMap)))
      throw new SandboxOwnershipError('The module-test method-ownership registry has an incompatible shape.',);
    return existing;
  }
  /**
   Allocate once per realm, retaining targets only while consumers do.
   */
  const registry: MethodRegistry = {
    version: 1,
    targets: new WeakMap(),
    getters: new WeakMap(),
  };
  global[REGISTRY_KEY] = registry;
  return registry;
}

/**
 Detects foreign descriptor changes without invoking a target getter.

 @param slot - lease whose installed accessor must still own the property
 
 @returns whether restoring this descriptor would avoid overwriting foreign state
 
 @example
 ```ts
 if (!methodSlotIntact(slot)) throw conflict;
 ```
 */
export function methodSlotIntact(slot: MethodSlot,): boolean {
  /**
   Descriptor inspection never calls the context-selecting getter.
   */
  const descriptor = Object.getOwnPropertyDescriptor(
    slot.target,
    slot.key,
  );
  return (descriptor?.get === slot.get) && (descriptor?.set === slot.set)
    && (descriptor?.configurable === true)
    && (descriptor?.enumerable
      === slot.original
        .enumerable);
}

/**
 Names a descriptor conflict without inventing another test's identity.

 @param slot - affected input property
 
 @param operation - call trying to acquire or release ownership
 
 @returns actionable harness error preserving the external descriptor
 
 @example
 ```ts
 throw methodSlotConflict({ slot, operation: 'restore' });
 ```
 */
export function methodSlotConflict({
  slot,
  operation,
}: {
  readonly slot: MethodSlot;
  readonly operation: string;
},): SandboxOwnershipError {
  return new SandboxOwnershipError(
    `${operation}: property "${String(slot.key,)}" on the supplied object changed outside its context-owned mock. `
      + 'Its current descriptor was preserved. Do not delete, redefine, or mix direct Sinon replacements '
      + 'with ctx.sinon replacements on this property while tests own it.',
  );
}
