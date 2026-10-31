# schemars 1.2.3 `JsonSchema` derive expands to `Result::unwrap`, which the repository's `disallowed_methods` lint denies

## Symptom

`cargo clippy --all-targets -- -D warnings` in a crate that derives `schemars::JsonSchema` fails with one pair of
errors per derived type,
all pointing at the derive attribute:

```text
error: use of a disallowed method `std::result::Result::unwrap`
  --> src/config_types.rs:26:24
   |
26 | #[derive(Clone, Debug, JsonSchema, PartialEq)]
   |                        ^^^^^^^^^^
   = note: handle errors with ? or explain intentional panic with expect
   = note: requested on the command line with `-D clippy::disallowed-methods`
   = note: this error originates in the macro `schemars::json_schema` which comes from the expansion of the derive macro `JsonSchema`
```

A second error per type names the macro `$crate::json_internal`,
also reached through the derive.
`cargo check` passes;
only clippy reports it,
because the ban lives in the repository's root `clippy.toml`
(`std::result::Result::unwrap`) and each package's `[lints.clippy] disallowed_methods = "deny"`.

## Root cause

The derive does not itself call `unwrap`.
It generates calls to schemars' declarative macro `json_schema!`,
and that
macro's expansion contains `.unwrap()` on a `Result` (schemars 1.2.3,
`src/macros.rs`):

```rust
macro_rules! json_schema {
    (
        {$($json_object:tt)*}
    ) => {
        <$crate::Schema as ::core::convert::TryFrom<_>>::try_from($crate::_private::serde_json::json!({$($json_object)*})).unwrap()
    };
```

The path is `schemars-1.2.3/src/macros.rs:105` in the crates.io archive.
`TryFrom<Value> for Schema` can fail only for a
JSON value that is neither an object nor a boolean,
and the derive passes an object,
so the call cannot panic in this
use.
Clippy's `disallowed_methods` checks the expanded call and does not skip it for coming from an external macro.

Earlier reading that was wrong:
the `unwrap` calls in `schemars_derive-1.2.3/src/*.rs` (`schema_exprs.rs:218`,
`bound.rs:281`) are in the proc macro's own code,
which runs at compile time and is not linted by the consuming crate.
Only the `json_schema!` expansion reaches the consumer.

## Verification

- Versions:
  `schemars 1.2.3`,
  `schemars_derive 1.2.3`,
  `jsonschema 0.60.2` (from the lockfile of
  `package/file-enforcer/core-rust`),
  `rustc 1.101.0-nightly (19833af72 2026-10-20)`.
- Failing:
  `cd package/file-enforcer/core-rust && cargo clippy --all-targets -- -D warnings` with the two
  `#![expect]` attributes in `src/config_types.rs` and `src/config_parts.rs` removed:
  14 errors,
  all at derive sites in
  those two files (7 derives,
  2 errors each).
- Passing:
  the same command with the attributes present prints nothing and exits 0.
  Because the attribute is
  `#![expect]`,
  not `#![allow]`,
  clippy reports an unfulfilled expectation if the lint ever stops firing there,
  so the
  suppression cannot go stale silently.
- Derive without serde:
  `#[derive(JsonSchema)]` compiles and emits a draft 2020-12 schema with no serde derive on the
  type (`src/schema.rs`);
  `#[schemars(...)]` attributes such as `tag`,
  `untagged`,
  `rename_all_fields`,
  `deny_unknown_fields`,
  `description` and `regex(pattern = ...)` all take effect (the committed
  `file-enforcer.schema.json` shows them).

## Verified workarounds

`#![expect(clippy::disallowed_methods, reason = "...")]` at the top of the two modules that hold only the derived type
declarations (`src/config_types.rs`,
`src/config_parts.rs`).
Tradeoff:
the expectation covers the whole module,
so a
real `Result::unwrap` added to either file would go unreported.
Those files hold type declarations and two tiny
accessor methods,
no other calls.
`clippy.toml`'s own note asks for exactly this form:
"a remaining use needs an
`#[expect]` with a reason".

The same cause hits the `serde_json::json!` macro:
its expansion (`$crate::json_internal`) also contains `.unwrap()`
on a `Result`,
and `cargo clippy` reported it at every `json!` call in `package/file-enforcer/core-rust`.
There the
macro was simply avoided:
`src/json_build.rs` builds objects from `Map::insert` calls.

## What does not work

- A configuration-only fix.
  Clippy's `disallowed-methods` entries in `clippy.toml` take a path,
  a reason and
  `allow-invalid`;
  none of them limits a ban to code outside macro expansions.
  The repository forbids loosening lint
  rules without approval,
  so removing the entry was never an option.
- Moving the derive into a separate crate:
  it would satisfy the lint but add a crate for one attribute.
- Implementing `JsonSchema` by hand for every type:
  removes the derive's `description` and `required` generation that
  keeps the schema and the types in step.

## Upstream filing decision

No issue or comment is drafted.
The constraints were not walked in full because the repository-side `#[expect]`
resolves the problem and the cost to a poster of an agent-written report was not weighed;
filing is the repository
owner's decision and carries risk to the poster's account.
Readiness-check items (reproduction on other release lines,
duplicate and work-in-progress search,
trust-system search) were not run.
External citations above point at the crates.io
archive of the pinned version rather than a commit permalink,
and were not loaded in `agent-browser`.
