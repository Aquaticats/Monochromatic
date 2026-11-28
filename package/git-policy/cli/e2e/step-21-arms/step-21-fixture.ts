/**
 What plan step 21's guest driver names: where the crate is unpacked and built inside the guest, the pool the arms add
 worktrees on, the lines a passing run has to find, and the bound each step gets.

 The names are the driver's own, so a log of this round reads against the plan's own words: the pool is the arm's
 `zfsstep21`, the checkout is a mounted dataset of the design's leaf name, and each line a run must find says which arm
 performed it, so an arm that returned without running fails the run instead of reporting a green suite.

 @example
 ```ts
 const { crateDirectory, armLog } = names();
 ```
 */

/**
 Where the wrapper's tree is unpacked and built inside the guest, which every account may traverse.
 */
export const crateDirectory = '/opt/step21-arms';

/**
 The pool the arms build their datasets in.
 */
export const pool = 'zfsstep21';

/**
 The guest folder the file-backed pool's image lives in.
 */
export const fixtureRoot = '/home/step21-fixture';

/**
 The checkout the design's dataset is mounted at.
 */
export const checkout = '/root/step21-checkout';

/**
 Where the guest's OpenZFS 2.4.4 userspace is installed, which the recipe builds from source.
 */
export const openZfsSbin = '/opt/openzfs-2.4.4/sbin';

/**
 The `PATH` the guest's own reads of OpenZFS need.
 */
export const openZfsPath: string = `${openZfsSbin}:/opt/openzfs-2.4.4/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin`;

/**
 The `PATH` the arms need: the guest's OpenZFS userspace, the Rust toolchain the guest was prepared with, the Node
 tooling the driver installed, then the system's own programs.
 */
export const openZfsPathWithTools: string = `${openZfsSbin}:/opt/openzfs-2.4.4/bin:/root/.cargo/bin:/usr/local/bin:/usr/local/sbin:/usr/sbin:/usr/bin:/sbin:/bin`;

/**
 The environment the arm reads, so it never opens a device on a machine somebody uses.
 */
export const guestVariable = 'CLI_GIT_ZFS_STEP21_GUEST';

/**
 The pool the arm's datasets stand on.
 */
export const poolVariable = 'CLI_GIT_ZFS_STEP21_POOL';

/**
 The OpenZFS userspace the arm reads datasets with.
 */
export const sbinVariable = 'CLI_GIT_ZFS_STEP21_SBIN';

/**
 The checkout root the arm mounts the design's dataset at.
 */
export const rootVariable = 'CLI_GIT_ZFS_STEP21_ROOT';

/**
 The stamp naming the archive a build unpacks, which the guest waits for before it copies anything.
 */
export const stampName = 'step-21-archive.sha256';

/**
 The arm's log inside the guest's shared folder.
 */
export const armLog = 'step-21-arms.log';

/**
 The build's log inside the guest's shared folder.
 */
export const buildLog = 'step-21-build.log';

/**
 The log the guest's own tool install writes inside the shared folder.
 */
export const toolsLog = 'step-21-tools.log';

/**
 The lines a run must find in the guest's arm log, one per arm it performed.
 */
export const expectedMarkers: readonly string[] = [
  'STEP21 fixture checkout=',
  'STEP21 pnpm control round=0',
  'STEP21 pnpm arm round=0',
  'STEP21 pnpm ratio per-round=',
  'STEP21 cargo lock-probe metadata-locked-bytes=',
  'STEP21 cargo control round=0',
  'STEP21 cargo arm round=0',
  'STEP21 cargo ratio per-round=',
  'STEP21 launcher control round=0',
  'STEP21 launcher arm round=0',
  'STEP21 launcher ratio per-round=',
  'STEP21 watch before-mount=StillTheSameTree',
  'STEP21 watch after-mount=Blind',
  'STEP21 watch covered-watch-saw=false fresh-watch-saw=true',
  'STEP21 watch names=slot,editor,fsmonitor',
  'STEP21 service listening socket-bytes=',
  'STEP21 queued drained-ms=',
  'STEP21 queued round=0',
  'STEP21 inline round=0',
  'STEP21 queued-vs-inline ratio per-round=',
  'STEP21 done',
];

/**
 Milliseconds the guest's build of the test target may take.
 One `rustc` of this crate's test target took 3m 16s on an idle machine and 89 minutes on this host at 2026-11-28's load,
 where several rounds compile the same crate at once, so the bound is three hours and a build that reaches it is the
 machine's answer rather than the arm's.
 */
export const crateBuildTimeoutMilliseconds = 10_800_000;

/**
 Milliseconds the guest's own tool install may take.
 */
export const toolsTimeoutMilliseconds = 900_000;

/**
 Milliseconds the guest's pool script may take, which is one `zpool create` and one `truncate`.
 */
export const poolTimeoutMilliseconds = 300_000;

/**
 Milliseconds the arm may take: five measured arms of three rounds each, every round an add of a small repository.
 */
export const armTimeoutMilliseconds = 3_600_000;

/**
 Milliseconds the host gives a detached script past its own bound before it stops waiting.
 */
export const finishMarginMilliseconds = 180_000;
