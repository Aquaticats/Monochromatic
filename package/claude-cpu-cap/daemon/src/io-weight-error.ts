/**
 Why the watcher could not set the IO weight: no weight file exists in a cgroup that does exist
 (`no-weight-file`), or a weight file could not be read or written (`file-access-failed`).
 */
export type IoWeightFailureReason = 'file-access-failed' | 'no-weight-file';

/**
 Raised when the `claude-code-bash` cgroup exists but its IO weight could not be set.
 The message names the affected files and what to check, so a log line built from it is complete by itself.

 @example
 ```ts
 throw new IoWeightError({
   reason: 'file-access-failed',
   files: ['/sys/fs/cgroup/x/io.bfq.weight'],
   message: 'could not set the default weight 10 in /sys/fs/cgroup/x/io.bfq.weight: EACCES: permission denied',
 },);
 ```
 */
export class IoWeightError extends Error {
  /**
   Which failure this is.
   */
  public readonly reason: IoWeightFailureReason;

  /**
   The weight files the failure concerns: the missing ones, or the ones whose write failed.
   */
  public readonly files: readonly string[];

  /**
   Creates one IO weight failure.

   @param reason - which failure this is

   @param files - the weight files the failure concerns

   @param message - plain-language diagnostic that names the files

   @param cause - the underlying read or write errors, when there are any

   @example
   ```ts
   new IoWeightError({ reason: 'no-weight-file', files: [], message: 'no weight file in /sys/fs/cgroup/x', },);
   ```
   */
  public constructor({
    reason,
    files,
    message,
    cause,
  }: Readonly<{
    reason: IoWeightFailureReason;
    files: readonly string[];
    message: string;
    cause?: unknown;
  }>,) {
    super(
      message,
      { cause, },
    );
    this.name = 'IoWeightError';
    this.reason = reason;
    this.files = files;
  }
}
