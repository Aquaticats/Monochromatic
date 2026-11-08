/**
 Names, defaults and bounds of the move tools, each with where it comes from. The bounds that keep a step from
 waiting forever follow the owner's rule "nothing should ever wait forever or indefinitely"; the plan
 (`doc/planning/repository-on-the-860-drive.md`, section "Deadlines") gives the transfer deadline, and the others are
 the script author's choices that the caller may override by flag.

 @module
 */

/**
 Seconds a single `btrfs send | btrfs receive` may take: the plan's value until a first measured transfer replaces it.
 */
export const DEFAULT_TRANSFER_DEADLINE_SECONDS = 14_400;

/**
 Seconds a single `rsync` of one external worktree may take: the plan's value, section "The copy".
 */
export const DEFAULT_EXTERNAL_DEADLINE_SECONDS = 7_200;

/**
 Seconds the whole run may take, the author's bound so that no run waits indefinitely; the guided runner passes a
 tighter value (the window's remainder) for the final pass.
 */
export const DEFAULT_TOTAL_DEADLINE_SECONDS = 86_400;

/**
 Seconds a quick command (snapshot, show, property, create, the dry-run report) may take, the author's bound.
 */
export const DEFAULT_COMMAND_DEADLINE_SECONDS = 600;

/**
 Grace in seconds between the terminate signal and the kill that `timeout` gives a transfer: the plan's value.
 */
export const KILL_GRACE_SECONDS = 30;

/**
 Free space in GiB the 860 must keep before each transfer: the plan's capacity policy, section "Capacity policy".
 */
export const DEFAULT_MIN_FREE_GIB = 150;

/**
 Bytes in a KiB.
 */
export const BYTES_PER_KIB = 1_024;

/**
 Bytes in a MiB.
 */
export const BYTES_PER_MIB: number = BYTES_PER_KIB * BYTES_PER_KIB;

/**
 Bytes in a GiB.
 */
export const BYTES_PER_GIB: number = BYTES_PER_MIB * BYTES_PER_KIB;

/**
 Milliseconds in a second.
 */
export const MILLISECONDS_PER_SECOND = 1_000;

/**
 MiB of output a child command may print: a listing of every subvolume of the filesystem.
 */
const CHILD_OUTPUT_MIB = 64;

/**
 Most output a child command may print, in bytes.
 */
export const MAX_OUTPUT_BYTES: number = CHILD_OUTPUT_MIB * BYTES_PER_MIB;

/**
 Inode number that every btrfs subvolume root has.
 */
export const SUBVOLUME_ROOT_INODE = 256n;

/**
 Resource bounds of one transfer's systemd scope, sized to one stream by the plan author (not measured).
 */
export const TRANSFER_SCOPE_BOUNDS: readonly string[] = [
  '--scope',
  '--collect',
  '--property=CPUQuota=200%',
  '--property=MemoryMax=2G',
  '--property=TasksMax=32',
  '--property=IOWeight=10',
];

/**
 Digits in the sequence part of a copy name such as `s003`.
 */
export const NAME_DIGITS = 3;

/**
 Prefix of copy names in the first pass.
 */
export const FIRST_PREFIX = 's';

/**
 Prefix of copy names in the final pass.
 */
export const FINAL_PREFIX = 'f';

/**
 File in the state folder that lists the subvolumes the freeze step froze.
 */
export const FREEZE_MANIFEST = 'freeze-manifest.tsv';

/**
 Start of the name the freeze manifest is renamed to once a thaw has set every subvolume it lists read-write, followed by the
 time of the thaw in milliseconds. The Git wrapper reads the manifest to tell a read-only failure caused by the move from any
 other (`package/git-policy/cli/src/native/repository_link_check/repository_move_freeze.rs`), so a manifest that is present
 means the subvolumes it lists are frozen, and the renamed one means they are not; nothing is deleted.
 */
export const FREEZE_MANIFEST_THAWED_PREFIX: string = `${FREEZE_MANIFEST}.thawed-`;

/**
 File in the state folder that lists the subvolumes of the latest first pass.
 */
export const FIRST_PASS_MANIFEST = 'first-pass-manifest.tsv';

/**
 Arguments that every `rsync` of an external worktree starts with; none deletes.
 */
export const EXTERNAL_RSYNC_ARGUMENTS: readonly string[] = [
  '--archive',
  '--hard-links',
  '--acls',
  '--xattrs',
  '--numeric-ids',
  '--one-file-system',
];


/**
 Exit status of `timeout` when it ended the command at its deadline.
 */
export const TERMINATED_BY_TIMEOUT = 124;

/**
 Mode of the state folder: the owner only, because records name paths of every worktree.
 */
export const STATE_FOLDER_MODE = 0o700;

/**
 MiB of output one Git call of the scan may print.
 */
const SCAN_OUTPUT_MIB = 256;

/**
 Most output one Git call of the scan may print, in bytes: a status listing of a worktree with many untracked files.
 */
export const SCAN_MAX_OUTPUT_BYTES: number = SCAN_OUTPUT_MIB * BYTES_PER_MIB;

/**
 Seconds each Git call of the scan may take when the caller gives no limit: the plan's value, section "Quiesce and
 freeze".
 */
export const SCAN_DEFAULT_PER_WORKTREE_SECONDS = 120;

/**
 Characters of a failed command's message a stop quotes.
 */
export const FAILURE_MESSAGE_LIMIT = 120;

/**
 Characters of a failed command's standard error a stop quotes.
 */
export const FAILURE_STDERR_LIMIT = 300;

/**
 Characters of a failed Git status a table row quotes.
 */
export const STATUS_FAILURE_LIMIT = 80;

/**
 Bytes the copy may write beyond the shared footprint because the kernel's send writes an extent in full instead of
 cloning it when the extent has more than 1024 references (`SEND_MAX_EXTENT_REFS`, `fs/btrfs/send.c`). Measured by the
 coordinator on 2026-11-07 from 21:48Z to 21:54Z in the extent tree of `/dev/dm-1` (read-only, all of `/home`, so an upper
 bound for the repository; the counts are lower bounds because `dump-tree` skips a block rewritten during the read): nine
 extents have more than 1024 references; six resolve entirely outside the repository, and the three whose owner is a
 deleted subvolume (6087) reference 13038362624 bytes (12.14 GiB) in total. The method and its limits are in
 `doc/planning/repository-on-the-860-drive.md`, section "The send plan".
 */
export const CLONE_LIMIT_ALLOWANCE_BYTES = 13_038_362_624;

/**
 MiB of random data the fixture's main checkout holds: stage `f2-populate` writes it with `head --bytes`, and each worktree
 holds a reflink of it, so one populated subvolume carries this much.
 */
export const FIXTURE_MAIN_DATA_MIB = 300;

/**
 Subvolumes the fixture populates: the main checkout and its two worktrees (a and b).
 */
export const FIXTURE_SUBVOLUME_COUNT = 3;

/**
 Copy sets the destination image still holds when the last fixture stage that writes one runs, each counted as a full set with
 no extent sharing (the worst case): the first pass (`recv`), the final pass (`final`), the deadline stage's first pass on fresh
 folders (`recv-f5a.*`), and one more for the partial receives and the corrupted-stream stage that leave folders until the images
 are deleted.
 */
export const FIXTURE_COPY_SETS_KEPT = 4;

/**
 Free space in GiB every fixture copy keeps before it starts a transfer: the guard minimum the fixture stages pass as
 `--min-free-gib`. One is the smallest the guard can state in whole GiB, and the fixture images are small.
 */
export const FIXTURE_GUARD_MIN_GIB = 1;

/**
 Margin on the fixture images' written size, in percent: the plan's own quarter margin on every size estimate
 (`doc/planning/repository-on-the-860-drive.md`, section "Capacity policy").
 */
export const FIXTURE_IMAGE_MARGIN_PERCENT = 25;

/**
 Size in whole GiB of each fixture image: what the stages write (the populated data of every subvolume in every copy set kept)
 plus the guard minimum, with the margin, rounded up. The images are sparse files, so the size costs nothing until written.

 @returns the size in GiB

 @example
 ```ts
 import { fixtureImageGiB, } from './constants.ts';

 const size = fixtureImageGiB();
 ```
 */
export function fixtureImageGiB(): number {
  /**
   MiB the fixture stages write, with no sharing between the copies.
   */
  const writtenMiB = FIXTURE_MAIN_DATA_MIB * FIXTURE_SUBVOLUME_COUNT
    * FIXTURE_COPY_SETS_KEPT;
  /**
   GiB the stages write plus the free space the guard keeps.
   */
  const neededGiB = ((writtenMiB * BYTES_PER_MIB) / BYTES_PER_GIB) + FIXTURE_GUARD_MIN_GIB;
  return Math.ceil(neededGiB * (1 + (FIXTURE_IMAGE_MARGIN_PERCENT / 100)),);
}

/**
 How many times the footprint measurement walks the checkout when worktrees removed meanwhile make a walk fail: the author's
 bound. One walk takes about 12.5 minutes (plan, section "Measurements") and agents remove a worktree a few times an hour, so
 one more walk usually succeeds; three bounds the wait.
 */
export const FOOTPRINT_MEASURE_ATTEMPTS = 3;
