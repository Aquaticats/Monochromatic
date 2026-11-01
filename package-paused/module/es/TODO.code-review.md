# Code review: packages/module/es

> Stale as of 2026-05-13: this review predates the current `src/types/**`
> taxonomy cleanup and the removal of `src/deprecated/`. Treat specific path
> inventories and counts below as historical until rechecked against current code.

**Date**: 2026-02-17

Summary: A functional-programming utility library
with a deeply nested directory hierarchy encoding type information in path names.
The code is generally well-documented and follows the project's conventions
(TSDoc, named functions, trailing commas, region markers).
The main concerns are: excessive `as any` casts, silent catch blocks in sinks,
several files exceeding the 100-line limit without justification, missing TSDoc on some functions,
a non-null assertion in `merge`, and significant testing gaps across source files.

## Findings

### BLOCKER

- `t object/t logger/f/t never/r s/p p/index.ts:100`:
   Uses `.catch()` callback pattern which is banned by project rules ("No `.then()`, `.catch()`, `.finally()`").
  `result.catch(() => { ... })` should be restructured to use `async`/`await` with `try`/`catch`.
  Suggested action: make `createMethod` return an `async` function or fire-and-forget with a helper
  that uses `try`/`catch`.

### WARNING

- `t boolean/f/t unknown/r s/p p/index.ts` (entire file): Missing TSDoc on exported function `$`.
  The function is a one-liner wrapping `Boolean()` but per project rules all declarations must have TSDoc.
  Suggested action: add TSDoc with `@param`, `@returns`, and `@example`.

- `t object/t array/f/t iterable/r s/p p/index.ts` (entire file): Missing TSDoc on exported function `$`.
  Suggested action: add TSDoc explaining the sync iterable-to-array conversion.

- `t object/t array/f/t iterable/p p/index.ts` (entire file): Missing TSDoc on exported function `$`.
  Suggested action: add TSDoc explaining the async iterable-to-array conversion.

- Multiple files exceed 100 lines without a justification comment at the top:
  - `t object/t jsonc/f/t string hasQuotedSyntax jsonc/r s/p n/customParsers.ts` (331 lines):
     has a comment about mutual recursion but not an explicit justification for exceeding the limit
  - `path/index.ts` (241 lines): no justification comment
  - `t object/t jsonc/f/t string hasQuotedSyntax jsonc/r s/p n/fastPath.ts` (239 lines): no justification comment
  - `t object/t jsonc/f/t string hasQuotedSyntax jsonc/r s/p n/index.ts` (164 lines)
  - `t string/t html/f/t string jsx/r s/p n/index.ts` (157 lines)
  - `t string/f/t any/export/r s/p p/index.ts` (156 lines)
  - `t object/t jsonc/f/t string hasQuotedSyntax jsonc/r s/p n/customParsers.startsWithComment.ts` (131 lines)
  - `t object/t htmlElement/f/t string jsx/r s/p n/index.ts` (131 lines)
  - `t object/t logger/f/t never/r s/p p/index.ts` (130 lines)
  - `t object/t array/t p string/t typeof/f/t unknown/r s/p p/index.ts` (125 lines)
  - `t object/t jsonc/f/t string hasQuotedSyntax jsonc/r s/p n/customParsers.recordHelpers.ts` (121 lines)
  - `t string/t typeof/f/t unknown/r s/p p/index.ts` (101 lines)

  Suggested action: either split them or add a `// Justification: ...` comment explaining why they exceed 100 lines.

- Silent catch blocks in all logger sinks (`console`, `file`, `opfs`, `sessionStorage`) discard errors without logging.
  Per project rules: "Every catch block must log the caught error for debugging."
  The logger sinks are a special case (they **are** the logging infrastructure),
  but the file sink and OPFS sink catch blocks silently swallow errors with `// Silently fail` comments.
  Suggested action: at minimum, set the `available` flag to `false` in the sink `$` function catch blocks
  (already done in the orchestrator, but not in individual sinks), or use `console.error` as a last-resort fallback.

- `t never/f/t never/onLoadRedirectingTo/r s/p p/index.ts`:
   Silently does nothing if no anchor element is found. Per project rules: "Never silently discard unexpected states."
  If this function is called, it should be
  because a redirect is expected. Finding no target element likely indicates a bug.
  Suggested action: throw an error or at minimum `console.warn` when no `.redirectingTo` anchor is found,
  or document that silent no-op is intentional.

- `t object/t promise/f/t number/wait/r a/p p/index.ts:43`:
   TSDoc `@example` block uses `.then()` pattern which contradicts project rules.
  Suggested action: rewrite the example to use `async`/`await`.

- Test coverage gap: 32 test files for 114 files with exports (roughly 28% coverage by file count).
  Major areas without tests include: `path/index.ts` (fallback implementations), `capitalize`,
  `trim`/`trimStart`/`trimEnd`, `toExport`, `onLoadRedirectingTo`, `boolean.from`, `array.from.iterable`, `record.pick`,
  `record.omit`, `merge`, `regexp.from`, `html/jsx`, and most type guard functions.
  Suggested action: prioritize tests for `path/index.ts` browser fallbacks, `merge`, `pick`/`omit`,
  and `toExport` which have the most complex logic.

### NIT

- `src/index.ts` only re-exports `types`. The `path` module is not re-exported from the package entry point.
  Suggested action: consider re-exporting `path` if it is intended for consumer use, or document why it is excluded.

- `t object/t logger/t sink/t file/p p/index.ts`:
   Writes log files to `node_modules/.monochromatic/` which is unconventional and will be wiped on `npm install`.
  Suggested action: consider using a `.cache` or `data` directory, or document this design choice.

- `t object/t logger/t sink/t sessionStorage/r s/p p/index.ts:43`:
   `lineCounter++` uses postfix increment which is a mutation without justification.
  Suggested action: add a brief comment explaining why mutable state is needed here.

- `customParsers.ts:101`: `['-', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9'].some(...)` could be simplified.
  Suggested action: use a regex or charCode range check for cleaner digit detection.

- `path/index.ts`: The `dirnameFallback` function uses a `for` loop with `let charIndex` and mutable `lastSlash`.
  The mutable variables lack justification comments.
  Suggested action: add brief justification comments for the `let` bindings.

- Several `as any` casts across the codebase lack justification comments:
  - `t object/t array/f/t iterable/r s/p p/index.ts:7-8` (no comments)
  - `t object/t array/f/t iterable/p p/index.ts:7-8` (no comments)
  - `t string/f/t any/export/r s/p p/index.ts:123` (no comment)
  - `t boolean/t is/t p object/t iterable/f/t any/r s/p p/index.ts:44` (no comment)
  - `t boolean/t is/t p object/t iterable/f/t any/r a/p p/index.ts:5` (no comment)
  - `t boolean/t is/t p object/t thenable/f/t any/r s/p p/index.ts:47` (no comment)
    Compared to `t object/t array/f/t number/t int/range/r s/p p/index.ts` which correctly has oxlint-disable comments.
    Suggested action: add oxlint-disable comments or justification for each `as any` cast.

### NON-ACTIONABLE

- The directory naming convention (`t boolean/t is/t p string/f/t unknown/r s/p p/`) encodes type system relationships
   in file paths but is extremely difficult to navigate.
  Short abbreviations
  (`t` = type, `f` = from, `r s` = restriction sync, `p p` = params positional, `p n` = params named) are not
  documented anywhere discoverable.
  Best-effort idea: add a `CONVENTIONS.md` file explaining the directory naming scheme,
  or consider flattening the hierarchy for the most commonly used utilities.

- The `$` naming convention (every exported function is named `$`) makes stack traces and IDE navigation harder
   since all functions have the same name.
  This appears to be a deliberate architectural choice for the namespace-based API design
  (`types.boolean.is.string.sync.positional.$`).
  Best-effort idea: the tradeoff may be acceptable,
  but consider whether function names could be more descriptive in stack traces.

- ~~82 files in `src/deprecated/` remain in the codebase.~~ **Resolved**: deprecated directory deleted;
   modules migrated to `module-dom`, `test-fixture-data-sequence`, and proper locations within the types tree.
