/**
 Defaults and fixed protocol values of the parallel cargo-mutants launcher and the gate-slot limiter.

 Every value a caller may want to change has a command-line option that overrides it; the constants only name the
 defaults, so no number appears bare in the logic.
 */

//region Per-unit systemd caps

/**
 Memory cap, as a systemd `MemoryMax` value, of one campaign unit.
 The unit holds the whole campaign process tree (the task, the container client and the build), not only the container,
 whose own limit the task sets.
 */
export const defaultUnitMemoryMax = '12G';

/**
 CPU cap, as a systemd `CPUQuota` value, of one campaign unit; `500%` is five hardware threads.
 */
export const defaultUnitCpuQuota = '500%';

/**
 Task limit, as a systemd `TasksMax` value (a whole number), of one campaign unit.
 A control group counts threads as well as processes, so the limit bounds both together.

 One unit is one file's campaign and holds only the host side of it: the shell, the slot limiter, `mise`, the task's
 Node process and its container client (`podman build` for the image, then `podman run`). The container's own processes
 sit in a `libpod-<id>.scope` outside the unit and carry the container's `--pids-limit`, so neither the slot count nor
 the container's process count multiplies into this value.

 No earlier campaign record carries a process count, so the value is derived from a measurement made on 2026-11-07
 without starting a campaign. `pids.peak` of a `systemd-run --user --scope` that ran
 `sh -c 'mise run <package>:campaign:gate-slot -- mise run //package/mutation-test/cli:run -- --help'` was 39 in
 three runs, and 40 with a `podman run --rm` of a local image added after it. A bare `sh` that ran `podman run --rm`
 alone peaked at 18 or 19, and one that ran `podman build` at 21. The widest host side is the chain (40) with one more
 container client beside it (about 20), so about 60. The value, 256, is a power of two over three times that, because
 a start that hits the limit fails with a bare `EAGAIN` that names no limit (see `src/process-limits.ts`), while a
 runaway fork loop is still stopped far below the user manager's own limit (`pids.max` 32768 of `user-1000.slice`).
 Raise it with `--unit-tasks-max` when a unit reports that failure.
 */
export const defaultUnitTasksMax = 256;

/**
 Disk weight, as a systemd `IOWeight` value, of one campaign unit.
 The owner's rule (`CLAUDE.md`, rule RXS) is never above 10, and lower for disk-heavy work; a unit left without the
 property gets systemd's default of 100. 10 is the value every other capped unit of the repository and every started
 campaign container (`lowestDiskWeight` in `src/container-limits.ts`) already uses, so the unit's own build, image and
 report writes weigh the same as the container's, which ignores the unit's value because it sits in its own scope.
 */
export const unitIoWeight = 10;

/**
 Highest `IOWeight` a unit this package starts may carry.
 The owner's rule (`CLAUDE.md`, rule RXS) is that a per-command cgroup never sets `IOWeight` above 10.
 */
export const maximumUnitIoWeight = 10;

//endregion Per-unit systemd caps

//region Campaign task

/**
 Test timeout, in seconds, forwarded to the campaign task for each mutant.
 Longer than the wrapper's own default so a loaded host does not turn slow mutants into timeouts.
 */
export const defaultTimeoutSeconds = 900;

/**
 Runs of one file's campaign before the launcher gives up; the first run plus two reruns of a failed baseline.
 */
export const defaultAttempts = 3;

/**
 Text cargo-mutants prints when the baseline run of the unmutated source fails.
 A load-sensitive test can fail it, so a log holding this text is the only reason to rerun a campaign.
 The three spaces are the report's column padding and are part of the text.
 */
export const defaultRetryMarker = 'FAILED   Unmutated baseline';

/**
 Flag the campaign task takes once per source-file scope.
 */
export const defaultScopeFlag = '--file';

/**
 Flag the campaign task takes for the per-mutant timeout in seconds.
 */
export const timeoutFlag = '--timeout';

/**
 Exit status cargo-mutants returns when at least one mutant timed out, in its build or in its tests, and the baseline
 passed; see `doc/troubleshooting/cargo-mutants-timeout-exit-status.md`.
 */
export const timeoutExitStatus = 3;

/**
 Exit status cargo-mutants returns when at least one mutant was missed and none timed out.
 */
export const missedExitStatus = 2;

/**
 Flag the campaign task takes for the per-mutant build limit in seconds; forwarded only when the caller names a limit.
 */
export const buildTimeoutFlag = '--build-timeout';

/**
 Flag the campaign task takes once per mutant-name regular expression; forwarded once per pattern the caller names.
 */
export const examineFlag = '--examine';

/**
 Flag the campaign task takes for the container memory limit; forwarded only when the caller names a limit.
 */
export const memoryFlag = '--memory';

/**
 Flag the campaign task takes for the container CPU count; forwarded only when the caller names a count.
 */
export const cpusFlag = '--cpus';

/**
 Flag the campaign task takes for the number of passes (1 or 2); forwarded only when the caller names the passes.
 */
export const passesFlag = '--passes';

/**
 Placeholder of the image-tag template that stands for the campaign label.
 */
export const tagLabelPlaceholder = '{label}';

/**
 Placeholder of the image-tag template that stands for the file's zero-based position in the argument list.
 The template must contain it, because two concurrent campaigns that share a tag mutate each other's image.
 */
export const tagIndexPlaceholder = '{index}';

/**
 Image-tag template: the label followed by the file's position.
 */
export const defaultTagTemplate: string = `${tagLabelPlaceholder}${tagIndexPlaceholder}`;

//endregion Campaign task

//region Names and locations

/**
 Prefix of every systemd unit name and every log file name when the caller names none.
 */
export const defaultNamePrefix = 'mutpar';

/**
 Environment variable that names the working tree the campaigns run in, read when `--root` is absent.
 */
export const rootEnvironmentVariable = 'MUT_ROOT';

/**
 Scratch location under the home directory, as path segments: scratch never lives in the system temporary directory,
 whose backing device is unknowable.
 */
export const scratchSegments = [
  'temp',
  'agent'
] as const;

/**
 Directory under the scratch directory that each unit's `TMPDIR` points at.
 */
export const unitTemporarySegment = 'tmp';

/**
 Longest systemd unit name, in characters, the launcher builds; systemd allows 255 including the `.service` suffix.
 */
export const maximumUnitNameLength = 200;

/**
 Longest image tag, in characters, that Podman accepts.
 */
export const maximumImageTagLength = 128;

//endregion Names and locations

//region Executables

/**
 Program that starts a transient unit.
 */
export const systemdRunProgram = 'systemd-run';

/**
 Shell that runs the retry loop inside the unit.
 */
export const shellProgram = 'sh';

/**
 Runtime that runs the gate-slot limiter and its own entry point.
 */
export const nodeProgram = 'node';

/**
 Search tool that looks for the retry marker in a campaign log.
 */
export const searchProgram = 'rg';

/**
 Task runner that starts the campaign task.
 */
export const taskRunnerProgram = 'mise';

/**
 Program that probes, once and without waiting, the kernel lock the earlier release of this package took on a slot file or an
 evidence directory (`src/legacy-flock.ts`). Nothing holds a lock with it while a command runs any more: the slots and the
 campaign are numbered claim files (`src/owner-claim.ts`).
 */
export const flockProgram = 'flock';

/**
 Program that lowers the scheduling priority of the slotted command.
 */
export const niceProgram = 'nice';

//endregion Executables

//region Gate slots

/**
 Campaign containers that may run at once across the whole host, when the caller names no other count.
 Six slots at the lowest nice priority fit the user's 75 percent CPU hold on the 16 core, 32 thread host
 (decided 2026-10-14); lower the count when the host degrades.
 */
export const defaultSlotCount = 6;

/**
 Nice adjustment that puts a slotted command behind interactive work; the highest value is the lowest priority.
 */
export const slotNiceAdjustment = 19;

/**
 Exit status the probe's `flock` reports when a process of the earlier release holds the lock, chosen so no other status of
 `flock` collides.
 */
export const slotConflictExitCode = 200;

/**
 Longest milliseconds a gate waits before it tries every slot again after all of them were held. A slot's release wakes the
 gate at once (`src/directory-watch.ts`); this bound is only the fallback for a holder that died, which makes no event.
 */
export const defaultPollIntervalMs = 15_000;

/**
 Milliseconds between the notices a waiting gate writes, as the wrapper's lock-wait notices are spaced
 (`package/git-policy/cli/src/native/push/wait_progress.rs`, ten seconds).
 */
export const defaultNoticeIntervalMs = 10_000;

/**
 Milliseconds a gate waits in silence before its first notice, never more than the notice interval itself
 (`wait_progress.rs`, one second), so brief contention between healthy gates prints nothing.
 */
export const noticeGraceMs = 1_000;

/**
 Directory under the scratch directory that holds the slot claim files; shared by every launcher on the host,
 so it must not vary with the per-launch scratch location unless the caller names it.
 */
export const lockDirectorySegment = 'gate-slots';

/**
 Environment variable that names the slot count, read when `--slot-count` is absent.
 */
export const slotCountEnvironmentVariable = 'GATE_SLOT_COUNT';

/**
 Environment variable that names the slot claim directory, read when `--lock-dir` is absent.
 */
export const lockDirectoryEnvironmentVariable = 'GATE_SLOT_DIRECTORY';

//endregion Gate slots

//region Accepted characters

/**
 Longest text accepted as a whole number option, so the value converts to a number exactly.
 */
export const maximumWholeNumberDigits = 10;

/**
 Characters a systemd unit name part keeps; every other character becomes a hyphen.
 */
export const unitNameCharacters = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-';

/**
 Characters a log file name part keeps; every other character, the path separator included, becomes a hyphen.
 */
export const logNameCharacters = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_.-';

/**
 Characters of a label or name prefix after its first character, which must be a letter or number.
 */
export const plainNameCharacters = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_-';

/**
 Characters of a systemd resource value such as `12G`, `500%` or `infinity`.
 */
export const systemdValueCharacters = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789.%';

/**
 Characters of an image tag after its first character, which must be a letter, number or underscore.
 */
export const imageTagCharacters = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_.-';

/**
 Characters of an environment variable name after its first character, which must be a letter or underscore.
 */
export const environmentNameCharacters = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_';

//endregion Accepted characters

/**
 Name of the file, in a pass's evidence directory, that says a start of a container was attempted there. It is written
 right before the start, so it does not prove a container ran (a kill in between leaves it), but a run that started and was
 killed before anything else was written is never without it.
 */
export const containerStartedFile = 'container-started.json';

/**
 Name of the file, inside an attempt's directory, that says the attempt was made by a runner that writes
 {@link containerStartedFile} before every container start. It is written when the attempt directory is made, long before
 a launch. Where it is there, an attempt with no started marker holds no container that started; an attempt without it may
 belong to an older runner (or to a newer one killed before it wrote the file), and its missing marker proves nothing. The
 record is per attempt, never per campaign: an older runner can resume a directory a newer one began, and its attempts must
 not be vouched for by the newer one's.
 */
export const attemptProtocolFile = 'attempt-protocol.json';

/**
 Version recorded in {@link attemptProtocolFile}: the started marker is written before every container start.
 */
export const startedMarkerProtocolVersion = 1;
