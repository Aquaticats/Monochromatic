# hcl-rs 0.19.8: evaluating `% 0` panics, integer overflow wraps silently, and `for` over an object is unsorted

`hcl::eval` answers three classes of ordinary HCL expressions differently from the HashiCorp
reference implementation,
and one of them aborts the process instead of returning an error.
All three are reachable from a configuration file a user writes,
so a tool that evaluates user HCL with this crate can crash or can produce a wrong value with no
diagnostic.

Found while designing meow's HCL evaluator
(`doc/planning/monorepo-manager-from-scratch-design.md`,
"Declarative configuration").
The three are kept in one document because they share one cause family:
`Value::Number` is a `u64`/`i64`/`f64` union evaluated with plain Rust operators,
and object values are an insertion-ordered map iterated in insertion order.

Status 2026-09-17:
unchanged at upstream `HEAD` `f7b17594f8d738343508f87dc9a24018e7fbeb8d` (2026-09-06),
reproduced there and in the published crate `hcl-rs` 0.19.8.

## Symptom

- `x = 5 % 0` panics,
  in release builds too.
   The panic message is `attempt to calculate the remainder with a divisor of zero`.
  With the default `panic = "unwind"` the panic can be caught;
   with `panic = "abort"`,
  which single-binary release profiles often use,
   the process dies with no diagnostic naming the file or the expression.
  The reference implementation returns `5`.
- `x = 1 / 0` returns `1.797693134862316e308` in a release build
   and panics with `assertion failed: value.is_finite()` in a build with debug assertions.
  The reference implementation returns positive infinity.
- Integer arithmetic wraps instead of widening.
   `x = 18446744073709551615 + 1` evaluates to `0`,
   and `x = 4611686018427387904 * 4` evaluates to `0`,
   where the reference implementation returns `18446744073709551616` for both.
  With debug assertions the same expressions panic with `attempt to add with overflow` and
   `attempt to multiply with overflow`.
- Large non-integer results saturate.
   `x = 10000000000000000000.5 * 2` evaluates to `18446744073709551614`,
   where the reference implementation returns `20000000000000000001`.
- `for` over an object visits keys in insertion order.
   `x = [for k, v in {b = 1, a = 2} : k]` evaluates to `["b", "a"]`,
   and the template form `"%{ for k, v in {b = 1, a = 2} }${k}%{ endfor }"` evaluates to `"ba"`,
   where the specification and the reference implementation give `["a", "b"]` and `"ab"`.
  The result is deterministic for a given input,
   but it differs from the reference and from what a reader of the specification expects,
   and it changes when an unrelated edit reorders the object literal.

Two more differences were measured and are not part of this cluster,
because they are design gaps rather than defects of these code paths:
a conditional whose branches have different types returns the taken branch
(`true ? {a = 1} : "s"` yields an object) where the reference reports
"Inconsistent conditional result types";
and `false && undefined_var` is an error where the reference returns `false`,
because the reference drops the diagnostics of the side that cannot change the result.

## Root cause

### Remainder and division by zero

The evaluator applies Rust's operators to `Number` directly:

```rust
// hcl-rs-0.19.8/src/eval/impls.rs:421-425
            (Number(lhs), Plus, Number(rhs)) => Number(lhs + rhs),
            (Number(lhs), Minus, Number(rhs)) => Number(lhs - rhs),
            (Number(lhs), Mul, Number(rhs)) => Number(lhs * rhs),
            (Number(lhs), Div, Number(rhs)) => Number(lhs / rhs),
            (Number(lhs), Mod, Number(rhs)) => Number(lhs % rhs),
```

`Number`'s remainder is an integer remainder when both sides are integers:

```rust
// hcl-primitives-0.1.12/src/number.rs:230-239 (abbreviated)
impl Rem for N {
    fn rem(self, rhs: Self) -> Self::Output {
        match coerce(self, rhs) {
            CoerceResult::PosInt(a, b) => N::PosInt(a % b),
```

`a % 0` on `u64` panics in every profile:
integer division and remainder by zero are checked unconditionally in Rust,
unlike overflow,
which is checked only with debug assertions.

Division goes through floating point and produces an infinity,
which the type forbids:

```rust
// hcl-primitives-0.1.12/src/number.rs:42-59 (abbreviated)
    fn from_finite_f64(value: f64) -> N {
        debug_assert!(value.is_finite());
```

With debug assertions the assertion fires;
without them the infinity is stored in a type documented as "Always finite"
(`hcl-primitives-0.1.12/src/number.rs:37`)
and prints as the largest finite double.

### Integer overflow and saturation

Addition and multiplication use unchecked operators,
so release builds wrap and debug builds panic:

```rust
// hcl-primitives-0.1.12/src/number.rs:178-218 (abbreviated)
            CoerceResult::PosInt(a, b) => N::PosInt(a + b),
            ...
            CoerceResult::PosInt(a, b) => N::PosInt(a * b),
```

When a float result happens to be integral,
it is cast back to `i64`:

```rust
// hcl-primitives-0.1.12/src/number.rs:53-58
        if no_fraction {
            #[allow(clippy::cast_possible_truncation)]
            N::from(value as i64)
        } else {
            N::Float(value)
        }
```

`as` saturates,
so `2.0000000000000002e19 as i64` becomes `i64::MAX`,
and the `N::from` conversion then stores it as a positive integer,
which is where `18446744073709551614` comes from.

The specification requires more:
numbers are "arbitrary-precision floating point",
integers are "represented with at least 256 bits",
and an implementation must produce an error when a literal or result cannot be represented
(`hashicorp/hcl` `spec.md:232-247`).
The reference implementation stores `big.Float` values,
so the same expressions widen instead of wrapping.

### Object iteration order

`Value::Object` is an insertion-ordered map:

```rust
// hcl-rs-0.19.8/src/value/mod.rs:15
pub type Map<K, V> = indexmap::IndexMap<K, V>;
```

and the `for` machinery iterates it directly:

```rust
// hcl-rs-0.19.8/src/eval/expr.rs:121-134 (abbreviated)
fn evaluate_collection(expr: &Expression, ctx: &Context) -> EvalResult<Vec<(Value, Value)>> {
    match expr.evaluate(ctx)? {
        ...
        Value::Object(object) => Ok(object
            .into_iter()
            .map(|(key, value)| (Value::from(key), value))
            .collect()),
```

The native-syntax specification is explicit:

> For object and map types,
> the _key_ is the string attribute name or element key,
> and the _value_
> is the attribute or element value.
> The elements are visited in the order defined by a
> lexicographic sort of the attribute names or keys.

(`hashicorp/hcl` `hclsyntax/spec.md:465-468`.)

The same function feeds the template `for` directive
(`hcl-rs-0.19.8/src/eval/expr.rs:156-167`),
so both forms are affected.

## Verification

- Versions under test:
   `hcl-rs` 0.19.8 (archive SHA-256
   `212c6fce17a8b9e0eab43ccf61c8bc5d2c51fc24ad52d19bd1d2be2eeba22d02`)
   and `hcl-primitives` 0.1.12 (archive SHA-256
   `bd662a8afeca01b5b5318f35baed70017b9f854bfa38bdcdadb87de946a49071`),
   plus the upstream clone at `f7b17594f8d738343508f87dc9a24018e7fbeb8d`.
- Reference implementation used as the oracle:
   `hashicorp/hcl` at `4932c1452af6` with `zclconf/go-cty` at `a918e1174fcf`.
- Toolchain:
   `nightly-2026-09-12`,
  Go 1.27.1.
- Isolation:
   `podman run --rm --init --memory=2g --cpus=2 --pids-limit=512 --network=none`,
   no credentials,
  no home directory,
  no repository mount.

Harness,
one Rust binary and one Go binary reading the same case file:

```rust
// ~/temp/agent/hcl-evaluator-2026-09-17/lab/probe-hcl/src/bin/semantics.rs (abbreviated)
let outcome = panic::catch_unwind(|| {
    let parsed = hcl::parse(&body)?;
    let mut ctx = Context::new();
    ctx.declare_func("upper", FuncDef::builder().param(ParamType::String).build(upper));
    parsed.evaluate(&ctx)
});
```

```go
// ~/temp/agent/hcl-evaluator-2026-09-17/lab/go-oracle/main.go (abbreviated)
file, diags := hclsyntax.ParseConfig(src, "case.hcl", hcl.InitialPos)
value, diags := attr.Expr.Value(ctx)
encoded, err := ctyjson.Marshal(value, value.Type())
```

Results,
with the crate built in a release profile (`panic = "unwind"`,
overflow checks off)
and again with overflow checks and debug assertions on:

- `x = 5 % 0`:
   release panic `attempt to calculate the remainder with a divisor of zero`;
   checked build the same panic;
   reference `5`.
- `x = 1 / 0`:
   release `1.797693134862316e308`;
   checked panic `assertion failed: value.is_finite()`;
   reference positive infinity.
- `x = 18446744073709551615 + 1`:
   release `0`;
   checked panic `attempt to add with overflow`;
   reference `18446744073709551616`.
- `x = 4611686018427387904 * 4`:
   release `0`;
   checked panic `attempt to multiply with overflow`;
   reference `18446744073709551616`.
- `x = 10000000000000000000.5 * 2`:
   release and checked `18446744073709551614`;
   reference `20000000000000000001`.
- `x = [for k, v in {b = 1, a = 2} : k]`:
   release and checked `["b", "a"]`;
   reference `["a", "b"]`.
- `x = "%{ for k, v in {b = 1, a = 2} }${k}%{ endfor }"`:
   release and checked `"ba"`;
   reference `"ab"`.
- `x = 0.1 + 0.2`:
   release and checked `0.30000000000000004`;
   reference `0.3`.
- `x = 123456789012345678901234567890`:
   release and checked parse error `unexpected token` pointing at column 1;
   reference `123456789012345678901234567890`.

The last row is a separate literal-precision limit of the same `Number` type:
the parser reports `unexpected token` and points at column 1 rather than at the literal.
An adjacent,
already-filed case is
`martinohmann/hcl-rs` issue #566,
"Negative integer literals below `i64::MIN` parse to positive numbers".

Cases that behave like the reference:
boolean operators,
comparisons,
string and collection operations,
`for` over tuples and lists,
conditional expressions with matching branch types,
splat operators,
and interpolation unwrapping.

## Verified workarounds

- Do not evaluate user-authored HCL with `hcl::eval` in a process that must survive,
   or run evaluation in a child process and treat its abort as a diagnostic that names the file.
  Tradeoff:
   a process boundary per evaluation,
  and the diagnostic can only name the file,
  not the expression.
- Build with `panic = "unwind"` and wrap evaluation in `catch_unwind`.
  Tradeoff:
   catches the remainder and overflow panics but not `panic = "abort"` builds,
   and a caught panic still carries no source location.
- Pre-scan the expression tree and reject the shapes that misbehave:
   any `%` or `/` whose right operand is a literal zero,
   and any integer literal outside the `i64` range.
  Tradeoff:
   catches only literal operands,
  not values computed at evaluation time.
- Sort object keys before iterating:
   feed `for` expressions from a sorted list built by the host application rather than from an object,
   or sort in a host function.
  Tradeoff:
   the configuration author has to remember which collections are sorted.
- Vendor or fork the crates with the patch under "Upstream filing artifact".
  Tradeoff:
   a fork to maintain,
  plus a crates.io release of the fork before dependent crates can be published.

## What does not work

- Choosing a release profile to dodge the panics:
   the remainder panic happens in release builds too,
  because Rust always checks division by zero.
- Catching the arithmetic problems through `ErrorKind`:
   the failing paths never construct an error;
   they panic or return a wrong value.
- Using `evaluate_in_place` instead of `evaluate`:
   it calls the same operator implementations
   (`hcl-rs-0.19.8/src/eval/impls.rs:432-437`).
- Wrapping objects in `tomap`-style conversions before `for`:
   the crate has no built-in functions at all (open issue #484),
   and any host function still returns a `Value::Object` that iterates in insertion order.

## Upstream filing artifact

### Upstream filing decision

1.  Is it really upstream's fault?
     Yes for the panics,
    the wraparound,
    and the saturation:
     a library that evaluates user input should not abort the process,
     and the maintainer states the crates follow the HCL specifications
     (issue #304,
    comment of 2024-04-09).
    The iteration order is also upstream's:
     the native-syntax specification names lexicographic order.
2.  Can upstream fix it?
     Yes.
    Checked arithmetic with a float fallback,
    a guard for a zero right-hand side,
     a range check before the `as i64` cast,
     and a sort in `evaluate_collection` are all local changes.
    A full fix for arbitrary precision would be larger,
     but is not needed to remove the panics and the wraparound.
3.  Are they supporting this use case?
     Yes.
    `crates/hcl-rs/tests/eval.rs` exercises binary operators,
    `for` expressions,
    and templates,
     and the crate's documentation presents `hcl::eval` as the way to evaluate HCL expressions.
4.  Would the repo welcome our contribution?
     Yes.
    `CONTRIBUTING.md` invites pull requests,
    asks for an issue first for breaking changes,
     asks for regression tests,
    and says nothing about AI assistance.
    Note that changing the results of arithmetic and of `for` over objects is behavior-visible,
     so the issue should come first,
    as `CONTRIBUTING.md` requires.
5.  Will they likely fix it?
     Likely for the panics;
     less certain for iteration order,
    which changes output for existing users,
     though it aligns with the specification the maintainer cites.
    No won't-fix statement exists.
    Issue #566,
    about the neighbouring literal-range problem,
    was opened 2026-09-04
     and has no maintainer reply yet,
    which is silence rather than a decline.
6.  Have we prototyped a minimal fix compatible with their architecture?
     Yes,
    see the diff and its verification.

### Duplicate search

Searched `martinohmann/hcl-rs` issues and pull requests,
open and closed,
on 2026-09-17
(recorded in `data/dup-search.jsonl` in the research scratchpad):
`overflow`,
`modulo`,
`remainder`,
`division by zero`,
`divide by zero`,
`panic eval`,
`arithmetic`,
`precision`,
`big number`,
`order`,
`sorted`,
`lexicographic`,
`for expression object`,
`iteration`.
No hit describes any of these three.
The nearest neighbours are #566 (negative literals below `i64::MIN` parse positive,
open,
no replies)
and #88 (a formatter subtraction overflow,
merged in 2022).
Since #566 is about the same `Number` representation but a different code path (literal parsing,
not arithmetic),
this stays a new report rather than a comment on #566.

### Prototype fix and verification

Applied in a disposable clone of `martinohmann/hcl-rs` at `f7b17594f8d738343508f87dc9a24018e7fbeb8d`
with `git remote set-url --push origin DISABLED`.
Three source changes:

```diff
--- a/crates/hcl-primitives/src/number.rs
+++ b/crates/hcl-primitives/src/number.rs
@@ impl N { fn from_finite_f64
-        if no_fraction {
+        // Only integral values inside the `i64` range convert to an integer; `as` saturates
+        // outside of it and would silently change the value.
+        #[allow(clippy::cast_precision_loss)]
+        let in_i64_range = value >= i64::MIN as f64 && value < i64::MAX as f64;
+
+        if no_fraction && in_i64_range {
@@ impl Add for N
-            CoerceResult::PosInt(a, b) => N::PosInt(a + b),
-            CoerceResult::NegInt(a, b) => N::NegInt(a + b),
+            CoerceResult::PosInt(a, b) => a
+                .checked_add(b)
+                .map_or_else(|| N::from_finite_f64(a as f64 + b as f64), N::PosInt),
+            CoerceResult::NegInt(a, b) => a
+                .checked_add(b)
+                .map_or_else(|| N::from_finite_f64(a as f64 + b as f64), N::from),
```

with the same treatment for `Sub` and `Mul`,
and:

```diff
--- a/crates/hcl-rs/src/eval/impls.rs
+++ b/crates/hcl-rs/src/eval/impls.rs
@@ impl Evaluate for BinaryOp
+            // Division and remainder by zero have no finite result; `Number` cannot hold infinity,
+            // and integer remainder by zero panics.
+            (Number(_), operator @ (Div | Mod), Number(rhs)) if rhs.as_f64() == Some(0.0) => {
+                return Err(ctx.error(ErrorKind::Message(format!(
+                    "binary operator `{operator}` with a zero right-hand side has no finite result"
+                ))))
+            }
--- a/crates/hcl-rs/src/eval/expr.rs
+++ b/crates/hcl-rs/src/eval/expr.rs
@@ fn evaluate_collection
-        Value::Object(object) => Ok(object
-            .into_iter()
-            .map(|(key, value)| (Value::from(key), value))
-            .collect()),
+        Value::Object(object) => {
+            // The HCL native syntax specification visits object and map elements "in the order
+            // defined by a lexicographic sort of the attribute names or keys".
+            let mut entries: Vec<(String, Value)> = object.into_iter().collect();
+            entries.sort_by(|(a, _), (b, _)| a.cmp(b));
+            Ok(entries
+                .into_iter()
+                .map(|(key, value)| (Value::from(key), value))
+                .collect())
+        }
```

and a new regression test file `crates/hcl-rs/tests/eval_spec_conformance.rs` with five tests:
lexicographic order for `for` expressions and for the template `for` directive,
errors for `% 0` and `/ 0`,
no wraparound on integer overflow,
and no saturation for large integral float results.

Verification,
both runs in the bounded container:

- Without the source changes and with the new test file:
   `cargo test --offline -p hcl-rs --test eval_spec_conformance` printed
   `test result: FAILED. 0 passed; 5 failed`,
   with two failures raised as panics inside `hcl-primitives`
   (`number.rs:183` and `number.rs:234`).
- With the source changes:
   `cargo test --offline --workspace` printed `ok` for all 24 test binaries with no failures,
   the crate's specsuite runner included.

The patch does not give the crate arbitrary-precision numbers.
It removes the panics and the silent wraparound by widening to floating point,
which is still lossy above 2^53 and therefore a smaller change than the specification asks for.

### Draft issue

~~~md
Title: [bug]: `% 0` panics, integer overflow wraps, and `for` over an object is not sorted

Labels: bug

### Description of the bug

Three evaluation results differ from the HashiCorp implementation, and one of them aborts the
process.

```rust
use hcl::eval::{Context, Evaluate};

fn eval(src: &str) -> String {
    let body = hcl::parse(src).unwrap();
    let attr = body.attributes().next().unwrap();
    format!("{:?}", attr.expr().evaluate(&Context::new()))
}

fn main() {
    println!("{}", eval("x = [for k, v in {b = 1, a = 2} : k]")); // ["b", "a"], expected ["a", "b"]
    println!("{}", eval("x = 18446744073709551615 + 1"));         // 0, expected 18446744073709551616
    println!("{}", eval("x = 4611686018427387904 * 4"));          // 0
    println!("{}", eval("x = 10000000000000000000.5 * 2"));       // 18446744073709551614
    println!("{}", eval("x = 1 / 0"));                            // 1.797693134862316e308
    println!("{}", eval("x = 5 % 0"));                            // panics
}
```

`x = 5 % 0` panics with `attempt to calculate the remainder with a divisor of zero` in release
builds too, because Rust checks integer division by zero unconditionally. With debug assertions,
`1 / 0` panics on `assertion failed: value.is_finite()` and the overflow cases panic with
`attempt to add with overflow` and `attempt to multiply with overflow`.

The reference implementation returns `5` for `5 % 0`, positive infinity for `1 / 0`,
`18446744073709551616` for both overflow cases, `20000000000000000001` for the float case, and
`["a", "b"]` for the `for` expression.

### Root cause

- `impl Evaluate for BinaryOp` (crates/hcl-rs/src/eval/impls.rs) applies Rust's `+`, `*`, `/`
  and `%` to `Number` directly. `Rem` for two integers is `a % b`, which panics for `b == 0`;
  `Add` and `Mul` use unchecked operators, so they wrap in release and panic in debug.
- `N::from_finite_f64` (crates/hcl-primitives/src/number.rs) casts an integral float back with
  `as i64`, which saturates for values outside the `i64` range.
- `evaluate_collection` (crates/hcl-rs/src/eval/expr.rs) iterates `Value::Object`, an
  `IndexMap`, in insertion order. `hclsyntax/spec.md` says object and map elements are visited
  "in the order defined by a lexicographic sort of the attribute names or keys", and the same
  function feeds the template `for` directive.

### Suggested fix

Checked arithmetic with a floating-point fallback in `Add`, `Sub` and `Mul`; a range check before
the `as i64` cast in `from_finite_f64`; an error for a zero right-hand side of `/` and `%` in
`BinaryOp::evaluate`; and a sort by key in `evaluate_collection`. A patch in that shape passes
`cargo test --workspace`, including the specsuite runner, and five new regression tests fail
without it.

Changing arithmetic results and iteration order is user-visible, so this is filed as an issue
first rather than as a pull request, per CONTRIBUTING.md. Happy to send the patch if the direction
is acceptable; a full arbitrary-precision number type would be a larger change and is not what
this patch attempts.

This report was prepared with AI assistance; the reproduction, the source trace, the patch, and
the test runs above were executed and checked before filing.
~~~
