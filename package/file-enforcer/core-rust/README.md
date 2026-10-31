# monochromatic-file-enforcer

The rule engine behind the `file-enforcer` command.
It reads `file-enforcer.config.jsonc`,
plans every rule against a virtual view of the worktree,
validates the whole run,
revalidates the files it read,
and publishes generated files one by one.
A per-worktree server serializes every write over an owner-only Unix socket.
The command line lives in [`package/file-enforcer/cli`](../cli/README.md);
this crate has no command-line parsing so that
other tools (meow) can link it.
The accepted design is [`doc/decision/file-enforcer-rust-rewrite.md`](../../../doc/decision/file-enforcer-rust-rewrite.md).

## How a run works

1. The config is parsed with `monochromatic-jsonc-edit` (`parse_jsonc_with_spans`),
   converted to plain JSON (duplicate keys and unpaired surrogates
   are rejected),
   and decoded by hand into `Config`.
   Every problem is reported as `file:line:col: /json/pointer: message`,
   with the position of the offending value
   (or of the key,
   for an unknown key);
   a syntax error has no pointer and reads `file:line:col: message`.
2. `RuleRegistry::build` turns each entry into a `Rule`.
3. `plan_run` runs the rules in config order against a `View`:
   the real files plus what earlier rules planned.
   Rule failures,
   two writers of one path that do not compose,
   and a rule reading a later rule's output are all
   collected,
   and a failing run writes nothing.
4. `compute_drift` compares the plan with the real files.
   `check` reports it,
   `write` publishes it,
   `check --expected-dir` renders it as paired trees for `diff -ru`.
5. For `write`,
   every real path the plan read is re-observed;
   if any changed,
   the plan is discarded and redone
   (three tries).
   Writers that do not use the server can still race this check;
   that is documented,
   not denied.
6. `publish` replaces each file atomically:
   write a hidden sibling named `.<name>.fe-staging-<pid>` with its final mode,
   `fsync`,
   rename.
   Atomicity is per file;
   a partial run reconverges on the next run.
   Symlinks are never followed,
   and every ancestor is rechecked with `lstat` at mutation time.

Exit codes:
0 clean,
1 drift or rule failure,
2 bad config or command line.
Every line written is one JSON object:
`schemaVersion`,
`sequence`,
`type`,
`code`,
`message`,
optional `hint`,
then
extra members such as `path` and `rule`.

## The rule-kind interface

Other crates (the dependency-catalog library,
the in-repo browserslist crate,
the JetBrains XML crate) plug in by
implementing two traits and registering a builder.
The engine never names them.

```text
// package/file-enforcer/core-rust/src/rule.rs
pub trait Rule: Send + Sync {
    fn id(&self) -> &str;
    fn kind(&self) -> &str;
    fn composes(&self) -> bool { false }
    fn plan(&self, view: &mut View) -> Result<RulePlan, Diagnostic>;
}
pub trait RuleBuilder: Send + Sync {
    fn build(&self, config: &RuleConfig) -> Result<Box<dyn Rule>, Diagnostic>;
}
// Register over the placeholder; the CLI and the server take a `RuleRegistry` by value.
registry.register("browserslist", Box::new(MyBuilder));
```

- `plan` reads only through the `View`:
  `read_text`,
  `read_bytes`,
  `mode_of`,
  `list`,
  and `expand_glob` from
  `view_glob`.
  Reads of planned paths see earlier rules' output.
  Every read is recorded,
  which is what revalidation,
  `watch` and `explain` use,
  so a rule cannot hide an input and
  does not declare its inputs.
- `plan` returns `RulePlan { changes, warnings }`.
  `Change` is `Write { path, content, mode }` (a `mode` of `None` keeps an existing file's mode),
  `Remove { path }`,
  `SetMode { path, mode }` or `RemoveDirectory { path }`.
  Paths are repo-relative with `/` separators and no `.` or `..` segments.
- `composes` is `true` for a rule that transforms a file other rules also transform,
  reading it through the view and
  writing the changed text (the `toml-keys` and `rust-catalog` kinds).
  Two rules writing one path fail the run unless both compose.
  A rule that reads a path must come after every rule that writes it,
  unless it writes that path itself.
- Return `Err(Diagnostic)` for a failure that stops the run (exit 1).
  Put non-fatal notices in `warnings`;
  they are printed as `warning` events and are not drift.
- The config entry is a variant of `RuleConfig` (in `config_types.rs`) decoded and schema-checked by this crate,
  so a
  new field means editing `config_types.rs`,
  `config_decode_rules.rs`,
  the corpus in `fixtures/config-corpus.jsonc`,
  and rewriting the committed schema (`mise run //package/file-enforcer/core-rust:schema:write`).

### Kinds that other crates implement

Until a builder is registered,
the default registry installs `NotWiredBuilder`,
whose rule plans no changes and emits
a `kind-not-wired` warning naming the rule and the library to link.
That is deliberately a warning with exit code 0:
the run stays usable,
and the warning says the rule's outputs are
neither written nor checked.

- `rust-catalog` (dependency-catalog library):
  fields `catalog` (the `rust-catalog.toml` path) and `manifests` (globs).
  It must compose,
  read each `Cargo.toml` through the view after the `toml-keys` rules,
  and plan both the catalog and
  every `[workspace.dependencies]` change in one pass so a failure writes nothing.
- `browserslist` (browserslist crate):
  fields `source` (`.browserslistrc`),
  `output`
  (`.browserslistrc.resolved.local.json`) and optional `environment`.
  It owns its output outright (`composes` stays `false`) and must write the JSON array the rolldown consumer reads.
- `jetbrains-harper` (XML edit crate),
  only in the per-user config:
  fields `productPrefixes`,
  `disabledLinters`,
  `excludePatterns` and `scopedServers`.
  The view's root is the home directory in that config,
  so the rule plans paths such as `.config/JetBrains/...`.

## Built-in kinds

- `concat`:
  one file from `{"file": ...}`,
  `{"var": ...}`,
  `{"concat": [...]}` and literal parts,
  with an optional mode.
- `mode-only`:
  sets a mode on a file that exists and never creates it.
- `package-licenses`:
  copies license texts into each package's `LICENSES` directory according to the manifest's license
  expression and prunes only the texts the rule owns.
  A manifest without a license expression is not managed.
- `mirror`:
  makes a directory an exact copy of another and owns the target outright.
  Source symlinks and special files are rejected;
  target links are removed as links.
- `toml-keys`:
  guarded key assignments and append-if-absent blocks in `Cargo.toml`,
  through `toml_edit`,
  so comments,
  order and unowned keys survive and a conforming file is not rewritten.
- `mise-path-list`:
  a Mise config listing the existing non-empty directories matching the globs.

## The server

`serve` binds `<runtime dir>/<16 hex digits of sha256(worktree root)>.sock` with mode 0600 inside a 0700 directory.
The runtime directory is `$XDG_RUNTIME_DIR/file-enforcer` when that directory is owned by the user with no group or
other access,
else `<cache dir>/file-enforcer/run`,
created 0700 and verified.
A socket path over 100 bytes fails with `socket-path-too-long`.

- Election:
  a client that finds no server takes `<key>.lock` (`File::try_lock`);
  only the holder probes the socket,
  removes it when the probe is refused (verified stale),
  and spawns the server detached (`setsid`,
  null standard
  streams).
  Others wait and reconnect.
- Handshake:
  the first request is `handshake` with the protocol version,
  the build identity (version,
  size and mtime of
  the executable) and the worktree identity (canonical path,
  device,
  inode).
  A mismatch gets error `-32001` and the server drains and exits;
  the client waits for the socket to disappear and
  starts a new one.
- Requests are JSON-RPC 2.0 lines.
  `run` takes `mode` (`write`,
  `check`,
  `explain`),
  `configPath`,
  and `expectedDir` or `path`;
  events stream back as
  `event` notifications,
  then the response carries `exitCode`.
  A connection lost before the response is reported and never replayed.
  `watch` streams events until the client leaves;
  `shutdown` stops the server.
- All runs execute one at a time under one lock.
- The server exits after its idle time with no connection,
  when its worktree root disappears or is replaced,
  and after
  a handshake mismatch.
- `watch` watches every path the last run read (files,
  listed directories,
  absent paths through their nearest
  existing ancestor,
  the config),
  re-checks when something changes,
  warns with `generated-file-edited` when a drifted
  file was itself among the changed paths,
  and then writes.
  Access events are ignored,
  because the run reads files.

## Tests

```sh
mise run //package/file-enforcer/core-rust:test
mise run //package/file-enforcer/core-rust:lint:types
```

`src/schema_tests.rs` holds the golden test (the committed `file-enforcer.schema.json` equals the derived schema) and the
corpus check:
every sample in `fixtures/config-corpus.jsonc` is decoded by hand and validated by `jsonschema`,
and the
test names every disagreement.
`decoderStricter` samples list what JSON Schema cannot express,
each with its reason.
The server tests use sockets under a short directory,
never `$XDG_RUNTIME_DIR`.

## Known limits

- Columns count Unicode scalar values and only LF ends a line,
  the rule `monochromatic-jsonc-edit` documents for its spans.
- `check` of `mise-path-list` depends on installed `node_modules`,
  as the accepted design says.
- The `JsonSchema` derive and `serde_json::json!` trip the repository's `disallowed_methods` lint;
  see
  [`doc/troubleshooting/schemars-derive-trips-disallowed-methods.md`](../../../doc/troubleshooting/schemars-derive-trips-disallowed-methods.md).
