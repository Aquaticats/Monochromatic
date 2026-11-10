/**
 The base class for errors whose name is the class that built them.

 @module
 */

/**
 Error subclass whose `name` follows the concrete class that constructed it.

 @remarks
 A subclass whose message is all it carries declares no constructor at all,
 so the name string and the class identifier cannot drift apart.
 `new.target` names the leaf class even through further subclassing.
 Published artifacts keep the name because every rolldown config in
 `package/config/rolldown` sets `mangle: false`, documented there as
 "Mangle breaks func.name and makes output difficult for users to audit".

 @example
 ```ts
 class TabbyMissingError extends NamedError {}

 new TabbyMissingError('no tabby').name; // => 'TabbyMissingError'
 ```
 */
export abstract class NamedError extends Error {
  /**
   @param message - what failed, in the words a reader needs to act on it

   @param options - native error options, forwarded unchanged so a chained `cause` survives
   */
  public constructor(message: string, options?: ErrorOptions) {
    super(message, options);
    this.name = new.target.name;
  }
}
