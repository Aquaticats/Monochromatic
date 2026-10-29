# Oxlint 1.85.0 `unicorn/no-useless-spread` and `unicorn/prefer-spread` safe fixes rewrite strings, typed arrays, and iterators into different values

## Symptom

`oxlint --fix` rewrites spreads and copies by method name,
without knowing the receiver type.
Plain `--fix` applies every rewrite,
and afterwards nothing reports the changed meaning.

`unicorn(no-useless-spread)` reports,
at error level through the `correctness` category:

```text
x unicorn(no-useless-spread): Using a spread operator here creates a new array unnecessarily.
  help: `text.slice` returns a new array. Spreading it into an array expression to create a new array is redundant.
```

`unicorn(prefer-spread)` reports:

```text
Prefer the spread operator (`...`) over array.slice()
Prefer the spread operator (`...`) over Array.from()
```

Surface patterns and what `--fix` turns them into:

- `[...text.slice(0, 3)]` on a string becomes `text.slice(0, 3)`:
  a string instead of an array of code points
  (repo issue 563;
  TS2339 when `.some` follows).
- `[...scoopGrams.map(f)]` on a `Uint8Array` becomes `scoopGrams.map(f)`:
  a `Uint8Array`,
  so later values wrap at 256;
  silent when the type is inferred.
- `[...queue.filter(f)]` on an iterator helper becomes `queue.filter(f)`:
  a lazy iterator instead of an array.
- `Array.from(scoopGrams.map(f))` becomes `[...scoopGrams.map(f)]` through `prefer-spread`,
  then `scoopGrams.map(f)` through `no-useless-spread` on the next pass.
- `scoopGrams.slice()` on a typed array becomes `[...scoopGrams]`:
  `number[]` instead of a typed-array copy
  (repo issue 565).
- `text.slice()` on a string becomes `[...text]`,
  which `typescript(no-misused-spread)` then reports.

## Root cause

### `no-useless-spread` guesses "new array" from the method name

`is_functional_array_method` matches the name on any receiver,
`crates/oxc_linter/src/rules/unicorn/no_useless_spread/const_eval.rs:229-249`:

```rust
fn is_functional_array_method(call_expr: &CallExpression) -> bool {
    is_method_call(
        call_expr,
        None,
        Some(&[
            "concat", "copyWithin", "filter", "flat", "flatMap", "map",
            "slice", "splice", "toReversed", "toSorted", "toSpliced", "with",
        ]),
        None,
        None,
    )
}
```

`split` and `reduce(callback, [])` are classified the same way (`const_eval.rs:178-185`,
`:261-263`).
Only receivers written as `new Uint8Array(…)` or `Uint8Array.from(…)` are excluded (`const_eval.rs:154-172`),
so a typed array held in an identifier or property,
a string,
or an iterator is treated as an array.

### The dangerous-fix declaration does not reach the emitted fix

The rule declares itself `fix_dangerous`
(`crates/oxc_linter/src/rules/unicorn/no_useless_spread/mod.rs:146`),
but the clone check emits with `diagnostic_with_fix`
(`mod.rs:431-433`):

```rust
ctx.diagnostic_with_fix(clone(span, is_array, name), |fixer| {
    fix_by_removing_array_spread(fixer, &array_or_obj_span, spread_elem)
});
```

`diagnostic_with_fix` hard-codes the safe kind,
`crates/oxc_linter/src/context/mod.rs:306-311`:

```rust
pub fn diagnostic_with_fix<C, F>(&self, diagnostic: OxcDiagnostic, fix: F)
{
    self.diagnostic_with_fix_of_kind(diagnostic, FixKind::SafeFix, fix);
}
```

and whether a fix applies depends on the emitted fix's kind,
`context/mod.rs:519`:

```rust
if self.parent.fix.can_apply(rule_fix.kind()) && !rule_fix.is_empty() {
```

`DangerousFix` is `Dangerous | Fix` (`crates/oxc_linter/src/fixer/fix.rs:44`),
so a rule declared dangerous may still emit safe fixes,
and plain `--fix` applies them.
The declaration is metadata only.

### `prefer-spread` guesses arrays from spelling

The `slice` arm (`crates/oxc_linter/src/rules/unicorn/prefer_spread.rs:107-152`) skips array literals,
`this`,
literal receivers,
direct `new Uint8Array(…)`/`new ArrayBuffer(…)` constructions,
and the identifier names `arrayBuffer`,
`blob`,
`buffer`,
`file`,
and `this` (`:173`).
For any other identifier,
`is_not_array` (`:203-256`) follows a `const` initializer and otherwise falls back to capitalization:

```rust
if ident.starts_with(|c: char| c.is_ascii_uppercase())
    && ident.cow_to_ascii_uppercase() != ident
{
    return true;
}
false
```

A `new Uint8Array(4)` initializer reaches `_ => return false`,
so `const scoopGrams = new Uint8Array(4); scoopGrams.slice()` and a string parameter `text.slice()` are both rewritten.
The fix is emitted with `ctx.diagnostic_with_fix` (`prefer_spread.rs:265`),
a safe fix,
under a `conditional_fix` declaration (`:46`).

### An earlier reading that was wrong

The first diagnosis in repo issue 563 treated the problem as `.slice` alone on strings.
The catalog in "Verification" shows the same rewrite on typed-array `map` and iterator `filter`,
and the two-rule chain through `Array.from`,
so any fix limited to `slice` receivers leaves most failures in place.

## Verification

Oxlint 1.85.0 (`node_modules/.pnpm/oxlint@1.85.0_oxlint-tsgolint@7.0.2002`),
oxc source at commit `f51e678` (2026-09-25),
after the last commits to these rules
(`no_useless_spread`:
`03ef0f27b`,
2026-08-24;
`prefer_spread.rs`:
`feb733b5a`,
2026-09-22).

```json
// .oxlintrc.json
{
  "plugins": ["unicorn"],
  "categories": { "correctness": "off" },
  "rules": { "unicorn/no-useless-spread": "error", "unicorn/prefer-spread": "error" }
}
```

```ts
// catalog.ts
declare const text: string;
declare const scoopGrams: Uint8Array;
declare const queue: IteratorObject<number>;
declare const toppings: readonly string[];
declare const source: Iterable<number>;

export const codePoints = [...text.slice(0, 3,),];
export const doubled = [...scoopGrams.map(Number,),];
export const firstThree = [...queue.filter(Boolean,),];
export const fromTyped = Array.from(scoopGrams.map(Number,),);
export const scoopCopy = scoopGrams.slice();
export const codeCopy = text.slice();
export const toppingCopy = [...toppings.slice(0, 2,),];
export const fromArray = [...Array.from(source,),];
```

```sh
oxlint --fix catalog.ts
oxlint --fix catalog.ts
```

### Rewrites that keep behavior

- `[...toppings.slice(0, 2)]` becomes `toppings.slice(0, 2)`.
- `[...Array.from(source)]` becomes `[...source]`.

### Rewrites that change the value

`unicorn(no-useless-spread)`:

- `[...text.slice(0, 3)]` becomes `text.slice(0, 3)`.
- `[...scoopGrams.map(Number)]` becomes `scoopGrams.map(Number)`.
- `[...queue.filter(Boolean)]` becomes `queue.filter(Boolean)`.

`unicorn(prefer-spread)`:

- `scoopGrams.slice()` becomes `[...scoopGrams]`.
- `text.slice()` becomes `[...text]`.

Both,
across two passes:

- `Array.from(scoopGrams.map(Number))` becomes `scoopGrams.map(Number)`.

## Verified workarounds

### Project replacement rules with progressive type evidence

`no-restricted-syntax/no-useless-spread` and `no-restricted-syntax/prefer-spread`
(`package/oxlint-plugin/no-restricted-syntax/src/rule/`)
replace both upstream rules in `@monochromatic-dev/config-oxlint`.
They keep every upstream check whose fix syntax proves safe,
ask the TypeScript 7 semantic bridge about ambiguous receivers,
fix only proven arrays,
leave proven typed arrays and strings alone,
and report untyped ambiguous code without a fix.
Tests:
`package/oxlint-plugin/no-restricted-syntax/src/spread-rules.unit.test.ts`.
Tradeoffs:

- Files outside every tsconfig project,
  and `any`-typed receivers,
  get reports without fixes where upstream fixed.
- The bridge costs one TypeScript 7 process,
  shared with the readonly plugin through one bundled chunk.
- Like upstream,
  fixes on proven arrays assume dense arrays:
  `[...sparse]` fills holes that `sparse.slice()` keeps.

### Turning the upstream rules off

Removes every unsafe rewrite with no new code.
Tradeoff:
loses the checks that are always safe
(`[a, ...[b]]`,
`new Set([...x])`,
`[...Array.from(x)]`),
which the replacement keeps.

## What does not work

- Guidance text in the repo's oxlint wrapper
  (`package/monorepo-task/core-exec/src/oxlint-guidance.ts`):
  the wrapper prints only the final lint after its fix loop
  (`package/monorepo-task/core-exec/src/oxlint-wrapper.ts:254-277`),
  so an autofixed diagnostic and its guidance never reach the author.
- Disabling one rule's autofix:
  oxlint has only global `--fix`,
  `--fix-suggestions`,
  and `--fix-dangerously` flags;
  the editor-only `rulesCustomization.<rule>.autofix` setting does not affect the CLI.
- Rule options:
  neither `NoUselessSpread` nor `PreferSpread` reads configuration.
- Waiting for upstream PRs 26160 and 26615:
  both are unreviewed,
  and neither covers iterator helpers,
  `map`/`filter` on typed arrays held in identifiers,
  `split`,
  `reduce`,
  or the fix-kind mismatch.

## Upstream filing artifact

### Upstream filing decision

`.out-of-scope/` has no entry for oxlint or oxc lint rules
(same file list as `doc/troubleshooting/oxlint-js-plugin-global-reference-env.md`).

#### `no-useless-spread`: duplicate of oxc issue 26159

Issue 26159 (open,
filed against 1.79.0) reports the string `.slice` case.
Pull requests 26160 and 26615 are open and unreviewed.
Reading both threads and diffs in full:

- 26160 classifies `slice` and `concat` receivers and withholds the fix for unknown `slice` receivers;
- 26615 bails for literal,
  `new Foo()`,
  and `const`-to-literal receivers,
  and excludes typed arrays from that bail.

Neither covers iterator helpers (`[...it.filter(f)]`),
`map`/`filter` on typed arrays held in identifiers,
`split`,
`reduce(callback, [])`,
or the fix-kind mismatch;
the issue itself calls `[...nif.split('')]` "fixed correctly",
which holds only for strings.
Those are additive,
so the artifact is a comment on issue 26159,
not a new issue.

1. **Upstream's fault:**
   yes;
   the rule declares `fix_dangerous` but emits safe fixes from name guesses.
2. **Upstream can fix it:**
   yes;
   the prototype below is 86 lines including tests.
3. **Supported use case:**
   yes;
   the rule targets TypeScript and JavaScript,
   and typed-array receivers were already fixed once (pull request 26067 for issue 25868).
4. **Contribution welcome:**
   yes,
   with disclosure (`CONTRIBUTING.md:12-21`);
   both open pull requests disclose AI assistance.
5. **Likely to fix:**
   plausible;
   issue 24107 was closed as not planned with a maintainer stating no plans to make the rule type-aware,
   which the fix-kind approach does not need.
6. **Minimal fix prototyped:**
   yes.
   [oxlint-spread-autofix.no-useless-spread.patch](oxlint-spread-autofix.no-useless-spread.patch),
   against `oxc-project/oxc` `f51e67812ed15cea95c9bddfe614d889685f0703`:
   `is_method_name_guess` in `const_eval.rs` marks hints that come only from a method name
   (`split`,
   the functional array methods,
   `reduce`),
   and `check_useless_clone` emits those as `FixKind::DangerousFix`,
   keeping `Array.from`,
   `Array.of`,
   `Object.keys`/`values`/`entries`,
   `await Promise.all`,
   and `new Array(n)` safe.
   Verified in
   `podman run --rm --memory=6g --cpus=4 … docker.io/library/rust:latest cargo test -p oxc_linter no_useless_spread`
   (rustc 1.98.1):
   post-patch the rule test passes with no snapshot change;
   with the new tests kept and the kind forced back to `SafeFix`,
   it fails on exactly the four name-guess cases:

   ```text
   Input: [...text.slice(0, 3)]  Expected: [...text.slice(0, 3)]  Actual: text.slice(0, 3)
   Input: [...text.split("|")]   Expected: [...text.split("|")]   Actual: text.split("|")
   Input: [...arr.reduce(...)]   Actual: arr.reduce((a, b) => a.push(b), [])
   Input: [...(foo ? x.map(f) : [])]  Actual: (foo ? x.map(f) : [])
   ```

   Not run:
   clippy and `cargo fmt`.

All six hold.
The comment is fileable once the person posting re-runs the reproduction and fills in the disclosure bracket;
posting needs the user's approval.

~~~md
Two gaps beyond the string `.slice` case, reproduced on oxlint 1.85.0 (and on `main` at f51e678):

1. The rewrite is not limited to `.slice`/`.concat`. Every name in `is_functional_array_method`
   (`const_eval.rs:229-249`) plus `split` and `reduce(cb, [])` is classified `NewArray` on any receiver, so plain
   `--fix` also does:
   - `[...typed.map(f)]` → `typed.map(f)` for a typed array held in an identifier (`Uint8Array`, values now wrap)
   - `[...iter.filter(f)]` → `iter.filter(f)` for iterator helpers (lazy iterator instead of array)
   Neither #26160 nor #26615 covers these receivers.

2. The rule is declared `fix_dangerous` (`no_useless_spread/mod.rs:146`), but `check_useless_clone` emits via
   `ctx.diagnostic_with_fix` (`mod.rs:431`), which is `FixKind::SafeFix` (`context/mod.rs:306-311`), and
   `finish_create_fix` gates on the emitted kind (`context/mod.rs:519`). So these name-guess rewrites run under plain
   `--fix`, not only `--fix-dangerously`.

A minimal fix that needs no type information: emit the clone fix as `DangerousFix` when the hint comes only from a
method name, keeping certain cases (`Array.from/of`, `Object.keys/values/entries`, `await Promise.all`,
`new Array(n)`) safe. Prototype (86 lines incl. tests) passes `cargo test -p oxc_linter no_useless_spread`; with the
new tests and the old kind it fails on exactly `[...text.slice(0, 3)]`, `[...text.split("|")]`,
`[...arr.reduce(..., [])]`, `[...(foo ? x.map(f) : [])]`:

<details><summary>patch</summary>

(paste doc/troubleshooting/oxlint-spread-autofix.no-useless-spread.patch)

</details>

AI disclosure: investigation, source trace, and prototype were done with an AI coding assistant.
[Filer: re-run the reproduction and the cargo test yourself, then replace this bracket with what you verified.]
~~~

#### `prefer-spread`: typed-array and string `slice()` receivers

Duplicate search in oxc issues and pull requests,
open and closed,
for `prefer-spread` with `slice`,
`typed array`,
`Uint8Array`,
and `string`:
pull request 19478 (merged 2026-02-17) covers direct `new Uint8Array(…)`/`new ArrayBuffer(…)` receivers,
pull request 23520 (merged 2026-06-16) skips literal and other syntactically non-array receivers,
pull request 26935 (merged 2026-09-22,
after 1.85.0) changed the rule again,
and issue 26873 (closed) covers `String#split('')`.
The prototype's clone includes all of them,
and identifier-held typed arrays and string parameters are still rewritten there,
so none is a candidate fix for this failure,
and no open report covers it.
The artifact is a new issue.

1. **Upstream's fault:**
   yes;
   the fix is a safe fix chosen from spelling heuristics.
2. **Upstream can fix it:**
   yes;
   the prototype below touches one file.
3. **Supported use case:**
   yes;
   upstream already excludes direct typed-array and buffer receivers,
   so typed arrays are an intended non-target.
4. **Contribution welcome:**
   yes,
   with disclosure (`CONTRIBUTING.md:12-21`).
5. **Likely to fix:**
   plausible;
   the rule receives regular receiver-exclusion fixes (pull requests 19478,
   23520,
   26935).
6. **Minimal fix prototyped:**
   yes.
   [oxlint-spread-autofix.prefer-spread.patch](oxlint-spread-autofix.prefer-spread.patch)
   (`crates/oxc_linter/src/rules/unicorn/prefer_spread.rs`,
   110 insertions and 8 deletions,
   against `f51e67812ed15cea95c9bddfe614d889685f0703`):
   the rule becomes `conditional_dangerous_fix`,
   as `no_null` and `prefer_array_flat` already declare;
   `slice`/`toSpliced` rewrites are `SafeFix` only when `is_known_array` proves the receiver
   (array literal,
   `Array.from`/`Array.of`,
   or a `const` bound to one),
   otherwise `DangerousFix`;
   `is_not_array` also returns true for `new <TypedArray|ArrayBuffer|SharedArrayBuffer>(…)`,
   covering `const` initializers;
   `Array.from(x)` stays safe.
   Verified with
   `podman run --rm --memory=6g --cpus=4 … docker.io/library/rust:latest cargo test -p oxc_linter prefer_spread`:
   post-patch both `prefer_spread` tests pass with no snapshot change;
   the control (new tests,
   old behavior) fails on exactly the new cases:

   ```text
   2 test cases expected to pass, but failed:
      1. const b = new Uint8Array(4); b.slice()
      2. const buf = new ArrayBuffer(8); buf.slice(0)
   5 fixes did not produce expected output:
      declare const scoopGrams: Uint8Array; scoopGrams.slice() -> ...; [...scoopGrams]
      function f(text: string) { return text.slice() } -> ... return [...text] }
      array.slice(0) -> [...array]
      array.toSpliced() -> [...array]
      let a = [1]; a.slice() -> let a = [1]; [...a]
   ```

   Not handled:
   holes in a proven array literal (`[1,,2]`) become `undefined` under the spread,
   and a local binding named `Array` would pass `is_known_array`.
   `toSpliced` exists only on `Array.prototype` among built-ins,
   so reviewers may prefer keeping it safe.
   Formatting was wrapped by hand;
   the container had no rustfmt.

All six hold.
The draft is fileable once the person filing re-runs the reproduction and fills in the disclosure bracket;
posting needs the user's approval.

~~~md
Title: linter(unicorn/prefer-spread): safe fix rewrites `.slice()` on typed arrays and strings held in variables

Template: `.github/ISSUE_TEMPLATE/linter_bug_report.yaml` (fill its version, config, and reproduction fields
from the sections below)

### Description

On oxlint 1.85.0 (and `main` at f51e678), plain `--fix` rewrites `x.slice()` / `x.slice(0)` / `x.toSpliced()` to
`[...x]` whenever `is_not_array` (`prefer_spread.rs:203-256`) cannot rule out an array. It follows a `const`
initializer and otherwise falls back to identifier capitalization, so:

- `declare const scoopGrams: Uint8Array; scoopGrams.slice()` → `[...scoopGrams]`: a `number[]` instead of a
  `Uint8Array` copy; later writes of 300 store 300 instead of wrapping to 44.
- `const b = new Uint8Array(4); b.slice()` → `[...b]`: the `new Uint8Array` initializer reaches
  `_ => return false`; only direct `new Uint8Array(...).slice()` is excluded (`:176-201`).
- `function f(text: string) { return text.slice(); }` → `[...text]`: an array of code points.

The fix is emitted with `ctx.diagnostic_with_fix` (`:265`), i.e. `FixKind::SafeFix`, under `conditional_fix` (`:46`).

### Reproduction

```json
{ "plugins": ["unicorn"], "categories": { "correctness": "off" }, "rules": { "unicorn/prefer-spread": "error" } }
```

```ts
declare const scoopGrams: Uint8Array;
declare const text: string;
export const scoopCopy = scoopGrams.slice();
export const codeCopy = text.slice();
```

`oxlint --fix` produces `[...scoopGrams]` and `[...text]`.

### Suggested fix

Keep the diagnostic, but make the `slice`/`toSpliced` rewrite a `DangerousFix` unless the receiver is provably an
array (array literal, `Array.from`/`Array.of`, or a `const` bound to one), and let `is_not_array` return `true` for
`new <TypedArray|ArrayBuffer|SharedArrayBuffer>(...)` initializers. Prototype (one file, +110/-8 incl. tests,
category `conditional_dangerous_fix` as in `no_null`/`prefer_array_flat`) passes `cargo test -p oxc_linter
prefer_spread`; with the old behavior the new cases fail as expected:

<details><summary>patch</summary>

(paste doc/troubleshooting/oxlint-spread-autofix.prefer-spread.patch)

</details>

AI disclosure: investigation, source trace, and prototype were done with an AI coding assistant.
[Filer: re-run the reproduction and the cargo test yourself, then replace this bracket with what you verified.]
~~~
