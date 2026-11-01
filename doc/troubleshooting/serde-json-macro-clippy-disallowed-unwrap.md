# serde_json 1.0.150: the json! macro expands to Result::unwrap, tripping clippy disallowed-methods in crates that ban unwrap

`serde_json::json!` with an expression value (anything that is not a literal/array/object
token tree) expands to `to_value(&expr).unwrap()`.
Under this repo's clippy configuration,
 which disallows `Result::unwrap`,
 the lint fires
INSIDE the macro expansion and attributes the error to the caller's crate,
 failing
`cargo clippy -- --deny warnings` on code that contains no visible `unwrap`.

Found while building `package/file-manager/variant-gtk-sticky`'s observed-state writer.

## Symptom

```txt
warning: use of a disallowed method `std::result::Result::unwrap`
  --> src/state_out.rs:53:20
  = note: this error originates in the macro `$crate::json_internal` which comes from the
    expansion of the macro `json`
```

repeated per interpolated expression;
 `--deny warnings` turns the build red.
The flagged span points at the `json!({ ... })` block,
 not at any `unwrap` in our source.

## Root cause

`serde_json-1.0.150/src/macros.rs:279` (crates.
io sources as vendored in the local cargo
registry cache):

```rust
        $crate::to_value(&$other).unwrap()
```

Every non-literal value interpolated into `json!` goes through this arm.
The `unwrap` is sound for types whose `Serialize` cannot fail,
 but clippy's
`disallowed-methods` operates syntactically on the expanded code and has no infallibility
notion,
 and lints in macro expansions are attributed to the expanding crate.

## Verification

Environment:
 serde_json 1.0.150,
 repo clippy config disallowing `Result::unwrap`.

Fails:

```rust
let snapshot = serde_json::json!({
    "activePath": inputs.active_path,   // expression value -> to_value(...).unwrap()
    "ready": true,                       // literal: fine on its own
});
```

Works cleanly (shipped in `file-manager-gtk-sticky/src/state_out.rs`):

```rust
let mut snapshot = serde_json::Map::new();
snapshot.insert("activePath".into(), serde_json::Value::from(inputs.active_path.clone()));
snapshot.insert("ready".into(), serde_json::Value::from(true));
let snapshot = serde_json::Value::Object(snapshot);
```

`Value::from` covers strings,
 integers,
 and bools with no fallible step,
 so the disallowed
method never appears in the expansion.

## Verified workarounds

- Build the object explicitly with `Map::insert` and `Value::from` (above).
  Tradeoff:
   more lines and no nesting sugar;
   for shallow objects the cost is trivial.
- Not used here but valid where `json!` is strongly preferred:
   a scoped
  `#[allow(clippy::disallowed_methods)]` on the enclosing item with a justification comment.
  Tradeoff:
   suppression plus this repo's documentation requirement for suppressions;
   the
  explicit-map form is cheaper.

- A crate-local wrapper macro that carries the one scoped allowance,
  shipped in `package/ide/app/src/json_value.rs` as `json_value!`.
  The macro expands to a `let` statement annotated with
  `#[allow(clippy::disallowed_methods, reason = "...")]` that holds `serde_json::json!(...)`,
  and yields the bound value.
  The IDE has 145 `json!` fixtures and wire messages in tests and in its inspection binaries that expand an unwrap,
  where rewriting each object as `Map::insert` calls would bury the JSON shape the test asserts.
  Tradeoff:
  one suppression with this reason string instead of one per call site,
  and a hand-written `unwrap` beside a `json_value!` call is still reported.
  Observed with the root `clippy.toml` mounted in the container lint:
  the library tests,
  the integration tests,
  and both binaries pass `cargo clippy --all-targets -- --deny warnings`,
  and a planted `Result::unwrap()` in the same files still fails it.
  The allowance covers a statement and not an item,
  so unlike the `zbus::interface` case no `const _: () = { ... }` wrapper is needed.
  Production code that builds only a few values uses `Value::from` and `Map::insert` instead
  (`package/ide/app/src/language/config/rust_analyzer.rs` and `incoming.rs`).

## What does not work

- Expecting the lint to skip external macro expansions:
   `disallowed-methods` fires on
  expansions expanded into your crate;
   there is no configuration on our side that scopes it to
  hand-written tokens only.

## Upstream filing decision

`.out-of-scope/` was checked:
 no serde_json or clippy exemption exists.

1. Really upstream's fault?
    No party is wrong:
    serde_json's `unwrap` is deliberate and
   documented as infallible for JSON-compatible types;
    clippy's syntactic lint behaves as
   designed;
    the collision is a local-policy interaction.
2. Can upstream fix it?
    serde_json could expand to an unreachable-panic form,
    but that is a
   stylistic preference,
    not a defect.
3. Supported use case?
    Both tools behave as documented.
4. Would the repo welcome the contribution?
    Not evaluated;
    constraint 1 fails.
5. Will they likely fix it?
    No signal either way;
    nothing needing a fix.
6. Prototyped minimal fix?
    Not applicable;
    consumer-side form recorded above.

Decision:
 nothing to file.
