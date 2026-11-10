/**
 Custom error subclasses for programmatic discrimination via `instanceof`.
 
 The top-level handler in {@link ./index.ts} treats {@link PromptCancelledError}
 as a clean abort and lets every other error surface, so the throw, not an
 exit-code assignment, decides the status.
 
 @module
 */

import { NamedError, } from '@monochromatic-dev/named-error-core/ts';

/* oxlint-disable unicorn/custom-error-definition -- Every class here takes its name from `NamedError`, which derives it from the concrete class, so no literal is left to check; `doc/troubleshooting/oxlint-custom-error-name.md` records why the rule cannot be satisfied here. */

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
export class AdbError extends NamedError {}

/**
 Thrown when the `adb` executable cannot be found on `PATH` (spawn `ENOENT`).
 Its message is a hint pointing at the platform-tools install.
 */
export class AdbNotFoundError extends AdbError {}

/**
 Thrown when no authorized device is connected (`adb devices` lists none in
 the {@link ./constants.ts CONNECTED_STATE} state). Its message names the
 missing device.
 */
export class NoDevicesError extends AdbError {}

/**
 Thrown when an adb invocation exits non-zero or otherwise fails to produce
 usable output. Its message names the command and what stderr said.
 */
export class AdbCommandError extends AdbError {}

/**
 Thrown when at least one exemption write failed, so the device does not match
 the selection the user confirmed. Its message has one line per application
 whose write failed, naming the application, the mode, and what adb answered.
 
 @example
 ```ts
 throw new ExemptApplyError('com.example (ignore): adb shell failed',);
 ```
 */
export class ExemptApplyError extends NamedError {}

/**
 Thrown by the clack prompt wrappers when the user cancels (Esc / Ctrl-C).
 Not an {@link AdbError}: the top-level handler treats it as a clean exit.
 */
export class PromptCancelledError extends NamedError {
  /**
   Construct with a fixed message; cancellation carries no extra detail.
   */
  constructor() {
    super('Prompt cancelled by user.',);
  }
}

/* oxlint-enable unicorn/custom-error-definition */
