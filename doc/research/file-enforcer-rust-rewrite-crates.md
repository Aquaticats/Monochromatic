# Rust crate research for the file-enforcer rewrite (clap + config with jsonc)

Research for [`doc/planning/file-enforcer-rust-rewrite.md`](../planning/file-enforcer-rust-rewrite.md),
gathered 2026-10-29 by a read-only research agent;
file reads and source citations only,
nothing was executed.

Date of research: 2026-10-29. Method: crates.io API (`https://crates.io/api/v1/crates/<name>`), shallow clones of each
repo (under `/var/home/user/temp/agent/<name>-2026-10-29`, HEAD of the default branch that day), `gh api` for repo
activity, WebFetch for editor docs. Nothing was compiled or run: a `cargo run` probe of config-rs was refused by the
permission classifier, so every behavior claim below is from reading source or docs, not from execution. "Downloads" are
crates.io all-time; "recent" is the crates.io `recent_downloads` field (about 90 days).

## 1. clap

- Current: clap 4.6.7, published 2026-09-14, MSRV 1.85, MIT OR Apache-2.0, 1.28 billion downloads, 270 million recent.
  https://crates.io/api/v1/crates/clap ; changelog entry https://github.com/clap-rs/clap/blob/master/CHANGELOG.md
- Repo active: last push 2026-10-06, 452 open issues (issues plus PRs per GitHub's counter).
  https://api.github.com/repos/clap-rs/clap
- A 5.0.0 is listed as "TBD", reachable only through the `unstable-v5` feature (CHANGELOG.md lines 7 to 22). 4.x is the
  stable line.
- Derive vs builder: both are first class. Derive (`Parser`, `Args`, `Subcommand`, `ValueEnum`) needs the `derive`
  feature (off by default). Any `Command`/`Arg` builder method can be used as a raw attribute in derive, so the two do
  not diverge in power. Mixing is documented. https://github.com/clap-rs/clap/blob/master/src/_derive/mod.rs (lines 1 to
  107: "Raw attributes are forwarded directly to the underlying clap builder", `global = true` example at line 107).
- Subcommands (`check`, `write`, `watch`): `#[derive(Subcommand)]` on an enum; builder knobs `subcommand_required`
  (command.rs:3142), `arg_required_else_help` (command.rs:2411), `propagate_version` (command.rs:1527), and
  `Arg::global` (arg.rs:917) for options shared by every subcommand. All in
  https://github.com/clap-rs/clap/blob/master/clap_builder/src/builder/command.rs and `.../arg.rs`.
- Default features: `std color help usage error-context suggestions`. Optional: `derive`, `env`, `cargo`, `wrap_help`,
  `unicode`, `string`, `deprecated`. https://github.com/clap-rs/clap/blob/master/Cargo.toml (`[features]`).
- Env-var fallback: `Arg::env` exists only with the `env` feature (arg.rs:2202 `#[cfg(feature = "env")]`,
  `pub fn env` at 2205). In derive: `#[arg(env = "NAME")]`, same feature needed.
- Value hints: `ValueHint` enum (AnyPath, FilePath, DirPath, ExecutablePath, CommandName, CommandString,
  CommandWithArguments, Username, Hostname, Url, EmailAddress, Other, Unknown), set by `Arg::value_hint`
  (arg.rs:1392; in derive it is a raw attribute). Consumed by the generated completion scripts
  (clap_complete/src/aot/shells/bash.rs:179, zsh.rs:406, fish.rs:307).
- Shell completion: clap_complete 4.6.11 (2026-09-15), 133 million downloads. Ahead-of-time generation via
  `clap_complete::generate` / `generate_to` (aot/generator/mod.rs:229, 284) for Bash, Elvish, Fish, PowerShell, Zsh
  (aot/shells/shell.rs:15 to 26). Nushell is a separate crate, clap_complete_nushell 4.6.2 (2026-08-11). Dynamic
  completion (`CompleteEnv`, `COMPLETE=$SHELL <bin>`) is behind `unstable-dynamic`, default features are empty
  (clap_complete/Cargo.toml `[features]`; clap_complete/src/env/mod.rs:1 to 11, 81 to 98). So dynamic completion is
  unstable API.
- Man pages: clap_mangen 0.3.3 (2026-08-12), 18 million downloads, 0.x. Renders ROFF from a `clap::Command`
  (`Man::new(cmd).render(&mut buf)`); its README recommends running it from `build.rs` or an xtask.
  https://github.com/clap-rs/clap/blob/master/clap_mangen/README.md
- Derive interaction with a config file: clap has no config-file layer. Precedence has to be hand-written (CLI over env
  over file) or delegated to a layering crate.

## 2. config-rs (`config` crate)

Facts first, judgement in section 7.

- Version: 0.15.27, published 2026-09-30, MSRV 1.88.0, MIT OR Apache-2.0, 125 million downloads, 21 million recent, 59
  releases, still pre-1.0. https://crates.io/api/v1/crates/config ;
  https://github.com/rust-cli/config-rs/blob/main/Cargo.toml (lines 100 to 140 for package and features)
- Maintenance: not archived; last push 2026-10-13; 110 open issues and 28 open PRs; at least 100 commits since 2026-01-01 (the API page was capped at 100)
  (renovate/dependency bumps included). Top contributors: epage (509 commits), matthiasbeyer (353), mehcode (189).
  Latest commits are `chore(deps)` and `perf(toml)` by Ed Page (2026-09-21 to 2026-10-06). Release cadence in
  CHANGELOG.md: 0.15.21 (2026-03-12) through 0.15.27 (2026-09-30), mostly small fixes. So: maintained in a low-intensity,
  fix-and-bump mode; the project lives under the `rust-cli` org. I found no maintainer statement about its future; the
  repo has an `AI_POLICY.md` and nothing about deprecation. Sources: `gh api repos/rust-cli/config-rs`,
  `gh api repos/rust-cli/config-rs/commits`, https://github.com/rust-cli/config-rs/blob/main/CHANGELOG.md
- Issue #371 "Differences between config-rs and figment?" (open) is a user comparison, not a maintainer statement.
  https://github.com/rust-cli/config-rs/issues/371
- Read-only: README says "this library can not be used to write changed configuration values back to the
  configuration file(s)". https://github.com/rust-cli/config-rs/blob/main/README.md (line 27 to 28)
- Formats and feature flags (Cargo.toml `[features]`): default = `toml json yaml ini ron json5 convert-case async`;
  also `corn`, `preserve_order`. The `FileFormat` enum is `Toml, Json, Yaml, Ini, Ron, Json5, Corn`
  (https://github.com/rust-cli/config-rs/blob/main/src/file/format/mod.rs lines 30 to 60).
- JSONC support: no native `Jsonc` variant. `Json` is plain `serde_json::from_str` (format/json.rs:12), which rejects
  comments and trailing commas. `Json5` goes through the `json5` crate (format/json5.rs:39), which accepts comments,
  trailing commas, unquoted keys; but its file extension mapping is only `json5` (format/mod.rs:99 to 100). So a
  `.jsonc` file loaded with `File::with_name("x")` is not auto-detected; use `File::from_str(text, FileFormat::Json5)`
  or `File::new(path, FileFormat::Json5)`, which parse with JSON5 rules (a superset of JSONC, so JSONC parses, but
  JSON5-only syntax is also silently accepted). Custom formats are supported through the `Format` trait
  (src/format.rs:16 to 25: `fn parse(&self, uri: Option<&String>, text: &str) -> Result<Map<String, Value>, Box<dyn
  Error + Send + Sync>>`), example in `examples/custom_file_format/main.rs`. So a strict JSONC `Format` backed by
  jsonc-parser is possible, but the parse result is flattened into config's own `Value` tree, which drops all
  positions.
- Layering: `ConfigBuilder` has defaults (`set_default`), sources added in order via `add_source` (later wins), and
  explicit overrides (`set_override`, highest priority). `Environment::with_prefix(..).separator(..)` is the env
  source. https://github.com/rust-cli/config-rs/blob/main/src/builder.rs (lines 12 to 23, 132 to 180).
- Error reporting: `ConfigError::FileParse { uri, cause }` carries the file name and the underlying parser's error
  boxed; `ConfigError::Type { origin, unexpected, expected, key }` carries origin (file or "Environment") and key path
  but no byte offset, line, or column (src/error.rs:43 to 102). Line/column appear only if the underlying parser puts
  them in its message (json5 does, see section 3). `ConfigError::Message` after a custom serde error loses key and
  origin: open issue #532 https://github.com/rust-cli/config-rs/issues/532 . The source itself carries
  `// TODO: Why is this called Origin but FileParse has a uri field?` (error.rs:68).
- Key case: `Environment` lowercases env keys and the prefix (src/env.rs:255, 269). Open issues: #340 "Environment
  variable names are converted to lower case", #577 "Use a HashMap for a config entry without changing the case"
  (reports that a JSON5 file's map keys come back lowercased). At HEAD the file parsers insert keys verbatim
  (format/json.rs:38, format/json5.rs:53) and `rg to_lowercase src` hits only env.rs:255, 269, 308 and value.rs
  coercion lines, so by source reading the folding is in `Environment`, and #577 concerns an older version or another
  path (not executed to confirm; `tests/testsuite/file_yaml.rs:208` has a comment "ensure that the map has only
  lowercase keys", and that test accepts either outcome). Also #625 "No option to exclude keys from case conversion", #672. List at
  `gh issue list --repo rust-cli/config-rs`.
- Other pain points seen in the open list: #568 (an unquoted `INF` string read from YAML deserializes wrongly), #596
  (env hierarchy docs ambiguous), #64 (command-line configuration, i.e. no CLI integration).
  Values are "loosely typed" (README line 17): strings convert to bools and numbers (value.rs lowercases and parses),
  which is the opposite of a strict schema check.
- Does it report spans? No. Nothing in `ConfigError` carries a span; the `Format` trait returns a boxed error with no
  position type.

## 3. JSONC and JSON5 parsing crates

All data from the crates.io API unless noted.

- jsonc-parser (dprint): 0.34.0, 2026-09-27, 12.1 million downloads (3.3 million recent), MIT, 52 releases, repo
  `dprint/jsonc-parser`, 1 open issue, last push 2026-09-27, commits through 2026-09 (JSON5 numbers/escapes, CST sort
  API). https://github.com/dprint/jsonc-parser
  - Syntax: all of comments, trailing commas, missing commas, single quotes, hex numbers, unquoted names, NaN/Infinity,
    JSON5 escapes are on by default (`ParseOptions` defaults all true, src/parse_to_ast.rs:51 to 89), each switchable;
    set all false for strict JSON.
  - Serde: feature `serde`, `parse_to_serde_value::<T>(text, &opts) -> Result<T, ParseError>` (src/serde.rs:61).
  - Error spans: syntax errors carry `range()`, 1-indexed `line_display()` and `column_display()` (src/errors.rs:156 to
    171). Serde type errors do NOT: `impl serde::de::Error for ParseError` builds `custom_err`, documented as "no
    position info" (errors.rs:179; serde.rs `fn custom`). So a wrong-typed field gets a message but no location.
    Workaround: `parse_to_ast` gives a ranged AST, and `serde_path_to_error` gives the path; join the two.
  - Lossless edits: yes, with feature `cst` (off by default). `CstRootNode::parse`, `object_value_or_set`, `get`,
    `set_value`, `append`, `remove`, `replace_with`, `set_trailing_commas`, `indent_text`; the doc example keeps a
    comment while replacing a value (src/lib.rs:36 to 68; cst/mod.rs:1111 to 1340). CST nodes are `Rc`-based (cst/mod.rs
    :1107, 1367), so the tree is not `Send`.
  - Maintenance: strongest of the group.
- json5 (callum-oakley/json5-rs): 1.3.6, 2026-10-28, 87 million downloads (17 million recent), MIT, 25 releases. 8 open
  issues, last push 2026-10-28 (perf work). Serde Deserialize and Serialize. Errors have `position()` giving 0-indexed
  line and column (src/error.rs:7 to 30, 205 to 211), and the deserializer attaches the position to visitor errors too
  (src/de.rs:525 to 526, many `with_position` call sites), so type errors can have a location. No CST, no comment
  preservation. Used by config-rs. https://github.com/callum-oakley/json5-rs
- serde_json_lenient (Google): 0.2.4, 2024-12-10, 2.9 million downloads (1.65 million recent). Fork of serde_json with
  switchable comments and trailing commas (`set_allow_comments`, `set_ignore_trailing_commas`, de.rs:257, 262). Errors
  expose `line()` and `column()` like serde_json (error.rs:32, 44), and the deserializer calls `fix_position` on
  errors (de.rs:416, 441, 484, 514, 541), so data errors get positions too. Last push 2024-12-28, 1 open issue; no
  commit since that date. README says unquoted keys are "[planned]". No round trip.
  https://github.com/google/serde_json_lenient
- serde_jsonc: 1.0.108, 2023-10-30, 122 thousand downloads. Its crates.io readme is a verbatim copy of serde_json's and
  I found no statement of what it adds; I could not confirm comment support from its published docs. No release since 2023-10-30.
  Treat as unverified and skip.
- json_comments (tmccombs): 0.2.2, 2023-11-03, 5.3 million downloads (725 thousand recent), Apache-2.0, last push
  2023-11-03. A `Read` adapter that turns `//`, `/* */` and `#` comments into spaces so serde_json can parse; no
  trailing-comma handling (grep for "comma" in src/lib.rs finds nothing). https://github.com/tmccombs/json-comments-rs . Because comments become spaces, serde_json's
  line/column should stay meaningful (inference from "changes all comments to spaces", src/lib.rs:59; not tested).
- json-strip-comments (oxc-project, a fork of json_comments): 3.1.2, 2026-07-27, 6.8 million downloads (2.5 million
  recent), Apache-2.0. Strips comments and trailing commas in place, filling with spaces (src/lib.rs:1, 103, 211 to
  236), so offsets are preserved. Active. https://github.com/oxc-project/json-strip-comments . Strip-then-serde_json is
  the cheapest path and keeps serde_json's line/column errors, but gives zero edit support.
- jsonc-to-json (vallentin): 0.1.1, 2023-06-23, 6 thousand downloads. Tokenizer-based comment and trailing comma
  removal; last push 2024-03-27. Too small to rely on; no commit since 2024-03-27.
- json-five (spyoungtech): 0.3.1, 2026-01-08, 1.38 million downloads (560 thousand recent), MIT, last push 2026-05-13, 5
  open issues. JSON5 parser with serde support; README claims "round-trip use cases with preservation/editing of
  whitespace and comments", with model-based and token-based edits. Newer and smaller than jsonc-parser; I did not
  test its key-level edit ergonomics. https://github.com/spyoungtech/json-five-rs
- Biome (`biome_json_parser`/`biome_json_syntax`): published, but biome_json_parser on crates.io is at 0.5.7 from
  2024-03 and biome_json_syntax at 0.7.0; these are internal crates of a workspace, not a stable API. Mentioned only to
  say they exist.
- Not found: any crate named `jsonc-rs`. A crate named `jsonc` 0.1.0 (2025-10-07, 3 thousand downloads, no repository
  link) exists; ignored.

## 4. Schema and editor support

- schemars 1.2.3, 2026-10-12, 593 million downloads (223 million recent), MIT, MSRV 1.74; 123 open issues, last push
  2026-10-12. Derives JSON Schema from serde types; default dialect is draft 2020-12; `SchemaSettings::draft07()`,
  `draft2019_09()`, `draft2020_12()` exist (schemars/src/generate.rs:26 to 106).
  https://github.com/GREsau/schemars
- `$schema` inside a config instance is an editor convention, not part of JSON Schema: the specification's `$schema`
  keyword "is used to declare which dialect of JSON Schema the schema was written for"
  (https://json-schema.org/understanding-json-schema/reference/schema). Consequence: the Rust config struct needs an
  explicit optional `$schema: Option<String>` field (renamed via `#[serde(rename = "$schema")]`), otherwise the
  generated schema (especially with `deny_unknown_fields`) will flag the key. This is my inference, not tested.
- VS Code: JSON files can be associated with a schema "in the JSON file itself using the $schema attribute" or through
  the `json.schemas` setting with `fileMatch` wildcards and `!` exclusions; the `jsonc` language mode is "used for the
  VS Code configuration files such as settings.json, tasks.json, or launch.json" and "also accepts trailing commas, but
  they are discouraged and the editor will display a warning".
  https://code.visualstudio.com/docs/languages/json
  Caveat: this was fetched through a summarizing tool, so exact wording should be rechecked before quoting.
- JetBrains: the IDE docs I fetched (https://www.jetbrains.com/help/webstorm/json.html and
  https://www.jetbrains.com/help/idea/json.html) do not mention `$schema` in an instance, `.jsonc`, or trailing commas.
  They describe schema association by file name or custom mappings, and `.json5` files treated as JSON5. So for
  JetBrains, `$schema` in instances and `.jsonc` handling are unconfirmed here. Shipping the schema to SchemaStore or
  documenting a manual mapping would be the safe route.
- figment 0.10.19: crates.io release is 2024-05-17 (45 million downloads, 13 million recent), but the repo is active
  (last push 2026-10-13; unreleased commits 2026-09-28 "Track full error origins via Error::origins()", 2026-10-11).
  Features: `json`, `toml`, `yaml`, `env`, `test`, `preserve_order` (Cargo.toml `[features]`). No JSON5/JSONC provider
  in-tree (grep for `json5|jsonc` in src returned nothing); providers are a trait, so a custom JSONC provider is
  possible. Merge semantics from doc examples: `join` keeps the existing value ("existing value retained", src/figment.rs:217 to 229), `merge` lets the incoming value replace it ("incoming value replaced existing", figment.rs:299 to 311), `adjoin`/`admerge` concatenate arrays. `Error::origins()` (src/error.rs:417) lists sources; `rg span|line|column` over src/error.rs finds no position type, so origin tracking only. https://github.com/SergioBenitez/Figment
- serde_path_to_error 0.1.20, 2025-09-15, 341 million downloads (108 million recent), dtolnay. Wraps any Deserializer
  and reports the field path (README example yields `dependencies.serde.version`). Path only, no spans.
  https://github.com/dtolnay/path-to-error
- miette 7.6.0, 2025-04-27, 87 million downloads (23 million recent), Apache-2.0. `Diagnostic` derive with
  `NamedSource` and `#[label] SourceSpan` renders annotated source snippets. Last release 2025-04-27.
  https://github.com/zkat/miette

## 5. Structured-edit crates for targets

- toml_edit 0.25.17+spec-1.1.0, 2026-10-09, 961 million downloads (257 million recent), MIT OR Apache-2.0, MSRV 1.85;
  repo `toml-rs/toml` pushed 2026-10-28. "parse and modify toml documents, while preserving comments, spaces and
  relative order of items" (crate README). `DocumentMut`, `Item`, `value()`. Its serde deserializer error exposes
  `span() -> Option<Range<usize>>` (crates/toml_edit/src/de/error.rs:28). Best-in-class and what cargo-edit uses.
  https://github.com/toml-rs/toml
- Format-preserving YAML editing is a real gap, with two live options:
  - yaml-edit 0.3.5 (jelmer), 2026-10-22, 243 thousand downloads, Apache-2.0, 11 releases since 2025-08, 1 open issue,
    pushed 2026-10-22. Built on rowan (lossless tree); README claims lossless parsing, in-place edits via
    `Mapping::set`, error recovery, position tracking; DESIGN.md states "byte-identical round trips"; TODO.md lists
    serde integration as not done. Young and pre-1.0. https://github.com/jelmer/yaml-edit
  - yamlpath (read-only route lookup) and yamlpatch (edits: `Op::Add`, `MergeInto`, `Remove`, `Append`,
    `RewriteFragment`, `ReplaceComment`, `EmplaceComment`), both from zizmor, on tree-sitter-yaml. yamlpatch 1.31.1
    stable (2026-10-15; 1.32.0-rc1 on 2026-10-23), 397 thousand downloads. README: "Comment and format-preserving YAML
    patch operations", and "not a substitute for comprehensive YAML processing libraries". Versions track zizmor's
    release, not semver for the crate alone. https://github.com/zizmorcore/zizmor/tree/main/crates/yamlpatch
  - Read-only YAML parsing for deserialization: serde_yaml 0.9.34+deprecated is archived ("no longer maintained", repo
    archived, last push 2024-03-25). Live successors by download: serde-saphyr 2.0.0 (2026-10-13), yaml_serde 0.10.7
    (2026-08-18, `github.com/yaml/yaml-serde`), serde_norway 0.9.42 (2024-12), serde_yaml_ng 0.10.0 (2024-05), saphyr
    0.3.3 (2026-10-27). I did not evaluate their error quality.
- JSON/JSONC key-level edits: jsonc-parser `cst` feature (section 3). Comment-preserving, keeps trailing-comma style
  (`uses_trailing_commas`, `set_trailing_commas`).
- Whole-file overwrites need no parser.

## 6. Hashing, watching, walking

- notify: stable 8.2.1 (2026-10-25); 9.0.0-rc.7 (2026-10-28) is a pre-release in heavy fix mode (CHANGELOG lists inotify,
  FSEvents, Windows fixes through 2026-10-28). 184 million downloads. License CC0-1.0. Docs list known problems:
  network filesystems may emit no events, WSL `/mnt/c` may not produce inotify events, Docker on macOS M1 fails with
  os error 38, and editors that replace files change event shapes (notify/src/lib.rs:35 to 79).
  https://github.com/notify-rs/notify ; debouncer: notify-debouncer-full stable 0.7.0 (2026-01-23), 0.8.0-rc.2
  (2026-05-02). A watch mode should debounce and treat "file replaced" as a normal event.
- twox-hash 2.1.5, 2026-10-04, 282 million downloads (70 million recent), MIT, MSRV 1.81. XXH3-128 is feature
  `xxhash3_128`, in the default feature list; one-shot call `XxHash3_128::oneshot(&[u8]) -> u128`
  (src/xxhash3_128.rs:37). https://github.com/shepmaster/twox-hash
- ignore 0.4.34 and globset 0.4.21, both 2026-10-27, from BurntSushi/ripgrep `crates/`, MSRV 1.88, Unlicense OR MIT,
  201 million and 275 million downloads. `ignore` is gitignore-aware directory walking (and is what ripgrep uses);
  `globset` compiles many globs into one matcher.

## 7. Implications

Strongest combination for a JSONC config with good error locations (assuming the tool also edits JSONC targets):

1. jsonc-parser (features `serde`, `cst`) as the single JSONC engine: it reads the config, and its CST edits
   `tsconfig.json` or `.vscode/settings.json` without losing comments. It is also the most actively maintained JSONC
   crate here (0.34.0, 2026-09-27).
2. For config errors with locations: parse with `parse_to_ast` (ranges), deserialize with `serde_path_to_error` for the
   field path, resolve the path against the AST to a byte range, and render with miette (or hand-print `file:line:col`).
   This is needed because jsonc-parser's serde type errors have no position (errors.rs:179). Alternative with less
   plumbing: json5 1.3.6 attaches positions to type errors itself, but gives up CST editing and accepts the full
   JSON5 grammar.
3. clap (derive, features `derive` and `env`) for the CLI; clap_complete for AOT completions, clap_mangen from an xtask.
   Layering CLI over env over file is about a screenful of code, which is why config-rs is optional.
4. schemars 1.x to emit a JSON Schema into the repo; add an explicit `$schema` field to the config struct; VS Code
   documented support covers `$schema` and `json.schemas`; JetBrains support is unconfirmed here.
5. Targets: toml_edit (clearly), jsonc-parser CST (JSON/JSONC), and for YAML either yaml-edit (rowan CST, young) or
   yamlpatch (tree-sitter, route-based ops, versioned with zizmor). There is no mature, widely used equivalent of
   toml_edit for YAML; plan for a spike on real target files.
6. Watch/hash/walk: notify 8.2.1 plus notify-debouncer-full 0.7.0 (avoid the 9.0 release candidate until stable),
   twox-hash `XxHash3_128::oneshot`, ignore + globset.

config-rs: a fit only if the goal is "layered values from many sources into a struct" and file-level diagnostics are
secondary. It is a liability for this tool's stated need, for three reasons verified in source: (a) it has no JSONC
format; JSON5 is the nearest built-in, and a custom `Format` would flatten the parse into its own `Value` tree where
positions are lost (format.rs:16 to 25); (b) `ConfigError` has no span or line field, only file and key path
(error.rs:43 to 102); (c) env keys are lowercased (env.rs:269) and the open case-conversion issues (#340, #625) show this
is a known sharp edge (file keys appear to be kept verbatim at HEAD, unexecuted). It is also read-only and still 0.x. Its strengths are real: active release stream, many
formats, env layering, and `Format` as an escape hatch. figment is the closer design alternative (merge/join, error
origins, unreleased improvements since May 2024), but it has no JSONC provider either, so the same position problem
applies. If "config" only meant "a configuration file", none of these layering crates is required.

Gaps and unverified items: no code was executed (an mvm throwaway VM could close this gap); serde_jsonc
capability unconfirmed; JetBrains `$schema` support unconfirmed; json-five edit ergonomics untested; YAML editor
fidelity on the repo's real targets untested; VS Code quotes came through a summarizing fetch.
