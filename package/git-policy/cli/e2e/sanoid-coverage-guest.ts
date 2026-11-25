#!/usr/bin/env node
/**
 Drive plan step 20's Sanoid arms in a throwaway `mvm` guest.

 The arms read a real dataset tree: recursive coverage, the exclusion sections and their order, the
 `recursive = zfs` control, the markers, retention pruning and the coverage comparison. They run Sanoid
 2.3.0 beside the Ubuntu package's own 2.2.0 as a control, because the project's measurements of the
 section-order effect were made on 2.3.0 and the earlier guest had 2.2.0.

 This driver owns the guest through the ZFS binding's own recipe
 (`package/zfs/request-to-openzfs-kernel-device/e2e/run-fixture.ts`): it creates the Ubuntu 24.04 guest,
 prepares it with OpenZFS 2.4.4, Rust and cargo-nextest, installs both Sanoid releases, builds this
 crate's binaries and tests inside it, runs the arm, and destroys the guest. The arm itself creates the
 file-backed pool, the decided layout, the markers and the configurations, and asserts every claim.

 Every line a run must find is printed by the arm as `SANOID-GUEST <claim>`. `create` refuses a name
 `mvm list` already reports, and `destroy` removes only a guest this driver created.

 Usage: node e2e/sanoid-coverage-guest.ts <create|prepare|build|arm|destroy|all> [--name <guest>] [--log-dir <dir>]
          [--keep-guest]
 */
import { spawnSync, } from 'node:child_process';
import { closeSync, mkdirSync, openSync, readFileSync, rmSync, } from 'node:fs';
import { join, resolve, } from 'node:path';

import { copyWrapperSnapshot, } from '../bin/native-wrapper-snapshot.ts';

//region Constants

/** Where the crate is unpacked and built inside the guest. */
const CRATE_DIR = '/opt/git-policy-cli-sanoid-coverage';
/** The file-backed pool the arm's layout sits on, which only the guest ever sees. */
const POOL = 'sanoidstep20';
/** The variable the arm reads: its value is the guest's name, so a run outside the guest prints SKIP. */
const GUEST_VARIABLE = 'CLI_GIT_ZFS_SANOID_GUEST';
/** The variable the arm reads for the pool's name. */
const POOL_VARIABLE = 'CLI_GIT_ZFS_SANOID_POOL';
/** The release of Sanoid the arms measure. */
const SANOID_RELEASE = '2.3.0';
/** Where the Sanoid 2.3.0 release is unpacked inside the guest. */
const SANOID_SOURCE = `/opt/sanoid-${SANOID_RELEASE}`;
/** The GitHub source archive of that release, which the guest downloads and checks. */
const SANOID_URL = `https://github.com/jimsalterjrs/sanoid/archive/refs/tags/v${SANOID_RELEASE}.tar.gz`;
/**
 sha256 of that archive, measured on 2026-11-25 from the tag `v2.3.0`
 (commit 8d4abf14b2bf8d6b98fc8e163fa9895486e8cdeb).
 A published archive that GitHub regenerates fails this check rather than measuring other bytes.
 */
const SANOID_SHA256 = '1d8735a271a34ec87ea46313a66f6f20bd38b583886924574d3c1f72ea173620';
/** The Sanoid release the Ubuntu package carries, kept beside the measured one as a control. */
const CONTROL_RELEASE = '2.2';
/** Milliseconds one host command may take. */
const HOST_COMMAND_TIMEOUT_MS = 120_000;
/** Milliseconds the guest's Sanoid install may take. */
const SANOID_INSTALL_TIMEOUT_MS = 600_000;
/** Milliseconds the guest's crate build may take. */
const CRATE_BUILD_TIMEOUT_MS = 2_400_000;
/** Milliseconds one arm may take. */
const ARM_TIMEOUT_MS = 1_200_000;
/** Seconds the guest's own `timeout` fires before `mvm`'s kill. */
const GUEST_TIMEOUT_MARGIN_SECONDS = 60;
/** The lines a green arm must print, one per claim it makes. */
const GREEN_MARKERS: readonly string[] = [
  'SANOID-GUEST layout-built',
  'SANOID-GUEST coverage-passes',
  'SANOID-GUEST 2-3-0-covers-home-only',
  'SANOID-GUEST 2-3-0-order-captures-the-exclusion',
  'SANOID-GUEST 2-3-0-zfs-recursion-captures-the-exclusion',
  'SANOID-GUEST 2-2-0-covers-home-only',
  'SANOID-GUEST 2-2-0-order-captures-the-exclusion',
  'SANOID-GUEST prune-keeps-the-pinned-origin',
  'SANOID-GUEST marker-finding-and-correction-apply',
  'SANOID-GUEST order-finding-and-correction-apply',
  'SANOID-GUEST recursion-finding-and-correction-apply',
  'SANOID-GUEST store-finding-and-nothing-reaches-it',
  'SANOID-GUEST guard-scope-names-what-it-cannot-prune',
];

//endregion Constants

//region Arguments

interface Options {
  readonly step: string;
  readonly name: string;
  readonly logDir: string;
  readonly keepGuest: boolean;
}

/** The step and the values the command line gave it. */
function options(): Options {
  const [ step = '', ...rest ] = process.argv.slice(2);
  const value = (flag: string): string | undefined => {
    const at = rest.indexOf(flag);
    return at === -1 ? undefined : rest[at + 1];
  };
  return {
    step,
    name: value('--name') ?? 'git-policy-sanoid-coverage',
    logDir: value('--log-dir') ?? resolve(import.meta.dirname, '..', 'target', 'sanoid-coverage-guest-logs'),
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
function host({ chosen, command, args, log, }: { readonly chosen: Options; readonly command: string; readonly args: readonly string[]; readonly log: string; }): void {
  const output = openSync(join(chosen.logDir, log), 'w', 0o600);
  try {
    const result = spawnSync(command, args, { stdio: [ 'ignore', output, output, ], timeout: HOST_COMMAND_TIMEOUT_MS, });
    if ((result.error !== undefined) || (result.status !== 0))
      throw result.error ?? new Error(`${command} ${args.join(' ')} exited ${String(result.status)}; see ${log}`);
  } finally {
    closeSync(output);
  }
}

/** One guest script, its whole output kept in its own log and returned. */
function guest({ chosen, script, log, timeout, }: { readonly chosen: Options; readonly script: string; readonly log: string; readonly timeout: number; }): string {
  const path = join(chosen.logDir, log);
  const output = openSync(path, 'w', 0o600);
  const command = `test "$(hostname)" = ${chosen.name} && timeout ${Math.floor(timeout / 1000) - GUEST_TIMEOUT_MARGIN_SECONDS} sh -ec ${shellQuote(script)}`;
  try {
    const result = spawnSync('mvm', [ '--backend', 'libvirt', 'exec', chosen.name, '--', command, ], { stdio: [ 'ignore', output, output, ], timeout, });
    if ((result.error !== undefined) || (result.status !== 0))
      throw result.error ?? new Error(`mvm exec exited ${String(result.status)}; see ${log}`);
  } finally {
    closeSync(output);
  }
  return readFileSync(path, 'utf8');
}

/** One step of the ZFS binding's guest recipe, which owns the guest's creation, preparation and destruction. */
function recipe({ chosen, step, }: { readonly chosen: Options; readonly step: string; }): void {
  const script = resolve(import.meta.dirname, '..', '..', '..', 'zfs', 'request-to-openzfs-kernel-device', 'e2e', 'run-fixture.ts');
  const result = spawnSync('node', [ script, step, '--name', chosen.name, '--log-dir', chosen.logDir, ], { stdio: 'inherit', timeout: HOST_COMMAND_TIMEOUT_MS * 60, });
  if ((result.error !== undefined) || (result.status !== 0))
    throw result.error ?? new Error(`the guest recipe's ${step} step exited ${String(result.status)}`);
}

//endregion Commands

//region Steps

/** Refuse a name another guest already holds. */
function create(chosen: Options): void {
  const listed = spawnSync('mvm', [ 'list', ], { encoding: 'utf8', timeout: HOST_COMMAND_TIMEOUT_MS, });
  if ((listed.stdout ?? '').split('\n').some((line) => { return line.trim().startsWith(chosen.name); }))
    throw new Error(`a guest named ${chosen.name} is already listed; name another with --name`);
  recipe({ chosen, step: 'create', });
}

/** Install OpenZFS 2.4.4, Rust and cargo-nextest, then both Sanoid releases. */
function prepare(chosen: Options): void {
  recipe({ chosen, step: 'prepare', });
  const script = [
    'set -eu',
    'export DEBIAN_FRONTEND=noninteractive',
    'apt-get --option Acquire::Retries=2 --yes install sanoid > /root/sanoid-install.log 2>&1',
    'installed=$(command -v sanoid)',
    'installed_version=$("$installed" --version)',
    'echo "installed $installed_version at $installed" >> /root/sanoid-install.log',
    `case "$installed_version" in *${CONTROL_RELEASE}*) ;; *) echo "the Ubuntu package is not ${CONTROL_RELEASE}: $installed_version" >&2; exit 1;; esac`,
    `curl --fail --location --retry 3 --max-time 600 --output /root/sanoid-${SANOID_RELEASE}.tar.gz ${SANOID_URL} >> /root/sanoid-install.log 2>&1`,
    `printf '%s  %s\\n' ${SANOID_SHA256} /root/sanoid-${SANOID_RELEASE}.tar.gz | sha256sum --check --strict - >> /root/sanoid-install.log 2>&1`,
    `tar --extract --gzip --file /root/sanoid-${SANOID_RELEASE}.tar.gz --directory /opt`,
    `measured=$(perl ${SANOID_SOURCE}/sanoid --version)`,
    `case "$measured" in *${SANOID_RELEASE}*) ;; *) echo "the archive is not ${SANOID_RELEASE}: $measured" >&2; exit 1;; esac`,
    'echo "measured $measured from the archive" >> /root/sanoid-install.log',
    'tail --lines=6 /root/sanoid-install.log',
  ].join('\n');
  guest({ chosen, script, log: 'guest-sanoid-install.log', timeout: SANOID_INSTALL_TIMEOUT_MS, });
}

/** Push a snapshot of this worktree and build this crate's binaries and tests inside the guest. */
async function build(chosen: Options): Promise<void> {
  const context = join(chosen.logDir, 'snapshot');
  rmSync(context, { recursive: true, force: true, });
  mkdirSync(context, { recursive: true, mode: 0o700, });
  await copyWrapperSnapshot({ context, });
  const tarball = join(chosen.logDir, 'subject.tar.gz');
  host({ chosen, command: 'tar', args: [ '--create', '--gzip', '--file', tarball, '--exclude=target', '--exclude=node_modules', '--directory', context, '.' ], log: 'host-package.log', });
  host({ chosen, command: 'mvm', args: [ '--backend', 'libvirt', 'push', chosen.name, tarball, '/root/subject.tar.gz' ], log: 'guest-push.log', });
  const script = [
    'set -eu',
    'if test -f /mnt/shared/subject.tar.gz; then cp /mnt/shared/subject.tar.gz /root/subject.tar.gz; fi',
    `rm --recursive --force ${CRATE_DIR}`,
    `mkdir --parents ${CRATE_DIR}`,
    `tar --extract --gzip --file /root/subject.tar.gz --directory ${CRATE_DIR}`,
    `cd ${CRATE_DIR}/package/git-policy/cli`,
    'export PATH=/root/.cargo/bin:$PATH',
    // Two jobs keep rustc and the linker inside the guest's memory: the host backs the guest's pages, and a build
    // that runs one compiler per vCPU peaked above what the guest could take when the host was short.
    'export CARGO_BUILD_JOBS=2',
    'if cargo build --release --locked --bins --tests > /root/sanoid-coverage-build.log 2>&1; then status=0; else status=$?; fi',
    'grep --extended-regexp "^(error|warning)" --after-context=6 /root/sanoid-coverage-build.log | tail --lines=80 || true',
    'tail --lines=5 /root/sanoid-coverage-build.log',
    'exit $status',
  ].join('\n');
  guest({ chosen, script, log: 'guest-crate-build.log', timeout: CRATE_BUILD_TIMEOUT_MS, });
}

/** Run the arm and refuse unless every claim it prints was printed. */
function arm(chosen: Options): void {
  const script = [
    'set -eu',
    `cd ${CRATE_DIR}/package/git-policy/cli`,
    'export PATH=/opt/openzfs-2.4.4/sbin:/opt/openzfs-2.4.4/bin:/root/.cargo/bin:$PATH',
    'export LD_LIBRARY_PATH=/opt/openzfs-2.4.4/lib',
    'export CARGO_BUILD_JOBS=2',
    `export ${GUEST_VARIABLE}=${chosen.name}`,
    `export ${POOL_VARIABLE}=${POOL}`,
    'if /root/.cargo/bin/cargo nextest run --release --locked --test native_binary --no-capture --status-level all ' +
      '--filterset "test(zfs_sanoid_coverage_guest)" > /root/sanoid-coverage-arm.log 2>&1; then status=0; else status=$?; fi',
    'cat /root/sanoid-coverage-arm.log',
    'exit $status',
  ].join('\n');
  const pulled = guest({ chosen, script, log: 'guest-arm.log', timeout: ARM_TIMEOUT_MS, });
  if (/(^|\n)SKIP/m.test(pulled))
    throw new Error('the arm printed SKIP inside the guest: its variables did not reach it');
  const missing: string[] = GREEN_MARKERS.filter((marker) => { return !pulled.includes(marker); });
  if (missing.length > 0)
    throw new Error(`the arm did not print: ${missing.join(', ')}`);
  process.stdout.write(`sanoid coverage arm passed ${String(GREEN_MARKERS.length)} claim(s)\n`);
}

/** Destroy the guest this driver made. */
function destroy(chosen: Options): void {
  if (chosen.keepGuest) {
    process.stdout.write(`kept ${chosen.name}\n`);
    return;
  }
  recipe({ chosen, step: 'destroy', });
}

//endregion Steps

async function main(): Promise<void> {
  const chosen: Options = options();
  mkdirSync(chosen.logDir, { recursive: true, mode: 0o700, });
  switch (chosen.step) {
    case 'create': {
      create(chosen);
      return;
    }
    case 'prepare': {
      prepare(chosen);
      return;
    }
    case 'build': {
      await build(chosen);
      return;
    }
    case 'arm': {
      arm(chosen);
      return;
    }
    case 'destroy': {
      destroy(chosen);
      return;
    }
    case 'all': {
      create(chosen);
      try {
        prepare(chosen);
        await build(chosen);
        arm(chosen);
      } finally {
        destroy(chosen);
      }
      return;
    }
    default: {
      throw new Error(`unknown step ${chosen.step}; give create, prepare, build, arm, destroy or all`);
    }
  }
}

await main();
