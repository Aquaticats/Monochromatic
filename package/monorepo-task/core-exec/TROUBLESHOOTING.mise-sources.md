# Mise `sources`/`outputs` staleness checking

Mise supports `sources` and `outputs` on tasks to skip re-execution when inputs
haven't changed.
 This works transitively through `depends`:
 a dependent task
with `sources`/`outputs` will be skipped when up-to-date.

## How it works (mise v2026.2+)

Source:
 [`src/task/task_source_checker.rs`][checker] in the mise repo.

[checker]: https://github.com/jdx/mise/blob/main/src/task/task_source_checker.rs

`sources_are_fresh` runs two checks in sequence:

1. **Metadata hash** (default) or **content hash** (opt-in via
   `task.source_freshness_hash_contents = true`).
   The default hashes `(path, file_size)` pairs.
   If the stored hash differs from the current hash,
    the task is stale
   ([line 149][L149]).

2. **Mtime comparison**:
    `max(source mtimes)` vs `max(output mtimes)`.
   If sources are newer,
    the task is stale ([lines 166--175][L166]).

[L149]: https://github.com/jdx/mise/blob/main/src/task/task_source_checker.rs#L149
[L166]: https://github.com/jdx/mise/blob/main/src/task/task_source_checker.rs#L166

After a successful run,
 `save_checksum` ([line 184][L184]) touches auto-output
files or validates explicit outputs exist.

[L184]: https://github.com/jdx/mise/blob/main/src/task/task_source_checker.rs#L184

## Critical glob pitfall: `**` vs `**/*`

Mise uses the Rust [`glob`][glob-crate] crate.
 In this crate,
 a trailing `**`
matches **directories** but not files directly inside the parent:

[glob-crate]: https://docs.rs/glob/latest/glob/

```toml
# BROKEN -- does not match src/index.ts, only matches src/ directory itself
sources = ["src/**"]

# CORRECT -- matches all files at any depth under src/
sources = ["src/**/*"]
```

When `src/**` is used,
 the glob resolves to zero files.
 Mise then falls back to
hashing only the `mise.toml` config file (which it always appends to the source
list on [line 114][L114]).
 Since `mise.toml` doesn't change between runs,
 the
hash always matches,
 and the auto-output touch file's mtime is always newer,
so the task is permanently "up-to-date" regardless of actual source changes.

[L114]: https://github.com/jdx/mise/blob/main/src/task/task_source_checker.rs#L114

**Always use `**/*` in mise `sources` and `outputs` glob patterns.
**

## Auto-outputs when `outputs` is omitted

When `sources` is defined but `outputs` is empty,
 mise automatically sets
`outputs` to `TaskOutputs::Auto` ([`mod.rs` line 996--997][auto]).
 This creates
a synthetic touch file in `~/.local/state/mise/task-auto-outputs/` that gets
touched after each successful run.
 The staleness check then compares source
mtimes against this touch file.

[auto]: https://github.com/jdx/mise/blob/main/src/task/mod.rs#L996

## Same-size file edits

The default metadata hash only covers `(path, file_size)`.
 If a source file is
edited without changing its byte count,
 the hash stays the same.
 The mtime
comparison (check 2) should still catch this,
 but only if the glob patterns
correctly match the files.
 With broken `**` globs,
 neither check works.

To handle same-size edits reliably,
 enable content hashing:

```toml
# In mise.toml [settings] or ~/.config/mise/config.toml
[settings.task]
source_freshness_hash_contents = true
```

This uses blake3 to hash file contents instead of metadata,
 at a small
performance cost.

## Example: correct usage with `depends`

```toml
[tasks.build]
depends = ["expensive-prep"]
run = "node src/build.ts"

[tasks.expensive-prep]
sources = ["../sibling-pkg/src/**/*"]
outputs = ["../sibling-pkg/dist/**/*"]
run = "mise run //package/sibling-pkg:build"
```

On the first run,
 both tasks execute.
 On subsequent runs,
 `expensive-prep` is
skipped when its sources haven't changed (hash matches and outputs are newer).
`build` always runs since it has no `sources` of its own.
