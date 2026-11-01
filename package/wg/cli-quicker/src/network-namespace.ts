import { createHash, } from 'node:crypto';
import { readlink, } from 'node:fs/promises';

/**
 Hex characters retained from network-namespace identity hash.
 */
const NETWORK_NAMESPACE_KEY_LENGTH = 32;

/**
 Stable identity for current Linux network namespace during its lifetime.
 */
export type NetworkNamespace = {
  /**
   Kernel namespace link identity.
   */
  readonly identity: string;

  /**
   Filesystem-safe collision-resistant ownership key.
   */
  readonly key: string;
};

/**
 Reads current network namespace and derives ownership key.
 
 @returns Namespace identity and SHA-256 key.
 
 @example
 ```ts
 await currentNetworkNamespace();
 ```
 */
export async function currentNetworkNamespace(): Promise<NetworkNamespace> {
  /**
   Kernel-provided namespace identity such as `net:[4026531840]`.
   */
  const identity = await readlink('/proc/self/ns/net',);
  return {
    identity,
    key: createHash('sha256',)
      .update(identity,)
      .digest('hex',)
      .slice(
      0,
      NETWORK_NAMESPACE_KEY_LENGTH,
      ),
  };
}
