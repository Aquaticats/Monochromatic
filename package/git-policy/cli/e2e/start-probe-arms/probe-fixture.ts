/**
 What the start probe's guest arms name: the built wrapper and the words that
 select its root mode, the record the probe writes, the dataset the design names,
 the configuration and the checkout, the file-backed pool the guest makes, and the
 bound each arm gets; and what one arm reads back out of a guest build's log.

 The words are choice 2's, so a bare mode word is refused under the installed name
 and this shape is what reaches the modes
 (`doc/decision/cli-git-zfs-root-modes-the-eleven-choices-of-the-one-program-design.md`).

 The log reading sits here because the arms share it and because a unit test may
 reach a unit-testable module by this name: `test-import/require-eventual-artifact`
 refuses a test that imports sibling source otherwise, and `bin/mvm-guest-*.ts` is
 the shared set this round changes nothing in.

 @example
 ```ts
 const { name, logDir } = options();
 ```
 */

import {
  doneStatusOf,
} from '../../bin/mvm-guest-command-line.ts';

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
 Milliseconds the host gives the guest's build past its own bound, before it stops
 waiting for the script's status line: the guest's own `timeout` fires a minute
 inside that bound, and the pull of a log this size costs the answer's own seconds.
 */
export const buildFinishMarginMilliseconds = 180_000;

/**
 How many lines of a guest build's own log a refusal carries, counted back from its
 status line: the two builds' own `tail --lines=3` outputs fit inside it, and a
 longer refusal is one nobody reads.
 */
export const tailLineCount = 12;

/**
 A guest build's log whose status line names no zero status, or names none at all.
 */
export type FailedGuestBuild = {
  /**
   True for a build that ended other than zero, or that never said how it ended.
   */
  readonly failed: true;
  /**
   The status the build's log names, or `without a status` when it names none.
   */
  readonly status: string;
  /**
   The log's last lines before its status line, as the guest wrote them.
   */
  readonly tail: readonly string[];
};

/**
 How a guest build ended, as the log it writes answers.
 */
export type GuestBuildEnd = {
  /**
   True for a build whose log's status line names zero.
   */
  readonly failed: false;
} | FailedGuestBuild;

/**
 The log's last lines before its status line.

 @param text - the whole log the guest wrote

 @returns the last {@link tailLineCount} lines with text in them, in the order the
 guest wrote them, with the status line left out because a refusal names that itself

 @example
 ```ts
 logTailOf({ text: 'error[E0308] …\nDONE=101\n' }); // ['error[E0308] …']
 ```
 */
export function logTailOf({ text, }: { readonly text: string; },): readonly string[] {
  /**
   Every line of the log but the status lines, which a `split` of a log that ends in
   a newline leaves an empty one of.
   */
  const lines = text
    .split('\n',)
    .filter(function hasText(line: string,): boolean {
      return line.length > 0;
    },)
    .filter(function isNotTheStatus(line: string,): boolean {
      return !line.trim()
        .startsWith('DONE=',);
    },);
  return lines.slice(-tailLineCount,);
}

/**
 How a guest build ended, read from the log the host pulled.

 The build runs as a detached script (`bin/mvm-guest-detached-script.ts`), so its
 log is the guest's own output with the status line the script appends after the two
 `tail --lines=3` outputs. A build the guest's kernel ended, or one a compiler
 refused, prints its own words and the tail is what a person reads.

 @param text - the whole log the guest wrote

 @returns whether the build ended zero, and for a failed one its status and the tail
 of its log

 @example
 ```ts
 const ended = guestBuildEndOf({ text: pulled });
 ```
 */
export function guestBuildEndOf({ text, }: { readonly text: string; },): GuestBuildEnd {
  /**
   The status the log's own `DONE=` line names, absent when the log holds none.
   */
  const ended = doneStatusOf({ text, },);
  if (ended.found && (ended.status === '0'))
    return { failed: false, };
  return {
    failed: true,
    status: ended.found ? ended.status : 'without a status',
    tail: logTailOf({ text, },),
  };
}

/**
 Milliseconds one probe arm may take.
 */
export const armTimeoutMilliseconds = 900_000;

/**
 Milliseconds the kill matrix and the timing arm may take: twenty-five minutes,
 the bound those two arms carried before this round, because their own work is
 several times one arm's. The guest's own `timeout` gives them 1440 seconds.
 */
export const longArmTimeoutMilliseconds = 1_500_000;

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
    // Quoted because the handler is one path and then the words that select a
    // root mode: unquoted, the shell would run the words as a command of their
    // own, which is what every arm built on this header met until now.
    `H='${handler}'`,
    `R='${raisedDeadlineHandler}'`,
    `REC=${record}`,
    `CONF=${configuration}`,
    `SRC='${probeDataset}@ready'`,
    `PAT='^${handler} start-probe$'`,
    'now_us() { echo $(( $(date +%s%N) / 1000 )); }',
    'leaks() { n=0; for ns in /proc/[0-9]*/ns/mnt; do if nsenter --mount="$ns" cat /proc/self/mountinfo 2>/dev/null | grep -q "git-worktree-probe@ready"; then n=$((n+1)); fi; done; echo "leaks=$n"; }',
    ...lines,
  ].join('\n',);
}
