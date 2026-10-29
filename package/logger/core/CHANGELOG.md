# @monochromatic-dev/module-logger

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `@monochromatic-dev/module-logger` is now `@monochromatic-dev/logger-core`.

## 0.4.6

### Patch changes

- First release of `@monochromatic-dev/logger-core` through `npm-publish.yml`,
  so it carries npm provenance and validates the package's new trusted publisher.
  0.4.5 under this name was the first, manual publish after the rename from `@monochromatic-dev/module-logger`,
  which is now deprecated.
  The public API is unchanged.

## 0.4.5

### Patch changes

- Bound file sink growth with retention sweeps at verification:
  14 days,
  512 MiB,
  and 50,000 files per destination directory,
  overridable with `MONOCHROMATIC_LOG_MAX_AGE_DAYS`,
  `MONOCHROMATIC_LOG_MAX_BYTES`,
  and `MONOCHROMATIC_LOG_MAX_FILES`.
  A `.logger-retention-sweep` stamp throttles sweeps to one walk per five minutes per directory,
  and sweeps delete only logger-owned names,
  never files modified within an hour.
- Nest log files under a UTC day directory,
  `{YYYY-MM-DD}/{timestamp}.{uuid}.log.jsonl`,
  and create each process's file on its first record.
  The nesting is a deliberate layout change under the existing project-local destination.
  Verification probes writability with a `{timestamp}.{uuid}.probe.tmp` round-trip it always removes,
  so zero-record processes leave zero log files.
  Legacy flat `*.log.jsonl` strays age out by name prefix.
- Add `MONOCHROMATIC_LOG_FILE=off`,
  which disables the file sink entirely,
  including explicit destinations,
  and export the retention surface (`LogRetentionPolicy`,
  `NO_LOG_RETENTION_SWEEP`,
  `DEFAULT_LOG_MAX_AGE_DAYS`,
  `DEFAULT_LOG_MAX_BYTES`,
  `DEFAULT_LOG_MAX_FILES`) from the node entry.
- Purge the pre-0.4.5 `*.log.jsonl` backlog older than 14 days from every worktree of this repository
  as part of landing this release,
  keeping the deliberate run directories.

## 0.4.4

### Patch changes

- Neutralize terminal control characters in direct internal diagnostics,
  including default-file fallback notices and strict destination failures.
  Reuse the console sink's existing neutralizer so caught filesystem paths cannot inject terminal commands.
  Preserve diagnostic text and ordinary multiline output.

## 0.4.3

### Patch changes

- Keep project-local default logs when writable,
  falling back to persistent user state when project discovery or file verification is unavailable.
  Announce the verified fallback directory and its reason once per sink.
  Explicit argument and environment destinations remain strict.
- Resolve XDG state paths only when absolute,
  with conventional user log layouts on macOS and Windows.
  Keep all filesystem probes lazy and preserve serialized append completions.
- Create new JSONL files with mode `0600` where Unix modes apply.
  The [file destination contract](doc/file-destinations.md) documents ownership and consumer verification.

## 0.4.2

### Patch changes

- Observe shared sink completions once per active operation,
  and reuse fulfilled completions for synchronous sinks.
  Synchronous logging bursts still deliver every record without retaining continuations per record.
- Batch complete file records into serial appends with shared completions.
  Finish serialization before admission so reentrant serializers cannot overlap file writes.
- Give each file sink an independently owned filename,
  create it exclusively,
  and publish its append queue after successful verification.
  Separate sinks initialized in the same millisecond cannot corrupt a shared JSONL file.
- Reject serializers that produce no JSON value without inserting invalid JSONL or disabling later writes.

## 0.4.1

### Patch changes

- Serialize complete file-sink appends in invocation order.
  Concurrent records exceeding a filesystem write chunk no longer interleave and corrupt JSONL.
  Each write completion includes its queue wait,
  preserving the existing logger flush contract.
- Update repository metadata to `Aquaticats/Monochromatic` for publishing after the repository transfer.

## 0.4.0

### Minor Changes

- The default logger is built by the first log or flush call instead of at import,
   the default sink list is created at that moment,
   and the package declares `sideEffects: false`.
  Importing the root entry or `tagged` now runs no sink discovery,
   no timers,
   and no I/O,
   so global-scope-restricted runtimes such as Cloudflare Workers start without the four sink-verification warnings.
  The `initPromise` root export is removed;
   `flush()` awaits readiness internally,
   and `createLogger` still returns its instance's `initPromise`.
  Commit `7d52ddd`.

## 0.3.0

### Minor Changes

- The root entry is now platform-neutral and neither built artifact contains a dynamic `import()`.
  `createFileSink` moved to `@monochromatic-dev/module-logger/node`;
   `createIndexedDbSink` and `createOpfsSink` moved to `@monochromatic-dev/module-logger/browser`.
  The default logger keeps file logging under the `node` export condition and IndexedDB under `default`;
   a Node consumer whose bundler resolves `default` no longer gets file logging.
  Commit `ce38d07`.

## 0.2.0

### Minor Changes

- Records logged before every sink has answered its verify now buffer under `STARTUP_BUFFER_CAP` (10000,
   exported).
  On overflow the oldest buffered record is dropped,
   and once initialization completes one `warn` record naming the dropped count is written to every available sink.
  Commit `fee2427`.
- Sinks now verify concurrently,
   each under a time limit.
  A backend probe that never answers no longer starves the sinks after it or keeps the logger from initializing.
  The limit is the new `createLogger` option `verifyTimeoutMs` (default `DEFAULT_VERIFY_TIMEOUT_MS`,
   5000 ms).
  Commit `ee58222`.

## 0.1.0

### Minor Changes

- First npm release.
  Console output neutralizes terminal control characters.
  `flush()` runs under a deadline (`flushDeadlineMs`).
  The published package exposes the built artifact only.
  Commit `77ba394`.
