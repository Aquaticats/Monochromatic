/**
 Prevent ordinary target mutations from overwriting live contextual method accessors. @module
 */
import { SandboxOwnershipError, } from './sandbox-error.ts';
import { findMethodSlot, } from './sandbox-slot.ts';
import { isSandboxTarget, } from './sandbox-value.ts';

export {
  isSandboxTarget,
  sandboxPropertyKey,
  SINON_VALIDATES_PROPERTY,
} from './sandbox-value.ts';

/**
 Refuses noncontextual replacement of a property with an active contextual owner.

 @param target - object supplied by the test
 
 @param key - exact property the operation might redefine
 
 @param operation - user-facing Sinon call
 
 @throws SandboxOwnershipError before delegating any conflicting mutation
 
 @example
 ```ts
 requireUnownedProperty({ target, key: 'warn', operation: 'ctx.sinon.replace' });
 ```
 */
export function requireUnownedProperty({
  target,
  key,
  operation,
}: {
  readonly target: object;
  readonly key: PropertyKey;
  readonly operation: string;
},): void {
  /**
   Aliases and inherited accessors must not bypass the shared lease.
   */
  const slot = findMethodSlot({
    target,
    key,
  },);
  if ((typeof slot) !== 'symbol') {
    throw new SandboxOwnershipError(
      `${operation} cannot replace property "${String(key,)}" on the supplied object while it has context-owned mocks. `
        + 'Use ctx.sinon.stub(object, property) or ctx.sinon.spy(object, property) for supported methods, '
        + 'or finish the owning tests before using other replacement APIs.',
    );
  }
}

/**
 Preflights whole-object operations without invoking any method or property getter.

 @param target - input to whole-object stubbing, spying, or timer installation
 
 @param operation - operation that would bypass context selection
 
 @throws SandboxOwnershipError if any visited method has an active contextual lease
 
 @example
 ```ts
 requireUnownedObject({ target, operation: 'ctx.sinon.stub(object)' });
 ```
 */
export function requireUnownedObject({
  target,
  operation,
}: {
  readonly target: object;
  readonly operation: string;
},): void {
  /**
   Track visited objects because proxy traps can synthesize cyclic prototype chains.
   */
  const visited = new Set<object>();
  /**
   Stop at standard object behavior rather than treating it as a stubbing target.
   */
  for (let cursor: unknown = target; isSandboxTarget(cursor) && (cursor !== Object.prototype); cursor = Object.getPrototypeOf(cursor)) {
    if (visited.has(cursor,))
      throw new SandboxOwnershipError(`${operation} encountered a cyclic prototype chain.`,);
    visited.add(cursor,);
    for (const key of Reflect.ownKeys(cursor,))
      requireUnownedProperty({
        target: cursor,
        key,
        operation,
      },);
  }
}
