//region Missing path error
// The one filesystem failure the package reads as an answer rather than a
// fault: nothing stands at the path (`ENOENT`). A first run has no ledger, no
// cache marker, no page; a program not installed cannot be spawned. Every other
// failure (a permission refused, a file where a directory belongs, a device
// error) means the question went unanswered, and reading it as absence is how
// a guard talks itself into deleting or overwriting data. Before this module
// the same check stood written out at every catch in three shapes, one of
// them a substring of the error's text.

/**
 Whether a caught value is the filesystem's answer that nothing stands at the
 path, `ENOENT` on a Node error, which a spawn of a program that is not
 installed raises too.

 @param error - caught value

 @returns Whether it is that answer; a plain object carrying the code is not,
 since only Node's own errors carry it

 @example
 ```ts
 if (isMissingPathError({ error, },)) return [];
 ```
 */
export function isMissingPathError({ error, }: { readonly error: unknown; },): boolean {
  return Error.isError(error,)
    && ('code' in error)
    && (error.code === 'ENOENT');
}

/**
 Rethrows a caught value unless it says nothing stands at the path, so the
 catch that calls it goes on to its "not there" answer only for that one
 failure.

 @param error - caught value

 @throws The caught value itself, unchanged, when it is anything but the
 missing-path answer

 @example
 ```ts
 catch (error) {
   rethrowUnlessMissingPath({ error, },);
   return [];
 }
 ```
 */
export function rethrowUnlessMissingPath({ error, }: { readonly error: unknown; },): void {
  if (!isMissingPathError({ error, },))
    throw error;
}

//endregion Missing path error
