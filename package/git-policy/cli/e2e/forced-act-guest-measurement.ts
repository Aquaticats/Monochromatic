/**
 Measure, in a guest, what the kernel answers a locked holder's copy before and after `systemctl --user stop`.

 The owner's answer of 2026-11-23 stops a locked holder's own unit through the account's own user manager, and no record
 measured that stop: the guests the earlier arms used had no user manager of the account's own. This step gives the
 guest one (`loginctl enable-linger`, quoted from the guest's own manual), starts a unit with `PrivateMounts=yes` that
 holds a copy of a slot mount, reads what the unmount and the move answer before the stop, asks the manager to stop the
 unit, and reads what they answer after it.

 It makes that state twice, and the forced act's arm makes it a third time, because a guest's systemd may give a unit
 of `PrivateMounts=yes` alone a mount namespace the initial user namespace still owns.

 Both calls the step reads are the system calls themselves, through `ctypes`: `umount` reports the `EINVAL` a locked
 mount answers as "not mounted", so a program's phrasing of the kernel's answer is not the answer.

 The step is a separate file because the driver's own file is already at the linter's line budget.
 */
import {
  readFile,
} from 'node:fs/promises';
import {
  join,
} from 'node:path';

import {
  guest,
} from '../bin/mvm-guest-script.ts';
import {
  host,
} from '../bin/mvm-guest-host-command.ts';
import type {
  GuestDriverOptions,
} from '../bin/mvm-guest-options.ts';

/**
 The folder the measurement step's slot and destination live under, which is the fixture root the arm reads too.
 */
const MEASUREMENT_ROOT = '/home/zfs-forced-act-fixture';
/**
 Milliseconds the measurement step may take: the guest starts one user manager, two transient units, and reads four
 kernel answers around each stop.
 */
const MEASUREMENT_TIMEOUT_MS = 600_000;
/**
 The lines the measurement step must print, one per fact it measured.

 The step makes the state twice: once with a unit of `PrivateMounts=yes` alone, and once with `PrivateUsers=yes` as
 well, because a guest's systemd may give the first shape a mount namespace the initial user namespace still owns.
 The arm makes it a third time, with the shape this guest locks.
 */
const MEASUREMENT_MARKERS = [
  'MEASURE phase=documentation',
  'MEASURE phase=linger',
  'MEASURE phase=slot',
  'MEASURE variant=private-mounts',
  'MEASURE variant=private-users',
  'MEASURE phase=finished',
] as const;

/**
 What:
  Measure, in the guest, what the kernel answers a locked holder's copy before and after `systemctl --user stop`.
 Why:
  The owner's answer of 2026-11-23 stops a locked holder's own unit through the account's own user manager, and no
  record measured that stop: the guests the earlier arms used had no user manager of the account's own. This step
  gives the guest one (`loginctl enable-linger`, quoted from the guest's own manual), starts a unit with
  `PrivateMounts=yes` that holds a copy of a slot mount, reads what the unmount and the move answer before the stop,
  asks the manager to stop the unit, and reads what they answer after it.

  It prints one `MEASURE` line per fact, which this step's own marker check reads, and its log is pulled back as
  `forced-act-measurement.log`.

 @param chosen - the run's options

 @throws Error naming the marker the guest's log lacks

 @example
 ```ts
 await measureTheStop({ name: 'forced-act', logDir: '/logs', step: 'user-manager' });
 ```
 */
export async function measureTheStop(chosen: GuestDriverOptions,): Promise<void> {
  /**
   The guest's own script: quote its manual, start the account's user manager, mount a slot in the guest's root
   namespace, and read the kernel's four answers around each stop.
   */
  const script = [
    'set -eu',
    `ROOT=${MEASUREMENT_ROOT}`,
    'SLOT="$ROOT/measure-slot"',
    'DEST="$ROOT/measure-dest"',
    'PROBE="$ROOT/probe.py"',
    'mkdir --parents "$ROOT" "$SLOT" "$DEST"',
    'echo "== the release this guest boots, and the manager it runs"',
    'uname --kernel-release',
    'systemctl --version | head --lines=1',
    'echo "MEASURE phase=documentation"',
    'echo "== the guest own manual on lingering, which starts an account own user manager with no login"',
    'for page in /usr/share/man/man1/loginctl.1.gz; do if test -f "$page"; then echo "page=$page"; gzip --decompress --stdout "$page" | grep --fixed-strings --after-context=6 -- linger || true; fi; done',
    'echo "== lingering, which starts the account own user manager"',
    'loginctl enable-linger root',
    'loginctl show-user root --property=Linger --value',
    'for look in $(seq 1 120); do if systemctl is-active --quiet user@0.service; then break; fi; sleep 0.5; done',
    'systemctl is-active user@0.service',
    'export XDG_RUNTIME_DIR=/run/user/0',
    'export DBUS_SESSION_BUS_ADDRESS=unix:path=/run/user/0/bus',
    'systemctl --user is-system-running || true',
    'echo "MEASURE phase=linger"',
    // The kernel's own answer, not util-linux's phrasing of it: `umount` prints "not mounted" for the `EINVAL` a
    // locked mount answers, so the probe calls the two system calls and prints the errno each one gave.
    'cat > "$PROBE" <<"PROBE_SOURCE"',
    'import ctypes',
    'import errno',
    'import os',
    'import sys',
    'libc = ctypes.CDLL("libc.so.6", use_errno=True)',
    'MS_SLAVE = 1 << 19',
    'MS_REC = 1 << 14',
    'MS_MOVE = 1 << 13',
    'mode, slot, dest = sys.argv[1], sys.argv[2], sys.argv[3]',
    'NS_GET_USERNS = 0xB701',
    'def answer(name, made):',
    '    if made == 0:',
    '        return name + "=accepted"',
    '    return name + "=" + errno.errorcode.get(ctypes.get_errno(), str(ctypes.get_errno()))',
    'if mode == "owner":',
    '    namespace = os.open("/proc/" + slot + "/ns/mnt", os.O_RDONLY)',
    '    print("owner=" + str(os.fstat(libc.ioctl(namespace, NS_GET_USERNS)).st_ino))',
    '    sys.exit(0)',
    'if mode == "move-and-unmount":',
    '    libc.mount(None, b"/", None, MS_REC | MS_SLAVE, None)',
    '    print(answer("move", libc.mount(slot.encode(), dest.encode(), None, MS_MOVE, None)))',
    'print(answer("unmount", libc.umount2(slot.encode(), 0)))',
    'PROBE_SOURCE',
    'measure() {',
    '  variant=$1',
    '  shift',
    '  unit="r11-measure-$variant"',
    '  echo "== one mount at the slot, made in the namespace the manager own tree descends from"',
    '  nsenter --target 1 --mount -- mount --types tmpfs tmpfs "$SLOT"',
    '  nsenter --target 1 --mount -- findmnt --noheadings --output TARGET "$SLOT"',
    '  echo "MEASURE phase=slot variant=$variant"',
    '  systemd-run --user --unit="$unit" --property=PrivateMounts=yes "$@" /usr/bin/sleep infinity > /dev/null',
    '  pid=',
    '  for look in $(seq 1 200); do pid=$(systemctl --user show --property=MainPID --value "$unit" 2>/dev/null || true); if test -n "$pid" && test "$pid" != 0; then break; fi; sleep 0.1; done',
    '  namespaces="mnt=$(readlink /proc/$pid/ns/mnt) user=$(readlink /proc/$pid/ns/user) ours=$(readlink /proc/self/ns/user)"',
    '  cgroup=$(grep "^0::" "/proc/$pid/cgroup" || true)',
    '  echo "MEASURE variant=$variant unit=$unit pid=$pid namespaces=$namespaces cgroup=$cgroup"',
    '  echo "MEASURE variant=$variant the-units-copy=$(grep --fixed-strings --count -- "$SLOT " "/proc/$pid/mountinfo" || true)"',
    '  owner=$(python3 "$PROBE" owner "$pid" "$SLOT" 2>&1 || true)',
    '  ours=$(readlink /proc/self/ns/user)',
    '  echo "MEASURE variant=$variant $owner the-guests-own-user-namespace=$ours"',
    '  before=$(nsenter --target "$pid" --mount -- python3 "$PROBE" move-and-unmount "$SLOT" "$DEST" 2>&1 || true)',
    '  stop=$(systemctl --user stop "$unit" 2>&1 || true)',
    '  sleep 0.5',
    '  state=$(systemctl --user is-active "$unit" 2>&1 || true)',
    '  holds=$(test -e "/proc/$pid/ns/mnt" && echo there || echo gone)',
    '  tasks=$(cat "/proc/$pid/cgroup" 2>/dev/null || echo gone)',
    '  after=$(nsenter --target 1 --mount -- python3 "$PROBE" unmount "$SLOT" "$DEST" 2>&1 || true)',
    '  echo "MEASURE variant=$variant before=$before"',
    '  echo "MEASURE variant=$variant stop=$stop after-state=$state after-namespace=$holds after-tasks=$tasks"',
    '  echo "MEASURE variant=$variant the-hosts-own-unmount-after=$after"',
    '  nsenter --target 1 --mount -- umount "$SLOT" 2>/dev/null || true',
    '}',
    'echo "== a unit of PrivateMounts=yes alone"',
    'measure private-mounts',
    'echo "MEASURE variant=private-mounts"',
    'echo "== the same unit with a user namespace of its own"',
    'measure private-users --property=PrivateUsers=yes',
    'echo "MEASURE variant=private-users"',
    'loginctl disable-linger root',
    'echo "MEASURE phase=finished"',
  ]
    .join('\n',);
  /**
   Everything the guest printed, which is the measurement's own log.
   */
  const printed: string = await guest({
    chosen,
    script,
    log: 'guest-forced-act-measurement.log',
    timeoutMilliseconds: MEASUREMENT_TIMEOUT_MS,
  },);
  for (const marker of MEASUREMENT_MARKERS) {
    if (!printed
      .includes(marker,))
      throw new Error(`the measurement's log lacks ${marker}; the guest made no such state`,);
  }
  await host({
    chosen,
    command: 'cp',
    args: [
      join(
        chosen.logDir,
        'guest-forced-act-measurement.log',
      ),
      join(
        chosen.logDir,
        'forced-act-measurement.log',
      ),
    ],
    log: 'host-keep-measurement.log',
  },);
}
