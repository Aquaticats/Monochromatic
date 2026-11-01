# AUDIT: regex-replacement parsing performance

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/desktop-daemon/hall-monitor` is now `package/hall-monitor/daemon`;
> `package/dev-script/watch-restart` is now `package/watch-restart/cli`;
> `package/dev-script/page-weight` is now `package/page-weight/cli`;
> `package/dev-script/task-util` is now `package/monorepo-task/core-exec`.

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/dev-script/catalog-tighten` is now `package/dependency-catalog/cli-tighten`;
> `package/dev-script/file-enforcer` is now `package/file-enforcer/core`;
> `package/dev-script/deps-cube` is now `package/dependency-catalog/cli-cube`;
> `package/module/hyperscript` is now `package/hyperscript/core`;
> `package/module/toml-edit` is now `package/toml-edit/core`.

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/cli/mvm` is now `package/mvm/cli`;
> `package/cli/terminal-exec` is now `package/terminal-exec/cli`;
> `package/cli/vmsync` is now `package/vmsync/cli`.

Audit of the parsers introduced by the no-regex sweep
(`HANDOVER.no-regex.md`,
 27 commits).
 The sweep replaced regex with
hand-written string parsers.
 To satisfy `no-function-root-let`,
 the
fixer models reached for recursive accumulator walkers instead of loops.
The result is O(n^2) time and O(n) stack depth on inputs where a single
linear pass is O(n) and stack-safe.

## Root cause

The fixers hit two rules at once:
 `no-restricted-syntax/no-regex` (use a
parser) and `no-function-root-let` (no `let` at function-body root).
 The
path of least resistance was a recursive helper that threads an immutable
accumulator,
 which sidesteps both rules but is algorithmically worse than
the regex it replaced.
 Two shapes recur:

- Cursor recursion:
   `return scan(idx + 1)`.
   O(n) calls,
   O(n) stack
  depth,
   no tail-call elimination in V8,
   so it overflows the stack on
  long input.
   Time is O(n) only if no accumulator copy happens.
- Accumulator recursion:
   `acc: acc + c` or `[...acc, token]` rebuilds
  the whole accumulator on every step.
   O(n^2) time on top of the O(n)
  stack depth.

Both are avoidable with the escape hatches the workspace already allows
for controlled mutation (see the fix approach below),
 so the regex ban is
not the problem;
 the recursion-instead-of-loop reflex is.

### Runtime target

The complexity figures above are the V8 (Node) figures,
 and the workspace
has moved its primary runtime to V8.
 This matters because Bun's
JavaScriptCore masks part of the defect today:
 JSC does proper tail-call
elimination,
 so the tail-positioned cursor recursions do not overflow
under Bun,
 and JSC uses rope strings,
 so `acc + c` concatenation measures
near-linear under Bun.
 Wave 1 children confirmed both empirically.
 V8 does
neither,
 so on the target runtime every one of these recursions overflows
the stack on long input.
 The array-spread accumulator (`[...acc, token]`)
is the exception that is already O(n^2) on every engine;
 wave 1 measured
`splitWhitespace` at 123 -> 327 -> 1301 -> 8027 ms across input doublings
under Bun.
 The fixes are engine-agnostic linear passes,
 so they are
correct for both runtimes and remove the dependence on JSC-specific
behavior ahead of the V8 migration.

## Criterion: what to fix, what to leave

Fix when the recursion is a flat scan over a string or a flat array of
lines/tokens:

- recursive cursor scan over a string/byte sequence
  (`return scan(idx + 1)`,
   `findRunEnd`,
   `skipWs`)
- accumulator recursion building a string (`acc: acc + c`) or array
  (`[...acc, token]`)
- repeated full-input rescans,
   or a second `indexOf` with no monotonic
  anchor that rescans to end-of-string each step (the
  `containsHiddenSegment` shape)

Leave alone (not the target;
 flagging these would be churn):

- structural recursion over a tree,
   AST,
   grid,
   graph,
   or filesystem,
  where recursion depth is bounded by structure depth,
   not input length
  (`module/toml-edit/src/resolve.ts` `walkTable`/`walkArray`,
   file-tree
  walks,
   any sudoku backtracking).
  Caveat:
   a member/call chain (`a.b.c`) or a left-associative operator
  chain (`a + b + c`) is a degenerate AST spine whose depth equals operand
  count,
   so its depth tracks input length,
   not nesting;
   it is NOT this
  exempt case and must flatten iteratively
  (see `doc/audit/chain-flatten-skewed-tree.md`)
- code already doing a single `for...of` pass or a monotonic `indexOf`
  walk with no accumulator copy
- regex deliberately kept with a justified disable
  (`pi/terminal-title` `COMMAND_NOISE_RE`,
   `pi/auto-mode` secret
  patterns,
   the `module/es` regexp tooling subtree)

## Confirmed sites (read in full)

- `module/hyperscript/src/html/index.ts` `camelToKebab`:
   recursive
  `walk`,
   `` acc: `${acc}-${c.toLowerCase()}` ``.
   O(n^2) time,
   O(n) stack.
- `dev-script/watch-restart/src/filters/hidden.ts`
  `containsHiddenSegment`:
   recursive `scanFromSeparator` with both
  `indexOf('/')` and `indexOf('\\')` from each separator;
   the backslash
  search rescans to end-of-string every step.
   O(n^2) worst case,
   O(n)
  stack.
   Documented as a hot path on every watch event.
- `cli/terminal-exec/src/desktop-entry-types.ts` `expandEscapes`:
  recursive `walk`,
   `acc: acc + c`.
   O(n^2) time,
   O(n) stack.
- `claude-code-plugin/source/src/lib/text-scan.ts` strip walker:
  recursive `walk`,
   `acc + text.slice(...)`.
   O(n^2) time,
   O(n) stack.
- `claude-code-plugin/source/src/handlers/bash-output-filter/filter-transforms.ts`:
  recursive `walk` plus a nested recursive `findRunEnd` that recurses
  once per character of a run.
   The function exists to collapse long
  repeated-character runs,
   so it overflows the stack on exactly its
  target input.
   Highest severity (arbitrary tool output).
- `desktop-daemon/editord/src/server/lsp/json-rpc.ts`:
   recursive
  `skipInlineWs` and `collectDigits` over LSP `Content-Length` headers.
  Adversarial-exploitable (a malicious server can send unbounded header
  whitespace).
- `desktop-daemon/hall-monitor/src/infra/lock.ts` `splitOnWhitespace`:
  recursive `walk` with `[...acc, token]`.
   O(n^2) time,
   O(n) stack.
- `dev-script/page-weight/src/html.ts` `skipWs`/`scanToken`:
   recursive
  cursor scans over HTML.
   O(n) stack on long input.
- `dev-script/task-util/src/tsgo-filter.ts` `step`/`scan`/
  `startOfDigitsBackwards`:
   recursive scans over compiler output.
- `module/toml-edit/src/keys.ts` `walk`:
   recursive cursor scan over a
  key string.
- `dev-script/inference-canary/src/codegen/sudoku-grid.ts` `walk`:
  line-grouping scan with `[...acc, block.join()]` (a text scan,
   not the
  sudoku solver).
   Confirms targets exist even inside codegen files;
   the
  child must split string scans from structural recursion per file.

## Per-package targets

Each package gets one child.
 The child confirms each flagged file against
the criterion above (reading the current implementation as the behavioral
spec),
 fixes the string-scanning sites,
 and leaves structural recursion.

- `claude-code-plugin/source`:
   `lib/text-scan.ts`,
  `handlers/bash-output-filter/{filter-patterns,filter-transforms,validation}.ts`,
  `handlers/guardrail.ts`,
  `handlers/stop-reminders/{uncertainty,uncertainty-strip,uncertainty-phrases,uncertainty-citations}.ts`,
  `handlers/terminal-title/formatter-utils.ts`
- `cli/terminal-exec`:
   `src/desktop-entry-types.ts`
- `cli/vmsync`:
   `src/boot.ts`,
   `src/qemu-img.ts`
- `cli/mvm`:
   `src/list.ts`
- `desktop-daemon/editord`:
   `src/server/lsp/json-rpc.ts`,
  `src/server/operations/resolve-fs-id.ts`,
  `src/client/editor/auto-indent.ts`
- `desktop-daemon/hall-monitor`:
   `src/infra/lock.ts`
- `dev-script/watch-restart`:
   `src/filters/hidden.ts`
- `dev-script/page-weight`:
   `src/html.ts`,
   `src/url-detect.ts`
- `dev-script/task-util`:
   `src/tsgo-filter.ts`,
   `src/oxlint-augment.ts`,
  `src/depends-resolve-glob.ts`
- `dev-script/deps-cube`:
   `src/probe-field-parsers.ts`
- `dev-script/catalog-tighten`:
   `src/index.ts`,
   `src/version-parse.ts`
- `dev-script/file-enforcer`:
   `src/io/glob.ts`,
  `src/package/mise.generate-index.ts`
- `dev-script/inference-canary`:
   `src/linter-artifacts-timestamp.ts`,
  `src/codegen/sudoku-grid.ts`,
   and other `codegen/*` files only where
  the recursion is a flat text scan (leave the solver/generator
  recursion)
- `dev-script/inference-canary-viewer`:
   `src/data/model-icons.ts`
- `module/hyperscript`:
   `src/html/index.ts`
- `module/toml-edit`:
   `src/keys.ts`,
   `src/comments.ts` (leave
  `src/resolve.ts`,
   it is AST recursion)
- `oxlint-plugin/tsdoc`:
   `src/rules/{empty-tags,structural-tags,tag-escaping,tag-names,tag-validation,type-annotations}.ts`
- `oxlint-plugin/stylistic`:
   `src/utility/chain.ts`,
  `src/utility/indent.ts`
- `rolldown-plugin/import-attributes`:
   `src/transform-helpers.ts`

## Severity

- High (arbitrary or large input,
   real stack-overflow exposure):
  `claude-code-plugin/source` (tool output,
   assistant responses),
  `page-weight` (HTML),
   `task-util` (compiler output),
   `editord`
  json-rpc (adversarial header),
   `module/hyperscript` and
  `watch-restart` (confirmed O(n^2),
   hot path).
- Medium (bounded but unvalidated input):
  `terminal-exec`,
   `hall-monitor`,
   `deps-cube`,
   `toml-edit`,
  `config/oxlint-*` (lint-time hot path over source files).
- Low (small bounded input,
   mostly correctness/readability):
  `vmsync`,
   `mvm`,
   `catalog-tighten`,
   `file-enforcer`,
  `inference-canary*`,
   `import-attributes`.

## Fix approach (handed to each child)

- One linear pass.
   Satisfy `no-function-root-let` via the allowed
  escape hatches,
   not recursion:
   an IIFE-with-`let`
  `(function name () { let acc = ''; for (const c of s) { ... } return acc; })()`,
  a helper that ends in `return <local-binding>`,
   `Array.reduce`,
   or a
  `for...of` building an array then `.join('')`.
   `AGENTS.md` documents
  these under the `no-function-root-let` rule.
- Do not reintroduce regex.
   Do not disable any lint rule.
- Behavior-preserving.
   The current implementation is the spec.
   If a
  function has no test,
   write equivalence tests capturing current
  behavior on the edge cases first,
   then refactor,
   then confirm green.

## Spawn plan

- One `spawn-claude` child per package,
   capped at 16 in flight.
- Children do not commit.
   They leave changes in the working tree and
  report changed files.
   The parent commits per package with explicit
  pathspecs to avoid concurrent git index contention.
- Children touch only files in their assigned package and must ignore
  and never revert any other working-tree changes (the tree carries
  unrelated out-of-scope modifications,
   see `HANDOVER.no-regex.md`).
- Children run package-scoped lint and test only
  (`mise run //package/<pkg>:lint`,
   `:test:unit`),
   never workspace-wide
  `mise run //:lint`,
   to avoid seeing siblings' in-progress edits.
