/**
 The lost-event guest's fixture and its cases, one shell script each.

 The cases are plan step 19's: an event a ZED start replays, a dropped event with and without a live caller, zedlets
 run at once, a stale receipt, and a pending unmount across a restart. Their evidence is the installed zedlet's own
 log and the receipts under `/run`, which the judge reads back in the guest.

 Everything the script changes is the guest's own: its file-backed pool, its `/etc/zfs/zed.d`, its `zfs-zed.service`
 and its `/run`. Nothing here reaches a person's machine, which is why the driver names the guest and the judge refuses
 outside it.

 @example
 ```ts
 await buildInGuest({ chosen });
 const pulled = await pullArmLog({ chosen });
 ```
 */

import { pullGuestLog, } from '../../bin/mvm-guest-detached-script.ts';
import { pushWorktreeSnapshot, } from '../../bin/mvm-guest-host-command.ts';
import { guest, } from '../../bin/mvm-guest-script.ts';
import { requireMarkers, } from '../../bin/mvm-guest-arm-log.ts';
import type { GuestFacts, } from '../../bin/mvm-guest-guest-facts.ts';
import { detachedStep, } from '../adoption-guest-arms/detached-step.ts';

/**
 Where the crate is unpacked and built inside the guest.
 */
export const crateDirectory = '/opt/lost-events/crate';

/**
 The `PATH` the guest's OpenZFS 2.4.4 userspace needs.
 */
export const openZfsPath: string =
  '/opt/openzfs-2.4.4/sbin:/opt/openzfs-2.4.4/bin:/sbin:/usr/sbin:/bin:/usr/bin';

/**
 The environment the judge reads, so it runs only in the guest the driver named.
 */
export const guestVariable = 'CLI_GIT_ZFS_LOST_EVENTS_GUEST';

/**
 The log the case script writes inside the guest's shared folder.
 */
export const armLog = 'lost-event-arm.log';

/**
 The log the judge writes inside the guest's shared folder.
 */
export const judgeLog = 'lost-event-judge.log';

/**
 The step log of the case script, inside the same folder.
 */
const armStepLog = 'guest-lost-event-arm.log';

/**
 The pool the cases build, which is the guest's own.
 */
export const pool = 'zfsr9';

/**
 Milliseconds the guest's build may take.
 Value:
  four times the 516 s a guest's build of this crate took on 2026-11-27
  (`e2e/README.md`,
  the doctor route's build), with the swap file the recipe adds,
  so a build that outlasts it is one this bound ends rather than a slow build.
 */
const buildBoundMilliseconds = 2_400_000;

/**
 Milliseconds one case script may take,
 which is several ZED restarts and their deliveries.
 Value:
  the arms' own bound in this package,
  four times the longest arm measured there on 2026-11-27.
 */
const armBoundMilliseconds = 900_000;

/**
 The milliseconds in one second, which a bound given in seconds is multiplied by.
 */
const millisecondsInASecond = 1_000;

/**
 Milliseconds the host waits past a bound,
 which is the pull of the log that bound's script wrote.
 */
const pullMarginMilliseconds = 180_000;

/**
 The lines the case script must print, one per claim it makes.
 */
export const expectedArmMarkers: readonly string[] = [
  'LOST-EVENT fixture-pool=',
  'LOST-EVENT zedlet-installed=active',
  'LOST-EVENT replay=answered',
  'LOST-EVENT dropped-without-a-caller=equal',
  'LOST-EVENT dropped-with-a-caller=no-receipt',
  'LOST-EVENT zedlets-at-once=',
  'LOST-EVENT stale-receipt=replaced',
  'LOST-EVENT pending-unmount=answered',
  'LOST-EVENT cases-done',
];

/**
 The one build the cases need: the crate's `native_binary` test target, whose
 single tests are both the installed zedlet and the judge.
 */
const buildScript = [
  'set -eu',
  'if test -f /mnt/shared/subject.tar.gz; then cp /mnt/shared/subject.tar.gz /root/subject.tar.gz; fi',
  'rm --recursive --force /opt/lost-events/crate',
  'mkdir --parents /opt/lost-events/crate',
  'tar --extract --gzip --file /root/subject.tar.gz --directory /opt/lost-events/crate',
  'cd /opt/lost-events/crate/package/git-policy/cli',
  'export PATH=/root/.cargo/bin:$PATH',
  'if ! swapon --show | grep --quiet /swapfile; then fallocate --length 6G /swapfile; chmod 600 /swapfile; mkswap /swapfile; swapon /swapfile; fi',
  'CARGO_BUILD_JOBS=1 cargo build --release --tests --locked --features test-only-switches > /root/lost-event-build.log 2>&1',
  'tail --lines=3 /root/lost-event-build.log',
  'built=$(find target/release/deps -maxdepth 1 -name "native_binary-*" -type f -perm -u+x | head --lines=1)',
  'test -n "$built"',
  String.raw`printf "%s\n" "$built" > /opt/lost-events/binary`,
  'echo "LOST-EVENT build-binary=$built"',
].join('\n',);

/**
 Build the crate's test target in the guest, detached, because it takes minutes.

 @param chosen - the run's guest and log folder

 @throws Error naming the status the build ended with

 @example
 ```ts
 await buildInGuest({ chosen });
 ```
 */
export async function buildInGuest({
  chosen,
}: {
  readonly chosen: GuestFacts;
},): Promise<void> {
  await pushWorktreeSnapshot({ chosen, },);
  await detachedStep({
    chosen,
    script: buildScript,
    log: 'lost-event-build-step.log',
    timeoutMilliseconds: buildBoundMilliseconds,
    finishTimeoutMilliseconds: buildBoundMilliseconds + pullMarginMilliseconds,
  },);
}

/**
 The case script: the fixture, then one case per delivery the plan names.

 @returns the shell script the guest runs
 */
function casesScript(): string {
  return [
    'set -eu',
    `export PATH=${openZfsPath}`,
    'export CLI_GIT_ZFS_LOST_EVENTS_GUEST=$(hostname)',
    'INTENT=org.monochromatic:git-worktree-clone-mount-desired-state',
    'NUDGE=org.monochromatic:git-worktree-clone-mount-sweep-nonce',
    'mkdir --parents /opt/lost-events',
    'rm --force /opt/lost-events/delivery.log /opt/lost-events/pass.log /opt/lost-events/request',
    'rm --force /opt/lost-events/count-replay /opt/lost-events/waiter /opt/lost-events/stale',
    'rm --force /opt/lost-events/pending-unmount /opt/lost-events/passes-around-dropped',
    'rm --recursive --force /run/git-worktree-clone-mount-results',
    'rm --recursive --force /run/git-worktree-clone-mount-sweeps',
    `if ! zpool list -H -o name | grep --quiet --fixed-strings ${pool}; then truncate --size 1G /opt/lost-events/pool.img; zpool create -f ${pool} /opt/lost-events/pool.img; fi`,
    `zfs list -H -o name ${pool}/clones > /dev/null 2>&1 || zfs create ${pool}/clones`,
    `zfs list -H -o name ${pool}/clones/one > /dev/null 2>&1 || zfs create ${pool}/clones/one`,
    `zfs list -H -o name ${pool}/elsewhere > /dev/null 2>&1 || zfs create ${pool}/elsewhere`,
    `printf '${String.raw`%s\n`}' ${pool}/clones > /opt/lost-events/parent`,
    `guid=$(zfs get --H --parsable --value --property=guid ${pool}/clones/one)`,
    'echo "LOST-EVENT fixture-pool=$guid"',
    'built=$(cat /opt/lost-events/binary)',
    String.raw`printf '#!/bin/sh\nexec %s --exact zfs_lost_event_zedlet_arm\n' "$built" > /etc/zfs/zed.d/history_event-mount_git_worktree_clone_datasets`,
    'chmod 0755 /etc/zfs/zed.d/history_event-mount_git_worktree_clone_datasets',
    'systemctl enable zfs-zed.service > /dev/null 2>&1 || true',
    'systemctl restart zfs-zed.service',
    'sleep 2',
    'echo "LOST-EVENT zedlet-installed=$(systemctl is-active zfs-zed.service)"',
    String.raw`nonce() { od -An -tx1 -N16 /dev/urandom | tr -d " \n"; }`,
    'passes() { if test -f /opt/lost-events/pass.log; then wc --lines < /opt/lost-events/pass.log; else echo 0; fi; }',
    'deliveries() { if test -f /opt/lost-events/delivery.log; then grep --count "^end$" /opt/lost-events/delivery.log || echo 0; else echo 0; fi; }',
    'wait_for_deliveries() { i=0; while test $i -lt 150; do if test "$(deliveries)" -ge "$1"; then return 0; fi; i=$((i+1)); sleep 0.2; done; echo "the delivery did not arrive" >&2; return 1; }',
    'wait_for_receipt() { i=0; while test $i -lt 150; do if grep --quiet --fixed-strings "$1" /run/git-worktree-clone-mount-results/* 2>/dev/null; then echo answered > "$2"; return 0; fi; i=$((i+1)); sleep 0.2; done; echo absent > "$2"; return 0; }',
    String.raw`request() { printf "%s %s %s %s\n" "$1" "$2" "$3" "$4" > /opt/lost-events/request; }`,
    '# 1. An event a ZED start replays: the intent is written while ZED is stopped, then ZED starts and delivers it.',
    'systemctl stop zfs-zed.service',
    'sleep 1',
    'replayed=$(nonce)',
    `request "$replayed" mount ${pool}/clones/one "$guid"`,
    `zfs set $INTENT="v1:$replayed:mount" ${pool}/clones/one`,
    'systemctl start zfs-zed.service',
    'wait_for_deliveries 1',
    'wait_for_receipt "$replayed" /opt/lost-events/count-replay',
    'echo "LOST-EVENT replay=$(cat /opt/lost-events/count-replay)"',
    '# 2. A dropped event with nobody waiting: another dataset of the same pool changes, and no pass may run.',
    'before=$(passes)',
    'delivered=$(deliveries)',
    `zfs set $NUDGE="$(nonce)" ${pool}/elsewhere`,
    'wait_for_deliveries $((delivered + 1))',
    'after=$(passes)',
    'if test "$before" = "$after"; then dropped_without=equal; else dropped_without=grew; fi',
    'echo "LOST-EVENT dropped-without-a-caller=$dropped_without"',
    "# 3. The same drop with a live caller: a waiter polls for the clone's receipt while the other dataset changes.",
    'waited=$(nonce)',
    `request "$waited" mount ${pool}/clones/one "$guid"`,
    'rm --recursive --force /run/git-worktree-clone-mount-results',
    '( i=0; while test $i -lt 60; do if grep --quiet --fixed-strings "$waited" /run/git-worktree-clone-mount-results/* 2>/dev/null; then echo receipt > /opt/lost-events/waiter; exit 0; fi; i=$((i+1)); sleep 0.2; done; echo no-receipt > /opt/lost-events/waiter ) &',
    'waiter=$!',
    'delivered=$(deliveries)',
    `zfs set $NUDGE="$(nonce)" ${pool}/elsewhere`,
    'wait_for_deliveries $((delivered + 1))',
    'wait $waiter',
    'echo "LOST-EVENT dropped-with-a-caller=$(cat /opt/lost-events/waiter)"',
    'after_live=$(passes)',
    'if test "$before" = "$after_live"; then echo equal > /opt/lost-events/passes-around-dropped; else echo grew > /opt/lost-events/passes-around-dropped; fi',
    '# 4. Zedlets run at once: two property changes in flight give ZED two deliveries to fork at once.',
    `request "$(nonce)" mount ${pool}/clones/one "$guid"`,
    'delivered=$(deliveries)',
    `zfs set $NUDGE="$(nonce)" ${pool}/clones/one &`,
    `zfs set $NUDGE="$(nonce)" ${pool}/clones/one &`,
    'wait',
    'wait_for_deliveries $((delivered + 2))',
    'echo "LOST-EVENT zedlets-at-once=$(grep --count "^start " /opt/lost-events/delivery.log)"',
    "# 5. A stale receipt: an older nonce's receipt is on disk when a fresh request's event is delivered.",
    'stale=$(nonce)',
    `request "$stale" mount ${pool}/clones/one "$guid"`,
    `zfs set $NUDGE="$(nonce)" ${pool}/clones/one`,
    'wait_for_deliveries $(( $(deliveries) + 1 ))',
    'wait_for_receipt "$stale" /opt/lost-events/stale-old',
    'fresh=$(nonce)',
    `request "$fresh" mount ${pool}/clones/one "$guid"`,
    `zfs set $NUDGE="$(nonce)" ${pool}/clones/one`,
    'wait_for_deliveries $(( $(deliveries) + 1 ))',
    'wait_for_receipt "$fresh" /opt/lost-events/stale',
    'echo "LOST-EVENT stale-receipt=$(cat /opt/lost-events/stale)"',
    '# 6. A pending unmount across a restart: the unmount is written while ZED is down and replayed when it starts.',
    'systemctl stop zfs-zed.service',
    'sleep 1',
    'unmounting=$(nonce)',
    `request "$unmounting" unmount ${pool}/clones/one "$guid"`,
    'rm --recursive --force /run/git-worktree-clone-mount-results',
    `zfs set $INTENT="v1:$unmounting:unmount" ${pool}/clones/one`,
    'systemctl start zfs-zed.service',
    'wait_for_deliveries $(( $(deliveries) + 1 ))',
    'wait_for_receipt "$unmounting" /opt/lost-events/pending-unmount',
    'echo "LOST-EVENT pending-unmount=$(cat /opt/lost-events/pending-unmount)"',
    'echo "LOST-EVENT cases-done"',
    `cp /opt/lost-events/delivery.log /mnt/shared/${armLog}`,
  ].join('\n',);
}

/**
 Run the cases in the guest and pull the log they wrote.

 @param chosen - the run's guest and log folder

 @returns the pulled case log

 @throws Error naming the marker the pulled log lacks

 @example
 ```ts
 const pulled = await pullArmLog({ chosen });
 ```
 */
export async function pullArmLog({
  chosen,
}: {
  readonly chosen: GuestFacts;
},): Promise<string> {
  await detachedStep({
    chosen,
    script: casesScript(),
    log: armStepLog,
    timeoutMilliseconds: armBoundMilliseconds,
    finishTimeoutMilliseconds: armBoundMilliseconds + pullMarginMilliseconds,
  },);
  /**
   The appliance log the guest wrote into its shared folder, pulled back as the
   arm's evidence.
   */
  const pulled: string = await pullGuestLog({
    chosen,
    file: armLog,
    log: 'host-pull-lost-event-arm.log',
  },);
  requireMarkers({
    text: pulled,
    markers: expectedArmMarkers,
    subject: 'the pulled lost-event case log',
    because: 'the guest ran the cases without performing one of them',
  },);
  return pulled;
}

/**
 The shell command that runs the judge, which reads the deliveries and the
 receipts back.

 @returns the shell script the guest runs

 @example
 ```ts
 await judgeArguments();
 ```
 */
export function judgeArguments(): string {
  return [
    'set -eu',
    `export PATH=${openZfsPath}`,
    'export CLI_GIT_ZFS_LOST_EVENTS_GUEST=$(hostname)',
    'built=$(cat /opt/lost-events/binary)',
    `"$built" --exact zfs_lost_event_triggers_guest --nocapture > /mnt/shared/${judgeLog} 2>&1`,
    `tail --lines=40 /mnt/shared/${judgeLog}`,
  ].join('\n',);
}

/**
 Run one bounding shell script in the guest and answer what it printed.

 @param chosen - the run's guest and log folder

 @param script - what the guest runs

 @param log - the log's name in the run's log folder

 @param seconds - the bound the host gives the command

 @returns what the guest wrote

 @throws Error naming the status the command exited with

 @example
 ```ts
 await caseScript({ chosen, script: judgeArguments(), log: judgeLog, seconds: 300 });
 ```
 */
export async function caseScript({
  chosen,
  script,
  log,
  seconds,
}: {
  readonly chosen: GuestFacts;
  readonly script: string;
  readonly log: string;
  readonly seconds: number;
},): Promise<string> {
  return await guest({
    chosen,
    script,
    log,
    timeoutMilliseconds: seconds * millisecondsInASecond,
  },);
}
