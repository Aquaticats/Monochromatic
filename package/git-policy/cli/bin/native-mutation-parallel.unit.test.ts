import { writeFile } from 'node:fs/promises';
import { join } from 'node:path';

import {
  type FakePodman,
  createFakePodman,
} from '@monochromatic-dev/mutation-test-cli-cargo-mutants/ts/fake-podman-fixture.ts';
import { runCaptured } from '@monochromatic-dev/mutation-test-cli-cargo-mutants/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  defaultGateImageTag,
  nativeTestImageRepository,
} from './native-gate-image-tag.ts';

/**
 Image ID of the gate image in the stand-in registry.
 */
const gateImage = 'a'.repeat(64);

/**
 Path of the wrapper script, which runs as TypeScript.
 */
const wrapperPath = join(
  import.meta.dirname,
  'native-mutation-parallel.ts',
);

/**
 Run the real wrapper with the stand-in `podman` first on `PATH`.

 @param fake - the fixture

 @param images - image reference to image ID, the stand-in registry

 @param logDirectory - what the launcher is told is its log directory

 @returns how the wrapper ended, with its standard output and error
 */
async function launch({
  fake,
  images,
  logDirectory,
}: {
  readonly fake: FakePodman;
  readonly images: Readonly<Record<string, string>>;
  readonly logDirectory: string;
}) {
  await writeFile(
    join(
      fake.directory,
      'images.json',
    ),
    JSON.stringify(images),
  );
  return await runCaptured({
    program: process.execPath,
    args: [
      wrapperPath,
      '--task',
      '//p:t',
      '--tag-env',
      'GIT_POLICY_NATIVE_IMAGE_TAG',
      '--setenv',
      `FAKE_DIR=${fake.directory}`,
      '--root',
      fake.workingDirectory,
      '--log-dir',
      logDirectory,
      'w',
      'a.rs',
    ],
    capture: true,
    environment: fake.environment,
  });
}

await describe({
  name: 'the native wrapper of the parallel launcher',
  children: [
    it({
      name: 'prints only the message of a tooling error, exits nonzero, and tells which gate image is missing',
      fn: async () => {
        await using fake = await createFakePodman();
        const run = await launch({
          fake,
          images: {},
          logDirectory: join(
            fake.directory,
            'logs',
          ),
        });
        expect(run.status).not.toBe(0);
        expect(run.stderr).toContain(`${nativeTestImageRepository}:${defaultGateImageTag}`);
        expect(run.stderr).not.toContain('    at ');
        expect(run.stderr).not.toContain('CargoMutantsToolingError');
      },
    }),
    it({
      name: 'keeps the stack trace of an unexpected error',
      fn: async () => {
        await using fake = await createFakePodman();
        // A file where the log directory belongs makes `mkdir` fail, which is no tooling error.
        const logDirectory = join(
          fake.directory,
          'a-file',
        );
        await writeFile(
          logDirectory,
          'a file',
        );
        const run = await launch({
          fake,
          images: { [`${nativeTestImageRepository}:${defaultGateImageTag}`]: gateImage },
          logDirectory,
        });
        expect(run.status).not.toBe(0);
        expect(run.stderr).toContain('    at ');
      },
    }),
  ],
});
