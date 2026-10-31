/**
 The fault injector of the gate image: `strace`, built statically from the pinned upstream source release.

 The fault-injection controls in `src/native/binary_fault_*_tests.rs` run the wrapper under `strace --inject` and report
 and check nothing when the tool is missing, so an image without it passes with fewer controls than the host runs.
 The image has no network and a package from the host links libraries the image lacks (`libunwind`, `liblzma`), so the
 host downloads the source archive once, verifies the checksum before anything is extracted, and the image builds a
 static executable from it in its own stage.
 */
import { createHash } from 'node:crypto';
import {
  copyFile,
  mkdir,
  readdir,
  readFile,
  writeFile,
} from 'node:fs/promises';
import {
  join,
  resolve,
} from 'node:path';

import { NativeVerificationError } from './native-verification-process.ts';

/**
 Pinned so every gate injects faults with the same tool; the checksum below belongs to this exact source archive.
 */
export const straceVersion = '6.13';

/**
 SHA-256 of `strace-6.13.tar.xz` as downloaded from the upstream release page. GitHub lists no digest for the
 asset, so this value was computed from the download and is checked on every later download.
 */
const straceSha256 = 'e209daf0ee038ca5adcc4c277e9273b4d51f46a2ff86da575d36742ac3508a17';

/**
 Upstream release asset, the source archive of the pinned version.
 */
const straceUrl = `https://github.com/strace/strace/releases/download/v${straceVersion}/strace-${straceVersion}.tar.xz`;

/**
 File name of the source archive in the build context.
 */
const contextFileName = 'strace.tar.xz';

/**
 Containerfile lines of the stage that builds `strace`. The stage starts from the same base as the test image, so the
 executable is static and runs on any C library; `libunwind` stack traces are left out because the tests never ask for
 them and the image has no such library.

 @param base - content-addressed base image ID shared with the test image
 @param jobs - parallel compile jobs the build container's CPUs allow

 @returns Containerfile lines ending with the built executable at a fixed stage path

 @example
 ```ts
 const lines = straceStageLines({ base, jobs: 2 });
 ```
 */
export function straceStageLines({ base, jobs, }: {
  readonly base: string;
  readonly jobs: number;
},): string[] {
  return [
    `FROM ${base} AS strace`,
    `COPY ${contextFileName} /strace/${contextFileName}`,
    'WORKDIR /strace',
    `RUN ["tar", "--extract", "--xz", "--file=${contextFileName}", "--strip-components=1"]`,
    `RUN ["sh", "-c", "./configure --quiet --enable-mpers=no --without-libunwind LDFLAGS=-static && make --quiet --jobs=${String(jobs)}"]`,
  ];
}

/**
 Containerfile lines of the test image that install the built `strace` where the tests find it on `PATH`.

 @returns lines to place in the test stage before it switches to the unprivileged user, ending with a start check

 @example
 ```ts
 const lines = straceCopyLines();
 ```
 */
export function straceCopyLines(): string[] {
  return [
    'COPY --from=strace /strace/src/strace /usr/local/bin/strace',
    'RUN ["strace", "-V"]',
  ];
}

/**
 Download the pinned source archive once into the package's `target/` cache, verify its checksum, and place it in the
 build context.

 @param context - build context directory receiving the archive

 @throws NativeVerificationError when the download fails or its checksum differs from the pinned one

 @example
 ```ts
 await copyStraceSource({ context });
 ```
 */
export async function copyStraceSource({ context, }: {
  readonly context: string;
},): Promise<void> {
  /**
   Download cache below the package's `target/`, which Cargo's clean leaves alone and Git ignores.
   */
  const cache = resolve(
    'target/verification',
    'strace-cache',
  );
  await mkdir(
    cache,
    { recursive: true },
  );
  /**
   File name of the cached archive.
   */
  const archiveName = `strace-${straceVersion}.tar.xz`;
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
  if ((cached === undefined) || (sha256Of(cached) !== straceSha256))
    await downloadArchive(archive);
  await copyFile(
    archive,
    join(
      context,
      contextFileName,
    ),
  );
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
  const response = await fetch(straceUrl);
  if (!response.ok)
    throw new NativeVerificationError(`Downloading ${straceUrl} failed with status ${String(response.status)}.`);
  /**
   The downloaded archive.
   */
  const bytes = Buffer.from(await response.arrayBuffer());
  /**
   Digest of the downloaded archive, compared with the pinned one before anything is written.
   */
  const actual = sha256Of(bytes);
  if (actual !== straceSha256)
    throw new NativeVerificationError(
      `The strace ${straceVersion} archive has SHA-256 ${actual}, not the pinned ${straceSha256}; nothing was written.`,
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
