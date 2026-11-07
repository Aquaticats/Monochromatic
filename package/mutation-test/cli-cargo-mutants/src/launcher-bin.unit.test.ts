import {
  readdir,
  readFile,
  writeFile,
} from 'node:fs/promises';
import { join } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  createFakePodman,
  type FakePodman,
} from './fake-podman-fixture.ts';
import { runCaptured } from './run-captured.ts';

/**
 Repository the cases give the launcher.
 */
const repository = 'localhost/product-test';

/**
 Image ID every tag of the cases names.
 */
const image = 'a'.repeat(64);

/**
 Name of the file in which the stand-in `systemd-run` records the units it was asked to start.
 */
const startLogName = 'systemd-run.log';

/**
 Path of the launcher script, which runs as TypeScript.
 */
const launcherPath = join(
  import.meta.dirname,
  '..',
  'bin',
  'mutation-parallel.ts',
);

/**
 Run the real launcher script with the stand-in `podman` and `systemd-run` first on `PATH`.

 @param fake - the fixture
 @param tags - unit tags the stand-in registry holds
 @param argv - arguments after the required ones and before the label
 @param labelAndFiles - the label and files
 @param environment - extra environment of the launcher

 @returns how the launcher ended, with its standard output and error
 */
async function launch({
  fake,
  tags,
  argv = [],
  labelAndFiles,
  environment = {},
}: {
  readonly fake: FakePodman;
  readonly tags: readonly string[];
  readonly argv?: readonly string[];
  readonly labelAndFiles: readonly string[];
  readonly environment?: Readonly<Record<string, string>>;
}) {
  /**
   The stand-in registry: every tag names the same image.
   */
  const registry = Object.fromEntries(tags.map(function entryOf(tag) {
    return [
      `${repository}:${tag}`,
      image,
    ];
  }));
  await writeFile(
    join(
      fake.directory,
      'images.json',
    ),
    JSON.stringify(registry),
  );
  return await runCaptured({
    program: process.execPath,
    args: [
      launcherPath,
      '--task',
      '//p:t',
      '--tag-env',
      'IMG_TAG',
      '--image-repository',
      repository,
      '--setenv',
      `FAKE_DIR=${fake.directory}`,
      '--root',
      fake.workingDirectory,
      '--log-dir',
      join(
        fake.directory,
        'logs',
      ),
      '--tmp-dir',
      join(
        fake.directory,
        'tmp',
      ),
      ...argv,
      ...labelAndFiles,
    ],
    capture: true,
    environment: {
      ...fake.environment,
      ...environment,
    },
  });
}

/**
 Unit names the stand-in `systemd-run` was asked to start, in call order.

 @param fake - the fixture

 @returns the names, or none when it was never called
 */
async function startedUnits(fake: FakePodman): Promise<readonly string[]> {
  if (!(await readdir(fake.directory)).includes(startLogName))
    return [];
  return (await readFile(
    join(
      fake.directory,
      startLogName,
    ),
    'utf8',
  ))
    .split('\n')
    .filter(function isName(name) {
      return name !== '';
    });
}

await describe({
  name: 'the launcher script',
  children: [
    it({
      name: 'exits 0 when every unit started',
      fn: async () => {
        await using fake = await createFakePodman();
        const run = await launch({
          fake,
          tags: ['w0', 'w1'],
          labelAndFiles: ['w', 'a.rs', 'b.rs'],
        });
        expect(run.status).toBe(0);
        expect((await startedUnits(fake)).length).toBe(2);
      },
    }),
    it({
      name: 'exits nonzero when systemd-run refuses one unit, reports each unit, and leaves the others started',
      fn: async () => {
        await using fake = await createFakePodman();
        const run = await launch({
          fake,
          tags: [
            'w0',
            'w1',
            'w2',
          ],
          labelAndFiles: ['w', 'a.rs', 'b.rs', 'c.rs'],
          environment: { FAKE_SYSTEMD_FAIL_UNIT: 'mutpar-w-b' },
        });
        expect(run.status).not.toBe(0);
        expect(await startedUnits(fake)).toEqual([
          'mutpar-w-a',
          'mutpar-w-b',
          'mutpar-w-c',
        ]);
        expect(run.stdout).toContain('mutpar-w-a: started');
        expect(run.stdout).toContain('mutpar-w-c: started');
        expect(run.stdout).toContain('mutpar-w-b: Failed to start transient service unit: fake systemd-run failure for mutpar-w-b');
        expect(run.stdout).toContain('wait with: systemctl --user is-active mutpar-w-a mutpar-w-c');
        expect(run.stderr).not.toContain('    at ');
      },
    }),
    it({
      name: 'prints only the message of a tooling error to standard error, exits nonzero and starts nothing',
      fn: async () => {
        await using fake = await createFakePodman();
        const run = await launch({
          fake,
          tags: ['w0'],
          labelAndFiles: ['w', 'a.rs', 'b.rs'],
        });
        expect(run.status).not.toBe(0);
        expect(run.stderr).toContain(`${repository}:w1`);
        expect(run.stderr).not.toContain('    at ');
        expect(run.stderr).not.toContain('CargoMutantsToolingError');
        expect(await startedUnits(fake)).toEqual([]);
      },
    }),
    it({
      name: 'keeps the stack trace of an unexpected error, which is a defect of the launcher or the host',
      fn: async () => {
        await using fake = await createFakePodman();
        // A file where the log directory belongs makes `mkdir` fail, which is no tooling error.
        await writeFile(
          join(
            fake.directory,
            'logs',
          ),
          'a file',
        );
        const run = await launch({
          fake,
          tags: ['w0'],
          labelAndFiles: ['w', 'a.rs'],
        });
        expect(run.status).not.toBe(0);
        expect(run.stderr).toContain('    at ');
        expect(await startedUnits(fake)).toEqual([]);
      },
    }),
  ],
});
