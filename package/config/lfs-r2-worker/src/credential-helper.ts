/**
 The Git credential helper that serves the LFS host, scoped to that host alone.

 The helper is the stock `git credential-store` reading a private file, so no
 secret ever reaches a Git config value, a URL or a command line. Its command
 holds only the helper name and the file's path, which is why it can sit in a
 repository's own configuration and be read by anyone.

 @module
 */

import { shellQuote, } from '@monochromatic-dev/shell-argument-core/ts';

/**
 An LFS host split into the parts a credential request needs.
 */
export type LfsHost = {
  /**
   Scheme and authority, as Git matches a `credential.<url>.helper` subsection.
   */
  readonly origin: string;
  /**
   Scheme without its colon, as a credential request spells `protocol`.
   */
  readonly protocol: string;
  /**
   Host and port, as a credential request spells `host`.
   */
  readonly host: string;
};

/**
 A host value that is not an absolute URL with a scheme this Worker is served on.
 */
export class LfsHostError extends Error {
  /**
   @param message - why the value cannot name an LFS host
   */
  constructor(message: string,) {
    super(message,);
    this.name = 'LfsHostError';
  }
}

/**
 Split an LFS host URL into the parts a credential request needs.

 @param host - absolute URL of the LFS server, such as the Worker URL

 @returns origin, protocol and host of that URL

 @throws {@link LfsHostError} when the value is not an absolute `http` or `https` URL

 @example
 ```ts
 parseLfsHost({ host: 'https://monochromatic-lfs.aquaticat.workers.dev' });
 ```
 */
export function parseLfsHost({ host, }: { readonly host: string; },): LfsHost {
  if (!URL.canParse(host,)) {
    throw new LfsHostError(`"${host}" is not an absolute URL`,);
  }
  /**
   Parsed host URL.
   */
  const url = new URL(host,);
  if ((url.protocol !== 'https:') && (url.protocol !== 'http:')) {
    throw new LfsHostError(`"${host}" is not an http or https URL`,);
  }
  return {
    origin: url.origin,
    protocol: url.protocol
      .slice(
      0,
      -1,
      ),
    host: url.host,
  };
}

/**
 The helper command and config key that serve one LFS host from a private file.
 */
export type CredentialHelper = {
  /**
   Config key naming the host, `credential.<origin>.helper`.
   */
  readonly key: string;
  /**
   Helper command, which holds a helper name and the file's path and never a secret.
   */
  readonly command: string;
};

/**
 Build the host-scoped helper that reads credentials from a private file.

 The command is spelled as a shell word list because Git runs a helper through the
 shell, so the file's path is quoted for it here rather than by whoever writes the
 key.

 @param host - LFS host the helper serves

 @param credentialFile - absolute path of the private credential file

 @returns config key and helper command

 @example
 ```ts
 storeCredentialHelper({ host: parseLfsHost({ host }), credentialFile });
 // { key: 'credential.https://lfs.example.helper', command: "store --file '/…/credentials'" }
 ```
 */
export function storeCredentialHelper({
  host,
  credentialFile,
}: {
  readonly host: LfsHost;
  readonly credentialFile: string
},): CredentialHelper {
  return {
    key: `credential.${host.origin}.helper`,
    command: `store --file ${shellQuote(credentialFile,)}`,
  };
}
