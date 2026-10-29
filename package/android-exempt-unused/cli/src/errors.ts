/**
 Custom error subclasses for programmatic discrimination via `instanceof`.
 
 The top-level handler in {@link ./index.ts} distinguishes these from unknown
 failures: known {@link AdbError}s become a one-line message plus a non-zero
 exit code, while {@link PromptCancelledError} is treated as a clean abort.
 
 @module
 */

import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

/**
 Base class for every adb-related failure raised by this CLI. Catch this to
 handle any adb problem uniformly; catch a subclass for a specific cause.
 
 @example
 ```ts
 try {
   await listDevices();
 } catch (error) {
   if (error instanceof AdbError) console.error(error.message,);
 }
 ```
 */
export class AdbError extends Error {
  /**
   Wrap `message` and the optional `cause`.
   
   @param message - Human-readable description of what adb interaction failed.
   
   @param options - Standard `ErrorOptions`; pass `{ cause }` to chain an
                    underlying error such as a `SubprocessError`.
   
   @mutates options - `super` may invoke a `cause` getter or proxy trap on supplied ErrorOptions
   */
  constructor(
    message: string,
    options?: ForeignBorrowed<Readonly<ErrorOptions>>,
  ) {
    super(
      message,
      options,
    );
    this.name = 'AdbError';
  }
}

/**
 Thrown when the `adb` executable cannot be found on `PATH` (spawn `ENOENT`).
 */
export class AdbNotFoundError extends AdbError {
  /**
   Wrap `message` and the optional `cause`.
   
   @param message - Human-readable hint pointing at platform-tools install.
   
   @param options - Standard `ErrorOptions`; pass `{ cause }` to chain the
                    spawn error.
   
   @mutates options - `super` may invoke a `cause` getter or proxy trap on supplied ErrorOptions
   */
  constructor(
    message: string,
    options?: ForeignBorrowed<Readonly<ErrorOptions>>,
  ) {
    super(
      message,
      options,
    );
    this.name = 'AdbNotFoundError';
  }
}

/**
 Thrown when no authorized device is connected (`adb devices` lists none in
 the {@link ./constants.ts CONNECTED_STATE} state).
 */
export class NoDevicesError extends AdbError {
  /**
   Wrap `message` and the optional `cause`.
   
   @param message - Human-readable description naming the missing device.
   
   @param options - Standard `ErrorOptions`; pass `{ cause }` to chain an
                    underlying error.
   
   @mutates options - `super` may invoke a `cause` getter or proxy trap on supplied ErrorOptions
   */
  constructor(
    message: string,
    options?: ForeignBorrowed<Readonly<ErrorOptions>>,
  ) {
    super(
      message,
      options,
    );
    this.name = 'NoDevicesError';
  }
}

/**
 Thrown when an adb invocation exits non-zero or otherwise fails to produce
 usable output. Carries the failing command context in its message.
 */
export class AdbCommandError extends AdbError {
  /**
   Wrap `message` and the optional `cause`.
   
   @param message - Human-readable description naming command plus stderr.
   
   @param options - Standard `ErrorOptions`; pass `{ cause }` to chain the
                    `SubprocessError`.
   
   @mutates options - `super` may invoke a `cause` getter or proxy trap on supplied ErrorOptions
   */
  constructor(
    message: string,
    options?: ForeignBorrowed<Readonly<ErrorOptions>>,
  ) {
    super(
      message,
      options,
    );
    this.name = 'AdbCommandError';
  }
}

/**
 Thrown by the clack prompt wrappers when the user cancels (Esc / Ctrl-C).
 Not an {@link AdbError}: the top-level handler treats it as a clean exit.
 */
export class PromptCancelledError extends Error {
  /**
   Construct with a fixed message; cancellation carries no extra detail.
   */
  constructor() {
    super('Prompt cancelled by user.',);
    this.name = 'PromptCancelledError';
  }
}
