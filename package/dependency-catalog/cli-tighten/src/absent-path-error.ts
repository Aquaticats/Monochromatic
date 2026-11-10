/**
 Distinguishes an absent filesystem path from one the process could not read.

 Both catalog-tighten scans read directories and manifests whose absence is
 ordinary (a repo with no `package/` layout, an importer with no manifest),
 while a permission or I/O failure is not: it says the scan could not answer.
 Folding the two together reports every catalog entry as uninstalled and every
 package as undeclared, which the caller can no longer tell from the truth.
 */

//region Absent path errors

/**
 Error codes that mean nothing stands at the path.

 `ENOENT` covers a missing entry and `ENOTDIR` a path component that is a file
 rather than a directory. Every other code, such as `EACCES` for a directory
 this process may not search or `EIO`, says the probe could not answer.
 */
const ABSENT_PATH_ERROR_CODES: ReadonlySet<string> = new Set([
  'ENOENT',
  'ENOTDIR',
],);

/**
 Reports whether a caught value says the probed path holds no entry.
 
 @param error - caught value from a filesystem call
 
 @returns whether the path is absent rather than uninspectable
 
 @example
 ```ts
 isAbsentPathError(Object.assign(new Error('missing',), { code: 'ENOENT', })) // true
 isAbsentPathError(Object.assign(new Error('denied',), { code: 'EACCES', })) // false
 ```
 */
export function isAbsentPathError(error: unknown,): boolean {
  if (!(Error.isError(error,)))
    return false;
  if (!('code' in error))
    return false;
  if ((typeof error.code) !== 'string')
    return false;
  return ABSENT_PATH_ERROR_CODES.has(error.code,);
}

//endregion Absent path errors
