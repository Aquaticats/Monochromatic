# Handover: no-pipe-tables markdownlint rule

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `@monochromatic-dev/module-or-throw` is now `@monochromatic-dev/or-throw-core`;
> `package/module/test` is now `package/test-harness/core`;
> `@monochromatic-dev/module-test` is now `@monochromatic-dev/test-harness-core`.

Implementing a markdownlint rule that bans Markdown pipe tables and autofixes them to HTML.
 Approved
plan:
 `/home/user/.claude/plans/wire-in-a-markdownlint-vectorized-kazoo.md` (the plan predates the
TypeScript pivot below;
 the rule behavior it describes still holds,
 the authoring/build/test approach
changed).

## Decisions locked in

- Ban Markdown pipe tables only;
   HTML `<table>` is acceptable but not preferred (pipe tables force each
  row onto one line,
   fighting the semantic-line-break rule;
   HTML allows multi-line cells).
- The rule reports and autofixes pipe tables to HTML `<table>` (one element per line,
   alignment
  preserved via `align` attr).
   Lists/headings are the preferred form;
   HTML is the mechanical fallback.
- The rule is authored in TypeScript under `src/` and built with tsdown,
   not hand-written `.mjs`.
   The
  rule files grew complex enough that TypeScript reads better.
   (This reverses an earlier `.mjs`
  decision;
   the user looked at the built-out files and pivoted.
  )
- Tests use the `@monochromatic-dev/module-test` harness and import the BUILT dist (the exact module
  markdownlint-cli2 loads),
   not the source.
   The user mandated testing against the built artifact.
- Housed under `package/markdownlint-plugins/` (a lint rule is a plugin,
   not config;
   precedent
  `package/oxlint-plugin/*`).
- `AGENTS.md` line 529 ("No tables;
   use headings or lists instead") stays UNCHANGED on purpose;
   the
  rationale belongs in `doc/philosophy/agents.md` (do not advertise the HTML escape hatch in the
  terse rule).
   This doc edit is still pending (task 5 below).

## Architecture (current)

- `src/token.ts`:
   `ReadonlyToken`,
   a fully-readonly structural view of a micromark token with
  `type: string`.
   This one type fixes two problems at once:
   the gfm-table `TokenTypeMap` augmentation
  does not resolve here,
   so `keyof TokenTypeMap` excludes `'table'` and a literal `token.type ===
  'table'` reports TS2367;
   and `prefer-readonly-parameter-types` flags markdownlint's deeply-mutable
  `MicromarkToken`.
   `MicromarkToken` is structurally assignable to `ReadonlyToken`,
   so tokens flow in
  without a cast.
- `src/to-html-table.ts`:
   pure token-to-HTML-lines transform.
   Uses `childrenOfType` (returns an array,
  never `undefined`) plus for-of iteration so absence is an empty result,
   never a written
  `T | undefined` (banned by `no-restricted-syntax/no-nullish-union`).
   Default export `toHtmlTable`.
- `src/no-pipe-tables.ts`:
   the rule.
   `collectTables` does an iterative deep walk (explicit stack,
   no
  recursion).
   `reportTable` emits the replace-first-line fix plus per-line deletes.
   Top-level-only fix;
  blockquote/indented tables report without a fix.
- `src/index.ts`:
   `export default [noPipeTables]` (the `customRules` entry) and re-exports
  `toHtmlTable` as a named export so the test can exercise the transform against the built dist.
- Build:
   tsdown `.node.ts` config (`tsdown.node.config.ts` re-exports
  `@monochromatic-dev/config-tsdown/.node.ts`) bundles `src/index.ts` to
  `dist/final/node/index.mjs` (+ `.d.mts`).
   The bundle is self-contained:
   type-only `markdownlint`
  imports are erased,
   `@monochromatic-dev/module-or-throw` is inlined,
   zero runtime deps.
- `tsconfig.json` extends `@monochromatic-dev/config-typescript` (neutral,
   this is a Node rule).
- `mise.toml`:
   `build`/`build:js`/`build:js:node`,
   watch variants,
   `lint`/`lint:oxlint`/`lint:types`,
  `test:unit` (standard glob template),
   and `buildAndTest` (build then test) mirroring `module/test`.

## Done (committed)

- `7f4b5e0f` refactor:
   the TypeScript package (src,
   tsconfig,
   tsdown config,
   `package.json`,
   `mise.toml`,
  README) replacing the old `.mjs`;
   pnpm-lock deps.
- `7eecee77` style:
   incidental formatter output in two unrelated files (user approved committing).
- `7b9c72f3` docs:
   prior handover rewrite for the TS architecture.
- `d172b7b4` feat(markdownlint):
   wired the rule into `.markdownlint-cli2.jsonc` (customRules -> the
  built dist,
   `no-pipe-tables: true`,
   `globs: ["**/*.md"]`,
   `gitignore: true`,
   extended `ignores`) and
  `mise.toml` (no-arg `lint:markdownlint` is now plain `markdownlint-cli2`;
   added `format:markdownlint`
  wired into the root `format` task).
- Earlier WIP `.mjs` package (`23f59a11`) is superseded by the refactor (its `.mjs` files deleted).
- Package gate is green:
   `build`;
   `lint:types` exit 0;
   `lint:oxlint` 0 warnings / 0 errors;
  `test:unit` 6/6 against the built dist;
   `buildAndTest` works.
- Issue #231 filed:
   the rule needs a built dist,
   so a fresh clone (no `dist/`) breaks markdown linting;
  no build preamble was wired into the lint task on purpose (rejected workaround).
   The issue asks for a
  better solution (e.g. run markdownlint-cli2 under Bun so `customRules` can point at source),
   open.
- Task 3 fixture verification PASSED through the real `mise run lint:markdownlint`:
   the dist `import()`s
  under Node via the real config and flags a pipe table at its first line (continuation lines reported
  too);
   HTML and prose fixtures pass;
   `format:markdownlint --` converts the pipe table to a well-formed
  `<table>` (alignment `:---`/`--:`/`:--:` -> left/right/center;
   escaped `\|` -> `|`);
   re-lint is clean
  (idempotent).
   Fixtures were deleted.

## Verified facts (do not re-investigate)

- Pipe-table micromark token type is `table`.
   Tree:
   `table` -> `tableHead` (header `tableRow` +
  `tableDelimiterRow`) and `tableBody` (body `tableRow`s).
   Cells `tableHeader`/`tableData`/
  `tableDelimiter`;
   cell text is the `tableContent` child's `.text`;
   `\|` is preserved (the transform
  unescapes it).
   Alignment from delimiter cell leading/trailing `:`.
- HTML `<table>` parses as `htmlFlow`,
   never a `table` token,
   so it is never flagged;
   the autofix is
  idempotent.
- markdownlint applies custom-rule `fixInfo` under `--fix`.
   `applyFix` is line-scoped
  (`{lineNumber, editColumn, deleteCount, insertText}`;
   `insertText` may contain `\n`;
  `deleteCount: -1` deletes a whole line).
   Multi-line replacement needs one replace-first-line fix plus
  per-line delete fixes (one report per table line until `--fix`).
   An absent fix normalizes to `null`.
- markdownlint type exports (`markdownlint`):
   `Rule`,
   `RuleParams`,
   `RuleOnError`,
  `RuleOnErrorFixInfo`,
   `MicromarkToken`,
   `LintError`;
   `applyFixes` is a value export.
   `lint` is from
  `markdownlint/sync`.
   `RuleParams.lines` is `readonly string[]`;
   `.parsers.micromark.tokens` is
  `MicromarkToken[]`.
- `Rule.parser` is `"markdownit" | "micromark" | "none"`;
   micromark is a choice,
   not mandated.
   We use
  micromark for its structured token tree.
   markdownlint's own table rules match via
  `filterByTypes([...])` (a `string[]`),
   avoiding the `=== 'table'` literal-overlap;
   our
  `ReadonlyToken` (`type: string`) achieves the same.
- `customRules` entries load via `await import(fileURL)` relative to the config file dir.
   Reference by
  relative path.
   The path must be the BUILT artifact (`dist/final/node/index.mjs`),
   since
  markdownlint-cli2 runs under Node and cannot import `.ts`.
- markdownlint-cli2 runtime is 0.22.1 bundling markdownlint 0.40.0.
- markdownlint-cli2 APPENDS the config `globs` to CLI file args (`markdownlint-cli2.mjs:415-418`,
  gated on `!noGlobs`).
   So an explicit single-file run ALSO drags in the whole `**/*.md` tree (this was
  the 1m+ slowness the user hit).
   The fix,
   already applied:
   the explicit-args branches of both
  `lint:markdownlint` and `format:markdownlint` pass `--no-globs`,
   so they lint/fix only the given
  files.
   The no-arg branches use config globs (whole tree) deliberately.
- `gitignore: true` is doing the real pruning of NESTED `node_modules`/`dist`.
   The `ignores` patterns
  (`node_modules/**`,
   `dist/**`) are anchored at the start,
   so they match only TOP-LEVEL dirs,
   not
  `package/*/node_modules` or `package/*/dist`.
   Do not drop `gitignore: true` without first adding
  `**/node_modules/**`,
   `**/dist/**`,
   etc. It may also be the dominant cost of the whole-tree run;
  measure before changing.

## Remaining work

Build the rule before any whole-tree `lint:markdownlint`/`format:markdownlint` run (dist is gitignored,
no build preamble;
 see #231):
 `mise run //package/markdownlint-plugins/no-pipe-tables:build`.

### Task 4: migrate existing pipe tables -- BLOCKED on a scope decision

CRITICAL,
 not yet resolved:
 expanding `globs` to `**/*.md` subjects EVERY previously-unlinted `.md`
(hundreds across `package/**` and `doc/**`) to ALL the enabled built-in rules (MD001,
 MD014,
 MD024,
MD025,
 MD026,
 MD034,
 MD036,
 MD040,
 MD052,
 MD053,
 MD054),
 not just `no-pipe-tables`.
 Before this change
only the 5 top-level `.md` were linted (`markdownlint-cli2 .` -> top-level `*.{md,markdown}`).
 So
"whole-tree zero errors" is potentially a large pre-existing-doc cleanup unrelated to pipe tables.

Next step is to MEASURE,
 then likely ASK the user (non-measurable scope decision):

1. Build,
    then run the whole-tree lint ONCE,
    capturing to a file (it is slow,
    ~1m+;
    the user rejected
   re-running it):
    `mise run lint:markdownlint > /tmp/md-lint-full.txt 2>&1`.
    Count errors by rule:
   `rg " error " /tmp/md-lint-full.txt | sed -E 's/.* error ([a-zA-Z0-9-]+).*/\\1/' | sort | uniq -c | sort -rn`.
2. If dominated by `no-pipe-tables`,
    just migrate:
    `mise run format:markdownlint` (whole-tree --fix)
   converts pipe tables to HTML.
    If there is a large body of OTHER-rule violations,
    surface a scope
   question:
    fix all now,
    or fix only pipe tables this round and defer/triage the rest (e.g. file a
   tracking issue;
    the other rules were never enforced on these files before).
3. Migration review:
    `git diff` (well-formed tables,
    alignment,
    inline-Markdown cells);
    render a
   converted doc with `agent-browser` to confirm the HTML table renders;
    hand-convert any
   blockquote/indented tables the rule reported without a fix.
4. Confirm `no-pipe-tables` errors are zero tree-wide;
    note the runtime;
    decide if `gitignore: true` is
   the slow part (see verified-facts note before touching it).

### Task 5: docs

- `doc/philosophy/agents.md`:
   add a `####` subsection under the "Relocated rule rationale" area titled
  e.g. "Documentation standards:
   why \"No tables\" stays terse despite HTML being tolerated".
   Content
  drafted in the plan file (section 5).
- `doc/troubleshooting/markdownlint-cli2.md`:
   the "subdirectory coverage out of scope" note is now
  resolved;
   record how coverage was expanded and the measured runtime.

## Commit/verification conventions

- Commit eagerly per logical unit,
   scoped pathspecs (no `git add -A`/`.`),
   on `main`.
- This handover doc uses lists,
   not tables (dogfooding the rule);
   keep it that way.
- /tmp has throwaway introspection scripts;
   leave them.
   Token facts above mean no re-introspection.
