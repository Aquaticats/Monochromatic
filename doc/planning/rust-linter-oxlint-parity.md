# Rust linter: oxlint feature parity

Working document for the grilling session started 2026-07-25.
Goal stated by the user:
make `package/linter/rust` (`monochromatic-rust-linter`) feature-parity with oxlint,
where "feature" means linter capabilities,
explicitly not the number of rules.

Status:
 decisions still being collected.
Nothing here is ratified until the user accepts;
promote to `doc/decision/` only then (DRR).

## Measured starting state

`package/linter/rust`,
 4386 lines across 13 files (`wc -l`),
 of which the
non-test implementation is `src/{cli,config,context,diagnostic,lib,main,rule}.rs`
plus `src/builtin/{max_lines,require_rustdoc}.rs`.
Comment density is high:
`src/config.rs` spends 317 lines on two path predicates because of the
`dum-dum-non-ts` `// What:` / `// Why:` convention.

What exists today:

- CLI:
   positional paths plus `--max <N>`,
  `--help`,
  `--version` (clap).
- Two rules,
   hardcoded on,
   hardcoded at error severity:
  `max-lines`,
  `require-rustdoc`.
- `all_rules()` in `src/rule.rs` returns a fixed `Vec<Box<dyn Rule>>`.
  The README calls the linter "pluggable";
  there is no plugin mechanism of any kind.
- Exemptions are hardcoded Rust predicates in `src/config.rs`
  (`max_lines_exempt`,
  `missing_rustdoc_exempt`),
  duplicated once per rule.
- Discovery:
  `ignore::WalkBuilder::new(..).build()`,
   sequential,
   gitignore-aware.
- Output:
   one hardcoded line format,
  `path:LINE: error[rule]: message`,
   printed to stdout.
- Exit codes:
   0 clean,
   1 violations,
   2 fatal.
- No config file,
   no severity control,
   no categories,
   no suppression,
  no autofix,
   no output formats,
   no parallelism,
   no LSP,
   no semantic analysis.

Repo footprint the linter serves:
310 tracked `.rs` files across 15 Cargo packages,
invoked per package through the `lint:rust` fanout in the root `mise.toml`.

## Measured oxlint feature surface

Installed version:
 1.75.0 (`oxlint --version`).
Sources:
`oxlint --help`,
`node_modules/oxlint/configuration_schema.json`,
and a real `--format json` run.

Config file top-level keys (`configuration_schema.json`):
`$schema`,
`categories`,
`env`,
`extends`,
`globals`,
`ignorePatterns`,
`jsPlugins`,
`options`,
`overrides`,
`plugins`,
`rules`,
`settings`.

`options` holds `denyWarnings`,
`maxWarnings`,
`reportUnusedDisableDirectives`.
`overrides` entries take `files`,
`excludeFiles`,
`rules`,
`env`,
`globals`,
`plugins`.
`categories` are `correctness`,
`suspicious`,
`pedantic`,
`perf`,
`style`,
`restriction`,
`nursery`,
 each set to an `AllowWarnDeny` value.
Rule entries are either a bare severity or `[severity, options]`.

CLI surface beyond config:
`-A/-W/-D` accumulation over rules and categories,
`--fix`,
`--fix-suggestions`,
`--fix-dangerously`,
`--ignore-path`,
`--ignore-pattern`,
`--no-ignore`,
`--quiet`,
`--deny-warnings`,
`--max-warnings`,
`--format` with `checkstyle`,
`default`,
`agent`,
`github`,
`gitlab`,
`json`,
`junit`,
`sarif`,
`stylish`,
`unix`,
`--debug=files,timings`,
`--silent`,
`--no-error-on-unmatched-pattern`,
`--threads`,
`--print-config`,
`--rules`,
`--init`,
`--lsp`,
`--disable-nested-config`,
`--type-aware`,
`--type-check`,
`--report-unused-disable-directives[-severity]`,
`--config`,
`--tsconfig`.

JSON diagnostic shape,
 captured from a real run:

```json
{ "diagnostics": [{ "message": "...", "code": "eslint(no-debugger)",
  "severity": "error", "causes": [], "url": "https://...", "help": "...",
  "filename": "/abs/path", "labels": [{ "span": { "offset": 27, "length": 9,
  "line": 2, "column": 15 } }], "related": [] }],
  "number_of_files": 1, "number_of_rules": 479, "threads_count": 16,
  "start_time": 0.283 }
```

Which oxlint features this repo actually leans on
(`package/config/oxlint/src/config-base.ts` and `overrides.ts`):
`categories`,
`env`,
`ignorePatterns`,
`options`,
`plugins`,
`rules`,
`settings`,
and a large glob-driven `overrides` list.
Those overrides are exactly the mechanism the Rust linter hardcodes:
`eslint/max-lines` is turned off for `**/fixture.*`,
`**/*.config.*`,
`**/canary-lint/**`,
`**/*.{test,bench}.ts`.

Four custom oxlint plugins are authored in-repo under `package/oxlint-plugin/`
(`no-restricted-syntax`,
`prefer-readonly-parameter-type`,
`stylistic`,
`tsdoc`),
so third-party rule authoring is load-bearing for TypeScript.

Existing consumer tooling that a shared diagnostic schema would unlock for free:
`package/dev-script/task-util/src/oxlint-wrapper.ts` (281 lines),
`oxlint-augment.ts` (683),
`oxlint-guidance.ts` (140),
`oxlint-fix-loop.ts`.

## Adopted without asking

These have one defensible answer given the target,
 so they are in scope by
default rather than by decision (QGR):

- Severity levels per rule (`allow`/`warn`/`deny`),
   and rule options as
  `[severity, options]`.
- `categories` with the same seven names,
   plus `-A/-W/-D` left-to-right
  accumulation over rules and categories.
- `overrides` by glob with `files`/`excludeFiles`/`rules`,
  `ignorePatterns`,
  `extends`,
   nested config discovery,
  `--config`,
  `--disable-nested-config`.
- `--ignore-path`,
  `--ignore-pattern`,
  `--no-ignore`.
- `--quiet`,
  `--deny-warnings`,
  `--max-warnings`,
  `--silent`,
  `--no-error-on-unmatched-pattern`.
- `--format` for every format oxlint ships,
   JSON matching the schema above so
  the existing wrapper tooling works unchanged.
- `--threads` and parallel walking
  (`WalkBuilder::build_parallel()`,
   already a dependency).
- `--print-config`,
  `--rules`,
  `--init`,
  `--debug=files,timings`.
- Summary line and exit-code semantics.
- A rich default renderer with source snippet and span labels.

## Open decisions

Recorded as they are answered.

### D1. Scope of the four expensive subsystems

Custom plugin loading,
 autofix,
 LSP,
 semantic/type-aware analysis.
Status:
 ANSWERED 2026-07-25.
In scope:
 autofix pipeline,
 custom plugin loading,
 language server.
Out of scope in the original 2026-07-25 decision:
 semantic and type-aware analysis.
On 2026-10-04,
the user selected full semantic enforcement for the new unified-linter `rust/require-explicit-types` rule.
That accepted addition supersedes this exclusion for the new rule;
see `doc/planning/unified-linter.md`.
The original constraints and consequences remain recorded as history.
Consequence:
 the `Rule` trait must carry fix edits from the start,
rule registration needs a runtime seam rather than `all_rules()`,
and diagnostics must be servable incrementally over LSP.
No dependency on `ra_ap_hir`,
`ra_ap_ide`,
 or a rustc driver;
`ra_ap_syntax` stays the only parser.

### D2. Inline disable directives

`README.md:66` and `README.md:132` state there is deliberately no inline
suppression,
 and `AGENTS.md` MXL and RDC say never disable those two rules.
oxlint ships `--report-unused-disable-directives` as a headline feature,
 and
`AGENTS.md` LN3/LN6 do allow justified `oxlint-disable-next-line` for TypeScript.

Measured usage:
1258 `oxlint-disable` occurrences across 624 TypeScript files,
1179 of them carrying a `--` justification;
`package/config/oxlint/src/config-base.ts` sets
`reportUnusedDisableDirectives: 'warn'` alongside `denyWarnings: true`.

Status:
 ANSWERED 2026-07-25.
Chosen:
 build the full directive mechanism plus
`--report-unused-disable-directives` and its severity variant,
and have each rule declare its own suppressibility.
`max-lines` and `require-rustdoc` declare themselves non-suppressible,
 so a
directive aimed at either is itself reported.
No config key,
 override,
 or nested config can flip that declaration,
which keeps MXL,
 MXR and RDC literally true.

Settled alongside it,
 not asked:

- Directives are comments,
   not attributes.
  Verified:
  `#[allow(monochromatic::max_lines)]` fails on stable with
  `error[E0710]: unknown tool name`,
   and the suggested fix
  `#![register_tool(monochromatic)]` is nightly-only.
- A justification after `--` is mandatory;
  a directive without one is itself a violation.
  `AGENTS.md` LN3 already requires this and 94 percent of existing TypeScript
  directives comply.
  This is a superset of oxlint's behaviour,
   not parity with it.

Built:
 2026-10-18,
in `package/linter/monochromatic-lint`,
on branch `feat/lint-directives`
(engine commit `b564c7ad15`,
rule declarations `82d69a4036`;
merge `6808166dfd`).
The spelling is `monochromatic-lint-disable`,
`-enable`,
`-disable-line` and `-disable-next-line`;
the grammar,
the stricter readings taken where this document was silent,
the new `directive/*` finding identifiers
and the oxlint-spelled options
`--report-unused-disable-directives` and `--report-unused-disable-directives-severity`
are in the package README section "Inline directives".
Rust directives are `//` and `/* */` comments;
doc comments are not directives,
although the coverage ledger listed "any comment token".
Markdown directives are HTML comments,
and MDX has none.
The `README.md:66` and `README.md:132` lines cited above no longer exist in the package README;
its section "Inline directives" replaces them.

### D3. Config file format and location

`AGENTS.md` AD2 prefers TypeScript config when logic is needed,
 but a Rust
binary cannot execute TypeScript;
oxlint itself gates TS configs behind "requires running via Node.js".
oxlint gets away with TypeScript configs only because oxlint itself runs inside
Node:
`node_modules/oxlint/bin/oxlint` is a Node shim,
 and the checked-in
`oxlint.config.ts` is read natively by that Node process.
A standalone Rust binary has no such host.

In-repo precedent checked (WC2):
`file-enforcer.config.ts` already owns generated `CLAUDE.md`,
`AGENTS.md`
mirrors,
`mise.toml`,
 and `package/linter/rust/Cargo.toml`,
so a generated root config is an established pattern.
Root config formats already in use:
`*.config.ts` (`oxlint`,
`cli-git`,
`file-enforcer`,
`playwright`),
`*.toml` (`clippy`,
`bunfig`,
`mise`),
and `*.json` (`dprint`,
`node.config`,
`socket`,
`tsconfig`).

Status:
 ANSWERED 2026-07-25.
Chosen:
 TOML,
 read natively by the Rust binary,
 for now.
Follow-up tracked in issue #400,
"Revisit rust-linter config format after oxlint parity lands
(Pkl,
 JSONC,
 or rendered TypeScript)".

Pkl was considered and set aside for this pass.
Facts that shaped that:

- The repo deliberately retired Pkl in issue #357,
  "build(cli-git):
   retire hk and Pkl after policy parity",
  which removed root `hk.pkl`,
   the `apple/pkl` mise tool declaration,
   and
  `.idea/pklSettings.xml`.
  No `pkl` binary is installed on this machine and `mise.toml` declares none.
- Rust cannot embed a Pkl evaluator.
  `rpkl` (0.8.0,
   June 2026,
   34 percent doc coverage) spawns `pkl server` and
  exchanges MessagePack over stdio.
  Pkl native binaries cover macOS amd64/aarch64,
   Linux amd64/aarch64,
  Alpine amd64,
   and Windows amd64.
  So "config is Pkl" would have meant either a generated-JSON build step or a
  runtime dependency on the `pkl` executable.

Consequences:

- The binary stays standalone:
   no Node,
   no JVM,
   no `pkl`,
   no generated artifact.
- oxlint's published `configuration_schema.json` is not reusable as-is;
  the TOML tree has to express the same concepts
  (`rules`,
  `categories`,
  `overrides`,
  `extends`,
  `ignorePatterns`,
  `options`)
  in TOML's array-of-tables shape.
- Pattern rules benefit:
   TOML multi-line literal strings avoid the escaping
  that JSON would have forced.

### D4. Plugin mechanism

Status:
 ANSWERED 2026-07-25.
Chosen:
 compile-time Rust rule crates PLUS a declarative pattern language
evaluated from the config.
Neither path loads code at runtime.

Rejected,
 with the evidence that rejected each:

- dylint (`trailofbits/dylint`).
  Requires `#![feature(rustc_private)]` and pins each lint library to a specific
  nightly toolchain,
   then rebuilds the linted package with that same toolchain.
  This repo has no `rust-toolchain.toml`,
   and
  `package/linter/rust/Cargo.toml:56` records verification on stable 1.96.0.
- JS/TypeScript runtime plugins,
   the literal oxlint design.
  `node_modules/oxlint/bin/oxlint` is a three-line Node shim over napi bindings
  (`@oxlint/binding-*`),
   and `dist/plugins.js` loads JS plugins into that same
  Node process,
   passing the AST over shared `ArrayBuffer`/`Uint8Array` buffers.
  Copying it means an AST serialization protocol,
   a runtime lifecycle,
   and a
  Node dependency inside a standalone Rust binary.
  `doc/troubleshooting/oxlint-js-plugin-lazy-child-enomem.md` records this
  repo already hitting a 276-second run and `spawn ENOMEM` from that design.
- WASM plugins via wasmtime or extism.
  Same serialization boundary as the JS option without matching the repo's
  authoring language;
  `rg` for `wasmtime|extism|wasm-bindgen|wasm32` across the repo's TOML and JSON
  finds nothing outside `node_modules`.

Consequences:

- `plugins` in the config names compiled-in rule crates and namespaces their
  rule ids as `plugin/rule-name`,
   matching oxlint's `eslint(no-debugger)` code
  shape in JSON output.
- The config format must be able to express pattern rules,
   which raises the
  stakes on D3.
- `AGENTS.md` SYB applies to the pattern language:
  it must not become an invented comment-string DSL for relations the syntax
  tree already expresses.

### D6. Pattern-rule language

Status:
 ANSWERED 2026-07-25.
Chosen:
 patterns are written as Rust snippets with metavariables,
 matched
structurally against the `ra_ap_syntax` CST,
 in the style of ast-grep and
semgrep.
Metavariables are encoded as ordinary identifiers (`META_X`) so the snippet
parses as valid Rust;
ast-grep's `$X` spelling is not assumed to parse under `ra_ap_syntax` and was
not adopted.
A `fix` snippet in the same form supplies the rewrite,
 so pattern rules
participate in the autofix pipeline decided in D1.

Accepted shape:

```toml
[[pattern]]
id      = "no-unwrap"
match   = "META_X.unwrap()"
fix     = 'META_X.expect("TODO: explain")'
message = "unwrap() panics; name the invariant"

not-inside = """
#[test]
fn META_F() {}
"""
```

This is the SYB-compliant answer:
the pattern is written in the destination language rather than in an invented
DSL describing it.

#### Verified against the real parser

The recorded shape was tested,
 not assumed,
 in a throwaway worktree against
`ra_ap_syntax` 0.0.335 (`git worktree add`,
 removed afterwards,
 per IWT/THR).
Result:
 there is no single parse entry point that accepts all three snippet
shapes,
 so the format needs a fragment strategy.

- `META_X.unwrap()` through `SourceFile::parse`:
   5 errors,
   root `ERROR`.
- `META_X.unwrap()` through `ast::Expr::parse`:
   0 errors,
   root
  `METHOD_CALL_EXPR`.
- `META_X.expect("TODO: explain")` through `ast::Expr::parse`:
   0 errors,
  root `METHOD_CALL_EXPR`.
- `#[test] fn META_F() {}` through `SourceFile::parse`:
   0 errors,
  `SOURCE_FILE > FN > ATTR`.
- `#[test] fn META_F() {}` through `ast::Expr::parse`:
   1 error,
   root `ERROR`.
- `let META_A = 1;` fails BOTH entry points,
   because a statement is neither an
  item nor an expression.
  Wrapping it as `fn __probe() { let META_A = 1; }` parses with 0 errors,
  giving `STMT_LIST > LET_STMT`;
  the same wrapper recovers `EXPR_STMT` and `FOR_EXPR`.

Resolved design,
 verified rather than guessed:
the matcher auto-detects fragment kind by cascade,
trying item (`SourceFile::parse`),
 then expression (`ast::Expr::parse`),
then statement (wrapped in a synthetic function body),
and taking the first parse that reports zero errors.
No fragment-kind key is needed in the TOML.

Also confirmed by the same probe:
the ast-grep `$X` spelling does not work here.
`$X.unwrap()` yields 1 error through the expression entry point and 6 through
the file entry point,
 so the identifier-shaped `META_X` encoding is load
bearing,
 not stylistic.

Available for the text-predicate half where one is needed:
`package/forbidden-strings/core`,
the in-house linear-time engine with fuzz and bench harnesses,
already consumed by `package/forbidden-strings/cli` over a path dependency.
Using it rather than `regex` satisfies RG2 and the `no-regex` lint rule by construction.

### D5. Crate layout

Status:
 ANSWERED 2026-07-25.
Chosen:
 separate repo packages joined by path dependencies.

- A core library package under `package/rust-module/`:
  rule trait,
   lint context,
   config model,
   run engine,
   diagnostics,
   fix model.
- A pattern-matcher package beside it:
  snippet parsing,
   metavariable binding,
   structural matching,
   rewrite emission.
- `package/linter/rust` keeps the CLI binary,
   the built-in rules,
   and the
  language server.
- Rule plugin crates are their own packages.

Reproduces the shape the repo already uses:
`package/forbidden-strings/core` is a library package consumed by
`package/forbidden-strings/cli` over a `../../forbidden-strings/core`
path dependency.
No Cargo workspace is introduced,
 so the invariant in
`doc/planning/cargo-toml-file-enforcer.md` ("the no-workspace architecture is
deliberate",
 fifteen standalone `Cargo.toml` files) holds unchanged.

New crate names must be checked against `forbidden-strings.append.local.txt`
before use (NCD),
 and each new package needs its own `Cargo.toml`,
`mise.toml`,
`README.md` and `LICENSES/` tree (AP1,
 AP2,
 DM1,
 PKG).

### D7. Comment convention at this scale

Measured:
`package/linter/rust/src` is 625 code lines carrying 2501 comment
lines,
 a ratio of 4.00,
 because `.agents/skills/dum-dum-non-ts/SKILL.md`
requires a What/Why/TypeScript-pseudocode block above every
concept-introducing line,
 down to bare `Ok(...)` and `None` tails and every
`&` borrow.

Status:
 ANSWERED 2026-07-25.
Chosen:
 full density on the first occurrence of a concept in a file;
later occurrences of that same concept in that file take an ordinary short
comment.

This requires amending the skill rather than quietly deviating from it (GAP).
The amendment is a deliverable of this work,
 not a side effect:
edit `.agents/skills/dum-dum-non-ts/SKILL.md`,
 which is the source that
file-enforcer mirrors into `.claude/skills/` and `.factory/skills/`,
and define the scope of "first occurrence" explicitly as per file.

Blast radius surfaced and accepted 2026-07-25.
The skill fires on every non-TypeScript general-purpose language it lists,
so the amendment is GLOBAL,
 not scoped to this linter:
it changes the convention for the other fourteen Rust packages,
the Kotlin linter rule set,
 and the music-player Android port.
The user chose global amendment over a linter-only carve-out,
on the reasoning that re-explaining a concept at every occurrence adds volume
without adding comprehension in any language,
 not just here.
Existing full-density files stay valid because they exceed the requirement.

## Status: implementation complete

Every work item has landed.
Four packages,
 220 tests,
 clippy and the linter's own dogfood run clean on all
of them.

The `README.md` correction recorded above is done:
the file claimed `require-rustdoc` had no fixtures carve-out,
 which was wrong,
and it listed module paths that no longer exist.
Both are fixed in the rewrite.

## Status: design confirmed, implementation started

Shared understanding reached 2026-07-25.

### Landed

- Skill amendment (`.agents/skills/dum-dum-non-ts/SKILL.md`),
   commit `6ea3f0be1`.
  Comment density is now full-block on a concept's first occurrence per file.
- Work item 1,
   core crate extraction,
   commit `6d399e303`.
  `package/rust-module/rust-linter-core` holds `Rule`,
  `LintContext`,
  `Config`,
  and the diagnostic,
   span and fix model.
  `crate::config`,
  `crate::context`,
  `crate::diagnostic` and `crate::rule` stay
  valid paths in the CLI crate through re-export,
   so no rule or test was
  rewritten.
  Verified:
   22 existing linter tests pass unchanged,
   29 new core tests,
  clippy and dogfood clean on both packages.
- Column defect found and fixed immediately after.
  Both rules built their span from a line number,
   and a line-based span can
  only report column 1.
  That was invisible before,
   because the old `Diagnostic` had no column,
   but
  the new one promises oxlint's `labels[].span.column`.
  Added `LintContext::span_at_offset`,
   which resolves a real line and column and
  clamps length to the starting line;
  `require-rustdoc` already had the offset in scope and now uses it.
  `max-lines` keeps the line-based span,
   where column 1 is truthful.
  Two regression tests assert an indented item reports column 5,
   not 1.

- Work item 2,
   TOML config layer,
   commit `faf502e14`.
  `rust-linter.toml` with `rules`,
  `categories`,
  `options`,
  `ignore-patterns`,
  glob `overrides` carrying `files` and `exclude-files`,
  `extends` chains with
  cycle detection,
   and nested discovery.
  The two exemption predicates are deleted;
  their policy lives in a `default.toml` compiled into the binary.
  Both named forks resolved:
  `extends` is a FULL merge,
   unlike oxlint's rules-only one,
  and a nested config LAYERS over its ancestors rather than replacing them.
  Rules now declare a category;
  `max-lines` is pedantic,
   verified against oxlint's own documentation page.
  Verified:
   67 core and 26 linter tests,
   plus the real binary exercised on a
  throwaway tree confirming a config file turns `max-lines` off while
  `require-rustdoc` keeps reporting,
   and `--disable-nested-config` restores the
  default.
- Clippy debt cleared,
   commit `9459b3216`.
  The linter crate now passes its own `lint:clippy` gate,
   and gained the
  `format:clippy` task its sibling Rust packages already had.

### Correction to the record

`README.md` claims `require-rustdoc` has no fixtures carve-out.
It is wrong.
The predicate it replaced exempted `fixture/`,
`test-fixture/` and `invalid/`
for BOTH rules,
 and the integration test
`undocumented_fixture_in_place_is_exempt` depends on that.
`default.toml` reproduces the real behaviour and
`require_rustdoc_shares_the_same_exemptions` pins it.
The README is corrected in work item 12.

### Known debt, owned

None outstanding.
The `implicit_return` debt recorded here earlier is cleared.

## Further items adopted without asking

Settled after the questions above,
 because each has one defensible answer:

- The hardcoded exemption predicates in `src/config.rs`
  (`max_lines_exempt`,
  `missing_rustdoc_exempt`) are deleted and reexpressed as
  glob `overrides` in the TOML config,
  which is what oxlint does for the same rules in
  `package/config/oxlint/src/overrides.ts`.
- Rule options move into config.
  `--max` survives as a convenience override of the `max-lines` option so
  existing invocations and the documented mise task keep working.
- The language server is a `--lsp` flag on the same binary,
   as in oxlint,
  not a separate binary.
- Fix kinds match oxlint's three:
   safe fix,
   suggestion,
   dangerous.
- The default renderer targets the same visual output oxlint produces,
  whose JSON shape (`labels[].span` with `offset`/`length`/`line`/`column`)
  is miette's.
- Delivery is staged commits on `main` under auto-push (APG,
   GCE),
  each leaving `mise run //package/linter/rust:test` and the dogfood task green,
  rather than one large landing.
- The crate dogfoods itself,
   so every new `.rs` file obeys MXR's 300 code-line
  cap with no disable,
   and RDC rustdoc on every item.

## Deferred to implementation

Not decisions,
 but design work that lands with the code:

- The exact TOML tree expressing `rules`,
  `categories`,
  `overrides`,
  `extends`,
  `ignorePatterns` and `options`.
- Metavariable semantics:
   binding,
   reuse within a pattern,
   repetition,
  and whether `not-inside` takes one pattern or a list.
- Directive syntax and its rule-name grammar.
- Which of the ten output formats share a serializer.

Two forks that look mechanical but are not,
so they are named here rather than left to be discovered:

- `extends` semantics.
  `oxlint.config.ts:7` documents that oxlint's `extends` merges rules ONLY:
  `categories`,
  `env`,
  `ignorePatterns`,
  `overrides` and `plugins` are not
  inherited,
   which is exactly why the root config spreads `base` instead of
  extending it.
  So "adopt `extends`" has two readings,
   bug-compatible partial merge or full
  merge,
   and they differ in observable behaviour.
- Nested config composition.
  The root `lint:rust` task fans out per package,
   so a package-level
  `rust-linter.toml` and the root one must compose in a defined order.
  oxlint's answer is nested config discovery with `--disable-nested-config`;
  whether a package config replaces or layers over the root one needs deciding.

## Known holes in the parity claim

Named so the final claim is checkable rather than asserted:

- `settings`,
   oxlint's plugin-level configuration,
   is implemented.
  A `[settings.<plugin>]` table is readable by that plugin through
  `LinterConfig::settings_for`;
  the layer does not type it,
   because it cannot know what any plugin's settings
  mean.
- Per-plugin CLI toggles (`--import-plugin`,
  `--disable-unicorn-plugin` and
  friends) have no counterpart.
  The `plugins` config key does the same job from configuration:
  absent enables every compiled-in plugin,
   and a present list is the complete
  set,
   so `plugins = []` turns every rule off.
- **Output formats.**
   oxlint ships ten (`default`,
  `agent`,
  `json`,
  `github`,
  `gitlab`,
  `unix`,
  `checkstyle`,
  `junit`,
  `sarif`,
  `stylish`).
  This linter emits JSONL only,
   and has no `--format` flag at all.
  Decided by the user on 2026-07-25,
   mid-implementation:
  all ten had been written and verified against oxlint's real output when the
  scope was cut back to JSONL.
  The per-record field set is still oxlint's diagnostic object
  (`message`,
  `code`,
  `severity`,
  `causes`,
  `filename`,
  `labels[].span`,
  `related`,
   optional `url` and `help`),
   so a consumer reads one JSONL line
  exactly as it reads one element of oxlint's `diagnostics` array.
  Verified:
   the repository's own oxlint parser,
  `package-deprecated/mcp/nvim/src/oxlint-parse.ts`,
  reads these records unmodified.
- `--tsconfig` is not applicable to a Rust linter and is explicitly out.
- `--type-aware` and `--type-check` are out by decision D1.
- `--rules` produces no output at all in oxlint 1.75.0 as invoked here:
  both stdout and stderr are empty at exit 0.
  Parity therefore means implementing what the flag is documented to do,
  listing every registered rule,
   not reproducing what it currently does.

## Work breakdown

Per TSK,
 separate items with independently verifiable completion criteria,
rather than one umbrella task:

1.  Core crate extraction:
     rule trait carrying fixes,
     lint context,
     diagnostic
    model with spans and labels.
     Verified by the two existing rules passing
    unchanged against the new trait.
2.  TOML config layer:
    `rules`,
    `categories`,
    `options`,
    `ignorePatterns`,
    glob `overrides`,
    `extends`,
     nested discovery.
     Verified by the hardcoded
    exemptions in `src/config.rs` being deleted and reexpressed as overrides
    with identical behaviour on the existing fixtures.
3.  Severity and CLI accumulation:
    `-A`/`-W`/`-D` over rules and categories,
    `--quiet`,
    `--deny-warnings`,
    `--max-warnings`,
    `--silent`.
    Verified by exit-code tests per combination.
4.  Output:
     JSONL only,
     one record per line,
     each carrying oxlint's diagnostic
    field set.
     Verified by feeding the records to the repository's existing
    oxlint parser.
     Superseded the original plan of all ten formats.
5.  Directive engine:
     parsing,
     justification enforcement,
     per-rule
    suppressibility,
    `--report-unused-disable-directives` and its severity
    variant.
     Verified by a fixture where a directive targets a
    non-suppressible rule.
     Built 2026-10-18 (merge `6808166dfd`);
    that fixture is `a_listed_unsuppressible_rule_is_refused_and_reported`
    in `package/linter/monochromatic-lint/src/directive_apply_tests.rs`.
6.  Autofix pipeline:
     fix model,
     three fix kinds,
     applier,
     fixpoint iteration.
    Verified end to end through a pattern rule with a `fix` snippet.
7.  Pattern matcher crate:
     fragment cascade,
     metavariable binding,
     structural
    match,
     rewrite emission.
     Verified against the probe cases recorded above.
8.  Plugin registry:
     rule crates as packages,
     namespaced ids,
    `plugins` config
    key.
     Verified by moving one built-in rule out into its own package.
9.  Parallel runner and discovery flags:
    `--threads`,
    `build_parallel()`,
    `--ignore-path`,
    `--ignore-pattern`,
    `--no-ignore`,
    `--no-error-on-unmatched-pattern`,
    `--debug=files,timings`.
    Verified by timing a full 310-file run against the sequential baseline.
10. Language server:
    `--lsp`,
     incremental document sync,
     code actions from the
    fix model.
     Verified in an editor,
     not by piped test input (VB1).
11. Introspection:
    `--print-config`,
    `--rules`,
    `--init`.
    Verified by round-tripping `--init` output back through the config loader.
12. Documentation:
    `README.md` rewritten,
     including the stale module list at
    `README.md:211` which still names `src/rule/max_lines.rs` although the file
    has been `src/builtin/max_lines.rs` for some time.
