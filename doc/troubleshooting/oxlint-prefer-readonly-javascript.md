# Oxlint inserts TypeScript readonly syntax into JavaScript private fields

## Symptom and scope

Oxlint `1.88.0` with `oxlint-tsgolint` `7.0.2004`
applies `typescript(prefer-readonly)` to an auto-mode `.mjs` file during `--fix`.
The resulting source contains `readonly #state;`.
The same formatter's plain-lint oracle then reports:

```text
# package/pi-plugin/auto-mode/native/contract/integration/policy-source/system-slot.mjs
TS(8009): The 'readonly' modifier can only be used in TypeScript files.
typescript(TS8009): The 'readonly' modifier can only be used in TypeScript files.
```

The reproduction uses a private frozen package snapshot at `57b2baacd2`.
It does not modify the working tree or installed linter.
The retained receipt is
`${HOME}/temp/agent/pi-mimo-preparation-failure-20261018/native-format-fixture.json`.
The formatter's final oracle reports 10,086 warnings and 90 errors across 494 native files.
Those are post-format findings,
not the package's complete lint backlog.

## Source identity and cause

The installed helper's source tag is `v7.0.2004`,
commit `a61289117febcb1a60cf3bc75ef2b6bbccc4b665`
in `oxc-project/tsgolint`.
The read-only source checkout is
`${HOME}/temp/agent/tsgolint-readonly-js-20261018`.

At `internal/rules/prefer_readonly/prefer_readonly.go:389`,
`PreferReadonlyRule.Run` installs its class analysis without a JavaScript-file exclusion.
At line 422,
the report builder unconditionally inserts a TypeScript modifier:

```go
// tsgolint/internal/rules/prefer_readonly/prefer_readonly.go:422 at v7.0.2004
fixes := []rule.RuleFix{rule.RuleFixInsertBefore(ctx.SourceFile, nameNode, "readonly ")}
```

The actual input diff is:

```diff
# package/pi-plugin/auto-mode/native/contract/integration/policy-source/system-slot.mjs
 class SystemPolicyInput {
- #state;
+ readonly #state;
```

The qualified compiler already exposes the grammar predicate needed to distinguish these files.
Its `internal/ast/utilities.go:1271` defines `IsSourceFileJS` using JavaScript and JSX script kinds.
The matching shim exports that function at `shim/ast/shim.go:1368`.
This is an invalid formatter result exposed by JavaScript grammar;
it is separate from the owned environment-inheritance test defect.

## Verification catalogs and prototype

The maintained package task reproduces the change inside a disposable 2 GiB,
two-CPU container:

```shell
# doc/troubleshooting/oxlint-prefer-readonly-javascript.md: retained private reproduction
cd "${HOME}/temp/agent/pi-mimo-preparation-failure-20261018"
mise run prepare-native-format 57b2baacd2
mise run private-native-format
```

The private tree's diff and `native-private-safe-format.log` retain the modifier and diagnostic output.
The working catalog must retain TypeScript private-field findings and fixes.
The failing catalog adds JavaScript,
CommonJS,
ESM and JSX cases with read-only-in-practice private fields.
They must produce no TypeScript-only modifier.

A fresh disposable prototype is at `${HOME}/temp/agent/upstream-prototype.XlprBA91`.
Its origin and exact source commit are verified before editing;
its push URL is disabled.
The new filename-specific regression cases are committed locally as `ccec2d9`.
They use the upstream `tsconfig.checkJs.json`,
which explicitly enables JavaScript input checking.
The complete existing `prefer_readonly` rule catalog remains enabled.
The prototype verification uses a private Go toolchain mount,
disposable module cache and home,
2 GiB memory and two CPUs.
Neither the workspace nor the real home or credentials is mounted.
The new cases fail for each JavaScript filename before the remedy.
The local remedy returns no listeners from `Run` when `ast.IsSourceFileJS(ctx.SourceFile)` is true.
The complete `prefer_readonly` package catalog then passes,
including its existing TypeScript findings and fixes.
The retained logs are `tsgolint-readonly-prototype-red.log`
and `tsgolint-readonly-prototype-green.log` in the reproduction directory.

The standalone helper builds successfully with Go `1.27.2`.
An actual maintained `format:oxlint` invocation uses that private binary through `OXLINT_TSGOLINT_PATH`
against a fresh copy of the original `system-slot.mjs`.
It preserves `#state;`,
produces no `readonly #state`,
and reports 60 warnings and zero errors.
Its exit status is still 1 because warnings remain;
this is not a zero-lint claim.
Node `26.12.0` subsequently accepts the resulting JavaScript with `--check`.
The retained logs are `readonly-formatter-integration.log`
and `readonly-formatter-javascript-syntax.log`.

The [local patch](./oxlint-prefer-readonly-javascript.patch) applies cleanly to the qualified upstream commit.
Local prototype commits are `ccec2d9`,
`0b64a8e` and `f615072`.
Go's generated `go.work.sum` changes remain outside that patch.
No installed helper,
workspace dependency or lint configuration is replaced.

The fetched upstream `main` rule is byte-identical to the qualified tag's rule:
SHA-256 `89790716cd54cd4e1b063db9e166ca594f5af398fbf234437465f878492fc2b8`.
This comparison checks that rule file,
not every upstream dependency or build input.

## Current workaround and limits

Keep the malformed private output out of the maintained native tree.
Apply independently reviewed file clusters and run syntax,
type and real consumer checks after formatting.
This preserves the JavaScript consumer while the formatter issue is investigated.
It does not complete the remaining native lint backlog.
No rule is disabled or weakened.

## Upstream filing decision

- Fault: the actual safe fix inserts a modifier forbidden by the input file's grammar.
- Fixability: the compiler already supplies a JavaScript-file predicate.
- Supported use: the upstream rule harness accepts explicit JavaScript filenames and provides
  `tsconfig.checkJs.json` with `allowJs` and `checkJs` enabled.
- Contribution policy: `README.md`,
  `CONTRIBUTING.md` and `.github/SECURITY.md` are read.
  The contribution guide welcomes external work;
  searches of those files find no AI-filing prohibition.
- Maintainer disposition: implementation issue 193 and merged implementation pull request 654 are read in full.
  They establish implementation support,
  not a decision about this JavaScript case.
  Tracker searches for `readonly JavaScript` return no issue or pull request.
  Broader `prefer-readonly` searches find implementation and maintenance work,
  including merged pull request 1149.
- Prototype: all added JavaScript cases fail before the guard and pass after it.
  The complete rule catalog passes and the standalone helper builds.
  The real formatter preserves JavaScript grammar with the private helper,
  and Node accepts the resulting file.

The `.out-of-scope` inventory is checked.
Its `low-impact-typescript-formatting.md` exclusion concerns token polish,
not insertion of syntax that breaks a JavaScript consumer.
No matching exemption is found.
The contribution guide's `CODE_OF_CONDUCT.md` link returns 404 when checked on 2026-10-18.
Its current location is `oxc-project/oxc/.github/CODE_OF_CONDUCT.md`,
which is read in full.
The searched contribution and conduct policies contain no AI-filing prohibition.
This records the checked policies,
not a claim about every possible policy source.

All filing checks have supporting evidence.
The following draft stays local because the current authorization covers this repository,
not publishing to the upstream repository.
No issue or pull request is filed upstream.

## Local upstream issue draft

Proposed title:
`prefer-readonly inserts a TypeScript modifier into JavaScript private fields`.

````markdown
<!-- Proposed oxc-project/tsgolint issue body, not published. -->
## Reproduction

With oxlint 1.88.0 and oxlint-tsgolint 7.0.2004,
enable type-aware `typescript/prefer-readonly` on this `.mjs` file:

```javascript
// readonly-example.mjs
class Example {
  #value = 1;
  get value() {
    return this.#value;
  }
}
```

The rule reports that `#value` is never reassigned and inserts `readonly `.
The resulting `readonly #value = 1;` is invalid JavaScript.
The next lint pass reports TS8009:
`The 'readonly' modifier can only be used in TypeScript files.`

JavaScript input should not receive a TypeScript-only modifier.
The local regression catalog covers `.js`,
`.cjs`,
`.mjs` and `.jsx` filenames with the upstream checkJs configuration.

## Source and verified remedy

At tag v7.0.2004,
`PreferReadonlyRule.Run` has no JavaScript exclusion,
and its fix builder unconditionally inserts `readonly `.
The fetched main rule file is unchanged.

Returning no listeners when `ast.IsSourceFileJS(ctx.SourceFile)` is true fixes these cases.
The complete existing prefer_readonly rule catalog still passes,
including TypeScript cases.
A helper built from that patch also preserves valid JavaScript through the actual oxlint formatter.
No existing rule or test catalog is disabled.
````
