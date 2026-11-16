/**
 The failure that stops a run before Pi starts: a place that holds a value of this machine's secret environment is there and
 cannot be read, so this code does not know which values to mask.

 Every source that is read raises it, and the driver starts no Pi when one does. A place that is not there is not this failure: a
 checkout with no store, a home with no shell start-up file, and a machine with no age identity are places that declare nothing,
 and the run goes on with what it has.

 @module
 */

/**
 A source that holds values and cannot be read.
 */
export class SecretEnvironmentUnreadableError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'SecretEnvironmentUnreadableError';

  /**
   What could not be read, named for a reader: the store, Pi's file, a shell start-up file, or the age identity.
   */
  readonly source: string;

  /**
   Builds the failure.

   @param source - what could not be read, named for a reader

   @param cause - why it could not be read, in words that never hold a value

   @example
   ```ts
   throw new SecretEnvironmentUnreadableError({ source: 'auth.json', cause: 'it is not JSON', },);
   ```
   */
  constructor({
    source,
    cause,
  }: {
    readonly source: string;
    readonly cause: string
  },) {
    super(
      `${source} holds values of this machine's secret environment and cannot be read, so no value of it could be masked before `
        + `it reached the model, and no run was started: ${cause}`,
    );
    this.source = source;
  }
}
