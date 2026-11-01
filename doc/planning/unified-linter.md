# Unified linter design

Design for `monochromatic-lint`,
one self-maintained linter written in Rust that replaces the Rust linter and the Markdown linter.
Every decision here comes from the design interview recorded in
[`doc/handover/unified-linter.md`](../handover/unified-linter.md),
which holds the quotes,
evidence,
rejected options,
and corrections.
Incumbent responsibilities and their owners:
[`doc/planning/unified-linter-coverage-ledger.md`](unified-linter-coverage-ledger.md).

Status:
confirmed by the user as the shared understanding on 2026-09-23.
On 2026-10-04,
the cli-git rewrite's implementing agent was also assigned ownership of the unified linter.
The user replaced HCL configuration with JSONC and explicitly required container tests,
mutation testing,
and fuzzing.
The historical `monochromatic-lint.config.hcl` is now `monochromatic-lint.config.jsonc` (2026-10-04).
Other settled behavior remains unchanged.
The user authorized implementation on 2026-10-04 with "Do it."
They subsequently selected a simple handwritten configuration merge for now,
removing the separate deepmerge port and fork-test prerequisite.

## Goal and scope

- One linter instead of two:
   "My objection is maintaining multiple linters when we can only maintain 1."
   (user)
- It replaces `package/linter/rust`,
   `package/rust-module/rust-linter-core`,
   `package/rust-module/rust-linter-pattern`,
   `package/rust-linter-plugin/builtin`,
   and `package/cli/markdown-lint`,
   and removes Node from Markdown linting.
- Languages:
   Rust,
   Markdown,
   and MDX.
  Kotlin stays on detekt;
   TypeScript stays on oxlint.
- Model:
   ESLint's shape,
   minus every knob without a consumer.
- Behavior parity:
   what each current rule reports and fixes stays exact,
   frozen by ported tests.
  Command-line flags,
   configuration,
   and output follow the new design;
   every consumer switches in the same change.

## Package layout

- One crate,
   `monochromatic-lint`,
   in `package/linter/monochromatic-lint`,
   with modules instead of plugin crates,
   because the repository's no-workspace rule gives every crate its own `target` directory
   and heavy dependencies would compile once per crate.
- Binary:
   `monochromatic-lint`.
- Configuration merging is a handwritten internal module over JSONC values.
  Keep the agreed ordered merge semantics,
  but do not build a generic deepmerge library,
  port customizers,
  or wait for the upstream fork.
- Dependencies:
  - `ra_ap_syntax`,
     pinned exactly,
     for Rust;
  - `satteri-pulldown-cmark` 0.6.3,
     `satteri-arena` 0.3.1,
     and `satteri-ast` 0.5.3,
     pinned with `=` and upgraded together
     ([`doc/audit/tech-unified-linter-markdown-parser-vet-2026-09-23.md`](../audit/tech-unified-linter-markdown-parser-vet-2026-09-23.md));
  - `monochromatic-jsonc-edit`,
     the repository's Rust JSONC package at `package/jsonc-edit/core-rust`,
     for configuration;
     no HCL dependency or compatibility parser;
  - no external configuration-merge dependency.
- Build-time C or assembly inside dependencies is allowed
   (`psm`,
   pulled in by Sätteri through `stacker`).
- Development builds skip fat LTO and `codegen-units = 1`;
   published release binaries keep the optimized profile.

## Files and languages

- The walker reads `.gitignore` files,
   `.ignore` files,
   `.git/info/exclude`,
   and the global gitignore,
   includes hidden directories except `.git`,
   and picks only `.rs`,
   `.md`,
   and `.mdx` files.
- ESLint's default ignores (`.git`,
   `node_modules`) are built in;
   repository-specific trees such as `package-paused` are ignored through the repository configuration.
- A file's extension chooses its language:
   `.rs` is Rust,
   `.md` is Markdown,
   `.mdx` is MDX.
- A file is linted only when at least one configuration block's `files` pattern matches it.

## Processors

Processors are always on and not configurable.
Each produces virtual files that configuration targets by path,
and every finding and fix maps back to the host file.

- Rust fences in Markdown and MDX:
   every fence whose info string starts with `rust` or `rs`,
   with any rustdoc attribute (`ignore`,
   `no_run`,
   `should_panic`,
   `compile_fail`,
   `edition20xx`).
  Virtual path:
   `<host>/<ordinal>.rs`,
   such as `doc/x.md/0.rs`.
- Rustdoc comments in Rust:
   each documented item's doc comment is one virtual Markdown file:
   a run of `///` lines,
   a run of `//!` lines,
   or one `/** */` or `/*! */` block.
  `#[doc = ...]` attributes are not processed.
  Virtual path:
   `<host>/<first line>.md`,
   such as `src/a.rs/42.md`.
- Doc tests:
   Rust fences inside rustdoc,
   two levels deep,
   including fences with no language,
   which rustdoc compiles as Rust.
  Virtual path:
   `<host>/<first line>.md/<ordinal>.rs`.
- Rust snippets are prepared the way rustdoc prepares doc tests:
   the `# ` hidden-line marker is stripped,
   leading `//!` lines stay at the top,
   and a fragment without `fn main` is wrapped in a synthetic one
   that is never reported or counted.
- Every Rust snippet needs a `//!` opening line,
   the same file-level `require-rustdoc` requirement as real files.
- Syntax errors are not findings,
   so `compile_fail` fences are linted for whatever parses.
- Fixes inside rustdoc re-add the comment prefix and indentation.
- TypeScript fences stay unlinted.

A glob such as `**/*.rs` matches virtual Rust files as well as real ones,
and `**/*.md` matches rustdoc virtual files;
configuration uses `ignores` such as `**/*.md/**` when a block must reach real files only.

## Configuration

- File name:
   `monochromatic-lint.config.jsonc`.
- Lookup:
   for each linted file,
   the nearest configuration file in its directory or an ancestor is used alone;
   files are never merged.
  `files` and `ignores` resolve against that file's directory.
- `--config <path>` skips lookup,
   and its patterns resolve against the working directory,
   so a temporary file outside the repository still matches repository paths.
- No built-in rule defaults:
   a rule runs only if the configuration turns it on.
- Shape:
   an ordered JSONC array of configuration objects,
   each with optional `name` and the existing `files`,
   `ignores`,
   and `rules` fields.
  Array order preserves the existing block-merge order;
   changing the syntax does not change lookup or merging.
- A block with only `ignores` removes matching files from linting entirely.
- Every other block needs `files`;
   it applies to a file when one `files` pattern matches and no `ignores` pattern does.
- A rule setting is always an object:
   `"rust/max-lines": { "severity": "error", "max": 300 }`,
   with severity `off`,
   `warn`,
   or `error`.
- For each file,
   the `rules` objects of every applying block merge together in block order through the internal merge function:
   objects merge key by key,
   arrays concatenate,
   and any other value,
   or a value whose type differs from the first,
   takes the last one.
  So a later `{ "severity": "warn" }` keeps an earlier `max`,
   and a later `exclude` list adds to an earlier one.
- JSONC comments and trailing commas are accepted.
  The configuration schema continues to reject `null` and duplicate keys,
   even though the generic JSONC value model can represent null.
  Variables,
   functions,
   imports,
   and expressions are not configuration features.
- Rule options have per-rule defaults,
   such as `max = 300` for `rust/max-lines`.

## Rules

Rust:

- `rust/max-lines`:
   code lines per file,
   blank and comment-only lines excluded,
   option `max` (default 300).
- `rust/require-rustdoc`:
   a doc comment on every documentable item,
   public and private,
   and on the file itself,
   with today's hardcoded cxx-qt carve-out.
- `rust/no-anonymous-functions`:
   added by the user on 2026-10-04;
   reject parsed closure expressions,
   including named-variable bindings,
   nested closures,
   and `move`,
   `async`,
   and `const` modifiers.
   Named functions,
   methods,
   function pointers,
   and async blocks remain allowed.
   No automatic extraction fix;
   captured state may require redesigning the callback interface.
   Macro token trees remain opaque to the current unexpanded syntax frontend.
   Rollout (user, 2026-10-05):
   `warn` everywhere at cutover,
   raised to `error` once the count reaches zero.
   The differential run of 2026-10-05 measured 1,215 findings in `.rs` files
   and 207 in Markdown snippets and doc tests.
   The user chose this over a per-file burn-down list,
   so nothing blocks a new closure until the rule is raised.

The user also requested explicit Rust type annotations on 2026-10-04,
including declaration types and generic call arguments.
The subsequent anonymous-function ban supersedes the inline closure in the motivating example;
a named callback is now required instead.
The user selected full semantic enforcement (option A) on 2026-10-04.
Add `rust/require-explicit-types` for declaration annotations and explicit generic arguments on resolved calls.
This supersedes the Rust parity plan's decision D1 restriction against semantic analysis for this rule.
Preserve `_` for unnameable function-item types such as `.map::<String, _>(user_name)`.
Method-name heuristics are not an acceptable substitute for callee resolution.
Standalone snippets can lack resolution context;
the implementation must expose that coverage boundary rather than report an unchecked call as verified.

Markdown (numbers in each rule's documentation):

- `markdown/heading-increment` (MD001).
- `markdown/commands-show-output` (MD014),
   fixable.
- `markdown/no-duplicate-heading` (MD024).
- `markdown/single-h1` (MD025).
- `markdown/no-trailing-punctuation` (MD026),
   fixable.
- `markdown/no-bare-urls` (MD034),
   fixable.
- `markdown/no-emphasis-as-heading` (MD036).
- `markdown/fenced-code-language` (MD040),
   fixable:
   inserts `text`,
   or `rust` inside rustdoc,
   where an unlabeled fence is a doc test.
- `markdown/link-image-reference-definitions` (MD053),
   fixable.
- `markdown/link-image-style` (MD054),
   fixable.
- `markdown/no-pipe-tables`,
   fixable to an HTML `<table>`.
- `markdown/semantic-line-breaks`,
   fixable,
   add-only.
- `markdown/lfs-image-url`,
   fixable,
   option `exclude` (gitignore-syntax patterns).

Processing findings from the core:

- a file that fails to parse,
   an MDX error,
   or a parser panic caught with `catch_unwind`;
- a fix refused because it would empty a non-empty file.

No rule can be suppressed:
there are no directive comments.

## Fixes

- Each finding carries at most one fix,
   applied all or nothing;
   it may hold several edits,
   and overlapping edits are rejected.
- `--fix` applies fixes in up to 10 passes,
   re-parsing each pass,
   stopping early on a circular fix,
   then lints once more.
- Edits are localized byte ranges;
   untouched bytes stay identical.
- Writes are atomic and keep the file mode.

## Command line

- Paths (default `.`),
   `--config`,
   `--fix`,
   `--stdin` with `--stdin-filename`,
   `--max-warnings`,
   `--quiet`,
   `--silent`,
   `--print-config`,
   `--init`,
   `--rules`,
   `--concurrency`,
   `--ignore-pattern`,
   `--ignore-path`,
   `--no-ignore`,
   `--no-error-on-unmatched-pattern`,
   `--debug`.
- `--stdin --fix` prints the fixed source to stdout and findings to stderr,
   as `markdown-lint` does today for cli-git.
- Exit codes:
   0 clean,
   1 findings that fail the run,
   2 setup or usage error.

## Output

- JSONL only,
   on stdout,
   one record per finding,
   in today's Rust linter shape (`message`,
   `code`,
   `severity`,
   `causes`,
   `filename`,
   `labels[].span`,
   `related`,
   optional `url` and `help`),
   with the rule id in `code`.
- Findings from virtual files report the host file and host positions.
- A clean run prints nothing.

## Distribution and consumers

- `monochromatic-lint` publishes through `.github/workflows/cargo-publish.yml`
   to crates.io with GitHub release assets in cargo-binstall's default naming,
   like `forbidden-strings`.
  The first publish claims the crate name and still waits for the user's explicit approval.
- Root `mise.toml` installs the binary as `"cargo:monochromatic-lint"` through cargo-binstall;
   `mise.lock` pins the version.
  Linter changes reach the repository through a release.
- Root tasks:
   one lint task in the `lint` aggregate and one `--fix` task in the `format` aggregate
   replace `lint:rust`,
   `lint:markdown`,
   `format:markdown`,
   and the 17 package `lint:rust` tasks.
  Rust linting thereby joins `mise run lint`,
   which it never did before.
- `package/git-policy/markdown-lint` keeps its policy and calls `monochromatic-lint`:
   it writes a one-rule configuration for `markdown/lfs-image-url`
   to a temporary file created with a proper temporary-file call,
   passes `--config`,
   `--stdin`,
   `--stdin-filename`,
   and `--fix`,
   and reads JSONL.

## Draft repository configuration

```jsonc
// monochromatic-lint.config.jsonc
[
  {
    "name": "ignored-trees",
    "ignores": [
      "package-paused/",
      "package-deprecated/",
      ".out-of-scope/"
    ]
  },
  {
    "name": "rust",
    "files": [
      "**/*.rs"
    ],
    "rules": {
      "rust/max-lines": {
        "severity": "error",
        "max": 300
      },
      "rust/require-rustdoc": {
        "severity": "error"
      },
      "rust/no-anonymous-functions": {
        "severity": "warn"
      }
    }
  },
  {
    "name": "rust-exemptions",
    "files": [
      "**/tests/**/*.rs",
      "**/*_tests.rs",
      "**/fuzz/**/*.rs",
      "**/*.fuzz/**/*.rs",
      "**/build.rs",
      "**/fixture/**/*.rs",
      "**/test-fixture/**/*.rs",
      "**/invalid/**/*.rs",
      "doc/audit/**/*.rs"
    ],
    "ignores": [
      "**/*.md/**",
      "**/*.mdx/**"
    ],
    "rules": {
      "rust/max-lines": {
        "severity": "off"
      },
      "rust/require-rustdoc": {
        "severity": "off"
      }
    }
  },
  {
    "name": "markdown",
    "files": [
      "**/*.md",
      "**/*.mdx"
    ],
    "rules": {
      "markdown/heading-increment": {
        "severity": "error"
      },
      "markdown/commands-show-output": {
        "severity": "error"
      },
      "markdown/no-duplicate-heading": {
        "severity": "error"
      },
      "markdown/single-h1": {
        "severity": "error"
      },
      "markdown/no-trailing-punctuation": {
        "severity": "error"
      },
      "markdown/no-bare-urls": {
        "severity": "error"
      },
      "markdown/no-emphasis-as-heading": {
        "severity": "error"
      },
      "markdown/fenced-code-language": {
        "severity": "error"
      },
      "markdown/link-image-reference-definitions": {
        "severity": "error"
      },
      "markdown/link-image-style": {
        "severity": "error"
      },
      "markdown/no-pipe-tables": {
        "severity": "error"
      },
      "markdown/semantic-line-breaks": {
        "severity": "error"
      },
      "markdown/lfs-image-url": {
        "severity": "error",
        "exclude": [
          "package/ssg/"
        ]
      }
    }
  },
  {
    "name": "rustdoc",
    "files": [
      "**/*.rs/*.md"
    ],
    "rules": {
      "markdown/single-h1": {
        "severity": "off"
      }
    }
  },
  {
    "name": "snippet-burn-down",
    "files": [
      "**/*.md/*.rs",
      "**/*.mdx/*.rs"
    ],
    "rules": {
      "rust/require-rustdoc": {
        "severity": "warn"
      },
      "rust/max-lines": {
        "severity": "warn"
      }
    }
  }
]
```

## Build order

1.  `monochromatic-lint` skeleton:
     command line,
     walker,
     JSONL output,
     exit codes,
     fix loop,
     processing findings.
2.  Rust language and its two rules,
     with the incumbent's user-visible tests ported as the specification.
3.  Markdown and MDX through Sätteri with explicit feature flags
     (never `DEFAULT_OPTIONS`),
     byte offsets with a 3-byte adjustment after a byte order mark,
     no astral correction,
     and the 13 rules with the 137 TypeScript tests ported.
4.  Processors and fix mapping.
5.  Configuration:
     JSONC parsing through the repository package,
     lookup,
     block matching.
    Rule-setting merge is handwritten inside this crate,
     as authorized on 2026-10-04.
    Preserve the agreed one-call semantics,
     using the repository's public JSON merge corpus as an independent regression source.
6.  Differential comparison against both incumbents over the whole repository,
     processors off,
     expecting identical findings and fixes
     except the findings issue #559 hides from the TypeScript linter.
7.  Cutover in one change,
     after the user approves the first publish:
     publish,
     install through mise,
     switch every consumer,
     update `AGENTS.md` MXR and RDC,
     delete the five incumbent packages and their registrations
     (pnpr configuration,
     file-enforcer configuration,
     `cargo-publish.yml`),
     auto-fix the rustdoc blocks,
     and add the snippet burn-down block.
8.  Fix the 905 snippet findings,
     then delete the burn-down block.

## Required verification

Container tests,
mutation testing,
and fuzzing are release acceptance requirements,
not optional follow-up work.
The combined plan specifies their concrete targets and gates:
[`cli-git-rust-implementation.md`](cli-git-rust-implementation.md#verification-gates).

- Container tests exercise the installed binary,
  file walking,
  configuration lookup and merges,
  stdin fixing,
  atomic writes,
  process shutdown,
  and cli-git's selected-rule integration in disposable fixtures.
- Mutation tests target configuration precedence,
  rule predicates,
  fix safety,
  virtual-to-host position mapping,
  and error/exit handling.
  Surviving non-equivalent mutants require stronger tests;
  exclusions and equivalent mutants require evidence.
- Fuzzing covers JSONC schema handling,
  merge value shapes,
  Markdown/MDX and Rust snippets,
  Unicode and BOM positions,
  nested processors,
  and overlapping or adversarial edits.
  Minimized failures become deterministic regressions.
- Resource limits apply to test and fuzz subprocesses.
  They do not silently adopt a production parse-time budget that the earlier design deferred.

## Deferred until a consumer exists

- `extends` and sharing between configuration files.
- Categories and built-in shareable configurations.
- A language server.
- Declarative pattern rules,
   the suggestion and dangerous fix levels,
   and their flags.
- Directive comments of any kind,
   including rule-setting comments.
- Output formats other than JSONL.
- `--rule`,
   `--format`,
   `--lsp`,
   and a flag for running one rule.
- A generic deepmerge package and customizers.
- A parse time budget.

## Not replicated from ESLint

`language`,
`languageOptions`,
`settings`,
`plugins`,
`processor`,
`basePath`,
per-block `extends`,
`linterOptions`,
numeric severities,
runtime plugins,
shareable configuration packages,
a public library API,
bulk suppressions,
caching,
`--fix-type`,
`--fix-dry-run`,
`--no-config-lookup`,
`--inspect-config`,
`--env-info`,
`--mcp`,
`--flag`,
`--stats`,
`--exit-on-fatal-error`,
`--output-file`,
color flags,
rule `meta.type`,
message-ID placeholders,
JSON Schema option validation,
deprecation metadata,
selectors,
code-path and scope analysis,
`<!-- eslint-skip -->`,
comment injection into fences,
and fence `filename=` metadata.

## Incumbent defects not carried over

- A multi-edit fix applied partly.
- Exemptions matched relative to the working directory instead of the configuration.
- The language server ignoring directives and ignore patterns.
- `run_cli_from_env` dropping the order of `-A`,
   `-W`,
   and `-D`.
- The advertised but unbuilt `--debug=timings`.
- The double offset correction after astral characters (#559).

## Risks

- Sätteri:
   one dominant maintainer,
   breaking minor releases about every three weeks,
   two private vulnerability reports unacknowledged until bruits/satteri#306,
   invalid MDX that can take minutes to parse,
   and one release panic site.
  The parity harness gates every upgrade.
- Build cost:
   about 28.5 s for a clean dev build of the Markdown side alone.
- Handwritten merging must preserve all-input type-mismatch semantics;
   a left-fold pairwise merge is not equivalent.
- Rust linting joining `mise run lint` exposes the Rust files no package task covered,
   handled by the `*.fuzz` and `doc/audit` exemptions.
- `mise run lint` is already red on existing Markdown debt (#294);
   the migration does not change that.

## Licences

- `monochromatic-lint`:
   LGPL-3.0-or-later,
   carrying markdownlint's MIT notice for the rule logic ported from it,
   which the TypeScript package claims to record but does not.
