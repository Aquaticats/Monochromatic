#!/usr/bin/env node
/**
 Drive the root start probe's arms in the throwaway `mvm` guest.

 The probe mounts, unshares a mount namespace and reads `/proc/spl/kstat/zfs`, so no control on a person's machine may
 run it. This driver borrows the ZFS binding's guest recipe for the parts that are not the probe's
 (`package/zfs/request-to-openfs-kernel-device/e2e/run-fixture.ts`): it creates the Ubuntu 24.04 guest, prepares it with
 OpenZFS 2.4.4 and the Rust toolchain, then builds this crate's handler inside it, prepares the probe dataset the design
 names, runs the arms, and destroys the guest.

 Usage: node e2e/start-probe-guest.ts <create|prepare|red|probe-dataset|green|no-configuration|m1-key|m1-read-write|m2-controls|m3-kill|m7-scan|m9-busy|timings|timings-boot-first|destroy|all> [--name <guest>]
          [--log-dir <dir>] [--keep-guest]

 `no-configuration` is the project's answer of 2026-11-22 for the probe: with no configuration file the record passes,
its source is the design's own name in the pool the mounted checkout dataset names, and a dataset renamed away leaves
the `ENOENT` record whose line names the two setup commands.

 `red` runs the arm before the probe exists and requires the refusal that says the mode is not one of this build's,
 which is what proves the green run below is a change and not an absence that always passed. `green` requires the
 record's own words: the attach arm passed, the mount the probe made is gone from every namespace afterwards, and a
 filtered `move_mount` fails that step with `EPERM`.

 `m1-key` is M1's unloaded-key control: the fixture's encrypted dataset attaches and passes with its key loaded and
 answers `EACCES` from `fsconfig(create)` with the key unloaded. `m1-read-write` is M1's read-only against read-write
 control: a read-only request for the probe dataset beside a read-write superblock of it answers `EBUSY`, while the
 probe's read-only attachment of that dataset's `ready` snapshot passes, because a snapshot is the documented
 exception.

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
/**
 The fixture's encrypted dataset, whose empty `ready` snapshot is the source of M1's unloaded-key control.
 It is never the configured source except inside that control, which restores the configuration afterwards.
 */
const PROBE_ENCRYPTED = 'zpcachyos/nosnap/git-worktree-probe-encrypted';
/** The passphrase file the encrypted fixture dataset's key is loaded from, which only the guest's root reads. */
const PROBE_KEY_FILE = '/root/git-worktree-probe-key';
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

/**
 The project's answer of 2026-11-22 for the probe: with no configuration file the probe finds its dataset by its
 design's own name in the pool the mounted checkout dataset names, and its checkout as the sweep finds one, so a
 missing file no longer leaves the record blocked. The dataset is then renamed away, and the same run reads the
 `ENOENT` refusal the design keeps, with the mode's line naming the two commands plan step 19's setup runs.
 */
function noConfiguration(chosen: Options): void {
  const script = probeScript([
    'check() {',
    '  if ! grep --quiet --fixed-strings "$1" "$2"; then',
    '    echo "MISSING $1 in $2"',
    '    cat "$2"',
    '    exit 1',
    '  fi',
    '}',
    'cp $CONF /root/no-configuration-config.jsonc',
    // Every exit puts the configuration back, so a failing check leaves the next arm a file to read.
    "trap 'cp /root/no-configuration-config.jsonc $CONF' EXIT",
    'rm --force $CONF',
    '$H start-probe > /root/no-configuration-run.log 2>&1 || true',
    'cat /root/no-configuration-run.log',
    'cp $REC /root/no-configuration-record.txt',
    'cat /root/no-configuration-record.txt',
    `check 'verdict passed' /root/no-configuration-record.txt`,
    'echo "NO-CONFIGURATION passed-with-no-file"',
    `check 'source ${PROBE_DATASET}@ready' /root/no-configuration-record.txt`,
    'echo "NO-CONFIGURATION source-is-the-designs-name"',
    `zfs rename ${PROBE_DATASET} ${PROBE_DATASET}-absent`,
    '$H start-probe > /root/no-configuration-absent-run.log 2>&1 || true',
    'cat /root/no-configuration-absent-run.log',
    'cp $REC /root/no-configuration-absent-record.txt',
    'cat /root/no-configuration-absent-record.txt',
    `check 'verdict blocked' /root/no-configuration-absent-record.txt`,
    'echo "NO-CONFIGURATION absent-dataset-blocks"',
    `check 'step attach-exclusive-create not-exercised errno ENOENT reason missing-source' /root/no-configuration-absent-record.txt`,
    'echo "NO-CONFIGURATION absent-dataset-names-its-cause"',
    'if grep --quiet --fixed-strings "zfs create" /root/no-configuration-absent-run.log && grep --quiet --fixed-strings "zfs snapshot" /root/no-configuration-absent-run.log; then',
    '  echo "NO-CONFIGURATION absent-dataset-names-the-setup-commands"',
    'else',
    '  echo "MISSING the setup commands in the mode line"; cat /root/no-configuration-absent-run.log; exit 1',
    'fi',
    `zfs rename ${PROBE_DATASET}-absent ${PROBE_DATASET}`,
    'echo "NO-CONFIGURATION done"',
  ]);
  guest({ chosen, script, log: 'guest-no-configuration.log', timeout: ARM_TIMEOUT_MS, });
}

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
    // M1's unloaded-key control needs an encrypted source whose key the step can unload and load again.
    `printf '%s' 'correct horse battery staple' > ${PROBE_KEY_FILE}`,
    `chmod 0600 ${PROBE_KEY_FILE}`,
    `zfs create -o encryption=on -o keyformat=passphrase -o keylocation=file://${PROBE_KEY_FILE} -o mountpoint=legacy -o canmount=off -o com.sun:auto-snapshot=false ${PROBE_ENCRYPTED}`,
    `zfs set org.monochromatic:git-worktree-probe-source=v1 ${PROBE_ENCRYPTED}`,
    `zfs snapshot ${PROBE_ENCRYPTED}@ready`,
    `zfs snapshot ${PROBE_DATASET}@ready`,
    `mkdir --parents $(dirname ${CONFIGURATION})`,
    `printf '%s\\n' '{' '  "startProbe": {' '    "dataset": "'${PROBE_DATASET}'",' '    "checkout": "'${CHECKOUT}'"' '  },' '  "account": "'${ACCOUNT}'"' '}' > ${CONFIGURATION}`,
    `chmod 0644 ${CONFIGURATION}`,
    // The checkout the design names is a mounted dataset, so a probe with no configuration file finds it the way the
    // sweep does (doc/planning/cli-git-zfs-start-probe-design.md, "The source: the probe dataset").
    `zfs create -o mountpoint=none ${POOL}/git-worktree-checkout`,
    `zfs create -o mountpoint=${CHECKOUT} ${POOL}/git-worktree-checkout/monochromatic-git-checkout`,
    `chown ${ACCOUNT}:${ACCOUNT} ${CHECKOUT}`,
    `zfs list -H -o name,mountpoint,canmount,encryption ${POOL} ${POOL}/nosnap ${PROBE_DATASET} ${PROBE_ENCRYPTED}`,
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
    `sed --in-place 's|${POOL}/nosnap/git-worktree-probe-absent|${PROBE_DATASET}|' ${CONFIGURATION}`,
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
    `check 'errno EBUSY' /root/record-held.txt 'held source answers EBUSY'`,
    'pkill --signal KILL --full "^/sbin/mount.zfs" || true',
    'wait || true',
    // M10's absent-source reading: the configuration names a dataset the pool does not hold.
    `sed --in-place 's|${PROBE_DATASET}|${POOL}/nosnap/git-worktree-probe-absent|' ${CONFIGURATION}`,
    `${HANDLER} start-probe > /root/probe-absent.log 2>&1 || true`,
    `sed --in-place 's|${POOL}/nosnap/git-worktree-probe-absent|${PROBE_DATASET}|' ${CONFIGURATION}`,
    `cp ${RECORD} /root/record-absent.txt`,
    `check 'source-existed no' /root/record-absent.txt 'absent source recorded'`,
    `check 'reason missing-source' /root/record-absent.txt 'absent source names its cause'`,
    `check 'errno ENOENT' /root/record-absent.txt 'absent source answers ENOENT'`,
    `check 'verdict blocked' /root/record-absent.txt 'absent source blocks the route'`,
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
    // M3's kill arm: hold-unshare parks the worker before any mount,
    // then a kill of the mode leaves the started record and no worker mount.
    // The wait is on the record the mode writes before its fork, bounded, because a fixed sleep races that write:
    // the integration pass of 2026-11-22 saw this control fail in 5 of 10 runs for exactly that reason.
    `CLI_GIT_TEST_ONLY_START_PROBE=hold-unshare ${HANDLER} start-probe > /root/probe-kill.log 2>&1 & mode=$!`,
    'started=false',
    'for attempt in 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31 32 33 34 35 36 37 38 39 40; do',
    `  if grep --quiet --fixed-strings "state started" ${RECORD} 2>/dev/null; then started=true; break; fi`,
    '  sleep 0.05',
    'done',
    // A mode that never wrote the started record is a failure of this control rather than of the kill, and it names
    // itself instead of surfacing as a missing record line later.
    `test "$started" = true || { echo "MISSING the started record before the kill"; cat ${RECORD} 2>/dev/null; exit 1; }`,
    'kill -KILL "$mode" || true',
    'wait "$mode" 2>/dev/null || true',
    `pkill --signal KILL --full "^${HANDLER} start-probe$" || true`,
    'sleep 0.2',
    `cp ${RECORD} /root/record-killed.txt`,
    `check 'state started' /root/record-killed.txt 'killed mode left a started record'`,
    'echo "GREEN done"',
  ].join('\n');
  guest({ chosen, script, log: 'guest-green.log', timeout: ARM_TIMEOUT_MS, });
}


//region Added guest arms

/** The lines the no-configuration arm must find, one per claim it makes. */
const NO_CONFIGURATION_MARKERS = [
  'NO-CONFIGURATION passed-with-no-file',
  'NO-CONFIGURATION source-is-the-designs-name',
  'NO-CONFIGURATION absent-dataset-blocks',
  'NO-CONFIGURATION absent-dataset-names-its-cause',
  'NO-CONFIGURATION absent-dataset-names-the-setup-commands',
  'NO-CONFIGURATION done',
] as const;

/** The shell header every added arm shares: the handler, the record, the source and the leak counter. */
function probeScript(lines: readonly string[]): string {
  return [
    'set -eu',
    `export PATH=${OPENZFS_PATH}`,
    `export LD_LIBRARY_PATH=${OPENZFS_PREFIX}/lib`,
    `H=${HANDLER}`,
    `REC=${RECORD}`,
    `CONF=${CONFIGURATION}`,
    `SRC='${PROBE_DATASET}@ready'`,
    `PAT='^${HANDLER} start-probe$'`,
    'now_us() { echo $(( $(date +%s%N) / 1000 )); }',
    'leaks() { n=0; for ns in /proc/[0-9]*/ns/mnt; do if nsenter --mount="$ns" cat /proc/self/mountinfo 2>/dev/null | grep -q "git-worktree-probe@ready"; then n=$((n+1)); fi; done; echo "leaks=$n"; }',
    ...lines,
  ].join('\n');
}

/** The shell assertion both M1 controls share: a record line must be there, and its absence fails the arm. */
const RECORD_LINE_ASSERTION = [
  'check() {',
  '  if ! grep --quiet --fixed-strings "$1" "$2"; then',
  '    echo "MISSING $1 in $2"',
  '    cat "$2"',
  '    exit 1',
  '  fi',
  '  echo "M1 $3"',
  '}',
] as const;

/**
 M1's unloaded-key control: the encrypted fixture dataset's `ready` snapshot attaches while its key is loaded,
 answers `EACCES` from `fsconfig(create)` once the key is unloaded, and attaches again once the key and the
 configuration are back, so the green arm's attach is a reading of the key path rather than of a source that always
 attaches.
 */
function m1Key(chosen: Options): void {
  const script = probeScript([
    ...RECORD_LINE_ASSERTION,
    'cp $CONF /root/m1-key-config.jsonc',
    `sed --in-place 's|${PROBE_DATASET}|${PROBE_ENCRYPTED}|' $CONF`,
    'echo "M1 source: $(grep --fixed-strings dataset $CONF)"',
    '$H start-probe > /root/m1-key-loaded.log 2>&1 || true',
    'cp $REC /root/m1-key-loaded-record.txt',
    `check 'verdict passed' /root/m1-key-loaded-record.txt 'key-loaded-passes'`,
    `check 'step attach-exclusive-create passed' /root/m1-key-loaded-record.txt 'key-loaded-creates-a-fresh-superblock'`,
    `check 'source-existed yes' /root/m1-key-loaded-record.txt 'key-loaded-finds-the-source'`,
    `zfs unload-key ${PROBE_ENCRYPTED}`,
    `keystatus=$(zfs get -H -o value keystatus ${PROBE_ENCRYPTED})`,
    'echo "M1 unloaded keystatus=$keystatus"',
    'test "$keystatus" = unavailable',
    '$H start-probe > /root/m1-key-unloaded.log 2>&1 || true',
    'cp $REC /root/m1-key-unloaded-record.txt',
    `check 'verdict failed' /root/m1-key-unloaded-record.txt 'key-unloaded-fails'`,
    `check 'step attach-exclusive-create failed errno EACCES' /root/m1-key-unloaded-record.txt 'key-unloaded-exclusive-create-EACCES'`,
    `check 'arm attach failed errno EACCES' /root/m1-key-unloaded-record.txt 'key-unloaded-attach-EACCES'`,
    'cp /root/m1-key-config.jsonc $CONF',
    `zfs load-key ${PROBE_ENCRYPTED}`,
    `keystatus=$(zfs get -H -o value keystatus ${PROBE_ENCRYPTED})`,
    'echo "M1 restored keystatus=$keystatus"',
    'test "$keystatus" = available',
    '$H start-probe > /root/m1-key-restored.log 2>&1 || true',
    'cp $REC /root/m1-key-restored-record.txt',
    `check 'verdict passed' /root/m1-key-restored-record.txt 'key-and-configuration-restored-passes'`,
    'echo "M1 key control done"',
  ]);
  guest({ chosen, script, log: 'guest-m1-key.log', timeout: ARM_TIMEOUT_MS, });
}

/**
 M1's read-only against read-write control: a read-only request for the probe dataset beside a read-write superblock
 of it answers `EBUSY` (`module/os/linux/zfs/zpl_super.c:936` to `:943`), while the probe's read-only attachment of
 that dataset's `ready` snapshot still passes, because a snapshot is that rule's exception, so the arm's flags need no
 adjustment.
 */
function m1ReadWrite(chosen: Options): void {
  const script = probeScript([
    ...RECORD_LINE_ASSERTION,
    `DATASET='${PROBE_DATASET}'`,
    'sort /proc/self/mountinfo | sed "s/^[0-9]* //" > /root/m1-rw-table-before.txt',
    'mkdir --parents /root/m1-rw-mount /root/m1-ro-mount',
    'mount -t zfs "$DATASET" /root/m1-rw-mount',
    'if awk \'$5 == "/root/m1-rw-mount" { print $6 }\' /proc/self/mountinfo | grep --quiet "^rw,"; then echo "M1 read-write-superblock-present"; else echo "M1 read-write-superblock-missing"; grep m1-rw-mount /proc/self/mountinfo; exit 1; fi',
    'if mount -o ro -t zfs "$DATASET" /root/m1-ro-mount > /root/m1-ro-mount.log 2>&1; then status=0; else status=$?; fi',
    'cat /root/m1-ro-mount.log; echo',
    'echo "M1 read-only-request-exit=$status"',
    'test "$status" -ne 0',
    'if grep --quiet --ignore-case busy /root/m1-ro-mount.log; then echo "M1 read-only-against-read-write-answers-EBUSY"; else echo "M1 read-only-request-answered-something-else"; exit 1; fi',
    '$H start-probe > /root/m1-rw-run.log 2>&1 || true',
    'cp $REC /root/m1-rw-record.txt',
    `check 'verdict passed' /root/m1-rw-record.txt 'snapshot-attach-passes-beside-a-read-write-superblock'`,
    `check 'step attach-exclusive-create passed' /root/m1-rw-record.txt 'snapshot-creates-a-fresh-superblock-beside-it'`,
    `check 'source-existed yes' /root/m1-rw-record.txt 'snapshot-finds-the-source-beside-it'`,
    'umount /root/m1-rw-mount',
    'rmdir /root/m1-rw-mount /root/m1-ro-mount',
    'sort /proc/self/mountinfo | sed "s/^[0-9]* //" > /root/m1-rw-table-after.txt',
    'if cmp --silent /root/m1-rw-table-before.txt /root/m1-rw-table-after.txt; then echo "M1 table-restored"; else echo "M1 table-not-restored"; exit 1; fi',
    'if test "$(leaks | tail --lines=1)" = "leaks=0"; then echo "M1 no-namespace-lists-the-source"; else echo "M1 a-namespace-still-lists-the-source"; exit 1; fi',
    'echo "M1 read-write control done"',
  ]);
  guest({ chosen, script, log: 'guest-m1-read-write.log', timeout: ARM_TIMEOUT_MS, });
}

/** M2's controls: an idle window, a deliberate mount, a deliberate write and a deliberate read. */
function m2Controls(chosen: Options): void {
  const script = probeScript([
    'sort /proc/self/mountinfo | sed "s/^[0-9]* //" > /root/m2-table-before.txt',
    `zpool sync ${POOL}`,
    `cat /proc/spl/kstat/zfs/${POOL}/txgs > /root/m2-idle-before.txt`,
    't0=$(awk \'NR > 2 { if ($1 > top) top = $1 } END { print top }\' /root/m2-idle-before.txt)',
    'sleep 2',
    `zpool sync ${POOL}`,
    `cat /proc/spl/kstat/zfs/${POOL}/txgs > /root/m2-idle-after.txt`,
    'if awk -v t0="$t0" \'NR > 2 && $1 >= t0 && ($4 != 0 || $6 != 0 || $8 != 0) { bad = 1 } NR > 2 && $1 >= t0 { seen = 1 } END { exit !(seen && !bad) }\' /root/m2-idle-after.txt; then echo "M2 idle-window-clean t0=$t0"; else echo "M2 idle-window-held-a-write"; exit 1; fi',
    'mkdir --parents /root/m2-mount',
    `mount -t zfs "$SRC" /root/m2-mount`,
    'sort /proc/self/mountinfo | sed "s/^[0-9]* //" > /root/m2-table-mounted.txt',
    'if cmp --silent /root/m2-table-before.txt /root/m2-table-mounted.txt; then echo "M2 deliberate-mount-not-seen"; exit 1; else echo "M2 deliberate-mount-seen"; fi',
    'umount /root/m2-mount',
    'sort /proc/self/mountinfo | sed "s/^[0-9]* //" > /root/m2-table-restored.txt',
    'if cmp --silent /root/m2-table-before.txt /root/m2-table-restored.txt; then echo "M2 deliberate-mount-restored"; else echo "M2 mount-table-not-restored"; exit 1; fi',
    `zfs create -o mountpoint=legacy ${POOL}/nosnap/m2-write 2>/dev/null || true`,
    'mkdir --parents /root/m2-write',
    `mount -t zfs ${POOL}/nosnap/m2-write /root/m2-write`,
    `zpool sync ${POOL}`,
    `cat /proc/spl/kstat/zfs/${POOL}/txgs > /root/m2-write-before.txt`,
    't0=$(awk \'NR > 2 { if ($1 > top) top = $1 } END { print top }\' /root/m2-write-before.txt)',
    'echo write > /root/m2-write/file',
    `zpool sync ${POOL}`,
    `cat /proc/spl/kstat/zfs/${POOL}/txgs > /root/m2-write-after.txt`,
    'if awk -v t0="$t0" \'NR > 2 && $1 >= t0 && ($4 != 0 || $6 != 0 || $8 != 0) { seen = 1 } END { exit !seen }\' /root/m2-write-after.txt; then echo "M2 deliberate-write-seen"; else echo "M2 deliberate-write-not-seen"; exit 1; fi',
    'umount /root/m2-write',
    `zfs destroy ${POOL}/nosnap/m2-write`,
    `zpool sync ${POOL}`,
    `cat /proc/spl/kstat/zfs/${POOL}/txgs > /root/m2-read-before.txt`,
    't0=$(awk \'NR > 2 { if ($1 > top) top = $1 } END { print top }\' /root/m2-read-before.txt)',
    `unshare --mount sh -c "mkdir -p /root/m2-read; mount -t zfs -o ro ${PROBE_DATASET}@ready /root/m2-read; ls -a /root/m2-read > /dev/null; umount /root/m2-read"`,
    `zpool sync ${POOL}`,
    `cat /proc/spl/kstat/zfs/${POOL}/txgs > /root/m2-read-after.txt`,
    'if awk -v t0="$t0" \'NR > 2 && $1 >= t0 && ($4 != 0 || $6 != 0 || $8 != 0) { bad = 1 } NR > 2 && $1 >= t0 { seen = 1 } END { exit !(seen && !bad) }\' /root/m2-read-after.txt; then echo "M2 deliberate-read-clean"; else echo "M2 deliberate-read-held-a-write"; exit 1; fi',
    'echo "M2 controls done"',
  ]);
  guest({ chosen, script, log: 'guest-m2-controls.log', timeout: ARM_TIMEOUT_MS, });
}

/** M3's kill matrix: every named hold point, each process alone, and the raised-deadline control. */
function m3Kill(chosen: Options): void {
  const script = probeScript([
    'sort /proc/self/mountinfo | sed "s/^[0-9]* //" > /root/m3-table-before.txt',
    'rm -f /root/m3-summary.txt',
    'check_after() { label="$1"; sleep 1.2; sort /proc/self/mountinfo | sed "s/^[0-9]* //" > /root/m3-table-after.txt; if cmp -s /root/m3-table-before.txt /root/m3-table-after.txt; then t=table-same; else t=TABLE-DIFFERS; fi; l=$(leaks | tail -n 1); left=$(pgrep -f "$PAT" | wc -l); st=$(grep -m1 "^state " $REC | cut -d" " -f2); $H start-probe > /root/m3-next.log 2>&1 || true; if grep -q "^verdict passed" $REC; then nx=next-run-passed; else nx=NEXT-RUN-DID-NOT-PASS; fi; echo "KILL $label $t $l leftover-processes=$left record-state-after-kill=$st $nx" | tee --append /root/m3-summary.txt; pkill -KILL -f "$PAT" || true; }',
    'for step in unshare propagate-private scratch-tmpfs scratch-tree walk-slot walk-refusal-controls attach-context attach-source attach-options attach-exclusive-create attach-mount-descriptor attach-place verify-mount scan-while-held scan-after-holder move unmount; do',
    '  CLI_GIT_TEST_ONLY_START_PROBE=hold-$step $H start-probe > /root/m3-control.log 2>&1 || true',
    '  echo "CONTROL hold-$step last-finished=$(grep "^last-finished " $REC | cut -d" " -f2) in-flight=$(grep "^in-flight " $REC | cut -d" " -f2) verdict=$(grep "^verdict " $REC | cut -d" " -f2)"',
    'done',
    'for step in unshare propagate-private scratch-tmpfs scratch-tree walk-slot walk-refusal-controls attach-context attach-source attach-options attach-exclusive-create attach-mount-descriptor attach-place verify-mount scan-while-held scan-after-holder move unmount; do',
    '  for target in mode worker both holder child; do',
    '    CLI_GIT_TEST_ONLY_START_PROBE=hold-$step $H start-probe > /root/m3-run.log 2>&1 &',
    '    mode=$!',
    '    sleep 0.2',
    '    worker=$(pgrep -P $mode -o || true)',
    '    holder=$(pgrep -P $mode -n || true)',
    '    child=""; if test -n "$worker"; then child=$(pgrep -P $worker -n || true); fi',
    '    case $target in',
    '      mode) kill -KILL $mode 2>/dev/null || true ;;',
    '      worker) test -n "$worker" && kill -KILL $worker 2>/dev/null || true ;;',
    '      both) kill -KILL $mode 2>/dev/null || true; test -n "$worker" && kill -KILL $worker 2>/dev/null || true ;;',
    '      holder) if test -n "$holder" && test "$holder" != "$worker"; then kill -KILL $holder 2>/dev/null || true; else echo "SKIP hold-$step holder no-holder-yet" | tee --append /root/m3-summary.txt; fi ;;',
    '      child) if test -n "$child"; then kill -KILL $child 2>/dev/null || true; else echo "SKIP hold-$step child no-child-yet" | tee --append /root/m3-summary.txt; fi ;;',
    '    esac',
    '    wait $mode 2>/dev/null || true',
    '    check_after "hold-$step kill-$target"',
    '  done',
    'done',
    'CLI_GIT_TEST_ONLY_START_PROBE=hold-unmount CLI_GIT_TEST_ONLY_START_PROBE_DEADLINE_MS=900000 $H start-probe > /root/m3-raised.log 2>&1 || true',
    'if grep -q "^verdict passed" $REC && grep -q "^step unmount passed" $REC; then echo "M3 raised-deadline-control-passed"; else echo "M3 raised-deadline-control-failed"; cat $REC; exit 1; fi',
    '$H start-probe > /root/m3-run.log 2>&1 & mode=$!; kill -KILL $mode 2>/dev/null || true; wait $mode 2>/dev/null || true; check_after "kill-immediately mode"',
    'if grep -q TABLE-DIFFERS /root/m3-summary.txt; then echo "M3 a kill changed the host table"; exit 1; fi',
    'if grep -Eq "leaks=[1-9]" /root/m3-summary.txt; then echo "M3 a kill left the source in a namespace"; exit 1; fi',
    'echo "M3 kill matrix done"',
  ]);
  guest({ chosen, script, log: 'guest-m3-kill.log', timeout: 1_500_000, });
}

/** M7's failure branches: a mounted source before the run, a late mount during it, and a pinned empty namespace. */
function m7Scan(chosen: Options): void {
  const script = probeScript([
    '$H start-probe > /root/m7-baseline.log 2>&1 || true',
    'if grep -q "^step scan-while-held passed" $REC && grep -q "^step scan-after-holder passed" $REC && grep -q "^step closing-scan passed" $REC; then echo "M7 baseline-scans-passed"; else echo "M7 baseline-scans-failed"; cat $REC; exit 1; fi',
    'mkdir --parents /root/m7-held-mount',
    `unshare --mount sh -c 'mount -t zfs ${PROBE_DATASET}@ready /root/m7-held-mount && touch /root/m7-held.txt && sleep 30' &`,
    'for wait in 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20; do test -f /root/m7-held.txt && break; sleep 0.5; done',
    'test -f /root/m7-held.txt',
    '$H start-probe > /root/m7-held.log 2>&1 || true',
    'if grep -q "^step attach-exclusive-create not-exercised" $REC && grep -q "reason source-held" $REC && grep -q "errno EBUSY" $REC; then echo "M7 held-source-not-exercised"; else echo "M7 held-source-not-seen"; cat $REC; exit 1; fi',
    'pkill --signal KILL --full "^/sbin/mount.zfs" || true',
    'wait || true',
    'mkdir --parents /root/m7-late',
    // The late mount must land after the probe's own attach and before its first scan, so the arm waits for the
    // probe's mount to appear in a namespace instead of sleeping a fixed time: the hold after the exclusive create
    // then keeps the window open for 700 ms, which is what the fixed 0.2 s sleep used to race against.
    'CLI_GIT_TEST_ONLY_START_PROBE=hold-attach-exclusive-create CLI_GIT_TEST_ONLY_START_PROBE_DEADLINE_MS=900000 $H start-probe > /root/m7-late-run.log 2>&1 & probe=$!',
    'attached=false',
    'for attempt in 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31 32 33 34 35 36 37 38 39 40; do',
    '  if test "$(leaks)" != "leaks=0"; then attached=true; break; fi',
    '  sleep 0.05',
    'done',
    'test "$attached" = true || { echo "M7 the probe never attached"; cat /root/m7-late-run.log; exit 1; }',
    `unshare --mount sh -c "mount -t zfs ${PROBE_DATASET}@ready /root/m7-late; exec sleep 30" > /dev/null 2>&1 & late=$!`,
    'wait $probe || true',
    'kill -KILL $late 2>/dev/null || true',
    'wait $late 2>/dev/null || true',
    'if grep -q "^step scan-while-held failed" $REC; then echo "M7 late-mount-failed-the-first-pass"; else echo "M7 late-mount-not-seen"; cat $REC; exit 1; fi',
    'mkdir --parents /root/m7-empty',
    'unshare --mount sh -c "exec sleep 30" > /dev/null 2>&1 & pinned=$!',
    'exec 9</proc/$pinned/ns/mnt',
    'kill -KILL $pinned 2>/dev/null || true',
    'wait $pinned 2>/dev/null || true',
    '$H start-probe > /root/m7-empty-run.log 2>&1 || true',
    'exec 9<&-',
    'if grep -q "^verdict passed" $REC; then echo "M7 empty-pinned-namespace-invisible"; else echo "M7 empty-pinned-namespace-named"; cat $REC; exit 1; fi',
    'echo "M7 scan arms done"',
  ]);
  guest({ chosen, script, log: 'guest-m7-scan.log', timeout: ARM_TIMEOUT_MS, });
}

/** F10's gate refusal: the real gate refuses in a unit without `/dev/zfs`, and the mode writes a skipped record. */
function gateSkip(chosen: Options): void {
  const script = probeScript([
    'rm -f $REC',
    'sort /proc/self/mountinfo | sed "s/^[0-9]* //" > /root/skip-table-before.txt',
    'systemd-run --unit=git-worktree-probe-skip-test --quiet --wait --collect --pipe --property=InaccessiblePaths=/dev/zfs $H start-probe > /root/skip-run.log 2>&1; echo "SKIP exit=$?"',
    'cat /root/skip-run.log',
    'cat $REC',
    'if grep -q "^verdict skipped" $REC && grep -q "^skip-reason device" $REC; then echo "SKIP gate-refusal-skipped"; else echo "SKIP gate-refusal-not-skipped"; exit 1; fi',
    'sort /proc/self/mountinfo | sed "s/^[0-9]* //" > /root/skip-table-after.txt',
    'if cmp --silent /root/skip-table-before.txt /root/skip-table-after.txt; then echo "SKIP host-table-unchanged"; else echo "SKIP host-table-changed"; exit 1; fi',
    'leaks | tail -n 1',
    'echo "SKIP done"',
  ]);
  guest({ chosen, script, log: 'guest-skip.log', timeout: ARM_TIMEOUT_MS, });
}

/** M9's extra-descriptor fixture: the unmount must answer EBUSY with the switch on and pass with it off. */
function m9Busy(chosen: Options): void {
  const script = probeScript([
    'rm -f $REC',
    'CLI_GIT_TEST_ONLY_START_PROBE_KEEP_DESCRIPTOR=1 $H start-probe > /root/m9-on.log 2>&1 || true',
    'cp $REC /root/m9-on-record.txt',
    'if grep -q "^step unmount failed errno EBUSY" /root/m9-on-record.txt; then echo "M9 extra-descriptor-unmount-EBUSY"; else echo "M9 extra-descriptor-not-seen"; cat /root/m9-on-record.txt; exit 1; fi',
    '$H start-probe > /root/m9-off.log 2>&1 || true',
    'cp $REC /root/m9-off-record.txt',
    'if grep -q "^verdict passed" /root/m9-off-record.txt; then echo "M9 switch-off-passes"; else echo "M9 switch-off-did-not-pass"; cat /root/m9-off-record.txt; exit 1; fi',
    'echo "M9 fixture done"',
  ]);
  guest({ chosen, script, log: 'guest-m9-busy.log', timeout: ARM_TIMEOUT_MS, });
}

/** Warm runs, cache-dropped runs and a guest with many mount namespaces, every round kept. */
function timings(chosen: Options): void {
  const script = probeScript([
    'echo "TIMING start $(date --iso-8601=seconds)"',
    'for round in 1 2 3 4 5; do',
    '  p0=$(now_us); $H start-probe > /root/timing-probe.log 2>&1 || true; p1=$(now_us); pv=$(grep "^verdict " $REC | cut -d" " -f2)',
    '  c0=$(now_us); CLI_GIT_TEST_ONLY_START_PROBE=disabled $H start-probe > /root/timing-control.log 2>&1 || true; c1=$(now_us); cv=$(grep "^verdict " $REC | cut -d" " -f2)',
    '  echo "WARM round=$round probe=$((p1-p0)) control=$((c1-c0)) worker-estimate=$(( (p1-p0) - (c1-c0) )) probe-verdict=$pv control-verdict=$cv"',
    'done',
    'for round in 1 2 3; do',
    `  zpool export ${POOL}`,
    '  sync; echo 3 > /proc/sys/vm/drop_caches',
    `  zpool import -d /home -o cachefile=none ${POOL}`,
    '  sync; echo 3 > /proc/sys/vm/drop_caches',
    '  p0=$(now_us); $H start-probe > /root/timing-cold.log 2>&1 || true; p1=$(now_us); pv=$(grep "^verdict " $REC | cut -d" " -f2)',
    '  c0=$(now_us); CLI_GIT_TEST_ONLY_START_PROBE=disabled $H start-probe > /root/timing-control.log 2>&1 || true; c1=$(now_us); cv=$(grep "^verdict " $REC | cut -d" " -f2)',
    '  echo "COLD round=$round probe=$((p1-p0)) control=$((c1-c0)) worker-estimate=$(( (p1-p0) - (c1-c0) )) probe-verdict=$pv control-verdict=$cv"',
    '  p0=$(now_us); $H start-probe > /root/timing-after-cold.log 2>&1 || true; p1=$(now_us); pv=$(grep "^verdict " $REC | cut -d" " -f2)',
    '  echo "AFTER-COLD round=$round probe=$((p1-p0)) probe-verdict=$pv"',
    'done',
    'ns_make() { count=$1; i=0; while test $i -lt $count; do unshare --mount sh -c "j=0; while test \\$j -lt 20; do mkdir -p /mnt/t\\$j; mount -t tmpfs t /mnt/t\\$j; j=\\$((j+1)); done; exec sleep 900" > /dev/null 2>&1 & i=$((i+1)); done; sleep 2; }',
    'echo "SCALE namespaces=$(for p in /proc/[0-9]*/ns/mnt; do readlink $p; done 2>/dev/null | sort -u | wc -l)"',
    'ns_make 100',
    'for round in 1 2 3 4 5; do',
    '  p0=$(now_us); $H start-probe > /root/timing-scale.log 2>&1 || true; p1=$(now_us); pv=$(grep "^verdict " $REC | cut -d" " -f2)',
    '  c0=$(now_us); CLI_GIT_TEST_ONLY_START_PROBE=disabled $H start-probe > /root/timing-control.log 2>&1 || true; c1=$(now_us); cv=$(grep "^verdict " $REC | cut -d" " -f2)',
    '  echo "MANY-NS namespaces=$(for p in /proc/[0-9]*/ns/mnt; do readlink $p; done 2>/dev/null | sort -u | wc -l) round=$round probe=$((p1-p0)) control=$((c1-c0)) worker-estimate=$(( (p1-p0) - (c1-c0) )) probe-verdict=$pv control-verdict=$cv"',
    'done',
    'ns_make 200',
    'for round in 1 2 3 4 5; do',
    '  p0=$(now_us); $H start-probe > /root/timing-scale.log 2>&1 || true; p1=$(now_us); pv=$(grep "^verdict " $REC | cut -d" " -f2)',
    '  c0=$(now_us); CLI_GIT_TEST_ONLY_START_PROBE=disabled $H start-probe > /root/timing-control.log 2>&1 || true; c1=$(now_us); cv=$(grep "^verdict " $REC | cut -d" " -f2)',
    '  echo "MANY-NS namespaces=$(for p in /proc/[0-9]*/ns/mnt; do readlink $p; done 2>/dev/null | sort -u | wc -l) round=$round probe=$((p1-p0)) control=$((c1-c0)) worker-estimate=$(( (p1-p0) - (c1-c0) )) probe-verdict=$pv control-verdict=$cv"',
    'done',
    'pkill -x sleep || true',
    'echo "TIMING done"',
  ]);
  guest({ chosen, script, log: 'guest-timings.log', timeout: 1_500_000, });
}

/** Reboot the guest, then keep the first probe run after that boot and five warm rounds after it. */
function timingsBootFirst(chosen: Options): void {
  spawnSync('mvm', ['--backend', 'libvirt', 'exec', chosen.name, '--', 'systemctl reboot'], { stdio: 'ignore', timeout: 60_000, });
  let ready = false;
  for (let attempt = 0; attempt < 180; attempt += 1) {
    const probe = spawnSync('mvm', ['--backend', 'libvirt', 'exec', chosen.name, '--', 'true'], { stdio: 'ignore', timeout: 10_000, });
    if (probe.status === 0) {
      ready = true;
      break;
    }
    spawnSync('sleep', ['1'], { stdio: 'ignore', });
  }
  if (!ready)
    throw new Error(`the guest ${chosen.name} did not answer after its reboot`);
  const script = probeScript([
    'echo "BOOT-FIRST start $(date --iso-8601=seconds)"',
    'systemctl stop zfs-zed.service zfs-mount.service zfs-load-module.service 2>/dev/null || true',
    'if test -d /sys/module/zfs; then modprobe --remove zfs || true; fi',
    'if test -d /sys/module/spl; then modprobe --remove spl || true; fi',
    'insmod /lib/modules/$(uname --kernel-release)/extra/spl.ko',
    'insmod /lib/modules/$(uname --kernel-release)/extra/zfs.ko',
    `zpool import -d /home -o cachefile=none ${POOL}`,
    'p0=$(now_us); $H start-probe > /root/boot-first.log 2>&1 || true; p1=$(now_us)',
    'echo "BOOT-FIRST probe=$((p1-p0)) verdict=$(grep "^verdict " $REC | cut -d" " -f2)"',
    'for round in 1 2 3 4 5; do',
    '  p0=$(now_us); $H start-probe > /root/timing-probe.log 2>&1 || true; p1=$(now_us); pv=$(grep "^verdict " $REC | cut -d" " -f2)',
    '  c0=$(now_us); CLI_GIT_TEST_ONLY_START_PROBE=disabled $H start-probe > /root/timing-control.log 2>&1 || true; c1=$(now_us); cv=$(grep "^verdict " $REC | cut -d" " -f2)',
    '  echo "AFTER-BOOT round=$round probe=$((p1-p0)) control=$((c1-c0)) worker-estimate=$(( (p1-p0) - (c1-c0) )) probe-verdict=$pv control-verdict=$cv"',
    'done',
    'echo "BOOT-FIRST done"',
  ]);
  guest({ chosen, script, log: 'guest-timings-boot-first.log', timeout: ARM_TIMEOUT_MS, });
}

//endregion Added guest arms

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
  'no-configuration': async (chosen: Options): Promise<void> => {
    if (!chosen.noBuild)
      await build(chosen);
    noConfiguration(chosen);
  },
  'm1-key': async (chosen: Options): Promise<void> => {
    if (!chosen.noBuild)
      await build(chosen);
    m1Key(chosen);
  },
  'm1-read-write': async (chosen: Options): Promise<void> => {
    if (!chosen.noBuild)
      await build(chosen);
    m1ReadWrite(chosen);
  },
  'm2-controls': async (chosen: Options): Promise<void> => {
    if (!chosen.noBuild)
      await build(chosen);
    m2Controls(chosen);
  },
  'm3-kill': async (chosen: Options): Promise<void> => {
    if (!chosen.noBuild)
      await build(chosen);
    m3Kill(chosen);
  },
  'm7-scan': async (chosen: Options): Promise<void> => {
    if (!chosen.noBuild)
      await build(chosen);
    m7Scan(chosen);
  },
  'm9-busy': async (chosen: Options): Promise<void> => {
    if (!chosen.noBuild)
      await build(chosen);
    m9Busy(chosen);
  },
  'gate-skip': async (chosen: Options): Promise<void> => {
    if (!chosen.noBuild)
      await build(chosen);
    gateSkip(chosen);
  },
  timings: async (chosen: Options): Promise<void> => {
    if (!chosen.noBuild)
      await build(chosen);
    timings(chosen);
  },
  'timings-boot-first': timingsBootFirst,
  destroy: (chosen) => recipe({ chosen, step: 'destroy', }),
  all: async (chosen: Options): Promise<void> => {
    let failure: unknown = null;
    let created = false;
    try {
      recipe({ chosen, step: 'create', });
      created = true;
      recipe({ chosen, step: 'prepare', });
      await build(chosen);
      probeDataset(chosen);
      green(chosen);
      noConfiguration(chosen);
      m1Key(chosen);
      m1ReadWrite(chosen);
      m2Controls(chosen);
      m3Kill(chosen);
      m7Scan(chosen);
      m9Busy(chosen);
      gateSkip(chosen);
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
const noConfigurationLog = join(chosen.logDir, 'guest-no-configuration.log');
if (chosen.step === 'no-configuration' || chosen.step === 'all') {
  const pulled = readFileSync(noConfigurationLog, 'utf8');
  for (const marker of NO_CONFIGURATION_MARKERS) {
    if (!pulled.includes(marker))
      throw new Error(`the no-configuration log lacks ${marker}; the guest ran the arm without proving it`);
  }
}
if (chosen.step === 'green' || chosen.step === 'all') {
  const pulled = readFileSync(greenLog, 'utf8');
  for (const marker of GREEN_MARKERS) {
    if (!pulled.includes(marker))
      throw new Error(`the green log lacks ${marker}; the guest ran the arm without proving it`);
  }
}
console.log(`git-policy probe guest: step '${chosen.step}' finished; logs in ${chosen.logDir}`);
