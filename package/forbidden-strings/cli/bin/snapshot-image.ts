/**
 The image of a snapshot: its Containerfile, its build and the Git check.
 */
import {
  readFile,
  writeFile,
} from 'node:fs/promises';
import { join } from 'node:path';

import {
  run,
  ScannerVerificationError,
} from './scanner-run.ts';
import {
  baseImage,
  limits,
  scanner,
} from './snapshot-constants.ts';

/**
 A built image and the Git it carries.
 */
export type BuiltImage = {
  readonly image: string;
  readonly version: string;
};

/**
 Write the Containerfile that bakes the copied sources, the vendored dependencies and the host tools.

 @param context - the build context

 @param additional - Containerfile lines for the host tools, placed after the base image

 @param command - what the container runs

 @param fuzzing - true to start in the fuzz sidecar's directory

 @example
 ```ts
 await writeContainerfile({ context, additional: [], command: ['cargo', 'test'], fuzzing: false });
 ```
 */
export async function writeContainerfile({
  context,
  additional,
  command,
  fuzzing,
}: {
  readonly context: string;
  readonly additional: readonly string[];
  readonly command: readonly string[];
  readonly fuzzing: boolean;
}): Promise<void> {
  await writeFile(
    join(
      context,
      'Containerfile',
    ),
    [
      `FROM ${baseImage}`,
      ...additional,
      'COPY package /work/package',
      'COPY vendor /work/vendor',
      'COPY cargo-config /work/.cargo',
      'COPY forbidden-strings.append.txt clippy.toml /work/',
      'COPY .github /work/.github',
      'RUN ["mkdir", "--parents", "/home/tester/.cargo"]',
      'RUN ["chown", "--recursive", "1000:1000", "/work", "/home/tester"]',
      'ENV HOME=/home/tester CARGO_HOME=/home/tester/.cargo CARGO_NET_OFFLINE=true CARGO_BUILD_JOBS=2 CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0 RAYON_NUM_THREADS=2',
      'USER 1000:1000',
      `WORKDIR /work/${fuzzing ? `${scanner}.fuzz` : scanner}`,
      `CMD ${JSON.stringify(command)}`,
      '',
    ].join('\n'),
  );
}

/**
 Build the context into an image and check the Git it carries.

 @param context - the build context with its Containerfile

 @param name - the label of the image tag

 @param sourceSha256 - digest of the copied sources, which makes the tag content-addressed

 @returns the content-addressed ID of the image, and the Git version it reports

 @throws ScannerVerificationError when the image carries another Git than the fixture requires

 @example
 ```ts
 const { image } = await buildImage({ context, name: 'test', sourceSha256 });
 ```
 */
export async function buildImage({
  context,
  name,
  sourceSha256,
}: {
  readonly context: string;
  readonly name: string;
  readonly sourceSha256: string;
}): Promise<BuiltImage> {
  /**
   File that receives the ID of the built image.
   */
  const imageFile = join(
    context,
    'image.id',
  );
  await run({
    command: 'podman',
    args: [
      'build',
      '--network=none',
      '--http-proxy=false',
      '--pull=never',
      '--memory=2g',
      '--cpu-period=100000',
      '--cpu-quota=200000',
      '--iidfile',
      imageFile,
      '--tag',
      `localhost/scanner-${name}:${sourceSha256}`,
      context,
    ],
  });
  /**
   The content-addressed ID of the built image.
   */
  const image = (await readFile(
    imageFile,
    'utf8',
  )).trim();
  /**
   The Git version text the image reports.
   */
  const version = (await run({
    command: 'podman',
    args: [
      'run',
      '--rm',
      ...limits,
      image,
      '/usr/bin/git',
      '--version',
    ],
    capture: true,
  })).stdout
    .trim();
  if (version !== 'git version 2.56.0') throw new ScannerVerificationError(`Wrong Git in scanner fixture: ${version}`);
  return {
    image,
    version,
  };
}
