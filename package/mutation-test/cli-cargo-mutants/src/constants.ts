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
 Flag the campaign task takes for the container memory limit; forwarded only when the caller names a limit.
 */
export const memoryFlag = '--memory';

/**
 Flag the campaign task takes for the container CPU count; forwarded only when the caller names a count.
 */
export const cpusFlag = '--cpus';

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
 Program that holds a slot lock while the slotted command runs.
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
 Exit status `flock` reports when another process holds the slot, chosen so no wrapped command's own status collides.
 */
export const slotConflictExitCode = 200;

/**
 Milliseconds to wait before every slot is tried again after all of them were held.
 */
export const defaultPollIntervalMs = 15_000;

/**
 Directory under the scratch directory that holds the slot lock files; shared by every launcher on the host,
 so it must not vary with the per-launch scratch location unless the caller names it.
 */
export const lockDirectorySegment = 'gate-slots';

/**
 Environment variable that names the slot count, read when `--slot-count` is absent.
 */
export const slotCountEnvironmentVariable = 'GATE_SLOT_COUNT';

/**
 Environment variable that names the slot lock directory, read when `--lock-dir` is absent.
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
