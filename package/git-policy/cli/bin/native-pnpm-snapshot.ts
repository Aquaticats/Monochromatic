/**
 The real `pnpm` of the gate image, taken from the `pnpm` row of `toolchain.lock.json`.

 Controls that run a real pnpm (`package_repair::real_pnpm_*`) must run in the gate, which has no network, so the image
 carries the locked release archive. The host downloads it once, checks the row's SHA-256 before anything is extracted,
 and puts the extracted tree in the build context. AGENTS.md: in containers and CI, a tool comes from its lock row's
 archive and checksum.
 */
import { NativeVerificationError, } from './native-verification-process.ts';

/**
 The archive row of the lock for the image's platform.
 */
export interface LockedArchive {
  /**
   Release version the row records.
   */
  readonly version: string;
  /**
   Address of the release archive.
   */
  readonly url: string;
  /**
   Lowercase hexadecimal SHA-256 of the archive.
   */
  readonly sha256: string;
}

/**
 Read the pnpm archive for the image's platform from the text of `toolchain.lock.json`.

 @param lockText - content of the lock file

 @returns the row's version, address and checksum

 @throws NativeVerificationError when the lock has no pnpm row or no archive for the platform
 */
export function lockedPnpm({ lockText, }: {
  readonly lockText: string;
},): LockedArchive {
  throw new NativeVerificationError(`not written yet ${lockText.length}`);
}

/**
 Containerfile lines that install the extracted pnpm tree and check that it starts.

 @returns lines to place after the base image is chosen
 */
export function pnpmCopyLines(): string[] {
  return [];
}

/**
 Download the locked archive into the package's cache, verify its checksum, and extract it into the build context.

 @param context - build context directory receiving the tree

 @throws NativeVerificationError when the download fails or the checksum differs from the lock row's
 */
export async function copyPnpm({ context, }: {
  readonly context: string;
},): Promise<void> {
  throw new NativeVerificationError(`not written yet ${context}`);
}
