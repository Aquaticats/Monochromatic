#!/usr/bin/env node
/**
 Build the instrumented codec target, then fuzz it for a bounded time in a
 container that has no network, no home folder, no device and no mount of the
 host's Rust sysroot.

 The image installs its own nightly toolchain and cargo-fuzz at build time,
 because the repository forbids mounting the host's sysroot into a container
 (that mix produced Clippy's E0514 on this host). Test runs then use
 `--network none`; only this image build may reach the network.
 */
import { execFileSync } from 'node:child_process';
import {
  cpSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from 'node:fs';
import { join, resolve } from 'node:path';

/** This package's directory, the working directory of both tasks. */
const source = process.cwd();
/** The crate under fuzz. */
const subject = resolve(source, '..', 'request-to-openzfs-kernel-device');
/** Repository root, four directories above this package. */
const repository = resolve(source, '..', '..', '..');
/** Gitignored scratch shared by the repository's campaigns, outside both copied trees. */
const scratch = join(repository, 'node_modules', '.monochromatic', 'zfs-binding-fuzz');
/** Where the fuzzer's failing inputs land, mounted into the run so they survive it. */
const artifacts = join(scratch, 'artifacts');
/** The corpus, mounted into the run so a later run starts from what earlier ones found. */
const corpus = join(scratch, 'corpus');
/** Image tag; concurrent worktrees set ZFS_BINDING_IMAGE_TAG so one run never reuses another's image. */
const tag = process.env.ZFS_BINDING_IMAGE_TAG ?? 'step-1';
/** The image holding the nightly toolchain, cargo-fuzz and both crates. */
const image = `localhost/zfs-request-to-openzfs-kernel-device-fuzz:${tag}`;
/** Seconds the target runs; a smoke bound, never a campaign. */
const seconds = 30;
/** The repository's toolchain lock, which names the nightly the image installs. */
const toolchainLock = JSON.parse(readFileSync(join(repository, 'toolchain.lock.json'), 'utf8'));
/** The nightly the lock pins, for example `nightly-2026-11-16`. */
const nightly = toolchainLock.tools.rust.version;
/** The date part of the nightly, the directory `static.rust-lang.org` publishes it under. */
const nightlyDate = nightly.startsWith('nightly-') ? nightly.slice('nightly-'.length) : null;
if (!nightlyDate) {
  throw new Error(`the toolchain lock names \`${nightly}\`, which is not a nightly release`);
}
/** The published nightly archive, unpacked rather than built or mounted from the host. */
const nightlyArchive = `https://static.rust-lang.org/dist/${nightlyDate}/rust-nightly-x86_64-unknown-linux-gnu.tar.xz`;

/**
 Run one command with its output on this terminal.

 @param command - program

 @param args - arguments
 */
function run(command, args) {
  execFileSync(command, args, { stdio: 'inherit', cwd: source });
}

/**
 The Containerfile, written into the disposable build context.
 */
const containerfile = [
  "# Built from the wrapper's gate base image, so Rust and cargo-nextest are",
  '# pacman current packages, exactly as the gate image has them.',
  'FROM localhost/git-policy-native-test-base:6fa67475da3a-97221c29742b',
  "# cargo-fuzz is pacman's cargo-fuzz in this image's repositories.",
  'RUN ["pacman", "--disable-sandbox", "--sync", "--refresh", "--noconfirm", "--needed", "cargo-fuzz"]',
  '# The nightly toolchain is the compiler the instrumented build needs. Its',
  '# date and name come from toolchain.lock.json, and it is the published',
  '# nightly archive, unpacked here rather than mounted from the host and',
  '# never built from source; rustup would conflict with the image\'s pacman rust.',
  `RUN ["curl", "--fail", "--silent", "--show-error", "--location", "--output", "/tmp/rust-nightly.tar.xz", "${nightlyArchive}"]`,
  'RUN ["tar", "--extract", "--xz", "--file", "/tmp/rust-nightly.tar.xz", "--directory", "/tmp"]',
  'RUN ["/tmp/rust-nightly-x86_64-unknown-linux-gnu/install.sh", "--prefix=/opt/rust-nightly", "--without=rust-docs"]',
  'ENV PATH=/opt/rust-nightly/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin',
  'COPY subject /work/package/zfs/request-to-openzfs-kernel-device',
  'COPY sidecar /work/package/zfs/request-to-openzfs-kernel-device.fuzz',
  'WORKDIR /work/package/zfs/request-to-openzfs-kernel-device.fuzz',
  '# The fetch needs the network; every later run is offline.',
  'RUN ["cargo", "fetch", "--locked"]',
  'ENV CARGO_NET_OFFLINE=true',
  'RUN ["sh", "-c", "cargo --version && rustc --version && cargo-fuzz --version"]',
  '',
].join('\n');

/**
 Copy the two crates into a fresh build context and build the image.
 */
function buildImage() {
  mkdirSync(scratch, { recursive: true });
  const context = mkdtempSync(join(scratch, 'context-'));
  const withoutBuildState = (entry) =>
    !entry.includes('/node_modules/') &&
    !entry.endsWith('/target') &&
    !entry.includes('/target/');
  cpSync(subject, join(context, 'subject'), { recursive: true, filter: withoutBuildState });
  cpSync(source, join(context, 'sidecar'), { recursive: true, filter: withoutBuildState });
  writeFileSync(join(context, 'Containerfile'), containerfile);
  run('podman', [
    'build',
    '--network', 'host',
    '--cpu-period', '100000',
    '--cpu-quota', '200000',
    '--memory', '2g',
    '--tag', image,
    '--file', join(context, 'Containerfile'),
    context,
  ]);
}

/**
 Run the target in a bounded container with no network and no host home folder.

 The corpus and artifacts directories are mounted from {@link scratch},
 so a failing input outlives the container that found it.

 @param buildOnly - stop after the instrumented build, keeping the binaries
 */
function runContainer(buildOnly) {
  const containerName = 'zfs-binding-fuzz';
  run('podman', ['rm', '--force', '--ignore', containerName]);
  mkdirSync(artifacts, { recursive: true });
  mkdirSync(corpus, { recursive: true });
  const inside = (name) => `/work/package/zfs/request-to-openzfs-kernel-device.fuzz/${name}`;
  const command = buildOnly
    ? ['cargo', 'fuzz', 'build', '--fuzz-dir', '.']
    : [
      'sh',
      '-c',
      `mkdir --parents corpus/nvlist_codec artifacts/nvlist_codec && cargo fuzz run --fuzz-dir . nvlist_codec corpus/nvlist_codec -- -max_total_time=${seconds} -rss_limit_mb=2048`,
    ];
  run('podman', [
    'run',
    '--rm',
    '--name', containerName,
    '--init',
    '--network', 'none',
    '--cpus', '2',
    '--memory', '2g',
    '--pids-limit', '512',
    '--blkio-weight', '10',
    '--volume', `${artifacts}:${inside('artifacts')}:Z`,
    '--volume', `${corpus}:${inside('corpus')}:Z`,
    image,
    ...command,
  ]);
}

/**
 The evidence lines a run records: the compiler and the fuzzer it used.
 */
function reportVersions() {
  run('podman', [
    'run',
    '--rm',
    '--network', 'none',
    '--cpus', '1',
    '--memory', '1g',
    '--pids-limit', '256',
    '--blkio-weight', '10',
    image,
    'sh',
    '-c',
    'rustc --version && cargo --version && cargo-fuzz --version',
  ]);
}

if (!existsSync(join(subject, 'Cargo.lock'))) {
  throw new Error('the crate has no Cargo.lock; run the crate\'s lock task first');
}
if (!existsSync(join(source, 'Cargo.lock'))) {
  throw new Error('the sidecar has no Cargo.lock; run the sidecar\'s lock task first');
}
buildImage();
reportVersions();
runContainer(process.argv.includes('--build-only'));
console.log(`A failing input and the growing corpus stay in ${scratch}`);
