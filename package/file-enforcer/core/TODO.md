# file-enforcer TODO

## Read cache improvements

- Cache currently assumes files are only modified by file-enforcer or a single external edit.
  If multiple external tools modify the same source file between re-runs,
   only the last version is seen.
- No cache size limit;
   for extremely large monorepos,
   memory usage could grow indefinitely.
  Consider an LRU eviction strategy if this becomes a problem.
- Glob expansion results (`expandGlob`) are not cached.
  Directory listings are re-scanned on every run even when no files were added or removed.
  Caching glob results with directory-level mtime invalidation would eliminate redundant scans.

## Watch mode reliability

- `fs.watch` is platform-dependent and can miss events on some Linux filesystems (notably NFS,
   FUSE mounts).
  Current watch-mode setup logs watcher failures,
   retries each watcher a bounded number of times,
  and fails watch mode closed when retries are exhausted.
  Consider a fallback to polling for unreliable backends.
- Debounce currently uses a fixed 100ms debounce period.
  Rapid burst edits (e.g.,
   `git checkout` touching many files) can still trigger multiple batched re-runs
  when the burst spans more than one debounce window.
- The watch loop blocks on a fail-closed promise.
  No graceful shutdown on SIGINT/SIGTERM;
   open file watchers and AbortControllers are only cleaned up
  during reruns or watcher failure paths.
  Add signal handling for clean shutdown.

## Write-protection notifications

- No notification deduplication;
   rapid external edits to the same file produce multiple notifications.

## Config execution model

- The entire config re-runs on any source change.
  For configs with many independent rules,
   this is wasteful when only one rule's sources changed.
  A future optimization:
   tag rules with their source paths and only re-run affected rules.
  The current in-memory cache makes full re-runs fast (~2ms warm) in a lazy-builder
   fixture. The root config has no lazy builder and records no manifest entries at
   all, so warm and cold runs cost the same there; see
   `doc/troubleshooting/file-enforcer-staleness.md`.
   Measure the real config before calling this low priority.
- No dry-run mode.
  Adding `--dry-run` would require switching to a descriptor pattern or wrapping write functions with a no-op.
  The current direct execution model was chosen explicitly to avoid this complexity;
   add only if there is a real use case.

## CLI

- No `--verbose` / `--quiet` flags to control log output.
- No `--config <path>` flag;
   config path is positional-only or found by walking up from cwd.
- No support for multiple config files (e.g.,
   per-package configs merged at the root).

## API gaps

- No `appendTo(dest, content)` operation for adding content without overwriting.
- No `prependTo(dest, content)` operation.
- No conditional logic built into the API (e.g.,
   "only overwrite if source is newer").
  Users can implement this in their config with plain TypeScript,
   which is the intended design.
- `exec()` does not support stdin,
   environment variables,
   or working directory configuration.
  Wrap with `Bun.spawn` directly for advanced use cases.
- Nested `PlatformCommands` literals require `as const` to satisfy the recursive `Command` type.
  Extract nested dispatch tables into typed constants for readability.
