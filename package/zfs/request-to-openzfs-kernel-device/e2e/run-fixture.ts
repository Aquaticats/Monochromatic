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

/** Destroy the guest and keep the list of what is left. */
function destroy(options: Options): void {
  host({ options, command: 'mvm', args: ['--backend', 'libvirt', 'destroy', options.name], log: 'mvm-destroy.log' });
  host({ options, command: 'mvm', args: ['--backend', 'libvirt', 'list'], log: 'mvm-list-after-destroy.log' });
}

//endregion Steps

//region Entry point

/** The steps by name, and the whole run. */
const steps: Record<string, (checked: Options) => void> = {
  create,
  prepare,
  run,
  record,
  destroy,
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
      // A failed step still leaves the suite logs in the guest: pull them so the
      // failure can be read, and never let a second failure replace the first.
      try {
        record(checked);
      } catch {
        // The step's own failure is the report; a failed pull must not hide it.
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
