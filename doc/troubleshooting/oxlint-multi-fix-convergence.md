# oxlint 1.65.0: `--fix` requires multiple passes when two rules' fixes touch overlapping source spans

`oxlint --fix` applies at most one fix per overlapping byte region per
pass.
 When a JS plugin's `fix()` returns multiple `Fix` objects from a
single `context.report` call,
 oxlint merges them into one `Fix` whose
span covers the bounding box of all the insertions;
 any other rule's
fix that lands inside that bounding box is then rejected as
overlapping.
 The user must run `oxlint --fix` again (and possibly
again) to apply the rejected fixes.
 ESLint solves the same problem by
iterating up to 10 times internally;
 oxlint plans to but has not.

This document records the failure case we hit in
`@monochromatic-dev/oxlint-plugin-stylistic`,
 the source-level call
chain that produces it,
 the workarounds we considered,
 and the upstream
status.
 The limitation is tracked at [oxc#16118][16118] and was
re-reported in our exact shape at [oxc#20711][20711];
 we do not file a
duplicate (see "Why we do not file this upstream" near the end).

## Symptom

Running the combined fixture once:

```bash
oxlint --fix -c package/test-fixture/oxlint-stylistic/.oxlintrc.fixture.json \
  package/test-fixture/oxlint-stylistic/src/invalid/chain-and-mixed-operators.ts
```

on this source

```ts
const r = a + b * obj.d.e.f;
```

produces

```ts
const r = a + (b * obj.d.e.f);
```

and leaves one diagnostic behind:

```text
× stylistic(chain-per-line): Chain has multiple boundaries on a single line; place each chain segment on its own line.
   ,-[chain-and-mixed-operators.ts:16:19]
 16 | const r = a + (b * obj.d.e.f);
    :                   ^^^^^^^^^
```

A second `oxlint --fix` invocation then applies the chain-per-line fix:

```ts
const r = a + (b * obj.d
  .e
  .f);
```

and the file is clean.
 The convergence test in
`package/oxlint-plugin/stylistic/src/oxlint-stylistic.unit.test.ts`
loops `oxlint --fix` twice for exactly this reason
(`oxlint-stylistic.unit.test.ts:586-606`):

```ts
// Two passes: oxlint applies one fix per overlapping byte region per
// pass, so when no-mixed-operators wraps a region containing the
// chain-per-line target, chain-per-line waits until the next pass.
// Two passes are sufficient for this fixture's rule interaction.
for (const _pass of [0, 1]) {
  try {
    await spawn('oxlint', ['--fix', '-c', FIXTURE_CONFIG, combinedCopy.filePath], { cwd: ROOT });
  }
  catch { /* --fix may exit non-zero when unfixable issues remain */ }
}
```

This is not a problem unique to our plugin.
 The maintainer-side
duplicate at [oxc#20711][20711] was filed by `schoero` against
`eslint-plugin-better-tailwindcss`'s `enforce-consistent-class-order`
plus `enforce-consistent-line-wrapping`,
 with identical symptoms.
 The
issue surfaces whenever two rules' autofixes touch overlapping byte
regions in the same file.

## Root cause

Oxlint's fix pipeline has three steps that combine into the failure:
(1) the JS-to-Rust bridge merges every multi-`Fix` return into a single
`Fix` with the encompassing span,
 (2) the fix-application loop sorts by
that span and applies one per region per pass,
 and (3) `--fix` does not
loop until stable.
 ESLint loops;
 oxlint does not.

Versions pinned for the source citations below:

- `oxlint@1.65.0` (npm) backed by upstream tag
  [`oxlint_v1.65.0`](https://github.com/oxc-project/oxc/releases/tag/oxlint_v1.65.0)
  at commit `25e5cbc76f887cf5c0c2bdfbef8d4a74fd1ce87d`
  (2026-05-15).
- `@oxlint/plugins@1.58.0` (npm).
- All Rust paths in the excerpts below are relative to the oxc repo
  root at that tag.

### Step 1: a JS plugin's `fix(): Fix[]` is merged into one Rust `Fix`

`@oxlint/plugins`'s JS runtime treats the return value of `fix(fixer)`
as either a single `Fix` or an iterable of `Fix`es and converts each to
a `FixReport` with `{ start, end, text }`
(`node_modules/.pnpm/oxlint@1.65.0_oxlint-tsgolint@0.23.0/node_modules/oxlint/dist/lint.js:13827-13833`,
from the released `lint.js`):

```js
if (SymbolIterator in fixes) {
    let fixReports = [];
    for (let fix of fixes) fix && fixReports.push(validateAndConvertFix(fix));
    return fixReports.length === 0 ? null : fixReports;
}
return [validateAndConvertFix(fixes)];
```

The Rust side reads `Vec<JsFix>` for each diagnostic
(`crates/oxc_linter/src/external_linter.rs:80-83`):

```rust
pub start: u32,
pub end: u32,
pub fixes: Option<Vec<JsFix>>,
pub suggestions: Option<Vec<JsSuggestion>>,
```

and routes any non-singleton through `CompositeFix::merge_fixes_fallible`
(`crates/oxc_linter/src/external_linter.rs:155-174`):

```rust
let res = if is_single {
    // ... single-fix fast path ...
} else {
    CompositeFix::merge_fixes_fallible(fixes.collect(), source_text)
};
```

`merge_fixes_fallible` (`crates/oxc_linter/src/fixer/fix.rs:589-651`)
sorts the inputs by span and produces a single `Fix` whose span is the
encompassing range of all the inputs:

```rust
fixes.sort_unstable_by_key(|a| a.span);

// safe, as fixes.len() > 1
let start = fixes[0].span.start;
let end = fixes[fixes.len() - 1].span.end;
// ... weave source slices and fix contents into `output` ...
let mut fix = Fix::new(output, Span::new(start, end));
```

For `no-mixed-operators` on `b * obj.d.e.f`
(`package/oxlint-plugin/stylistic/src/rule/no-mixed-operators.ts:93-114`),
the rule emits

```ts
return [
  fixer.insertTextBeforeRange([offender.start, offender.end], '('),
  fixer.insertTextAfterRange([offender.start, offender.end], ')'),
];
```

which are two zero-width insertions:
 `[b_start, b_start]` carrying `'('`
and `[f_end, f_end]` carrying `')'`.
 After `merge_fixes_fallible`,
 the
diagnostic carries one merged `Fix` with `span = [b_start, f_end]` and
`content = "(b * obj.d.e.f)"`.
 The merge widens the span from "two
zero-width insertions" to "an entire range replacement" even though
the inner bytes are byte-for-byte unchanged.

### Step 2: the fix loop rejects anything inside an applied fix's span

`crates/oxc_linter/src/fixer/mod.rs:381-436` is the fix-application
loop:

```rust
self.messages.sort_unstable_by_key(|m| m.fixes.span());
let mut fixed = false;
let mut output = String::with_capacity(source_text.len());
let mut last_pos: u32 = 0;

for mut m in self.messages {
    let fix = match &m.fixes {
        PossibleFixes::None => None,
        PossibleFixes::Single(fix) => Some(fix),
        PossibleFixes::Multiple(multiple) => multiple.get(self.fix_index as usize),
    };
    let Some(Fix { content, span, .. }) = fix else { /* skip */ };
    let start = span.start;
    let end = span.end;
    // ...
    // Skip fixes that overlap with a previously applied fix. Boundary-adjacent fixes
    // (e.g. [0, 5] and [5, 10]) are considered overlapping to match ESLint's behavior.
    let overlaps = fixed && last_pos >= start;
    if overlaps {
        filtered_messages.push(m);
        continue;
    }

    m.fixed = true;
    fixed = true;
    // ...
    last_pos = end;
}
```

After `no-mixed-operators` applies with span `[b_start, f_end]`,
`last_pos = f_end`.
 `chain-per-line`'s merged fix for `obj.d.e.f` has
span `[e_dot, f_dot]` (the bounding box of the two `\n + indent`
insertions,
 see `package/oxlint-plugin/stylistic/src/rule/chain-per-line.ts:259-267`).
Its `start = e_dot` satisfies `last_pos >= start` (since `f_end >
e_dot`),
 so the chain-per-line message is pushed into
`filtered_messages` and shown to the user as a remaining diagnostic
instead of being applied.

The `PossibleFixes::span()` getter that the sort key calls is
`crates/oxc_linter/src/fixer/fix.rs:381-389`:

```rust
pub fn span(&self) -> Span {
    match self {
        PossibleFixes::None => SPAN,
        PossibleFixes::Single(fix) => fix.span,
        PossibleFixes::Multiple(fixes) => {
            fixes.iter().map(|fix| fix.span).reduce(Span::merge).unwrap_or(SPAN)
        }
    }
}
```

For a JS-plugin diagnostic this is always `Single(merged_fix)` because
of Step 1.
 `PossibleFixes::Multiple` is reserved for native Rust rules
emitting alternative fixes and never reached via the JS plugin path.

### Step 3: `--fix` does not loop until stable

Nothing in `apps/oxlint` or `crates/oxc_linter` re-runs the linter
after a fix-application pass.
 The only fix loop is the one inside
`fixer::Fixer::fix`,
 which is single-pass.
 ESLint,
 by contrast,
 runs
the rules again on the fixed output up to 10 times until no new fixes
appear ([ESLint custom-rules docs][eslint-iter]).
 Without that loop,
any rejected fix stays rejected until the user invokes `oxlint --fix`
again manually.

The earlier hypothesis "maybe the issue is in how we use `createOnce`"
was wrong.
 `createOnce` is a per-plugin perf optimisation
(`crates/oxc_linter/src/external_linter.rs` and the `VisitorWithHooks`
shape in `@oxlint/plugins/index.d.ts:2773-2776`) that lets a plugin
reuse one visitor instance across files plus `before`/`after` lifecycle
hooks.
 It does not change the JS-to-Rust merge path or the fix loop;
swapping to `create` produces the same merge behaviour.
 The limitation
is below the plugin layer.

## Verification

### Reproduction

The combined fixture
`package/test-fixture/oxlint-stylistic/src/invalid/chain-and-mixed-operators.ts`
is the minimal case:

```ts
declare const a: number;
declare const b: number;
declare const obj: { d: { e: { f: number; }; }; };

const r = a + b * obj.d.e.f;

export { r, };
```

with `stylistic/no-mixed-operators` and `stylistic/chain-per-line`
both enabled in
`package/test-fixture/oxlint-stylistic/.oxlintrc.fixture.json`.

Single-pass run (run from repo root):

```bash
cp package/test-fixture/oxlint-stylistic/src/invalid/chain-and-mixed-operators.ts /tmp/cam-test.ts
oxlint --fix -c package/test-fixture/oxlint-stylistic/.oxlintrc.fixture.json /tmp/cam-test.ts
cat /tmp/cam-test.ts
```

Expected output:
 parens applied,
 chain still on one line,
 one
remaining `stylistic(chain-per-line)` diagnostic.
 Repeating the
`oxlint --fix` invocation a second time clears it.

### Patterns that converge in one pass

- Either rule firing alone.
   `chain-per-line` on `obj.b.c.d.toString().trim()`
  or `no-mixed-operators` on `a + b * c` both converge cleanly.
- Two rules firing on disjoint source regions.
   E.g. `array-element-per-line`
  on `[1, 2, 3]` plus `object-property-per-line` on `{a: 1, b: 2}` on
  separate lines.
- Any rule pair where the merged fix spans do not overlap by oxlint's
  rule (`last_pos >= start`).

### Patterns that need two passes

- `stylistic/no-mixed-operators` + `stylistic/chain-per-line` on the
  same expression (the case above and any shape where the wrap encloses
  a chain that needs splitting).
- More generally:
   any rule whose autofix returns `Fix[]` from a single
  diagnostic combined with any other rule whose fix point lands inside
  the bounding box of those fixes.
   The shape is independent of the
  specific rules;
   it follows from the merge in Step 1.

## Verified workarounds

### A. Accept two passes (current state)

What we ship:
 the autofix tests run `oxlint --fix` twice on the
combined fixture,
 and the documentation notes that real-codebase
combined cases need a second `--fix` run.

```ts
// package/oxlint-plugin/stylistic/src/oxlint-stylistic.unit.test.ts:586-606
for (const _pass of [0, 1]) {
  try {
    // oxlint-disable-next-line eslint/no-await-in-loop -- second pass must read the first pass's output from disk
    await spawn('oxlint', ['--fix', '-c', FIXTURE_CONFIG, combinedCopy.filePath], { cwd: ROOT });
  }
  catch { /* --fix may exit non-zero when unfixable issues remain */ }
}
```

Tradeoffs:

- Users invoking `oxlint --fix` from an editor save-on-fix integration
  may need to save twice to fully converge in the rare combined case.
- Diagnostics shown after the first pass include the deferred
  `chain-per-line` warnings;
   the user can be confused about whether
  the fix "worked.
  " The fix did apply once;
   it just did not apply
  twice.
- No code change required;
   nothing to revisit when upstream lands
  [oxc#16118][16118]:
   the second pass becomes a no-op and the test
  loop is harmless.
- The shape extends to any future rule pair that exhibits the same
  conflict;
   the test infrastructure already handles it.

### B. Split diagnostics: one `context.report` per insertion

Refactor `no-mixed-operators` to emit two diagnostics per offender
(one for the opening paren,
 one for the closing paren).
 Each
`context.report` carries a single zero-width `Fix`,
 so
`merge_fixes_fallible` is never invoked and the spans stay zero-width
in the fix loop.
 With zero-width spans at `[b_start, b_start]` and
`[f_end, f_end]`,
 `chain-per-line`'s fix at `[e_dot, f_dot]` sorts
between them and is applied in the same pass.

Sketch:

```ts
// In place of one diagnostic with `fix(): [insertBefore, insertAfter]`:
context.report({
  loc: { /* point at the offender's start */ },
  messageId: 'open',
  fix: (fixer) => fixer.insertTextBeforeRange([offender.start, offender.start], '('),
});
context.report({
  loc: { /* point at the offender's end */ },
  messageId: 'close',
  fix: (fixer) => fixer.insertTextAfterRange([offender.end, offender.end], ')'),
});
```

Tradeoffs:

- User sees two diagnostics per logical issue ("Found 2 errors"
  instead of "Found 1").
   For an `oxlint` invocation the count rises
  by one per affected offender.
- Messages can be made informative ("missing `(` before nested
  mixed-operator expression" / "missing `)` after ...") so the
  duplication reads as a paired pair,
   not as a confused linter.
  Setting `loc` per diagnostic makes the report point at the actual
  missing token,
   which is arguably more useful than the current
  single-diagnostic placement at the parent node.
- No coupling to other rules.
   Generalises:
   any rule whose autofix is
  N disjoint insertions can be refactored this way to dodge
  `merge_fixes_fallible`.
- Single-pass convergence with `chain-per-line` and with any future
  rule that lands inside the offender's bounding box.

### C. Combined rule: merge `no-mixed-operators` and `chain-per-line`

Write a single `stylistic/expression-structure` rule whose visitor
pair covers both `BinaryExpression`/`LogicalExpression` (paren
wrapping) and `MemberExpression`/`CallExpression` (chain splitting),
with the autofix emitting one `replaceTextRange` per logical issue
that handles both concerns in one Fix.

Tradeoffs:

- Loses suppression granularity.
   `oxlint-disable-next-line
  stylistic/no-mixed-operators` no longer exists;
   users disable
  `expression-structure` which covers more concerns than they likely
  want.
   Mitigation:
   keep both rule IDs as aliases that both delegate
  to the combined implementation,
   or expose suppression at the level
  of the diagnostic's `messageId`.
- The combined rule sees both detections in a single visitor pass and
  can emit a coordinated `replaceTextRange` with the wrap-plus-split
  output,
   so the fix span is `[offender.start, offender.end]` once and
  no other rule competes for it.
- The two concerns share enough utility code today (`chain.ts`,
  `has-parens.ts`) that the refactor is mostly visitor-glue;
   the
  detection helpers stay untouched.

### D. Coordinated rules via shared module state

Keep both rules separate but coordinate their autofixes through a
shared utility module that tracks "planned wrap" spans.
 The outer
rule's visitor records its planned wrap;
 the inner rule's visitor
defers (both diagnostic and fix) when its target is inside a planned
wrap.
 The outer rule's autofix then pre-applies the inner rule's
intended split inline.

Sketch (state owned in a new `utility/coordinated-fixes.ts`):

```ts
// utility/coordinated-fixes.ts
const plannedWraps = new Set<string>(); // keyed by `${start}-${end}`

export function planWrap(start: number, end: number): void {
  plannedWraps.add(`${start}-${end}`);
}
export function isInsidePlannedWrap(start: number, end: number): boolean {
  for (const key of plannedWraps) {
    const [s, e] = key.split('-').map(Number);
    if (start >= s && end <= e) return true;
  }
  return false;
}
export function resetPlannedWraps(): void { plannedWraps.clear(); }
```

Each rule clears `plannedWraps` in its `before` hook to handle
per-file resets safely under `createOnce`.
 AST traversal visits
parents before children (pre-order),
 so `no-mixed-operators` on the
outer `BinaryExpression` records the wrap before `chain-per-line`'s
visitor fires on the inner `MemberExpression`.

Tradeoffs:

- Shared mutable state across rule modules;
   module isolation is
  weakened.
   The state is reset per file via `before` hooks,
   but a
  worker thread that processes multiple files in sequence holds the
  state across the boundary,
   which is fragile if the hook does not
  fire (e.g. early-exit in `before`).
- Requires the outer rule's autofix to know how the inner rule
  formats its output (to pre-apply the split).
   The natural place for
  that knowledge is a shared formatter helper,
   which the rules can
  both depend on;
   that helper grows over time as new combined cases
  surface.
- Preserves the one-diagnostic-per-concern UX.
   One `--fix` pass
  converges.

### E. Upstream PR to implement [oxc#16118][16118]

Add the fix-iteration loop to oxlint's core (model on ESLint's "up to
10 iterations").
 This is the path the maintainer accepted in
[their 2026-05-05 comment][16118-camc314] when niieani offered to
contribute,
 with the caveat "it might not get merged/reviewed for a
while - I'm focusing on embedded framework support,
 and this is a
feature that interacts with some of the core of oxlint so it needs a
decent amount of thought put into it.
"

Tradeoffs:

- Highest leverage:
   every oxlint plugin user benefits.
   Our specific
  rule pair becomes a non-issue.
- High latency and review risk.
   No PR exists in the
  `oxc-project/oxc` repo as of 2026-05-20 (verified by
  `gh pr list --search 'fix recursively iterate' --state open`).
  The maintainer's "interacts with the core" hedge suggests review
  cycles before merge.
- The fix is straightforward in shape (iterate `lint -> apply ->
  re-lint` with a cap) but landing it requires care about diagnostic
  deduplication across passes and worker-thread parallelism.
   The
  upstream issue body acknowledges this:
   "Where there are multiple
  rounds,
   we'd need to make sure that only diagnostics from the last
  round are output.
  "

### F. task-oxlint fix-until-stable wrapper (current state for the command path)

`task-oxlint` (the `oxlint` wrapper at
`package/monorepo-task/core-exec/src/oxlint-wrapper.ts`) loops `oxlint --fix` to a
fixpoint when the caller passes `--fix`,
 so a single `task-oxlint --fix` (and the
`format:oxlint` task that calls it) applies the overlapping and fix-induced fixes
that previously needed a manual second run.
 Verified on `oxlint@1.67.0`.

Convergence detection was the hard part.
 `oxlint --fix` signals through neither
its exit code nor its stdout whether a pass changed any file:

- A `--fix` pass that applies fixes and leaves nothing unfixable exits zero with
  `Found 0 warnings and 0 errors.`,
   byte-identical to a genuine no-op pass,
   even
  when its own fix introduced a fresh fixable violation that the next pass
  rewrites.
   Reproduced on `chain-and-mixed-operators.ts`:
   passes two and three
  both exit zero with identical stdout,
   yet each still mutates the file;
   only
  pass four is a true no-op.
- `oxlint --fix --format json` carries `number_of_files`,
   `number_of_rules`,
   and
  `start_time`,
   but no count of fixes applied.

So neither the exit code nor a stdout comparison of the `--fix` run is a sound
stop condition.
 The signal that does track the file is a plain (no-fix) lint of
the post-fix state:
 it reports every remaining violation,
 fixable or not,
 so its
output moves while the file moves and settles when the file stops changing.
 The
loop runs two oxlint invocations per pass:

1.  `oxlint --fix` to apply fixes.
2.  a plain `oxlint` lint (fix flags stripped) as the convergence oracle.

It stops when the oracle reports zero diagnostics (a file with no violation is a
guaranteed fixpoint,
 sound across severities unlike the exit code,
 which is zero
whenever no errors remain even with a fixable warning pending),
 or the oracle's
normalized output matches the previous pass (an unfixable remainder has
stabilized),
 or it matches a non-adjacent earlier pass (an autofix oscillation,
see below),
 or an oxlint run fails to execute,
 or a cap of eight passes is hit.

Normalizing the oracle has two parts,
 both load-bearing:

- The volatile `Finished in <n>ms ...` footer is dropped;
   its duration differs
  every run.
- Diagnostic blocks are sorted.
   Over a multi-file,
   multi-threaded run oxlint
  emits the same blocks in non-deterministic order (verified:
   two identical
  no-fix lints of a 2531-file tree produced the same 41419 lines shuffled).
  Without block-sorting,
   consecutive oracles never compare equal and the loop
  hits the cap on every whole-repo run.

The normalizer only handles oxlint's default reporter.
 `oracleArgs` keeps a
caller-supplied `--format`,
 so `task-oxlint --fix --format json` would compare
JSON carrying the volatile `start_time` and never converge (files still
converge;
 the loop just wastes passes and warns).
 `format:oxlint` always uses
the pinned default reporter,
 so this cannot bite the real task.

The two-runs-per-pass cost is the price of correctness given oxlint exposes no
fix-applied signal;
 workaround (A) already treats running `--fix` twice as the
baseline.
 `--fix` alone triggers the loop:
 `--fix-suggestions` or
`--fix-dangerously` passed without `--fix` stay single-pass,
 because the
plain-lint oracle is not verified to track suggestion-applied changes.
 The loop
lives in `package/monorepo-task/core-exec/src/oxlint-fix-loop.ts` with unit tests
in `oxlint-fix-loop.unit.test.ts`.

This workspace has a real autofix oscillation that the loop surfaces.
`package-paused/webapp-forge/server/src/data/db.ts` flips between two states on
every `--fix`:
 two stylistic rules disagree on the continuation indent of a
chained `.includes(...)` call (four spaces versus six),
 so one rewrites what the
other just wrote.
 `oxlint --fix` over the whole repo therefore cannot converge.
The loop detects the two-state cycle (the oracle revisits an earlier state) at
the third pass and stops with a `cycle` warning naming the conflict,
 rather than
grinding all eight passes.
 Fixing it is a rule-config decision (reconcile the two
rules' indent,
 disable one rule's autofix,
 or exclude paused packages from
`format:oxlint`);
 the wrapper only reports it.

## What does not work

- **External wrapper script as a complete substitute for the upstream fix.
  **
  `task-oxlint` now loops `oxlint --fix` to a fixpoint for the command path (see
  workaround (F)),
   but a wrapper cannot cover the IDE save-on-fix path:
   editors
  call `oxlint --fix` directly and bypass the wrapper.
   The wrapper is the chosen
  fix for the `task-oxlint` and `format:oxlint` path,
   not a replacement for the
  upstream iteration in (E).
- **Use `fixer.replaceTextRange([start, end], '(' + source + ')')`
  from `no-mixed-operators` instead of two insertions.
  ** Produces the
  same merged `Fix` shape:
   one `Fix` with span
  `[offender.start, offender.end]` and the wrapped content.
   The fix
  loop rejects any inner fix exactly as in the current behaviour.
   The
  merge is incidental;
   the encompassing span is fundamental.
- **Reorder the rules so `chain-per-line` sorts first.
  ** The sort key
  is the fix's span `start`.
   Even if `chain-per-line`'s start is
  earlier in some configurations,
   `no-mixed-operators`'s span starts
  at `offender.start <= chain.start` for the relevant case,
   so it
  always sorts first.
   And in the symmetric case (chain-per-line sorts
  first),
   `last_pos = chain.end` rejects `no-mixed-operators` instead.
  Either way one rule wins.
- **Switch from `createOnce` to `create`.
  ** The JS-to-Rust merge
  happens at the `context.report` call,
   not at visitor instantiation.
  Identical merge behaviour for both rule entry points.
- **Use `:exit` traversal.
  ** Defers when the visitor fires (post-order
  rather than pre-order) but does not change how `context.report`'s
  fixes are encoded or how the fix loop applies them.
- **Use suggestions (`suggest: Suggestion[]`) for one rule's fix.
  **
  Suggestions are applied via `--fix-suggestions`,
   not `--fix`.
   Each
  suggestion's fix array is still routed through
  `merge_fixes_fallible`,
   so the same span-widening applies.
   Suggesting
  the wrap means the wrap is not auto-applied;
   suggesting the chain
  split means the chain split is not auto-applied.
   Either is a UX
  regression.
- **Plugin-level fix-priority flag.
  ** No such API exists.
  `RuleMeta.type` (`'problem' | 'suggestion' | 'layout'`) and
  `RuleMeta.fixable` (`'code' | 'whitespace'`) do not affect the
  fix-loop overlap behaviour.

## Why we do not file this upstream

Walking the 5-constraint check explicitly:

1. **Is it really upstream's fault?
   ** Yes.
    The merge in
   `merge_fixes_fallible` and the single-pass fix loop are oxlint
   core;
    the JS plugin API cannot route around them.
2. **Can upstream fix it?
   ** Yes.
    The shape of the fix is well known
   (ESLint's "up to 10 iterations") and the issue body at
   [oxc#16118][16118] sketches it.
    The work touches the fix loop
   itself,
    not the algebraic core.
3. **Are they supporting this use case?
   ** Yes.
    The JS plugin API
   exists precisely so external rules can autofix,
    and multiple
   first-party rules in oxlint compose into autofix (e.g.
   `eslint-plugin-better-tailwindcss` in the duplicate report).
   The maintainers acknowledge the bug.
4. **Will they likely fix it?
   ** Likely but slow.
    niieani offered to
   PR on 2026-05-04 and camc314 accepted on 2026-05-05,
    with the
   caveat that review may take a while because the change "interacts
   with some of the core of oxlint.
   " Verified no in-flight PR as of
   2026-05-20.
5. **Have we prototyped a minimal fix compatible with their
   architecture?
   ** Not applicable.
    The issue already exists,
    the
   maintainer has accepted a future PR,
    and the design path is known.
   Filing another issue or PR ahead of the queued community PR adds
   noise.
    We invest in (A) or one of (B)-(D) locally and inherit the
   upstream fix when it lands.

Decision:
 **do not file**.
 The constraint that would normally push us
to prototype (auto-prototype rule when 1-4 hold) is satisfied by the
existing issues;
 the public record already contains everything a fresh
filing would add.
 The auto-prototype rule's purpose is to make our
upstream report actionable;
 here,
 the actionable report exists.
 Adding
another duplicate would be noise.

Re-evaluate this decision when:

- niieani's offered PR is filed and stalls.
- A new oxlint release ships [oxc#16118][16118] (watch the changelog
  in `npm/oxlint/CHANGELOG.md` in the oxc repo for entries like
  "feat(linter):
   apply fixes recursively").
- A different combined-rule case arises in this workspace and the
  trade-offs in (B)-(D) become preferable to waiting.

## Cross-references

- Upstream tracking:
   [oxc-project/oxc#16118][16118] ("Linter:
   Apply
  fixes recursively"),
   OPEN since 2025-11-25,
   last updated
  2026-05-05.
- Upstream duplicate with our exact shape:
  [oxc-project/oxc#20711][20711] ("linter:
   multipass fixes don't
  work with js plugins"),
   CLOSED 2026-05-02 as duplicate of #16118.
- ESLint's behaviour we are diverging from:
  [ESLint custom-rules docs][eslint-iter].
- Our rules and tests:
  - `package/oxlint-plugin/stylistic/src/rule/no-mixed-operators.ts:93-114`
  - `package/oxlint-plugin/stylistic/src/rule/chain-per-line.ts:251-268`
  - `package/test-fixture/oxlint-stylistic/src/invalid/chain-and-mixed-operators.ts`
  - `package/oxlint-plugin/stylistic/src/oxlint-stylistic.unit.test.ts:566-617`
- Width-conflict oscillation,
  a failure class beside the span-overlap one recorded here:
  when
  `chain-per-line`'s renderer emitted one flat region indent while `stylistic/indentation` expected
  per-anchor nesting,
  fixture b8 looped under `--fix` until commit `310a3fa94c` unified chain rendering
  with the indent model.
  Record:
  `doc/troubleshooting/oxlint-typescript-indent-rule.md`.
- Issue this implementation closed:
   [Aquaticat/Monochromatic#209][issue-209].

[16118]: https://github.com/oxc-project/oxc/issues/16118
[16118-camc314]: https://github.com/oxc-project/oxc/issues/16118#issuecomment-4378431486
[20711]: https://github.com/oxc-project/oxc/issues/20711
[eslint-iter]: https://eslint.org/docs/latest/extend/custom-rules#applying-fixes
[issue-209]: https://github.com/Aquaticat/Monochromatic/issues/209
