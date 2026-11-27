/**
 M2's controls: an idle window, a deliberate mount, a deliberate write and a
 deliberate read.

 The idle window is the fence's own negative control: a window with no work in it
 commits every row empty, so a fence that reports a write after a mount the probe
 made is a fence that reads something. The mount control shows the host's table
 sees a deliberate mount, the write control shows a deliberate write reaches a
 committed row, and the read control shows a read reaches none.

 @example
 ```ts
 await m2Controls({ chosen });
 ```
 */

import type { GuestRun, } from '../guest-host/guest-run.ts';
import { guest, } from '../guest-host/guest-side-scripts.ts';
import {
  armTimeoutMilliseconds,
  pool,
  probeDataset,
  probeScript,
} from './probe-fixture.ts';

/**
 The log M2's controls write inside the run's log folder.
 */
const controlsLog = 'guest-m2-controls.log';

/**
 Run M2's four controls.

 @param chosen - the run's guest and log folder

 @throws Error naming the guest's status, which a failing check makes nonzero

 @example
 ```ts
 await m2Controls({ chosen });
 ```
 */
export async function m2Controls({
  chosen,
}: {
  readonly chosen: GuestRun;
},): Promise<void> {
  /**
   The controls, in the order the design's windows are read: idle, mount, write,
   read.
   */
  const script = probeScript({
    lines: [
      'sort /proc/self/mountinfo | sed "s/^[0-9]* //" > /root/m2-table-before.txt',
      `zpool sync ${pool}`,
      `cat /proc/spl/kstat/zfs/${pool}/txgs > /root/m2-idle-before.txt`,
      't0=$(awk \'NR > 2 { if ($1 > top) top = $1 } END { print top }\' /root/m2-idle-before.txt)',
      'sleep 2',
      `zpool sync ${pool}`,
      `cat /proc/spl/kstat/zfs/${pool}/txgs > /root/m2-idle-after.txt`,
      'if awk -v t0="$t0" \'NR > 2 && $1 >= t0 && ($4 != 0 || $6 != 0 || $8 != 0) { bad = 1 } NR > 2 && $1 >= t0 { seen = 1 } END { exit !(seen && !bad) }\' /root/m2-idle-after.txt; then echo "M2 idle-window-clean t0=$t0"; else echo "M2 idle-window-held-a-write"; exit 1; fi',
      'mkdir --parents /root/m2-mount',
      'mount -t zfs "$SRC" /root/m2-mount',
      'sort /proc/self/mountinfo | sed "s/^[0-9]* //" > /root/m2-table-mounted.txt',
      'if cmp --silent /root/m2-table-before.txt /root/m2-table-mounted.txt; then echo "M2 deliberate-mount-not-seen"; exit 1; else echo "M2 deliberate-mount-seen"; fi',
      'umount /root/m2-mount',
      'sort /proc/self/mountinfo | sed "s/^[0-9]* //" > /root/m2-table-restored.txt',
      'if cmp --silent /root/m2-table-before.txt /root/m2-table-restored.txt; then echo "M2 deliberate-mount-restored"; else echo "M2 mount-table-not-restored"; exit 1; fi',
      `zfs create -o mountpoint=legacy ${pool}/nosnap/m2-write 2>/dev/null || true`,
      'mkdir --parents /root/m2-write',
      `mount -t zfs ${pool}/nosnap/m2-write /root/m2-write`,
      `zpool sync ${pool}`,
      `cat /proc/spl/kstat/zfs/${pool}/txgs > /root/m2-write-before.txt`,
      't0=$(awk \'NR > 2 { if ($1 > top) top = $1 } END { print top }\' /root/m2-write-before.txt)',
      'echo write > /root/m2-write/file',
      `zpool sync ${pool}`,
      `cat /proc/spl/kstat/zfs/${pool}/txgs > /root/m2-write-after.txt`,
      'if awk -v t0="$t0" \'NR > 2 && $1 >= t0 && ($4 != 0 || $6 != 0 || $8 != 0) { seen = 1 } END { exit !seen }\' /root/m2-write-after.txt; then echo "M2 deliberate-write-seen"; else echo "M2 deliberate-write-not-seen"; exit 1; fi',
      'umount /root/m2-write',
      `zfs destroy ${pool}/nosnap/m2-write`,
      `zpool sync ${pool}`,
      `cat /proc/spl/kstat/zfs/${pool}/txgs > /root/m2-read-before.txt`,
      't0=$(awk \'NR > 2 { if ($1 > top) top = $1 } END { print top }\' /root/m2-read-before.txt)',
      `unshare --mount sh -c "mkdir -p /root/m2-read; mount -t zfs -o ro ${probeDataset}@ready /root/m2-read; ls -a /root/m2-read > /dev/null; umount /root/m2-read"`,
      `zpool sync ${pool}`,
      `cat /proc/spl/kstat/zfs/${pool}/txgs > /root/m2-read-after.txt`,
      'if awk -v t0="$t0" \'NR > 2 && $1 >= t0 && ($4 != 0 || $6 != 0 || $8 != 0) { bad = 1 } NR > 2 && $1 >= t0 { seen = 1 } END { exit !(seen && !bad) }\' /root/m2-read-after.txt; then echo "M2 deliberate-read-clean"; else echo "M2 deliberate-read-held-a-write"; exit 1; fi',
      'echo "M2 controls done"',
    ],
  },);
  await guest({
    chosen,
    script,
    log: controlsLog,
    timeout: armTimeoutMilliseconds,
  },);
}
