/**
 Containerfile text of the gate's test image: the linter and `strace` stages, then the source snapshot, the vendored
 crates, and a first offline compile so later runs rebuild only what changed.
 */
import {
  linterCopyLines,
  linterStageLines,
} from './native-linter-snapshot.ts';
import { nextestCopyLines, } from './native-nextest-snapshot.ts';
import {
  straceCopyLines,
  straceStageLines,
} from './native-strace-snapshot.ts';

/**
 Containerfile of the test image, built without network from a content-addressed base.

 @param base - content-addressed base image ID shared by every stage

 @param jobs - parallel compile jobs the build container's CPUs allow

 @returns the Containerfile text, ending in a newline

 @example
 ```ts
 const text = testContainerfile({ base, jobs: 2 });
 ```
 */
export function testContainerfile({
  base,
  jobs,
}: {
  readonly base: string;
  readonly jobs: number;
},): string {
  return [
    // The Markdown policy's linter is built from this snapshot in its own stage; only its executable is kept.
    ...linterStageLines({ base }),
    // The fault-injection controls run the wrapper under `strace`; without it they report and check nothing.
    ...straceStageLines({
      base,
      jobs,
    }),
    `FROM ${base}`,
    ...nextestCopyLines(),
    ...straceCopyLines(),
    ...linterCopyLines(),
    'COPY package /work/package',
    // Some crate archives carry files only their owner may read, so the tester must own the vendored copy.
    'COPY --chown=1000:1000 vendor /work/vendor',
    'COPY --chown=1000:1000 cargo-config /work/.cargo',
    'COPY clippy.toml /work/clippy.toml',
    'COPY cli-git.config.jsonc /work/cli-git.config.jsonc',
    'RUN ["mkdir", "--parents", "/home/tester/.cargo"]',
    'RUN ["chown", "--recursive", "1000:1000", "/work/package", "/home/tester"]',
    `ENV HOME=/home/tester CARGO_HOME=/home/tester/.cargo CARGO_NET_OFFLINE=true CARGO_BUILD_JOBS=${String(jobs)} CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0`,
    // A test that waits on a lock nothing releases (a mutant that never releases one) fails after 20 seconds
    // instead of waiting out the 30 minute product limit, which cargo-mutants would record as a timeout.
    'ENV CLI_GIT_LOCK_WAIT_SECONDS=20 CLI_GIT_PUSH_LOCK_WAIT_SECONDS=20',
    'USER 1000:1000',
    'WORKDIR /work/package/git-policy/cli',
    // Compile once while the image is built. The test run, each planted control and each mutant then rebuild
    // only what changed, instead of the linked scanner and its dependencies every time.
    'RUN ["cargo", "nextest", "run", "--offline", "--locked", "--all-targets", "--no-run"]',
    // nextest's default concurrency: one test per CPU the container may use.
    'CMD ["cargo", "nextest", "run", "--offline", "--locked", "--all-targets"]',
    '',
  ].join('\n');
}
