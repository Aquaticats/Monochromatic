/**
 M1's two controls: the encrypted fixture dataset's `ready` snapshot attaching
 while its key is loaded and answering `EACCES` once it is unloaded, and a read-only
 request for the probe dataset beside a read-write superblock of it answering
 `EBUSY`.

 The first is what makes the green arm's attach a reading of the key path rather
 than of a source that always attaches. The second is the kernel's own rule
 (`module/os/linux/zfs/zpl_super.c:936` to `:943`), whose exception is a snapshot,
 which is why the probe's read-only attachment of that dataset's `ready` snapshot
 still passes.

 @example
 ```ts
 await m1Key({ chosen });
 ```
 */

import type {
  GuestFacts,
} from '../../bin/mvm-guest-guest-facts.ts';
import {
  guest,
} from '../../bin/mvm-guest-script.ts';


import {
  armTimeoutMilliseconds,
  probeDataset,
  probeEncrypted,
  probeScript,
  recordLineAssertion,
} from './probe-fixture.ts';

/**
 The log M1's unloaded-key control writes inside the run's log folder.
 */
const keyLog = 'guest-m1-key.log';

/**
 The log M1's read-only control writes inside the run's log folder.
 */
const readWriteLog = 'guest-m1-read-write.log';

/**
 M1's unloaded-key control.

 @param chosen - the run's guest and log folder

 @throws Error naming the guest's status, which a failing check makes nonzero

 @example
 ```ts
 await m1Key({ chosen });
 ```
 */
export async function m1Key({
  chosen,
}: {
  readonly chosen: GuestFacts;
},): Promise<void> {
  /**
   The control: the source attached with its key loaded, then unloaded, then
   loaded again with the configuration restored.
   */
  const script = probeScript({
    lines: [
      ...recordLineAssertion,
      'cp $CONF /root/m1-key-config.jsonc',
      `sed --in-place 's|${probeDataset}|${probeEncrypted}|' $CONF`,
      'echo "M1 source: $(grep --fixed-strings dataset $CONF)"',
      '$H start-probe > /root/m1-key-loaded.log 2>&1 || true',
      'cp $REC /root/m1-key-loaded-record.txt',
      `check 'verdict passed' /root/m1-key-loaded-record.txt 'key-loaded-passes'`,
      `check 'step attach-exclusive-create passed' /root/m1-key-loaded-record.txt 'key-loaded-creates-a-fresh-superblock'`,
      `check 'source-existed yes' /root/m1-key-loaded-record.txt 'key-loaded-finds-the-source'`,
      `zfs unload-key ${probeEncrypted}`,
      `keystatus=$(zfs get -H -o value keystatus ${probeEncrypted})`,
      'echo "M1 unloaded keystatus=$keystatus"',
      'test "$keystatus" = unavailable',
      '$H start-probe > /root/m1-key-unloaded.log 2>&1 || true',
      'cp $REC /root/m1-key-unloaded-record.txt',
      `check 'verdict failed' /root/m1-key-unloaded-record.txt 'key-unloaded-fails'`,
      `check 'step attach-exclusive-create failed errno EACCES' /root/m1-key-unloaded-record.txt 'key-unloaded-exclusive-create-EACCES'`,
      `check 'arm attach failed errno EACCES' /root/m1-key-unloaded-record.txt 'key-unloaded-attach-EACCES'`,
      'cp /root/m1-key-config.jsonc $CONF',
      `zfs load-key ${probeEncrypted}`,
      `keystatus=$(zfs get -H -o value keystatus ${probeEncrypted})`,
      'echo "M1 restored keystatus=$keystatus"',
      'test "$keystatus" = available',
      '$H start-probe > /root/m1-key-restored.log 2>&1 || true',
      'cp $REC /root/m1-key-restored-record.txt',
      `check 'verdict passed' /root/m1-key-restored-record.txt 'key-and-configuration-restored-passes'`,
      'echo "M1 key control done"',
    ],
  },);
  await guest({
    chosen,
    script,
    log: keyLog,
    timeoutMilliseconds: armTimeoutMilliseconds,
  },);
}

/**
 M1's read-only against read-write control.

 @param chosen - the run's guest and log folder

 @throws Error naming the guest's status, which a failing check makes nonzero

 @example
 ```ts
 await m1ReadWrite({ chosen });
 ```
 */
export async function m1ReadWrite({
  chosen,
}: {
  readonly chosen: GuestFacts;
},): Promise<void> {
  /**
   The control: a read-write mount of the dataset, then a read-only request
   beside it, then the probe's own attachment of its snapshot.
   */
  const script = probeScript({
    lines: [
      ...recordLineAssertion,
      `DATASET='${probeDataset}'`,
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
    ],
  },);
  await guest({
    chosen,
    script,
    log: readWriteLog,
    timeoutMilliseconds: armTimeoutMilliseconds,
  },);
}
