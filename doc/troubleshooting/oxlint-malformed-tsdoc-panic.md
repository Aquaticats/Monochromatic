# Oxlint 1.90.0 can panic while reporting malformed TypeScript after a TSDoc comment closes too early

## Symptom

During task-runner cleanup,
an edit inserted a block comment inside an existing TSDoc example in
`package/monorepo-task/cli/src/load.ts`.
Its closing delimiter ended the outer comment,
leaving the example and later prose as invalid TypeScript.
TypeScript reported `TS1443: Module declaration names may only use ' or " quoted strings.`
and subsequent syntax errors.

Oxlint's type-aware backend then failed instead of completing its diagnostics:

```text
# Captured from mise run //package/monorepo-task/cli:format:oxlint
panic: runtime error: slice bounds out of range [4138:4134]
github.com/typescript-eslint/tsgolint/internal/rules/no_unsafe_member_access
no_unsafe_member_access.go:125
```

This is separate from the default reporter panic documented in
[oxlint-default-formatter-render-panic.md](oxlint-default-formatter-render-panic.md).
Here the failing component was the Go type-aware rule,
before normal diagnostic rendering completed.

## Source trace and limits

The inspected tsgolint checkout is `a61289117febcb1a60cf3bc75ef2b6bbccc4b665`.
Its [unsafe-member diagnostic construction][source] reads the property span,
trims that span,
then slices the source text without checking the resulting bounds:

```go
// internal/rules/no_unsafe_member_access/no_unsafe_member_access.go:122
if ast.IsPropertyAccessExpression(node) {
    property = node.Name()
    loc := utils.TrimNodeTextRange(ctx.SourceFile, property)
    propertyName = "." + ctx.SourceFile.Text()[loc.Pos():loc.End()]
}
```

The captured panic reports a starting offset greater than its ending offset.
The stack identifies this slicing operation.
The malformed-input source was not reduced far enough to establish which parser or trimming operation created that range.
The inspected checkout is source evidence for the reported call site,
not a claim that it is the exact embedded backend revision of the installed Oxlint binary.

## Verification and workaround

The failing repository source was committed at `bc8cbc4f49`.
The nested comment was removed in `6eff3c81fa`.
The package then type-checked,
and the final scoped lint run completed with `Found 0 warnings and 0 errors`.

```sh
# Repository root, with the package dependencies installed
mise run //package/monorepo-task/cli:lint:types
mise run //package/monorepo-task/cli:lint:oxlint
```

Failing catalog:
the nested block-comment delimiter in the TSDoc example,
TypeScript syntax diagnostics,
and the type-aware backend's invalid slice.
Working catalog:
the repaired comment,
successful type checking,
and completed lint diagnostics on the same package.

Repair malformed source first and re-run both checks.
Do not count the panic as a completed lint run,
and do not disable type-aware rules to hide it.
The source fix addresses the repository input;
it does not claim to repair upstream handling of malformed syntax.

## Upstream filing decision

Searching `.out-of-scope/` for Oxlint,
tsgolint,
and malformed-source exclusions found no matching entry.
The upstream search for `no_unsafe_member_access` found
[tsgolint issue 527][issue],
whose title already names this panic family.
Its relationship to this exact malformed TSDoc input remains unverified.

Upstream diagnostic robustness is implicated,
but the exact range producer and a minimal reproducer remain unestablished.
Fixability,
contribution acceptance,
and likely maintainer action were not assessed for this unisolated instance.
No upstream patch or new report is claimed,
and no duplicate issue was filed.
The existing report and the repaired local input are the current evidence.

The source selection was opened in `agent-browser`
and matched against the rendered slice expression.

[source]: https://github.com/oxc-project/tsgolint/blob/a61289117febcb1a60cf3bc75ef2b6bbccc4b665/internal/rules/no_unsafe_member_access/no_unsafe_member_access.go#L119-L130
[issue]: https://github.com/oxc-project/tsgolint/issues/527
