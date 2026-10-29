//region Unknown-error helpers

/**
 Returns whether unknown caught value carries expected error code.
 
 @param error - Unknown caught value.
 
 @param code - Error code to match.
 
 @returns Whether caught value has requested code.
 
 @example
 ```ts
 const absent = caughtErrorHasCode({ error, code: 'ENOENT' });
 ```
 */
export function caughtErrorHasCode(
  {
    error,
    code,
  }: {
    readonly code: string;
    readonly error: unknown;
  },
): boolean {
  if ((typeof error) !== 'object')
    return false;
  if (error === null)
    return false;
  /**
   Unknown code-like property from caught value.
   */
  const { code: actualCode, } = error as { readonly code?: unknown; };
  if ((typeof actualCode) !== 'string')
    return false;

  return actualCode === code;
}

//endregion Unknown-error helpers
