/**
 CLI argument parsing utilities.
 
 Shared between `server.ts` (port resolution) and `lib/db.ts` (database path resolution)
 to avoid duplicating the `--name=value` extraction logic.
 */

/**
 Sentinel returned by {@link getArgumentValue} when the flag is not present.
 
 A unique `Symbol` keeps "absent" out of a nullish union (banned by
 `no-nullish-union`); callers narrow with `=== ARGUMENT_ABSENT`.
 */
export const ARGUMENT_ABSENT: unique symbol = Symbol('named command line flag absent from argv',);

/**
 Extracts the value of a `--name=value` CLI argument from `process.argv`.
 
 @param name - Argument name without the `--` prefix
 
 @returns Extracted value, or {@link ARGUMENT_ABSENT} when the argument is absent
 
 @example
 ```ts
 const port = getArgumentValue('port'); // '3000' when invoked with --port=3000
 ```
 */
export function getArgumentValue(name: string,): string | typeof ARGUMENT_ABSENT {
  /**
   Match string for the `--name=` portion preceding the value.
   */
  const prefix = `--${name}=`;
  /**
   First `process.argv` entry starting with the prefix, if any.
   */
  const argument = process.argv
    .find(function hasPrefix(entry,) {
      return entry.startsWith(prefix,);
    },);
  if (argument === undefined)
    return ARGUMENT_ABSENT;
  return argument.slice(prefix.length,);
}
