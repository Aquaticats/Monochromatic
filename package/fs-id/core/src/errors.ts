/**
 Filesystem identity resolution errors.
 
 @module
 */

import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

/**
 Readonly cause options accepted by custom errors.
 
 @example
 ```ts
 const options: FsIdErrorOptions = { cause: new Error('command'), };
 ```
 */
type FsIdErrorOptions = {
  readonly cause?: unknown;
};

/**
 Error raised when supported-platform identity mechanisms all fail.
 
 @example
 ```ts
 throw new FsIdResolutionError('unable to resolve identity');
 ```
 */
export class FsIdResolutionError extends Error {
  /**
   Creates a filesystem identity resolution error.
   
   @param message - Why no usable identity could be produced
   
   @param options - Optional underlying failure
   
   @mutates options - `super` may invoke a `cause` getter or proxy trap on supplied options
   
   @example
   ```ts
   new FsIdResolutionError('failed', { cause: new Error('command'), });
   ```
   */
  public constructor(
    message: string,
    options?: ForeignBorrowed<FsIdErrorOptions>,
  ) {
    super(
      message,
      options,
    );
    this.name = 'FsIdResolutionError';
  }
}

/**
 Error raised when host platform has no filesystem identity strategy.
 
 @example
 ```ts
 throw new UnsupportedFsIdPlatformError('freebsd');
 ```
 */
export class UnsupportedFsIdPlatformError extends Error {
  /**
   Creates an unsupported-platform error.
   
   @param platform - Unsupported Node platform identifier
   
   @example
   ```ts
   new UnsupportedFsIdPlatformError('aix');
   ```
   */
  public constructor(platform: string,) {
    super(`unsupported platform for filesystem ID resolution: ${platform}`,);
    this.name = 'UnsupportedFsIdPlatformError';
  }
}
