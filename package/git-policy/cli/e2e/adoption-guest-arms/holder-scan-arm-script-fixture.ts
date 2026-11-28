/**
 The guest script plan step 9's holder-scan arm runs, kept where a unit test holds its text without a guest.

 `holder-scan-arm.ts` runs this script and judges the log it leaves; what the script says is here, so a test can read the
 step that takes down a previous run's mounts before the arm removes the folder they stand on.

 @example
 ```ts
 const script = holderScanScript({ chosen, log: 'holder-scan-arm.log' });
 ```
 */

import { shellQuote, } from '../../bin/mvm-guest-command-line.ts';
import type { GuestFacts, } from '../../bin/mvm-guest-guest-facts.ts';



import {
  crateDirectory,
  expectedScanMarkers,
  expectedScanMeasurements,
  openZfsPath,
  scanCheckout,
  scanGuestVariable,
  storePool,
} from './adoption-fixture.ts';

/**
 The script the holder-scan arm runs inside the guest: its own layout, one nextest run of plan step 9's arms, and the
 lines the driver reads back.

 @param chosen - the run's guest, whose name the arm's test binary reads

 @param log - the log's name inside the guest's shared folder, which the script writes and the driver pulls

 @returns the whole shell script, one line per step

 @example
 ```ts
 const script = holderScanScript({ chosen, log: 'holder-scan-arm.log' });
 ```
 */
export function holderScanScript({
  chosen,
  log,
}: {
  readonly chosen: GuestFacts;
  readonly log: string;
},): string {
  return [
    'set -eu',
    `export PATH=${openZfsPath}`,
    `rm --recursive --force ${scanCheckout}`,
    `mkdir --parents ${scanCheckout}`,
    `cd ${crateDirectory}/package/git-policy/cli`,
    'export PATH=/root/.cargo/bin:$PATH',
    `export ${scanGuestVariable}=${chosen.name}`,
    `export CLI_GIT_ZFS_SCAN_POOL=${storePool}`,
    'export CLI_GIT_ZFS_SCAN_SBIN=/opt/openzfs-2.4.4/sbin',
    `export CLI_GIT_ZFS_SCAN_CHECKOUT=${scanCheckout}`,
    'if cargo nextest run --release --locked --features test-only-switches --test native_binary --no-capture --status-level all '
      + `--filterset "test(zfs_holder_scan_guest)" > /root/${log} 2>&1; then status=0; else status=$?; fi`,
    `tail --lines=80 /root/${log}`,
    'test "$status" = 0',
    `if grep --quiet '^SKIPPED' /root/${log}; then echo 'the holder-scan arm returned without running'; exit 1; fi`,
    ...expectedScanMarkers.map(function markerLine(marker,) {
      return `grep --quiet --fixed-strings ${shellQuote(marker,)} /root/${log}`;
    },),
    ...expectedScanMeasurements.map(function measurementLine(marker,) {
      return `grep --quiet --fixed-strings ${shellQuote(marker,)} /root/${log}`;
    },),
    `cp /root/${log} /mnt/shared/${log}`,
  ].join('\n',);
}
