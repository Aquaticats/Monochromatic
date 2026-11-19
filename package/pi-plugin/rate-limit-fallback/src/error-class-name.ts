/**
 Name a caught value's class without reading its message, which may hold a credential.
 @module
 */

/**
 Read a caught value's class name.

 `Error.isError` is the modern check, but Pi declares `node >=22.19.0` and that function
 arrived in Node 24.3, so this reads a `name` string off the value instead of detecting the
 Error type by prototype. The message is never read: a transport failure can echo the request,
 and a credential-resolution failure can name the key it read.

 @param error - value a catch block caught

 @returns the value's own `name`, or its `typeof` when it carries none

 @example
 ```ts
 describeCaught('stand-in transport failure'); // 'string'
 ```
 */
export function describeCaught(error: unknown,): string {
  if (((typeof error) === 'object')
    && (error !== null)
    && ('name' in error)) {
    /**
     Candidate class name the value reports.
     */
    const name: unknown = Reflect.get(
      error,
      'name',
    );
    if ((typeof name) === 'string')
      return name;
  }
  return typeof error;
}
