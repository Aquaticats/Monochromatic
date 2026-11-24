#!/usr/bin/env node
/**
 Drive plan step 18's `doctor` checks for steps 2 to 9 in a throwaway `mvm` guest.

 The checks read `/dev/zfs`, a real pool's dataset properties and the kernel's delegation reply, so only a guest with a
 file-backed pool can show them. This driver owns the guest through the ZFS binding's own recipe
 (`package/zfs/request-to-openzfs-kernel-device/e2e/run-fixture.ts`): it creates the Ubuntu 24.04 guest, prepares it with
 OpenZFS 2.4.4, Rust and cargo-nextest, builds this crate's binaries and tests inside it, runs the arm, and destroys the
 guest. The arm itself creates the design's layout, the store, the installed files and the records, asserts every group
 passes, then breaks one fact at a time and asserts the finding.

 Every line a run must find is printed by the arm as `DOCTOR-GUEST <claim>`. `create` refuses a name `mvm list` already
 reports, and `destroy` removes only a guest this driver created.

 Usage: node e2e/doctor-zfs-route-guest.ts <create|prepare|build|arm|destroy|all> [--name <guest>] [--log-dir <dir>]
          [--keep-guest]
 */
import { spawnSync, } from 'node:child_process';
import { closeSync, mkdirSync, openSync, readFileSync, rmSync, } from 'node:fs';
import { join, resolve, } from 'node:path';

import { copyWrapperSnapshot, } from '../bin/native-wrapper-snapshot.ts';

//region Constants

/** Where the crate is unpacked and built inside the guest. */
const CRATE_DIR = '/opt/git-policy-cli-doctor-route';
/** The file-backed pool the arm's layout sits on, which only the guest ever sees. */
const POOL = 'zpcachyos';
/** The variable the arm reads: its value is the guest's name, so a run outside the guest prints SKIP. */
const GUEST_VARIABLE = 'CLI_GIT_ZFS_DOCTOR_GUEST';
/** The variable the arm reads for the pool's name. */
const POOL_VARIABLE = 'CLI_GIT_ZFS_DOCTOR_POOL';
/** Milliseconds one host command may take. */
const HOST_COMMAND_TIMEOUT_MS = 120_000;
/** Milliseconds the guest's crate build may take. */
const CRATE_BUILD_TIMEOUT_MS = 2_400_000;
/** Milliseconds one arm may take. */
const ARM_TIMEOUT_MS = 900_000;
/** Seconds the guest's own `timeout` fires before `mvm`'s kill. */
const GUEST_TIMEOUT_MARGIN_SECONDS = 60;
/** The lines a green arm must print, one per claim it makes. */
const GREEN_MARKERS: readonly string[] = [
  'DOCTOR-GUEST group-one-passes',
  'DOCTOR-GUEST group-two-passes',
  'DOCTOR-GUEST group-three-passes',
  'DOCTOR-GUEST unloaded-key-is-a-finding',
  'DOCTOR-GUEST container-canmount-is-a-finding',
  'DOCTOR-GUEST nested-row-is-a-finding',
  'DOCTOR-GUEST misfiled-record-is-a-finding',
  'DOCTOR-GUEST staging-forms-are-removed-and-nothing-else-is',
  'DOCTOR-GUEST rename-grant-is-a-finding',
  'DOCTOR-GUEST readiness-from-another-boot-is-a-finding',
  'DOCTOR-GUEST later-steps-are-unknown',
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
    name: value('--name') ?? 'git-policy-doctor-route',
    logDir: value('--log-dir') ?? resolve(import.meta.dirname, '..', 'target', 'doctor-route-guest-logs'),
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

/** Install the guest's tools: OpenZFS 2.4.4, Rust and cargo-nextest. */
function prepare(chosen: Options): void {
  recipe({ chosen, step: 'prepare', });
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
    'cargo build --release --locked --bins --tests > /root/doctor-route-build.log 2>&1',
    'tail --lines=3 /root/doctor-route-build.log',
  ].join('\n');
  guest({ chosen, script, log: 'guest-crate-build.log', timeout: CRATE_BUILD_TIMEOUT_MS, });
}

/** Run the arm and refuse unless every claim it prints was printed. */
function arm(chosen: Options): void {
  const script = [
    'set -eu',
    `cd ${CRATE_DIR}/package/git-policy/cli`,
    'export PATH=/root/.cargo/bin:$PATH',
    `export ${GUEST_VARIABLE}=${chosen.name}`,
    `export ${POOL_VARIABLE}=${POOL}`,
    'if /root/.cargo/bin/cargo nextest run --release --locked --test native_binary --no-capture --status-level all ' +
      '--filterset "test(zfs_doctor_route_guest)" > /root/doctor-route-arm.log 2>&1; then status=0; else status=$?; fi',
    'cat /root/doctor-route-arm.log',
    'exit $status',
  ].join('\n');
  const pulled = guest({ chosen, script, log: 'guest-arm.log', timeout: ARM_TIMEOUT_MS, });
  if (/(^|\n)SKIP/m.test(pulled))
    throw new Error('the arm printed SKIP inside the guest: its variables did not reach it');
  const missing: string[] = GREEN_MARKERS.filter((marker) => { return !pulled.includes(marker); });
  if (missing.length > 0)
    throw new Error(`the arm did not print: ${missing.join(', ')}`);
  process.stdout.write(`doctor route arm passed ${String(GREEN_MARKERS.length)} claim(s)\n`);
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
