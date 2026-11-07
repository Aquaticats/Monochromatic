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
 Bytes in a GiB.
 */
export const BYTES_PER_GIB: number = 2 ** 30;

/**
 Milliseconds in a second.
 */
export const MILLISECONDS_PER_SECOND = 1_000;

/**
 Most output a child command may print, in bytes.
 */
export const MAX_OUTPUT_BYTES: number = 64 * 1_024
  * 1_024;

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
 Most output one Git call of the scan may print, in bytes: a status listing of a worktree with many untracked files.
 */
export const SCAN_MAX_OUTPUT_BYTES: number = 256 * 1_024
  * 1_024;

/**
 Seconds each Git call of the scan may take when the caller gives no limit: the plan's value, section "Quiesce and
 freeze".
 */
export const SCAN_DEFAULT_PER_WORKTREE_SECONDS = 120;
