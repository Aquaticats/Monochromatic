# Oxlint node/no-sync handover

> References below this note that no longer resolve in the tree.
> Each entry gives the value that identifier has now:
> `package/git-policy/cli/src/parsers` is now `package/git-policy/cli/src/native/command`
> (2026-10-13 `642b6d72cb`).

> Scratch-path note:
> `/tmp/agent` paths in this document are historical.
> Use `~/temp/agent` for current work.

## Current request

User wants two outcomes:

- A troubleshooting doc focused on oxlint `node/no-sync` flagging Optique `parseSync`.
- A custom `no-restricted-syntax/no-sync` rule that bans only sync APIs for constructs that are inherently async
  but exposed as sync APIs.
  For now this includes only Node sync methods.

User explicitly said not to fix existing code,
so do not change the current `parseSync` callsites in `package/git-policy/cli/src/parsers/`.

## Work completed

- Created `doc/troubleshooting/oxlint-node-no-sync.md`.
- Committed it as `4df2be188` with message `docs(oxlint): document node no-sync suffix false positive`.
- Recorded the upstream prototype patch and verification in `doc/troubleshooting/oxlint-node-no-sync.patch`
  and committed it as `264548a5b`.
  Later refreshed the patch so the upstream prototype handles aliased imports,
  destructuring aliases,
  variable aliases,
  dynamic `await import()`,
  `.apply()`/`.call()`,
  and local `require`/`process` shadows.
- Added project rule `no-restricted-syntax/no-sync`,
   fixtures,
   config wiring,
   and readonly allowlist updates across
  follow-up commits through `6ad8bbac0`.
- Added namespace,
   `.apply()`,
   `.call()`,
   and shadowed-global fixture coverage in `a363eff67` while keeping Set-based
  alias tracking.

## Prototype state

User pointed out that troubleshooting-doc constraints sorta-hold,
so the auto-prototype step had to run.
That is now done.

Fresh disposable upstream clone:

```text
/tmp/agent/oxc-no-sync-prototype-LmG4Snov
```

Origin and commit:

```text
https://github.com/oxc-project/oxc.git
d8c6b550c8802cc68f8e404f279cdc603692b3b6
```

Prototype artifacts:

- `doc/troubleshooting/oxlint-node-no-sync.patch`
- Updated `doc/troubleshooting/oxlint-node-no-sync.md`

Verification commands passed:

```bash
cargo fmt --manifest-path /tmp/agent/oxc-no-sync-prototype-LmG4Snov/Cargo.toml --package oxc_linter --check
git -C /tmp/agent/oxc-no-sync-prototype-LmG4Snov diff --check
cargo test --manifest-path /tmp/agent/oxc-no-sync-prototype-LmG4Snov/Cargo.toml --package oxc_linter node::no_sync::test
```

`cargo clippy --manifest-path /tmp/agent/oxc-no-sync-prototype-LmG4Snov/Cargo.toml --package oxc_linter --lib --tests`
was rerun after increasing local `/tmp` quota and exited successfully.
It still prints existing upstream warnings outside the prototype `no_sync.rs` changes,
but no `no_sync.rs` clippy warning remains.

Prototype doc work is complete.

## Local rule state

Custom rule implementation is present and targeted verification passed.

Changed files:

- `package/oxlint-plugin/no-restricted-syntax/src/rule/no-sync.ts`
- `package/oxlint-plugin/no-restricted-syntax/src/rule/no-sync.constants.ts`
- `package/oxlint-plugin/no-restricted-syntax/src/rule/no-sync.syntax.ts`
- `package/oxlint-plugin/no-restricted-syntax/src/rule/no-sync.provenance.ts`
- `package/oxlint-plugin/no-restricted-syntax/src/rule/no-sync.node-builtin-source.ts`
- `package/oxlint-plugin/no-restricted-syntax/src/rule/no-sync.node-sync-binding.ts`
- `package/oxlint-plugin/no-restricted-syntax/src/rule/no-sync.node-sync-member.ts`
- `package/oxlint-plugin/no-restricted-syntax/src/index.ts`
- `package/oxlint-plugin/no-restricted-syntax/src/oxlint-no-restricted-syntax.unit.test.ts`
- `package/test-fixture/oxlint-no-restricted-syntax/.oxlintrc.fixture.json`
- `package/test-fixture/oxlint-no-restricted-syntax/src/invalid/no-sync.ts`
- `package/test-fixture/oxlint-no-restricted-syntax/src/valid/no-sync.ts`
- `package/config/oxlint/src/rule/restriction.ts`
- `package/config/oxlint/src/overrides.ts`
- `package/config/oxlint/src/rule/prefer-readonly-parameter-types.allow-pkg.ts`
- `package/oxlint-plugin/no-restricted-syntax/README.md`

Policy shape:

- `no-restricted-syntax/no-sync` reports Node builtin sync APIs resolved through imports,
  `require()`,
  `process.getBuiltinModule()`,
  destructuring,
  simple aliases,
  and `.apply()`/`.call()` from a sync member.
- Fixture coverage includes namespace imports,
  named-import aliases,
  member aliases,
  `.apply()`/`.call()`,
  and local `require`/`process` shadows.
- Shared oxlint config disables upstream `node/no-sync` and enables the project rule.
- Existing `parseSync` callsites are untouched.

## Verification completed

Passed:

```bash
mise run //package/oxlint-plugin/no-restricted-syntax:format:oxlint
mise run //package/oxlint-plugin/no-restricted-syntax:lint:oxlint
mise run //package/oxlint-plugin/no-restricted-syntax:lint:types
mise run //package/oxlint-plugin/no-restricted-syntax:build
mise run //package/oxlint-plugin/no-restricted-syntax:test:unit
mise run //package/config/oxlint:lint:oxlint
mise run //package/config/oxlint:lint:types
```

Boundary check:

```bash
mise run //package/git-policy/cli:lint:oxlint
```

This now reports zero warnings and no `parseSync` findings.
It still fails on two unrelated existing `catch-binding` errors in `package/git-policy/cli`.

Dirty unrelated files left for caller awareness:

- `mise.lock` has a CMake `4.3.3` to `4.3.4` update from mise tool resolution.
- `package/oxlint-plugin/no-restricted-syntax/src/rule/no-immediate-mutation.syntax.ts`,
  `package/oxlint-plugin/no-restricted-syntax/src/rule/no-immediate-mutation.ts`,
  and `package/test-fixture/oxlint-no-restricted-syntax/src/valid/no-immediate-mutation.ts`
  are modified by concurrent work and were not touched for this task.
