# Oxlint 1.71.0 `unicorn/no-immediate-mutation` flags Set and Map clone-plus-mutate patterns that need a spread temp to inline

> Scratch-path note:
> `/tmp/agent` paths in this document are historical.
> Use `~/temp/agent` for current work.

## Symptom

Oxlint reports `unicorn(no-immediate-mutation)` for this shape:

```ts
const nextSeen = new Set(seen,);
nextSeen.add(variable,);
```

The emitted diagnostic is:

```text
Do not call `.add()` immediately after initializing a Set.
help: Add the element to the Set initializer array.
```

The same pattern appears for `Map`:

```ts
const nextEntries = new Map(entries,);
nextEntries.set(key, value,);
```

That diagnostic suggests this rewrite:

```ts
const nextSeen = new Set([
  ...seen,
  variable,
],);
```

That rewrite is semantically valid for normal iterables,
but it forces an intermediate array allocation before `Set` consumes the iterable.
For clone-plus-mutate code that intentionally keeps `Set` membership behavior and avoids a temporary spread array,
the lint is noise.

There are two diagnostic variants:

- Set variant:
   `Do not call `.
  add()` immediately after initializing a Set.`
- Map variant:
   `Do not call `.
  set()` immediately after initializing a Map.`

## Root cause

Source clone:
`/tmp/agent/oxc-no-immediate-mutation-20260629-1`,
origin `https://github.com/oxc-project/oxc.git`,
commit `d8c6b550c8802cc68f8e404f279cdc603692b3b6`.

The rule documents the intended transformation as moving immediate mutation into the initializer.
`crates/oxc_linter/src/rules/unicorn/no_immediate_mutation.rs:53-90`:

```rust
// crates/oxc_linter/src/rules/unicorn/no_immediate_mutation.rs:53-90
declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows mutating a variable immediately after initialization.
    ///
    /// ### Why is this bad?
    ///
    /// When you initialize a variable and immediately mutate it,
    /// it's cleaner to include
    /// the mutation in the initialization.
    /// This makes the code more readable and reduces
    /// the number of statements.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// const array = [1, 2];
    /// array.push(3);
    ///
    /// const object = {foo: 1};
    /// object.bar = 2;
    ///
    /// const set = new Set([1, 2]);
    /// set.add(3);
    ///
    /// const map = new Map([["foo", 1]]);
    /// map.set("bar", 2);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// const array = [1, 2, 3];
    ///
    /// const object = {foo: 1, bar: 2};
    ///
    /// const set = new Set([1, 2, 3]);
    ///
    /// const map = new Map([["foo", 1], ["bar", 2]]);
    /// ```
```

The runtime traversal looks only at expression statements and the immediately previous statement.
`crates/oxc_linter/src/rules/unicorn/no_immediate_mutation.rs:108-141`:

```rust
// crates/oxc_linter/src/rules/unicorn/no_immediate_mutation.rs:108-141
impl Rule for NoImmediateMutation {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        // We look for ExpressionStatements that might be mutations
        let AstKind::ExpressionStatement(expr_stmt) = node.kind() else {
            return;
        };

        // Get parent to find sibling statements
        let parent = ctx.nodes().parent_node(node.id());

        let statements: &[Statement<'a>] = match parent.kind() {
            AstKind::BlockStatement(block) => &block.body,
            AstKind::Program(program) => &program.body,
            AstKind::FunctionBody(body) => &body.statements,
            AstKind::StaticBlock(block) => &block.body,
            AstKind::SwitchCase(case) => &case.consequent,
            _ => return,
        };

        // Find the index of current statement
        let Some(current_idx) = statements.iter().position(|stmt| stmt.span() == expr_stmt.span)
        else {
            return;
        };

        // Need at least one statement before this one
        if current_idx == 0 {
            return;
        }

        let prev_stmt = &statements[current_idx - 1];

        // Check what kind of mutation we're looking at and match with previous statement
        check_mutation(&expr_stmt.expression, prev_stmt, ctx);
    }
}
```

The Set and Map initializer classifier records only the constructor identity.
It does not distinguish `new Set([a])` from `new Set(existingIterable)`.
`crates/oxc_linter/src/rules/unicorn/no_immediate_mutation.rs:350-379`:

```rust
// crates/oxc_linter/src/rules/unicorn/no_immediate_mutation.rs:350-379
/// Get the init type from an expression
fn get_expression_init_type<'a>(expr: &Expression<'a>, ctx: &LintContext<'a>) -> Option<InitType> {
    match expr {
        Expression::ArrayExpression(_) => Some(InitType::Array),
        Expression::ObjectExpression(_) => Some(InitType::Object),
        Expression::NewExpression(new_expr) => get_new_expression_type(new_expr, ctx),
        _ => None,
    }
}

/// Get the init type from a new expression (new Set(),
/// new Map())
fn get_new_expression_type<'a>(
    new_expr: &NewExpression<'a>,
    ctx: &LintContext<'a>,
) -> Option<InitType> {
    let callee = new_expr.callee.get_inner_expression();
    let Expression::Identifier(id) = callee else {
        return None;
    };

    // Only match global Set/Map constructors
    if !id.is_global_reference(ctx.scoping()) {
        return None;
    }

    match id.name.as_str() {
        "Set" | "WeakSet" => Some(InitType::Set),
        "Map" | "WeakMap" => Some(InitType::Map),
        _ => None,
    }
}
```

The mutation reporter then reports every immediate `.add()` on `InitType::Set` and every immediate `.set()` on
`InitType::Map`,
 after checking only mutation-call argument shape.
It has no access to whether the constructor argument was an array literal or an arbitrary iterable.
`crates/oxc_linter/src/rules/unicorn/no_immediate_mutation.rs:221-244`:

```rust
// crates/oxc_linter/src/rules/unicorn/no_immediate_mutation.rs:221-244
            if call.arguments.is_empty() {
                return;
            }
            ctx.diagnostic(array_mutation_diagnostic(call.span, method_name));
        }
        (InitType::Set, "add") => {
            // add() must have exactly one argument (not spread, not empty)
            if call.arguments.len() != 1 {
                return;
            }
            if call.arguments.first().is_some_and(Argument::is_spread) {
                return;
            }
            ctx.diagnostic(set_add_diagnostic(call.span));
        }
        (InitType::Map, "set") => {
            // set() must have exactly two arguments (not spread)
            if call.arguments.len() != 2 {
                return;
            }
            if call.arguments.iter().any(Argument::is_spread) {
                return;
            }
            ctx.diagnostic(map_set_diagnostic(call.span));
```

The diagnostic help text is hard-coded to say Set and Map additions can move into the initializer array.
`crates/oxc_linter/src/rules/unicorn/no_immediate_mutation.rs:38-47`:

```rust
// crates/oxc_linter/src/rules/unicorn/no_immediate_mutation.rs:38-47
fn set_add_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Do not call `.add()` immediately after initializing a Set.")
        .with_help("Add the element to the Set initializer array.")
        .with_label(span)
}

fn map_set_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Do not call `.set()` immediately after initializing a Map.")
        .with_help("Add the entry to the Map initializer array.")
```

So the behavior is not a parser quirk.
It is a rule-shape limitation:
Set and Map initializers are classified by constructor name alone,
then mutation reporting assumes an initializer-array rewrite is always the preferred form.

## Verification

Version under test:
`node_modules/oxlint/package.json` reports `1.71.0`.
The source clone commit was `d8c6b550c8802cc68f8e404f279cdc603692b3b6`.

Runnable harness:

```bash
scratch_dir=$(mktemp --directory /tmp/agent/oxlint-no-immediate-mutation.verify.XXXXXXXX)
cat > "$scratch_dir/cases.ts" <<'TS'
const setFromArray = new Set([1, 2]);
setFromArray.add(3);

const setFromEmpty = new Set();
setFromEmpty.add(3);

const setFromIterable = new Set(existingSet);
setFromIterable.add(3);

const mapFromArray = new Map([["a", 1]]);
mapFromArray.set("b", 2);

const mapFromIterable = new Map(existingEntries);
mapFromIterable.set("b", 2);

const alreadyInlineSet = new Set([...existingSet, 3]);
const alreadyInlineMap = new Map([...existingEntries, ["b", 2]]);
TS
cd "$scratch_dir"
/var/home/user/Monochromatic/node_modules/.bin/oxlint \
  --allow all \
  --warn unicorn/no-immediate-mutation \
  --format json \
  cases.ts
```

Failing Set variant:

```text
Do not call `.add()` immediately after initializing a Set.
help: Add the element to the Set initializer array.
line 2: setFromArray.add(3);
line 5: setFromEmpty.add(3);
line 8: setFromIterable.add(3);
```

Failing Map variant:

```text
Do not call `.set()` immediately after initializing a Map.
help: Add the entry to the Map initializer array.
line 11: mapFromArray.set("b", 2);
line 14: mapFromIterable.set("b", 2);
```

Patterns that work cleanly under upstream oxlint:

```ts
const alreadyInlineSet = new Set([...existingSet, 3]);
const alreadyInlineMap = new Map([...existingEntries, ["b", 2]]);
```

Patterns that fail but require a spread temp to satisfy the suggestion:

```ts
const setFromIterable = new Set(existingSet);
setFromIterable.add(3);

const mapFromIterable = new Map(existingEntries);
mapFromIterable.set("b", 2);
```

Patterns that fail and can move into an existing array literal without the same extra spread-temp concern:

```ts
const setFromArray = new Set([1, 2]);
setFromArray.add(3);

const mapFromArray = new Map([["a", 1]]);
mapFromArray.set("b", 2);
```

## Verified workarounds

### Replace the upstream rule with a project rule

Repo commit `079cef1aa` disables `unicorn/no-immediate-mutation` and adds
`no-restricted-syntax/no-immediate-mutation`.
The replacement still reports array,
 object,
 `Object.assign`,
 Set from an array literal,
and Map from an array literal.
It allows Set and Map clone-plus-mutate patterns when folding would require a temporary spread array.

Verification commands run after the replacement:

```bash
mise run //package/oxlint-plugin/no-restricted-syntax:lint:types
mise run //package/oxlint-plugin/no-restricted-syntax:lint:oxlint
mise run //package/oxlint-plugin/no-restricted-syntax:test:unit
```

Observed unit-test output included:

```text
PASS no-immediate-mutation forms reports each initializer kind except Set and Map clone exceptions
PASS valid fixtures no-immediate-mutation accepts Set and Map clone-plus-mutate patterns
```

Tradeoff:
the repo now owns an oxlint JS-plugin rule mirroring a subset of upstream behavior.
That is intentional because the wanted policy differs from oxlint 1.71.0.

### Keep upstream and use a scoped disable

Keep the upstream rule enabled and write:

```ts
const nextSeen = new Set(seen,);
// oxlint-disable-next-line unicorn/no-immediate-mutation -- Mutates only a fresh Set to avoid building a temporary spread array.
nextSeen.add(variable,);
```

Tradeoff:
every clone-plus-mutate site needs a local explanation,
and `reportUnusedDisableDirectives` starts reporting the comment once the built-in rule is disabled or fixed.
This happened after the project replacement disabled upstream.

### Rewrite through a spread array

Use the upstream-suggested initializer shape:

```ts
const nextSeen = new Set([
  ...seen,
  variable,
],);
```

Tradeoff:
this is lint-clean under upstream oxlint,
but it allocates an intermediate array and iterates the source iterable into that array before `Set` consumes it.

## What does not work

- `oxlint --fix` with oxlint 1.71.0:
  a scratch run with `--fix --allow all --warn unicorn/no-immediate-mutation` reported the diagnostic and left
  `const nextSeen = new Set(seen); nextSeen.add(variable);` unchanged.
  The rule docs page says an auto-fix is planned but not implemented.
- Expression-dodging with `return new Set(seen).add(variable)`:
  it avoids the immediate-statement pattern but keeps the same mutation in a less readable expression.
- Configuring the upstream rule more narrowly:
  `unicorn/no-immediate-mutation` has no option for Set or Map constructor argument shape.
- Keeping a disable comment after replacing the rule:
  oxlint reports it as an unused directive because the built-in diagnostic no longer fires.

## Rust prototype

Prototype clone:
`/tmp/agent/oxc-no-immediate-mutation-prototype.ufNRqZu4`,
origin `https://github.com/oxc-project/oxc.git`,
commit `d8c6b550c8802cc68f8e404f279cdc603692b3b6`.

Prototype patch:
[oxlint-no-immediate-mutation-set-clone.patch](oxlint-no-immediate-mutation-set-clone.patch).

The prototype makes Set and Map initializer kinds carry whether folding the later mutation would require a spread temp.
`crates/oxc_linter/src/rules/unicorn/no_immediate_mutation.rs:101-105` in the prototype:

```rust
// crates/oxc_linter/src/rules/unicorn/no_immediate_mutation.rs:101-105
enum InitType {
    Array,
    Object,
    Set { needs_spread_temp: bool },
    Map { needs_spread_temp: bool },
}
```

The reporter skips only Set and Map calls whose initializer needs the spread temp.
`crates/oxc_linter/src/rules/unicorn/no_immediate_mutation.rs:226-247` in the prototype:

```rust
// crates/oxc_linter/src/rules/unicorn/no_immediate_mutation.rs:226-247
        (InitType::Set { needs_spread_temp: false }, "add") => {
            // add() must have exactly one argument (not spread, not empty)
            if call.arguments.len() != 1 {
                return;
            }
            if call.arguments.first().is_some_and(Argument::is_spread) {
                return;
            }
            ctx.diagnostic(set_add_diagnostic(call.span));
        }
        (InitType::Set { needs_spread_temp: true }, "add") => {}
        (InitType::Map { needs_spread_temp: false }, "set") => {
            // set() must have exactly two arguments (not spread)
            if call.arguments.len() != 2 {
                return;
            }
            if call.arguments.iter().any(Argument::is_spread) {
                return;
            }
            ctx.diagnostic(map_set_diagnostic(call.span));
        }
        (InitType::Map { needs_spread_temp: true }, "set") => {}
```

The helper considers a single non-array-literal constructor argument to require a spread temp.
`crates/oxc_linter/src/rules/unicorn/no_immediate_mutation.rs:377-398` in the prototype:

```rust
// crates/oxc_linter/src/rules/unicorn/no_immediate_mutation.rs:377-398
    let needs_spread_temp = collection_init_needs_spread_temp(new_expr);

    match id.name.as_str() {
        "Set" | "WeakSet" => Some(InitType::Set { needs_spread_temp }),
        "Map" | "WeakMap" => Some(InitType::Map { needs_spread_temp }),
        _ => None,
    }
}

/// Check whether folding a later Set.add()/Map.set() into this initializer would require
/// spreading an existing iterable into a temporary array.
fn collection_init_needs_spread_temp(new_expr: &NewExpression<'_>) -> bool {
    if new_expr.arguments.len() != 1 {
        return false;
    }

    let Some(arg) = new_expr.arguments.first().and_then(Argument::as_expression) else {
        return false;
    };

    !matches!(arg.get_inner_expression(), Expression::ArrayExpression(_))
}
```

The prototype adds pass cases for existing iterables.
`crates/oxc_linter/src/rules/unicorn/no_immediate_mutation.rs:801-807` and
`crates/oxc_linter/src/rules/unicorn/no_immediate_mutation.rs:858-863` in the prototype:

```rust
// crates/oxc_linter/src/rules/unicorn/no_immediate_mutation.rs:801-807
        "const set = new Set([1, 2]);
            set.add(set.size);",
        "const set = new Set(existingSet);
            set.add(3);",
        "const set = new Set(getValues());
            set.add(3);",
```

```rust
// crates/oxc_linter/src/rules/unicorn/no_immediate_mutation.rs:858-863
        r#"const map = new Map([["foo", 1]]);
            map.set(map.size, 2);"#,
        r#"const map = new Map(existingMap);
            map.set("bar", 2);"#,
        r#"const map = new Map(getEntries());
            map.set("bar", 2);"#,
```

The first prototype verification attempt used the clone's default target directory under `/tmp/agent` and failed with:

```text
rustc-LLVM ERROR: IO failure on output stream: Disk quota exceeded
error: could not compile `oxc_linter` (lib test)
```

The target directory was removed from the disposable clone,
and the successful verification used a target directory outside `/tmp`:

```bash
# /tmp/agent/oxc-no-immediate-mutation-prototype.ufNRqZu4
env CARGO_TARGET_DIR=/var/home/user/temp/oxc-no-immediate-mutation-target \
  cargo test --package oxc_linter no_immediate_mutation::test
```

Output:

```text
running 1 test
test rules::unicorn::no_immediate_mutation::test ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1162 filtered out; finished in 0.09s
```

## Upstream filing decision

`.out-of-scope/` was checked with:

```bash
find .out-of-scope -maxdepth 1 -type f -print
```

No oxlint or Oxc exemption was present.

Duplicate search was run with these commands:

```bash
gh search issues --repo oxc-project/oxc "no-immediate-mutation Set.add Map.set spread" --state open --limit 10
gh search issues --repo oxc-project/oxc "no-immediate-mutation Set.add Map.set spread" --state closed --limit 10
gh search prs --repo oxc-project/oxc "no-immediate-mutation Set.add Map.set spread" --state open --limit 10
gh search prs --repo oxc-project/oxc "no-immediate-mutation Set.add Map.set spread" --state closed --limit 10
```

Those searches returned no matching issue or pull request.
Broader searches for `unicorn/no-immediate-mutation` found the implementation PR `#17118`,
the release PR `#17260`,
 the rule-tracking PR `#15989`,
 and broad umbrella issue `#684`,
but no issue about Set or Map clone-plus-mutate exceptions.

### Constraint check

**Is it really upstream's fault?
**
Yes.
The diagnostic is emitted by oxlint's built-in rule,
and the source classifies Set and Map constructors without considering constructor argument shape.

**Can upstream fix it?
**
Yes.
The Rust prototype records whether folding would require a spread temp and keeps existing array-literal reports.

**Are they supporting this use case?
**
Soft yes.
The rule explicitly supports Set and Map mutation shapes,
and the requested exception stays within that same rule surface.
The docs do not state that avoiding an intermediate iterable allocation is a non-goal.

**Would the repo welcome our contribution?
**
Yes with care.
`CONTRIBUTING.md` says,
 "We welcome and appreciate any form of contributions.
"
It also requires AI usage disclosure,
contributor responsibility for AI-generated issues or PRs,
and reviewed,
 tested content.
The linter bug template exists at `.github/ISSUE_TEMPLATE/linter_bug_report.yaml` and asks for version,
command,
 config,
 and reproduction.

**Will they likely fix it?
**
Soft yes.
No duplicate or maintainer rejection was found.
The rule is already implemented upstream and listed in the linter area.
The docs say an auto-fix is planned but not implemented,
which shows continued rule maintenance.

**Have we prototyped a minimal fix compatible with their architecture?
**
Yes.
The prototype patch changes one Rust rule file,
adds pass cases to the existing test vector,
and passes the targeted upstream test command.

### Draft upstream issue

~~~md
Title: `unicorn/no-immediate-mutation` should allow Set/Map clone-plus-mutate when inlining requires a spread temp

Labels: A-linter

AI assistance disclosure: this report was drafted with AI assistance. The reproduction command,
source trace, local workaround, and Rust prototype test output below were checked in a local clone.

## Version

oxlint 1.71.0

Source commit used for the trace and prototype:
`d8c6b550c8802cc68f8e404f279cdc603692b3b6`

## Command

```bash
oxlint --allow all --warn unicorn/no-immediate-mutation --format json cases.ts
```

## Config

No config file. The command enables only `unicorn/no-immediate-mutation` after `--allow all`.

## Reproduction

```ts
const setFromIterable = new Set(existingSet);
setFromIterable.add(3);

const mapFromIterable = new Map(existingEntries);
mapFromIterable.set("b", 2);
```

## What happened

Oxlint reports both immediate mutations:

```text
Do not call `.add()` immediately after initializing a Set.
help: Add the element to the Set initializer array.

Do not call `.set()` immediately after initializing a Map.
help: Add the entry to the Map initializer array.
```

The suggested shape requires materializing an intermediate spread array:

```ts
const setFromIterable = new Set([
  ...existingSet,
  3,
]);

const mapFromIterable = new Map([
  ...existingEntries,
  ["b", 2],
]);
```

That is a different cost profile from moving `set.add(3)` into an existing array literal such as
`new Set([1, 2, 3])`.

## Source trace

`get_new_expression_type` classifies Set and Map initializers solely by constructor identity:

```rust
match id.name.as_str() {
    "Set" | "WeakSet" => Some(InitType::Set),
    "Map" | "WeakMap" => Some(InitType::Map),
    _ => None,
}
```

`check_call_mutation` then reports every immediate `.add()` on `InitType::Set` and `.set()` on `InitType::Map`,
after checking only the mutation-call arguments:

```rust
(InitType::Set, "add") => {
    if call.arguments.len() != 1 {
        return;
    }
    if call.arguments.first().is_some_and(Argument::is_spread) {
        return;
    }
    ctx.diagnostic(set_add_diagnostic(call.span));
}
(InitType::Map, "set") => {
    if call.arguments.len() != 2 {
        return;
    }
    if call.arguments.iter().any(Argument::is_spread) {
        return;
    }
    ctx.diagnostic(map_set_diagnostic(call.span));
}
```

## Expected behavior

Please keep reporting cases that can fold into an existing initializer without this spread-temp issue:

```ts
const setFromArray = new Set([1, 2]);
setFromArray.add(3);

const mapFromArray = new Map([["a", 1]]);
mapFromArray.set("b", 2);
```

Please allow cases where folding the mutation requires materializing an arbitrary iterable into a temporary array:

```ts
const setFromIterable = new Set(existingSet);
setFromIterable.add(3);

const mapFromIterable = new Map(existingEntries);
mapFromIterable.set("b", 2);
```

## Prototype

A local prototype changed `InitType::Set` and `InitType::Map` to carry `needs_spread_temp: bool`,
computed from the Set/Map constructor argument.
When the constructor has exactly one non-array-literal expression argument,
`Set.add` and `Map.set` reports are skipped.

The targeted test passed:

```bash
env CARGO_TARGET_DIR=/var/home/user/temp/oxc-no-immediate-mutation-target \
  cargo test --package oxc_linter no_immediate_mutation::test
```

```text
running 1 test
test rules::unicorn::no_immediate_mutation::test ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1162 filtered out; finished in 0.09s
```
~~~
