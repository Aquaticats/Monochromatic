# dprint-plugin-exec: silent no-op when only extension fallback fires; incompatibility with in-place file fixers; cacheKeyFilesHash always null in resolved per-command config

This file groups three independent dprint-plugin-exec quirks
that bit this workspace's format pipeline.
 Each gets its own
canonical section.

---

## Bug 1: `exec` plugin silently does nothing because extension fallback returns the first-registered plugin only

### Symptom

The `exec` section in a dprint config defines commands to run
after formatting.
 The commands never execute;
 no errors
appear in normal output.
 Debug logging
(`dprint fmt --log-level debug`) shows the exec plugin
instance is created but never formats any file;
 the
`"Formatted file"` log entry lacks the `(Plugin N/M)`
annotation that multi-plugin chaining produces.

### Root cause

dprint's plugin name resolution has two paths (source:
`crates/dprint/src/plugins/name_resolution.rs:54-88`):

1.  **Associations path** (lines 57-64):
     iterates all plugins
    with `associations` globs,
     collects every match into a
    vector,
     and returns the full list.
     This is the only path
    that enables multi-plugin chaining.
2.  **Extension fallback** (lines 77-84):
     when no association
    matches,
     looks up the file extension and returns the
    **first** plugin that registered that extension.
     The loop
    body returns a single-element vector immediately:

    ```rust
    return vec![plugin_name.clone()];
    ```

When multiple plugins claim the same extension (e.g. `malva`
and `exec` both claim `.css`),
 the extension fallback picks
whichever plugin was registered first (determined by the
order in the `plugins` array).
 The exec plugin always loses
because it is listed last.

### Verification

Version under test:
 dprint with the exec plugin pinned in
`mise.toml`.
 Reproduce by removing `associations` from both
plugins,
 running `dprint fmt --log-level debug some.css`,
 and
observing only the malva plugin runs (no `(Plugin N/M)`
annotation).

### Verified workaround: add dprint-level `associations` to every chaining plugin

```jsonc
// dprint config
"malva": {
  "associations": ["**/*.css", "**/*.scss", "**/*.less", "**/*.sass"],
  // ... malva options
},
"exec": {
  "associations": ["**/*.css"],
  // ... exec options
}
```

With this config,
 a `.css` file matches both `malva` and
`exec` via associations.
 `malva` runs first (listed earlier
in `plugins`),
 then `exec` chains its output.

Tradeoff:
 every plugin's full extension surface must be
duplicated into its `associations`.
 Missing an extension
breaks formatting silently (see Gotcha below).

### Gotcha: include-only `associations` exclude extension matches

When a plugin has `associations` that are **include-only**
(no negation patterns),
 the `is_not_associations_excluded`
guard (`name_resolution.rs:90-96`) rejects any file that
matches the plugin by extension but **not** by association.
Adding `"associations": ["**/*.css"]` to `malva` would
silently break formatting for `.scss`,
 `.less`,
 and `.sass`
files;
 `malva` would refuse to handle them because they
match its extensions but not its associations.

The fix is to list **all** extensions the plugin handles in
its associations:

```jsonc
"associations": ["**/*.css", "**/*.scss", "**/*.less", "**/*.sass"]
```

### Debug verification

Run `dprint fmt --log-level debug <file>` and look for the
plugin count annotation.
 Single-plugin formatting prints:

```text
Formatted file: path/to/file.css in 0ms
```

Multi-plugin chaining prints:

```text
Formatted file: path/to/file.css in 0ms (Plugin 1/2)
Formatted file: path/to/file.css in 312ms (Plugin 2/2)
```

The annotation is emitted at
`crates/dprint/src/format.rs:335-336`.

### What does not work

- Listing `exec` first in the `plugins` array without
  associations:
   changes which plugin wins the extension
  fallback,
   but exec then claims the file alone and the
  primary formatter (malva) is skipped.
- Setting `"associations": []` (empty array) on exec:
   never
  matches anything;
   exec never runs.
- Setting `"associations": null`:
   dprint validates the type
  and refuses the config.

### Why we do not file this upstream

1.  **Is it really upstream's fault?
    ** Borderline;
     the
    extension fallback's "first wins" rule is documented but
    unintuitive when chaining is the goal.
2.  **Can upstream fix it?
    ** They could warn when extension
    fallback masks chaining;
     non-trivial change.
3.  **Are they supporting this use case?
    ** Yes;
     chaining is
    the documented feature.
4.  **Will they likely fix it?
    ** No movement noticed;
     the
    workaround is documented.
5.  **Have we prototyped a minimal fix?
    ** No.

Decision:
 no upstream report.

### Why this project moved away from exec

Even with associations configured correctly,
 the exec plugin
has fundamental incompatibilities with in-place file fixers
(see Bug 2).
 The `format` task in `mise.toml` runs dprint
first,
 then stylelint and oxlint in parallel as separate
processes.

---

## Bug 2: Exec plugin incompatible with in-place file fixers like `oxlint --fix`

### Symptom

Configuring `oxlint --fix {{file_path}}` with `"stdin":
false` in exec causes:

```text
Error formatting path/to/file.ts. Message: Child process exited with code 1:
```

Even when oxlint successfully auto-fixes some issues
(visible by running `oxlint --fix` directly),
 dprint reports
the formatting as failed.

### Root cause

The exec plugin's file mode (`"stdin": false`) does **not**
mean "modify the file in-place.
" It means "don't pipe file
content to stdin.
" The result is still read from
**stdout**.

Source citations (exec plugin source 0.6.0):

- `handler.rs:162-253`:
   format flow:
  1.  Spawns the command with `stdin: Stdio::null()` and
      `stdout: Stdio::piped()`.
  2.  Captures the command's stdout as formatted content.
  3.  Waits for the child process to exit.
  4.  If exit code is 0:
       returns captured stdout as the new
      file content.
  5.  If exit code is non-zero:
       returns an error,
       discarding
      everything.
- `handler.rs:299-314`:
   exit-code check:

  ```rust
  if exit_status.success() {
      return Ok(ok_text);
  }
  Err(anyhow!("Child process exited with code {}: {}", ...))
  ```

This creates three incompatibilities with `oxlint --fix`:

1.  Output destination:
     `oxlint` writes fixes to the file
    on disk and writes diagnostic messages to stdout/stderr.
    The exec plugin captures diagnostic text from stdout and
    treats it as the "formatted" file content,
     corrupting
    the file.
2.  Exit-code semantics:
     `oxlint` exits 1 when any
    unfixable errors remain,
     even after successfully
    auto-fixing others.
     exec requires exit code 0;
     there is no
    option to accept non-zero exit codes.
3.  In-memory chaining:
     when multiple plugins chain
    (e.g. typescript formatter then exec),
     dprint passes the
    in-memory formatted result from the previous plugin to
    the next.
     In file mode,
     the command receives a
    `{{file_path}}` that still points to the **original
    on-disk content**;
     the previous plugin's output is
    invisible.

### Verification

Version under test:
 dprint-plugin-exec 0.6.0;
 oxlint 1.55.0.

Reproduce:
 configure exec with `oxlint --fix {{file_path}}`
and `"stdin": false`,
 run `dprint fmt some.ts`,
 observe the
"exited with code 1" error even when oxlint fixed the file.

### Verified workaround: run oxlint outside dprint

Skip exec for in-place fixers.
 Run them as separate steps:

```toml
# mise.toml
[tasks.format]
run = ["dprint fmt", { tasks = ["format:stylelint", "format:oxlint"] }]
```

dprint runs first,
 then stylelint and oxlint run in
parallel.
 Avoids stdout/exit-code/in-memory chaining
problems entirely.

Tradeoff:
 the `format:oxlint` task runs from the monorepo
root without `--type-aware`.
 oxlint's type-aware mode
resolves the nearest `tsconfig.json` from the working
directory,
 and the root `tsconfig.json` does not include each
package's `src` directory (see
[oxlint troubleshooting](oxlint.md#bug-1-type-aware-resolves-the-wrong-tsconfigjson-from-monorepo-root)).
Running type-aware from the root would silently miss type
information.
 A proper fix would require per-package fan-out
(like the `lint:oxlint` task template);
 not worth the
complexity in the format pipeline because type-aware
auto-fixable rules are rare and the lint task already
catches them.

### What does not work

- Setting `"stdin": false` and expecting the tool to modify
  the file in-place:
   exec always reads stdout as the result.
- Ignoring the exit code:
   there is no
  `"allowNonZeroExitCode"` option in exec plugin 0.6.0.
- Piping via stdin (`"stdin": true`):
   oxlint does not
  support reading source from stdin (no `--stdin` or
  `--stdin-filename` flags).

### Tools that work with exec

Tools that read from stdin and write the formatted result to
stdout are compatible:

- `stylelint --fix --stdin --stdin-filename {{file_path}}`
  (stdin mode)
- `prettier --stdin-filepath {{file_path}}` (stdin mode)
- `shfmt` (stdin mode,
   reads stdin by default)

Requirements:
 exit code 0 on success,
 formatted content
written to stdout.

### Why we do not file this upstream

1.  **Is it really upstream's fault?
    ** Borderline;
     the
    stdin/file-mode dichotomy is documented but does not
    cover "modify on disk".
2.  **Can upstream fix it?
    ** Yes;
     add an "in-place" mode that
    re-reads the file after the command exits.
     Non-trivial
    API change.
3.  **Are they supporting this use case?
    ** Not currently;
    only stdin-based formatters.
4.  **Will they likely fix it?
    ** Unknown;
     depends on demand.
5.  **Have we prototyped a minimal fix?
    ** No.

Decision:
 no upstream report;
 run in-place fixers outside
dprint.

---

## Bug 3: `cacheKeyFilesHash` shows `null` per command in resolved config; expected behaviour

### Symptom

Running `dprint output-resolved-config` shows
`"cacheKeyFilesHash": null` for each exec command,
 even
though the referenced files exist:

```json
{
  "executable": "pnpm",
  "args": [
    "exec",
    "stylelint",
    "--fix",
    "--stdin",
    "--stdin-filename",
    "{{file_path}}"
  ],
  "cwd": "/path/to/repo",
  "cacheKeyFilesHash": null
}
```

### Root cause

Expected behaviour.
 During config resolution
(`configuration.rs:153-156`),
 each command's hash is
extracted via `.take()` and collected into a separate
vector:

```rust
if let Some(cache_key_files_hash) = command_config.cache_key_files_hash.take() {
    cache_key_file_hashes.push(cache_key_files_hash);
}
```

`.take()` replaces the value with `None`,
 so the serialised
per-command config always shows `null`.
 The individual
hashes are combined into the **global** `cacheKey` at the
top of the exec config section:

```json
{
  "cacheKey": "a6c9dbc6...",
  "commands": [...]
}
```

### Verification

Version under test:
 dprint-plugin-exec 0.6.0.
 Run `dprint
output-resolved-config` and inspect the JSON;
 the global
`cacheKey` is non-null while every per-command
`cacheKeyFilesHash` is `null`.
 If a file cannot be read,
 a
diagnostic is emitted and config resolution returns early
(`configuration.rs:249-253`);
 the global cacheKey would be
absent in that case.

### Verified workaround

No action needed.
 To verify cache key files are being read,
check the global `cacheKey` field.
 If `cacheKey` is non-null,
the files were hashed successfully.

Tradeoff:
 the per-command `cacheKeyFilesHash: null` looks like
a bug;
 the misleading appearance is the only cost.
 Document
that this is expected;
 no behaviour change required.

### What does not work

- Trying to compute the per-command hash from the global
  cacheKey:
   the global value is a digest of all hashes
  combined,
   not reversible into per-command pieces.
- Re-reading the source files manually to verify:
   doable but
  redundant;
   the global cacheKey already encodes the result.

### Why we do not file this upstream

1.  **Is it really upstream's fault?
    ** Borderline;
     the
    `null` per-command output is correct given the
    implementation,
     but presents as confusing.
2.  **Can upstream fix it?
    ** Yes;
     preserve the hash in the
    serialised output or omit the field rather than emit
    `null`.
3.  **Are they supporting this use case?
    ** The `cacheKey` at
    the top is the supported answer.
4.  **Will they likely fix it?
    ** Unknown.
5.  **Have we prototyped a minimal fix?
    ** No.

Decision:
 no upstream report.
 The misleading-but-correct
output is documented here.
