# Oxlint 1.71.0 `node/no-sync` flags non-Node `parseSync` calls by suffix alone

> Scratch-path note:
> `/tmp/agent` paths in this document are historical.
> Use `~/temp/agent` for current work.

Note 2026-11-09T16:47:44-05:00:
these citations in this record name paths that no longer exist.
`package/git-policy/cli/src/parser/` is not in the tree;
its files were deleted on 2026-10-13 in `642b6d72cb`.
`package/git-policy/cli/src/parser/push.ts` left the tree on 2026-10-13 in `642b6d72cb`.

## Symptom

`mise run //package/git-policy/cli:lint:oxlint` reports `node(no-sync)` warnings for Optique CLI parser calls:

```text
! node(no-sync): Unexpected sync method: 'parseSync'.
  ,- [src/parsers/push.ts:67:23]
66 |        */
67 | ,->   const parseResult = parseSync(
68 | |       pushRegionParser,
69 | |       postSubcommandArgs,
70 | `->   );
```

The same diagnostic appears at these package paths:

- `package/git-policy/cli/src/parsers/stash.ts:121`
- `package/git-policy/cli/src/parsers/add.ts:289`
- `package/git-policy/cli/src/parsers/reset.ts:149`
- `package/git-policy/cli/src/parsers/clean.ts:161`
- `package/git-policy/cli/src/parsers/commit.ts:244`
- `package/git-policy/cli/src/parsers/push.ts:67`
- `package/git-policy/cli/src/parsers/status.ts:102`
- `package/git-policy/cli/src/parsers/status.ts:158`

Every address in that list spells `src/parsers/`,
which no ref of this repository ever carried:
the directory was `package/git-policy/cli/src/parser/` (singular).
Those seven TypeScript files were deleted on 2026-10-13 by `642b6d72cb`,
and the package's source is now the Rust tree under `package/git-policy/cli/src/native/`.

The import is not a Node builtin API.
`package/git-policy/cli/src/parsers/push.ts:3` imports it from Optique:

```ts
// package/git-policy/cli/src/parsers/push.ts:3
import { parseSync, } from '@optique/core/parser';
```

`push.ts` lived at `package/git-policy/cli/src/parser/push.ts` and was deleted with the
TypeScript wrapper on 2026-10-13 (`642b6d72cb`);
the excerpt keeps the recorded shape.

So the warning is a false positive for this repo's intended rule meaning:
ban sync Node APIs,
not every function whose name ends in `Sync`.

## Root cause

Source clone:
`/tmp/agent/oxc-js-plugin-sync-20260629`,
origin `https://github.com/oxc-project/oxc.git`,
commit `da0e5bf6687b4bc5f376898f2d59832c6419ce15`.

Oxlint's `node/no-sync` rule checks every call expression and extracts a sync-looking name from the callee.
`crates/oxc_linter/src/rules/node/no_sync.rs:79-96`:

```rust
// crates/oxc_linter/src/rules/node/no_sync.rs:79-96
fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
    let AstKind::CallExpression(call_expr) = node.kind() else {
        return;
    };

    let Some(property_name) = get_sync_property_name(&call_expr.callee) else {
        return;
    };

    if self.0.ignores.contains(property_name) {
        return;
    }

    if self.0.allow_at_root_level && get_enclosing_function(node, ctx).is_none() {
        return;
    }

    ctx.diagnostic(no_sync_diagnostic(call_expr.span, property_name));
}
```

The extractor is suffix-based.
It does not inspect imports,
resolved modules,
or whether the callee belongs to a Node builtin.
`crates/oxc_linter/src/rules/node/no_sync.rs:100-120`:

```rust
// crates/oxc_linter/src/rules/node/no_sync.rs:100-120
fn get_sync_property_name<'a>(expr: &'a Expression<'a>) -> Option<&'a str> {
    match expr.get_inner_expression() {
        Expression::Identifier(ident) if ident.name.as_str().ends_with("Sync") => {
            Some(ident.name.as_str())
        }
        Expression::StaticMemberExpression(member) => {
            if member.property.name.as_str().ends_with("Sync") {
                Some(member.property.name.as_str())
            } else {
                get_sync_property_name(&member.object)
            }
        }
        Expression::ComputedMemberExpression(member) => {
            if let Some(name) = member.static_property_name()
                && name.as_str().ends_with("Sync")
            {
                return Some(name.as_str());
            }
            get_sync_property_name(&member.object)
        }
        _ => None,
    }
}
```

The only configuration escape hatches are `allowAtRootLevel` and `ignores`.
`crates/oxc_linter/src/rules/node/no_sync.rs:24-28`:

```rust
// crates/oxc_linter/src/rules/node/no_sync.rs:24-28
struct NoSyncConfig {
    /// Whether synchronous methods should be allowed at the top level of a file.
    allow_at_root_level: bool,
    /// Function names to ignore.
    ignores: FxHashSet<CompactStr>,
}
```

That design explains the Optique report:
`parseSync()` is an `Identifier` ending in `Sync`,
so it reports even though it is imported from `@optique/core/parser`.

## Verification

Version under test:
`pnpm-lock.yaml:5340-5341` pins `oxlint@1.71.0`.

Runnable harness:

```bash
mise run //package/git-policy/cli:lint:oxlint
```

Observed output includes eight `node(no-sync)` warnings for Optique `parseSync` calls,
plus unrelated existing `catch-binding` errors in the same package.
One representative warning:

```text
! node(no-sync): Unexpected sync method: 'parseSync'.
  ,- [src/parsers/status.ts:102:23]
101 |        */
102 | ,->   const parseResult = parseSync(
103 | |       statusPreParser,
104 | |       preSubcommandArgs,
105 | `->   );
```

Patterns that fail under upstream `node/no-sync`:

- Direct `Identifier` calls ending in `Sync`,
  such as `parseSync()`,
  regardless of import source.
- Member calls ending in `Sync`,
  such as `fs.readFileSync()`.
- Static computed member calls ending in `Sync`,
  such as `fs['readFileSync']()`.
- Chained calls whose object contains a sync-looking member,
  such as `fs.readFileSync.apply(...)`.

Patterns that work cleanly under upstream `node/no-sync`:

- Non-called references,
  such as `const parser = parseSync`.
- Calls whose final extracted name does not end in `Sync`,
  such as `parse()`.
- Ignored names configured through `ignores`,
  such as `parseSync` when added to the rule config.

## Verified workarounds

### Replace upstream rule with a project-specific rule

Disable `node/no-sync` and enable a custom `no-restricted-syntax/no-sync` rule that only reports sync APIs
reached through Node builtin modules.
This is the chosen repo-local workaround.

Tradeoff:
the project owns Node API detection logic.
That is intentional here because the wanted policy is narrower than upstream's suffix rule.

### Configure `ignores`

Add each non-Node sync-looking API name,
such as `parseSync`,
to `node/no-sync`'s `ignores` list.

Tradeoff:
this suppresses every call with that name,
including a future Node or project API with the same identifier.
The ignore list also grows whenever another library exposes a legitimate sync-named API.

### Rename or wrap the library call

Import Optique's mode-dispatching `parse` helper or write a local wrapper whose name does not end in `Sync`.

Tradeoff:
this changes working code only to appease a lint heuristic.
It also hides the more precise policy we want:
ban Node sync APIs,
not arbitrary sync-named functions.

## What does not work

- Keeping `node/no-sync` enabled without `ignores`:
  the rule continues to report any called suffix match.
- Relying on the `node` plugin namespace to imply Node-only resolution:
  the implementation never checks import sources.
- Treating the diagnostic as proof that `parseSync` is a Node API:
  the emitting tool is oxlint's Node plugin,
  but the callee source here is `@optique/core/parser`.

## Upstream filing decision

`.out-of-scope/` was checked with:

```bash
find .out-of-scope -maxdepth 2 -type f -print
```

No oxlint-specific exemption was present.

Duplicate searches returned no matching open or closed issue or PR:

```bash
gh search issues --repo oxc-project/oxc "node/no-sync parseSync" --state open --limit 20
gh search issues --repo oxc-project/oxc "node/no-sync parseSync" --state closed --limit 20
gh search prs --repo oxc-project/oxc "node/no-sync parseSync" --state open --limit 20
gh search prs --repo oxc-project/oxc "node/no-sync parseSync" --state closed --limit 20
gh search issues --repo oxc-project/oxc "no-sync Sync false positive" --state open --limit 20
gh search issues --repo oxc-project/oxc "no-sync Sync false positive" --state closed --limit 20
gh search prs --repo oxc-project/oxc "no-sync Sync false positive" --state open --limit 20
gh search prs --repo oxc-project/oxc "no-sync Sync false positive" --state closed --limit 20
```

Constraint check:

- Is it really upstream's fault?
  Yes.
  The diagnostic is emitted by upstream oxlint and the suffix-only implementation causes this repo's false positive.
  The current behavior is explainable,
  but the rule name and examples are Node-focused while the implementation reports non-Node callees.
- Can upstream fix it?
  Yes.
  A stricter implementation can resolve bindings to Node builtin modules before reporting.
- Are they supporting this use case?
  Soft yes.
  The docs frame the rule as Node.js sync API policy,
  and the examples use `fs.existsSync()` and `fs.readFileSync()`.
  The existing `ignores` option covers false positives,
  but it is a workaround rather than the rule's core meaning.
- Would the repo welcome our contribution?
  Yes with care.
  `CONTRIBUTING.md` welcomes contributions and allows AI assistance with disclosure,
  but requires human review and tested submissions.
- Will they likely fix it?
  Soft yes.
  No matching issue or PR was found,
  and no recent `no_sync.rs` path-specific history was present in the shallow source clone.
  There is no explicit maintainer signal against a narrower Node-provenance interpretation.
- Have we prototyped a minimal fix compatible with their architecture?
  Yes.
  The prototype patch is recorded in [oxlint-node-no-sync.patch](oxlint-node-no-sync.patch).

### Prototype

Fresh disposable upstream clone:
`/tmp/agent/oxc-no-sync-prototype-LmG4Snov`.

Origin and commit:

```text
https://github.com/oxc-project/oxc.git
d8c6b550c8802cc68f8e404f279cdc603692b3b6
```

Prototype summary:

- Replaced suffix-only reporting with provenance-aware reporting.
- Continued to report sync APIs reached through Node builtin imports,
  CommonJS `require()`,
  `process.getBuiltinModule()`,
  namespace/default module imports,
  destructured sync API bindings,
  variable aliases,
  dynamic `await import()`,
  and `.apply()`/`.call()` chaining through a sync member.
- Reports aliased named imports and destructuring aliases by the imported Node API name,
  so `ignores: ["readFileSync"]` still works when local code calls `read()`.
- Added pass cases for `parseSync()` imported from `@optique/core/parser`,
  non-Node local `parseSync`,
  type-only imports,
  and local `require`/`process` shadows.
- Updated `crates/oxc_linter/src/snapshots/node_no_sync.snap` for the new fixture catalog.

Patch:
[oxlint-node-no-sync.patch](oxlint-node-no-sync.patch).

Verification commands:

```bash
cargo fmt --manifest-path /tmp/agent/oxc-no-sync-prototype-LmG4Snov/Cargo.toml --package oxc_linter --check
git -C /tmp/agent/oxc-no-sync-prototype-LmG4Snov diff --check
cargo test --manifest-path /tmp/agent/oxc-no-sync-prototype-LmG4Snov/Cargo.toml --package oxc_linter node::no_sync::test
```

`cargo clippy --manifest-path /tmp/agent/oxc-no-sync-prototype-LmG4Snov/Cargo.toml --package oxc_linter --lib --tests`
was also run after increasing local `/tmp` quota.
It exited successfully.
It still prints existing upstream warnings outside the prototype `no_sync.rs` changes,
but no `no_sync.rs` clippy warning remains.

Verification output:

```text
running 1 test
test rules::node::no_sync::test ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1162 filtered out; finished in 0.05s
```

Decision:
fileable after human review and AI disclosure.
We are not filing it from this session;
the repo-local rule below is still the chosen immediate workaround.

## Upstream filing artifact

~~~md
Title: linter: node/no-sync reports non-Node parseSync imports by suffix alone

Labels: A-linter

Oxlint version: 1.71.0

AI assistance disclosure:

This report was drafted with AI assistance.
A human should verify the reproduction,
source trace,
prototype patch,
and workaround before filing.

Command:

```bash
mise run //package/git-policy/cli:lint:oxlint
```

Config:

The repo enables the `node` plugin and category-based linting,
which enables `node/no-sync`.

What happened:

`node/no-sync` reports `parseSync()` calls imported from `@optique/core/parser`:

```ts
import { parseSync, } from '@optique/core/parser';

const parseResult = parseSync(parser, args);
```

Expected:

The rule would report sync APIs from Node builtin modules,
such as `node:fs` `readFileSync`,
but would not report unrelated library APIs that merely end in `Sync`.

Source trace:

`crates/oxc_linter/src/rules/node/no_sync.rs:100-120` checks identifiers and member properties with
`.ends_with("Sync")`.
It does not inspect import declarations or module provenance before reporting.

Prototype:

A working prototype is recorded in `doc/troubleshooting/oxlint-node-no-sync.patch`.
It resolves sync-looking calls through Node builtin imports,
CommonJS `require()`,
`process.getBuiltinModule()`,
variable aliases,
destructuring aliases,
dynamic `await import()`,
and `.apply()`/`.call()` before reporting.
The targeted upstream test passes with:

```bash
cargo test \
  --manifest-path /tmp/agent/oxc-no-sync-prototype-LmG4Snov/Cargo.toml \
  --package oxc_linter \
  node::no_sync::test
```

Suggested fix:

Use the prototype shape in `crates/oxc_linter/src/rules/node/no_sync.rs`:
collect or resolve bindings from Node builtin modules and report sync-looking calls only when the callee resolves to
those bindings.
~~~
