# Audit: `let` usage across the monorepo

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/webapp-productivity/doodle-widget` is now `package/doodle-widget/app`;
> `package/dev-script/task-util` is now `package/monorepo-task/core-exec`.

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/module/logger` is now `package/logger/core`.

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/cli/forbidden-strings` is now `package/forbidden-strings/cli`;
> `package/cli/vmsync` is now `package/vmsync/cli`.

Generated 2026-05-10 from a static survey of TypeScript sources under `package/`,
excluding `dist`,
 `node_modules`,
 and generated files.
The audit underpins the design decision of whether to add a custom oxlint rule banning `let`,
banning it only at function-body root scope,
 or leaving the prose policy in `AGENTS.md`
("`const` over `let`;
 comment any deviation from immutability") as-is.

## Executive summary

The codebase contains **785 `let` declarations across 319 files**.
Roughly half are necessary mutation primitives (state machines,
 parser cursors,
 accumulators
where extraction or `reduce` would obscure intent more than it tightens scope);
the other half are pattern-replaceable (numeric loop counters,
 conditional-init scope-leaks,
caches that could route through `memoize`).

Existing enforcement is `eslint/prefer-const: 'warn'`,
which only catches `let` that is never reassigned;
it does not flag deliberate mutation.
No `oxlint-disable` comments justify any `let` except for three special cases
(module-init pattern,
 `using` binding).

A blanket `no-let` rule forces hundreds of justifying disable comments.
A function-body-root `no-let` rule
(AST shape:
 `FunctionDeclaration > BlockStatement > VariableDeclaration[kind='let']`)
catches the scope-leak cases that the user's IIFE point describes,
while permitting tight-scope mutation inside loops,
 blocks,
 IIFEs,
 and switch cases.

## Quantitative findings

### Top-line counts

- **785** total `let` declarations
- **319** files contain at least one
- **283** non-test source files (700 occurrences)
- **36** test files (85 occurrences)
- **0** exported `let` declarations
- **92** `let` declarations at file column 1 (module-level scope)
- **133** are C-style for-loop counters (`for (let i = ...; ...; ...)`)

### Density distribution

How many files have N `let` declarations:

- 1 let:
   133 files
- 2 to 3 lets:
   125 files
- 4 to 5 lets:
   36 files
- 6 to 10 lets:
   20 files
- 11+ lets:
   5 files

The long tail is short:
 the top 5 files (`figma/kiwi/src/index.ts` with 19,
`forbidden-strings/src/mise.port-betterleaks.ts` with 19,
`messages-demo/src/lib/seed.ts` with 13,
`messages-demo/src/client/composer.worker.ts` with 13,
`test-fixture/data-sequence/src/generator.0to999.ts` with 18) account for 82 occurrences,
about 10% of the total.

### Per-package distribution

Top contributors (total `let` count per package):

- `webapp-content/messages-demo`:
   105
- `desktop-daemon/editord`:
   69
- `module/es`:
   46
- `webapp-forge/server`:
   43
- `webapp-productivity/doodle-widget`:
   36
- `webapp-forge/seed`:
   31
- `pi/auto-mode`:
   28
- `module/logger`:
   24
- `figma/kiwi`:
   24
- `ssg/aquati.cat`:
   23
- `dev-script/inference-canary`:
   23
- `pi/morph-compact`:
   22

Higher densities cluster in packages with heavy data-transformation or stateful workloads
(parsers,
 scenario simulators,
 editor / canvas state,
 networking).

### Pattern-based categorization

Counts derived by regex over declaration shape;
 categories overlap
(e.g.,
 a for-loop counter `let i = 0` matches both "for-loop counter" and "numeric-literal init"):

- **For-loop counter** (`for (let X = ...)`):
   133 occurrences
- **Numeric-literal init** (`let X = N;`):
   327 occurrences
- **Empty-string init** (`let X = '';`):
   51 occurrences
- **Boolean-literal init** (`let X = true|false;`):
   77 occurrences
- **Null init** (`let X = null;`):
   40 occurrences
- **Typed-undefined init** (`let X: T | undefined`):
   48 occurrences

Combined and de-duplicated,
 pattern-matched declarations cover roughly 85% of the total;
the remaining 15% are complex initializers
(function calls,
 expressions,
 object literals constructed at declaration).

## Scope analysis

Scope leak is the central concern raised by the IIFE alternative:
mutable variables declared at function-body root scope live for the rest of the function,
risk later reads of stale or partially-initialized state,
and pollute the function's reachable variable set.

### Module-level (92 declarations)

Found by `^let` (zero leading whitespace).
These intentionally have module scope;
 IIFE wrapping does not apply.
Sample purposes (from a 30-line sample):

- Caches:
   `cachedManager`,
   `cachedIsRoot`,
   `cachedApiKey`,
   `workspaceRootsCache`,
  `pagefindApi`,
   `cachedTool`,
   `cpuMax`,
   `memoryMax`.
- Counters and one-shot flags:
   `composerBooted`,
   `pendingSubmit`,
   `cachePopulated`,
  `warnedMissingKey`,
   `loadAttempted`,
   `skippedCount`,
   `alreadyTightCount`,
   `notFoundCount`,
  `lastProcessedEventId`,
   `counter` (test fixtures).
- Timers and lifecycle handles:
   `promotionTimer`,
   `collapseTimer`,
   `extensionApi`.
- Module-init slots:
   `defaultBackendsBuilder`,
   `modelDirs`.

Alternatives:

- **Closure factory**:
   replace with a memoizing closure returned at module init.
  Adds indirection;
   loses module-scope identity that some test fixtures rely on.
- **`memoize` from `module-es`**:
   directly addresses caching patterns.
  Currently used only once in the entire monorepo,
  despite at least 20 module-level caches that fit its signature.
- **`Map` / `WeakMap`**:
   replaces a single mutable slot with a mutable container of `const`.
  Lint-clean but still mutable.
- **Accept as-is**:
   module scope is intended;
   nothing to "tighten".
  Document each with TSDoc explaining the cache invariant.

### Function-body root (the scope-leak case)

Examples (from spot-check of 6 representative files):

- `mise.port-betterleaks.ts:225-231` declares seven `let` variables
  (`id`,
   `description`,
   `regex`,
   `pathScope`,
   `secretGroup`,
   `skipReport`,
   `hasRequired`)
  inside a TOML rule parser;
   the seven names remain in scope
  for the rest of the enclosing function after the scan completes.
- `judge.ts:343-345` declares `depth`,
   `inString`,
   `escape` for a brace-balanced JSON scanner;
  all three persist past the scanning loop.
- `seed.ts:267-270` declares `seq`,
   `charCount`,
   `firstMd`,
   `chunkCount`
  before a chunk-rendering loop;
   same shape.

These are the cases the IIFE / function-extract refactor targets.

### Block / loop scoped

Loop counters (`for (let i = 0; ...)`) are block-scoped;
 no leak past the loop body.
Lets declared inside `if`/`else`/`while` bodies similarly stay bounded.
These are not scope-leak cases;
 the question for these is the lint rule shape,
 not refactor need.

### IIFE scoped

40 genuine IIFEs in the workspace
(matched by `}\s*)\s*(\s*)`,
 the immediately-invoked closing form).
Some currently wrap `let`-using bodies to bound their scope;
the pattern is established but rare.

## Existing enforcement

### Already on

- `eslint/prefer-const: 'warn'` (in `package/config/oxlint/src/rule/restriction.ts:112`).
  Catches `let` declarations that are never reassigned;
  any surviving `let` in the codebase is one that does reassign.
- `eslint/init-declarations: 'off'` (in overrides);
  the codebase does not require an initializer on `let` declarations.

### Existing disable comments for prefer-const / no-let

Only three sites use an `oxlint-disable` for `prefer-const` or `no-let`:

1. `webapp-productivity/done-h-css-test/src/client/components/task-detail-autofill.ts:100`:
   `using _loadingGuard = { ... }` declared via `using` binding (TS5 explicit-resource-management).
   `using` is structurally `let`-like per spec;
    the disable preserves the keyword.
2. `module/es/src/types/.../backends.ts:69`:
   `let defaultBackendsBuilder: DefaultBackendsBuilder | undefined`,
   commented "Intentional:
    configured once at module load by platform entry.
   "
3. `module/es/src/types/.../customParsers.dispatch.ts:29`:
   Same module-init pattern.

The `AGENTS.md` rule "comment any deviation from immutability" is universally unmet across the other 782 declarations.

### `using` declarations

260 `using` declarations across 158 files (TS5 explicit-resource-management feature).
These bind via the `using` keyword,
 not `let`,
 so they do not trip `prefer-const`
and would not trip a new `no-let` rule.

## Refactor opportunities by pattern

### For-loop counters (133 occurrences)

`for (let i = 0; i < N; i += 1)` and `i++` forms are the most uniform category.
97 of 133 (73%) use unit increment (`++` or `+= 1`) starting from 0.

Replacements:

- **Iterate the source collection directly**:
   when the body reads `coll[i]`,
  switch to `for (const item of coll)` or `coll.forEach(...)`.
- **`for (const i of range(n))`**:
   requires a `range` helper;
  `module-es` already has one (used once for `range` in `module/es/src/types/...range/...`).
- **`Array.from({ length: n }, (_, i) => ...)`** when the result is collected into an array.

Sample mechanical migration (from `webapp-forge/seed/src/rng.unit.test.ts:38`):

```typescript
// Before
for (let i = 0; i < 100; i += 1) {
  /* ... */
}
// After
for (const _i of range(100,)) {
  /* ... */
}
```

The 36 of 133 (27%) that use decrement or non-unit step
(e.g.,
 walking buffers backwards,
 scanning in strides) need case-by-case handling.

### Numeric and string accumulators (~280 after deduping from for-loop counters)

Patterns like:

```typescript
let total = 0;
for (const chunk of chunks)
  total += chunk.byteLength;
```

Single-accumulator-then-return rewrites to `reduce`:

```typescript
const total = chunks.reduce((s, c,) => s + c.byteLength, 0,);
```

Multi-accumulator cases (e.g.,
 `force-push.ts:332-333` with `priorOid` and `appliedTotal`)
reduce poorly because each iteration depends on the prior iteration's result and emits side effects.
Extraction to a helper function preserves the imperative style while shrinking scope:

```typescript
function applyBurst(config: Config,): { priorOid: Oid; appliedTotal: number; } {
  let priorOid = ZERO_OID;
  let appliedTotal = 0;
  for (let i = 0; i < config.burstEvents; i += 1) {
    /* ... */
  }
  return { priorOid, appliedTotal, };
}
const { priorOid, appliedTotal, } = applyBurst(config,);
```

The `let` survives in the helper but doesn't leak to the caller.

### Conditional init (48 typed-undefined declarations, plus untyped variants)

The strongest scope-leak case.
Example shape (from `mise.port-betterleaks.ts:225-231`):

```typescript
let id: string | undefined;
let description: string | undefined;
let regex: string | undefined;
// scan loop assigns to each
while (i < lines.length) {
  /* ... */
}
out.push({ id, description, regex, },);
```

Alternatives:

- **Extract to function** (preferred):
   the scanner becomes
  `function parseRuleBody(lines, startIndex): { rule: RawRule; nextIndex: number; }`.
  All seven mutable variables live inside;
   the call site gets `const rule = ...`.
  Bonus:
   the parser becomes unit-testable in isolation.
- **IIFE wrap**:
   keeps the code structurally similar but tightens scope.
  Less testable than a named function;
   useful when the parsed value is used only locally.
- **`reduce` over the scanned lines**:
   clean for simple field-collection,
  awkward when the scan logic involves nested loops or lookahead.

### State machines and parsers

The kiwi binary parser (`figma/kiwi/src/index.ts`) shows the legitimate end of the spectrum:
varint reading,
 length-prefixed strings,
 ZIP central-directory scans.
Each `let` is genuinely a state register;
the algorithm IS mutation.
Extraction to helper functions is possible but produces dozens of tiny helpers
each holding one cursor.

The JSON scanner (`judge.ts:343-345`) is similar but smaller (3 state variables).
Extracting to `function findBalancedClose(text, start): number | undefined { ... }` is clean.

### Late binding to ternary candidates

A heuristic match for `let X = a; ... X = b within 4 lines` returns only 6 candidates monorepo-wide.
The pattern is rarer than expected;
many of those 6 also fail the ternary refactor on closer reading
(intervening side effects,
 multiple branches,
 or early returns).

Spot-checked from earlier analysis:

- `seed.ts:349` (clean ternary):
   `let bytes = 0; if/else if/else if/else` cascade
  with all branches assigning.
   Trivially replaces with a nested ternary expression.
- `mise.port-betterleaks.ts:380` (retract):
  conditional branch also calls `lines.push(...)`;
   a ternary refactor duplicates the condition check.
- `fallbacks.ts:54`:
   three potential reassignments across the function,
   not a one-liner swap.
- `mise.port-betterleaks.ts:159`:
   chainable to `replaceAll(...).replaceAll(...)`,
  but the existing form has inline comments between each step explaining WHY each replacement happens.

### Module-level caches

20+ caches at module scope (`cachedManager`,
 `cachedApiKey`,
 `pagefindApi`,
 etc.).
The `memoize` utility from `module-es` exists and would replace many of these
with a one-liner:

```typescript
// Before
let cachedTool: NotificationTool | null | undefined = undefined;
function getNotificationTool(): NotificationTool | null {
  if (cachedTool !== undefined)
    return cachedTool;
  cachedTool = computeTool();
  return cachedTool;
}

// After
const getNotificationTool = memoize(
  function getNotificationTool(): NotificationTool | null {
    return computeTool();
  },
);
```

Currently `memoize` is used once in the whole monorepo.
Migrating module-level caches to `memoize` is a real cleanup with or without a `no-let` lint rule.

## Lint rule design options

### Option A: blanket `no-let` rule

AST selector:
 `VariableDeclaration[kind='let']`.

- Catches every `let`.
- Forces a disable comment with justification on each surviving instance.
- Adds an estimated 600 to 700 disable comments to the codebase
  (785 lets minus the for-loop counters that mechanically migrate to `for-of`).
- Per-disable-comment overhead:
   at minimum one line above each `let` saying
  `// oxlint-disable-next-line no-let -- loop accumulator` or similar.
- Maximum strictness;
   corresponds to "no mutable function-local state,
   ever.
  "

### Option B: function-body-root `no-let` rule

AST selector:

```text
FunctionDeclaration > BlockStatement > VariableDeclaration[kind='let'],
FunctionExpression > BlockStatement > VariableDeclaration[kind='let'],
ArrowFunctionExpression > BlockStatement > VariableDeclaration[kind='let']
```

- Catches `let` declared as a direct child of a function body.
- Does NOT catch `let` inside `for`,
   `while`,
   `if`,
   `switch`,
   IIFE bodies,
   or nested blocks.
- Migration path:
   wrap in IIFE,
   extract to helper,
   or use `reduce`.
  The disable comment is still available with justification.
- Estimated catches:
   scope-leaky declarations only;
  far fewer than the blanket version.
  Rough estimate:
   ~250 catches based on the typed-undefined plus accumulator-then-return pattern counts.

### Option C: module-level `no-let` rule

AST selector:
 `Program > VariableDeclaration[kind='let']`.

- Catches the 92 module-level `let` declarations.
- Forces migration to `memoize`,
   `Map`,
   or explicit factory.
- Smallest catch set;
   biggest per-catch return (cache invariants currently undocumented for most).

### Option D: pattern-targeted rules

Multiple narrower rules:

- `no-typed-undefined-let`:
   catch `let X: T | undefined = undefined`
  to push toward extract-function or IIFE.
- `no-module-level-let`:
   equivalent of Option C.
- `no-let-then-conditional-reassign`:
   catch `let X = a; if (cond) X = b`
  to push toward ternary.

The codebase has 19 custom no-restricted-syntax rules already
(`no-arrow-function`,
 `no-enum`,
 `no-for-in`,
 `no-switch`,
 `no-try-finally`,
 etc.).
Adding more fits the established pattern;
the cost is per-rule plugin code,
 which is mechanical given the existing examples
(see `package/oxlint-plugin/no-restricted-syntax/src/rule/no-variable-function-expression.ts`).

### `no-disable-*` companion rule

Following the established pattern of `no-disable-no-arrow-function`,
 `no-disable-no-enum`,
 etc.,
any new let-related rule could also ship a `no-disable-*` rule
that bans inline suppression entirely.
That removes the escape hatch and forces structural fixes,
at the cost of permanent rigidity if a legitimate exception emerges later.

## Migration cost estimate

For each option,
 the rough one-time cost in agent-hours
(reading code path,
 applying refactor,
 verifying tests pass):

- **Option A blanket ban**:
   8 to 16 hours.
  Many trivial changes (disable comment per for-loop counter)
  plus ~50 real refactors hidden in the noise.
  Significant noise added to the codebase.
- **Option B function-body-root**:
   4 to 8 hours.
  Each catch is a real scope-leak case worth fixing structurally.
  Touches ~150 declarations.
- **Option C module-level**:
   2 to 4 hours.
  92 declarations,
   almost all of which are caches that route to `memoize`
  or one-shot init slots that route to factory functions.
- **Option D pattern-targeted**:
   similar to Option C plus marginal cost per added rule.

These are estimates from declaration count and per-case complexity;
they have not been calibrated against actually doing a sample slice.

## Recommendation

**Option B (function-body-root) plus a focused Option C cleanup pass.
**

Reasoning:

1. The current `prefer-const: 'warn'` already enforces "don't mutate what you don't have to.
   "
   The remaining 785 lets are genuinely reassigned;
   none are accidentally mutable.
2. The user's IIFE point identifies the actual concern:
   not mutability per se,
    but mutable variables leaking past their useful lifetime
   into the rest of a function's scope.
   Option B's AST selector directly targets that.
3. Option C (module-level) is a separate cleanup
   that benefits from a dedicated audit and migration to `memoize` independent of any new lint rule.
   Most module-level lets have undocumented cache invariants;
   migrating them to `memoize` simultaneously documents intent and removes the `let`.
4. Option A (blanket ban) generates hundreds of low-value disable comments
   for cases (loop counters,
    intentional state machine state)
   that have no better alternative.
   The noise floor argues against it.
5. Option D rules are useful as a follow-up to refine specific catch sets
   once Option B reveals which scope-leak shapes recur in practice.

A one-shot audit-and-refactor pass for the ~50 to 150 highest-value cases
(module-level caches,
 typed-undefined conditional inits in long functions,
 late-binding ternary candidates)
captures most of the value with or without enacting Option B.
That pass should be done first;
deciding on the permanent rule is easier with the refactored codebase as the starting point.

## Implementation status

2026-05-10:
 Option B (function-body-root) and Option C (module-level) shipped as two separate oxlint rules in
 `package/oxlint-plugin/no-restricted-syntax/`:

- `no-function-root-let` (`src/rules/no-function-root-let.ts`):
   visits `FunctionDeclaration` and `FunctionExpression`;
   reports each `let` declared as a direct child of the function body.
   Two AST-level allowlists:
  1. **IIFE callee**:
      the function's `parent.type === 'CallExpression' && parent.callee === fn`.
      Covers both named-function IIFEs `(function () { let x; })()`
      and arrow IIFEs `(() => { let x; })()` (the latter still trips `no-arrow-function`;
      prefer the named form).
  2. **Helper-function shape**:
      the function body ends with `return <Identifier>` where the identifier resolves to a function-body-root binding
      (`let`,
      `const`,
      or `function`).
      `concat()` in `package/webapp-forge/stress/src/scenarios/force-push.ts` is the paradigm.
      Conservative on purpose:
      `return total * 2` or `return { a, b }` does NOT match.
- `no-module-root-let` (`src/rules/no-module-root-let.ts`):
   visits `Program`;
   reports each `let` declared at the top level,
   including `export let x` (parsed as `ExportNamedDeclaration` wrapping a `VariableDeclaration`).
   No allowlists.

Both rules:

- Registered in `package/oxlint-plugin/no-restricted-syntax/src/index.ts`.
- Enabled at `'error'` in `package/config/oxlint/src/rule/restriction.ts` (shipped at `'warn'`;
   flipped to `'error'` 2026-06-01 after the migration reached zero reports across the linted tree).
- Have no corresponding `no-disable-*` companion rule;
   disable-with-justification is the contracted escape.
- Referenced in `AGENTS.md` under the `const`/`let` policy.

## Migration steps

The rules shipped at `'warn'` so they surfaced the existing footprint without blocking CI.
 The migration to `'error'` completed 2026-06-01.
 The phase history below is retained for the record;
 the status table records the final state.

### Phase 1: Capture the post-warn baseline

Run a full lint pass and count per-rule reports:

```bash
mise run //:lint 2>&1 | rg -c 'no-function-root-let'
mise run //:lint 2>&1 | rg -c 'no-module-root-let'
```

Spot-check 10 reports per rule.
 Confirm:

- None is inside a loop body,
   explicit block,
   switch case,
   IIFE callee,
   or helper-shape function (those would be heuristic bugs and block further migration until fixed).
- None is inside generated code,
   test fixtures,
   or third-party vendored files that should be added to a per-package override in
   `package/config/oxlint/src/overrides.ts`.

Record the counts in the "Status table" below.

### Phase 2: Mechanical refactor wins (target: 20-40% of reports)

Sites where the remediation is a one-line `const` rewrite:

- `let x = 0; if (cond) x = a; else x = b;` -> `const x = cond ? a : b;`
- `let total = 0; for (const item of items) total += item.value;` ->
`const total = items.reduce(function add (sum, item,) { return sum + item.value; }, 0,);`
- `let result = expr; result = transform(result);` -> `const result = transform(expr);` (or chain).

Each site is a small,
 independent commit;
 land in batches per package.

### Phase 3: Helper extractions and IIFE wraps (target: 30-50% of reports)

Two structural moves silence the report without disable comments:

- **Extract a helper that ends with `return <binding>`**.
   The function-shape allowlist fires;
   the call site becomes `const result = helperName(args);`.
   Best when the mutation block has clear inputs and one returned value.
   Bonus:
   the helper is unit-testable in isolation.
- **Wrap with a named-function IIFE**:
   `const x = (function compute () { let acc = init; /* ... */ return acc; })();`.
   Best when the mutation block is tightly coupled to surrounding statements and cannot be cleanly extracted.

Neither pattern needs a disable comment if the AST shape matches the heuristic.

### Phase 4: Disable-with-justification for the residue

Sites where neither refactor nor allowlist-shape applies:

- Genuine multi-let state machines (Mulberry32 PRNG step in `package/webapp-content/messages-demo/src/lib/seed.ts:143`;
   the kiwi binary parser in `package/figma/kiwi/src/index.ts`).
- Parsers with conditional-init fields and side-effecting branches (the TOML rule parser in
`package/cli/forbidden-strings/src/mise.port-betterleaks.ts:225-231`).
- Module-level memoization caches that have no clean Map/memoize replacement
(`package/module/logger/src/sinks/console.ts:8,11,100,160`).

Each disable comment must name the specific constraint:

```ts
// oxlint-disable-next-line no-restricted-syntax/no-function-root-let -- Mulberry32 PRNG state: per-step mutation is the algorithm
let value = seed | 0;
```

```ts
// oxlint-disable-next-line no-restricted-syntax/no-module-root-let -- console-sink memoization cache; warmed lazily on first emit, no Map/memoize equivalent for boolean-flag form
let verified = false;
```

A bare `-- needed` or omitted justification is not acceptable;
 the comment is the contract.

### Phase 5: Flip severity to `'error'`

Once the warning count is zero (every report is refactored,
 allowlist-shaped,
 or carries a justified disable comment),
 change both entries in `package/config/oxlint/src/rule/restriction.ts` from `'warn'` to `'error'`.

### Status table

<table>
<thead>
<tr>
<th>Phase</th>
<th>State</th>
<th>Date</th>
<th>Notes</th>
</tr>
</thead>
<tbody>
<tr>
<td>0: rules landed</td>
<td>DONE</td>
<td>2026-05-10</td>
<td>Two rule files + registration + `'warn'` enablement + AGENTS.md.</td>
</tr>
<tr>
<td>1: baseline capture</td>
<td>DONE</td>
<td>2026-06-01</td>
<td>Whole-repo `mise '//package/...:lint:oxlint'` reports zero instances of either rule.
Plugin firing confirmed against a throwaway violator probe (both rules reported as expected)
and against the live disable inventory. No open reports remained to spot-check;
the existing justified-disable directives stand in as the audited residue.</td>
</tr>
<tr>
<td>2: mechanical refactors</td>
<td>DONE</td>
<td>2026-06-01</td>
<td>Landed incrementally before the flip (e.g. `module/logger/src/sinks/console.ts` module-state moved into a `const`
container).</td>
</tr>
<tr>
<td>3: helper / IIFE</td>
<td>DONE</td>
<td>2026-06-01</td>
<td>Landed incrementally; helper-shape and IIFE allowlists absorb the structural cases.</td>
</tr>
<tr>
<td>4: disable-with-justification</td>
<td>DONE</td>
<td>2026-06-01</td>
<td>Residue carries block and next-line disables across `messages-demo`, `pi/auto-mode`, `dev-script/task-util`,
`cli/vmsync`, `pi/morph-compact`, `typeface/aquaticat`, and others; each names a concrete constraint (parser cursor,
state machine, singleton timer, PRNG state). No bare justifications.</td>
</tr>
<tr>
<td>5: flip to `'error'`</td>
<td>DONE</td>
<td>2026-06-01</td>
<td>Both entries in `restriction.ts` set to `'error'`. Post-flip fanout: zero reports of either rule;
the only failing package is `module/es` on unrelated rules (out of scope for this migration).</td>
</tr>
</tbody>
</table>

### Verification targets

These targets were defined against the 2026-05-10 tree.
 By the 2026-06-01 flip they had all converged to zero open reports:

- `package/webapp-forge/stress/src/scenarios/force-push.ts` (`concat()`):
   NO REPORT (helper shape,
   returns root const `out`).
- `package/webapp-content/messages-demo/src/lib/seed.ts` (`rng()`):
   the Mulberry32 PRNG is now a justified block disable (`seed.ts:204-226`) naming the two-variable state-machine
   constraint.
- `package/module/logger/src/sinks/console.ts`:
   refactored;
   module state moved into a `const` container with a `Symbol` sentinel,
   so no module-root `let` remains.
- `package/module/es/src/path/fallbacks.ts`:
   NO REPORT (helper shape).

Heuristic firing was reconfirmed on 2026-06-01 with a throwaway probe (one function-root and one module-root `let`):
 both rules reported as designed,
 ruling out a silently-disabled plugin.

Coverage gap noted during the 2026-06-01 baseline:
 packages without a `lint:oxlint` task are never checked by these rules.
 `package/figma/to-penpot` is one such package and still holds a module-root `let uuidCounter` in `src/index.ts`.
 Closing that gap is a separate lint-coverage task,
 not part of this migration.

## Appendix: source data

Generated files used during the audit
(stored under `/tmp/`,
 regenerable from this document's queries):

- `/tmp/let-audit.txt`:
   full match list,
   file:
  line:
  declaration
- `/tmp/let-by-file.txt`:
   per-file count,
   sorted descending
- `/tmp/let-reassign-candidates.txt`:
   declarations followed by reassignment within four lines
- `/tmp/forloops.txt`:
   all `for (let X = ...)` declarations

The queries are all `ripgrep` invocations against TypeScript sources
excluding `**/dist/**` and `**/node_modules/**`.
None of the data required parsing the TypeScript AST;
a more precise scope-leak audit would benefit from doing so.
