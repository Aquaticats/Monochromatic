/**
 The real `pnpm` of the gate image, taken from the `pnpm` row of `toolchain.lock.json`.

 Controls that run a real pnpm (`package_repair::real_pnpm_*`) must run in the gate, which has no network, so the image
 carries the locked release archive. The host downloads it once, checks the row's SHA-256 before anything is extracted,
 and puts the extracted tree in the build context. AGENTS.md: in containers and CI, a tool comes from its lock row's
 archive and checksum.
 */
import { createHash, } from 'node:crypto';
import {
  mkdir,
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import {
  join,
  resolve,
} from 'node:path';

import {
  NativeVerificationError,
  runCommand,
} from './native-verification-process.ts';

/**
 The platform of the gate image: CachyOS on x86-64, which links the GNU C library, so the lock's glibc archive applies.
 */
const platform = 'linux-x64';

/**
 Directory of the build context that holds the extracted tree, and the directory of the image that receives it.
 */
const contextDirectory = 'pnpm-tree';

/**
 Where the extracted tree lives in the image; the directory joins `PATH`, so the test process finds `pnpm`.
 */
const imageDirectory = '/opt/pnpm';

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
  /**
   The parsed lock, trusted only as far as the checks below.
   */
  const lock = JSON.parse(lockText) as {
    readonly tools?: {
      readonly pnpm?: {
        readonly version?: string;
        readonly artifacts?: Readonly<Record<string, { readonly url?: string; readonly sha256?: string; } | undefined>>;
      };
    };
  };
  /**
   The pnpm row.
   */
  const row = lock.tools?.pnpm;
  if (row?.version === undefined)
    throw new NativeVerificationError('toolchain.lock.json has no pnpm row with a version.');
  /**
   The row's archive for the image's platform.
   */
  const archive = row.artifacts?.[platform];
  if ((archive?.url === undefined) || (archive.sha256 === undefined))
    throw new NativeVerificationError(`The pnpm row of toolchain.lock.json has no ${platform} archive with an address and a checksum.`);
  return {
    version: row.version,
    url: archive.url,
    sha256: archive.sha256,
  };
}

/**
 Containerfile lines that install the extracted pnpm tree and check that it starts.

 The tree is copied whole because the release archive holds the executable beside its `dist` directory; the mode makes it
 readable and runnable by the tester user.

 @returns lines to place after the base image is chosen

 @example
 ```ts
 const lines = pnpmCopyLines();
 ```
 */
export function pnpmCopyLines(): string[] {
  return [
    `COPY --chmod=0755 ${contextDirectory} ${imageDirectory}`,
    // The build runs as the tester, who cannot write `/usr/local/bin`, so the directory joins the search path instead of a link.
    `ENV PATH=${imageDirectory}:$PATH`,
    'RUN ["pnpm", "--version"]',
  ];
}

/**
 Download the locked archive into the package's cache, verify its checksum, and extract it into the build context.

 @param context - build context directory receiving the tree

 @throws NativeVerificationError when the download fails or the checksum differs from the lock row's

 @example
 ```ts
 await copyPnpm({ context });
 ```
 */
export async function copyPnpm({ context, }: {
  readonly context: string;
},): Promise<void> {
  /**
   The lock row, read from the repository's lock file.
   */
  const locked = lockedPnpm({
    lockText: await readFile(
      join(
        import.meta.dirname,
        '..',
        '..',
        '..',
        '..',
        'toolchain.lock.json',
      ),
      'utf8',
    ),
  });
  /**
   Download cache below the package's `target/`, which Cargo's clean leaves alone and Git ignores.
   */
  const cache = resolve(
    'target/verification',
    'pnpm-cache',
  );
  await mkdir(
    cache,
    { recursive: true },
  );
  /**
   Path of the cached archive, named by version so a new row downloads a new file.
   */
  const archive = join(
    cache,
    `pnpm-${locked.version}-${platform}.tar.gz`,
  );
  /**
   Content of the cached archive, or nothing when no valid copy exists.
   */
  const cached = await readFile(archive)
    .catch(function noCopy() {
      return undefined;
    });
  if ((cached === undefined) || (sha256Of(cached) !== locked.sha256))
    await downloadArchive({
      locked,
      archive,
    });
  /**
   Directory of the build context that receives the extracted tree.
   */
  const tree = join(
    context,
    contextDirectory,
  );
  await rm(
    tree,
    {
      recursive: true,
      force: true,
    },
  );
  await mkdir(
    tree,
    { recursive: true },
  );
  await runCommand({
    command: 'tar',
    args: [
      '--extract',
      '--gzip',
      `--file=${archive}`,
      `--directory=${tree}`,
    ],
    capture: true,
  });
}

/**
 Download the locked archive after checking its checksum.

 @param locked - the lock row

 @param archive - cache path to write

 @throws NativeVerificationError when the download fails or its checksum differs from the lock row's
 */
async function downloadArchive({ locked, archive, }: {
  readonly locked: LockedArchive;
  readonly archive: string;
},): Promise<void> {
  /**
   The release asset's HTTP response.
   */
  const response = await fetch(locked.url);
  if (!response.ok)
    throw new NativeVerificationError(`Downloading ${locked.url} failed with status ${String(response.status)}.`);
  /**
   The downloaded archive.
   */
  const bytes = Buffer.from(await response.arrayBuffer());
  /**
   Digest of the downloaded archive, compared with the lock row's before anything is written.
   */
  const actual = sha256Of(bytes);
  if (actual !== locked.sha256)
    throw new NativeVerificationError(
      `The pnpm ${locked.version} archive has SHA-256 ${actual}, not the lock row's ${locked.sha256}; nothing was extracted.`,
    );
  await writeFile(
    archive,
    bytes,
  );
}

/**
 Digest an archive for comparison with the lock row's checksum.

 @param bytes - archive content

 @returns lowercase hexadecimal SHA-256
 */
function sha256Of(bytes: Uint8Array): string {
  return createHash('sha256')
    .update(bytes)
    .digest('hex');
}
