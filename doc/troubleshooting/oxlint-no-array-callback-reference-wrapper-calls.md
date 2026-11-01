# oxlint 1.72.0 `unicorn/no-array-callback-reference` reports arity-wrapper calls

## Symptom

With oxlint 1.72.0 and the built-in `unicorn/no-array-callback-reference` rule enabled,
array iterator callbacks wrapped through an explicit arity helper are still reported:

```typescript
// doc/troubleshooting/oxlint-no-array-callback-reference-wrapper-calls.md
const index = items.findIndex(unary(isBig,),);
```

The diagnostic is the same as a bare callback reference:

```text
Avoid passing a function reference directly to iterator methods
help: Wrap the function in an arrow function to explicitly pass only the element argument
```

That is a mismatch for this workspace because arrows are banned locally and `unary(isBig,)`
already builds a wrapper whose runtime callback receives only one argument.
An inline named function expression is clean:

```typescript
// doc/troubleshooting/oxlint-no-array-callback-reference-wrapper-calls.md
const index = items.findIndex(function probeC(item,): boolean {
  return isBig(item,);
},);
```

## Root cause

`pnpm-lock.yaml:5444` to `pnpm-lock.yaml:5445` pins oxlint 1.72.0 with npm integrity
`sha512-1rhdZIP/EvoI91ABIwNU5Q8+bWf8mjrS5UzIOZld4d4bXxJvtlUhlQvaoTogIGin/qdErMOrwaIJvCSIAKTLhA==`:

```yaml
# pnpm-lock.yaml
oxlint@1.72.0:
  resolution: {integrity: sha512-1rhdZIP/EvoI91ABIwNU5Q8+bWf8mjrS5UzIOZld4d4bXxJvtlUhlQvaoTogIGin/qdErMOrwaIJvCSIAKTLhA==, tarball: https://registry.npmjs.org/oxlint/-/oxlint-1.72.0.tgz}
```

The installed config schema exposes the rule as `RuleNoConfig`,
 so there is no local option
that can say "allow wrapper call expressions".
`node_modules/oxlint/configuration_schema.json:8692` to
`node_modules/oxlint/configuration_schema.json:8694` says:

```json
// node_modules/oxlint/configuration_schema.json
"unicorn/no-array-callback-reference": {
  "$ref": "#/definitions/RuleNoConfig"
},
```

The cloned upstream source at commit `6ab439a9025f3e67af672ef1d281746a8ff225fd`
shows the same generated TypeScript type.
`apps/oxlint/src-js/package/config.generated.ts:1507` says:

```typescript
// apps/oxlint/src-js/package/config.generated.ts
"unicorn/no-array-callback-reference"?: RuleNoConfig;
```

The Rust rule only visits method calls whose names are array iterator names and whose
argument count is one or two.
`crates/oxc_linter/src/rules/unicorn/no_array_callback_reference.rs:66` to
`crates/oxc_linter/src/rules/unicorn/no_array_callback_reference.rs:89` says:

```rust
// crates/oxc_linter/src/rules/unicorn/no_array_callback_reference.rs
let is_relevant_method = is_method_call(
    call_expr,
    None,
    Some(&[
        "every",
        "filter",
        "find",
        "findLast",
        "findIndex",
        "findLastIndex",
        "flatMap",
        "forEach",
        "map",
        "some",
    ]),
    Some(1),
    Some(2),
) || is_method_call(
    call_expr,
    None,
    Some(&["reduce", "reduceRight"]),
    Some(1),
    Some(2),
);
```

The callback classifier then treats ordinary call expressions as reportable,
except `.bind(...)` calls.
`crates/oxc_linter/src/rules/unicorn/no_array_callback_reference.rs:124` to
`crates/oxc_linter/src/rules/unicorn/no_array_callback_reference.rs:140` says:

```rust
// crates/oxc_linter/src/rules/unicorn/no_array_callback_reference.rs
fn should_wrap_callback(expr: &Expression) -> bool {
    match expr {
        Expression::Identifier(ident) if is_allowed_builtin(&ident.name) => false,
        Expression::ConditionalExpression(cond_expr) => {
            should_wrap_callback(&cond_expr.consequent)
                || should_wrap_callback(&cond_expr.alternate)
        }
        Expression::CallExpression(call_expr) => {
            if let Some(member_expr) = call_expr.callee.get_member_expr()
                && let Some(prop_name) = member_expr.static_property_name()
                && prop_name == "bind"
            {
                return false;
            }

            true
        }
```

So `items.findIndex(unary(isBig,),)` is not configurable in oxlint 1.72.0:
the first argument is an `Expression::CallExpression`,
 and `should_wrap_callback` returns `true`.

## Verification

The replacement rule is covered by fixtures in
`package/test-fixture/oxlint-no-restricted-syntax/src/`:

- `invalid/no-array-callback-reference.ts` keeps the unsafe cases:
  multi-parameter `findIndex(hasIndexFootgun,)`,
  multi-parameter `filter(probe.hasIndexFootgun,)`,
  and unknown wrapper call `some(makePredicate(isBig,),)`.
- `valid/no-array-callback-reference.ts` proves the requested clean cases:
  direct unary `findIndex(isBig,)`,
  member unary `findIndex(probe.isBig,)`,
  `findIndex(unary(isBig,),)`,
  `map(binary(...),)`,
   an inline named `function probeC(...)`,
   and `map(Number,)`.

The package unit test command exercised both catalogs:

```sh
# /var/home/user/Monochromatic
mise run //package/oxlint-plugin/no-restricted-syntax:test:unit
```

Observed passing suites included:

```text
[oxlint-no-restricted-syntax] [valid fixtures] PASS ... no-array-callback-reference accepts explicit arity wrapper calls ...
[oxlint-no-restricted-syntax] [substantive rules] PASS ... reports no-array-callback-reference when violated ...
```

The implementation also passed the package oxlint and type checks:

```sh
# /var/home/user/Monochromatic
mise run //package/oxlint-plugin/no-restricted-syntax:lint:oxlint
mise run //package/oxlint-plugin/no-restricted-syntax:lint:types
```

Both completed with zero reported lint and type errors.

## Verified workarounds

### Replace the built-in rule at the workspace boundary

The landed workaround disables the built-in rule and enables a project-owned JS plugin rule:

```typescript
// package/config/oxlint/src/rule/restriction.ts
'unicorn/no-array-callback-reference': 'off',
'no-restricted-syntax/no-array-callback-reference': 'warn',
```

Tradeoff:
 this preserves the repo's direct-reference guard but means future upstream changes to
`unicorn/no-array-callback-reference` will not automatically apply.
 The project rule copies only
the shapes this workspace needs:
 direct identifiers and local object-literal member references with statically-known unary arity,
 conditional and sequence expressions containing direct references,
 `unary(...)` and `binary(...)` callback-position calls as explicit arity wrappers,
 and the common non-array receiver allowlist.
 It does not inspect imported function declarations or arbitrary wrapper call return values.

### Use an inline named function expression

This works with both upstream and local policy:

```typescript
// doc/troubleshooting/oxlint-no-array-callback-reference-wrapper-calls.md
items.findIndex(function probeC(item,): boolean {
  return isBig(item,);
},);
```

Tradeoff:
 this is verbose when an existing named function already expresses the domain concept.

### Use `unary` or `binary` after the local replacement

This now works under the project rule:

```typescript
// doc/troubleshooting/oxlint-no-array-callback-reference-wrapper-calls.md
items.findIndex(unary(isBig,),);
items.map(binary(render,),);
```

Tradeoff:
 this relies on `@monochromatic-dev/function-arity-core` being available to the
consumer package,
 and it intentionally hides later extra iterator arguments from the wrapped
function.

## What does not work

### Configuring the built-in rule

`RuleNoConfig` in the installed schema and upstream generated config type means there is no
`allowCallExpression`,
 `allowWrapper`,
 or equivalent setting for
`unicorn/no-array-callback-reference` in oxlint 1.72.0.

### Keeping `unary(isBig,)` while the built-in rule stays enabled

The upstream classifier returns `true` for `Expression::CallExpression` except `.bind(...)`,
so an arity wrapper remains reportable when the built-in rule runs.

### Following the built-in arrow-function help text

The help text asks for an arrow callback,
 but `no-restricted-syntax/no-arrow-function` bans arrows
in this workspace.
 The local fix is an inline named function expression or the project-owned rule
plus `unary`/`binary`.

## Upstream filing decision

`.out-of-scope/` was checked.
 No exemption mentions oxlint or this rule.
The upstream tracker was searched with these commands:

```sh
# /var/home/user/Monochromatic
gh search issues --repo oxc-project/oxc 'no-array-callback-reference unary call expression' --limit 10
gh search prs --repo oxc-project/oxc 'no-array-callback-reference unary call expression' --limit 10
gh search issues --repo oxc-project/oxc 'no-array-callback-reference' --limit 20
gh search prs --repo oxc-project/oxc 'no-array-callback-reference' --limit 20
```

The narrow search returned no matches.
 The broad search found related false-positive work for
Effect array-like methods,
 especially `oxc-project/oxc#18876` and `oxc-project/oxc#19633`,
but not the arity-wrapper policy question.

### Six-constraint check

1.  Is it really upstream's fault?
    No.
    The rule's documented help and source intentionally classify non-`.bind(...)` call expressions
    as callbacks that should be wrapped.
    This repo wants a different policy because arrows are banned and `unary`/`binary` are explicit
    arity wrappers.
2.  Can upstream fix it?
    Yes in principle,
    by adding an option or changing the classifier,
    but constraint 1 fails because the current behavior is not clearly an upstream bug.
3.  Are they supporting this use case?
    Not directly.
    The schema is `RuleNoConfig`,
    and the help text points to arrow callbacks,
    not named-function or arity-helper workflows.
4.  Would the repo welcome our contribution?
    The cloned `CONTRIBUTING.md` says contributions are welcome and permits AI assistance with
    disclosure,
    review,
    and testing.
    The linter bug template exists at `.github/ISSUE_TEMPLATE/linter_bug_report.yaml`.
    This constraint would pass for a real bug report,
    but it does not override constraint 1.
5.  Will they likely fix it?
    Unknown.
    The broad tracker search shows maintainers fixed the related Effect false positive,
    but there is no signal that they want project-specific wrapper policy in the built-in rule.
6.  Have we prototyped a minimal fix compatible with their architecture?
    No,
    because constraints 1 and 3 fail.
    The minimal local fix is implemented at the consumer boundary instead.

### Filing artifact

Do not file as-is.
 This is a project policy replacement,
 not an upstream bug report.
If future evidence shows upstream wants configurable callback-wrapper policy,
 file a feature request
rather than a bug.

~~~md
Title: linter: make unicorn/no-array-callback-reference configurable for wrapper call expressions

Context:
oxlint 1.72.0 exposes unicorn/no-array-callback-reference as RuleNoConfig, and the Rust classifier
reports Expression::CallExpression callbacks except .bind(...). That means array.findIndex(unary(fn))
is reported even though unary(fn) returns a wrapper with explicit arity.

Why this is not currently fileable:
This is a local policy mismatch, not a demonstrated upstream bug. The current help text recommends
an arrow wrapper, and the current source intentionally reports ordinary call expressions.

Potential feature request if upstream wants it:
Add an option that allows selected wrapper calls,
for example unary(...) and binary(...),
while keeping bare identifiers,
member expressions,
and arbitrary wrapper calls reportable.
~~~
