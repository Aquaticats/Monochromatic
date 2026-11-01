# ESLint model and Sätteri Rust API for the unified linter

Read-only research from 2026-09-23 for [`doc/handover/unified-linter.md`](../handover/unified-linter.md).
Clones lived under `~/temp/agent/<name>-2026-09-23`;
`path:line` citations are relative to the named repository.
Versions read:
ESLint v10.11.0 (tag `3c0b7c6e`,
released 2026-09-18),
`eslint/rewrite` `main` `e05a298`,
`@eslint/markdown` 8.0.3,
`@eslint/json` 2.1.0,
`@eslint/css` 2.0.0,
`@eslint-community/eslint-plugin-eslint-comments` 4.8.1,
`bruits/satteri` `main` with crates.io releases of 2026-08-19.

## ESLint flat configuration

- Object keys:
   `name` and `basePath` (metadata,
   `lib/config/flat-config-array.js:24`),
   `files`,
   `ignores`,
   `extends`,
   `language` (`"plugin/lang"`),
   `languageOptions`,
   `linterOptions`,
   `processor`,
   `plugins`,
   `rules`,
   `settings`
   (`docs/src/use/configure/configuration-files.md:66-87`).
  Legacy eslintrc keys such as `overrides` and `root` throw (`lib/config/flat-config-schema.js:548-568`).
- The built-in default matches only JavaScript files (`lib/config/default-config.js:18-29`),
   so other languages need `files`.
- An object whose only key is `ignores` acts globally;
   defaults `**/node_modules/` and `.git/` come first.
- `extends` (through `defineConfig`):
   `files` become the cross product of base and extension patterns,
   `ignores` concatenate,
   extensions come before the base (`eslint/rewrite` `packages/config-helpers/src/define-config.js:323-490`).
- Matching objects merge in array order,
   later winning (`packages/config-array/src/config-array.js:1232-1241`);
   `rules` merge by id,
   and a severity-only entry keeps earlier options (`flat-config-schema.js:453-500`).

## Configuration lookup

- ESLint v10 searches upward from each linted file's directory;
   the v9 flag for this was removed (`docs/src/use/migrate-to-10.0.0.md:100-113`).
- The nearest configuration file is used alone;
   files are never merged (`lib/config/config-loader.js:533-561`,
   `385-395`).
- `--config` skips the search;
   `--no-config-lookup` uses no file.

## Language plugins

- `Language`:
   `fileType`,
   `lineStart`,
   `columnStart`,
   `nodeTypeKey`,
   `validateLanguageOptions()`,
   `parse()` returning `{ok, ast}` or `{ok: false, errors}` without throwing,
   and `createSourceCode()`
   (`eslint/rewrite` `packages/core/src/types.ts:1150-1230`).
- `SourceCode`:
   `ast`,
   `text`,
   `getLoc()`,
   `getRange()`,
   `traverse()`,
   and optional `getDisableDirectives()`,
   `getInlineConfigNodes()`,
   `applyInlineConfig()`,
   `finalize()` (`types.ts:1354-1450`).
- Rules may declare `meta.languages`;
   a rule configured
for a language it does not declare throws `rule-unsupported-language` (`lib/config/config.js:262-300`).
- `@eslint/markdown` finds directives in mdast `html` nodes
   matching `<!--\s*eslint(?:-enable|-disable(?:(?:-next)?-line)?)?(?:\s|-->)`
   (`src/language/markdown-source-code.js:34-36`,
   `72-108`,
   `177-250`);
   MDX `{/* */}` comments are not recognized.

## Inline directives

- `eslint-disable` and `eslint-enable` are ranges;
   `eslint-disable-line` covers the comment's line;
   `eslint-disable-next-line` covers the line after the comment's end
   (`docs/src/use/configure/rules.md:198-335`;
   `lib/linter/apply-disable-directives.js:505-530`).
- A description follows two or more dashes surrounded by whitespace
   (`eslint/rewrite` `packages/plugin-kit/src/config-comment-parser.js:209-220`).
- Inline rule configuration `/* eslint rule: "error" */` applies to the whole file;
   configuring the same rule twice is an error (`lib/linter/linter.js:1131`).
- `linterOptions.noInlineConfig` (default false),
   `reportUnusedDisableDirectives` (default `"warn"`,
   each report carrying a removal fix),
   `reportUnusedInlineConfigs` (default `"off"`).
- ESLint core can neither require a description nor make a rule non-suppressible;
   eslint/eslint#16431 was closed as not planned.
  `@eslint-community/eslint-plugin-eslint-comments` supplies `require-description` and `no-restricted-disable`,
   and works on other languages through `getDisableDirectives()`.

## Bulk suppressions

- `eslint-suppressions.json`,
   keyed by working-directory-relative path and rule id,
   holding a violation count;
   only `error` severity counts
   (`lib/services/suppressions-service.js:24`,
   `239-272`).
- At or under the stored count,
   the rule's messages for that file are suppressed;
   over it,
   all of them report.
- `--suppress-all`,
   `--suppress-rule`,
   `--prune-suppressions`;
   unused suppressions exit 2 unless `--pass-on-unpruned-suppressions`.

## Rules and fixes

- `meta`:
   `type` (`problem`,
   `suggestion`,
   `layout`),
   `docs`,
   `messages`,
   `fixable` (`code` or `whitespace`,
   required to emit fixes),
   `hasSuggestions`,
   `schema`,
   `defaultOptions`,
   `deprecated`,
   `languages`.
- `context.report({messageId, node|loc, data, fix, suggest})`;
   fixer methods insert,
   remove,
   and replace text or ranges.
- Several edits in one report merge into one atomic fix;
   overlapping edits in one report throw (`lib/linter/file-report.js:253-289`).
- Suggestions are never applied by `--fix`.
- Up to 10 fix passes,
   stopping early on a circular fix (`lib/linter/linter.js:43`,
   `1520-1660`);
   a fix touching the previous one waits a pass (`lib/linter/source-code-fixer.js:56-150`).
- `--fix-type` selects `problem`,
   `suggestion`,
   `layout`,
   or `directive` by rule `meta.type`.

## Processors

- `preprocess(text, filename)` returns blocks;
   `postprocess(messages, filename)` maps positions and fixes back;
   `supportsAutofix` defaults to false
   (`docs/src/extend/custom-processors.md:17-133`).
- Virtual block paths are `<file>/<index>_<block filename>` (`lib/services/processor-service.js:77`),
   which configuration targets with globs such as `**/*.md/*.js`.
- `@eslint/markdown`'s processor keeps a per-line range map for indentation and blockquote prefixes,
   injects preceding `<!-- eslint ... -->` comments into the block,
   and honours `<!-- eslint-skip -->` (`src/processor.js:140-450`).

## Formatters and command line

- Built-in formatters:
   `stylish` (default),
   `json`,
   `json-with-metadata`,
   `html`.
- `LintResult`:
   `filePath`,
   `messages`,
   `suppressedMessages`,
   error,
   fatal,
   warning,
   and fixable counts,
   `output`,
   `source`.
- `LintMessage`:
   `ruleId`,
   `severity` 1 or 2,
   `fatal`,
   `message`,
   `messageId`,
   1-based `line`,
   `column`,
   `endLine`,
   `endColumn`,
   `fix: {range, text}`,
   `suggestions`.
  Ranges are JavaScript string indices,
   so UTF-16 code units (inference from JavaScript semantics;
   the docs do not say).
- `--stdin` with `--stdin-filename`;
   `--fix` writes files and errors on piped input;
   `--fix-dry-run` writes nothing and puts `output` in JSON.
- `--rule 'id: [error, opts]'` merges over file configuration;
   `--no-config-lookup` plus `--rule` runs only command-line rules.
- `--max-warnings`,
   `--quiet`,
   `--cache`,
   `--print-config`,
   `--init`,
   `--report-unused-disable-directives(-severity)`,
   suppression flags.
- Exit codes:
   0 clean,
   1 errors or too many warnings,
   2 configuration or internal error.

## `@eslint/markdown` and MDX

- Languages `markdown/commonmark` and `markdown/gfm`;
   parser `mdast-util-from-markdown`;
   `languageOptions.frontmatter` is `false`,
   `"yaml"`,
   `"toml"`,
   or `"json"`.
- No MDX support;
   eslint/markdown#316 asking for it is open.
- `eslint-mdx` is a JavaScript parser plus a remark-lint bridge,
   not a language plugin.

## Sätteri Rust API

- Crates (MIT):
   `satteri-pulldown-cmark` 0.6.3,
   `satteri-arena` 0.3.1,
   `satteri-ast` 0.5.3,
   `satteri-plugin-api` 0.5.3,
   `satteri-mdxjs` 0.3.13,
   and others;
   npm `satteri` 0.10.5,
   all released 2026-08-19.
- `satteri_pulldown_cmark::parse(source, options)` returns an arena and a list of MDX errors,
   with no Node involved (`crates/satteri-pulldown-cmark/src/arena_build.rs:74`);
   a cargo probe against the crates.io releases ran it.
- The tree is a flat arena of mdast-named nodes,
   not an mdast struct tree (`crates/satteri-arena/src/node.rs:66-86`);
   typed views exist for a few node kinds only.
- GFM tables,
   footnotes,
   strikethrough,
   task lists,
   and autolink literals;
   YAML and TOML frontmatter;
   MDX nodes behind the `mdx` feature,
   which pulls in oxc 0.121.
- Rust `DEFAULT_OPTIONS` enable math and omit TOML (`arena_build.rs:51-59`),
   unlike the npm default the repository uses,
   so a port must set the flags explicitly.
- Offsets are UTF-8 bytes;
   lines are 1-based;
   columns are 1-based UTF-16 code units (`crates/satteri-arena/src/line_index.rs`).
  The npm binding converts offsets to UTF-16,
   since upstream commit `6696c1c` (#174),
   released in npm 0.10.0.
- The npm tree is materialized from the same Rust arena (`crates/satteri-napi-binding/src/lib.rs:472-487`).
- Health:
   repository created 2026-03-22,
   336 commits,
   249 by one author;
   pre-1.0 with minor bumps that change behavior;
   no stated Rust API stability policy;
   open issue #306 says private vulnerability reports are not reviewed.
  Exact `=` pins upgraded together are needed.
- Fallbacks:
   `markdown` (markdown-rs) 1.0.0 builds a real mdast with MDX and byte offsets,
   inactive for about 17 months;
   comrak and pulldown-cmark have no MDX.

## Consequence found in the incumbent

The repository's `parse.ts` still converts Sätteri offsets as if they were code points,
so offsets after an astral character are wrong by one per character.
Tracked as <https://github.com/Aquaticat/Monochromatic/issues/559>.
