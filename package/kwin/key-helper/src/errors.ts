/**
 Custom error classes for the key-helper daemon, so callers can distinguish
 failure kinds by type rather than by string matching (PP4).
 
 @module
 */

/**
 Base class for every error the daemon throws, letting a single `instanceof`
 check catch all daemon-originated failures.
 
 @example
 ```ts
 try { ... } catch (e) { if (e instanceof KeyHelperError) { ... } }
 ```
 */
export class KeyHelperError extends Error {
  /**
   @param message - Human-readable description of what failed and why
   */
  constructor(message: string) {
    super(message);
    this.name = 'KeyHelperError';
  }
}

/**
 Thrown when a key token in a combo string has no evdev key code mapping, so
 the injection cannot be built and must not be sent half-formed.
 
 @example
 ```ts
 throw new UnknownKeyError('hyper');
 ```
 */
export class UnknownKeyError extends KeyHelperError {
  /**
   @param key - Offending token that was not found in the evdev key table
   */
  constructor(key: string) {
    super(`unknown key: ${key}`);
    this.name = 'UnknownKeyError';
  }
}

/**
 Thrown when a command the daemon ran did not exit successfully: it exited
 nonzero, a signal ended it, or it could not be started. The message names the
 command, how it ended, and whatever it wrote to stderr.
 
 @example
 ```ts
 throw new CommandFailedError("'ydotool key' exited with code 7");
 ```
 */
export class CommandFailedError extends KeyHelperError {
  /**
   @param message - The invocation, how it ended, and its stderr
   */
  constructor(message: string) {
    super(message);
    this.name = 'CommandFailedError';
  }
}

/**
 Thrown when no Neovim listen socket exists, so there is nothing to send input
 to. Callers treat this as "Neovim not running" rather than a hard failure.
 
 @example
 ```ts
 throw new NoNvimSocketError();
 ```
 */
export class NoNvimSocketError extends KeyHelperError {
  /**
   @param refusals - Why each socket that refused a connection refused it; empty when the directory held no socket at all
   */
  constructor({ refusals = [], }: { readonly refusals?: readonly string[]; } = {}) {
    super(refusals.length === 0
      ? 'no nvim socket found'
      : `no nvim socket accepted the input: ${refusals.join('; ')}`,);
    this.name = 'NoNvimSocketError';
  }
}

/**
 Thrown when a launch request carries neither a desktop file nor a resource
 class, so no instance can be started.
 
 @example
 ```ts
 throw new NoAppIdentityError();
 ```
 */
export class NoAppIdentityError extends KeyHelperError {
  /**
   Construct with a fixed, descriptive message.
   */
  constructor() {
    super('no app identity provided');
    this.name = 'NoAppIdentityError';
  }
}

/**
 Thrown when neither `$XDG_RUNTIME_DIR` nor the process UID is available, so
 there is no way to name the directory Neovim writes its listen sockets into.
 Scanning a guessed path would look in another user's directory, so discovery
 fails instead.
 
 @example
 ```ts
 throw new NoRuntimeDirectoryError();
 ```
 */
export class NoRuntimeDirectoryError extends KeyHelperError {
  /**
   Construct with a fixed, descriptive message.
   */
  constructor() {
    super('no runtime directory: neither XDG_RUNTIME_DIR nor a process UID is available');
    this.name = 'NoRuntimeDirectoryError';
  }
}

/**
 Thrown when an install step cannot proceed, with the caught value in the
 message so the operator sees the fault the step refused to paper over.
 
 @example
 ```ts
 throw new InstallError(`cannot inspect ${linkPath}: ${caughtValueText(error)}`);
 ```
 */
export class InstallError extends KeyHelperError {
  /**
   @param message - What the install was doing and the caught failure
   */
  constructor(message: string) {
    super(message);
    this.name = 'InstallError';
  }
}
