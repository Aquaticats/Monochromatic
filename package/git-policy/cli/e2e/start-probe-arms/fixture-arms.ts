/**
 The two arms that make the fixture the other arms read: the red arm, which runs
 before the probe exists and requires the refusal that says the mode is not one of
 this build's, and the probe dataset the design names, with its empty `ready`
 snapshot, the encrypted dataset beside it and the root-owned configuration.

 The red arm is what proves the green run is a change and not an absence that
 always passed.

 @example
 ```ts
 await red({ chosen });
 ```
 */

import {
  type GuestFacts,
} from '../../bin/mvm-guest-guest-facts.ts';
import {
  guest,
} from '../../bin/mvm-guest-script.ts';


import {
  account,
  armTimeoutMilliseconds,
  checkout,
  configuration,
  handler,
  openZfsPath,
  openZfsPrefix,
  pool,
  poolImage,
  poolImageSize,
  probeDataset,
  probeEncrypted,
  probeKeyFile,
} from './probe-fixture.ts';

/**
 The log the red arm writes inside the run's log folder.
 */
const redLog = 'guest-red.log';

/**
 The log the probe-dataset arm writes inside the run's log folder.
 */
const probeDatasetLog = 'guest-probe-dataset.log';

/**
 The step that must fail before the probe exists: the mode word is not one this
 build takes.

 @param chosen - the run's guest and log folder

 @throws Error naming the guest's status, which the arm makes nonzero when it met
   something other than the absence it expected

 @example
 ```ts
 await red({ chosen });
 ```
 */
export async function red({
  chosen,
}: {
  readonly chosen: GuestFacts;
},): Promise<void> {
  /**
   The arm: the handler run, and its answer read as the absence of the mode.
   */
  const script = [
    'set -eu',
    `${handler} start-probe > /root/red.log 2>&1 || status=$?`,
    `status=\${status:-0}`,
    'cat /root/red.log',
    // A red arm passes only when the refusal is the absence of the mode, and says so.
    'if grep --quiet "start-probe" /root/red.log && grep --quiet "not a mode" /root/red.log; then',
    '  echo "RED the probe mode does not exist yet: $(cat /root/red.log | head --lines=1)"',
    '  exit 0',
    'fi',
    'echo "RED the arm did not meet the absence it expected"',
    'exit 1',
  ].join('\n',);
  await guest({
    chosen,
    script,
    log: redLog,
    timeoutMilliseconds: armTimeoutMilliseconds,
  },);
}

/**
 Create the pool and the probe dataset the design names, with its empty `ready`
 snapshot and the configuration.

 @param chosen - the run's guest and log folder

 @throws Error naming the guest's status

 @example
 ```ts
 await probeDatasetArm({ chosen });
 ```
 */
export async function probeDatasetArm({
  chosen,
}: {
  readonly chosen: GuestFacts;
},): Promise<void> {
  /**
   The fixture: one file-backed pool, the probe dataset and the encrypted one
   beside it, their `ready` snapshots, and the configuration naming them.
   */
  const script = [
    'set -eu',
    `export PATH=${openZfsPath}`,
    `export LD_LIBRARY_PATH=${openZfsPrefix}/lib`,
    // The pool is a file-backed image of this throwaway guest, and nothing here is on a person's machine.
    `truncate --size ${poolImageSize} ${poolImage}`,
    `zpool create -f -o cachefile=none -O mountpoint=none ${pool} ${poolImage}`,
    `zfs create -o canmount=off -o mountpoint=none ${pool}/nosnap`,
    `zfs create -o mountpoint=legacy -o canmount=off -o com.sun:auto-snapshot=false ${probeDataset}`,
    `zfs set org.monochromatic:git-worktree-probe-source=v1 ${probeDataset}`,
    // M1's unloaded-key control needs an encrypted source whose key the step can unload and load again.
    `printf '%s' 'correct horse battery staple' > ${probeKeyFile}`,
    `chmod 0600 ${probeKeyFile}`,
    `zfs create -o encryption=on -o keyformat=passphrase -o keylocation=file://${probeKeyFile} -o mountpoint=legacy -o canmount=off -o com.sun:auto-snapshot=false ${probeEncrypted}`,
    `zfs set org.monochromatic:git-worktree-probe-source=v1 ${probeEncrypted}`,
    `zfs snapshot ${probeEncrypted}@ready`,
    `zfs snapshot ${probeDataset}@ready`,
    `mkdir --parents $(dirname ${configuration})`,
    `printf '%s\\n' '{' '  "startProbe": {' '    "dataset": "'${probeDataset}'",' '    "checkout": "'${checkout}'"' '  },' '  "account": "'${account}'"' '}' > ${configuration}`,
    `chmod 0644 ${configuration}`,
    // The checkout the design names is a mounted dataset, so a probe with no configuration file finds it the way the
    // sweep does (doc/planning/cli-git-zfs-start-probe-design.md, "The source: the probe dataset").
    `zfs create -o mountpoint=none ${pool}/git-worktree-checkout`,
    `zfs create -o mountpoint=${checkout} ${pool}/git-worktree-checkout/monochromatic-git-checkout`,
    `chown ${account}:${account} ${checkout}`,
    `zfs list -H -o name,mountpoint,canmount,encryption ${pool} ${pool}/nosnap ${probeDataset} ${probeEncrypted}`,
  ].join('\n',);
  await guest({
    chosen,
    script,
    log: probeDatasetLog,
    timeoutMilliseconds: armTimeoutMilliseconds,
  },);
}
