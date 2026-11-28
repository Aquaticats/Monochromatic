/**
 What step 22's guest measurement names: the guest's paths and pools, the bound
 each step gets, and the lines a passing run has to find.

 The names are the design's, so a log of this round reads against the plan's own
 words: the pool is the arm's `zfsstep22`, the checkout is a mounted dataset of
 the design's leaf name, and the holder-scan arm beside it builds its own layout
 under a pool of its own so the two arms never read each other's datasets.

 @example
 ```ts
 const { name, logDir } = options();
 ```
 */

/**
 Where the wrapper's tree is unpacked and built inside the guest, which every
 account may traverse.
 */
export const crateDirectory = '/opt/step22-measurement';

/**
 The pool the arm's arms build their datasets in.
 */
export const pool = 'zfsstep22';

/**
 The guest folder the file-backed pool's image lives in.
 */
export const fixtureRoot = '/home/step22-fixture';

/**
 The pool the holder-scan arm beside this one builds its own layout under.
 */
export const scanPool = 'git-policy-cli-store';

/**
 The checkout the holder-scan arm walks.
 */
export const scanCheckout = '/root/holder-scan-checkout';

/**
 The checkout the design's dataset is mounted at.
 */
export const checkout = '/root/step22-checkout';

/**
 Where the guest's OpenZFS 2.4.4 userspace is installed, which the recipe builds
 from source.
 */
export const openZfsSbin = '/opt/openzfs-2.4.4/sbin';

/**
 The `PATH` the guest's OpenZFS 2.4.4 userspace needs.
 */
export const openZfsPath: string = `${openZfsSbin}:/opt/openzfs-2.4.4/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin`;

/**
 The `PATH` the measurement arm needs: the guest's OpenZFS userspace, then the
 Rust toolchain the guest was prepared with, then the system's own programs.
 */
export const openZfsPathWithCargo: string = `${openZfsSbin}:/opt/openzfs-2.4.4/bin:/root/.cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin`;

/**
 The environment the arm reads, so it never opens a device on a machine somebody
 uses.
 */
export const guestVariable = 'CLI_GIT_ZFS_STEP22_GUEST';

/**
 The arm's log inside the guest's shared folder.
 */
export const armLog = 'step-22-measurements-arm.log';

/**
 The holder-scan arm's log inside the guest's shared folder.
 */
export const holderScanLog = 'holder-scan-arm.log';

/**
 The timing line the holder-scan arm prints, which a run that only returned
 never writes.
 */
export const holderScanMeasurement = 'MEASUREMENT holder-scan-ms=';

/**
 The lines a run must find in the guest's arm log, one per arm it performed.
 */
export const expectedMarkers: readonly string[] = [
  'STEP22 pool=',
  'STEP22 add arm=git round=0',
  'STEP22 add arm=clone round=0',
  'STEP22 add-total control-ms=',
  'STEP22 sweep clones=1',
  'STEP22 sweep clones=64',
  'STEP22 parent-sweep spread run=0 clones=64',
  'STEP22 parent-sweep round=0 clones=512',
  'STEP22 parent-sweep times clones=512',
  'STEP22 parent-sweep done',
  'STEP22 unmount holder=none',
  'STEP22 forced-arm t-wait-ms=',
  'STEP22 forced-arm t-stop-ms=',
  'STEP22 retention after-removal=',
  'STEP22 pinned case=incompressible stage=after-expiry',
  'STEP22 pinned case=incompressible readings snapshot-used-at-snapshot=',
  'STEP22 pinned case=incompressible frees freed-by-removal-clone-used=',
  'freed-by-expiry-dataset=',
  'STEP22 pinned case=repeated-byte stage=after-expiry',
  'STEP22 pinned case=repeated-byte readings snapshot-used-at-snapshot=',
  'STEP22 pinned case=repeated-byte frees freed-by-removal-clone-used=',
  'STEP22 request-budget names=2048',
  'STEP22 done',
];

/**
 Milliseconds the guest's build of the test target may take.

 Two measurements size it: the round of 2026-11-27's fourth `run` built the
 target in 53 minutes at one job, and an eighth attempt at the same tree met the
 old ninety-minute bound unfinished while the host ran at a load average of 86
 with many rounds compiling the same crate. A bound a loaded host can reach is
 no bound on the build but on the machine, so it is two hours here.
 */
export const crateBuildTimeoutMilliseconds = 7_200_000;

/**
 Milliseconds the guest arm may take, which one run of every arm it performs
 fits in: the clones the past-64 arm makes and takes away are the longest part.
 */
export const armTimeoutMilliseconds = 3_600_000;

/**
 Milliseconds the guest's pool script may take, which is one `zpool create` and
 one `truncate` per pool.
 */
export const poolTimeoutMilliseconds = 300_000;

/**
 Milliseconds the host gives a detached script past its own bound before it
 stops waiting: the guest's `timeout` fires a minute before `mvm` would kill the
 command, and the pull after it costs the answer's own seconds.
 */
export const finishMarginMilliseconds = 120_000;

/**
 Milliseconds the host gives the two long steps past their own bound: the
 crate's build and the measurement arm both write a log the host then pulls.
 */
export const longFinishMarginMilliseconds = 180_000;
