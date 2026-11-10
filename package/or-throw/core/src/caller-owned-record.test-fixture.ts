/**
 Fixture: an instance of a caller-owned type.
 
 `class` is banned outside the `Error`/`Element` suffix allow-list, and an
 object literal carries `Object.prototype`, so the instance is built by giving
 an object literal a prototype of its own. That is the runtime shape of a class
 instance: a prototype that is neither `Object.prototype` nor `null`, plus own
 enumerable fields.
 
 @module
 */

/**
 Prototype of the caller-owned type: one inherited field, no own fields.
 */
const CALLER_OWNED_PROTOTYPE = Object.freeze({ kind: 'caller-owned record', },);

/**
 Builds an instance of a caller-owned type.
 
 @returns Object whose prototype is the caller-owned prototype and whose own
          enumerable fields are data rather than container elements
 
 @example
 ```ts
 createCallerOwnedRecord(); // { first: 1 }, prototype { kind: 'caller-owned record' }
 ```
 */
export function createCallerOwnedRecord(): object {
  /**
   Own field of the caller-owned type: a domain value, not a container element.
   */
  const record: object = { first: 1, };
  Object.setPrototypeOf(
    record,
    CALLER_OWNED_PROTOTYPE,
  );
  return record;
}
