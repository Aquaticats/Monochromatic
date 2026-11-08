/**
 Containerfile text of the gate's test image: one test stage whose compile is
 split into layers, so later runs rebuild only the layer a change reaches.

 The test stage orders its layers from the input that changes least to the one that changes most:
 the registry dependencies (manifests, lock files and vendored sources), the path crates the wrapper links
 (the scanner and the JSONC crate), and the wrapper's own sources, the only layer a test or source edit reaches.
 Every build step costs a fixed amount on a busy host, so the stage keeps its step count low as well.
 */
import {
  imagePackageRoot,
  imageRootDirectory,
  touchedCargoRun,
  builderPreludeLines,
} from './native-layered-build.ts';
import {
  pathCrates,
  subjectCrate,
} from './native-wrapper-snapshot.ts';

/**
 Top-level directories of `package` the path-crate layer copies, one `COPY` step each: crates share
 a step when their paths share a top-level directory.

 @returns each distinct top-level directory once, in crate order
 */
function pathCrateRoots(): readonly string[] {
  return [...new Set(pathCrates.map(function topDirectory(crate) {
    return crate.split('/')[0] ?? crate;
  }))];
}

/**
 Cargo arguments that run every target of the wrapper, the image's default command, and the prefix of its compile.
 */
export const runArguments = [
  'cargo',
  'nextest',
  'run',
  '--offline',
  '--locked',
  '--all-targets',
  '--no-fail-fast',
  // A control that needs btrfs prints one line naming itself and the filesystem it found, then passes; that line is
  // captured output of a successful test, which nextest hides for a green run unless it is asked for. Without this the
  // gate reads as full coverage while a hundred and twenty controls did not run (see `CLI_GIT_BTRFS_ABSENT_ACKNOWLEDGED`).
  '--success-output=final',
] as const;

/**
 Cargo arguments that compile every target of the wrapper without running any. A build failure always stops the
 build, so the failure option does not apply here.
 */
const compileArguments = [
  ...runArguments.slice(1)
    .filter(function keepsBuildOption(argument) {
      return argument !== '--no-fail-fast' && argument !== '--success-output=final';
    }),
  '--no-run',
] as const;

/**
 Containerfile of the test image, built without network from a content-addressed base.

 @param base - content-addressed base image ID shared by every stage

 @param jobs - parallel compile jobs the build container's CPUs allow

 @param settingsFile - name of the repository's settings file at the repository root, copied beside `clippy.toml`

 @returns the Containerfile text, ending in a newline

 @example
 ```ts
 const text = testContainerfile({ base, jobs: 2, settingsFile: 'cli-git.config.jsonc' });
 ```
 */
export function testContainerfile({
  base,
  jobs,
  settingsFile,
}: {
  readonly base: string;
  readonly jobs: number;
  readonly settingsFile: string;
},): string {
  return [
    `FROM ${base}`,
    // One environment step. A test that waits on a lock nothing releases (a mutant that never releases one) fails
    // after 20 seconds instead of waiting out the 30 minute product limit, which cargo-mutants would record as a timeout.
    ...builderPreludeLines({
      environment: [
        'HOME=/home/tester',
        'CARGO_HOME=/home/tester/.cargo',
        'CARGO_NET_OFFLINE=true',
        `CARGO_BUILD_JOBS=${String(jobs)}`,
        'CARGO_PROFILE_DEV_DEBUG=0',
        'CARGO_INCREMENTAL=0',
        // The gate runs on an overlay filesystem, where the btrfs controls cannot run. They fail loudly unless the run
        // states that this is known; with it they print one line per control that did not run. `native:test:host`
        // runs them on btrfs and sets nothing.
        'CLI_GIT_BTRFS_ABSENT_ACKNOWLEDGED=1',
      ].join(' '),
    }),
    // Some crate archives carry files only their owner may read, so the tester must own the vendored copy.
    // The stand-in crates, the vendored sources and the Cargo mapping arrive in one step.
    `COPY --chown=1000:1000 ${imageRootDirectory} /work`,
    `WORKDIR ${imagePackageRoot}/${subjectCrate}`,
    // nextest's default concurrency: one test per CPU the container may use. `--no-fail-fast` runs every test after a
    // failure too, so one flaky test never hides the rest; the exit status stays non-zero and nextest prints its
    // summary and the failed-test list last.
    `CMD ${JSON.stringify([...runArguments])}`,
    // Dependency layer: compiles every registry crate once, from the stand-ins. Its cache key holds no source file.
    `RUN ${JSON.stringify([
      'cargo',
      ...compileArguments
    ])}`,
    `COPY clippy.toml ${settingsFile} /work/`,
    // Path-crate layer: the scanner's build script and the matcher compile here, and again only when they change.
    ...pathCrateRoots()
      .map(function pathCrateCopy(root) {
        return `COPY --chown=1000:1000 package/${root} ${imagePackageRoot}/${root}`;
      }),
    touchedCargoRun({
      touchedPaths: pathCrates.map(function pathCrateDirectory(crate) {
        return `${imagePackageRoot}/${crate}`;
      }),
      cargoArguments: compileArguments,
    }),
    // Subject layer: the wrapper's sources and the README its tests include. The test run, each planted control and
    // each mutant then rebuild only what changed, instead of the linked scanner and its dependencies every time.
    `COPY --chown=1000:1000 package/${subjectCrate} ${imagePackageRoot}/${subjectCrate}`,
    touchedCargoRun({
      touchedPaths: [
        `${imagePackageRoot}/${subjectCrate}/src`,
        `${imagePackageRoot}/${subjectCrate}/README.md`,
      ],
      cargoArguments: compileArguments,
    }),
    '',
  ].join('\n');
}
