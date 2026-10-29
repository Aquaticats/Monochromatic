/**
 The test runner of the gate image: a prebuilt, statically linked `cargo-nextest`, pinned by version and checksum.

 The image has no network, and a binary built on the host links the host's newer C library, so the runner is the
 project's own musl release asset. The host downloads it once, verifies the checksum before anything is extracted, and
 copies the executable into the build context.
 */
// The bin scripts sit outside the package tsconfig's include list, so type-aware lint checks them in an inferred
// program that loads no ambient Node types; this reference loads them, so `process` and `node:` imports stay typed
// in this file on its own instead of depending on a sibling module that carries the same reference.
/// <reference types="node" />
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
} from './native-verification-process.mjs';

/** Pinned so every gate runs the same runner; the checksum below belongs to this exact release asset. */
export const nextestVersion = '0.9.148';

/** SHA-256 of `cargo-nextest-0.9.148-x86_64-unknown-linux-musl.tar.gz`, equal to the digest GitHub lists for the asset. */
const nextestSha256 = 'bf92cb81450872d960ffa13857a83084d557ec0cdc141663d263bb23ef756f97';

/** Release asset, a static executable that runs on any C library. */
const nextestUrl = `https://github.com/nextest-rs/nextest/releases/download/cargo-nextest-${nextestVersion}/cargo-nextest-${nextestVersion}-x86_64-unknown-linux-musl.tar.gz`;

/** File name of the executable in the build context and in the image. */
const contextFileName = 'cargo-nextest';

/**
 Containerfile lines that install the runner next to `cargo`, so `cargo nextest` resolves it.

 @returns {string[]} lines to place after the base image is chosen, then check that the runner starts
 */
export function nextestCopyLines() {
  return [
    `COPY ${contextFileName} /usr/local/cargo/bin/cargo-nextest`,
    'RUN ["cargo", "nextest", "--version"]',
  ];
}

/**
 Download the pinned release asset once into the package's `target/` cache, verify its checksum, and place the
 executable in the build context.

 @param {{ context: string }} request - build context directory receiving the executable
 @throws {NativeVerificationError} when the download fails or its checksum differs from the pinned one
 */
export async function copyNextest({ context }) {
  const cache = resolve(
    'target/verification',
    'nextest-cache',
  );
  await mkdir(
    cache,
    { recursive: true },
  );
  const archiveName = `cargo-nextest-${nextestVersion}-musl.tar.gz`;
  const archive = join(
    cache,
    archiveName,
  );
  // No cached copy yet when the name is absent; the download below fills it. Any read failure of a present copy is real.
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

 @param {string} archive - cache path to write
 @throws {NativeVerificationError} when the download fails or its checksum differs from the pinned one
 */
async function downloadArchive(archive) {
  const response = await fetch(nextestUrl);
  if (!response.ok)
    throw new NativeVerificationError(`Downloading ${nextestUrl} failed with status ${String(response.status)}.`);
  const bytes = Buffer.from(await response.arrayBuffer());
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
 @param {Uint8Array} bytes - archive content
 @returns {string} lowercase hexadecimal SHA-256
 */
function sha256Of(bytes) {
  return createHash('sha256')
    .update(bytes)
    .digest('hex');
}
