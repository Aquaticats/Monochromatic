/**
 Snapshot inputs for the Markdown linter the native wrapper's `markdown/autofix` policy starts as a child process:
 the linter crate's sources, the JSONC crate it shares with the wrapper, and the registry dependencies its own
 lockfile names.

 The gate image builds the linter from these sources in a stage of its own, on the same audited base as the
 wrapper, and copies only the executable into the test image. The container has no network, so the registry
 sources are vendored on the host from the Cargo cache, as the scanner's are. A binary built on the host is never
 copied in: the host's C library is newer than the image's.
 */
import {
  join,
  resolve,
} from 'node:path';

import {
  builderPreludeLines,
  imagePackageRoot,
  imageRootDirectory,
  touchedCargoRun,
} from './native-layered-build.ts';
import { vendorLockedDependencies } from './native-scanner-snapshot.ts';

/**
 Directory below the build context that holds the linter stage's repository-shaped sources.
 */
export const linterContextDirectory = 'linter-stage';

/**
 Where the test image holds the linter, outside every PATH a test builds, so only an explicit path reaches it.
 */
export const imageLinterPath = '/opt/monochromatic-lint/monochromatic-lint';

/**
 Variable naming the linter executable inside the test image. The tests read it, never the wrapper:
 a test that needs the real linter puts this exact file on the PATH it builds, and fails when the variable is unset.
 */
export const linterVariable = 'GIT_POLICY_NATIVE_TEST_LINTER';

/**
 One file or directory to copy from the repository into the build context.
 */
export type SnapshotEntry = {
  readonly from: string;
  readonly to: string;
};

/**
 Crates of the linter stage, as directories below `package`: the linter and the JSONC crate it shares with the wrapper.
 */
export const linterCrates = [
  'linter/monochromatic-lint',
  'jsonc-edit/core-rust',
] as const;

/**
 Entries of each linter-stage crate that a library and executable build reads: its manifest, lockfile and `src`.
 */
const linterCrateEntries = [
  'Cargo.toml',
  'Cargo.lock',
  'src',
] as const;

/**
 Copy entries for the linter crate and its JSONC dependency, resolved from the wrapper package directory the gate
 runs in, into the linter stage's own directory of the build context.

 @param context - build context directory receiving the linter stage's sources

 @returns entries keeping the repository layout, so the linter's relative path dependency resolves;
   a library and executable build of the linter reads only its manifest, lockfile and `src`

 @example
 ```ts
 const entries = linterSnapshotEntries({ context });
 ```
 */
export function linterSnapshotEntries({ context, }: {
  readonly context: string;
},): readonly SnapshotEntry[] {
  return linterCrates.flatMap(function crateEntries(crate) {
    return linterCrateEntries.map(function crateEntry(name) {
      return {
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
      };
    });
  });
}

/**
 Vendor the linter's locked registry dependencies into its stage directory, with the source mapping that stage reads.
 Every archive the linter's lockfile names must already be in the host's Cargo cache.

 @param context - build context whose linter stage directory already holds the copied crates

 @example
 ```ts
 await vendorLinterDependencies({ context });
 ```
 */
export async function vendorLinterDependencies({ context, }: {
  readonly context: string;
},): Promise<void> {
  await vendorLockedDependencies({
    context: join(
      context,
      linterContextDirectory,
    ),
    manifest: 'package/linter/monochromatic-lint/Cargo.toml',
    vendorName: `${imageRootDirectory}/vendor`,
    configName: `${imageRootDirectory}/.cargo`,
  });
}

/**
 Cargo environment of the linter stage: the unprivileged user's home, no network, two compile jobs, no debug
 information and no incremental state.
 */
const linterBuildEnvironment = 'HOME=/home/tester CARGO_HOME=/home/tester/.cargo CARGO_NET_OFFLINE=true CARGO_BUILD_JOBS=2 CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0';

/**
 Containerfile lines of the stage that builds the linter. The stage starts from the same base as the test image,
 so the executable links against the C library it runs with. It uses the development profile without debug
 information, which the linter's own container suite shows fits the 2 GiB build bound; nothing here runs a test.

 The registry dependencies compile in a layer keyed on the manifests, the lock files and the vendored sources only,
 from empty stand-ins for the crates' targets; the real sources are copied afterwards, so a change to the linter or
 JSONC sources recompiles those two crates and nothing else.

 @param base - content-addressed base image ID shared with the test image

 @returns Containerfile lines ending with the built executable at a fixed stage path

 @example
 ```ts
 const lines = linterStageLines({ base });
 ```
 */
export function linterStageLines({ base, }: {
  readonly base: string;
},): string[] {
  /**
   Where the linter crate sits in the image.
   */
  const crateDirectory = `${imagePackageRoot}/${linterCrates[0]}`;
  /**
   Cargo arguments of the linter build.
   */
  const buildArguments = [
    'build',
    '--offline',
    '--locked',
    '--bin',
    'monochromatic-lint',
  ];
  return [
    `FROM ${base} AS linter`,
    ...builderPreludeLines({ environment: linterBuildEnvironment }),
    `COPY --chown=1000:1000 ${linterContextDirectory}/${imageRootDirectory} /work`,
    `WORKDIR ${crateDirectory}`,
    `RUN ${JSON.stringify(['cargo', ...buildArguments])}`,
    `COPY --chown=1000:1000 ${linterContextDirectory}/package ${imagePackageRoot}`,
    touchedCargoRun({
      touchedPaths: linterCrates.map(function sourceDirectory(crate) {
        return `${imagePackageRoot}/${crate}/src`;
      }),
      cargoArguments: buildArguments,
    }),
  ];
}

/**
 Containerfile lines of the test image that copy the built linter in.

 @returns lines to place in the test stage once the unprivileged user exists

 @example
 ```ts
 const lines = linterCopyLines();
 ```
 */
export function linterCopyLines(): string[] {
  return [`COPY --from=linter ${imagePackageRoot}/${linterCrates[0]}/target/debug/monochromatic-lint ${imageLinterPath}`];
}

/**
 Environment assignment that names the linter for the tests, for the test stage's single `ENV` step.
 */
export const linterEnvironment: string = `${linterVariable}=${imageLinterPath}`;
