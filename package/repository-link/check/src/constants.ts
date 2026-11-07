/**
 Names, statuses and bounds of the repository link check, each with where it comes from. The plan is
 `doc/planning/repository-on-the-860-drive.md`, section "Work items per tool", item W1.

 @module
 */

/**
 Exit status of the Git wrapper's check: Git's own fatal status.
 */
export const EXIT_GIT_FATAL = 128;

/**
 Exit status of every other tool's check: `EX_TEMPFAIL`, a failure that a later try can cure.
 */
export const EXIT_TEMPFAIL = 75;

/**
 Seconds the check waits for the filesystem before it reports that the drive did not answer. The plan asks for "a few
 seconds"; `stat` on a stalled drive can block in the kernel, so the caller enforces it.
 */
export const DEFAULT_DEADLINE_SECONDS = 5;

/**
 Milliseconds in a second.
 */
export const MILLISECONDS_PER_SECOND = 1_000;

/**
 Folder of the identity file below the home folder, and its name: the plan's `~/.config/monochromatic-dev/repository-link.json`.
 */
export const IDENTITY_RELATIVE_PATH = '.config/monochromatic-dev/repository-link.json';

/**
 File inside `.git` that holds the first commit hash of `main`, written at cut-over: a folder named `.git` alone is not
 an identity.
 */
export const ROOT_COMMIT_MARKER = 'monochromatic-root-commit';

/**
 The mountinfo file of the running process.
 */
export const MOUNTINFO_PATH = '/proc/self/mountinfo';

/**
 Folder of udev's links from filesystem UUID to block device.
 */
export const BY_UUID_FOLDER = '/dev/disk/by-uuid';

/**
 Mountinfo field number (counting from one) of the mount point; the fields before the dash separator are fixed.
 */
export const MOUNTINFO_MOUNT_POINT_FIELD = 5;

/**
 Seconds a wait of `repository-link-run --wait-seconds` pauses between two asks: the plan's "re-asks every second".
 */
export const WAIT_STEP_SECONDS = 1;

/**
 Mode of the installed bins.
 */
export const INSTALLED_BIN_MODE = 0o755;

/**
 Mode of the identity file: readable by the owner's other tools, writable only by the owner.
 */
export const IDENTITY_FILE_MODE = 0o644;

/**
 Name of the folder below the data home that holds the installed copy.
 */
export const INSTALL_FOLDER = 'monochromatic-dev/repository-link';
