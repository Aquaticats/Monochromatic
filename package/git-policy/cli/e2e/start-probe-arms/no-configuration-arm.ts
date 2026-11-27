/**
 The project's answer of 2026-11-22 for the probe, proven by a run: with no
 configuration file the probe finds its dataset by its design's own name in the
 pool the mounted checkout dataset names, and its checkout as the sweep finds one,
 so a missing file no longer leaves the record blocked. The dataset is then renamed
 away, and the same run reads the `ENOENT` refusal the design keeps, with the mode's
 line naming the two commands plan step 19's setup runs.

 @example
 ```ts
 await noConfiguration({ chosen });
 ```
 */

import {
  type GuestFacts,
} from '../../bin/mvm-guest-guest-facts.ts';
import {
  guest,
} from '../../bin/mvm-guest-script.ts';


import {
  armTimeoutMilliseconds,
  probeDataset,
  probeScript,
} from './probe-fixture.ts';

/**
 The log the no-configuration arm writes inside the run's log folder.
 */
export const noConfigurationLog = 'guest-no-configuration.log';

/**
 Run the no-configuration arm: no file passes, and a dataset renamed away blocks.

 @param chosen - the run's guest and log folder

 @throws Error naming the guest's status, which a failing check makes nonzero

 @example
 ```ts
 await noConfiguration({ chosen });
 ```
 */
export async function noConfiguration({
  chosen,
}: {
  readonly chosen: GuestFacts;
},): Promise<void> {
  /**
   The arm: the configuration removed, one run that must pass, the dataset
   renamed away, and one run that must name its cause.
   */
  const script = probeScript({
    lines: [
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
      `check 'source ${probeDataset}@ready' /root/no-configuration-record.txt`,
      'echo "NO-CONFIGURATION source-is-the-designs-name"',
      `zfs rename ${probeDataset} ${probeDataset}-absent`,
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
      `zfs rename ${probeDataset}-absent ${probeDataset}`,
      'echo "NO-CONFIGURATION done"',
    ],
  },);
  await guest({
    chosen,
    script,
    log: noConfigurationLog,
    timeoutMilliseconds: armTimeoutMilliseconds,
  },);
}
