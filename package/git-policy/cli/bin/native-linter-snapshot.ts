/**
 Snapshot inputs for the Markdown linter the native wrapper's `markdown/autofix` policy starts as a child process:
 the linter crate's sources, the JSONC crate it shares with the wrapper, and the registry dependencies its own
 lockfile names.

 The gate image builds the linter from these sources in a stage of its own, on the same audited base as the
 wrapper, and copies only the executable into the test image. The container has no network, so the registry
 sources are vendored on the host from the Cargo cache, as the scanner's are. A binary built on the host is never
 copied in: the host's C library is newer than the image's.
 */
// The bin scripts sit outside the package tsconfig's include list, so type-aware lint checks them in an inferred
// program that loads no ambient Node types; this reference loads them, so `process` and `node:` imports stay typed
// in this file on its own instead of depending on a sibling module that carries the same reference.
/// <reference types="node" />
import {
  join,
  resolve,
} from 'node:path';

import { vendorLockedDependencies } from './native-scanner-snapshot.ts';

/** Directory below the build context that holds the linter stage's repository-shaped sources. */
export const linterContextDirectory = 'linter-stage';

/** Where the test image holds the linter, outside every PATH a test builds, so only an explicit path reaches it. */
export const imageLinterPath = '/opt/monochromatic-lint/monochromatic-lint';

/**
 Variable naming the linter executable inside the test image. The tests read it, never the wrapper:
 a test that needs the real linter puts this exact file on the PATH it builds, and fails when the variable is unset.
 */
export const linterVariable = 'GIT_POLICY_NATIVE_TEST_LINTER';

/**
 One file or directory to copy from the repository into the build context.

 @typedef {{ from: string, to: string }} SnapshotEntry
 */

/**
 Copy entries for the linter crate and its JSONC dependency, resolved from the wrapper package directory the gate
 runs in, into the linter stage's own directory of the build context.

 @param {{ context: string }} request - build context directory receiving the linter stage's sources
 @returns {SnapshotEntry[]} entries keeping the repository layout, so the linter's relative path dependency resolves;
   a library and executable build of the linter reads only its manifest, lockfile and `src`
 */
export function linterSnapshotEntries({ context }) {
  /** @type {SnapshotEntry[]} */
  const entries = [];
  for (const [crate, names] of /** @type {const} */ ([
    [
      'linter/monochromatic-lint',
      [
        'Cargo.toml',
        'Cargo.lock',
        'src',
      ],
    ],
    [
      'jsonc-edit/core-rust',
      [
        'Cargo.toml',
        'Cargo.lock',
        'src',
      ],
    ],
  ])) {
    for (const name of names) {
      entries.push({
        from: resolve(
          '../..',
          crate,
          name,
        ),
        to: join(
          context,
          linterContextDirectory,
          'package',
          crate,
          name,
        ),
      });
    }
  }
  return entries;
}

/**
 Vendor the linter's locked registry dependencies into its stage directory, with the source mapping that stage reads.
 Every archive the linter's lockfile names must already be in the host's Cargo cache.

 @param {{ context: string }} request - build context whose linter stage directory already holds the copied crates
 */
export async function vendorLinterDependencies({ context }) {
  await vendorLockedDependencies({
    context: join(
      context,
      linterContextDirectory,
    ),
    manifest: 'package/linter/monochromatic-lint/Cargo.toml',
  });
}

/**
 Containerfile lines of the stage that builds the linter. The stage starts from the same base as the test image,
 so the executable links against the C library it runs with. It uses the development profile without debug
 information, which the linter's own container suite shows fits the 2 GiB build bound; nothing here runs a test.

 @param {{ base: string }} request - content-addressed base image ID shared with the test image
 @returns {string[]} Containerfile lines ending with the built executable at a fixed stage path
 */
export function linterStageLines({ base }) {
  return [
    `FROM ${base} AS linter`,
    `COPY --chown=1000:1000 ${linterContextDirectory}/package /work/package`,
    `COPY --chown=1000:1000 ${linterContextDirectory}/vendor /work/vendor`,
    `COPY --chown=1000:1000 ${linterContextDirectory}/cargo-config /work/.cargo`,
    'RUN ["mkdir", "--parents", "/home/tester/.cargo"]',
    'RUN ["chown", "--recursive", "1000:1000", "/home/tester"]',
    'ENV HOME=/home/tester CARGO_HOME=/home/tester/.cargo CARGO_NET_OFFLINE=true CARGO_BUILD_JOBS=2 CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0',
    'USER 1000:1000',
    'WORKDIR /work/package/linter/monochromatic-lint',
    'RUN ["cargo", "build", "--offline", "--locked", "--bin", "monochromatic-lint"]',
  ];
}

/**
 Containerfile lines of the test image that copy the built linter in and name it for the tests.

 @returns {string[]} lines to place in the test stage before it switches to the unprivileged user
 */
export function linterCopyLines() {
  return [
    `COPY --from=linter /work/package/linter/monochromatic-lint/target/debug/monochromatic-lint ${imageLinterPath}`,
    `ENV ${linterVariable}=${imageLinterPath}`,
  ];
}
