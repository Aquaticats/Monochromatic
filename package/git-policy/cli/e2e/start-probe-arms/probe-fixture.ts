/**
 What the start probe's guest arms name: the built wrapper and the words that
 select its root mode, the record the probe writes, the dataset the design names,
 the configuration and the checkout, the file-backed pool the guest makes, and the
 bound each arm gets.

 The words are choice 2's, so a bare mode word is refused under the installed name
 and this shape is what reaches the modes
 (`doc/decision/cli-git-zfs-root-modes-the-eleven-choices-of-the-one-program-design.md`).

 @example
 ```ts
 const { name, logDir } = options();
 ```
 */

/**
 Where the crate is unpacked and built inside the guest, which every account may
 traverse.
 */
export const crateDirectory = '/opt/git-policy-cli-start-probe';

/**
 The command every arm starts: the wrapper this guest built, and the words that
 select a root mode on any file.
 */
export const handler: string = `${crateDirectory}/package/git-policy/cli/target/release/monochromatic-git-policy-cli monochromatic-git-policy-cli zfs-root`;

/**
 The same command built with the probe's raised-deadline feature, which the
 release compiles out.

 Choice 11 answers that a build made for a control says so in its manifest, so the
 one control whose point is a step that finishes after the shipped bound runs this
 build and never the release's bytes
 (`doc/decision/cli-git-zfs-root-modes-the-eleven-choices-of-the-one-program-design.md`,
 choice 11).
 */
export const raisedDeadlineHandler: string = `${crateDirectory}/package/git-policy/cli/target/raised-deadline/release/monochromatic-git-policy-cli monochromatic-git-policy-cli zfs-root`;

/**
 The record the probe writes and the arms read.
 */
export const record = '/run/git-worktree-clone-mount-probe/result';

/**
 The probe dataset the design names
 (`doc/planning/cli-git-zfs-start-probe-design.md`, "The source").
 */
export const probeDataset = 'zpcachyos/nosnap/git-worktree-probe';

/**
 The fixture's encrypted dataset, whose empty `ready` snapshot is the source of
 M1's unloaded-key control. It is never the configured source except inside that
 control, which restores the configuration afterwards.
 */
export const probeEncrypted = 'zpcachyos/nosnap/git-worktree-probe-encrypted';

/**
 The passphrase file the encrypted fixture dataset's key is loaded from, which only
 the guest's root reads.
 */
export const probeKeyFile = '/root/git-worktree-probe-key';

/**
 The fixture's file-backed pool, created by this driver and destroyed with the
 guest.
 */
export const pool = 'zpcachyos';

/**
 The pool's image inside the guest.
 */
export const poolImage = '/home/zfs-probe-pool.img';

/**
 The pool image's size: the probe mounts read-only and writes no block.
 */
export const poolImageSize = '1G';

/**
 Where the wrapper's configuration lives, as the build names it.
 */
export const configuration = '/etc/monochromatic-git-policy-cli/git-worktree-clone-mounts.jsonc';

/**
 The account the guest image creates, which the configuration pins as the managed
 account.
 */
export const account = 'ubuntu';

/**
 The checkout path the configuration pins, which the checkout-root arm opens
 read-only.
 */
export const checkout = '/home/git-worktree-probe-checkout';

/**
 The prefix the guest's OpenZFS 2.4.4 userspace is installed under.
 */
export const openZfsPrefix = '/opt/openzfs-2.4.4';

/**
 The `PATH` the guest's OpenZFS 2.4.4 userspace needs.
 */
export const openZfsPath: string = `${openZfsPrefix}/sbin:${openZfsPrefix}/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin`;

/**
 Milliseconds the guest's crate build may take: two release builds of the wrapper,
 one of them into a target folder of its own.
 */
export const crateBuildTimeoutMilliseconds = 2_400_000;

/**
 Milliseconds one probe arm may take.
 */
export const armTimeoutMilliseconds = 900_000;

/**
 How many arm bounds the kill matrix gets: sixteen hold points, each run as five
 kills and one control, each with its own run of the wrapper after it.
 */
const killMatrixArmBounds = 2;

/**
 Milliseconds the kill matrix may take.
 */
export const killMatrixTimeoutMilliseconds: number = armTimeoutMilliseconds
  * killMatrixArmBounds;

/**
 How many arm bounds the timing arms get: warm rounds, cold rounds with the page
 cache dropped, and a guest with three hundred extra mount namespaces.
 */
const timingArmBounds = 2;

/**
 Milliseconds one timing arm may take.
 */
export const timingTimeoutMilliseconds: number = armTimeoutMilliseconds * timingArmBounds;

/**
 The lines a green run must find, one per claim it makes.
 */
export const greenMarkers: readonly string[] = [
  'PROBE verdict passed',
  'PROBE attach passed',
  'PROBE host table unchanged',
  'PROBE source absent afterwards',
  'PROBE refused step attach-place errno EPERM',
  'PROBE timed-out in flight unmount',
  'PROBE killed mode left a started record',
  'PROBE held source not exercised',
  'PROBE absent source recorded',
  'PROBE txg window clean',
  'PROBE scan saw the worker and the holder',
  'PROBE scan saw the worker alone',
  'PROBE refusal controls refused',
  'GREEN done',
];

/**
 The lines the no-configuration arm must find, one per claim it makes.
 */
export const noConfigurationMarkers: readonly string[] = [
  'NO-CONFIGURATION passed-with-no-file',
  'NO-CONFIGURATION source-is-the-designs-name',
  'NO-CONFIGURATION absent-dataset-blocks',
  'NO-CONFIGURATION absent-dataset-names-its-cause',
  'NO-CONFIGURATION absent-dataset-names-the-setup-commands',
  'NO-CONFIGURATION done',
];

/**
 The shell assertion both M1 controls share: a record line must be there, and its
 absence fails the arm.
 */
export const recordLineAssertion: readonly string[] = [
  'check() {',
  '  if ! grep --quiet --fixed-strings "$1" "$2"; then',
  '    echo "MISSING $1 in $2"',
  '    cat "$2"',
  '    exit 1',
  '  fi',
  '  echo "M1 $3"',
  '}',
];

/**
 The shell header every added arm shares: the handler, the record, the source and
 the leak counter, then the arm's own lines.

 @param lines - the arm's own shell lines

 @returns the whole script, its header first

 @example
 ```ts
 probeScript({ lines: ['$H start-probe'] });
 ```
 */
export function probeScript({
  lines,
}: {
  readonly lines: readonly string[];
},): string {
  return [
    'set -eu',
    `export PATH=${openZfsPath}`,
    `export LD_LIBRARY_PATH=${openZfsPrefix}/lib`,
    `H=${handler}`,
    `R=${raisedDeadlineHandler}`,
    `REC=${record}`,
    `CONF=${configuration}`,
    `SRC='${probeDataset}@ready'`,
    `PAT='^${handler} start-probe$'`,
    'now_us() { echo $(( $(date +%s%N) / 1000 )); }',
    'leaks() { n=0; for ns in /proc/[0-9]*/ns/mnt; do if nsenter --mount="$ns" cat /proc/self/mountinfo 2>/dev/null | grep -q "git-worktree-probe@ready"; then n=$((n+1)); fi; done; echo "leaks=$n"; }',
    ...lines,
  ].join('\n',);
}
