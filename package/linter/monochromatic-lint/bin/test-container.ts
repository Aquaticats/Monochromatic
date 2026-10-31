#!/usr/bin/env node
/**
 Run the full suite by default, or one explicitly named slice in the same isolated source snapshot.
 */
import {
  cp,
  mkdir,
  mkdtempDisposable,
  writeFile,
} from 'node:fs/promises';
import {
  join,
  resolve,
} from 'node:path';

import { runCaptured } from '@monochromatic-dev/mutation-test-cli-cargo-mutants/ts';

// The gate image runs the suite with the runner the git-policy/cli gate pins by version and checksum.
import { copyNextest } from '../../../git-policy/cli/bin/native-nextest-snapshot.ts';

/**
 Scratch lives inside the package's gitignored node_modules/.monochromatic, never the system temporary directory,
 whose backing device is unknowable.
 */
const scratchBase = join(
  import.meta.dirname,
  '..',
  'node_modules',
  '.monochromatic',
  'monochromatic-lint',
);

/**
 Crate entries the snapshot copies from this package and from the JSONC editing crate.
 */
const copiedNames: readonly string[] = [
  'Cargo.toml',
  'Cargo.lock',
  'src',
  'fixtures',
];

/**
 Build the snapshot image from the package sources and run the suite, or one nextest test-name filter, in it.

 @param source - the package directory whose sources the snapshot copies

 @param context - the build context directory to fill

 @param testImage - the name the snapshot image gets

 @param filter - the nextest test-name filter, or nothing for the full suite

 @throws Error when vendoring, the image build or the container run fails

 @example
 ```ts
 await buildAndRun({ source: process.cwd(), context, testImage });
 ```
 */
async function buildAndRun({
  source,
  context,
  testImage,
  filter,
}: {
  readonly source: string;
  readonly context: string;
  readonly testImage: string;
  readonly filter?: string;
}): Promise<void> {
  await Promise.all(copiedNames.flatMap(function copyOf(name) {
    return [
      cp(
        join(
          source,
          name,
        ),
        join(
          context,
          'monochromatic-lint',
          name,
        ),
        { recursive: true },
      ),
      cp(
        resolve(
          source,
          '../../jsonc-edit/core-rust',
          name,
        ),
        join(
          context,
          'jsonc-edit',
          name,
        ),
        { recursive: true },
      ),
    ];
  }));
  /**
   Where `cargo vendor` places the dependency sources inside the context.
   */
  const vendorPath = join(
    context,
    'vendor',
  );
  /**
   What the offline vendoring left behind: the source mapping on standard output.
   */
  const vendor = await runCaptured({
    program: 'cargo',
    args: [
      'vendor',
      '--locked',
      '--offline',
      '--versioned-dirs',
      vendorPath,
    ],
    capture: true,
  });
  if (vendor.status !== 0) throw new Error(`Cargo vendor failed: ${vendor.stderr}`);
  /**
   The Cargo configuration that maps the vendored sources to their place in the image.
   */
  const config = vendor.stdout
    .replaceAll(
    JSON.stringify(vendorPath),
    '"/work/vendor"',
    );
  if (!config.includes('directory = "/work/vendor"'))
    throw new Error('Cargo vendor did not emit the expected source mapping');
  await mkdir(
    join(
      context,
      'cargo-config',
    ),
    { recursive: true },
  );
  await writeFile(
    join(
      context,
      'cargo-config/config.toml',
    ),
    config,
  );
  await cp(
    resolve(
      source,
      '../../../clippy.toml',
    ),
    join(
      context,
      'clippy.toml',
    ),
  );
  // The registry guard test validates the repository's own configuration, so the snapshot carries it.
  await cp(
    resolve(
      source,
      '../../../monochromatic-lint.config.jsonc',
    ),
    join(
      context,
      'monochromatic-lint.config.jsonc',
    ),
  );
  await cp(
    join(
      source,
      'test.Containerfile',
    ),
    join(
      context,
      'Containerfile',
    ),
  );
  await copyNextest({ context });
  /**
   What the image build left behind.
   */
  const build = await runCaptured({
    program: 'podman',
    args: [
      'build',
      '--network=none',
      '--http-proxy=false',
      '--pull=never',
      '--memory=2g',
      '--cpu-period=100000',
      '--cpu-quota=200000',
      '--tag',
      testImage,
      context,
    ],
    capture: false,
  });
  if (build.status !== 0) throw new Error(`Container build failed: ${build.status}`);
  /**
   What the test container left behind.
   */
  const run = await runCaptured({
    program: 'podman',
    args: [
      'run',
      '--rm',
      '--init',
      '--network=none',
      '--memory=2g',
      '--cpus=2',
      '--pids-limit=128',
      testImage,
      ...(filter === undefined ? [] : [
        'cargo',
        'nextest',
        'run',
        '--offline',
        '--locked',
        '--all-targets',
        filter,
      ]),
    ],
    capture: false,
  });
  if (run.status !== 0) throw new Error(`Container test failed: ${run.status}`);
}

/**
 Read the optional test-name filter, then run the suite in a build context that is removed afterwards.

 @throws Error when more than one filter is given or the run fails

 @example
 ```ts
 await main();
 ```
 */
async function main(): Promise<void> {
  /**
   The optional nextest test-name filter, then anything extra, which is rejected.
   */
  const [filter, ...extra] = process.argv
    .slice(2);
  if (extra.length > 0) throw new Error('Expected at most one nextest test-name filter');
  await mkdir(
    scratchBase,
    { recursive: true },
  );
  /**
   The disposable build context, removed when this function ends.
   */
  await using context = await mkdtempDisposable(join(
    scratchBase,
    'monochromatic-lint-test-',
  ));
  await buildAndRun({
    source: process.cwd(),
    context: context.path,
    // Concurrent worktrees set MONOCHROMATIC_LINT_IMAGE_TAG so one snapshot never runs another's image.
    testImage: `localhost/monochromatic-lint-test:${process.env
      .MONOCHROMATIC_LINT_IMAGE_TAG
      ?? 'development'}`,
    ...(filter === undefined ? {} : { filter }),
  });
}

await main();
