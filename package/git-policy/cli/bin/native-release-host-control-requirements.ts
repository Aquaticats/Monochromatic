/**
 What a wrapper release requires of the btrfs controls on this host: the run, the deadline it is given, and the names a
 green run has to hold.

 The gate container runs on an overlay filesystem, where each control that needs the btrfs ioctls prints `BTRFS CONTROL
 NOT RUN` and passes, so the container gate is green without them. The owner decided on 2026-11-10 that a real wrapper
 release waits for `native:test:host:btrfs` on a btrfs host and that a control which did not run fails it.
 */
//region The run and its deadline

/**
 The mise task whose run is the evidence a release waits for: every target on this host, with no acknowledgement
 variable set, on a fixture root that must be on btrfs.
 */
export const hostControlTask = '//package/git-policy/cli:native:test:host:btrfs';

/**
 Deadline the gate gives that run, in seconds.

 A person waits behind this run for a release, and nothing should ever wait forever or indefinitely (the owner's words),
 so a run that hangs is stopped and refuses the release instead of holding it open. The value leaves room for a slower
 host than the one the run was timed on, without leaving a hung run in place for a working day.
 */
export const hostControlLimitSeconds = 7_200;

/**
 Seconds the full run of the task took on this host, which the deadline above is sized from.

 Measured on 2026-11-10 in its own `systemd-run --user --scope` limited to 600% CPU and 20 GB, with this host's other
 agents working.
 */
export const measuredFullRunSeconds = 0;

/**
 When that run was timed, which a refusal quotes beside the deadline it sizes.
 */
export const measuredFullRunDate = '2026-11-10';

/**
 Milliseconds in one second, for the deadline the run is given.
 */
export const millisecondsPerSecond = 1_000;

//endregion The run and its deadline

//region The names a green run holds

/**
 One test the release must see pass, by the binary nextest prints for it and the name the test has within it.
 */
export type HostControlName = {
  /**
   Binary ID in nextest's status line, such as `monochromatic-git-policy-cli` or
   `monochromatic-git-policy-cli::native_binary`.
   */
  readonly suite: string;
  /**
   The test's name within that binary, which is its module path and function name.
   */
  readonly test: string;
};

/**
 The controls a green run must name, one from each path that runs btrfs code: the ioctls themselves, the migration that
 builds subvolumes, and a binary-level control that goes through `on_btrfs_or_acknowledged`.

 A renamed or deleted control refuses the release until this list is brought up to date, which is the point: the release
 waits for a run whose own lines prove these controls happened.
 */
export const requiredHostControls: readonly HostControlName[] = [
  {
    suite: 'monochromatic-git-policy-cli',
    test: 'subvolume::fs_btrfs_ioctl::tests::btrfs_create_snapshot_and_destroy_each_work_once',
  },
  {
    suite: 'monochromatic-git-policy-cli',
    test: 'subvolume::subvolume_btrfs_tests::a_real_migration_makes_subvolumes_and_clones_file_data',
  },
  {
    suite: 'monochromatic-git-policy-cli::native_binary',
    test: 'worktree_ladder::a_worktree_added_from_the_main_checkout_is_a_subvolume',
  },
];

/**
 The reason a control's own line gives when it needs a filesystem that is not btrfs, which is the one `BTRFS CONTROL NOT
 RUN` line a btrfs host prints.
 */
const otherFilesystemReason = 'needs a filesystem that is not btrfs';

/**
 What a control prints when it found nothing to check here.
 */
const notRunMarker = 'BTRFS CONTROL NOT RUN';

/**
 The tag nextest prints in front of a test that passed.
 */
export const passingTag = 'PASS';

/**
 The tag nextest prints in front of a test that failed.
 */
export const failingTag = 'FAIL';

/**
 Read one of nextest's status lines as the binary and the test it names.

 The reporter prints `PASS [ 0.176s] (1/1) <binary> <test name>` for a test that passed, and the same shape for one that
 failed. The line is scanned rather than matched, in one pass over the line: the tag decides the kind, the bracket that
 holds the run's counter ends at the first `)`, and what follows is the binary up to the next space and then the test's
 name.

 @param line - one line of the run's output

 @param tag - the status tag the line must start with

 @returns the binary and test name as the only entry, or no entry when the line is not a status line of that tag

 @example
 ```ts
 statusNamesOf({ line: '        PASS [   0.176s] (1/1) a b::c', tag: passingTag });
 ```
 */
export function statusNamesOf({
  line,
  tag,
}: {
  readonly line: string;
  readonly tag: string;
}): readonly HostControlName[] {
  /**
   The line without its leading progress indent.
   */
  const trimmed = line.trimStart();
  if (!trimmed.startsWith(`${tag} [`))
    return [];
  /**
   End of the bracket that holds the test's duration.
   */
  const durationEnd = trimmed.indexOf(']');
  if (durationEnd === (-1))
    return [];
  /**
   End of the bracket that holds the run's counter.
   */
  const counterEnd = trimmed.indexOf(
    ')',
    durationEnd,
  );
  if (counterEnd === (-1))
    return [];
  /**
   The binary and then the test's name, after the counter.
   */
  const name = trimmed.slice(counterEnd + 1)
    .trim();
  /**
   Where the binary ends and the test's name begins.
   */
  const boundary = name.indexOf(' ');
  if (boundary === (-1))
    return [];
  /**
   The binary nextest names.
   */
  const suite = name.slice(
    0,
    boundary,
  );
  /**
   The test's name within it.
   */
  const test = name.slice(boundary + 1)
    .trim();
  if ((suite === '') || (test === ''))
    return [];
  return [{
    suite,
    test,
  }];
}

/**
 Every test one tag's status lines name, in the order the run printed them.

 @param output - the run's captured output

 @param tag - the status tag to read

 @returns one entry per matching line

 @example
 ```ts
 namesIn({ output, tag: passingTag });
 ```
 */
export function namesIn({
  output,
  tag,
}: {
  readonly output: string;
  readonly tag: string;
}): readonly HostControlName[] {
  return output.split('\n')
    .flatMap(function namesOfLine(line) {
      return statusNamesOf({
        line,
        tag,
      });
    });
}

/**
 The lines a run printed for controls that found nothing to check here.

 @param output - the run's captured output

 @returns the lines, each as the control wrote it, without its indent

 @example
 ```ts
 notRunLines(passingOutput);
 ```
 */
export function notRunLines(output: string): readonly string[] {
  return output.split('\n')
    .filter(function isNotRunLine(line) {
      return line.includes(notRunMarker);
    })
    .map(function trimmed(line) {
      return line.trim();
    });
}

/**
 Whether a control's own line says it needs a filesystem that is not btrfs, which is the one such line a btrfs host
 prints and is not a skip.

 @param line - a line the run printed for a control that did not run

 @returns true when the control needs another filesystem by its own words

 @example
 ```ts
 needsAnotherFilesystem(notRunLine);
 ```
 */
export function needsAnotherFilesystem(line: string): boolean {
  return line.includes(otherFilesystemReason);
}

/**
 The run's last lines that hold text, for a run that failed without naming a test.

 @param output - the run's captured output

 @returns the last few non-empty lines, without their indent

 @example
 ```ts
 lastLines('error: could not compile');
 ```
 */
export function lastLines(output: string): readonly string[] {
  /**
   How many of the run's last lines a refusal quotes, enough for a compiler's error and the note under it.
   */
  const quotedLineCount = 10;
  return output.split('\n')
    .filter(function hasText(line) {
      return line.trim() !== '';
    })
    .slice(-quotedLineCount)
    .map(function trimmed(line) {
      return line.trim();
    });
}

//endregion The names a green run holds
