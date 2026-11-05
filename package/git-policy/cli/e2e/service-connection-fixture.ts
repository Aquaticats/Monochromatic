/**
 Connection probes of the commit service's socket and of the files it leaves.

 @module
 */

import { once, } from 'node:events';
import { connect, } from 'node:net';

//region Constants

/**
 Error codes that say a path holds nothing: no socket file, or a socket file nothing listens on.
 */
const ABSENT_CODES: ReadonlySet<unknown> = new Set([
  'ENOENT',
  'ECONNREFUSED',
]);

//endregion Constants

//region Connection

/**
 Whether a failure says only that the path holds nothing.

 @param error - value a connection or a directory read threw

 @returns true for a missing path or a refused connection

 @example
 ```ts
 isAbsent(Object.assign(new Error('refused'), { code: 'ECONNREFUSED' })); // => true
 ```
 */
export function isAbsent(error: unknown,): boolean {
  return ((typeof error) === 'object')
    && (error !== null)
    && ('code' in error)
    && ABSENT_CODES.has(error.code,);
}

/**
 Whether something accepts connections on a socket path.

 @param socket - path of the unix socket

 @returns true when a connection was made

 @throws the connection's failure when it is not a refusal

 @example
 ```ts
 await listening('/work/s1/service/runtime/monochromatic-git-policy-cli/service.sock');
 ```
 */
export async function listening(socket: string,): Promise<boolean> {
  /**
   The connection attempt.
   */
  const attempt = connect(socket,);
  try {
    await once(
      attempt,
      'connect',
    );
  }
  catch (error) {
    attempt.destroy();
    if (isAbsent(error,))
      return false;
    throw error;
  }
  attempt.destroy();
  return true;
}

//endregion Connection
