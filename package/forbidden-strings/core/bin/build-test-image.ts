#!/usr/bin/env node
/**
 Bake the engine crate into the source snapshot image that a mutation campaign layers cargo-mutants over.

 The snapshot holds `Cargo.toml`, `Cargo.lock`, `src`, `tests` and `.cargo` (the cargo-mutants configuration with the
 proofs of the equivalent mutants), plus the fetched dependencies; nothing is mounted at campaign time.
 The base image with the toolchain, cargo-nextest and cargo-mutants is built first by `test:mutation:image`.
 */
import {
  cp,
  mkdir,
  mkdtempDisposable,
} from 'node:fs/promises';
import { join } from 'node:path';

import {
  buildLimitArguments,
  parseContainerLimitOptions,
  runPodman,
} from '@monochromatic-dev/mutation-test-cli-cargo-mutants/ts';

/**
 Concurrent worktrees set FORBIDDEN_REGEX_IMAGE_TAG so one snapshot never mutates another's image.
 */
const imageTag = process.env
  .FORBIDDEN_REGEX_IMAGE_TAG
  ?? 'development';

/**
 Crate directory, whose files go into the snapshot.
 */
const crate = join(
  import.meta.dirname,
  '..',
);

/**
 Crate entries baked into the snapshot.
 */
const snapshotEntries: readonly string[] = [
  'Cargo.toml',
  'Cargo.lock',
  'src',
  'tests',
  '.cargo',
];

/**
 Scratch lives inside the package's gitignored node_modules/.monochromatic, never the system temporary directory,
 whose backing device is unknowable.
 */
const scratchBase = join(
  crate,
  'node_modules',
  '.monochromatic',
  'forbidden-regex',
);

/**
 Build bounds from `--memory` and `--cpus`; this task takes no other option.
 */
const {
  limits,
  rest,
} = parseContainerLimitOptions({
  options: process.argv
    .slice(2),
  acceptPidsLimit: false,
  leadingOnly: false,
});
if (rest.length > 0)
  throw new TypeError(`Only --memory and --cpus are accepted, got ${rest.join(' ')}.`);
await mkdir(
  scratchBase,
  { recursive: true },
);
/**
 Disposable build context holding the snapshot files.
 */
await using context = await mkdtempDisposable(join(
  scratchBase,
  'forbidden-regex-test-',
));
await Promise.all(snapshotEntries.map(async function copyEntry(entry) {
  await cp(
    join(
      crate,
      entry,
    ),
    join(
      context.path,
      entry,
    ),
    { recursive: true },
  );
}));
await runPodman({
  args: [
    'build',
    '--http-proxy=false',
    '--pull=never',
    ...buildLimitArguments(limits),
    '--file',
    join(
      crate,
      'test.Containerfile',
    ),
    '--tag',
    `localhost/forbidden-regex-test:${imageTag}`,
    context.path,
  ],
  capture: false,
  allowFailure: false,
});
