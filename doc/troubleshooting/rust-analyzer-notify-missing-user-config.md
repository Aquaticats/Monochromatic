# rust-analyzer `1.100.0-nightly (1303417)` with server-side file watching warns `notify error: No path was found.` at start when the user configuration file does not exist

With `files.watcher = "server"`,
rust-analyzer asks its file watcher to watch `rust-analyzer/rust-analyzer.toml` in the user's configuration directory.
When that file does not exist the watch fails,
and rust-analyzer prints a warning on standard error each time it loads its file configuration.
The warning names no path.
The IDE's Language module inherits the server-side setting from Helix,
and `helix-lsp` logs every standard-error line at ERROR
([`helix-lsp-transport-error-level-records.md`](helix-lsp-transport-error-level-records.md)),
so the application log shows the warning as an error.

## Symptom

On rust-analyzer's standard error, two times during every start:

```text
2026-10-05T14:50:17.468643919-04:00  WARN notify error: No path was found.
```

In the application log:

```text
ERROR helix_lsp::transport: rust-analyzer err <- "2026-10-05T14:50:17.468643919-04:00  WARN notify error: No path was found.\n"
```

Conditions:

- `files.watcher` is `"server"`.
  Helix ships that value for rust-analyzer (`languages.toml:252` at `ba40e547426b0f9896c8bdc699a4ab11f2b37dbc`),
  and the IDE uses Helix's definition.
- `rust-analyzer/rust-analyzer.toml` does not exist below `$XDG_CONFIG_HOME`,
  or below `~/.config` when that variable is unset.

Analysis is not affected: hover, definitions, and diagnostics work.

## Root cause

Source: `src/tools/rust-analyzer` in `rust-lang/rust` at `1303417c416e1595173d9689e7394c31e136ae95`,
the commit the installed binary reports,
and `notify` 8.2.0, the version in that tree's `Cargo.lock`.

The list of folders to load always gets an entry for the user configuration file,
and that entry is always marked as watched (`crates/load-cargo/src/lib.rs:366`):

```rust
// crates/load-cargo/src/lib.rs:366
if let Some(user_config_path) = user_config_dir_path {
    let ratoml_path = {
        let mut p = user_config_path.to_path_buf();
        p.push("rust-analyzer.toml");
        p
    };

    let file_set_roots = vec![VfsPath::from(ratoml_path.to_owned())];
    let entry = vfs::loader::Entry::Files(vec![ratoml_path]);

    res.watch.push(res.load.len());
    res.load.push(entry);
```

The directory comes from the platform's configuration directory (`crates/rust-analyzer/src/config.rs:1184`):

```rust
// crates/rust-analyzer/src/config.rs:1184
pub fn user_config_dir_path() -> Option<AbsPathBuf> {
    let user_config_path = if let Some(path) = env::var_os("__TEST_RA_USER_CONFIG_DIR") {
        std::path::PathBuf::from(path)
    } else {
        dirs::config_dir()?.join("rust-analyzer")
    };
    Some(AbsPathBuf::assert_utf8(user_config_path))
}
```

With server-side watching the watcher is asked to watch every file of a watched entry,
whether it exists or not (`crates/vfs-notify/src/lib.rs:287`),
and a failed watch is logged as a warning (`crates/vfs-notify/src/lib.rs:352` and `:368`):

```rust
// crates/vfs-notify/src/lib.rs:287
loader::Entry::Files(files) => files
    .into_iter()
    .map(|file| {
        if do_watch {
            watch(file.as_ref());
        }
```

```rust
// crates/vfs-notify/src/lib.rs:352
fn watch(&mut self, path: &Path) {
    if let Some((watcher, _)) = &mut self.watcher {
        log_notify_error(watcher.watch(path, RecursiveMode::Recursive));
    }
}
```

```rust
// crates/vfs-notify/src/lib.rs:368
fn log_notify_error<T>(res: notify::Result<T>) -> Option<T> {
    res.map_err(|err| tracing::warn!("notify error: {}", err)).ok()
}
```

`notify` inspects the path before it adds a recursive watch (`notify/src/inotify.rs:403`),
and turns "not found" into an error that carries no path (`notify/src/error.rs:85`):

```rust
// notify/src/inotify.rs:403
if !is_recursive || !metadata(&path).map_err(Error::io_watch)?.is_dir() {
    return self.add_single_watch(path, false, true);
}
```

```rust
// notify/src/error.rs:85
pub fn io_watch(err: io::Error) -> Self {
    if err.kind() == io::ErrorKind::NotFound {
        Self::path_not_found()
    } else {
        Self::io(err)
    }
}
```

The message appends the paths only when the error has some (`notify/src/error.rs:110`),
which is why the warning ends after `No path was found.`:

```rust
// notify/src/error.rs:110
let error = match self.kind {
    ErrorKind::PathNotFound => "No path was found.".into(),
```

The warning appears two times
because rust-analyzer sends its file configuration to the watcher two times during a start,
and each configuration holds the same entry.
With `RA_LOG=vfs_notify=debug` the server's log shows both:

```text
vfs-notify event event=Message(Config(Config { version: 1, load: [...
  Files([AbsPathBuf("/home/user/.config/rust-analyzer/rust-analyzer.toml")]) ... watch: [14, 15]
WARN notify error: No path was found.
vfs-notify event event=Message(Config(Config { version: 2, load: [...
  Files([AbsPathBuf("/home/user/.config/rust-analyzer/rust-analyzer.toml")]) ... watch: [14, 15]
WARN notify error: No path was found.
```

### A reading that was wrong

The first suspect was the IDE's bubblewrap confinement:
a directory the sandbox hides or a cargo redirect that points at a path not yet created.
The evidence against it:
the same project opened without confinement produces the same two warnings,
and a configuration directory that contains the file removes both warnings with and without confinement.

## Verification

Versions:
`rust-analyzer 1.100.0-nightly (1303417 2026-09-21)` from the rustup toolchain `nightly-2026-09-22`;
`helix-lsp` at `ba40e547426b0f9896c8bdc699a4ab11f2b37dbc`.

Harness: the IDE's real start path with a plan file.
`extra_languages` adds variables to the server's environment or settings to its configuration.

```sh
# package/ide/app, after any inspect:language task built target/debug/ide-language-inspect
target/debug/ide-language-inspect plan.json 2> log.txt
rg --count 'notify error' log.txt
```

```json
{
  "project": "/absolute/path/to/a/disposable/cargo/project",
  "extra_languages": "[language-server.rust-analyzer]\nenvironment = { XDG_CONFIG_HOME = \"/absolute/path/to/a/disposable/config\" }\n",
  "steps": [
    { "do": "open", "file": "src/main.rs" },
    { "do": "ready", "seconds": 120 },
    { "do": "sleep", "milliseconds": 4000 },
    { "do": "close" }
  ]
}
```

Measured on 2026-10-05 on a disposable project with a committed `Cargo.lock`.

### Variants without the warning

- Confined, `XDG_CONFIG_HOME` names a directory that contains an empty `rust-analyzer/rust-analyzer.toml`: none.
- Unconfined (`"unconfined": true`), same directory: none.
- Confined, `[language-server.rust-analyzer.config.files]` with `watcher = "client"`: none;
  rust-analyzer then registers its watch patterns with the client (`client/registerCapability`, two times).

### Variants with the warning

- Confined, default environment: two.
- Unconfined, default environment: two.
- Confined, `XDG_CONFIG_HOME` names an empty directory: two.
- Confined, `RA_LOG = "vfs_notify=debug"`: two, each after a `Message(Config(...))` that lists the file.

## Verified workarounds

The server is unchanged, and so is its warning.
Since 2026-10-06 the application logs the two lines at INFO instead of ERROR, with their text kept,
as every line a server writes to standard error
([`helix-lsp-transport-error-level-records.md`](helix-lsp-transport-error-level-records.md), "Verified workarounds");
the repository owner kept server-side file watching.
No other handling is applied:

- **A configuration directory that contains the file.**
  Pointing the server's `XDG_CONFIG_HOME` at a private directory
  with an empty `rust-analyzer/rust-analyzer.toml` removes the warning.
  Tradeoff: the server no longer reads the user's real rust-analyzer configuration,
  so settings the user keeps there stop applying inside the IDE.
- **Client-side watching.**
  `files.watcher = "client"` removes rust-analyzer's own watcher and with it the warning.
  Tradeoff: rust-analyzer then learns about changed files only from the client.
  The Language module reports only reloads of the displayed file
  (`reload` in `package/ide/app/src/language/lifecycle.rs`),
  so edits to other project files made outside the IDE would no longer reach rust-analyzer
  until the application forwards project-wide file events.

## What does not work

- **Creating the file for the user.**
  The path is in the user's real configuration directory.
  The application is a reader and must not write there;
  inside the sandbox that directory is read-only anyway.
- **Changing the confinement recipe.**
  The warning does not depend on confinement (see "A reading that was wrong").

## Upstream filing decision

`.out-of-scope/` was checked: no entry covers rust-analyzer.

Tracker searches on 2026-10-05 in `rust-lang/rust-analyzer`:
issues for `"No path was found"` and `notify error`,
pull requests for `user config rust-analyzer.toml watch`.
`notify error` returned [#19560][ra-19560], open since 2025-04-10, opened by a maintainer:
`WARN notify error: Input watch path is neither a file nor a directory.` in the logs on startup,
with the remark that something is configured wrong, and no diagnosis in the thread.
That text is `notify`'s message for the same failed watch on Windows (`notify/src/windows.rs:171`),
so the report describes this behavior on another platform.
The watched entry and the `watch` function are unchanged on `master`
(`ce37b245369ade4e2a18063ae4d60a17606c6d26`, compared on 2026-10-05).

1.  Is it really upstream's fault?
    Yes. The server watches a path it has no reason to expect, and the maintainer's report says so.
2.  Can upstream fix it?
    Yes, for example by watching the file only when it exists, or by watching its directory.
3.  Are they supporting this use case?
    Yes. Server-side watching is a documented value of `files.watcher`.
4.  Would the repo welcome our contribution?
    Not in the form this session can produce.
    `AI_POLICY.md` on `master` allows AI as a tool with disclosure,
    but requires comments to maintainers and issue descriptions to be written by humans in their own words,
    and forbids issues and pull requests opened by autonomous agents.
    A drafted comment would break that rule.
5.  Will they likely fix it?
    Plausible: a maintainer opened the report; nothing has happened in the thread since.
6.  Have we prototyped a minimal fix?
    No. Constraint 4 fails, so no prototype was built.

Decision: do not file, and post no comment.
What the existing thread lacks is the cause:
the user-configuration entry in `ProjectFolders::new` is watched whether or not the file exists.
A human who wants to add that to [#19560][ra-19560] has to write it in their own words;
the trace in "Root cause" is the material for it.

~~~md
(do not post as-is: rust-analyzer's AI policy requires comments written by a human)

Facts a human comment on #19560 could state, each verified in "Root cause" and "Verification":
- The failing watch is `<config dir>/rust-analyzer/rust-analyzer.toml`, pushed to `watch` unconditionally in
  `ProjectFolders::new` (crates/load-cargo/src/lib.rs).
- It only shows with `files.watcher = "server"`; on Linux with notify 8.2.0 the text is `No path was found.`.
- Creating the file makes the warning disappear; it is printed once per file configuration sent to the watcher.
~~~

[ra-19560]: https://github.com/rust-lang/rust-analyzer/issues/19560

## Evidence

Measured on 2026-10-05 in the worktree `.claude/worktrees/ide-language-verify`.

- Variant runs with their full logs: `~/temp/agent/ide-language-verify-notify-3TUp6k/`
  (`results.json`, one `*.log.txt` per variant).
- Client-side watching: `~/temp/agent/ide-language-verify-watcher-rOVM1n/`.
- Source files at the cited commit: `~/temp/agent/ide-language-verify-r1/ra-1303417/`;
  `notify` 8.2.0: `~/temp/agent/notify-20261005/`.
