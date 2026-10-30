/**
 Scoped removal of a snapshot's build context.
 */
import { rm } from 'node:fs/promises';

/**
 Describe the removal of a build context when a scope ends.

 @param directory - the build context

 @returns an object whose disposal deletes the directory

 @example
 ```ts
 await using cleanup = removalOf(fixture.context);
 ```
 */
export function removalOf(directory: string): AsyncDisposable {
  return {
    async [Symbol.asyncDispose](): Promise<void> {
      await rm(
        directory,
        {
          recursive: true,
          force: true,
        },
      );
    },
  };
}
