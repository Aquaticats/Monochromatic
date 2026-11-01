/**
 Member attachment pinning upstream `quick-lru`'s class-prototype
 descriptor flags.
 
 Upstream carries every member on its class prototype, where methods are
 writable, non-enumerable, configurable data properties and `size`-family
 members are non-enumerable, configurable accessors. This module copies a
 member table onto the factory's shell with exactly those flags, so the
 fuzz sidecar's surface check can pin the fork against upstream.
 
 @module
 */

//region Constants

/**
 `Node.js` custom inspection symbol upstream `quick-lru` implements as
 `[Symbol.for('nodejs.util.inspect.custom')]`.
 */
export const NODE_INSPECT_SYMBOL: symbol = Symbol.for('nodejs.util.inspect.custom',);

//endregion Constants

//region Attachment

/**
 Copies every member of one object onto another with upstream's
 class-prototype descriptor flags.
 
 Only `enumerable` changes: a member table written as an object literal
 already carries upstream's writable-or-accessor shape and configurable
 flag, and upstream members are exactly that with `enumerable: false`.
 
 @param options - Shell to attach onto and member table to copy.
 
 @throws Error when a listed member suddenly has no own descriptor, an
 unreachable state for an object literal.
 
 @example
 ```ts
 attachMembers({
   self,
   members: {
     get size(): number {
       return 0;
     },
   },
 },);
 ```
 */
export function attachMembers(options: {
  /**
   Shell object gaining every member.
   */
  readonly self: object;
  /**
   Member table whose own keys and descriptors are copied over.
   */
  readonly members: object;
},): void {
  for (const name of Reflect.ownKeys(options.members,)) {
    /**
     Own descriptor of this table member, carrying upstream's shape.
     */
    const descriptor = Object.getOwnPropertyDescriptor(
      options.members,
      name,
    );
    // mutation-test-disable-next-line conditional -- unreachable defensive guard: Reflect.ownKeys only yields own keys, so every listed member always has a descriptor; the throw keeps the impossible case loud instead of silently skipping it
    if (descriptor === undefined)
      // mutation-test-disable-next-line string -- same unreachable defensive guard: this message text renders only in a state Reflect.ownKeys cannot produce
      throw new Error(`member descriptor missing for ${String(name,)}`);

    Object.defineProperty(
      options.self,
      name,
      {
        ...descriptor,
        enumerable: false,
      },
    );
  }
}

//endregion Attachment
