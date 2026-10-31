# file-enforcer command line

The `file-enforcer` binary over [`monochromatic-file-enforcer`](../core-rust/README.md).
It generates and checks the files this repository derives from other files,
as described in
[`doc/decision/file-enforcer-rust-rewrite.md`](../../../doc/decision/file-enforcer-rust-rewrite.md).

## Commands

```sh
file-enforcer write                         # regenerate every drifted file
file-enforcer check                         # report drift, exit 1 when anything differs
file-enforcer check --expected-dir /tmp/d   # also write /tmp/d/actual and /tmp/d/expected for diff tools
file-enforcer watch                         # keep files current, warn on hand edits
file-enforcer explain CLAUDE.md             # which rules read or write this path
file-enforcer user write                    # rules that write outside the repository
file-enforcer user check
```

- `--config <path>` names the repository config;
  without it the nearest `file-enforcer.config.jsonc` above the current
  directory is used,
  and its directory is the worktree root.
- `--user-config <path>` names the per-user config;
  the default is
  `<config dir>/monochromatic-dev/file-enforcer/user.config.jsonc`.
- Unknown flags are rejected.

## Rule kinds linked here

The engine (`monochromatic-file-enforcer`) does not link the libraries behind three rule kinds;
this binary does,
through `production_registry` in `src/kinds.rs`,
which the server uses.

- `rust-catalog`:
  `monochromatic-dependency-catalog`.
  Reads `rust-catalog.toml` and every manifest the globs match through the view,
  so it sees what the `toml-keys` rules planned,
  and plans the catalog and every `[workspace.dependencies]` table in one pass.
  It composes,
  and belongs after the `toml-keys` rules.
- `browserslist`:
  `monochromatic-browserslist`.
  Reads the query file and the data `lock` (a repository-root `browserslist-data.lock.json`) and owns its output.
  The pinned data comes from the user cache,
  and a missing pinned tarball is fetched once from the npm registry and verified against the lock;
  nothing unpinned is ever fetched.
  The resolver implements no config sections,
  so `environment` selects nothing:
  a query file without sections resolves its default queries,
  and a `[section]` line is refused as `not implemented`.
- `jetbrains-harper` (per-user config only):
  `monochromatic-xml-edit` and `serde_json`.
  The view root is the home directory;
  the rule reads `<config>/JetBrains/<product>/options` of the newest product matching `productPrefixes`,
  where `<config>` is `XDG_CONFIG_HOME` (read when the binary starts) if it is an absolute path below the home directory,
  and `.config` under the home directory if it is unset,
  empty or relative
  (the XDG Base Directory rule;
  the retired TypeScript tool used any set value as it stood).
  An absolute value outside the home directory is a `jetbrains-harper-skipped` warning naming it,
  because the view cannot read outside its root.
  It edits `option` values attribute by attribute,
  so comments inside managed entries survive,
  and it skips with a `jetbrains-harper-skipped` warning when the IDE,
  its settings files or the base Harper server are absent.
  The base server match,
  the options copied to scoped servers,
  the schema default and the `harper-ls.excludePatterns` key are fixed in the rule,
  carried over from the retired TypeScript policy.

## Output

Standard output is JSON lines only,
including clap's help,
version and usage errors (event types `help`,
`version`,
`error` with code `usage-error`).
Each line has `schemaVersion`,
`sequence` (per process),
`type`,
`code`,
`message`,
and optionally `hint`,
`path`
and `rule`.
Exit codes:
0 clean,
1 drift or rule failure,
2 bad command line or config.

## The server

Each command is a client of a server that starts on demand,
one per worktree (and one for the per-user config),
and exits
when idle.
`FILE_ENFORCER_IDLE_SECONDS` sets the idle time of servers a command starts (default 120);
containers and CI set it low for
deterministic shutdown.
The socket lives under `$XDG_RUNTIME_DIR/file-enforcer`,
or `<cache dir>/file-enforcer/run` when that variable is unset
or unusable.
The hidden `serve` command is what the first client runs,
detached.

## Tests

```sh
mise run //package/file-enforcer/cli:test
```

The tests run the built binary in scratch worktrees with their own runtime directories,
and a guard stops any server a
test leaves.

## Closure lint suppression

The unified linter runs `rust/avoidable-closure` over this crate,
and the rule accepts no comment directive and no per-line configuration.
One module carries the rustc `expect` attribute the rule accepts:

- `src/args.rs`:
  the inner `#![cfg_attr(dylint_lib = "monochromatic_lint_closure", expect(avoidable_closure, reason = "..."))]`
  covers the six closures that clap's derive generates in `FromArgMatches` for the required fields.
  Their spans are the field types the author wrote
  (`String`,
  `u64`,
  `PathBuf`,
  `bool`),
  so no closure exists in the source to refactor,
  and the derive output cannot be edited.
  Configuration cannot replace the attribute:
  the only lever is a path exemption,
  which would also exempt the author-written code of the file.
  The linter's own `src/cli_options.rs` carries the same attribute for the same reason
  (`package/linter/monochromatic-lint/README.md`,
  section "Suppression record").
  A lint that skipped closures inside items marked `#[automatically_derived]` would make the attribute unnecessary.
