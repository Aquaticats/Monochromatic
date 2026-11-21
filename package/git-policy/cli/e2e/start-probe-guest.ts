#!/usr/bin/env node
/**
 Drive the root start probe's arms in the throwaway `mvm` guest.

 The probe mounts, unshares a mount namespace and reads `/proc/spl/kstat/zfs`, so no control on a person's machine may
 run it. This driver borrows the ZFS binding's guest recipe for the parts that are not the probe's
 (`package/zfs/request-to-openfs-kernel-device/e2e/run-fixture.ts`): it creates the Ubuntu 24.04 guest, prepares it with
 OpenZFS 2.4.4 and the Rust toolchain, then builds this crate's handler inside it, prepares the probe dataset the design
 names, runs the arms, and destroys the guest.

 Usage: node e2e/start-probe-guest.ts <create|prepare|red|probe-dataset|green|destroy|all> [--name <guest>]
          [--log-dir <dir>] [--keep-guest]

 `red` runs the arm before the probe exists and requires the refusal that says the mode is not one of this build's,
 which is what proves the green run below is a change and not an absence that always passed. `green` requires the
 record's own words: the attach arm passed, the mount the probe made is gone from every namespace afterwards, and a
 filtered `move_mount` fails that step with `EPERM`.

 `create` refuses a name `mvm list` already reports, and `all` destroys only a guest its own `create` returned.
 */
import { spawnSync, } from 'node:child_process';
import { closeSync, mkdirSync, openSync, readFileSync, rmSync, } from 'node:fs';
import { join, resolve, } from 'node:path';

import { copyWrapperSnapshot, } from '../bin/native-wrapper-snapshot.ts';

//region Constants

/** Where the crate is unpacked and built inside the guest, which every account may traverse. */
const CRATE_DIR = '/opt/git-policy-cli-start-probe';
/** The handler binary the arms start. */
const HANDLER = `${CRATE_DIR}/package/git-policy/cli/target/release/mount-git-worktree-clone-datasets`;
/** The record the probe writes and the arms read. */
const RECORD = '/run/git-worktree-clone-mount-probe/result';
/** The probe dataset the design names (doc/planning/cli-git-zfs-start-probe-design.md, "The source"). */
const PROBE_DATASET = 'zpcachyos/nosnap/git-worktree-probe';
/** The fixture's file-backed pool, created by this driver and destroyed with the guest. */
const POOL = 'zpcachyos';
/** The pool's image inside the guest. */
const POOL_IMAGE = '/home/zfs-probe-pool.img';
/** The pool image's size: the probe mounts read-only and writes no block. */
const POOL_IMAGE_SIZE = '1G';
/** Where the handler's configuration lives, as the build names it. */
const CONFIGURATION = '/etc/monochromatic-git-policy-cli/git-worktree-clone-mounts.jsonc';
/** The account the guest image creates, which the configuration pins as the managed account. */
const ACCOUNT = 'ubuntu';
/** The checkout path the configuration pins, which the checkout-root arm opens read-only. */
const CHECKOUT = '/home/git-worktree-probe-checkout';
/** The `PATH` the guest's OpenZFS 2.4.4 userspace needs. */
const OPENZFS_PREFIX = '/opt/openzfs-2.4.4';
const OPENZFS_PATH = `${OPENZFS_PREFIX}/sbin:${OPENZFS_PREFIX}/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin`;
/** Milliseconds one host command may take. */
const HOST_COMMAND_TIMEOUT_MS = 120_000;
/** Milliseconds the guest's crate build may take. */
const CRATE_BUILD_TIMEOUT_MS = 2_400_000;
/** Milliseconds one probe arm may take. */
const ARM_TIMEOUT_MS = 900_000;
/** Seconds the guest's own `timeout` fires before `mvm`'s kill. */
const GUEST_TIMEOUT_MARGIN_SECONDS = 60;
/** The lines a green run must find, one per claim it makes. */
const GREEN_MARKERS = [
  'PROBE verdict passed',
  'PROBE attach passed',
  'PROBE host table unchanged',
  'PROBE source absent afterwards',
  'PROBE refused step attach-place errno EPERM',
  'PROBE timed-out in flight unmount',
  'PROBE killed mode left a started record',
  'PROBE held source not exercised',
  'PROBE absent source recorded',
  'PROBE txg window clean',
  'PROBE scan saw the worker and the holder',
  'PROBE scan saw the worker alone',
  'PROBE refusal controls refused',
  'GREEN done',
] as const;

//endregion Constants

//region Arguments

/** The step, the guest name, the log directory, whether `all` keeps the guest and whether `green` rebuilds. */
interface Options {
  readonly step: string;
  readonly name: string;
  readonly logDir: string;
  readonly keepGuest: boolean;
  readonly noBuild: boolean;
}

/** The step and the values the command line gave it. */
function options(): Options {
  const [step = '', ...rest] = process.argv.slice(2);
  const value = (flag: string): string | undefined => {
    const at = rest.indexOf(flag);
    return at === -1 ? undefined : rest[at + 1];
  };
  return {
    step,
    name: value('--name') ?? 'git-policy-start-probe',
    logDir: value('--log-dir') ?? resolve(import.meta.dirname, '..', 'target', 'start-probe-guest-logs'),
    keepGuest: rest.includes('--keep-guest'),
    noBuild: rest.includes('--no-build'),
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
    const result = spawnSync(command, args, {
      stdio: ['ignore', output, output,],
      timeout: HOST_COMMAND_TIMEOUT_MS,
    });
    if ((result.error !== undefined) || (result.status !== 0))
      throw result.error ?? new Error(`${command} ${args.join(' ')} exited ${String(result.status)}; see ${log}`);
  } finally {
    closeSync(output);
  }
}

/** One guest script, its whole output kept in its own log. */
function guest({ chosen, script, log, timeout, }: { readonly chosen: Options; readonly script: string; readonly log: string; readonly timeout: number; }): string {
  const output = openSync(join(chosen.logDir, log), 'w', 0o600);
  const command = `test "$(hostname)" = ${chosen.name} && timeout ${Math.floor(timeout / 1000) - GUEST_TIMEOUT_MARGIN_SECONDS} sh -ec ${shellQuote(script)}`;
  try {
    const result = spawnSync('mvm', ['--backend', 'libvirt', 'exec', chosen.name, '--', command], {
      stdio: ['ignore', output, output,],
      timeout,
    });
    if ((result.error !== undefined) || (result.status !== 0))
      throw result.error ?? new Error(`mvm exec exited ${String(result.status)}; see ${log}`);
  } finally {
    closeSync(output);
  }
  return readFileSync(join(chosen.logDir, log), 'utf8');
}

/** One step of the ZFS binding's guest recipe, which owns the guest's creation, preparation and destruction. */
function recipe({ chosen, step, }: { readonly chosen: Options; readonly step: string; }): void {
  const script = resolve(import.meta.dirname, '..', '..', '..', 'zfs', 'request-to-openzfs-kernel-device', 'e2e', 'run-fixture.ts');
  const result = spawnSync('node', [script, step, '--name', chosen.name, '--log-dir', chosen.logDir,], {
    stdio: 'inherit',
    timeout: HOST_COMMAND_TIMEOUT_MS * 60,
  });
  if ((result.error !== undefined) || (result.status !== 0))
    throw result.error ?? new Error(`the guest recipe's ${step} step exited ${String(result.status)}`);
}

/**
 Build this crate's handler inside the guest from a snapshot of the worktree, so the arm compiles against the path
 crates the wrapper links rather than against a copy somebody remembered to add.
 */
async function build(chosen: Options): Promise<void> {
  const context = join(chosen.logDir, 'snapshot');
  rmSync(context, { recursive: true, force: true, });
  mkdirSync(context, { recursive: true, mode: 0o700, });
  await copyWrapperSnapshot({ context, });
  const tarball = join(chosen.logDir, 'subject.tar.gz');
  host({ chosen, command: 'tar', args: ['--create', '--gzip', '--file', tarball, '--exclude=target', '--directory', context, '.'], log: 'host-package.log', });
  host({ chosen, command: 'mvm', args: ['--backend', 'libvirt', 'push', chosen.name, tarball, '/root/subject.tar.gz'], log: 'guest-push.log', });
  const script = [
    'set -eu',
    'if test -f /mnt/shared/subject.tar.gz; then cp /mnt/shared/subject.tar.gz /root/subject.tar.gz; fi',
    `rm --recursive --force ${CRATE_DIR}`,
    `mkdir --parents ${CRATE_DIR}`,
    `tar --extract --gzip --file /root/subject.tar.gz --directory ${CRATE_DIR}`,
    `cd ${CRATE_DIR}/package/git-policy/cli`,
    'export PATH=/root/.cargo/bin:$PATH',
    'cargo build --release --locked --bins > /root/probe-build.log 2>&1',
    'tail --lines=3 /root/probe-build.log',
  ].join('\n');
  guest({ chosen, script, log: 'guest-crate-build.log', timeout: CRATE_BUILD_TIMEOUT_MS, });
}

//endregion Commands

//region Steps

/** The step that must fail before the probe exists: the mode word is not one this build takes. */
function red(chosen: Options): void {
  const script = [
    'set -eu',
    `${HANDLER} start-probe > /root/red.log 2>&1 || status=$?`,
    'status=${status:-0}',
    'cat /root/red.log',
    // A red arm passes only when the refusal is the absence of the mode, and says so.
    'if grep --quiet "start-probe" /root/red.log && grep --quiet "not a mode" /root/red.log; then',
    '  echo "RED the probe mode does not exist yet: $(cat /root/red.log | head --lines=1)"',
    '  exit 0',
    'fi',
    'echo "RED the arm did not meet the absence it expected"',
    'exit 1',
  ].join('\n');
  guest({ chosen, script, log: 'guest-red.log', timeout: ARM_TIMEOUT_MS, });
}

/** Create the pool and the probe dataset the design names, with its empty `ready` snapshot and the configuration. */
function probeDataset(chosen: Options): void {
  const script = [
    'set -eu',
    `export PATH=${OPENZFS_PATH}`,
    `export LD_LIBRARY_PATH=${OPENZFS_PREFIX}/lib`,
    // The pool is a file-backed image of this throwaway guest, and nothing here is on a person's machine.
    `truncate --size ${POOL_IMAGE_SIZE} ${POOL_IMAGE}`,
    `zpool create -f -o cachefile=none -O mountpoint=none ${POOL} ${POOL_IMAGE}`,
    `zfs create -o canmount=off -o mountpoint=none ${POOL}/nosnap`,
    `zfs create -o mountpoint=legacy -o canmount=off -o com.sun:auto-snapshot=false ${PROBE_DATASET}`,
    `zfs set org.monochromatic:git-worktree-probe-source=v1 ${PROBE_DATASET}`,
    `zfs snapshot ${PROBE_DATASET}@ready`,
    `mkdir --parents $(dirname ${CONFIGURATION})`,
    `printf '%s\\n' '{' '  "gitWorktreeProbeDataset": "'${PROBE_DATASET}'",' '  "gitWorktreeCheckout": "'${CHECKOUT}'",' '  "account": "'${ACCOUNT}'"' '}' > ${CONFIGURATION}`,
    `chmod 0644 ${CONFIGURATION}`,
    `mkdir --parents ${CHECKOUT}`,
    `chown ${ACCOUNT}:${ACCOUNT} ${CHECKOUT}`,
    `zfs list -H -o name,mountpoint,canmount ${POOL} ${POOL}/nosnap ${PROBE_DATASET}`,
  ].join('\n');
  guest({ chosen, script, log: 'guest-probe-dataset.log', timeout: ARM_TIMEOUT_MS, });
}

/**
 Run the arms the design's measurements name, all in one guest script so every reading is in one log:
 M1's attach and its `EBUSY` control, M2's table and txg fences, M3's kills and its held deadline, M4's timings,
 M5's refusal controls, M7's scan, M9's accepted release and its filtered refusal, and M10's absent-source reading.
 */
function green(chosen: Options): void {
  const script = [
    'set -eu',
    `export PATH=${OPENZFS_PATH}`,
    `export LD_LIBRARY_PATH=${OPENZFS_PREFIX}/lib`,
    // Every claim is asserted from the record itself, and a missing line fails the arm rather than printing nothing.
    'check() {',
    '  if ! grep --quiet --fixed-strings "$1" "$2"; then',
    '    echo "MISSING $1 in $2"',
    '    cat "$2"',
    '    exit 1',
    '  fi',
    '  echo "PROBE $3"',
    '}',
    'cat /proc/self/mountinfo | sort > /root/mountinfo-before.txt',
    `zpool sync ${POOL}`,
    `cat /proc/spl/kstat/zfs/${POOL}/txgs > /root/txgs-before.txt`,
    't0=$(awk \'NR > 2 { if ($1 > top) top = $1 } END { print top }\' /root/txgs-before.txt)',
    'echo "PROBE start fence $t0"',
    // M4: five interleaved rounds, and the control is the same binary with the probe disabled.
    'for round in 1 2 3 4 5; do',
    '  start=$(date +%s%N)',
    `  ${HANDLER} start-probe > /root/probe-run.log 2>&1 || true`,
    '  end=$(date +%s%N)',
    '  echo "ROUND $round $(( (end - start) / 1000 )) us"',
    `  cp ${RECORD} /root/record-probe.txt`,
    '  control_start=$(date +%s%N)',
    `  CLI_GIT_TEST_ONLY_START_PROBE=disabled ${HANDLER} start-probe > /root/probe-control.log 2>&1 || true`,
    '  control_end=$(date +%s%N)',
    '  echo "CONTROL $round $(( (control_end - control_start) / 1000 )) us"',
    `  cp ${RECORD} /root/record-control.txt`,
    'done',
    'echo "PROBE record after the passing run:"; cat /root/record-probe.txt',
    `check 'verdict passed' /root/record-probe.txt 'verdict passed'`,
    `check 'step attach-exclusive-create passed' /root/record-probe.txt 'attach passed'`,
    `check 'step walk-refusal-controls passed' /root/record-probe.txt 'refusal controls refused'`,
    `check 'step scan-while-held passed' /root/record-probe.txt 'scan saw the worker and the holder'`,
    `check 'step scan-after-holder passed' /root/record-probe.txt 'scan saw the worker alone'`,
    `check 'step move passed' /root/record-probe.txt 'move passed'`,
    `check 'step unmount passed' /root/record-probe.txt 'unmount passed'`,
    `check 'step closing-scan passed' /root/record-probe.txt 'closing scan clear'`,
    `check 'verdict skipped' /root/record-control.txt 'disabled control wrote a skipped record'`,
    'echo "PROBE disabled control record:"; cat /root/record-control.txt',
    // M2: the end fence commits whatever the probe wrote, and every committed row at or above the start row is empty.
    `zpool sync ${POOL}`,
    `cat /proc/spl/kstat/zfs/${POOL}/txgs > /root/txgs-after.txt`,
    'if awk -v t0="$t0" \'NR > 2 && $1 >= t0 && ($4 != 0 || $6 != 0 || $8 != 0) { bad = 1 } NR > 2 && $1 >= t0 { seen = 1 } END { exit !(seen && !bad) }\' /root/txgs-after.txt; then',
    '  echo "PROBE txg window clean"',
    'else',
    '  echo "PROBE txg window held a write"; awk -v t0="$t0" \'NR > 2 && $1 >= t0\' /root/txgs-after.txt; exit 1',
    'fi',
    'cat /proc/self/mountinfo | sort > /root/mountinfo-after.txt',
    'if cmp --silent /root/mountinfo-before.txt /root/mountinfo-after.txt; then echo "PROBE host table unchanged"; else echo "PROBE host table changed"; exit 1; fi',
    'for namespace in /proc/[0-9]*/ns/mnt; do',
    '  if nsenter --mount="$namespace" cat /proc/self/mountinfo 2>/dev/null | grep --quiet "git-worktree-probe@ready"; then echo "PROBE source still mounted in $namespace"; exit 1; fi',
    'done',
    'echo "PROBE source absent afterwards"',
    // M1's held-source control: a second namespace mounts the snapshot, so the exclusive create answers EBUSY.
    'rm --force /root/held.txt',
    'rm --recursive --force /root/held-mount',
    'mkdir --parents /root/held-mount',
    `unshare --mount sh -c '/sbin/mount.zfs ${PROBE_DATASET}@ready /root/held-mount && touch /root/held.txt && sleep 30' &`,
    'for wait in 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20; do test -f /root/held.txt && break; sleep 0.5; done',
    'test -f /root/held.txt',
    `${HANDLER} start-probe > /root/probe-held.log 2>&1 || true`,
    `cp ${RECORD} /root/record-held.txt`,
    `check 'step attach-exclusive-create not-exercised' /root/record-held.txt 'held source not exercised'`,
    `check 'reason source-held' /root/record-held.txt 'held source names its cause'`,
    'pkill --signal KILL --full "^/sbin/mount.zfs" || true',
    'wait || true',
    // M10's absent-source reading: the configuration names a dataset the pool does not hold.
    `sed --in-place 's|${PROBE_DATASET}|${POOL}/nosnap/git-worktree-probe-absent|' ${CONFIGURATION}`,
    `${HANDLER} start-probe > /root/probe-absent.log 2>&1 || true`,
    `cp ${RECORD} /root/record-absent.txt`,
    `check 'source-existed no' /root/record-absent.txt 'absent source recorded'`,
    `sed --in-place 's|${POOL}/nosnap/git-worktree-probe-absent|${PROBE_DATASET}|' ${CONFIGURATION}`,
    // M9's refused arm: a filtered move_mount answers EPERM, which fails that step and refuses the probe.
    `systemd-run --unit=git-worktree-probe-test --quiet --wait --collect --property=SystemCallFilter=~move_mount --property=SystemCallErrorNumber=EPERM ${HANDLER} start-probe || true`,
    `cp ${RECORD} /root/record-refused.txt`,
    `check 'step attach-place failed' /root/record-refused.txt 'refused step attach-place errno EPERM'`,
    `check 'errno EPERM' /root/record-refused.txt 'refused step errno EPERM'`,
    // M3's held arm: a test-only hold after the move makes the deadline fire and names the step in flight.
    `CLI_GIT_TEST_ONLY_START_PROBE=hold-move ${HANDLER} start-probe > /root/probe-hold.log 2>&1 || true`,
    `cp ${RECORD} /root/record-timeout.txt`,
    `check 'verdict timed-out' /root/record-timeout.txt 'timed-out'`,
    `check 'in-flight unmount' /root/record-timeout.txt 'timed-out in flight unmount'`,
    `check 'last-finished move' /root/record-timeout.txt 'timed-out last finished move'`,
    // M3's kill arm: the mode is killed while the worker is held, so the record stays started.
    `env CLI_GIT_TEST_ONLY_START_PROBE=hold-mode ${HANDLER} start-probe > /root/probe-kill.log 2>&1 & mode=$!`,
    'sleep 0.12',
    'kill -KILL "$mode" || true',
    'wait "$mode" 2>/dev/null || true',
    `cp ${RECORD} /root/record-killed.txt`,
    `check 'state started' /root/record-killed.txt 'killed mode left a started record'`,
    'pkill --signal KILL --full "^/opt/git-policy-cli-start-probe/package/git-policy/cli/target/release/mount-git-worktree-clone-datasets start-probe$" || true',
    'sleep 0.2',
    'echo "GREEN done"',
  ].join('\n');
  guest({ chosen, script, log: 'guest-green.log', timeout: ARM_TIMEOUT_MS, });
}

//endregion Steps

const steps: Record<string, (chosen: Options) => Promise<void> | void> = {
  create: (chosen) => recipe({ chosen, step: 'create', }),
  prepare: (chosen) => recipe({ chosen, step: 'prepare', }),
  red: async (chosen: Options): Promise<void> => {
    await build(chosen);
    red(chosen);
  },
  'probe-dataset': probeDataset,
  green: async (chosen: Options): Promise<void> => {
    if (!chosen.noBuild)
      await build(chosen);
    green(chosen);
  },
  destroy: (chosen) => recipe({ chosen, step: 'destroy', }),
  all: async (chosen: Options): Promise<void> => {
    let failure: unknown = null;
    let created = false;
    try {
      recipe({ chosen, step: 'create', });
      created = true;
      recipe({ chosen, step: 'prepare', });
      await build(chosen);
      red(chosen);
      probeDataset(chosen);
      green(chosen);
    } catch (thrown) {
      failure = thrown;
    }
    if (chosen.keepGuest && created)
      console.log(`git-policy probe guest: --keep-guest kept '${chosen.name}' to inspect and destroy`);
    else if (!chosen.keepGuest && created) {
      try {
        recipe({ chosen, step: 'destroy', });
      } catch (destruction) {
        if (failure === null)
          throw destruction;
        console.error(`git-policy probe guest: destroying '${chosen.name}' also failed: ${String(destruction)}`);
      }
    } else if (!chosen.keepGuest)
      console.error(`git-policy probe guest: no guest was destroyed, because this run did not create '${chosen.name}'`);
    if (failure !== null)
      throw failure;
  },
};

const chosen = options();
mkdirSync(chosen.logDir, { recursive: true, mode: 0o700, });
const step = steps[chosen.step];
if (step === undefined)
  throw new Error(`unknown step '${chosen.step}'; use ${Object.keys(steps).join(', ')}`);
await step(chosen);
const greenLog = join(chosen.logDir, 'guest-green.log');
if (chosen.step === 'green' || chosen.step === 'all') {
  const pulled = readFileSync(greenLog, 'utf8');
  for (const marker of GREEN_MARKERS) {
    if (!pulled.includes(marker))
      throw new Error(`the green log lacks ${marker}; the guest ran the arm without proving it`);
  }
}
console.log(`git-policy probe guest: step '${chosen.step}' finished; logs in ${chosen.logDir}`);
