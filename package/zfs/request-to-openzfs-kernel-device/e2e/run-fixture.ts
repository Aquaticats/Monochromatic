// Recipe: the throwaway mvm guest the ZFS binding's live tests run in.
//
// What:  creates an Ubuntu 24.04 guest, installs the tools the plan names, gives
//        the kernel build directory the kernel's own BTF so the module is built
//        with BTF, builds OpenZFS 2.4.4 from the release tarball against the
//        guest's own kernel headers, builds this crate inside the guest, runs the
//        live suites as root and as the delegated account (each failing when a live
//        arm skipped), records versions and teardown evidence, and destroys the
//        guest on request.
// Why the digests: the tarball and the rustup installer are checked against the
//        sha256 this file pins, so the guest is rebuilt from the same bytes.
// Why:   a live request opens /dev/zfs, so it must never run on the host; this guest
//        is the only place the crate's live tests run, and its pools are 1 GiB
//        file-backed images it destroys with each test.
// Why built inside the guest: the host's glibc is newer than Ubuntu 24.04's, so a
//        host-built binary would not start there; building in the guest keeps the
//        test binary matched to the guest's own libc and needs no cross toolchain.
//
// Usage: node e2e/run-fixture.ts <create|prepare|run|record|destroy|all>
//          [--name <guest>] [--log-dir <dir>] [--crate <dir>] [--keep-guest]
// Every step writes one log per command under the log directory and prints it.

import { spawnSync, } from 'node:child_process';
import { closeSync, mkdirSync, openSync, } from 'node:fs';
import { join, } from 'node:path';

//region Constants

/** The guest user mvm's Ubuntu image creates; the delegated arms run as it. */
const ACCOUNT = 'ubuntu';
/** Where OpenZFS 2.4.4 is installed inside the guest. */
const OPENZFS_PREFIX = '/opt/openzfs-2.4.4';
/** Where the release tarball is unpacked inside the guest. */
const OPENZFS_SOURCE = '/root/zfs-2.4.4';
/** The release the design pins, which the module and userspace must report. */
const OPENZFS_RELEASE = '2.4.4-1';
/** The fixture's own directory inside the guest, above its pools' images. */
const FIXTURE_ROOT = '/home/zfs-step-2-fixture';
/**
 Where the crate is unpacked and built inside the guest.
 `/root` is unreadable to the delegated account,
 so the tree lives under `/opt`,
 which every account may traverse.
 */
const CRATE_DIR = '/opt/zfs-step-2-crate';
/** The pool the delegated-account arms use, which the root suite prepares. */
const ACCOUNT_POOL = 'zfsstep2-account';
/**
 The step 3 arm's own fixture root:
 the file-backed pool, its three datasets and the empty snapshot the wrapper's ZFS controls clone.
 It is separate from the step 2 root so neither suite can read the other's leftovers.
 */
const STEP3_FIXTURE_ROOT = '/home/zfs-step-3-fixture';
/** The step 3 pool, whose counters the wrapper's measurement reads. */
const STEP3_POOL = 'zfsstep3';
/** The snapshot of the empty checkout dataset the wrapper's controls clone. */
const STEP3_CLONE_SNAPSHOT = `${STEP3_POOL}/checkout@step-3-clone`;
/**
 Where the wrapper's packages are unpacked inside the guest.
 The paths below keep their repository-relative place,
 because the packages depend on each other by `../../` paths.
 */
const STEP3_WRAPPER_ROOT = '/opt/zfs-step-3-repo';
/** The wrapper crate itself. */
const STEP3_WRAPPER_DIR = `${STEP3_WRAPPER_ROOT}/package/git-policy/cli`;
/** The wrapper executable the arm starts as `git`. */
const STEP3_WRAPPER = `${STEP3_WRAPPER_DIR}/target/release/monochromatic-git-policy-cli`;
/**
 The wrapper crate's own packages and the ones its manifests name by path,
 which are the whole tree the guest needs:
 a crate whose manifest names `../../` must find those directories at the same relative place.
 */
const STEP3_WRAPPER_PACKAGES: readonly string[] = [
  'package/git-policy/cli',
  'package/catalog-specifier/core-rust',
  'package/forbidden-strings/cli',
  'package/forbidden-strings/core',
  'package/jsonc-edit/core-rust',
  'package/linter/monochromatic-lint',
];
/** The `PATH` the guest's OpenZFS 2.4.4 userspace needs. */
const OPENZFS_PATH = `${OPENZFS_PREFIX}/sbin:${OPENZFS_PREFIX}/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin`;
/**
 The guest kernel release the plan pins until step 23 supplies the host kernel.
 The `mvm` Ubuntu 24.04 image boots it,
 and the module BTF the kernel exposes at
 {@link MODULE_BTF} exists only for the
 kernel whose headers the module is built against,
 so a moved image is refused here rather than
 measured under an unrecorded release.
 */
const EXPECTED_KERNEL_RELEASE = '6.8.0-147-generic';
/** The running kernel's own BTF, which module BTF is built against. */
const KERNEL_BTF = '/sys/kernel/btf/vmlinux';
/** Where the running kernel exposes each loaded module's BTF. */
const MODULE_BTF = '/sys/kernel/btf/zfs';
/**
 sha256 of `zfs-2.4.4.tar.gz`,
 from the project's signed `zfs-2.4.4.sha256.asc` asset
 (`https://github.com/openzfs/zfs/releases/download/zfs-2.4.4/zfs-2.4.4.sha256.asc`).
 */
const OPENZFS_TARBALL_SHA256 = '2a3c70d55a37cc71618a95a60e81ad66530201eb118d37741dc92efcf848c8b1';
/**
 The rustup release the installer comes from.
 Pinned: {@link RUSTUP_INIT_SHA256} is that release's `rustup-init` digest,
 so an unpinned download would
 need a new digest with every rustup release.
 */
const RUSTUP_VERSION = '1.29.3';
/** The pinned installer's own archive path. */
const RUSTUP_INIT_URL = `https://static.rust-lang.org/rustup/archive/${RUSTUP_VERSION}/x86_64-unknown-linux-gnu/rustup-init`;
/** sha256 of that installer (`rustup-init.sha256` beside it in the same release). */
const RUSTUP_INIT_SHA256 = 'b1c8cb14e30889e7e2999e6b69e62d9e9b9ffc0f4df8180564d306d9680e6c55';
/**
 The stable Rust release the guest installs.
 Pinned: it is the release the 2026-11-19 review-fix guest run built this crate
 with, as its `versions.log` records,
 so a newer stable is a deliberate move of this
 constant rather than an unrecorded change between runs.
 */
const RUST_TOOLCHAIN = '1.100.0';
/**
 The `cargo-nextest` release the guest installs.
 Pinned: it is the release the same guest run recorded,
 and `versions.log` proves what a
 run used.
 */
const NEXTEST_VERSION = '0.9.152';
/**
 Milliseconds one host command may take:
 a create,
 a push,
 a pull or the tarball.
 */
const HOST_COMMAND_TIMEOUT_MS = 120_000;
/** Seconds one guest download may take before `curl` gives up. */
const DOWNLOAD_TIMEOUT_SECONDS = 300;
/** How often `curl` retries a download that failed mid-transfer. */
const DOWNLOAD_RETRIES = 2;
/** Milliseconds the package install may take on a cold mirror. */
const PREPARE_PACKAGES_TIMEOUT_MS = 1_900_000;
/** Milliseconds the OpenZFS userspace and module build may take. */
const PREPARE_BUILD_TIMEOUT_MS = 2_500_000;
/** Milliseconds the Rust toolchain and `cargo-nextest` install may take. */
const PREPARE_TOOLCHAIN_TIMEOUT_MS = 1_600_000;
/** Milliseconds the crate's release build inside the guest may take. */
const CRATE_BUILD_TIMEOUT_MS = 1_500_000;
/** Milliseconds one live suite may take. */
const SUITE_TIMEOUT_MS = 1_200_000;
/** Milliseconds the version and teardown records may take. */
const RECORD_TIMEOUT_MS = 180_000;
/**
 Seconds the guest's own `timeout` fires before mvm's kill,
 so an overrun is written to
 the guest's log instead of losing the transport.
 */
const GUEST_TIMEOUT_MARGIN_SECONDS = 60;

/** The Debian packages the guest needs: the build tools, `pahole`, `strace`, Git. */
const PACKAGES = [
  'acl',
  'autoconf',
  'automake',
  'bison',
  'build-essential',
  'curl',
  'dwarves',
  'flex',
  'gawk',
  'git',
  'libaio-dev',
  'libattr1-dev',
  'libblkid-dev',
  'libelf-dev',
  'libssl-dev',
  'libtirpc-dev',
  'libtool',
  'libudev-dev',
  'libzstd-dev',
  'pkg-config',
  'python3-packaging',
  'strace',
  'uuid-dev',
  'zlib1g-dev',
].join(' ');

//endregion Constants

//region Arguments

interface Options {
  readonly step: string;
  readonly name: string;
  readonly logDir: string;
  readonly crate: string;
  readonly wrapperCrate: string;
  readonly keepGuest: boolean;
}

/** The step, the guest name, the log directory, the crate source directory and whether `all` keeps the guest. */
function options(): Options {
  const [step = '', ...rest] = process.argv.slice(2);
  const value = (flag: string): string | undefined => {
    const at = rest.indexOf(flag);
    return at === -1 ? undefined : rest[at + 1];
  };
  const packageDir = join(import.meta.dirname, '..');
  return {
    step,
    name: value('--name') ?? 'zfs-step-2-fixture',
    logDir: value('--log-dir') ?? join(packageDir, 'target', 'e2e-logs'),
    crate: value('--crate') ?? packageDir,
    wrapperCrate: value('--wrapper-repository') ?? join(import.meta.dirname, '../../../..'),
    keepGuest: rest.includes('--keep-guest'),
  };
}

//endregion Arguments

//region Commands

/** Quote one block of shell for the guest's `sh -ec`, quotes included. */
function shellQuote(script: string): string {
  return `'${script.replaceAll("'", `'\\''`)}'`;
}

/** One host command whose whole output goes to its own log. */
interface HostCommand {
  readonly options: Options;
  readonly command: string;
  readonly args: readonly string[];
  readonly log: string;
}

/** Run one host command whose whole output goes to its own log. */
function host({ options, command, args, log }: HostCommand): void {
  const output = openSync(join(options.logDir, log), 'w', 0o600);
  try {
    const result = spawnSync(command, args, {
      stdio: ['ignore', output, output],
      timeout: HOST_COMMAND_TIMEOUT_MS,
    });
    if (result.error || result.status !== 0)
      throw result.error ?? new Error(`${command} ${args.join(' ')} exited ${result.status}; see ${log}`);
  } finally {
    closeSync(output);
  }
}

/** One guest script, its log and the time it may take. */
interface GuestScript {
  readonly options: Options;
  readonly script: string;
  readonly log: string;
  readonly timeout: number;
}

/** Run one guest script through the guest agent, its output in its own log. */
function guest({ options, script, log, timeout }: GuestScript): void {
  const output = openSync(join(options.logDir, log), 'w', 0o600);
  const command =
    `test "$(hostname)" = ${options.name} && timeout ${Math.floor(timeout / 1000) - GUEST_TIMEOUT_MARGIN_SECONDS} sh -ec ${shellQuote(script)}`;
  try {
    const result = spawnSync(
      'mvm',
      ['--backend', 'libvirt', 'exec', options.name, '--', command],
      { stdio: ['ignore', output, output], timeout, },
    );
    if (result.error || result.status !== 0)
      throw result.error ?? new Error(`mvm exec exited ${result.status}; see ${log}`);
  } finally {
    closeSync(output);
  }
}

/** The environment every live suite reads, as shell assignments. */
function fixtureEnvironment(options: Options): string {
  return `MONOCHROMATIC_ZFS_FIXTURE_GUEST=${options.name} ` +
    `MONOCHROMATIC_ZFS_FIXTURE_SBIN=${OPENZFS_PREFIX}/sbin ` +
    `MONOCHROMATIC_ZFS_FIXTURE_ROOT=${FIXTURE_ROOT} ` +
    `MONOCHROMATIC_ZFS_FIXTURE_ACCOUNT=${ACCOUNT} ` +
    `MONOCHROMATIC_ZFS_FIXTURE_ACCOUNT_POOL=${ACCOUNT_POOL} ` +
    `MONOCHROMATIC_ZFS_FIXTURE_SOURCE=${OPENZFS_SOURCE}`;
}

//endregion Commands

//region Steps

/** Create the guest. */
function create(options: Options): void {
  host({ options, command: 'mvm', args: ['--backend', 'libvirt', 'create', options.name], log: 'mvm-create.log' });
}

/** Install the tools, build OpenZFS 2.4.4, and install Rust and nextest. */
function prepare(options: Options): void {
  const install = [
    'set -eu',
    'export DEBIAN_FRONTEND=noninteractive',
    'apt-get --option Acquire::Retries=2 update > /root/zfs-prepare.log 2>&1',
    `apt-get --option Acquire::Retries=2 --yes install ${PACKAGES} "linux-headers-$(uname --kernel-release)" >> /root/zfs-prepare.log 2>&1`,
    `test "$(uname --kernel-release)" = ${EXPECTED_KERNEL_RELEASE}`,
    `curl --fail --location --retry ${DOWNLOAD_RETRIES} --max-time ${DOWNLOAD_TIMEOUT_SECONDS} --output /root/zfs-2.4.4.tar.gz https://github.com/openzfs/zfs/releases/download/zfs-2.4.4/zfs-2.4.4.tar.gz >> /root/zfs-prepare.log 2>&1`,
    `printf '%s  %s\\n' ${OPENZFS_TARBALL_SHA256} /root/zfs-2.4.4.tar.gz | sha256sum --check --strict -`,
    'tar --extract --gzip --file /root/zfs-2.4.4.tar.gz --directory /root',
    `mkdir --parents ${FIXTURE_ROOT}`,
    `chmod 0755 ${FIXTURE_ROOT}`,
    'tail --lines=5 /root/zfs-prepare.log',
  ].join('\n');
  guest({ options, script: install, log: 'guest-prepare.log', timeout: PREPARE_PACKAGES_TIMEOUT_MS });

  const build = [
    'set -eu',
    `cd ${OPENZFS_SOURCE}`,
    // Ubuntu's headers ship no `vmlinux`, and 6.8's scripts/Makefile.modfinal
    // runs `pahole -J --btf_base vmlinux $@` for an out-of-tree module whenever
    // CONFIG_DEBUG_INFO_BTF_MODULES is set (Ubuntu's is): the only missing piece
    // is that file, so the kernel's own BTF is copied into the build directory
    // and the module is built with `.BTF`, which the kernel then exposes at
    // /sys/kernel/btf/zfs for the layout test.
    `mkdir --parents /lib/modules/$(uname --kernel-release)/build`,
    `if test ! -f /lib/modules/$(uname --kernel-release)/build/vmlinux; then cp ${KERNEL_BTF} /lib/modules/$(uname --kernel-release)/build/vmlinux; fi`,
    `./configure --prefix=${OPENZFS_PREFIX} --with-linux=/lib/modules/$(uname --kernel-release)/build > /root/zfs-build.log 2>&1`,
    'make --jobs=4 >> /root/zfs-build.log 2>&1',
    'make --jobs=4 install >> /root/zfs-build.log 2>&1',
    'depmod --all',
    'systemctl stop zfs-zed.service zfs-mount.service zfs-load-module.service 2>/dev/null || true',
    'if test -d /sys/module/zfs; then modprobe --remove zfs || true; fi',
    'if test -d /sys/module/spl; then modprobe --remove spl || true; fi',
    `insmod /lib/modules/$(uname --kernel-release)/extra/spl.ko`,
    `insmod /lib/modules/$(uname --kernel-release)/extra/zfs.ko`,
    `test -f ${MODULE_BTF}`,
    `LD_LIBRARY_PATH=${OPENZFS_PREFIX}/lib PATH=${OPENZFS_PATH} zfs version >> /root/zfs-build.log 2>&1`,
    `test "$(cat /sys/module/zfs/version)" = ${OPENZFS_RELEASE}`,
    'tail --lines=5 /root/zfs-build.log',
  ].join('\n');
  guest({ options, script: build, log: 'guest-build.log', timeout: PREPARE_BUILD_TIMEOUT_MS });

  const toolchain = [
    'set -eu',
    'if test ! -x /root/.cargo/bin/cargo; then',
    `  curl --fail --location --retry ${DOWNLOAD_RETRIES} --max-time ${DOWNLOAD_TIMEOUT_SECONDS} --output /root/rustup-init ${RUSTUP_INIT_URL}`,
    `  printf '%s  %s\\n' ${RUSTUP_INIT_SHA256} /root/rustup-init | sha256sum --check --strict -`,
    '  chmod +x /root/rustup-init',
    `  /root/rustup-init --yes --profile minimal --default-toolchain ${RUST_TOOLCHAIN} > /root/zfs-toolchain.log 2>&1`,
    'fi',
    'if test ! -x /root/.cargo/bin/cargo-nextest; then',
    `  /root/.cargo/bin/cargo install cargo-nextest --locked --version ${NEXTEST_VERSION} >> /root/zfs-toolchain.log 2>&1`,
    'fi',
    '/root/.cargo/bin/cargo --version',
    '/root/.cargo/bin/cargo nextest --version',
  ].join('\n');
  guest({ options, script: toolchain, log: 'guest-toolchain.log', timeout: PREPARE_TOOLCHAIN_TIMEOUT_MS });
}

/** Push the crate, build it inside the guest, and run the root and account suites. */
function run(options: Options): void {
  const tarball = join(options.logDir, 'subject.tar.gz');
  host({
    options,
    command: 'tar',
    args: ['--create', '--gzip', '--file', tarball, '--exclude=target', '--directory', options.crate, '.'],
    log: 'host-package.log',
  });
  host({
    options,
    command: 'mvm',
    args: ['--backend', 'libvirt', 'push', options.name, tarball, '/root/subject.tar.gz'],
    log: 'guest-push.log',
  });
  const build = [
    'set -eu',
    // A virtiofs push lands under /mnt/shared and keeps only the file name; a
    // guest-agent push honours the path. Both routes end with the tarball at /root.
    'if test -f /mnt/shared/subject.tar.gz; then cp /mnt/shared/subject.tar.gz /root/subject.tar.gz; fi',
    `rm --recursive --force ${CRATE_DIR}`,
    `mkdir --parents ${CRATE_DIR}`,
    `tar --extract --gzip --file /root/subject.tar.gz --directory ${CRATE_DIR}`,
    `cd ${CRATE_DIR}`,
    'export PATH=/root/.cargo/bin:$PATH',
    'cargo build --release --tests --locked > /root/zfs-crate-build.log 2>&1',
    'tail --lines=3 /root/zfs-crate-build.log',
  ].join('\n');
  guest({ options, script: build, log: 'guest-crate-build.log', timeout: CRATE_BUILD_TIMEOUT_MS });

  // The root suite excludes the delegated arms, so no live test may skip here:
  // an arm that skipped would be a passing test whose assertions never ran, and
  // `SKIP <test>` at column 0 is what a returning live test prints. The match is
  // `^SKIP live_` because a returning live test names itself `live_...`; the
  // device-free `fixture_tests` prints the host-run notice while proving that
  // branch, and it is not a live arm.
  //
  // The absence check is an explicit `if`, because POSIX `set -e` ignores a
  // command whose status is inverted by `!`, so `! grep --quiet` would never
  // stop the script.
  // `--no-capture` runs the tests serially and prints their output, so the
  // suite's `MEASUREMENT` lines are in the log. `--filterset` (not `-E`) and the
  // long flags below keep every option spelled out.
  const rootSuite = [
    'set -eu',
    `cd ${CRATE_DIR}`,
    'export PATH=/root/.cargo/bin:$PATH',
    `export ${fixtureEnvironment(options)}`,
    'if cargo nextest run --release --locked --run-ignored all --no-fail-fast --status-level all ' +
      '--no-capture --filterset "not test(live_account_tests)" ' +
      '> /root/zfs-root-suite.log 2>&1; then status=0; else status=$?; fi',
    'tail --lines=30 /root/zfs-root-suite.log',
    'test "$status" = 0',
    `if grep --quiet '^SKIP live_' /root/zfs-root-suite.log; then echo 'a live arm returned without running'; exit 1; fi`,
    `grep --quiet '^MEASUREMENT guest kernel=' /root/zfs-root-suite.log`,
    `grep --quiet '^layout zfs_cmd bytes=' /root/zfs-root-suite.log`,
    `grep --quiet '^MEASUREMENT promoted-batch ' /root/zfs-root-suite.log`,
    `grep --quiet '^MEASUREMENT snapshot-ioctl ' /root/zfs-root-suite.log`,
  ].join('\n');
  guest({ options, script: rootSuite, log: 'guest-root-suite.log', timeout: SUITE_TIMEOUT_MS });

  // The account episode is its own script so the delegated run keeps every
  // assertion without a second layer of shell quoting: the suite must pass, no
  // live arm may skip (the same explicit check, for the same `set -e` reason),
  // and a line only a delegated arm prints must be there.
  const accountScript = [
    'set -eu',
    `cd ${CRATE_DIR}`,
    'if cargo nextest run --release --locked --run-ignored all --no-fail-fast --status-level all ' +
      '--no-capture --filterset "test(live_account_tests)" ' +
      `> /home/${ACCOUNT}/zfs-account-suite.log 2>&1; then status=0; else status=$?; fi`,
    `tail --lines=30 /home/${ACCOUNT}/zfs-account-suite.log`,
    'test "$status" = 0',
    `if grep --quiet '^SKIP live_' /home/${ACCOUNT}/zfs-account-suite.log; then echo 'a live arm returned without running'; exit 1; fi`,
    `grep --quiet '^MEASUREMENT account ' /home/${ACCOUNT}/zfs-account-suite.log`,
  ].join('\n');
  const accountSuite = [
    'set -eu',
    `chown --recursive ${ACCOUNT}:${ACCOUNT} ${CRATE_DIR}`,
    `cp --archive /root/.cargo /home/${ACCOUNT}/.cargo`,
    `cp --archive /root/.rustup /home/${ACCOUNT}/.rustup`,
    `chown --recursive ${ACCOUNT}:${ACCOUNT} /home/${ACCOUNT}/.cargo /home/${ACCOUNT}/.rustup`,
    `runuser --preserve-environment --user ${ACCOUNT} -- env HOME=/home/${ACCOUNT} ` +
      `CARGO_HOME=/home/${ACCOUNT}/.cargo RUSTUP_HOME=/home/${ACCOUNT}/.rustup ` +
      `PATH=/home/${ACCOUNT}/.cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin ` +
      `${fixtureEnvironment(options)} ` +
      `sh -ec ${shellQuote(accountScript)}`,
  ].join('\n');
  guest({ options, script: accountSuite, log: 'guest-account-suite.log', timeout: SUITE_TIMEOUT_MS });
}

/** Record versions, the assertions' environment and the teardown evidence. */
function record(options: Options): void {
  const versions = [
    'set -eu',
    '{ uname --kernel-release',
    'cat /sys/module/zfs/version',
    `LD_LIBRARY_PATH=${OPENZFS_PREFIX}/lib PATH=${OPENZFS_PATH} zfs version`,
    'git --version',
    'pahole --version',
    'strace --version | head --lines=1',
    '/root/.cargo/bin/rustc --version',
    '/root/.cargo/bin/cargo --version',
    '/root/.cargo/bin/cargo nextest --version',
    '} | tee /root/zfs-versions.log',
  ].join('\n');
  guest({ options, script: versions, log: 'guest-versions.log', timeout: RECORD_TIMEOUT_MS });
  const teardown = [
    'set -eu',
    '{ echo "pools after the suites:"',
    `LD_LIBRARY_PATH=${OPENZFS_PREFIX}/lib PATH=${OPENZFS_PATH} zpool list -H -o name || true`,
    'echo "datasets after the suites:"',
    `LD_LIBRARY_PATH=${OPENZFS_PREFIX}/lib PATH=${OPENZFS_PATH} zfs list -H -o name -r || true`,
    'echo "pool images left:"',
    `find ${FIXTURE_ROOT} -name 'pool.img' -print`,
    '} | tee /root/zfs-teardown.log',
  ].join('\n');
  guest({ options, script: teardown, log: 'guest-teardown.log', timeout: RECORD_TIMEOUT_MS });
  const share = [
    'set -eu',
    'test -d /mnt/shared',
    `cp /root/zfs-root-suite.log /home/${ACCOUNT}/zfs-account-suite.log /root/zfs-versions.log /root/zfs-teardown.log /mnt/shared/`,
  ].join('\n');
  guest({ options, script: share, log: 'guest-share-logs.log', timeout: RECORD_TIMEOUT_MS });
  for (
    const [from, to] of [
      ['/mnt/shared/zfs-root-suite.log', 'root-suite.log'],
      ['/mnt/shared/zfs-account-suite.log', 'account-suite.log'],
      ['/mnt/shared/zfs-versions.log', 'versions.log'],
      ['/mnt/shared/zfs-teardown.log', 'teardown.log'],
    ] as const
  ) {
    host({
      options,
      command: 'mvm',
      args: ['--backend', 'libvirt', 'pull', options.name, from, join(options.logDir, to)],
      log: `host-pull-${to}`,
    });
  }
}

//region The step 3 arm: the wrapper's ZFS clone controls

/**
 The environment the wrapper's ZFS controls read inside the guest.
 The guest variable is what lets an arm change a dataset at all: on any other machine the helper refuses,
 because everything ZFS-changing runs in this throwaway guest and never on the host.
 */
function step3Environment(options: Options): string {
  return `CLI_GIT_TEST_ROOT=${STEP3_FIXTURE_ROOT}/checkout ` +
    `CLI_GIT_ZFS_FIXTURE_GUEST=${options.name} ` +
    `CLI_GIT_ZFS_TOOL_DIRECTORY=${OPENZFS_PREFIX}/sbin ` +
    `CLI_GIT_ZFS_SECOND_DATASET=${STEP3_FIXTURE_ROOT}/second ` +
    `CLI_GIT_ZFS_DIFFERENT_KEY_DATASET=${STEP3_FIXTURE_ROOT}/diffkey ` +
    `CLI_GIT_ZFS_CLONE_SNAPSHOT=${STEP3_CLONE_SNAPSHOT} ` +
    `CLI_GIT_BTRFS_ABSENT_ACKNOWLEDGED=1`;
}

/** The OpenZFS userspace's library and program paths, as shell assignments. */
const STEP3_OPENZFS = `LD_LIBRARY_PATH=${OPENZFS_PREFIX}/lib PATH=/root/.cargo/bin:${OPENZFS_PREFIX}/sbin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin`;

/**
 Make the step 3 pool:
 a 1 GiB encrypted image,
 the checkout dataset the fixture root stands on,
 a second dataset under the same key,
 a dataset that is an encryption root of its own,
 and an empty snapshot of the checkout dataset for the controls that need a mounted clone.
 */
function step3Pool(options: Options): void {
  const script = [
    'set -eu',
    `export ${STEP3_OPENZFS}`,
    `mkdir --parents ${STEP3_FIXTURE_ROOT}`,
    `if zpool list -H -o name | grep --quiet "^${STEP3_POOL}$"; then zpool destroy -f ${STEP3_POOL}; fi`,
    `rm --recursive --force ${STEP3_FIXTURE_ROOT}/checkout ${STEP3_FIXTURE_ROOT}/second ${STEP3_FIXTURE_ROOT}/diffkey ${STEP3_FIXTURE_ROOT}/late`,
    `rm --force ${STEP3_FIXTURE_ROOT}/pool.img ${STEP3_FIXTURE_ROOT}/raw-key ${STEP3_FIXTURE_ROOT}/other-key`,
    `truncate --size 1G ${STEP3_FIXTURE_ROOT}/pool.img`,
    `head --bytes 32 /dev/urandom > ${STEP3_FIXTURE_ROOT}/raw-key`,
    `head --bytes 32 /dev/urandom > ${STEP3_FIXTURE_ROOT}/other-key`,
    // The recordsize, the compression and the encryption are the measurements' fixture's, so the counters this run
    //  reads are of the pool the plan measured on.
    `zpool create -o cachefile=none -O mountpoint=none -O encryption=aes-256-gcm -O keyformat=raw -O keylocation=file://${STEP3_FIXTURE_ROOT}/raw-key -O recordsize=32K -O compression=zstd-3 ${STEP3_POOL} ${STEP3_FIXTURE_ROOT}/pool.img`,
    `zfs create -o mountpoint=${STEP3_FIXTURE_ROOT}/checkout ${STEP3_POOL}/checkout`,
    `zfs create -o mountpoint=${STEP3_FIXTURE_ROOT}/second ${STEP3_POOL}/second`,
    `zfs create -o mountpoint=${STEP3_FIXTURE_ROOT}/diffkey -o encryption=aes-256-gcm -o keyformat=raw -o keylocation=file://${STEP3_FIXTURE_ROOT}/other-key ${STEP3_POOL}/diffkey`,
    // The snapshot is taken while the checkout dataset holds nothing but its mountpoint, so a clone of it mounts empty
    //  and an add can fill it.
    `zfs snapshot ${STEP3_CLONE_SNAPSHOT}`,
    'echo "step 3: block cloning switch $(cat /sys/module/zfs/parameters/zfs_bclone_enabled)"',
    `zfs list -o name,mountpoint,encryptionroot,mounted -r ${STEP3_POOL}`,
    `zpool get -H -o property,value bcloneused,bclonesaved,bcloneratio ${STEP3_POOL}`,
  ].join('\n');
  guest({ options, script, log: 'step3-pool.log', timeout: RECORD_TIMEOUT_MS });
}

/** Push the wrapper crate, build its tests inside the guest, and run the ZFS controls and the measurement. */
function step3Arms(options: Options): void {
  const tarball = join(options.logDir, 'wrapper.tar.gz');
  host({
    options,
    command: 'tar',
    args: ['--create', '--gzip', '--file', tarball, '--exclude=target', '--exclude=node_modules', '--directory', options.wrapperCrate, ...STEP3_WRAPPER_PACKAGES],
    log: 'step3-host-package.log',
  });
  host({
    options,
    command: 'mvm',
    args: ['--backend', 'libvirt', 'push', options.name, tarball, '/root/wrapper.tar.gz'],
    log: 'step3-guest-push.log',
  });
  const build = [
    'set -eu',
    'if test -f /mnt/shared/wrapper.tar.gz; then cp /mnt/shared/wrapper.tar.gz /root/wrapper.tar.gz; fi',
    // The tree is unpacked over the one the last run left, so the build directory it already holds is reused and a
    //  changed file is the only thing rebuilt; a file this revision dropped stays behind, and nothing reads it.
    `mkdir --parents ${STEP3_WRAPPER_ROOT}`,
    `tar --extract --gzip --file /root/wrapper.tar.gz --directory ${STEP3_WRAPPER_ROOT}`,
    `cd ${STEP3_WRAPPER_DIR}`,
    'export PATH=/root/.cargo/bin:$PATH',
    // The guest has a network, and the crate's own lockfile is what it builds, so the build is the one a person gets
    //  from this commit rather than a set of versions this recipe would pin.
    // Two compiling jobs: the guest has four cores and a linking test binary of this crate needs more than a gigabyte,
    //  and an out-of-memory kill here would look like a compile failure.
    'cargo build --release --tests --locked --jobs 2 > /root/step3-wrapper-build.log 2>&1',
    'tail --lines=3 /root/step3-wrapper-build.log',
  ].join('\n');
  guest({ options, script: build, log: 'step3-wrapper-build.log', timeout: CRATE_BUILD_TIMEOUT_MS });

  // The arms run one at a time: the pool's block-clone counters count the whole pool, and the arm that proves the end of
  //  the clone rung turns the kernel's one block-cloning switch off for its own length.
  const suite = [
    'set -eu',
    `cd ${STEP3_WRAPPER_DIR}`,
    `export ${STEP3_OPENZFS}`,
    // The binary controls put their sockets in a folder under this account's home, which an `mvm exec` without a login
    //  shell does not set.
    'export HOME=/root',
    `export ${step3Environment(options)}`,
    'if cargo nextest run --release --test-threads 1 --no-fail-fast --status-level all --no-capture ' +
      '--test native_binary --filterset "test(worktree_clone_zfs)" ' +
      '> /root/step3-suite.log 2>&1; then status=0; else status=$?; fi',
    'if cargo nextest run --release --test-threads 1 --no-fail-fast --status-level all --no-capture ' +
      '--lib --filterset "test(zfs_a_real_mount_answers_with_its_kind)" ' +
      '>> /root/step3-suite.log 2>&1; then real_status=0; else real_status=$?; fi',
    'tail --lines=40 /root/step3-suite.log',
    'test "$status" = 0',
    'test "$real_status" = 0',
    'if grep --quiet "ZFS CONTROL NOT RUN" /root/step3-suite.log; then echo "a ZFS control did not run"; exit 1; fi',
    'grep --quiet "zfs_a_real_mount_answers_with_its_kind" /root/step3-suite.log',
    'grep --quiet "^MEASUREMENT zfs clone saved" /root/step3-suite.log',
  ].join('\n');
  guest({ options, script: suite, log: 'step3-suite.log', timeout: SUITE_TIMEOUT_MS });
}

/** One late-mount arm: what a confined consumer already started sees of a clone mounted afterwards. */
interface VisibilityArm {
  readonly propagation: string;
  readonly parentBind: boolean;
  readonly expected: 'VISIBLE' | 'HIDDEN';
}

/**
 The four arms the acceptance text names: shared and private propagation, with and without the read-only parent bind.
 The expected words are the measured ones of the step 3 run of 2026-11-20, not a guess: a read-only bind of the worktrees
 parent hides a mount made afterwards only under a private checkout, and a shared checkout propagates it into the
 consumer under the bind too. The design proposal's "Confined callers" says such a bind "can hide" a later mount, which
 holds for the private arm alone; the step 3 report records that reading.
 */
const VISIBILITY_ARMS: readonly VisibilityArm[] = [
  { propagation: 'shared', parentBind: false, expected: 'VISIBLE' },
  { propagation: 'private', parentBind: false, expected: 'HIDDEN' },
  { propagation: 'shared', parentBind: true, expected: 'VISIBLE' },
  { propagation: 'private', parentBind: true, expected: 'HIDDEN' },
];

/** The shell one confined consumer runs, quoted for the outer script. */
function lateMountConsumer(arm: VisibilityArm): string {
  const lines = [
    'set -u',
    `if test ${arm.parentBind ? 'yes' : 'no'} = yes; then mount --rbind ${STEP3_FIXTURE_ROOT} ${STEP3_FIXTURE_ROOT}; mount --options remount,ro,bind ${STEP3_FIXTURE_ROOT}; fi`,
    `: > /tmp/step3-gate-7-ready`,
    // The host writes the go file after it mounts the clone; the loop is bounded so a lost signal still ends the arm.
    `i=0`,
    `while test ! -e ${STEP3_FIXTURE_ROOT}/gate-7-go; do i=$((i + 1)); if test $i -gt 150; then break; fi; sleep 0.1; done`,
    `if test -e ${STEP3_FIXTURE_ROOT}/late/marker; then echo VISIBLE > /tmp/step3-gate-7-result; else echo HIDDEN > /tmp/step3-gate-7-result; fi`,
  ];
  return lines.join('\n');
}

/** One late-mount arm's shell: start the consumer, mount the clone after it, signal it, and read what it saw. */
function lateMountArm(arm: VisibilityArm): readonly string[] {
  const consumer =
    `runuser --preserve-environment --user ${ACCOUNT} -- env HOME=/home/${ACCOUNT} PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin ` +
    `unshare --user --map-root-user --mount --propagation ${arm.propagation} -- sh -ec ${shellQuote(lateMountConsumer(arm))}`;
  return [
    `rm --force /tmp/step3-gate-7-ready /tmp/step3-gate-7-result ${STEP3_FIXTURE_ROOT}/gate-7-go`,
    `if mountpoint --quiet ${STEP3_FIXTURE_ROOT}/late; then zfs unmount ${STEP3_POOL}/gate-7-late; fi`,
    `${consumer} &`,
    'consumer=$!',
    'i=0',
    'while test ! -e /tmp/step3-gate-7-ready; do i=$((i + 1)); if test $i -gt 150; then break; fi; sleep 0.1; done',
    `zfs mount ${STEP3_POOL}/gate-7-late`,
    `echo go > ${STEP3_FIXTURE_ROOT}/gate-7-go`,
    'wait $consumer',
    'seen=$(cat /tmp/step3-gate-7-result)',
    `echo "gate 7 late-mount propagation=${arm.propagation} parent-bind=${arm.parentBind ? 'yes' : 'no'}: $seen (expected ${arm.expected})"`,
    `test "$seen" = ${arm.expected}`,
    `zfs unmount ${STEP3_POOL}/gate-7-late`,
    `rm --force ${STEP3_FIXTURE_ROOT}/gate-7-go`,
  ];
}

/**
 What:
  H1 gate 7, as the plan's acceptance text quotes it, run inside the guest.
 Why:
  The confined consumer the wrapper will serve is a user namespace,
  and OpenZFS declares `FS_USERNS_MOUNT`,
  so the gate asks whether a confined
   namespace can attach a clone at all,
  what a permitted host mount shows as the control,
  what a consumer already running sees of a mount made later,
   and whether rung 2 still serves an add inside that consumer.
  `kernel.apparmor_restrict_unprivileged_userns` is 1 on this guest,
  which makes a confined consumer's mount attempt fail for the wrong reason,
  so the arms set it to 0 and say so in their log.
 */
function step3GateSeven(options: Options): void {
  const mountAttempt = (flags: string): string => {
    const option = flags.length === 0 ? '' : ` ${flags}`;
    return [
      `mkdir --parents /tmp/step3-gate-7-mount`,
      `if mount -t zfs${option} ${STEP3_POOL}/gate-7-clone /tmp/step3-gate-7-mount 2>/tmp/step3-gate-7-mount-error; then`,
      `  echo "CONFINED MOUNT ATTACHED${option}"; umount /tmp/step3-gate-7-mount`,
      `else`,
      `  echo "CONFINED MOUNT REFUSED${option}: $(tr --squeeze-repeats " " < /tmp/step3-gate-7-mount-error)"`,
      `fi`,
    ].join('\n');
  };
  const confined = (script: string): string =>
    `runuser --preserve-environment --user ${ACCOUNT} -- env HOME=/home/${ACCOUNT} LD_LIBRARY_PATH=${OPENZFS_PREFIX}/lib PATH=${OPENZFS_PREFIX}/sbin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin ` +
    `unshare --user --map-root-user --mount --propagation private -- sh -ec ${shellQuote(script)}`;
  const reflink = [
    'set -eu',
    `mkdir --parents ${STEP3_FIXTURE_ROOT}/checkout/account/repo`,
    `cd ${STEP3_FIXTURE_ROOT}/checkout/account/repo`,
    'git init --quiet --initial-branch=main',
    'mkdir --parents src',
    'printf "leaf\\n" > src/leaf.txt',
    // The file is aged past the index's raciness window and the index refreshed, so the source is provably clean and the
    //  clone rung is offered it; a file written in the same second as its own index entry is no proof either way.
    `touch --date "2 minutes ago" src/leaf.txt`,
    'git add --all',
    'git -c user.name=Fixture -c user.email=fixture@example.invalid commit --quiet --message=files',
    'git update-index --refresh',
    `PATH=${STEP3_FIXTURE_ROOT}/checkout/account/bin:/usr/bin:/bin MONOCHROMATIC_VERBOSE=true git worktree add --quiet -b reflink-arm ../reflink-arm 2> /tmp/step3-gate-7-reflink.log`,
    'grep --quiet "clones served" /tmp/step3-gate-7-reflink.log',
    'echo "gate 7 confined consumer: rung 2 served the add inside the user namespace"',
  ].join('\n');
  const script = [
    'set -eu',
    `export ${STEP3_OPENZFS}`,
    'echo "gate 7: kernel.apparmor_restrict_unprivileged_userns before the arms $(cat /proc/sys/kernel/apparmor_restrict_unprivileged_userns)"',
    // A confined consumer's mount attempt fails for the wrong reason while this is 1, so the arms take it to 0; the file
    //  is the kernel's and this guest is thrown away.
    'echo 0 > /proc/sys/kernel/apparmor_restrict_unprivileged_userns',
    'echo "gate 7: kernel.apparmor_restrict_unprivileged_userns during the arms $(cat /proc/sys/kernel/apparmor_restrict_unprivileged_userns)"',
    // The clone the mount arms ask for, with a marker inside it, and the account's own place on the checkout dataset.
    `zfs destroy -r ${STEP3_POOL}/gate-7-clone 2>/dev/null || true`,
    `mkdir --parents ${STEP3_FIXTURE_ROOT}/gate-7-clone-slot`,
    `zfs clone -o mountpoint=${STEP3_FIXTURE_ROOT}/gate-7-clone-slot ${STEP3_CLONE_SNAPSHOT} ${STEP3_POOL}/gate-7-clone`,
    `echo marker > ${STEP3_FIXTURE_ROOT}/gate-7-clone-slot/marker`,
    `rm --recursive --force ${STEP3_FIXTURE_ROOT}/checkout/account`,
    `mkdir --parents ${STEP3_FIXTURE_ROOT}/checkout/account/bin`,
    `chown --recursive ${ACCOUNT}:${ACCOUNT} ${STEP3_FIXTURE_ROOT}/checkout/account`,
    `ln --symbolic ${STEP3_WRAPPER} ${STEP3_FIXTURE_ROOT}/checkout/account/bin/git`,
    `echo "gate 7: OpenZFS declares the flag the gate asks about: $(grep --count FS_USERNS_MOUNT ${OPENZFS_SOURCE}/module/os/linux/zfs/zpl_super.c) lines of zpl_super.c"`,
    // The permitted host mount is the control: without it a refusal proves nothing. The clone mounted itself as it was
    //  created, so it is unmounted first and mounted again as the arm's own mount.
    `zfs unmount ${STEP3_POOL}/gate-7-clone`,
    `zfs mount ${STEP3_POOL}/gate-7-clone`,
    `grep --fixed-strings --quiet "${STEP3_FIXTURE_ROOT}/gate-7-clone-slot" /proc/self/mountinfo`,
    'echo "gate 7 control: root mounted the clone at the slot"',
    `zfs unmount ${STEP3_POOL}/gate-7-clone`,
    // A confined namespace may not attach it with any flags, whatever OpenZFS declares.
    `confined_mounts=$(${confined([mountAttempt(''), mountAttempt('-o ro'), mountAttempt('-o rw'), mountAttempt('-o zfsutil'), mountAttempt('-o ro,zfsutil'), 'echo "gate 7: in-namespace user $(id -u)"'].join('\n'))})`,
    'echo "$confined_mounts"',
    'if echo "$confined_mounts" | grep --quiet "CONFINED MOUNT ATTACHED"; then echo "a confined namespace attached the clone"; exit 1; fi',
    'echo "gate 7: every confined mount attempt was refused"',
    // The late-mount arms: the clone is created unmounted with a marker, and mounted after the consumer starts.
    `zfs destroy -r ${STEP3_POOL}/gate-7-late 2>/dev/null || true`,
    `zfs clone -o mountpoint=${STEP3_FIXTURE_ROOT}/late ${STEP3_CLONE_SNAPSHOT} ${STEP3_POOL}/gate-7-late`,
    `echo marker > ${STEP3_FIXTURE_ROOT}/late/marker`,
    `zfs unmount ${STEP3_POOL}/gate-7-late`,
    ...VISIBILITY_ARMS.flatMap((arm: VisibilityArm) => { return lateMountArm(arm); }),
    // Rung 2 inside the confined consumer.
    // Rung 2 inside the confined consumer, with the pool's own count around it: the add must be served by clones and the
    //  pool must show the blocks it shared.
    `zpool sync ${STEP3_POOL}`,
    `cloned_before=$(zpool get -H -p -o value bcloneused ${STEP3_POOL})`,
    `confined_reflink=$(${confined(reflink)})`,
    'echo "$confined_reflink"',
    `zpool sync ${STEP3_POOL}`,
    `cloned_after=$(zpool get -H -p -o value bcloneused ${STEP3_POOL})`,
    'echo "gate 7: the confined add counted $((cloned_after - cloned_before)) cloned bytes"',
    'test $((cloned_after - cloned_before)) -gt 0',
    `zfs destroy -r ${STEP3_POOL}/gate-7-late`,
    `zfs destroy -r ${STEP3_POOL}/gate-7-clone`,
  ].join('\n');
  guest({ options, script, log: 'step3-gate-7.log', timeout: SUITE_TIMEOUT_MS });
}

/** Destroy the step 3 pool, then record what the pool and the fixture root hold afterwards. */
function step3Teardown(options: Options): void {
  const script = [
    'set -eu',
    `export ${STEP3_OPENZFS}`,
    `zpool destroy -f ${STEP3_POOL} 2>/dev/null || true`,
    '{ echo "pools after the step 3 arm:"',
    'zpool list -H -o name || true',
    'echo "datasets after the step 3 arm:"',
    'zfs list -H -o name -r || true',
    `echo "pool images left:"`,
    `find ${STEP3_FIXTURE_ROOT} -name "pool.img" -print`,
    '} | tee /root/step3-teardown.log',
  ].join('\n');
  guest({ options, script, log: 'step3-teardown.log', timeout: RECORD_TIMEOUT_MS });
}

/** Record the step 3 arm's versions and pull its logs. */
function step3Record(options: Options): void {
  const versions = [
    'set -eu',
    `export ${STEP3_OPENZFS}`,
    '{ uname --kernel-release',
    'cat /sys/module/zfs/version',
    'zfs version',
    'git --version',
    'echo "block cloning switch: $(cat /sys/module/zfs/parameters/zfs_bclone_enabled)"',
    'echo "apparmor_restrict_unprivileged_userns: $(cat /proc/sys/kernel/apparmor_restrict_unprivileged_userns)"',
    '/root/.cargo/bin/rustc --version',
    '/root/.cargo/bin/cargo nextest --version',
    '} | tee /root/step3-versions.log',
  ].join('\n');
  guest({ options, script: versions, log: 'step3-versions.log', timeout: RECORD_TIMEOUT_MS });
  // Only the logs the guest wrote to files are pulled: the pool, suite and gate 7 steps already stream their output into
  //  this run's log directory, and the suite's own file holds the whole run rather than the tail the step prints.
  const share = [
    'set -eu',
    'test -d /mnt/shared',
    'for name in step3-suite step3-teardown step3-versions step3-wrapper-build; do if test -f /root/$name.log; then cp /root/$name.log /mnt/shared/; fi; done',
  ].join('\n');
  guest({ options, script: share, log: 'step3-guest-share-logs.log', timeout: RECORD_TIMEOUT_MS });
  for (
    const name of [
      'step3-suite',
      'step3-teardown',
      'step3-versions',
      'step3-wrapper-build',
    ] as const
  ) {
    host({
      options,
      command: 'mvm',
      args: ['--backend', 'libvirt', 'pull', options.name, `/mnt/shared/${name}.log`, join(options.logDir, `${name}.log`)],
      log: `step3-host-pull-${name}.log`,
    });
  }
}

/**
 Run the whole step 3 arm in the prepared guest and always leave the pool and the fixture root destroyed,
 with the teardown evidence and the logs pulled out even when an arm failed.
 */
function step3(options: Options): void {
  let failure: unknown = null;
  try {
    step3Pool(options);
    step3Arms(options);
    step3GateSeven(options);
  } catch (thrown) {
    failure = thrown;
  }
  // The pool is the arm's own and is destroyed whatever happened, so a failed run leaves the guest as it found it; a
  //  failed teardown is reported beside the arm's failure rather than in place of it.
  try {
    step3Teardown(options);
  } catch (thrown) {
    console.error(`zfs step 3 arm: destroying the pool also failed: ${thrown}`);
  }
  try {
    step3Record(options);
  } catch (thrown) {
    console.error(`zfs step 3 arm: recording also failed: ${thrown}`);
  }
  if (failure !== null)
    throw failure;
}

//endregion The step 3 arm: the wrapper's ZFS clone controls

/** Destroy the guest and keep the list of what is left. */
function destroy(options: Options): void {
  host({ options, command: 'mvm', args: ['--backend', 'libvirt', 'destroy', options.name], log: 'mvm-destroy.log' });
  host({ options, command: 'mvm', args: ['--backend', 'libvirt', 'list'], log: 'mvm-list-after-destroy.log' });
}

//endregion Steps

//region Entry point

/** The steps by name, and the whole run. */
/**
 What:
  Run the steps in order and destroy the guest afterwards,
  keeping the guest only when `--keep-guest` was given.
 Why:
  The step 3 arm's own steps pull their logs before they end,
  so a failed run of them leaves the same evidence a run of `all` leaves,
  and
   the guest is thrown away either way.
 */
function runThenDestroy(checked: Options, ordered: readonly ((checkedStep: Options) => void)[]): void {
  let failure: unknown = null;
  try {
    for (const each of ordered) each(checked);
  } catch (thrown) {
    failure = thrown;
  }
  if (checked.keepGuest) {
    console.log(`zfs fixture: --keep-guest kept '${checked.name}' to inspect and destroy`);
  } else {
    try {
      destroy(checked);
    } catch (destruction) {
      console.error(`zfs fixture: destroying '${checked.name}' also failed: ${destruction}`);
    }
  }
  if (failure !== null)
    throw failure;
}

const steps: Record<string, (checked: Options) => void> = {
  create,
  prepare,
  run,
  record,
  destroy,
  wrapper: step3,
  arm: (checked: Options): void => {
    runThenDestroy(checked, [create, prepare, step3,]);
  },
  all: (checked: Options): void => {
    let failure: unknown = null;
    try {
      create(checked);
      prepare(checked);
      run(checked);
      record(checked);
    } catch (thrown) {
      failure = thrown;
    }
    if (failure !== null) {
      // A failed step still leaves the suite logs in the guest: try to record
      // them, so a run that reached both suites keeps its logs, and never let a
      // failed record replace the step's failure.
      try {
        record(checked);
      } catch {
        // The step's own failure is the report; a failed record must not hide it.
      }
    }
    if (checked.keepGuest) {
      console.log(`zfs step 2 fixture: --keep-guest kept '${checked.name}' to inspect and destroy`);
    } else if (failure === null) {
      destroy(checked);
    } else {
      // The guest is destroyed even when a step failed, which is what makes a
      // failed run leave nothing behind; a failed destruction is reported beside
      // the step's failure rather than in place of it.
      try {
        destroy(checked);
      } catch (destruction) {
        console.error(`zfs step 2 fixture: destroying '${checked.name}' also failed: ${destruction}`);
      }
    }
    if (failure !== null)
      throw failure;
  },
};

const chosen = options();
mkdirSync(chosen.logDir, { recursive: true, mode: 0o700, });
const step = steps[chosen.step];
if (step === undefined)
  throw new Error(`unknown step '${chosen.step}'; use ${Object.keys(steps).join(', ')}`);
step(chosen);
console.log(`zfs step 2 fixture: step '${chosen.step}' finished; logs in ${chosen.logDir}`);

//endregion Entry point
