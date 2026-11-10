/**
 Error subclass every named error class in this repository builds on.

 @module
 */

/**
 Placeholder for the name derivation this package exists to own.

 @remarks
 This revision only forwards the message and options,
 so a subclass still reports the base class name.
 */
export abstract class NamedError extends Error {
  public constructor(message: string, options?: ErrorOptions) {
    super(message, options);
  }
}
