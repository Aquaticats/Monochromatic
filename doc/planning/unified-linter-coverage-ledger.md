# Unified linter coverage ledger

Every responsibility the incumbent Rust linter and Markdown linter carry,
who consumes it,
and its owner in the unified linter (rule RCO in the `choosing-technology` skill).
Design interview state:
[`doc/handover/unified-linter.md`](../handover/unified-linter.md).

Source:
a read-only survey on 2026-09-23 with file and line citations,
spot-checked by the parent session
(`md040-fenced-code-language.ts:154` inserts `text`;
no `rust-linter.toml` exists outside ignored trees;
the root `lint` aggregate in `mise.toml` does not list `lint:rust`;
`context.rs:141-153` keeps `parse.syntax_node()` and no crate reads `parse.errors()`).
Paths are repository-relative;
line numbers are as of commit `8d73b1bab`.

The source survey and early-round owner states are historical.
Current implementation choices are governed by `doc/planning/unified-linter.md`,
including decisions settled after this inventory.
On 2026-10-04,
the cli-git rewrite agent also became the unified-linter owner,
JSONC replaced HCL,
and container,
mutation,
and fuzz tests became explicit acceptance gates.

## New Rust requirements

The user added these requirements on 2026-10-04;
they are not incumbent parity work:

- Reject anonymous functions using the new `rust/no-anonymous-functions` rule.
  The implementation detects actual `CLOSURE_EXPR` nodes,
  including assigned,
  nested,
  move,
  async,
  and const closures.
  It does not expand macro token trees or rewrite captured state automatically.
  Named functions,
  methods,
  and function pointers stay allowed.
  Configuration registration and the focused container suite passed:
  5 tests,
  77 unrelated tests filtered out.
  This is not a full-suite pass.
  Focused mutation verification caught both viable mutants;
  the remaining mutant is unviable because `Diagnostic` has no `Default` implementation.
  The new Rust-style ASAN target ran 1,274 verified executions with exit 0
  (`campaign-xNoaiG`).
  The separate container Clippy gate passed after applying the newly enforced shadowing policy
  without relaxing it.
  Rule dispatch in the unfinished executable remains pending.
- Require explicit Rust annotations,
  including the motivating `parse::<u16>()`,
  `.map::<String, _>(user_name)`,
  and `.collect::<Vec<String>>()` calls.
  The closure ban supersedes the originally shown inline closure.
  The user selected full enforcement (option A),
  authorizing semantic resolution for `rust/require-explicit-types`
  and superseding the prior syntax-only restriction for this rule.
  Preserve `_` for unnameable function-item types.
  Do not mistake unresolved standalone snippet calls for verified annotation coverage.

## Native implementation verification

The active implementation record is
[`cli-git-rust-implementation.md`](../handover/cli-git-rust-implementation.md),
whose `Handoff 2026-10-05` section carries the full current state.

Implemented and verified:
configuration merge and discovery,
Rust rules (max-lines, rustdoc, anonymous functions, explicit types),
native Markdown/MDX adapter and rules,
JSONL/stdin-fix output routing,
literal/glob discovery,
bounded fixpoint loop,
embedded processors (virtual Rust and Rustdoc extraction with host-mapped fixes),
and the forbidden-strings in-process scanner API.

Verification evidence:
170-test container gate (`proc_0fca`),
processor delegate 203-test suite with Clippy and guard-focused mutation
(image `62a207e21b353bf4`),
scanner delegate full suite with ASan fuzzing (256,297 runs) and panic controls,
and fuzz campaign `campaign-KqhBLf` covering merge, configuration,
rust-style, explicit-types, and Markdown/MDX including tables and prose.

Open mutation state:
full-snapshot campaign `proc_3d86` recorded 18 survivors on its older snapshot;
Markdown-scoped rerun `proc_ea56` recorded 17 survivors
centered on table cell boundaries, continuation prefixes,
and prose break/block-start guards.
Both survivor inventories need disposition or new controls before any completion claim.

Remaining implementation:
linter executable orchestration and consumer migration,
native cli-git wrapper (configuration, policies, Git operations, transactions),
Windows-native scanner mutation verification,
and native LFS URL normalization vetting
(planning record: `native-lfs-url-normalization-evaluation.md`).

## Rust linter

Packages:
`package/linter/rust` (CLI and language server),
`package/rust-module/rust-linter-core`,
`package/rust-module/rust-linter-pattern`,
`package/rust-linter-plugin/builtin`.

### Rules

- `builtin/max-lines`:
   code lines per file counted with the real lexer,
   blank and comment-only lines excluded,
   reported where the budget is crossed
   (`max_lines.rs:133-219`).
  Category `pedantic`,
   non-suppressible (`max_lines.rs:110-112`),
   not fixable.
  Option `max`,
   default 300 (`default.toml:21`),
   precedence `--max`,
   then configured `max`,
   then 300 (`lib.rs:573-594`).
  Owner:
   Rust rules plugin,
   exact behavior (decided,
   round 1 parity).
- `builtin/require-rustdoc`:
   doc comment on fn,
   struct,
   enum,
   union,
   trait,
   type alias,
   const,
   static,
   module,
   extern crate,
   use,
   impl block,
   enum variant,
   record and tuple fields,
   and the file itself through `//!`
   (`require_rustdoc.rs:103-120`).
  Detection through `ra_ap_syntax` `DocCommentIter` (`289-306`).
  Hardcoded cxx-qt carve-out:
   files mentioning `cxx_qt` or `cxx_qt_lib` skip `use` items and trait-impl members
   (`164-182`,
   `202-217`,
   `265-267`).
  Macros are not covered.
  Non-suppressible (`501-512`),
   not fixable.
  Owner:
   Rust rules plugin,
   exact behavior (decided).
- Declarative `[[pattern]]` rules:
   keys `id`,
   `match`,
   `message`,
   `fix`,
   `help`,
   `severity` (default `error`;
   `config/file.rs:103-141`).
  Category `restriction`,
   suppressible,
   fix registered at Suggestion trust (`pattern_rule.rs:167-181`),
   dropped when the template names an unbound metavariable.
  An unparseable `match` exits 2.
  Matcher:
   `META_` metavariables,
   fragment parsed as item,
   then expression,
   then statement in a fn body (`fragment.rs:82`),
   structural and whitespace-insensitive,
   strings and comments skipped,
   repeated metavariables must bind identical text.
  Owner:
   Rust language plugin,
   Rust-only (decided,
   round 2),
   deferred until a consumer exists in the final design.
  The historical HCL format choice was replaced by JSONC on 2026-10-04;
   no declarative pattern-rule format is being added in this implementation.
- `builtin/invalid-disable-directive`,
   always `error`:
   a directive without justification,
   or aimed at a non-suppressible rule
   (`directive/apply.rs:19`,
   `164-195`).
  Owner:
   core,
   both languages (open:
   directive spelling).
- `builtin/unused-disable-directive`,
   opt-in (`apply.rs:16`,
   `105-117`,
   `197-211`).
  Owner:
   core,
   ESLint-shaped `reportUnusedDisableDirectives` (open:
   spelling).

### Command line

Flags (`cli.rs`):
`--max`,
`-c`/`--config`,
`--disable-nested-config`,
`-A`/`-W`/`-D` applied left to right,
`--lsp`,
`--print-config`,
`--rules`,
`--init`,
`--threads`,
`--ignore-path`,
`--ignore-pattern`,
`--no-ignore`,
`--no-error-on-unmatched-pattern`,
`--debug`,
`--fix`,
`--fix-suggestions`,
`--fix-dangerously`,
`--report-unused-disable-directives`,
`--report-unused-disable-directives-severity`,
`--quiet`,
`--silent`,
`--deny-warnings`,
`--max-warnings`,
positional paths defaulting to `.`.
`--debug=timings` is advertised but not implemented (`lib.rs:269`).
Owner:
ESLint-shaped flags,
every capability kept (decided,
round 2);
the mapping is open.

### Configuration

- `rust-linter.toml`,
   discovered from the process working directory up to the root,
   merged outermost first,
   on top of a compiled-in `default.toml`
   (`load.rs:25`,
   `198-264`;
   `config.rs:29`,
   `68-72`).
- `extends` with cycle detection;
   keys `include-patterns`,
   `ignore-patterns`,
   `options`,
   `categories`,
   `rules`,
   `overrides`,
   `pattern`,
   `plugins`,
   `settings`,
   all `deny_unknown_fields`.
- Severities `off`,
   `warn`,
   `error`,
   with `allow` and `deny` aliases;
   seven oxlint categories,
   only `correctness` on by default.
- Resolution order:
   category default,
   `[categories]`,
   pattern severity,
   `[rules]`,
   overrides in order,
   the winning layer's include and exclude,
   then `-A`/`-W`/`-D`
   (`resolve.rs:235-313`).
- Unused surfaces:
   `settings` has no reader,
   and rules receive only `max_lines`,
   so no other rule option reaches a rule.
- Measured use:
   no `rust-linter.toml` exists;
   every consumer runs on built-in defaults.
- Owner:
   the unified linter's JSONC loader,
   using the repository's Rust JSONC package.
  Configuration is an ordered array of blocks;
   nearest-file lookup and ordered deepmerge semantics are settled in the final design.

### Default exemptions

Both rules are off for `**/tests/**`,
`**/*_tests.rs`,
`**/fuzz/**`,
`build.rs`,
`**/fixture/**`,
`**/test-fixture/**`,
and `**/invalid/**` (`default.toml:28-78`).
The last three go beyond `AGENTS.md` MXR and RDC.
The walker is gitignore-aware and picks only `.rs` files;
an explicitly named file bypasses the walk.
Owner:
compiled-in default configuration (decided,
parity).

### Suppression directives

- `rust-linter-disable`,
   `-enable`,
   `-disable-line`,
   `-disable-next-line` in any comment token,
   including doc and block comments;
   text inside strings is ignored.
- Rules named by `id` or `plugin/id`;
   none named means all.
- Justification after `--` is mandatory;
   without it the directive suppresses nothing and is reported.
- Non-suppressible rules refuse the directive and keep the finding.
- A `disable` lasts until the next `enable` or end of file.
- Measured use:
   no directives outside the linter's own crates.
- Owner:
   core,
   for both languages;
   spelling open (depends on the name).
- Built 2026-10-18 (merge `6808166dfd`) as `monochromatic-lint-disable` and siblings:
   doc comments are not directives (this list said "any comment token"),
   only full rule identifiers are accepted,
   and the Markdown form is an HTML comment.
   See the package README section "Inline directives".

### Autofix

- Fix kinds Safe,
   Suggestion,
   Dangerous,
   gated by `--fix`,
   `--fix-suggestions`,
   `--fix-dangerously`.
- Up to 10 fix passes per file,
   then a reporting pass;
   edits sorted,
   overlaps skipped.
- Defect:
   the applier flattens edits across fixes (`fix/apply.rs:41-46`),
   so a multi-edit fix can land partly,
   against the "A Fix is atomic" claim in the core README.
  Only single-edit fixes exist today.
- Owner:
   core;
   the trust model under ESLint's fix and suggestion split is open.

### Output and exit codes

- JSONL only,
   one oxlint-shaped record per finding;
   a clean run prints nothing.
- `--quiet` hides warnings that still count;
   `--silent` prints nothing.
- Exit 0 clean,
   1 failing findings,
   2 setup error.
- Owner:
   ESLint-shaped formatters,
   JSONL kept as a capability (decided,
   round 2);
   the default is open.

### Parallelism

`--threads`,
defaulting to available parallelism;
the directory walk itself is sequential,
although the parity plan called for a parallel walk.
Owner:
core.

### Language server

- Hand-rolled JSON-RPC over stdio;
   `initialize`,
   `shutdown`,
   `exit`,
   `didOpen`,
   `didChange`,
   `didClose`;
   full document sync;
   diagnostics only.
- Defects:
   `lint_text` applies neither ignore patterns nor directives,
   so the editor can disagree with the CLI;
   URIs lose percent-decoding.
- Measured use:
   no editor configuration registers it.
- Owner:
   one server for both languages;
   framework open
   (`doc/audit/tech-meow-language-server-framework-vet-2026-09-17.md`
   recommends `lsp-server` 0.10.0 with `gen-lsp-types` 0.11.0).

### Introspection and library surface

- `--print-config`,
   `--rules`,
   `--init`.
- `run_cli` and `run_cli_from_env`;
   the latter drops ordered `-A`/`-W`/`-D` overrides and has no callers.
- No other `Cargo.toml` depends on these crates.
- `ra_ap_syntax` pinned `=0.0.335` in the CLI and core.
- Owner:
   ESLint-shaped `--print-config` and `--init`;
   `--rules` kept as a capability.

## Markdown linter

Package:
`package/cli/markdown-lint`.

### Engine

- Sätteri `markdownToMdast` and `mdxToMdast` with GFM and frontmatter (`parse.ts:1-22`,
   `58-85`);
   catalog pin `satteri >=0.10.5`.
- Code-point offsets converted to UTF-16 when astral characters appear.
- MDX nodes and their subtrees skipped by the shared walker (`walk.ts:14`,
   `96`).
- Owner:
   Markdown language plugin;
   parser crate open (research running).

### Rules

No rule takes options;
former markdownlint settings are hardcoded.

- `MD001` heading-increment:
   report-only.
- `MD014` commands-show-output:
   fixable,
   strips `$ ` prompts.
- `MD024` no-duplicate-heading:
   `siblings_only`,
   report-only.
- `MD025` single-h1:
   report-only.
- `MD026` no-trailing-punctuation:
   `.` and `:`,
   fixable.
- `MD034` no-bare-urls:
   `http`,
   `https`,
   `ftp`,
   and email;
   fixable by wrapping in angle brackets.
- `MD036` no-emphasis-as-heading:
   report-only.
- `MD040` fenced-code-language:
   fixable by inserting `text` (`md040-fenced-code-language.ts:154`);
   the package README wrongly calls it report-only.
- `MD053` reference-definitions:
   fixable,
   removes unused and duplicate definitions.
- `MD054` link-image-style:
   shortcut style only,
   fixable to collapsed style.
- `no-pipe-tables`:
   fixable to an HTML `<table>`.
- `semantic-line-breaks`:
   add-only fix after `,` `.` `;` `:` `?` `!`
   followed by whitespace or end of prose,
   with abbreviation,
   ellipsis,
   closing-delimiter,
   and bold-span guards.
- `lfs-image-url`:
   fixable rewrite to `<objectBase>/<oid>/<path>`,
   reading `.lfsconfig`,
   root `.gitattributes`,
   and the file oid;
   inert without `.lfsconfig` or for `--lfs-image-exclude` paths.
- Synthetic `markdown-lint-error`:
   a per-file processing failure becomes a finding.
- Synthetic `markdown-lint-safety`:
   refuses a fix that would empty a non-empty file.
- Owner:
   Markdown rules plugin,
   exact behavior (decided,
   round 1);
   synthetic findings move to core (open:
   whether they cover Rust too).

### Command line

`--fix`,
`--format=pretty|json` with `--json` winning,
repeatable `--lfs-image-exclude=` and `--rule=`,
`--stdin-path=` (one path,
no positional paths,
`.md` or `.mdx`),
`--help`.
Unknown flags are usage errors.
Owner:
ESLint-shaped flags (decided,
round 2);
`--lfs-image-exclude` becomes rule configuration or a flag (open).

### Configuration and discovery

- No configuration file and no severities:
   every finding fails the run.
- Walker:
   `.md` and `.mdx` only,
   nested `.gitignore` layers through the npm `ignore` package,
   built-in ignores `.git`,
   `node_modules`,
   `package-paused`,
   `package-deprecated`,
   `.out-of-scope`,
   explicit files bypass the walk,
   overlapping roots deduplicated,
   files over 5 MiB skipped silently.
- Owner:
   one walker in core;
   the built-in ignores become default configuration (decided,
   parity).

### Suppression

None;
the repository's Markdown holds no leftover `markdownlint-disable` comments.
Owner:
core directives gain an HTML-comment form for Markdown (decided,
round 2 ESLint surfaces;
spelling open).

### Autofix

Half-open offset edits,
applied from the end,
overlaps dropped,
re-parsed each pass for up to 10 passes,
then linted once more;
atomic writes that keep the file mode.
Owner:
core.

### Output and exit codes

- `pretty`:
   `path:line:col  ruleId  message`;
   `json`:
   an array of `{path,line,column,ruleId,message,fixable}`.
- Report on stdout,
   the fixed-file count on stderr;
   in stdin mode with `--fix`,
   fixed source on stdout and the report on stderr.
- Exit 0 clean,
   1 violations remain,
   2 usage error.
- Library exports (`index.ts`) have no consumer outside the package's tests and the git policy's subprocess call.
- Owner:
   ESLint-shaped formatters and `--stdin` handling (decided,
   round 2);
   exact mapping open.

## Commit-time integration

Package:
`package/git-policy/markdown-lint`,
registered as `markdown/autofix` in `cli-git.config.ts:22-43`.

- Default severity `warn`,
   warn-safe;
   triggers pre-forward,
   post-commit,
   manual-push,
   direct-check,
   direct-fix.
- Options `command` (default `['node','package/cli/markdown-lint/src/cli.ts']`),
   `rules` (default `['lfs-image-url']`),
   `exclude` (also forwarded as `--lfs-image-exclude`).
- One candidate at a time through the stdin fix mode;
   exit 1 means violations,
   read from stderr;
   exit 2 or anything else raises `MarkdownLintPluginError`.
- Findings `markdown-autofix` with a full-content patch,
   or `markdown-violation`.
- Non-UTF-8 candidates skipped.
- Also exports `createFullContentPatch`,
   consumed by `package/git-policy/repository/src/dependent-version-bump-policy.ts:17`.
- Shipped into cli-git through file-enforcer mirrors (`file-enforcer.config.ts:2294-2310`,
   `2270-2273`).
- Owner:
   stays,
   pointed at the new binary (decided,
   round 2 cutover plan);
   how it finds the binary is open.

## Consumers

- Root `lint:rust` fans out to 17 package `lint:rust` tasks,
   each `cargo run --quiet --manifest-path <rel>/linter/rust/Cargo.toml -- .` in debug profile.
- `mise run lint` does not run the Rust linter:
   the aggregate's comment says step 1 reaches `lint:rust` through each package's `lint`,
   but those are `cargo check`,
   cargo dispatches,
   or Android Gradle Lint.
  The same claim appears in `doc/planning/monorepo-manager-from-scratch-design.md:1892`.
- Rust crates without `lint:rust`:
   `package/forbidden-strings/cli.fuzz`
   and `package/forbidden-strings/core.fuzz`,
   whose path escapes the `**/fuzz/**` default.
- Root `lint:markdown` and `format:markdown`
   run `node package/cli/markdown-lint/src/cli.ts --lfs-image-exclude=package/ssg/`,
   inside the `lint` and `format` aggregates.
- `file-enforcer.config.ts`:
   `CARGO_PROFILE_LINTER` (`1570`,
   `1624`),
   Cargo manifest management,
   and the cli-git mirror.
- `package/config/pnpr/config.yaml:50`,
   `79`:
   private-registry publishing entries for the Markdown linter.
- `package/config/lfs-r2-worker/README.md:77-80`:
   the server contract `lfs-image-url` relies on.
- No CI workflow,
   editor configuration,
   or hook runs either linter,
   apart from cli-git.
- `AGENTS.md`:
   MXR names `monochromatic-rust-linter`;
   RDC names `require-rustdoc` and the cxx-qt carve-out;
   MXL,
   LN3,
   MD1,
   MD3,
   MD4,
   MD5,
   and MD6 map to rules.
- Owner:
   every consumer switches in one change (decided,
   round 2).

## Size and health

Measured with `tokei` 15.0.0;
code lines,
tests excluded unless named.

- Rust:
   2,661 source and 2,962 test lines;
   257 `#[test]` functions.
- Markdown linter:
   3,969 source and 2,132 test lines of TypeScript;
   137 tests.
- Commit policy:
   485 source and 458 test lines;
   14 tests.
- Commits touching the Rust linter packages in the last 90 days:
   36 for `package/linter/rust` including its old path,
   15 core,
   2 pattern,
   4 builtin.
- Commits touching the Markdown linter in the last 90 days:
   40 including its old path.
- Open issues:
   #400 (Rust linter configuration format),
   #232,
   #293,
   #294,
   #447,
   #513,
   #516,
   #491,
   #289,
   #58,
   and #110 (stale,
   tracks the retired `markdownlint-cli2`).

## Related documents

- `doc/planning/rust-linter-oxlint-parity.md`:
   the Rust linter's own grilling record (decisions D1 to D7);
   rejected dylint (nightly `rustc_private`),
   runtime JavaScript plugins,
   WebAssembly plugins,
   Pkl,
   ast-grep `$X` metavariables,
   and ten output formats.
  Undelivered:
   parallel walk,
   `--debug=timings`,
   LSP code actions and incremental sync,
   the `not-inside` pattern key.
- `doc/planning/load-bearing-code-languages.md`:
   approval is explicit and scoped;
   `doc/decision/monorepo-manager-all-rust.md` approves Rust for meow only.
  The user's round 1 answer ("Rust only.
  I've already done the vetting.")
  is the approval for this work.
- `doc/troubleshooting/markdownlint-cli2.md`:
   why `markdownlint-cli2` was retired.
- `doc/handover/markdownlint-no-pipe-tables.md`:
   why pipe tables convert to HTML.
- `doc/handover/markdown-lint-satteri-benchmark.md`:
   the parser swap and the rejected Sätteri-plugin rewrite.
- `doc/troubleshooting/satteri-offsets.md`
   and `doc/troubleshooting/napi-loader-bundled-out-of-package.md`:
   the offset and bundling problems the Node half carries.
- `doc/planning/lfs-readme-image-rendering.md`:
   origin of `lfs-image-url` (#476).
- `package/cli/markdown-lint/doc/semantic-line-breaks-breaks-bold.md`:
   why the rule guards bold spans.
- `doc/planning/learning-rust-formatting-boundary.md`
   and `doc/handover/issue-401-formatting-investigation.md`:
   a proposed repository-owned CSS checker,
   adjacent scope.

## Verification ownership

The implementing agent owns container tests,
mutation testing,
and fuzzing for the Rust wrapper,
unified linter,
and their integration.
The concrete targets and acceptance criteria are recorded in
[`cli-git-rust-implementation.md`](cli-git-rust-implementation.md#verification-gates).
No retained responsibility is complete solely because unit tests compile or pass.

## Incidental defects

Recorded here for the port,
not fixed during the design interview.

- Package README says `MD040` is report-only;
   it fixes.
- `mise.toml:890-891` says `lint:markdown` runs under Bun;
   it runs under Node.
- `mise.toml:953-957` says dprint formats Markdown;
   the dprint Markdown plugin is disabled.
- `mise.toml:908-909` cites #235 as open;
   it closed on 2026-06-03.
- `rust-linter-core/Cargo.toml:44-46` mentions JSON,
   GitLab,
   and SARIF formats that were cut.
- `doc/troubleshooting/markdownlint-cli2.md` says the replacement runs under Bun.
- `package/linter/rust/README.md:7-23` says clippy cannot require docs on private items;
   `cargo clippy --explain missing_docs_in_private_items` documents such a lint.
