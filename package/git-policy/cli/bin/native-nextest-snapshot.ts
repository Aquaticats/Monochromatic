/**
 The test runner of the gate image: a prebuilt, statically linked `cargo-nextest`, pinned by version and checksum.

 The image has no network, and a binary built on the host links the host's newer C library, so the runner is the
 project's own musl release asset. The host downloads it once, verifies the checksum before anything is extracted, and
 copies the executable into the build context.
 */
import { createHash } from 'node:crypto';
import {
  mkdir,
  readdir,
  readFile,
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
 Pinned so every gate runs the same runner; the checksum below belongs to this exact release asset.
 */
export const nextestVersion = '0.9.148';

/**
 SHA-256 of `cargo-nextest-0.9.148-x86_64-unknown-linux-musl.tar.gz`, equal to the digest GitHub lists for the asset.
 */
const nextestSha256 = 'bf92cb81450872d960ffa13857a83084d557ec0cdc141663d263bb23ef756f97';

/**
 Release asset, a static executable that runs on any C library.
 */
const nextestUrl = `https://github.com/nextest-rs/nextest/releases/download/cargo-nextest-${nextestVersion}/cargo-nextest-${nextestVersion}-x86_64-unknown-linux-musl.tar.gz`;

/**
 File name of the executable in the build context and in the image.
 */
const contextFileName = 'cargo-nextest';

/**
 Containerfile lines that install the runner next to `cargo`, so `cargo nextest` resolves it.

 @returns lines to place after the base image is chosen, then check that the runner starts

 @example
 ```ts
 const lines = nextestCopyLines();
 ```
 */
export function nextestCopyLines(): string[] {
  return [
    `COPY ${contextFileName} /usr/local/cargo/bin/cargo-nextest`,
    'RUN ["cargo", "nextest", "--version"]',
  ];
}

/**
 Download the pinned release asset once into the package's `target/` cache, verify its checksum, and place the
 executable in the build context.

 @param context - build context directory receiving the executable

 @throws NativeVerificationError when the download fails or its checksum differs from the pinned one

 @example
 ```ts
 await copyNextest({ context });
 ```
 */
export async function copyNextest({ context, }: {
  readonly context: string;
},): Promise<void> {
  /**
   Download cache below the package's `target/`, which Cargo's clean leaves alone and Git ignores.
   */
  const cache = resolve(
    'target/verification',
    'nextest-cache',
  );
  await mkdir(
    cache,
    { recursive: true },
  );
  /**
   File name of the cached archive.
   */
  const archiveName = `cargo-nextest-${nextestVersion}-musl.tar.gz`;
  /**
   Path of the cached archive.
   */
  const archive = join(
    cache,
    archiveName,
  );
  // No cached copy yet when the name is absent; the download below fills it. Any read failure of a present copy is real.
  /**
   Content of the cached archive, or nothing when no copy exists yet.
   */
  const cached = (await readdir(cache)).includes(archiveName)
    ? await readFile(archive)
    : undefined;
  if ((cached === undefined) || (sha256Of(cached) !== nextestSha256))
    await downloadArchive(archive);
  await runCommand({
    command: 'tar',
    args: [
      '--extract',
      '--gzip',
      `--file=${archive}`,
      `--directory=${context}`,
      contextFileName,
    ],
    capture: true,
  });
}

/**
 Download the pinned archive into the cache after checking its checksum.

 @param archive - cache path to write
 
 @throws NativeVerificationError when the download fails or its checksum differs from the pinned one
 */
async function downloadArchive(archive: string): Promise<void> {
  /**
   The release asset's HTTP response.
   */
  const response = await fetch(nextestUrl);
  if (!response.ok)
    throw new NativeVerificationError(`Downloading ${nextestUrl} failed with status ${String(response.status)}.`);
  /**
   The downloaded archive.
   */
  const bytes = Buffer.from(await response.arrayBuffer());
  /**
   Digest of the downloaded archive, compared with the pinned one before anything is written.
   */
  const actual = sha256Of(bytes);
  if (actual !== nextestSha256)
    throw new NativeVerificationError(
      `The cargo-nextest ${nextestVersion} archive has SHA-256 ${actual}, not the pinned ${nextestSha256}; nothing was extracted.`,
    );
  await writeFile(
    archive,
    bytes,
  );
}

/**
 Digest an archive for comparison with the pinned checksum.

 @param bytes - archive content
 
 @returns lowercase hexadecimal SHA-256
 */
function sha256Of(bytes: Uint8Array): string {
  return createHash('sha256')
    .update(bytes)
    .digest('hex');
}
