# TypeScript 7.0.2's type serializer panics on an instantiated generic tuple
(`interface conversion: checker.TypeData is *checker.TypeReference, not *checker.TupleType`)
while summarizing `Parameters<Fn>`, and 7.1.0-beta no longer builds the panicking shape

## Symptom

Analyzing a call whose argument type instantiates a generic parameter's tuple type,
for example the fixture shape

```ts
// package/oxlint-plugin/prefer-readonly-parameter-type/src/effect-summaries.unit.test.ts
export function take<Fn extends (...args: never[]) => unknown,>(
  fn: Fn,
  args: Parameters<Fn>,
): void {
  void fn;
  void args;
}
```

crashed the analysis process on `typescript@7.0.2` with a Go interface-conversion panic whose
stable middle reads verbatim:

```text
checker.TypeData is *checker.TypeReference, not *checker.TupleType
```

The crash appeared whenever the compiler serialized the instantiated tuple type back to
JavaScript,
so it surfaced during effect-summary construction and killed the whole process,
not one call.
Any tool asking the 7.0.2 API for that type's data hit the same wall.

## Root cause

### The serializer asserts tuple data on every tuple-flagged reference

At tag `typescript/v7.0.2` of `microsoft/typescript-go`
(commit `2bd066d87f5bafd315be9f40889d0a60b9e58e0b`),
the API's type serialization takes the tuple branch for any type whose object flags carry
both `ObjectFlagsReference` and `ObjectFlagsTuple`:

```go
// internal/api/proto.go:671 (typescript/v7.0.2)
		if objectFlags&checker.ObjectFlagsReference != 0 {
			var ref *checker.TypeReference
			if objectFlags&checker.ObjectFlagsTuple != 0 {
				tuple := t.AsTupleType()
				ref = tuple.AsTypeReference()
```

### `AsTupleType` is an unchecked interface assertion

```go
// internal/checker/types.go:693 (typescript/v7.0.2)
func (t *Type) AsTupleType() *TupleType                   { return t.data.(*TupleType) }
```

For an instantiated generic tuple such as `Parameters<Fn>` the type's data is a `*TypeReference`,
not a `*TupleType`.
The assertion at `types.go:693` therefore panics,
and Go's runtime assembles the message from reflection,
naming the field,
the actual dynamic type,
and the asserted type.
That is why the exact panic text appears in no source file:
only the format lives in the runtime,
while `checker.TypeData`,
`*checker.TypeReference`,
and `*checker.TupleType` are reflect names.
The plugin pinned the stable middle as
`TYPESCRIPT_TUPLE_PANIC_FRAGMENT`
before commit `62ca73197b` retired the category,
and that historical string is the verbatim quote above.

### TypeScript 7.1.0-beta stops building the panicking shape

The assertion survives in current source
(`internal/api/proto.go:885` on `microsoft/typescript-go` `main`,
checked 2026-10-20),
still reading `tuple := t.AsTupleType()`.
What changed is the type construction behind it:
the same fixture shapes no longer reach the serializer as tuple-flagged
`*TypeReference` data,
so no panic occurs.
Measured,
not inferred:
a probe battery of 16 tuple-carrying shapes,
including the fixture above,
built 22 effect summaries with zero omissions on `7.1.0-beta`
(`${HOME}/temp/agent/omission-probe-2026-10-20.ts` during the move),
and the suite that crashed at 7.0.2 runs green on 7.1.0-beta.

An earlier reading of this investigation treated
`rg -c 'checker\.TypeData is' <tsc binary>`
as evidence of the fix.
It is not:
Go assembles the message from reflect names at panic time,
so the literal fragment is absent from every binary,
including 7.0.2's.
The run-level absence is the proof.

## Verification

Versions under test:
`typescript@7.0.2` with `@typescript/typescript-linux-x64@7.0.2`
(panic captured during the 2026-10-20 move),
and `typescript@7.1.0-beta` with `@typescript/typescript-linux-x64@7.1.0-beta`
(npm `beta` dist-tag,
published 2026-10-20T16:45:47.402Z).
Compiler source under `microsoft/typescript-go`,
tag `typescript/v7.0.2` = `2bd066d87f5bafd315be9f40889d0a60b9e58e0b`.

Runnable harness on the pinned workspace:

```sh
# /var/home/user/Monochromatic
mise run //package/oxlint-plugin/prefer-readonly-parameter-type:test:unit
```

The suite exercises the fixture through the built plugin,
including the consumer fixture that stages its own install of the pinned compiler.

Failing catalog,
scoped to `typescript@7.0.2`:

- a call of `take<Fn>(fn, args: Parameters<Fn>)` crashed summary serialization with the panic
  quoted in `Symptom`;
- the 16-shape probe battery could not complete,
  which is why the plugin recorded the affected callables as deliberate omissions instead of
  summaries.

Working catalog,
`typescript@7.1.0-beta`:

- the same fixture summarizes completely;
  the probe battery built 22 summaries with zero omissions;
- `package/oxlint-plugin/prefer-readonly-parameter-type/src/effect-summaries.unit.test.ts`'
s tuple fixture persists `omittedCallableKeys: []` and `omissionReasons: []`,
  asserted as the cold-scan baseline in
  `restores deliberate callable omissions from persistent cache`.

## Verified workarounds

### Move to `typescript@7.1.0-beta` (applied)

The catalog pin `'typescript': '7.1.0-beta'` in `pnpm-workspace.yaml` carries the fix.
Tradeoff:
the workspace tracks a prerelease compiler,
and the workspace's 24-hour `minimumReleaseAge` hold needed scoped
`minimumReleaseAgeExclude` entries for the day-one beta
(kept until `7.1.0` stable by decision on 2026-10-20).

### Record the affected callables as deliberate omissions (historical, 7.0.2 only)

Before the move,
the plugin caught the panic during summary construction and persisted the affected callables as
`typescript-tuple-serialization-failed` omissions,
withholding their effects fail-closed.
Tradeoff:
offers depending on those callables were withheld rather than computed,
and the panic classification itself was only a label on an upstream defect.
The category was retired in commit `62ca73197b` once nothing could emit it;
the generic `direct-summary-construction-failed` recorder remains for any caught failure.

## What does not work

- Catching the panic in consumer code and retrying:
  the process is already dead,
  and the assertion runs inside the compiler's own serializer,
  so no consumer-side call path can route around it.
- Asking the reference's target instead:
  consumer code never reaches `AsTupleType`;
  the failing assertion is on the serialization path between the checker and the API boundary.
- Asserting fix status from the binary's strings:
  the panic text is reflect-generated at runtime,
  so string absence proves nothing either way.

## Upstream filing decision

`.out-of-scope/` holds no typescript-go or compiler-serialization exemption
(checked `bun-install.md`,
`cargo-workspace.md`,
`claude-code-upstream-bugs.md`,
`codex-harness.md`,
`jsr.md`,
`lightningcss.md`,
`low-impact-typescript-formatting.md`,
`module-es-monolith.md`,
`pi-gpt55-long-context.md`,
`terminal-title-fork-parity-tests.md`,
and `typescript-project-references.md`).
Nothing to file and nothing to add:
the panicking shape is gone in `7.1.0-beta`,
so this incident is closed by the upgrade rather than by a report.
No draft issue or comment is kept.

1.  Really upstream's fault?
    Yes.
    The unchecked `t.data.(*TupleType)` assertion at
    `internal/checker/types.go:693` (typescript/v7.0.2) is upstream code.
2.  Can upstream fix it?
    They did.
    The same inputs serialize without panic on 7.1.0-beta,
    while the assertion at `internal/api/proto.go:885` remains for genuinely tuple-backed types.
3.  Are they supporting this use case?
    Yes.
    The API's own type serialization carried the type,
    and the 7.1 API documentation presents type data crossing the boundary as supported.
4.  Would the repo welcome our contribution?
    Not evaluated,
    because the incident is fixed upstream and opening an external issue needs the user's
    authorization.
5.  Will they likely fix it?
    Moot;
    already fixed before this report.
6.  Have we prototyped a minimal fix compatible with their architecture?
    Not applicable;
    the upgrade is the verified remedy and no upstream change is proposed.
