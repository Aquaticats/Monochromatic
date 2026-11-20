/**
 The codes the operating system gives a read inside a process's folder when the process ended: one for a folder or a record that
 is already gone, and one for a record whose task the kernel released between the folder being listed and the record being read.

 A process list always holds such entries, and reading one is no failure: on 2026-11-19 a driver that read the program of a run's
 process found `ESRCH` where the process had ended mid-scan, ended the run `idle` on it, and the unit stopped with the run's own
 watcher still to be waited for.

 @module
 */

/**
 The failures that mean the process behind a listed entry ended, so nothing can be read about it.

 `ENOENT` is the folder or the record that is already gone, which Linux answers once the entry is removed. `ESRCH` is the record
 the kernel no longer has a task for: `/proc/<pid>/comm`, `cgroup`, `stat` and `cmdline` all answer it from the read, after the
 folder has been listed and before the record has been read. The `cwd` link is the exception, which answers `ENOENT`.
 */
export const PROCESS_ENDED_CODES: ReadonlySet<string> = new Set<string>([
  'ENOENT',
  'ESRCH',
]);
